//! 帧率采集插件（基线口径的执行侧）。
//!
//! 口径（与 docs/fps-baseline.md 同文）：
//! - warmup 丢弃 [`WARMUP_SECS_DEFAULT`] 秒（窗口创建/首帧编译/缓存预热）；
//! - 平均帧率 = warmup 后帧数 / warmup 后时长；
//! - `--bench-secs S`（S > 0）时运行 S 秒后打印 `[BENCH]` 汇总行并以
//!   `AppExit::Success` 退出（自动采集）；S = 0 常驻，仅每秒打印 `[STATS]`。

use bevy::prelude::*;

/// warmup 丢弃时长（秒）——基线口径常量。
pub const WARMUP_SECS_DEFAULT: f64 = 2.0;

#[derive(Resource, Debug)]
struct BenchState {
    warmup_secs: f64,
    /// 自启动累计的真实时长（含 warmup）。
    wall_secs: f64,
    /// 1 秒窗口起始以来的帧数。
    window_frames: u64,
    window_secs: f64,
    /// warmup 后累计帧数 / 时长。
    measure_frames: u64,
    measure_secs: f64,
    finished: bool,
}

pub struct BenchPlugin {
    pub warmup_secs: f64,
}

impl Default for BenchPlugin {
    fn default() -> Self {
        Self {
            warmup_secs: WARMUP_SECS_DEFAULT,
        }
    }
}

impl Plugin for BenchPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BenchState {
            warmup_secs: self.warmup_secs,
            wall_secs: 0.0,
            window_frames: 0,
            window_secs: 0.0,
            measure_frames: 0,
            measure_secs: 0.0,
            finished: false,
        })
        .add_systems(Update, bench_sample);
    }
}

/// 依据: AppExit 事件写入形态 `MessageWriter<AppExit>`——0.19 官方示例
/// `bevy-0.19.1/examples/app/custom_loop.rs:36-38`（`EventWriter` 已并入
/// Message 系列命名，bevy_ecs-0.19.1/src/lib.rs:80 prelude 导出；核实 2026-09-26）。
fn bench_sample(
    time: Res<Time>,
    config: Res<SimConfigImport>,
    mut state: ResMut<BenchState>,
    mut stats: ResMut<SimStatsImport>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.finished {
        return;
    }
    let dt = time.delta_secs_f64(); // 依据: bevy_time-0.19.1/src/time.rs:290（核实 2026-09-26）
    state.wall_secs += dt;
    state.window_secs += dt;
    state.window_frames += 1;
    stats.frame_count += 1;

    // 每 ~1 秒打印一次瞬时统计（窗口闭合即打印，跨秒误差 ≤ 1 帧）。
    if state.window_secs >= 1.0 {
        stats.fps_1s = state.window_frames as f64 / state.window_secs;
        info!(
            "[STATS] t={:.1}s tick={} fps_1s={:.1} avg_fps={:.1}",
            state.wall_secs, stats.tick, stats.fps_1s, stats.avg_fps
        );
        state.window_frames = 0;
        state.window_secs = 0.0;
    }

    if state.wall_secs >= state.warmup_secs {
        state.measure_frames += 1;
        state.measure_secs += dt;
    }
    // 稳态平均（实时刷新；常驻模式下也可 BRP 读到）。
    if state.measure_secs > 0.0 {
        stats.avg_fps = state.measure_frames as f64 / state.measure_secs;
    }

    if config.bench_secs > 0.0 && state.wall_secs >= config.bench_secs as f64 {
        state.finished = true;
        info!(
            "[BENCH] n={} seed={} bench_secs={:.1} warmup_secs={:.1} measure_secs={:.3} measure_frames={} avg_fps={:.1}",
            config.entity_count,
            config.seed,
            config.bench_secs,
            state.warmup_secs,
            state.measure_secs,
            state.measure_frames,
            stats.avg_fps
        );
        exit.write(AppExit::Success);
    }
}

// 类型别名：避免 bench 模块对 sim::SimConfig / sim::SimStats 的全限定引用噪音。
use crate::sim::{SimConfig as SimConfigImport, SimStats as SimStatsImport};

#[cfg(test)]
mod tests {
    #[test]
    fn warmup_constant_is_calibrated() {
        // 基线口径常量：warmup 2.0 秒（docs/fps-baseline.md 同文）。
        assert_eq!(super::WARMUP_SECS_DEFAULT, 2.0);
    }
}

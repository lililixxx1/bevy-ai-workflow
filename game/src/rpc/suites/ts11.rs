//! TS-11《阶梯性能回归验证》的游戏进程内断言（两阶段判定第二阶段）。
//!
//! 与 `assets-methodology/taskset/ts-11-perf-ladder.md` 验收清单的对应：
//! - #1（`avg_fps > 0` 且 `fps_1s > 0`，采集链路活）→ [`bench_stats_alive`]；
//! - #2（`tick > 0`，50000 实体下模拟仍每帧推进）→ [`sim_ticking_at_scale`]
//!   （附带核对实体数 == 启动值——阶梯口径的配置自证；tick 跨调用递增由
//!   task-runner `run` 模式的两连调比对 `snapshot.tick` 完成）；
//! - #3（`[BENCH]` 汇总行：n=50000、avg_fps 与 BRP 读值 ±10%、warmup=2.0）→
//!   工具端 `bench-judge` 子命令（日志文件在进程外，退出后解析判定；BRP 读值
//!   由本套件 detail 携带 avg_fps 实测值供其引用）；
//! - #4（基线对照 avg_fps ≥ 基线 × 0.8）→ 工具端 `bench-judge --baseline-fps`。
//!
//! 口径注：`avg_fps` 在 warmup（2s）后才开始累计——驱动序列须在启动 ≥3s 后
//! 首次调用本套件（bench.rs：`measure_secs > 0` 才刷新 avg_fps）。

use bevy::prelude::*;

use super::{tc, TestCase};
use crate::sim::{SimConfig, SimStats, Wanderer};

/// TS-11 套件入口（约定只读世界）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    vec![bench_stats_alive(world), sim_ticking_at_scale(world)]
}

/// 清单 #1：双帧率指标非零（采集链路活体）。
fn bench_stats_alive(world: &mut World) -> TestCase {
    let Some(stats) = world.get_resource::<SimStats>() else {
        return tc("bench_stats_alive", false, "缺 SimStats 资源".into());
    };
    tc(
        "bench_stats_alive",
        stats.avg_fps > 0.0 && stats.fps_1s > 0.0,
        format!(
            "avg_fps={:.3} > 0 且 fps_1s={:.3} > 0（warmup 后实测值，供 bench-judge 双源对照引用）",
            stats.avg_fps, stats.fps_1s
        ),
    )
}

/// 清单 #2：模拟在大体量实体下仍每帧推进（tick > 0）+ 实体数 == 启动口径。
fn sim_ticking_at_scale(world: &mut World) -> TestCase {
    let (tick, expect_count) = {
        let (Some(stats), Some(config)) = (
            world.get_resource::<SimStats>(),
            world.get_resource::<SimConfig>(),
        ) else {
            return tc("sim_ticking_at_scale", false, "缺 SimStats 或 SimConfig 资源".into());
        };
        (stats.tick, config.entity_count as usize)
    };
    let count = {
        let mut q = world.query::<&Wanderer>();
        q.iter(world).count()
    };
    tc(
        "sim_ticking_at_scale",
        tick > 0 && count == expect_count,
        format!(
            "tick={tick} > 0；实体 {count} == entity_count {expect_count}（跨调用 tick 递增由工具端比对 snapshot.tick）"
        ),
    )
}

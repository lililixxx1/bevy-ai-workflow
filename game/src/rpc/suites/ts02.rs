//! TS-02《速度上限配置化》的游戏进程内断言（两阶段判定第二阶段）。
//!
//! 与 `assets-methodology/taskset/ts-02-speed-cap.md` 验收清单的对应：
//! - #1（`max_speed == 9.0` 字面）→ 工具端 `--expect-max-speed 9.0` 字面值绑定
//!   （`SimConfig.max_speed` 经 `world.get_resources` 核对）；
//! - #2 → [`velocity_within_configured_cap`]（全量扫描强于清单的「采样 ≥100」；
//!   「至少 1 个 > 3.0」在启动口径 max_speed=9.0 下成立，条件化断言）；
//! - #3 → [`origin_invariant_to_max_speed`]（清单为跨进程对照口径——与
//!   `--max-speed 3.0` 默认上限运行比对 `origin`；进程内等价物：用
//!   [`DEFAULT_MAX_SPEED`] 重抽序列比对当前世界全部实体，抽取顺序口径
//!   [`sim::draw_initial`] 单一落点保证 origin 不消费 max_speed，逐位一致）。

use bevy::prelude::*;

use super::{tc, TestCase};
use crate::rng::SplitMix64;
use crate::sim::{self, Velocity, Wanderer, DEFAULT_MAX_SPEED};

/// TS-02 套件入口（约定只读世界）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    vec![
        velocity_within_configured_cap(world),
        origin_invariant_to_max_speed(world),
    ]
}

/// 清单 #2：全部 Wanderer 速度模长 ∈ (0, max_speed]；max_speed 显著高于默认值时
/// 须存在模长 > 默认上限的实体（新上限真实生效，而非全部贴默认分布）。
fn velocity_within_configured_cap(world: &mut World) -> TestCase {
    let Some(config) = world.get_resource::<crate::sim::SimConfig>() else {
        return tc("velocity_within_configured_cap", false, "缺 SimConfig 资源".into());
    };
    let cap = config.max_speed;
    let mut total = 0u32;
    let mut violations = 0u32;
    let mut above_default = 0u32;
    let mut max_seen: f32 = 0.0;
    let mut q = world.query::<(&Wanderer, &Velocity)>();
    for (_, v) in q.iter(world) {
        total += 1;
        let len = v.linear.length();
        max_seen = max_seen.max(len);
        if !(len > 0.0 && len <= cap) {
            violations += 1;
        }
        if len > DEFAULT_MAX_SPEED {
            above_default += 1;
        }
    }
    // 「至少 1 个 > 3.0」仅在启动上限显著超过默认（>4.0）时才可断（默认口径下
    // 理论上也可能出现极小概率的 >3.0 抽取，但清单语义是「9.0 启动下必有」）。
    let above_ok = cap > 4.0 && above_default >= 1 || cap <= 4.0;
    tc(
        "velocity_within_configured_cap",
        total > 0 && violations == 0 && above_ok,
        format!(
            "{total} 个实体模长全部 ∈ (0, {cap}]（越界 {violations} 个，最大实测 {max_seen:.4}）；>默认上限 {DEFAULT_MAX_SPEED} 的 {above_default} 个（cap>4 时须 ≥1）"
        ),
    )
}

/// 清单 #3：当前世界全部实体 origin/phase 与默认上限下的重抽序列逐位一致
/// （max_speed 只缩放速度抽取最后一步——抽取顺序未破坏）。
fn origin_invariant_to_max_speed(world: &mut World) -> TestCase {
    let Some(config) = world.get_resource::<crate::sim::SimConfig>() else {
        return tc("origin_invariant_to_max_speed", false, "缺 SimConfig 资源".into());
    };
    let (seed, count, cap) = (config.seed, config.entity_count, config.max_speed);
    // 同一 rng 序列两遍：第一遍按当前 cap（复现世界初值），第二遍按默认上限。
    let mut redraw_default = std::collections::HashMap::with_capacity(count as usize);
    let mut rng = SplitMix64::new(seed);
    for index in 0..count {
        let (origin, _, phase) = sim::draw_initial(&mut rng, seed, index, DEFAULT_MAX_SPEED);
        redraw_default.insert(index, (origin, phase));
    }
    let mut mismatches = 0u32;
    let mut total = 0u32;
    let mut q = world.query::<&Wanderer>();
    for w in q.iter(world) {
        total += 1;
        let ok = redraw_default
            .get(&w.index)
            .is_some_and(|(o, p)| *o == w.origin && *p == w.phase);
        if !ok {
            mismatches += 1;
        }
    }
    tc(
        "origin_invariant_to_max_speed",
        mismatches == 0 && total == count,
        format!(
            "{total} 个实体 origin/phase 与默认上限 {DEFAULT_MAX_SPEED} 重抽序列逐位一致（不符 {mismatches} 个；当前 cap={cap}，跨进程对照为测量口径，进程内等价替换）"
        ),
    )
}

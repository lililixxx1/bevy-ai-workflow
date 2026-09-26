//! TS-01《实体数量参数化与计数核对》的游戏进程内断言——任务测试集两阶段判定
//! 的第二阶段（脚本判定）首个载体（M2 验收③）。
//!
//! 与任务文件 `assets-methodology/taskset/ts-01-entity-count.md` 验收清单的对应：
//! - #1 → [`wanderer_count_matches_config`]；
//! - #2 → [`config_metadata_consistent`]（entity_count / seed 双侧核对 + 未暂停）；
//! - #3 → [`sim_ticking`]（tick > 0）+ 工具端二次调用比对 `snapshot.tick`
//!   （两次采样递增的完整断言由 task-runner 驱动）；
//! - #4 → [`wanderers_have_velocity`]（行内双组件全量成立）；清单括注的
//!   「同 seed 下 linear 跨进程一致」是**测量口径**（需对照另一次进程的证据），
//!   进程内可断言的等价物是 [`initial_state_matches_seed_redraw`]：当前世界
//!   全部实体的初值与 seed + max_speed 的重抽序列逐位一致（进程内确定性）。

use bevy::prelude::*;

use super::{tc, TestCase};
use crate::rng::SplitMix64;
use crate::sim::{self, SimConfig, SimMetadata, SimStats, Velocity, Wanderer};

/// TS-01 套件入口（约定只读世界）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    vec![
        wanderer_count_matches_config(world),
        config_metadata_consistent(world),
        sim_ticking(world),
        wanderers_have_velocity(world),
        initial_state_matches_seed_redraw(world),
    ]
}

/// 清单 #1：BRP 可见实体数 == 启动注入值。
fn wanderer_count_matches_config(world: &mut World) -> TestCase {
    let Some(config) = world.get_resource::<SimConfig>() else {
        return tc("wanderer_count_matches_config", false, "缺 SimConfig 资源".into());
    };
    let expect = config.entity_count as usize;
    let mut q = world.query::<&Wanderer>();
    let count = q.iter(world).count();
    tc(
        "wanderer_count_matches_config",
        count == expect,
        format!("query 计数 {count} == SimConfig.entity_count {expect}"),
    )
}

/// 清单 #2：启动配置与 Startup 快照一致（entity_count / seed），且未暂停。
fn config_metadata_consistent(world: &mut World) -> TestCase {
    let (Some(config), Some(meta)) = (
        world.get_resource::<SimConfig>(),
        world.get_resource::<SimMetadata>(),
    ) else {
        return tc("config_metadata_consistent", false, "缺 SimConfig 或 SimMetadata 资源".into());
    };
    let pass = config.entity_count == meta.entity_count
        && config.seed == meta.seed
        && !config.paused;
    tc(
        "config_metadata_consistent",
        pass,
        format!(
            "config(entity_count={}, seed={}, paused={}) ↔ metadata(entity_count={}, seed={})，paused 须为 false",
            config.entity_count, config.seed, config.paused, meta.entity_count, meta.seed
        ),
    )
}

/// 清单 #3 前半：模拟已每帧推进（tick > 0）；后半（tick2 > tick1）由工具端
/// 间隔二次调用比对 `snapshot.tick`。
fn sim_ticking(world: &mut World) -> TestCase {
    let Some(stats) = world.get_resource::<SimStats>() else {
        return tc("sim_ticking", false, "缺 SimStats 资源".into());
    };
    tc(
        "sim_ticking",
        stats.tick > 0,
        format!(
            "tick={} > 0（elapsed_secs={:.3}；第二次采样递增由工具端比对 snapshot.tick）",
            stats.tick, stats.elapsed_secs
        ),
    )
}

/// 清单 #4 前半：Wanderer 实体行内同时持有 Velocity（BRP 行内双组件口径的
/// 进程内等价物）。
fn wanderers_have_velocity(world: &mut World) -> TestCase {
    let wanderers = {
        let mut q = world.query::<&Wanderer>();
        q.iter(world).count()
    };
    let both = {
        let mut q = world.query::<(&Wanderer, &Velocity)>();
        q.iter(world).count()
    };
    tc(
        "wanderers_have_velocity",
        wanderers > 0 && wanderers == both,
        format!("Wanderer 计数 {wanderers}，Wanderer+Velocity 计数 {both}（须相等且 > 0）"),
    )
}

/// 清单 #4 括注的进程内确定性：全部实体初值（origin / linear / phase）与
/// seed + max_speed 下的重抽序列逐位一致（复用 [`sim::draw_initial`] 单一落点，
/// 抽取顺序口径不变）。
fn initial_state_matches_seed_redraw(world: &mut World) -> TestCase {
    let Some(config) = world.get_resource::<SimConfig>() else {
        return tc("initial_state_matches_seed_redraw", false, "缺 SimConfig 资源".into());
    };
    // 拷出所需字段：后续 query 需要可变借用，不与资源的不可变借用共存。
    let (seed, count, max_speed) = (config.seed, config.entity_count, config.max_speed);
    let mut rng = SplitMix64::new(seed);
    let mut redraw = std::collections::HashMap::with_capacity(count as usize);
    for index in 0..count {
        let (origin, linear, phase) = sim::draw_initial(&mut rng, seed, index, max_speed);
        redraw.insert(index, (origin, linear, phase));
    }
    let mut mismatches = 0u32;
    let mut total = 0u32;
    let mut q = world.query::<(&Wanderer, &Velocity)>();
    for (w, v) in q.iter(world) {
        total += 1;
        let ok = redraw
            .get(&w.index)
            .is_some_and(|(o, l, p)| *o == w.origin && *l == v.linear && *p == w.phase);
        if !ok {
            mismatches += 1;
        }
    }
    tc(
        "initial_state_matches_seed_redraw",
        mismatches == 0 && total == count,
        format!(
            "{total} 个实体初值与 seed={seed} + max_speed={max_speed} 重抽序列逐位一致（不符 {mismatches} 个；跨进程一致为测量口径，不在进程内断言）"
        ),
    )
}

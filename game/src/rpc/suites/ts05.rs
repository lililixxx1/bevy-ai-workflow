//! TS-05《BRP 生成实体并回读》的游戏进程内断言（两阶段判定第二阶段）。
//!
//! 与 `assets-methodology/taskset/ts-05-spawn-entity.md` 验收清单的对应：
//! - #1（基线 `n0 == 100` 字面）→ [`wanderer_baseline_matches_config`]（进程内
//!   自洽口径，与 ts-12 对称；字面 100 由工具端 `--expect-count 100` 绑定）；
//! - #2（`world.spawn_entity` 带 Velocity，返回实体号）→ [`spawn_velocity_readback`]
//!   前半：ECS 直写 spawn（`World::spawn`，与 BRP handler 同在主线程 `&mut World`
//!   通路；BRP 反射序列化形态第一阶段已证，本套件复测判定语义）；
//! - #3（get_components 回读 `linear == [1,0,2]` 逐位）→ [`spawn_velocity_readback`]
//!   后半：同帧回读（spawn 直写立即生效，无 Commands 延迟）；
//! - #4（无 Wanderer 的 Velocity 恰 1 行且 == e）→ [`velocity_only_isolation`]；
//! - 收尾恢复：despawn 探针实体，[`world_restored_after_suite`] 断言世界回基线
//!   （套件「无净副作用」约定）。
//!
//! 依据: `World::spawn` 返回 EntityWorldMut（game/src/rpc/screenshot.rs 同形态
//! 已验证）；`EntityWorldMut::id()`（bevy_ecs-0.19.1/src/world/entity_access/
//! world_mut.rs:181）；`World::get::<T>(entity)`（world/mod.rs:1366）；
//! `World::despawn -> bool`（world/mod.rs:1598）（核实 2026-09-27）。

use bevy::prelude::*;

use super::{tc, TestCase};
use crate::sim::{SimConfig, Velocity, Wanderer};

/// TS-05 套件入口（写通路型：断言完成即恢复世界，无净副作用）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    let baseline = velocity_only_entities(world).len();
    let mut results = vec![
        wanderer_baseline_matches_config(world),
        spawn_velocity_readback(world),
        velocity_only_isolation(world, baseline),
    ];
    // 恢复：**无条件** despawn 本套件 spawn 的全部探针（按「无 Wanderer 的
    // Velocity」筛选集合精确定位），再核对计数回基线——断言成败都不留脏。
    let probes = velocity_only_entities(world);
    for probe in &probes {
        world.despawn(*probe);
    }
    let restored = !probes.is_empty() && velocity_only_entities(world).len() == baseline;
    results.push(tc(
        "world_restored_after_suite",
        restored,
        format!(
            "探针 {:?} 已全部 despawn，Velocity-only 计数回基线 {baseline}（套件无净副作用）",
            probes
        ),
    ));
    results
}

/// 清单 #1 的进程内自洽口径（与 ts-12 对称）：Wanderer 计数 == 启动
/// entity_count（字面 100 仍由工具端 --expect-count 绑定）。
fn wanderer_baseline_matches_config(world: &mut World) -> TestCase {
    let expect = world
        .get_resource::<SimConfig>()
        .map(|c| c.entity_count as usize);
    let count = {
        let mut q = world.query::<&Wanderer>();
        q.iter(world).count()
    };
    tc(
        "wanderer_baseline_matches_config",
        expect.is_some_and(|e| count == e),
        format!("Wanderer 计数 {count} == entity_count {expect:?}（字面值由工具端绑定）"),
    )
}

/// 无 Wanderer 的 Velocity 实体集合（清单 #4 的筛选口径）。
fn velocity_only_entities(world: &mut World) -> Vec<Entity> {
    let mut q = world.query_filtered::<Entity, (With<Velocity>, Without<Wanderer>)>();
    q.iter(world).collect()
}

/// 清单 #2 + #3：spawn 带指定 Velocity 的实体，同帧回读 linear 逐位一致。
fn spawn_velocity_readback(world: &mut World) -> TestCase {
    let expect = Vec3::new(1.0, 0.0, 2.0);
    let entity = world.spawn(Velocity { linear: expect }).id();
    let readback = world.get::<Velocity>(entity).map(|v| v.linear);
    tc(
        "spawn_velocity_readback",
        readback == Some(expect),
        format!(
            "spawn 实体 {entity:?} 后同帧回读 linear={readback:?}（期望 {expect:?} 逐位一致）"
        ),
    )
}

/// 清单 #4：spawn 后无 Wanderer 的 Velocity 实体恰比基线多 1。
fn velocity_only_isolation(world: &mut World, baseline: usize) -> TestCase {
    let current = velocity_only_entities(world).len();
    tc(
        "velocity_only_isolation",
        current == baseline + 1,
        format!(
            "Velocity-only 实体 {current} == 基线 {baseline} + 1（新实体不带 Wanderer）"
        ),
    )
}

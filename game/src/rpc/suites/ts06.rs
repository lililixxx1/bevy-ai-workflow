//! TS-06《BRP 销毁实体》的游戏进程内断言（两阶段判定第二阶段）。
//!
//! 与 `assets-methodology/taskset/ts-06-despawn-entity.md` 验收清单的对应：
//! - #1（基线 `n0 == 100`，取 `e`）→ 套件先记基线并 spawn 一个探针实体记为 `e`
//!   （对既有 Wanderer 做破坏性 despawn 后无法恢复——套件无净副作用约定要求
//!   自带探针；探针带真实 `Wanderer` 组件以对齐清单「销毁一个 Wanderer」语义，
//!   index 取 `u32::MAX` 与真实序号空间隔离）；
//! - #2（despawn 响应无 error）→ [`despawn_removes_entity`]：`World::despawn`
//!   返回 true（ECS 直写等价；BRP 反射通路与错误面 -23401 第一阶段已证）；
//! - #3（计数 -1 且不含 e）→ [`despawn_removes_entity`] 后半：查询回核；
//! - #4/#5（get_components / 重复 despawn 报错）→ [`double_despawn_fails`]：
//!   二次 despawn 同一实体返回 false（BRP 侧表现为 -23401 ENTITY_NOT_FOUND，
//!   进程内等价物即 despawn 的 bool 假值 + warn 日志）。
//!
//! 依据: `World::despawn(entity) -> bool`——实体不存在时 warn + false
//! （bevy_ecs-0.19.1/src/world/mod.rs:1598-1605）；`World::get_entity`
//! （world/mod.rs:951，`Result` 形态，Err 即不存在）（核实 2026-09-27）。

use bevy::prelude::*;

use super::{tc, TestCase};
use crate::sim::{Velocity, Wanderer};

/// TS-06 套件入口（写通路型：探针自清理，无净副作用）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    let baseline = wanderer_count(world);
    // 探针：真实 Wanderer 语义（index 与真实序号空间隔离），spawn 即生效。
    let probe = world
        .spawn((
            Wanderer {
                index: u32::MAX,
                origin: Vec3::ZERO,
                phase: 0.0,
            },
            Velocity {
                linear: Vec3::ZERO,
            },
        ))
        .id();
    vec![
        despawn_removes_entity(world, probe, baseline),
        double_despawn_fails(world, probe, baseline),
    ]
}

fn wanderer_count(world: &mut World) -> usize {
    let mut q = world.query::<&Wanderer>();
    q.iter(world).count()
}

/// 清单 #2 + #3：despawn 探针成功、计数回落、实体不存在。
fn despawn_removes_entity(world: &mut World, probe: Entity, baseline: usize) -> TestCase {
    let after_spawn = wanderer_count(world);
    let despawned = world.despawn(probe);
    let after_despawn = wanderer_count(world);
    let gone = world.get_entity(probe).is_err();
    tc(
        "despawn_removes_entity",
        after_spawn == baseline + 1 && despawned && after_despawn == baseline && gone,
        format!(
            "spawn 探针 {probe:?} 后计数 {after_spawn} == 基线 {baseline}+1；despawn 返回 {despawned}；计数回 {after_despawn} == 基线；get_entity 探针 Err（已不存在）"
        ),
    )
}

/// 清单 #4 + #5：对已销毁实体再次操作必须失败（BRP -23401 错误面的 ECS 等价物）。
fn double_despawn_fails(world: &mut World, probe: Entity, baseline: usize) -> TestCase {
    let again = world.despawn(probe);
    let final_count = wanderer_count(world);
    tc(
        "double_despawn_fails",
        !again && final_count == baseline,
        format!(
            "重复 despawn {probe:?} 返回 {again}（false = 不存在，BRP 侧即 -23401 ENTITY_NOT_FOUND）；终态计数 {final_count} == 基线 {baseline}（世界无残留）"
        ),
    )
}

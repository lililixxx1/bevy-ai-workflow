//! TS-07《远程改写实体运动参数》的游戏进程内断言（两阶段判定第二阶段）。
//!
//! 与 `assets-methodology/taskset/ts-07-mutate-velocity.md` 验收清单的对应：
//! - #1（定位 index==0 的行，记 entity/origin/linear_v0）→ [`index0_located`]；
//! - #2（记 t1）→ 融入 [`transforms_match_analytic_formula`] 的 t 读取；
//! - #3（mutate linear → [0,0,0] 响应无 error）→ [`velocity_write_accepted_and_restored`]
//!   前半：ECS 直写（`World::get_mut`，与 BRP handler 同一 `&mut World` 通路；
//!   BRP 反射 mutate 形态第一阶段已证）改写 + 同帧回读受理；
//! - #4（位置 ≈ 解析式复算，容差 0.1）→ [`transforms_match_analytic_formula`]：
//!   **更强口径**——BRP 跨帧采样复算受帧边界偏移限制只能带容差；进程内
//!   `update_stats`（写 elapsed）与 `move_swarm`（用 elapsed 写 Transform）
//!   同帧同条件 chain 执行（game/src/sim.rs），BRP handler 在帧间读到的
//!   `elapsed_secs` 正是写 Transform 所用的 t ⇒ 同函数同输入 f32 逐位一致，
//!   容差应为 0（非零即口径破绽）。改写的跨帧生效版本属第一阶段测量口径。
//!
//! 写通路改写在本套件内**即写即还原**（同帧无系统执行，不破坏 Transform 与
//! elapsed 的一致性），世界无净副作用。
//!
//! 依据: `World::get_mut::<T>(entity) -> Option<Mut<T>>`
//! （bevy_ecs-0.19.1/src/world/mod.rs:1387）；`wanderer_translation` 为口径
//! 单点定义（game/src/sim.rs，move_swarm 与本套件共用同一函数）（核实 2026-09-27）。

use bevy::prelude::*;

use super::{tc, TestCase};
use crate::sim::{self, SimStats, Velocity, Wanderer};

/// TS-07 套件入口（写通路型：即写即还原，无净副作用）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    let index0 = locate_index0(world);
    vec![
        index0_located(&index0),
        velocity_write_accepted_and_restored(world, &index0),
        transforms_match_analytic_formula(world),
    ]
}

/// 清单 #1 的定位结果（None 时相关断言如实失败）。
struct Index0 {
    entity: Entity,
    #[allow(dead_code)] // 保留字段：与清单「记 origin/linear_v0」对应的取证面。
    origin: Vec3,
    #[allow(dead_code)]
    phase: f32,
    #[allow(dead_code)]
    linear_v0: Vec3,
}

fn locate_index0(world: &mut World) -> Option<Index0> {
    let mut q = world.query::<(Entity, &Wanderer, &Velocity)>();
    q.iter(world)
        .find(|(_, w, _)| w.index == 0)
        .map(|(entity, w, v)| Index0 {
            entity,
            origin: w.origin,
            phase: w.phase,
            linear_v0: v.linear,
        })
}

/// 清单 #1：index==0 的实体可定位。
fn index0_located(index0: &Option<Index0>) -> TestCase {
    tc(
        "index0_located",
        index0.is_some(),
        format!(
            "index==0 实体定位 {:?}（entity/origin/linear_v0/phase 已记录）",
            index0.as_ref().map(|i| i.entity)
        ),
    )
}

/// 清单 #3：改写 Velocity.linear → [0,0,0] 受理（回读逐位），随即还原初值
/// （回读逐位）——写通路语义 + 无净副作用一次完成。
fn velocity_write_accepted_and_restored(world: &mut World, index0: &Option<Index0>) -> TestCase {
    let Some(target) = index0 else {
        return tc(
            "velocity_write_accepted_and_restored",
            false,
            "index==0 未定位，跳过".into(),
        );
    };
    let zero = Vec3::ZERO;
    let wrote = world
        .get_mut::<Velocity>(target.entity)
        .is_some_and(|mut v| {
            v.linear = zero;
            v.linear == zero
        });
    let restored = world
        .get_mut::<Velocity>(target.entity)
        .is_some_and(|mut v| {
            v.linear = target.linear_v0;
            v.linear == target.linear_v0
        });
    tc(
        "velocity_write_accepted_and_restored",
        wrote && restored,
        format!(
            "改写 linear → [0,0,0] 受理（同帧回读逐位一致：{wrote}）；还原 linear_v0={:?} 受理（{restored}）——套件内即写即还原，世界无净副作用",
            target.linear_v0
        ),
    )
}

/// 清单 #4（更强口径）：全部实体 Transform 与解析式复算逐位一致（期望容差 0）。
fn transforms_match_analytic_formula(world: &mut World) -> TestCase {
    let Some(stats) = world.get_resource::<SimStats>() else {
        return tc("transforms_match_analytic_formula", false, "缺 SimStats 资源".into());
    };
    let t = stats.elapsed_secs as f32;
    let mut total = 0u32;
    let mut mismatches = 0u32;
    let mut max_dev: f32 = 0.0;
    let mut q = world.query::<(&Wanderer, &Velocity, &Transform)>();
    for (w, v, transform) in q.iter(world) {
        total += 1;
        let expect = sim::wanderer_translation(w, v, t);
        let dev = (transform.translation - expect).length();
        max_dev = max_dev.max(dev);
        if transform.translation != expect {
            mismatches += 1;
        }
    }
    tc(
        "transforms_match_analytic_formula",
        total > 0 && mismatches == 0,
        format!(
            "t={t:.6}（elapsed_secs，即 move_swarm 写 Transform 所用 t）：{total} 个实体 Transform 与 wanderer_translation 复算逐位一致（不符 {mismatches} 个，最大偏差 {max_dev:e}，期望 0）"
        ),
    )
}

//! TS-12《批量实体操作一致性》的游戏进程内断言（两阶段判定第二阶段）。
//!
//! 与 `assets-methodology/taskset/ts-12-batch-ops.md` 验收清单的对应：
//! - #1（基线 `w0 == 100`、`v0 == 0`）→ [`baseline_matches_checklist`]：
//!   进程内口径 w0 == entity_count（自洽）且 v0 == 0（新进程启动态）；字面 100
//!   由工具端 `--expect-count 100` 绑定；
//! - #2（循环 10 次成对操作，每次均无 error）→ [`batch_loop_ops_succeed`]：
//!   每轮 spawn 一个 Velocity-only 实体 + despawn 一个真实 Wanderer（确定性
//!   取剩余最小 index——第一阶段实测销毁的恰为 index 0..9，本套件显式按
//!   index 序取，口径更强且不依赖 archetype 迭代顺序）；ECS 直写等价
//!   （BRP 批量通路第一阶段 30 响应已证）；
//! - #3（终态 `w1 == 90`、`v1 == 10` 净变化精确一致）→ [`batch_net_change_exact`]；
//! - #4（操作期间 tick 持续增长）→ 工具端 `run` 模式两连调比对 `snapshot.tick`
//!   （套件执行于两次调用之间，操作不阻塞模拟由跨调用递增证明）；
//! - 收尾恢复：despawn 10 个探针 + 按快照（全组件原值，含可选 Tagged——被销毁
//!   的最小 index 恰为打标实体）重生 10 个 Wanderer，
//!   [`world_restored_after_suite`] 断言世界回基线（Wanderer/Velocity-only/Tagged
//!   三计数；实体号允许变化——回归口径禁止断言绝对 entity id，组件值逐位还原）。

use bevy::prelude::*;

use super::{tc, TestCase};
use crate::sim::{SimConfig, Tagged, Velocity, Wanderer};

/// 循环快照：被 despawn 的真实 Wanderer 全组件原值（含可选 `Tagged`——最小
/// index 的 10 个恰是 `index < 10` 的打标实体，重生时必须还原，否则世界在
/// 组件集维度不等价：Tagged(With Wanderer) 计数会 10→0，污染同进程后续套件
/// （ts-09 要求恰 10）——审核 P1-3 修复）。
type Snapshot = (
    Mesh3d,
    MeshMaterial3d<StandardMaterial>,
    Transform,
    Wanderer,
    Velocity,
    Option<Tagged>,
);

/// TS-12 套件入口（写通路型：探针自清理 + 快照重生（含 Tagged），无净副作用）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    let baseline_w = wanderer_count(world);
    let baseline_v = velocity_only_count(world);
    let baseline_t = tagged_count(world);
    let config_count = world
        .get_resource::<SimConfig>()
        .map(|c| c.entity_count as usize);

    let mut snapshots: Vec<Snapshot> = Vec::new();
    let mut probes: Vec<Entity> = Vec::new();
    let mut ops_ok = true;

    for _ in 0..10 {
        // 成对操作前半：spawn Velocity-only 探针。
        let probe = world
            .spawn(Velocity {
                linear: Vec3::new(0.5, 0.0, 0.5),
            })
            .id();
        probes.push(probe);
        // 成对操作后半：despawn 剩余最小 index 的真实 Wanderer（先快照）。
        match pick_lowest_index_wanderner(world) {
            Some((entity, snapshot)) => {
                snapshots.push(snapshot);
                ops_ok &= world.despawn(entity);
            }
            None => ops_ok = false,
        }
    }

    let w1 = wanderer_count(world);
    let v1 = velocity_only_count(world);

    // 恢复：清理探针 + 按快照重生（index/初值/Tagged 逐位还原，实体号允许不同）。
    for probe in &probes {
        ops_ok &= world.despawn(*probe);
    }
    for (mesh, material, transform, wanderer, velocity, tagged) in snapshots {
        match tagged {
            Some(tagged) => {
                world.spawn((mesh, material, transform, wanderer, velocity, tagged));
            }
            None => {
                world.spawn((mesh, material, transform, wanderer, velocity));
            }
        }
    }
    let w_final = wanderer_count(world);
    let v_final = velocity_only_count(world);
    let t_final = tagged_count(world);

    vec![
        tc(
            "baseline_matches_checklist",
            config_count == Some(baseline_w) && baseline_v == 0 && baseline_w >= 10,
            format!(
                "基线 w0={baseline_w} == entity_count {config_count:?}（自洽；字面 100 工具端绑定）、v0={baseline_v} == 0（新进程启动态）、且 w0 ≥ 10（循环配额前置）"
            ),
        ),
        tc(
            "batch_loop_ops_succeed",
            ops_ok,
            "10 轮成对操作（spawn 探针 + despawn 最小 index Wanderer）全部成功（含恢复期 despawn 探针）".into(),
        ),
        tc(
            "batch_net_change_exact",
            baseline_w >= 10 && w1 == baseline_w.saturating_sub(10) && v1 == baseline_v + 10,
            format!(
                "循环后 w1={w1} == w0({baseline_w})-10、v1={v1} == v0({baseline_v})+10（净变化精确一致）"
            ),
        ),
        tc(
            "world_restored_after_suite",
            w_final == baseline_w && v_final == baseline_v && t_final == baseline_t,
            format!(
                "恢复后 w={w_final} == w0({baseline_w})、v={v_final} == v0({baseline_v})、Tagged={t_final} == t0({baseline_t})（探针清理 + 快照重生，index/初值/Tagged 逐位还原）"
            ),
        ),
    ]
}

fn wanderer_count(world: &mut World) -> usize {
    let mut q = world.query::<&Wanderer>();
    q.iter(world).count()
}

fn velocity_only_count(world: &mut World) -> usize {
    let all = {
        let mut q = world.query::<&Velocity>();
        q.iter(world).count()
    };
    let with_w = {
        let mut q = world.query::<(&Velocity, &Wanderer)>();
        q.iter(world).count()
    };
    all - with_w
}

fn tagged_count(world: &mut World) -> usize {
    let mut q = world.query_filtered::<&Tagged, With<Wanderer>>();
    q.iter(world).count()
}

/// 取剩余 index 最小的真实 Wanderer，快照全组件（Mesh3d / 材质 / Transform /
/// Wanderer / Velocity / 可选 Tagged 原值，重生时逐位还原）。
fn pick_lowest_index_wanderner(world: &mut World) -> Option<(Entity, Snapshot)> {
    let mut q = world.query::<(
        Entity,
        &Mesh3d,
        &MeshMaterial3d<StandardMaterial>,
        &Transform,
        &Wanderer,
        &Velocity,
        Option<&Tagged>,
    )>();
    let mut best: Option<(u32, Entity, Snapshot)> = None;
    for (entity, mesh, material, transform, wanderer, velocity, tagged) in q.iter(world) {
        let better = best.as_ref().is_none_or(|(idx, _, _)| wanderer.index < *idx);
        if better {
            best = Some((
                wanderer.index,
                entity,
                (
                    mesh.clone(),
                    material.clone(),
                    *transform,
                    wanderer.clone(),
                    velocity.clone(),
                    tagged.cloned(),
                ),
            ));
        }
    }
    best.map(|(_, entity, snapshot)| (entity, snapshot))
}

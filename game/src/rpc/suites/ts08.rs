//! TS-08《相机轨道参数远程调整》的游戏进程内断言（两阶段判定第二阶段）。
//!
//! 与 `assets-methodology/taskset/ts-08-camera-rig.md` 验收清单的对应：
//! - #1（恰 1 行 Camera+Transform，XZ 距离 ≈ 90.0 容差 0.1）→
//!   [`camera_singleton`] + [`orbit_distance_matches_radius`]（相位 A）；
//! - #2（radius == 90.0、height == 45.0）→ [`rig_defaults_and_transition`] 相位 A
//!   的默认值断言（`CameraRig::default` 字面，game/src/camera.rs）；
//! - #3（mutate radius → 60.0 响应无 error）→ [`rig_defaults_and_transition`]
//!   相位 A 后半：ECS 直写 `resource_mut`（BRP mutate_resources 通路第一阶段
//!   已证）；
//! - #4（距离 ≈ 60.0 容差 0.1，下一帧起生效）→ 相位 B 的
//!   [`orbit_distance_matches_radius`]（跨帧生效需隔帧采样——task-runner
//!   `run` 模式的两连调间隔即生效窗口），随后**还原 radius=90.0**（无净副作用）。
//!
//! **两连调状态机**：本套件按 `CameraRig.radius` 当前值分相位——90.0（初始）
//! 断言默认值并改 60.0；60.0（已改）断言随动生效并还原 90.0。断言名跨相位
//! 稳定，两次调用后世界还原；单次调用后世界停在 radius=60（第二次调用收尾）。
//!
//! **进程史前提**（PIT-M-010，2026-09-29 补注）：相位 A 的默认 90/45 断言隐含
//! 「本进程未加载过关卡」——表现层在 `LevelState` 跨帧持久后会把
//! `CameraRig.radius` 收拢到棋盘视角 14（`game/src/present.rs`）。套件自身
//! 原子无副作用，但驱动侧若在本套件前 launch 过关卡即假红；回归驱动须把
//! 套件排在持久世界变更之前（或每套件独立进程，run-all 口径）。
//!
//! 依据: `World::resource_mut::<R>() -> Mut<R>`（bevy_ecs-0.19.1/src/world/
//! mod.rs:2287）；相机轨道 `orbit_camera` 每帧以 rig 当前值写 Transform
//! （game/src/camera.rs）（核实 2026-09-27）。

use bevy::prelude::*;

use super::{tc, TestCase};
use crate::camera::CameraRig;

const RADIUS_DEFAULT: f32 = 90.0;
const RADIUS_MUTATED: f32 = 60.0;
const HEIGHT_DEFAULT: f32 = 45.0;
/// 清单口径容差（第一阶段实测 |Δ|≈1.6e-7，0.1 为清单原文）。
const TOL: f32 = 0.1;

/// TS-08 套件入口（写通路型：两连调状态机，第二次调用末尾还原，无净副作用）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    let phase_b = world
        .get_resource::<CameraRig>()
        .is_some_and(|rig| rig.radius == RADIUS_MUTATED);
    vec![
        camera_singleton(world),
        orbit_distance_matches_radius(world),
        rig_defaults_and_transition(world, phase_b),
    ]
}

/// 清单 #1 前半：恰 1 个带 Transform 的相机。
fn camera_singleton(world: &mut World) -> TestCase {
    let mut q = world.query_filtered::<&Transform, With<Camera3d>>();
    let count = q.iter(world).count();
    tc(
        "camera_singleton",
        count == 1,
        format!("Camera3d + Transform 实体 {count} 个（清单要求恰 1）"),
    )
}

/// 清单 #1/#4：相机 XZ 平面到原点距离 ≈ 当前 radius（容差 0.1）——相位 A 断
/// 默认轨道，相位 B 断 mutate 后轨道随动（隔帧已生效）。
fn orbit_distance_matches_radius(world: &mut World) -> TestCase {
    let Some(rig) = world.get_resource::<CameraRig>().cloned() else {
        return tc("orbit_distance_matches_radius", false, "缺 CameraRig 资源".into());
    };
    let mut q = world.query_filtered::<&Transform, With<Camera3d>>();
    let Some(distance) = q.iter(world).next().map(|t| t.translation.xz().length()) else {
        return tc("orbit_distance_matches_radius", false, "缺相机实体".into());
    };
    let phase = if rig.radius == RADIUS_MUTATED { "B(已改60)" } else { "A(默认90)" };
    tc(
        "orbit_distance_matches_radius",
        (distance - rig.radius).abs() <= TOL,
        format!(
            "相位 {phase}：XZ 距离 {distance:.9} ≈ radius {}（|Δ|={:.3e} ≤ {TOL}；mutate 随动经帧间生效）",
            rig.radius,
            (distance - rig.radius).abs()
        ),
    )
}

/// 清单 #2/#3：相位 A——默认值 90/45 字面断言后改 60；相位 B——改写受理（60）
/// 断言 + 高度不变断言 + 还原 90。
fn rig_defaults_and_transition(world: &mut World, phase_b: bool) -> TestCase {
    let mut rig = world.resource_mut::<CameraRig>();
    if phase_b {
        let accepted = rig.radius == RADIUS_MUTATED && rig.height == HEIGHT_DEFAULT;
        rig.radius = RADIUS_DEFAULT;
        tc(
            "rig_defaults_and_transition",
            accepted,
            format!(
                "相位 B（第二次调用）：mutate 受理 radius == {RADIUS_MUTATED}、height == {HEIGHT_DEFAULT}；已还原 radius={RADIUS_DEFAULT}（世界回默认）"
            ),
        )
    } else {
        let defaults_ok = rig.radius == RADIUS_DEFAULT && rig.height == HEIGHT_DEFAULT;
        rig.radius = RADIUS_MUTATED;
        tc(
            "rig_defaults_and_transition",
            defaults_ok,
            format!(
                "相位 A（首次调用）：radius == {RADIUS_DEFAULT} 且 height == {HEIGHT_DEFAULT}（CameraRig::default 字面）成立={defaults_ok}；已改写 radius={RADIUS_MUTATED}（第二次调用断言随动并还原）"
            ),
        )
    }
}

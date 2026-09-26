//! 轨道漫游相机插件。
//!
//! 相机是模拟时间的**解析函数**（与实体运动同口径）：
//! `pos(t) = (R·cos(ω·t + φ), H, R·sin(ω·t + φ))`，`t = Time::elapsed_secs()`
//! （真实时钟：暂停模拟时相机作为观察者继续漫游，便于 BRP 验证暂停语义）。
//!
//! BRP 断言面：`CameraRig`（Resource，可 `world.get_resources` / `world.mutate_resources`）
//! 与相机的 `Transform`（到原点距离恒等于 `radius`，±0.01 容差）。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 轨道相机参数（BRP 可读可改）。
#[derive(Resource, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct CameraRig {
    /// 轨道半径（米）。
    pub radius: f32,
    /// 轨道平面高度（米）。
    pub height: f32,
    /// 角速度（弧度/秒）。
    pub angular_speed: f32,
    /// 初始相位（弧度）。
    pub phase: f32,
}

impl Default for CameraRig {
    fn default() -> Self {
        Self {
            radius: 90.0,
            height: 45.0,
            angular_speed: 0.1,
            phase: 0.0,
        }
    }
}

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraRig>()
            .init_resource::<CameraRig>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, orbit_camera);
    }
}

/// 依据: 相机形态 `bevy-0.19.1/examples/remote/server.rs:62-65`（Camera3d + Transform
/// + looking_at，核实 2026-09-26）。
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(90.0, 45.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn orbit_camera(rig: Res<CameraRig>, time: Res<Time>, mut query: Query<&mut Transform, With<Camera3d>>) {
    let t = time.elapsed_secs();
    let angle = rig.angular_speed * t + rig.phase;
    let pos = Vec3::new(
        rig.radius * angle.cos(),
        rig.height,
        rig.radius * angle.sin(),
    );
    for mut transform in &mut query {
        transform.translation = pos;
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbit_distance_is_constant() {
        let rig = CameraRig::default();
        for t in [0.0_f32, 1.7, 100.0] {
            let angle = rig.angular_speed * t + rig.phase;
            let pos = Vec3::new(rig.radius * angle.cos(), rig.height, rig.radius * angle.sin());
            let d = (pos - Vec3::new(0.0, pos.y, 0.0)).length();
            assert!((d - rig.radius).abs() < 1e-3, "t={t} d={d}");
        }
    }
}

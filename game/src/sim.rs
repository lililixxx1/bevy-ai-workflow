//! 动态实体模拟插件（M1 第一役 demo 主体）。
//!
//! 确定性口径（帧率基线与任务测试集共用的「可复现」定义）：
//! - 实体初值（origin / velocity / phase）由 `--seed` 经 SplitMix64 顺序生成：
//!   同种子 + 同 `--count` ⇒ 每个实体（按 `Wanderer::index`）初值逐位相同；
//! - 位置是模拟时间 `t`（`SimStats::elapsed_secs`，仅未暂停时累计）的**解析函数**：
//!   `translation = origin + linear * t + Y * BOUNCE_AMP * sin(TAU * BOUNCE_HZ * t + phase)`
//!   （常量见 [`BOUNCE_AMP`] / [`BOUNCE_HZ`]）。同种子同 t ⇒ 所有实体位置逐位相同，
//!   与帧率无关——BRP 端可据 `SimStats.elapsed_secs` 精确复算并断言任意实体位置。
//! - 暂停（`SimConfig::paused = true`，可经 BRP `world.mutate_resources` 翻转）：
//!   tick / elapsed / 实体位置全部冻结（运动系统与统计系统共用同一 run condition）。
//!
//! BRP 反射注册（SKILL.md §3.4 强制）：本插件 build 中显式 `register_type`
//! 全部四个类型；BRP 全路径即 Rust 模块路径（`game::sim::Wanderer` 等）。

use crate::rng::{phase_hash, SplitMix64};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 实体分布半宽（xz 平面 ∈ [-AREA, AREA]，米）。
pub const AREA: f32 = 120.0;
/// 速度模长上限（米/秒）。
pub const MAX_SPEED: f32 = 3.0;
/// 垂直扰动幅度（米）。
pub const BOUNCE_AMP: f32 = 0.8;
/// 垂直扰动频率（赫兹）。
pub const BOUNCE_HZ: f32 = 0.7;

/// 模拟配置（只读侧；`paused` 可经 BRP 翻转）。
#[derive(Resource, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct SimConfig {
    /// 动态实体数（启动时由 CLI 注入）。
    pub entity_count: u32,
    /// 随机种子（口径常量默认 20260926）。
    pub seed: u64,
    /// 采集时长（秒）；0 = 常驻运行。
    pub bench_secs: f32,
    /// 暂停标志：true 时 tick / elapsed / 位置冻结。
    #[serde(default)]
    pub paused: bool,
}

/// 模拟统计（每帧更新，BRP 断言的主要只读状态面）。
#[derive(Resource, Reflect, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct SimStats {
    /// 模拟 tick 数（每未暂停帧 +1；BRP 断言「每帧真实更新」的直接证据）。
    pub tick: u64,
    /// 渲染帧计数（含暂停帧，由 bench 模块维护）。
    pub frame_count: u64,
    /// 累计模拟时间（秒，仅未暂停时累计；解析式运动的 t）。
    pub elapsed_secs: f64,
    /// 最近 1 秒窗口帧率（bench 模块每秒刷新）。
    pub fps_1s: f64,
    /// 稳态平均帧率（warmup 丢弃后；bench 模块刷新）。
    pub avg_fps: f64,
}

/// 动态实体标记：index 与确定性初值（BRP `world.query` 的过滤组件）。
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Wanderer {
    /// 实体序号（0..entity_count，初值生成顺序号）。
    pub index: u32,
    /// 解析式运动的基准位置（米）。
    pub origin: Vec3,
    /// 扰动相位（弧度，由 (seed, index) 哈希派生）。
    pub phase: f32,
}

/// 漂移速度（恒定初值；BRP 可读可比对）。
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Velocity {
    /// 平均漂移速度向量（米/秒）。
    pub linear: Vec3,
}

pub struct SimPlugin {
    pub config: SimConfig,
}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .init_resource::<SimStats>()
            .register_type::<SimConfig>()
            .register_type::<SimStats>()
            .register_type::<Wanderer>()
            .register_type::<Velocity>()
            .add_systems(Startup, spawn_swarm)
            .add_systems(
                Update,
                (update_stats, move_swarm)
                    .chain()
                    .run_if(|config: Res<SimConfig>| !config.paused),
            );
    }
}

/// Startup：按配置生成实体群 + 地面 + 光照。
///
/// 依据: spawn_batch 签名 `bevy_ecs-0.19.1/src/system/commands/mod.rs:587-593`；
/// Mesh3d/MeshMaterial3d/材质形态 `bevy-0.19.1/examples/remote/server.rs:32-44`；
/// DirectionalLight 默认无阴影 `bevy_light-0.19.1/src/directional_light.rs:147-160`
/// （核实 2026-09-26）。
fn spawn_swarm(
    mut commands: Commands,
    config: Res<SimConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    let mesh = meshes.add(Cuboid::new(0.6, 0.6, 0.6));
    let body_material = materials.add(Color::srgb_u8(124, 144, 255));
    let ground_material = materials.add(Color::srgb_u8(34, 36, 48));

    // 地面（不参与模拟，提供参照）。
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(AREA * 1.2))),
        MeshMaterial3d(ground_material),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // 光照：平行光（默认无阴影，50000 实体下阴影代价不纳入本基线）。
    commands.spawn((
        DirectionalLight {
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_3)),
    ));

    // 确定性初值：同种子同 count 下逐位相同。
    // spawn_batch 要求迭代器 'static（commands/mod.rs:589），故所需值全部
    // 拷出/移入 move 闭包，不借用系统参数。
    let seed = config.seed;
    let mut rng = SplitMix64::new(seed);
    let iter = (0..config.entity_count).map(move |index| {
        let origin = Vec3::new(
            rng.next_range_f32(AREA),
            0.8,
            rng.next_range_f32(AREA),
        );
        let dir = Vec2::new(rng.next_range_f32(1.0), rng.next_range_f32(1.0));
        let speed = rng.next_f32() * MAX_SPEED;
        let linear = Vec3::new(dir.x, 0.0, dir.y).normalize_or_zero() * speed;
        let phase = phase_hash(seed, index);
        let translation = wanderer_translation(&Wanderer { index, origin, phase }, &Velocity { linear }, 0.0);
        (
            Mesh3d(mesh.clone()),
            MeshMaterial3d(body_material.clone()),
            Transform::from_translation(translation),
            Wanderer { index, origin, phase },
            Velocity { linear },
        )
    });
    commands.spawn_batch(iter);

    info!(
        "[SIM] spawned {} wanderers | seed={} | area={} | max_speed={}",
        config.entity_count, config.seed, AREA, MAX_SPEED
    );
}

/// 解析式运动公式（口径单点定义，测试与 BRP 断言共用）。
pub fn wanderer_translation(w: &Wanderer, v: &Velocity, t: f32) -> Vec3 {
    w.origin + v.linear * t
        + Vec3::Y * (BOUNCE_AMP * (std::f32::consts::TAU * BOUNCE_HZ * t + w.phase).sin())
}

/// Update：统计先行（写 tick / elapsed），运动后行（读 elapsed）——chain 显式排序。
fn update_stats(mut stats: ResMut<SimStats>, time: Res<Time>) {
    stats.tick += 1;
    stats.elapsed_secs += f64::from(time.delta_secs());
}

fn move_swarm(stats: Res<SimStats>, query: Query<(&Wanderer, &Velocity, &mut Transform)>) {
    let t = stats.elapsed_secs as f32;
    for (w, v, mut transform) in query {
        transform.translation = wanderer_translation(w, v, t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytic_motion_is_deterministic_per_seed_and_time() {
        let mut rng = SplitMix64::new(20260926);
        let w = Wanderer {
            index: 7,
            origin: Vec3::new(rng.next_range_f32(AREA), 0.8, rng.next_range_f32(AREA)),
            phase: phase_hash(20260926, 7),
        };
        let v = Velocity {
            linear: Vec3::new(1.5, 0.0, -0.5),
        };
        let a = wanderer_translation(&w, &v, 12.5);
        let b = wanderer_translation(&w, &v, 12.5);
        assert_eq!(a, b, "同种子同时间 ⇒ 同位置（逐位）");
        let c = wanderer_translation(&w, &v, 12.6);
        assert_ne!(a, c, "时间推进 ⇒ 位置变化（每帧动态更新）");
    }

    #[test]
    fn same_seed_and_count_same_initial_state() {
        let gen = || {
            let mut rng = SplitMix64::new(42);
            (0..3)
                .map(|_| {
                    (
                        rng.next_range_f32(AREA),
                        rng.next_range_f32(1.0),
                        rng.next_f32(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(gen(), gen(), "初值序列与抽取顺序绑定，跨调用一致");
    }
}

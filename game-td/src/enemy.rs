//! 敌人与寻路（教程 07 Step 2/4/5/6）：组件 + `FixedUpdate` 位置式移动。
//!
//! 设计（教程 07 Step 2）：
//! - [`Enemy`]：纯标记，供后续塔 / 投射物 `With<Enemy>` 查询；
//! - [`PathFollower`]：导航状态（`path_id` / `waypoint_index` / **预计算的世界坐标
//!   `target`**）——`move_enemies` 每 tick 跑在所有敌人上，直接存 `Vec2` 省掉
//!   HashMap 查找与 `tile_to_world` 换算；
//! - [`MoveSpeed`]：newtype 包裹 `f32`，不同敌种可不同速。
//!
//! 移动策略（教程 07 Step 5）：**位置式**——`Transform` 是唯一真值，每 tick 朝
//! `target` 走 `speed * dt`，抵达（距离 ≤ 0.01）即推进航点；不携带跨段余量
//! （tutorial「Why not carry over overshoot」）。到达末航点移除 `PathFollower`，
//! 由 [`cleanup_finished_enemies`] 回收（教程 07 Step 6：把「到终点」与
//! 「后续结算」解耦，Part 11 会在此扣命）。
//!
//! 时间源（核查 2026-10-30）：`FixedUpdate` 内 `Res<Time>` = `Time<Fixed>`——
//! `run_fixed_main_schedule` 每 tick 以 `Time<Fixed>::as_generic()` 覆写默认时钟
//! （`bevy_time-0.20.0/src/fixed.rs:243-261`；文档 `src/time.rs:31-59`），
//! 故 `time.delta_secs()` = 固定步长 1/64 s（`DEFAULT_TIMESTEP`
//! `src/fixed.rs:76` = 15625 µs）。教程正文写「60 Hz by default」为近似表述，
//! 行为规格（每 tick 走 `speed × 固定步长`）不受影响。

use bevy::prelude::*;

use crate::level::LevelData;

/// 敌人标记（教程 07 Step 2）：玩法系统只认这个，不关心寻路细节。
///
/// derive 形态按 SKILL.md §3.4（BRP 查询要求 Reflect + `#[reflect(Component)]`）；
/// `#[type_path = "game_td"]` 冻结反射类型路径（同 map.rs 注释：BRP 证据脚本按
/// `game_td::Enemy` 查注册表）。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub struct Enemy;

/// 寻路状态（教程 07 Step 2）：`target` 为**世界坐标**，spawn / 推进时预计算。
#[derive(Component)]
pub struct PathFollower {
    pub path_id: String,
    pub waypoint_index: usize,
    pub target: Vec2,
}

/// 移动速度（教程 07 Step 2）：世界单位 / 秒（教程 07 Step 3：3 格/秒 × 64px = 192）。
#[derive(Component)]
pub struct MoveSpeed(pub f32);

/// 网格坐标 → 世界坐标（教程 07 Step 4）：与 Part 2 的居中算式同式。
///
/// `pub` 供塔（教程 Part 8）做射程与投射物换算复用（教程 07 Step 4 原话）。
pub fn tile_to_world(tile: [u32; 2], map_width: f32, map_height: f32) -> Vec2 {
    let tile_size = 64.0;
    let origin_x = -map_width * tile_size / 2.0 + tile_size / 2.0;
    let origin_y = -map_height * tile_size / 2.0 + tile_size / 2.0;
    Vec2::new(
        origin_x + tile[0] as f32 * tile_size,
        origin_y + tile[1] as f32 * tile_size,
    )
}

/// 每固定步推进所有敌人（教程 07 Step 5 `move_enemies`）。
pub fn move_enemies(
    mut commands: Commands,
    mut query: Query<(Entity, &mut PathFollower, &mut Transform, &MoveSpeed), With<Enemy>>,
    level: Res<LevelData>,
    time: Res<Time>,
) {
    let map_width = level.map.width as f32;
    let map_height = level.map.height as f32;

    for (entity, mut follower, mut transform, speed) in query.iter_mut() {
        let current = transform.translation.truncate();
        let to_target = follower.target - current;
        let distance = to_target.length();

        if distance <= 0.01 {
            // Snap distance: avoid division by zero / atan2 instability when
            // the enemy is already on (or extremely close to) the target tile.
            advance_waypoint(
                &mut commands,
                entity,
                &mut follower,
                &level,
                map_width,
                map_height,
            );
            continue;
        }

        let direction = to_target / distance;
        let step = speed.0 * time.delta_secs();
        let angle = direction.y.atan2(direction.x);

        if distance <= step {
            // Reach (or would overshoot) the waypoint this frame.
            transform.translation.x = follower.target.x;
            transform.translation.y = follower.target.y;
            transform.rotation = Quat::from_rotation_z(angle);
            advance_waypoint(
                &mut commands,
                entity,
                &mut follower,
                &level,
                map_width,
                map_height,
            );
        } else {
            let new_pos = current + direction * step;
            transform.translation.x = new_pos.x;
            transform.translation.y = new_pos.y;
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}

/// 推进到下一航点（教程 07 Step 6）：末航点到达后移除 `PathFollower`（不立即 despawn）。
fn advance_waypoint(
    commands: &mut Commands,
    entity: Entity,
    follower: &mut PathFollower,
    level: &LevelData,
    map_width: f32,
    map_height: f32,
) {
    let waypoints = &level.paths[&follower.path_id].waypoints;
    follower.waypoint_index += 1;

    if follower.waypoint_index >= waypoints.len() {
        commands.entity(entity).remove::<PathFollower>();
    } else {
        follower.target = tile_to_world(waypoints[follower.waypoint_index], map_width, map_height);
    }
}

/// 回收已到终点的敌人（教程 07 Step 6）：`Enemy` 在、`PathFollower` 不在 = 到终点。
pub fn cleanup_finished_enemies(
    mut commands: Commands,
    query: Query<Entity, (With<Enemy>, Without<PathFollower>)>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

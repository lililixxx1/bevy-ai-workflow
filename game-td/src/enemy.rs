//! 敌人与寻路（教程 07 Step 2/4/5/6）：组件 + `FixedUpdate` 位置式移动；
//! P10 起加扁平化刷怪调度（教程 10）：`[enemy_types.*]` + `[[waves]]` 在装载期
//! 展开为按时间排序的 `SpawnEvent` 队列，`spawn_wave_enemies` 每固定步弹出到期事件。
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

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::level::LevelData;
use crate::{GameEntity, GameResult, GameState};

/// 敌人标记（教程 07 Step 2）：玩法系统只认这个，不关心寻路细节。
///
/// derive 形态按 SKILL.md §3.4（BRP 查询要求 Reflect + `#[reflect(Component)]`）；
/// `#[type_path = "game_td"]` 冻结反射类型路径（同 map.rs 注释：BRP 证据脚本按
/// `game_td::Enemy` 查注册表）。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub struct Enemy;

/// 生命值（教程 09 Step 1）：`pub` 字段因为塔模块要读写。
///
/// 元组结构体经 ReflectSerializer 落成 JSON 数组（`TupleStructSerializer`
/// `bevy_reflect-0.20.0/src/serde/ser/tuple_structs.rs:11`），证据脚本按 `[hp]` 取值。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub struct Health(pub f32);

/// 单条刷怪事件（教程 10 Step 3）：纯数据，不挂实体，只在调度队列里存在。
#[derive(Debug, Clone)]
pub struct SpawnEvent {
    /// 相对关卡开始的触发秒数（`sort_by(total_cmp)` 的依据）。
    pub time: f32,
    /// 图集索引。
    pub sprite: usize,
    /// 世界单位 / 秒。
    pub speed: f32,
    /// 初始生命。
    pub health: f32,
    /// 路径 id（`[paths.<id>]` 键）。
    pub path: String,
}

/// 扁平化刷怪调度（教程 10 Step 3/4）。
///
/// `events` 为按 `time` 排序后的 `VecDeque`；`elapsed` 是固定步长累积的关卡时钟。
/// `spawned` 是已消费事件计数（教程外的最小记账字段：供 `td.wave` 方法与断言
/// 对拍「已刷 = 总数 - 待刷」，不给玩法逻辑增加语义）。
/// `texture` / `atlas` 为预加载句柄（教程 10 Step 3 注：避免每帧 `asset_server.load`）。
#[derive(Resource)]
pub struct SpawnSchedule {
    pub events: VecDeque<SpawnEvent>,
    pub elapsed: f32,
    pub spawned: u32,
    texture: Handle<Image>,
    atlas: Handle<TextureAtlasLayout>,
}

impl SpawnSchedule {
    /// 待刷事件数（`td.wave` 与断言共用）。
    pub fn pending(&self) -> usize {
        self.events.len()
    }
}

/// 由关卡数据构建扁平化调度（教程 10 Step 4 `build_spawn_schedule`）：
/// 逐波 → 逐组 → 逐只展开事件，最后按 `time` 全局排序（跨波自然交错）。
pub fn build_spawn_schedule(
    level: &LevelData,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> SpawnSchedule {
    let mut events: Vec<SpawnEvent> = Vec::new();

    for wave in &level.waves {
        let mut time = wave.start_time;
        for group in &wave.enemies {
            let def = &level.enemy_types[&group.enemy_type];
            for _ in 0..group.count {
                events.push(SpawnEvent {
                    time,
                    sprite: def.sprite,
                    speed: def.speed,
                    health: def.health,
                    path: wave.path.clone(),
                });
                time += wave.spawn_interval;
            }
        }
    }

    events.sort_by(|a, b| a.time.total_cmp(&b.time));

    SpawnSchedule {
        events: events.into(),
        elapsed: 0.0,
        spawned: 0,
        texture: asset_server.load("Tilesheet/towerDefense_tilesheet.png"),
        // 0.20 锚点：`from_grid(tile_size, columns, rows, padding, offset)`
        // `bevy_image-0.20.0/src/texture_atlas.rs:133-139`。
        atlas: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(64),
            23,
            13,
            None,
            None,
        )),
    }
}

/// 按调度刷怪（教程 10 Step 5 `spawn_wave_enemies`）：推进关卡钟 → 弹出所有到期事件。
///
/// 挂 `FixedUpdate`（`time.delta_secs()` = 固定步长，教程 10「Time::delta_secs() in
/// FixedUpdate」）；`while let` 处理单 tick 多个事件到期。
///
/// P12：P11 的 `Option<Res<GameOver>>` 早退退役——门控改由
/// `.run_if(in_state(GameState::InGame))` 承担（教程 12「`.run_if(in_state(...))`」：
/// GameOver 态整条 FixedUpdate 链不跑，无需函数内守卫）。
pub fn spawn_wave_enemies(
    mut schedule: ResMut<SpawnSchedule>,
    mut commands: Commands,
    level: Res<LevelData>,
    time: Res<Time>,
) {
    schedule.elapsed += time.delta_secs();

    while let Some(event) = schedule.events.front() {
        if event.time > schedule.elapsed {
            break;
        }
        let event = schedule.events.pop_front().unwrap();
        schedule.spawned += 1;

        let waypoints = &level.paths[&event.path].waypoints;
        let spawn_tile = waypoints[0];
        let target_tile = waypoints[1];

        let map_width = level.map.width as f32;
        let map_height = level.map.height as f32;
        let spawn = tile_to_world(spawn_tile, map_width, map_height);
        let target = tile_to_world(target_tile, map_width, map_height);

        commands.spawn((
            Sprite::from_atlas_image(
                schedule.texture.clone(),
                TextureAtlas {
                    layout: schedule.atlas.clone(),
                    index: event.sprite,
                },
            ),
            Transform::from_xyz(spawn.x, spawn.y, 1.0),
            Enemy,
            PathFollower {
                path_id: event.path,
                waypoint_index: 1,
                target,
            },
            MoveSpeed(event.speed),
            Health(event.health),
            // 教程 12 Step 3：玩法实体统一挂 GameEntity（cleanup_level 清场用）。
            GameEntity,
        ));
    }
}

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

/// 基地生命（教程 11 Step 1）：漏怪扣命，归零即败。
///
/// `i32` 而非 `f32`——命是离散量（教程 11 Step 1 注）；起始值由关卡装载注入（5），
/// P12 起随 `cleanup_level` 回收、每回合重建。
#[derive(Resource)]
pub struct BaseLives(pub i32);

/// 本局已判负/判胜标记（教程 12 Step 4 `GameFinished`）：**资源**而非 `Local<bool>`。
///
/// 关键理由（教程 12 Step 4 注）：`Local` 活到 app 结束——玩家回菜单再开一局时
/// 旧 `true` 会压制新一局的胜负判定；本资源在 `cleanup_level` 里移除，每回合归零。
#[derive(Resource)]
pub struct GameFinished;

/// 回收到达终点的敌人并扣命（教程 11 Step 2 `process_base_reachers`）：
/// 查询与原 `cleanup_finished_enemies` 相同，唯一新增 = 先扣命再 despawn。
/// `Enemy` 在、`PathFollower` 不在 = 到终点（教程 07 Step 6 的判定）。
pub fn process_base_reachers(
    mut commands: Commands,
    mut lives: ResMut<BaseLives>,
    query: Query<Entity, (With<Enemy>, Without<PathFollower>)>,
) {
    for entity in &query {
        lives.0 -= 1;
        commands.entity(entity).despawn();
    }
}

/// 胜负判定（教程 12 Step 4 `check_game_state`）：
/// - 败：`BaseLives <= 0`；
/// - 胜：刷怪队列空 **且** 场上无敌人（两个条件缺一不可——最后一批还在走时不能判胜）。
///
/// P11 → P12 的两处变化（教程 12 Step 4）：
/// ① `Local<bool>` 门闩换成 `Option<Res<GameFinished>>`——随 `cleanup_level` 回收，
///    重开局不会带着旧判定；
/// ② 判定结果走 `NextState<GameState>` 进入状态机（`OnExit(InGame)` 先 `cleanup_level`、
///    再 `OnEnter(GameOver)` 读 `GameResult`，教程 12 Step 9 的顺序契约）。
pub fn check_game_state(
    mut commands: Commands,
    finished: Option<Res<GameFinished>>,
    lives: Res<BaseLives>,
    schedule: Res<SpawnSchedule>,
    alive: Query<(), With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if finished.is_some() {
        return;
    }
    if lives.0 <= 0 {
        info!("Game Over — the base has been destroyed!");
        commands.insert_resource(GameResult::Defeat);
        commands.insert_resource(GameFinished);
        next_state.set(GameState::GameOver);
    } else if schedule.events.is_empty() && alive.iter().count() == 0 {
        info!("Victory — all enemies defeated!");
        commands.insert_resource(GameResult::Victory);
        commands.insert_resource(GameFinished);
        next_state.set(GameState::GameOver);
    }
}

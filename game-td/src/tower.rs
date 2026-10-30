//! 塔放置（教程 08 行为规格；P8）+ 索敌 / 伤害 / 弹药冷却（教程 09；P9）。
//!
//! 教程（0.18 形态）只是行为规格，本模块每个 API 按 bevy 0.20.0 本地源码重写
//! （对照底稿 `docs/evidence/t044-td-spike/w2/api-drift.md` Part 08/09 节）：
//! - `Single<&Window>` / `Single<(&Camera, &GlobalTransform)>`（`bevy_ecs` prelude
//!   `Single`，`bevy_ecs-0.20.0/src/lib.rs:99`）；
//! - `Window::cursor_position()` 返回 `Option<Vec2>` 逻辑像素
//!   （`bevy_window-0.20.0/src/window.rs:620`）；
//! - `Camera::viewport_to_world_2d(&GlobalTransform, Vec2) -> Result<Vec2, _>`
//!   （`bevy_camera-0.20.0/src/camera.rs:710`）；
//! - `Visibility::Visible/Hidden`（`bevy_camera-0.20.0/src/visibility/mod.rs:83` 起）；
//! - `Timer::from_seconds` / `tick` / `just_finished`、`TimerMode::Repeating`
//!   （`bevy_time-0.20.0/src/timer.rs:61`/`:280`/`:110`/`:497`）；
//! - `with_children` 父子层级（`bevy_ecs-0.20.0/src/hierarchy.rs:366`；
//!   0.18 的 `Parent` 在 0.20 为 `ChildOf`——漂移表 Part 09 摩擦级 1，本模块不直接命名它）。
//!
//! 设计（教程 08/09 Walkthrough）：
//! - 塔 = **两个实体**：`Tower` 基座（静态）+ `TowerTurret` 炮塔（独立旋转）；
//! - `TowerPreview`：常驻的两个半透明幽灵实体，靠 `Visibility` 开关而非 spawn/despawn；
//! - `PlacedTowers`：已占格 `HashSet`，O(1) 放置校验（路径 / 已占用都拒绝）；
//! - `TowerAtlas`：预加载的图集句柄资源（教学「Shared handles as resources」）；
//! - `world_to_tile`：`tile_to_world` 的逆变换，**必须用 round 而非 floor**
//!   （`tile_to_world` 把 sprite 放在格中心，教程 08 Step 3）；
//! - P9 `attack_enemies`：单系统完成索敌 → 旋转 → 冷却到期扣血 → 就地 despawn，
//!   敌人先快照成 `Vec<(Entity, Vec2)>` 释放查询借用（教程 09「Snapshot pattern」），
//!   两条查询以 `Without<Enemy>` / `Without<TowerTurret>` 显式互斥（防 B0001）。
//!
//! BRP 面（本工程扩展层，非教程）：[`spawn_tower_entities`] 与 [`is_placeable`]
//! 同时被点击系统与 `td.spawn_tower` handler 复用——BRP 无鼠标注入通道，测试
//! 放置必须走确定性方法面（`td.` 前缀纪律）。

use std::collections::HashSet;

use bevy::prelude::*;

use crate::enemy::{tile_to_world, Enemy, Health, PathFollower};
use crate::map::{MapLayout, TileType};
use crate::GameEntity;

/// 基座图集索引（教程 08 Step 1 `TOWER_BASE`）。
pub const TOWER_BASE: usize = 180;

/// 炮塔图集索引（教程 08 Step 1 `TOWER_TOP`）。
pub const TOWER_TOP: usize = 203;

/// 攻击半径（教程 09 Step 2）：3 格 = 192px。
pub const ATTACK_RANGE: f32 = 192.0;

/// 单发伤害（教程 09 Step 2）：3 发打死 100 HP 敌人。
pub const DAMAGE: f32 = 34.0;

/// 攻击冷却（教程 09 Step 2）：每秒两发。
pub const ATTACK_COOLDOWN: f32 = 0.5;

/// 枪口焰图集索引（教程 09 Step 3）。
pub const FIRE_SPRITE: usize = 295;

/// 枪口焰存续时长（教程 09 Step 3）。
pub const MUZZLE_FLASH_DURATION: f32 = 0.15;

/// 塔基座标记（教程 08 Step 1）：其它系统（P9 塔查询）按它查询，故 `pub`。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub struct Tower;

/// 炮塔标记（教程 08 Step 1，`pub(crate)`）。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub(crate) struct TowerTurret;

/// 放置预览幽灵标记（教程 08 Step 1，`pub(crate)`）。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub(crate) struct TowerPreview;

/// 攻击半径组件（教程 09 Step 2，`pub(crate)`）。
#[derive(Component)]
pub(crate) struct AttackRange(pub f32);

/// 单发伤害组件（教程 09 Step 2，`pub(crate)`）。
#[derive(Component)]
pub(crate) struct Damage(pub f32);

/// 每塔独立冷却（教程 09 Step 2：Timer 组件而非全局资源，各塔节奏独立）。
#[derive(Component)]
pub(crate) struct AttackTimer(pub Timer);

/// 通用存续计时（教程 09 Step 3）：任何挂它的实体到期即回收。
#[derive(Component)]
pub(crate) struct DespawnTimer(pub Timer);

/// 枪口焰标记（教程 09 Step 3，`pub(crate)`）。
#[derive(Component)]
pub(crate) struct MuzzleFlash;

/// 共享图集句柄（教程 08 Step 2）：Startup 装载一次，系统与 BRP handler 共用。
#[derive(Resource, Clone)]
pub struct TowerAtlas {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// 已占用格（教程 08 Step 1）：`HashSet` 而非瓦片组件——
/// tilemap 实体归插件所有，不挂自定义组件（教程 08 Step 1 注）。
#[derive(Resource, Default)]
pub struct PlacedTowers(pub HashSet<[u32; 2]>);

/// 世界坐标 → 网格坐标（教程 08 Step 3）：`tile_to_world` 的逆。
///
/// 用 `.round()` 对齐「spawn 在格中心」的坐标语义；`.floor()` 会把右半格的
/// 光标错归到左邻格（教程 08 Step 3 注）。越界返回 `None`。
pub fn world_to_tile(world: Vec2, map_width: u32, map_height: u32) -> Option<[u32; 2]> {
    let tile_size = 64.0;
    let origin_x = -(map_width as f32) * tile_size / 2.0 + tile_size / 2.0;
    let origin_y = -(map_height as f32) * tile_size / 2.0 + tile_size / 2.0;

    let tx = ((world.x - origin_x) / tile_size).round() as i32;
    let ty = ((world.y - origin_y) / tile_size).round() as i32;

    if tx >= 0 && tx < map_width as i32 && ty >= 0 && ty < map_height as i32 {
        Some([tx as u32, ty as u32])
    } else {
        None
    }
}

/// 放置合法性（教程 08 Step 4）：草地 + 未占用。路径 / 越界 / 已占均拒绝。
pub fn is_placeable(map: &MapLayout, placed: &PlacedTowers, tile: [u32; 2]) -> bool {
    let is_grass = map.get(tile[0] as i32, tile[1] as i32) == Some(TileType::Grass);
    is_grass && !placed.0.contains(&tile)
}

/// 光标下的可放置格（教程 08 Step 4 `hovered_placeable_tile`）：
/// 光标屏幕坐标 → 世界坐标 → 网格坐标 → 合法性，一条 `and_then` 链。
fn hovered_placeable_tile(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    map_layout: &MapLayout,
    placed: &PlacedTowers,
) -> Option<[u32; 2]> {
    let tile = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
        .and_then(|world| world_to_tile(world, map_layout.width, map_layout.height))?;

    is_placeable(map_layout, placed, tile).then_some(tile)
}

/// 预加载塔图集（教程 08 Step 2 `setup_tower_atlas`）。
pub fn setup_tower_atlas(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("Tilesheet/towerDefense_tilesheet.png");
    // 0.20 锚点：`from_grid(tile_size, columns, rows, padding, offset)`
    // `bevy_image-0.20.0/src/texture_atlas.rs:133-139`。
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 23, 13, None, None);
    commands.insert_resource(TowerAtlas {
        texture,
        layout: texture_atlas_layouts.add(layout),
    });
}

/// 生成塔实体对（教程 08 Step 7 的 spawn 段）：基座 + 炮塔。
///
/// 点击系统与 BRP `td.spawn_tower` handler 共用同一段生成逻辑（BRP handler
/// 经 `World::commands` 拿到 `Commands`，`bevy_ecs-0.20.0/src/world/mod.rs:314`），
/// 两条通道的塔形态不会漂移。返回塔中心世界坐标。
pub fn spawn_tower_entities(
    commands: &mut Commands,
    atlas: &TowerAtlas,
    tile: [u32; 2],
    map_width: f32,
    map_height: f32,
) -> Vec2 {
    let pos = tile_to_world(tile, map_width, map_height);

    // 基座（静态，不旋转）。教程 12 Step 3：挂 GameEntity。
    commands.spawn((
        Tower,
        GameEntity,
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: TOWER_BASE,
            },
        ),
        Transform::from_xyz(pos.x, pos.y, 2.0),
    ));

    // 炮塔（P9 起朝敌人独立旋转；战斗属性随塔创建——教程 09 Step 6）。
    commands.spawn((
        TowerTurret,
        GameEntity,
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: TOWER_TOP,
            },
        ),
        Transform::from_xyz(pos.x, pos.y, 2.1),
        AttackRange(ATTACK_RANGE),
        Damage(DAMAGE),
        AttackTimer(Timer::from_seconds(ATTACK_COOLDOWN, TimerMode::Repeating)),
    ));

    pos
}

/// 生成两个常驻预览幽灵（教程 08 Step 5）：半透明、初始隐藏。
pub fn spawn_placement_preview(mut commands: Commands, atlas: Res<TowerAtlas>) {
    let tinted = |index: usize| Sprite {
        color: Color::srgba(1.0, 1.0, 1.0, 0.5),
        ..Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index,
            },
        )
    };

    commands.spawn((
        TowerPreview,
        GameEntity,
        tinted(TOWER_BASE),
        Transform::from_xyz(0.0, 0.0, 2.0),
        Visibility::Hidden,
    ));
    commands.spawn((
        TowerPreview,
        GameEntity,
        tinted(TOWER_TOP),
        Transform::from_xyz(0.0, 0.0, 2.1),
        Visibility::Hidden,
    ));
}

/// 每帧按悬停更新预览（教程 08 Step 6）：合法格移动并显示，否则隐藏。
pub fn update_placement_preview(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    map_layout: Res<MapLayout>,
    placed: Res<PlacedTowers>,
    mut preview_q: Query<(&mut Transform, &mut Visibility), With<TowerPreview>>,
) {
    let (cam, cam_transform) = *camera;

    let Some(tile) = hovered_placeable_tile(&window, cam, cam_transform, &map_layout, &placed)
    else {
        for (_, mut vis) in preview_q.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    };

    let pos = tile_to_world(tile, map_layout.width as f32, map_layout.height as f32);

    for (mut transform, mut vis) in preview_q.iter_mut() {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
        *vis = Visibility::Visible;
    }
}

/// 左键点击放置（教程 08 Step 7）：同一次校验 → 记账 → 生成实体对。
pub fn place_tower_on_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    map_layout: Res<MapLayout>,
    mut placed: ResMut<PlacedTowers>,
    atlas: Res<TowerAtlas>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let (cam, cam_transform) = *camera;

    let Some(tile) = hovered_placeable_tile(&window, cam, cam_transform, &map_layout, &placed)
    else {
        return;
    };

    placed.0.insert(tile);
    spawn_tower_entities(
        &mut commands,
        &atlas,
        tile,
        map_layout.width as f32,
        map_layout.height as f32,
    );
}

/// 索敌 + 旋转 + 冷却扣血 + 就地 despawn（教程 09 Step 4 `attack_enemies`）。
///
/// 单系统完成整条链路（教程 09「Design decision: one combined system」）：
/// 快照敌人位置释放借用 → 逐塔找最近目标 → 旋转 → `just_finished` 时扣血 →
/// 血量 ≤0 就地 `despawn`（队列命令，帧末生效）+ 生成枪口焰。
pub fn attack_enemies(
    time: Res<Time>,
    atlas: Res<TowerAtlas>,
    mut turrets: Query<
        (Entity, &mut Transform, &mut AttackTimer, &Damage, &AttackRange),
        (With<TowerTurret>, Without<Enemy>),
    >,
    mut enemies: Query<
        (Entity, &Transform, &mut Health),
        (With<Enemy>, With<PathFollower>, Without<TowerTurret>),
    >,
    mut commands: Commands,
) {
    // 快照敌军位置后释放 enemies 查询借用（教程 09「Snapshot pattern」）。
    let enemy_positions: Vec<(Entity, Vec2)> = enemies
        .iter()
        .map(|(e, t, _)| (e, t.translation.truncate()))
        .collect();

    for (turret_entity, mut turret_transform, mut timer, damage, range) in turrets.iter_mut() {
        timer.0.tick(time.delta());
        let turret_pos = turret_transform.translation.truncate();

        // 线性扫描最近目标（教程 09 Step 4；小规模足够）。
        let mut nearest: Option<(Entity, f32)> = None;
        for &(entity, pos) in &enemy_positions {
            let dist = turret_pos.distance(pos);
            if dist <= range.0 && nearest.map_or(true, |(_, best)| dist < best) {
                nearest = Some((entity, dist));
            }
        }

        let Some((target, _)) = nearest else {
            continue;
        };

        // 持续朝目标旋转（炮塔贴图朝上，敌人贴图朝右，故减 π/2——教程 09 Step 4）。
        let direction = enemy_positions
            .iter()
            .find(|(e, _)| *e == target)
            .map(|(_, p)| *p - turret_pos)
            .unwrap_or(Vec2::X);
        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
        turret_transform.rotation = Quat::from_rotation_z(angle);

        // 冷却到期：扣血、死亡就地回收、枪口焰。
        if timer.0.just_finished() {
            if let Ok((_, _, mut health)) = enemies.get_mut(target) {
                health.0 -= damage.0;
                if health.0 <= 0.0 {
                    commands.entity(target).despawn();
                }
            }

            // 枪口焰：父实体带 DespawnTimer，两个火苗 sprite 作子实体（教程 09 Step 4）。
            // 教程 12 Step 3：父子同挂 GameEntity——清场时父实体递归 despawn 会让
            // 子实体的 despawn 命令失败，故 cleanup_level 用 queue_silenced（教程 12 Step 6）。
            let texture = atlas.texture.clone();
            let layout = atlas.layout.clone();
            let mut flash_id = None;
            commands.entity(turret_entity).with_children(|turret_children| {
                flash_id = Some(
                    turret_children
                        .spawn((
                            MuzzleFlash,
                            GameEntity,
                            DespawnTimer(Timer::from_seconds(
                                MUZZLE_FLASH_DURATION,
                                TimerMode::Once,
                            )),
                            // 父实体须带 Transform/Visibility，子渲染传播才成立（教程 09 注）。
                            Transform::default(),
                            Visibility::default(),
                        ))
                        .id(),
                );
            });
            if let Some(flash_id) = flash_id {
                commands.entity(flash_id).with_children(|flash_children| {
                    for i in 0..2 {
                        let x_offset = if i == 0 { -6.0 } else { 6.0 };
                        flash_children.spawn((
                            GameEntity,
                            Sprite::from_atlas_image(
                                texture.clone(),
                                TextureAtlas {
                                    layout: layout.clone(),
                                    index: FIRE_SPRITE,
                                },
                            ),
                            Transform::from_xyz(x_offset, 32.0, 2.2),
                        ));
                    }
                });
            }
        }
    }
}

/// 回收到期特效实体（教程 09 Step 5 `despawn_timed`）：通用、与特效种类无关。
pub fn despawn_timed(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DespawnTimer)>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

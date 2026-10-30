//! game-td —— M6 塔防（T047-W1）：外部教程 Part 4–7 的**行为**复刻。
//!
//! 教程（JerryAZR/bevy-tower-defense，MIT）是行为规格、**非实现来源**：其代码为
//! bevy 0.18.1 形态，本工程每个 API 均按 bevy 0.20.0 / bevy_ecs_tilemap 0.20.0
//! 本地源码查证后重写（锚点见使用处注释；对照底稿
//! `docs/evidence/t044-td-spike/w2/api-drift.md` Part 04 节）。
//!
//! 阶段：
//! - P1：App + Camera2d + 红方块 Sprite；
//! - P2：tilesheet 图集 + 15×10 逐格 Sprite 网格 + `MapTile`/`PathTile` 标记；
//! - P3：`bevy_ecs_tilemap` 单 draw call 等价替换（`TileBundle` + `TilemapBundle`）；
//! - P4：硬编码 `match` → **auto-tiling**——逻辑层 `TileType` + `MapLayout`
//!   资源、规则书 `TileRules`，视觉索引在 spawn 时按**邻域**解析（教程 Part 4
//!   Step 1–4）。道路形态与图集索引与 P3 逐格一致（教程 04「What stayed the same」）；
//! - P5：按教程 05 拆分为 `map.rs`（地图数据）/ `tiling.rs`
//!   （auto-tiling 规则）/ 本文件（薄接线层）——**纯结构重构、行为零漂移**
//!   （教程 05「moved verbatim; no behavior changed」）；
//! - P6：按教程 06 关卡数据外置——`level.rs` 读
//!   `assets/levels/level_01.toml`（`[map]` + `[paths.<id>].waypoints` 宽 1 中心线），
//!   加载时展开为宽 3 道路并补齐拐角对角格；`build_demo_map()` 退役。
//!   **默认关卡与 P5 行为零变化**（waypoints `[[0,5],[14,5]]` ≡ 原 3 行横向道路）；
//! - P7（本文件所在阶段）：按教程 07 加敌人寻路——`enemy.rs`（`Enemy` /
//!   `PathFollower` / `MoveSpeed` + 位置式移动 + 终末回收）。Startup 拆为
//!   `(load_level_data → spawn_tilemap → spawn_test_enemy).chain()`，移动挂
//!   `FixedUpdate`；单测试敌自 waypoint[0] 出发朝 waypoint[1]（+x）。
//!
//! 窗口契约（Lead 预置，覆盖教程默认窗口）：1280×720、Windowed、title "game-td"。
//! BRP 常驻见 [`brp`]：`127.0.0.1:15702`（显式回环绑定）+ 唯一自定义方法 `td.screenshot`。

mod brp;
mod enemy;
mod level;
mod map;
mod tiling;

use bevy::prelude::*;
// WindowMode 不在 prelude，经 bevy::window 引入（bevy_window-0.20.0/src/window.rs:1338）。
use bevy::window::WindowMode;
use bevy_ecs_tilemap::prelude::*;

use enemy::{cleanup_finished_enemies, move_enemies, Enemy, MoveSpeed, PathFollower};
use level::{build_map_from_level, load_level, LevelData};
use map::{MapLayout, MapTile, PathTile, TileType};
use tiling::{build_rules, TileRules};

/// 单格像素尺寸（教程 Part 2 Step 3 行为规格：64px 格）。
const TILE_SIZE: f32 = 64.0;

/// 默认关卡文件（教程 06 Step 6：路径相对进程 CWD = `game-td/`）。
const DEFAULT_LEVEL_PATH: &str = "assets/levels/level_01.toml";

fn main() -> AppExit {
    App::new()
        // 窗口契约：1280×720 窗口化 + title "game-td"。
        // 依据: `bevy-0.20.0/examples/window/window_settings.rs` 与主仓
        // game/src/main.rs:39-49（0.20 已过编译，核实 2026-10-29）。
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "game-td".into(),
                        resolution: (1280, 720).into(),
                        mode: WindowMode::Windowed,
                        ..default()
                    }),
                    ..default()
                })
                // 像素美术取样：64px 瓦片在 DPI 1.25 下放大，线性过滤会糊边。
                // 依据: `ImagePlugin::default_nearest()` `bevy_image-0.20.0/src/image.rs:209`；
                // tilemap 采样器取自本插件默认值
                // （`bevy_ecs_tilemap-0.20.0/src/render/mod.rs:153-156`）。
                .set(ImagePlugin::default_nearest()),
        )
        // tilemap 渲染插件（教程 Part 3 Step 2）：`TilemapPlugin` 0.20.0/src/lib.rs:62。
        .add_plugins(TilemapPlugin)
        .add_plugins(brp::BrpPlugin)
        // 教程 07 Step 1/7：数据 → 渲染 → 敌人的启动链（.chain() 保证次序：
        // 后段系统读前段插入的资源）。
        .add_systems(Startup, (load_level_data, spawn_tilemap, spawn_test_enemy).chain())
        // 教程 07 Step 7：移动与回收挂固定步长调度（与帧率解耦）。
        .add_systems(FixedUpdate, (move_enemies, cleanup_finished_enemies))
        .run()
}

/// 关卡数据装载（教程 07 Step 1 `load_level_data`）：纯数据、无渲染依赖，
/// 把 `MapLayout` / `TileRules` / `LevelData` 三个资源先放进 ECS。
fn load_level_data(mut commands: Commands) {
    let level = load_level(DEFAULT_LEVEL_PATH);
    let map = build_map_from_level(&level);
    let rules = build_rules();

    commands.insert_resource(map);
    commands.insert_resource(rules);
    commands.insert_resource(level);
}

/// P4/P5/P6/P7：按 `MapLayout` + `TileRules` 解析视觉索引并建 tilemap
/// （教程 Part 4 Step 4 + 06 Step 6 + 07 Step 1 `spawn_tilemap`）。
///
/// 顺序契约（教程 Part 3 Step 3–5 + 04 Step 4）：先拿父实体 → 逐格 spawn
/// `TileBundle` 并填 `TileStorage` → 最后给父实体 insert `TilemapBundle`。
/// 0.20 锚点同 P3（`TileBundle` `tiles/mod.rs:114-124`、`TileStorage::empty`/`set`
/// `tiles/storage.rs:32`/`:65`、`TilemapAnchor::Center` `anchor.rs:13`）。
///
/// 教程 05 Step 3 / 06 Step 6 / 07 Step 1：spawn 循环与 P4 **逐字相同**
/// （重构 / 数据外置 / 启动拆分都不动接线）；P7 只是把数据来源从系统内
/// `load_level` 换成资源读取。
fn spawn_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map: Res<MapLayout>,
    rules: Res<TileRules>,
) {
    commands.spawn(Camera2d);

    // tilesheet：1472×832 PNG = 23 列 × 13 行 × 64px（教程 Part 2 Step 1 规格）。
    let texture_handle: Handle<Image> = asset_server.load("Tilesheet/towerDefense_tilesheet.png");

    let map_size = TilemapSize {
        x: map.width,
        y: map.height,
    };
    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    let grid_size = tile_size.into();

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map.width {
        for y in 0..map.height {
            let pos = TilePos { x, y };
            let tile_type = map
                .get(x as i32, y as i32)
                .expect("spawn 循环限定在 MapLayout 尺寸内");

            // 视觉索引来自邻域解析（P4 的核心替换点：不再看坐标）。
            let visual_index = rules.resolve(tile_type, pos, &map);

            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(visual_index),
                        ..Default::default()
                    },
                    tile_type,
                    MapTile,
                ))
                .id();

            if tile_type == TileType::Path {
                commands.entity(tile_entity).insert(PathTile);
            }

            tile_storage.set(&pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type: TilemapType::Square,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });
}

/// 单测试敌（教程 07 Step 3 `spawn_test_enemy`）：从 `main_road` 首航点点出，
/// 立即朝第二航点走（`waypoint_index = 1`）。
///
/// 行为规格（教程 07 Step 3）：
/// - sprite = 同一 Kenney 图集、index 245、图集网格 64px × 23 列 × 13 行；
/// - `z = 1.0` 压在地图（z ≈ 0）之上；
/// - `MoveSpeed(192.0)` = 3 格/秒 × 64px/格；
/// - 位置 / 目标用与 [`enemy::tile_to_world`] 同式的居中换算（教程 Step 3 内联式，
///   Step 4 把同式抽成 helper 供塔复用）。
fn spawn_test_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    level: Res<LevelData>,
) {
    let texture = asset_server.load("Tilesheet/towerDefense_tilesheet.png");
    // 0.20 锚点：`from_grid(tile_size, columns, rows, padding, offset)`
    // `bevy_image-0.20.0/src/texture_atlas.rs:133-139`。
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 23, 13, None, None);
    let atlas_layout = texture_atlas_layouts.add(layout);

    let waypoints = &level.paths["main_road"].waypoints;
    let spawn_tile = waypoints[0];
    let target_tile = waypoints[1];

    let tile_size = 64.0;
    let map_width = level.map.width as f32;
    let map_height = level.map.height as f32;
    let origin_x = -map_width * tile_size / 2.0 + tile_size / 2.0;
    let origin_y = -map_height * tile_size / 2.0 + tile_size / 2.0;

    let x = origin_x + spawn_tile[0] as f32 * tile_size;
    let y = origin_y + spawn_tile[1] as f32 * tile_size;
    let target = Vec2::new(
        origin_x + target_tile[0] as f32 * tile_size,
        origin_y + target_tile[1] as f32 * tile_size,
    );

    commands.spawn((
        // 0.20 锚点：`Sprite::from_atlas_image(image, atlas)`
        // `bevy_sprite-0.20.0/src/sprite.rs:64`；`TextureAtlas { layout, index }`
        // `bevy_image-0.20.0/src/texture_atlas.rs:214-219`。
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: atlas_layout,
                index: 245,
            },
        ),
        Transform::from_xyz(x, y, 1.0),
        Enemy,
        PathFollower {
            path_id: "main_road".to_string(),
            waypoint_index: 1,
            target,
        },
        MoveSpeed(192.0),
    ));
}

//! game-td —— M6 塔防 spike（T044-W1）：外部教程 Part 1→2→3 的**行为**复刻。
//!
//! 教程（JerryAZR/bevy-tower-defense，MIT）是行为规格、**非实现来源**：其代码为
//! bevy 0.18.1 形态，本工程每个 API 均按 bevy 0.20.0 本地源码查证后重写
//! （锚点见各使用处注释与证据档 `docs/evidence/t044-td-spike/w1/` 的查证台账）。
//!
//! 阶段（最终提交态 = P3）：
//! - P1：App + DefaultPlugins + Startup setup；Camera2d ×1 + 红色方块 Sprite ×1
//!   （`Color::srgb(1.0, 0.0, 0.0)`、100×100、世界原点）——证据 `v4-assert-p1.log`；
//! - P2：移除红方块，载入 tilesheet 建 `TextureAtlasLayout`（64×64、23 列×13 行），
//!   生成 15 列×10 行网格（居中于原点）并挂 `MapTile` / `PathTile` 标记
//!   ——证据 `v5-assert-p2.log`；
//! - P3（当前）：以 `bevy_ecs_tilemap` 单 draw call 等价替换 P2 的逐格 Sprite
//!   （`TileBundle` + 父实体 `TilemapBundle` + `TileStorage`；标记保持、索引不变）
//!   ——证据 `v6-assert-p3.log`。
//!
//! 窗口契约（Lead 预置，覆盖教程默认窗口）：1280×720、Windowed、title "game-td"
//! （对齐主仓 `game/src/main.rs` 的 WindowPlugin 形态）。
//!
//! BRP 常驻见 [`brp`]：`127.0.0.1:15702`（显式回环绑定）+ 唯一自定义方法
//! `td.screenshot`。

mod brp;

use bevy::prelude::*;
// WindowMode 不在 prelude，经 bevy::window 引入（依据: 主仓 game/src/main.rs 同形态；
// bevy_window-0.20.0/src/window.rs:1338 的 WindowMode 枚举，核实 2026-10-29）。
use bevy::window::WindowMode;
use bevy_ecs_tilemap::prelude::*;

/// 网格地图瓦片标记（教程 Part 2 Step 4）：所有 150 格都挂。
/// derive 形态按 SKILL.md §3.4：BRP 查询要求 Reflect + `#[reflect(Component)]`，
/// 并在 [`brp::BrpPlugin`] 的 build 中显式 `register_type`。
#[derive(Component, Reflect)]
#[reflect(Component)]
struct MapTile;

/// 道路瓦片标记（教程 Part 2 Step 4）：`tile_index != 129` 的格子加挂。
#[derive(Component, Reflect)]
#[reflect(Component)]
struct PathTile;

/// 地图尺寸常量（教程 Part 2 Step 3 的行为规格：15 列 × 10 行、64px 格）。
const COLS: u32 = 15;
const ROWS: u32 = 10;
const TILE_SIZE: f32 = 64.0;

fn main() -> AppExit {
    App::new()
        // 窗口契约：1280×720 窗口化 + title "game-td"。
        // 依据: WindowPlugin/Window 形态 `bevy-0.20.0/examples/window/window_settings.rs`
        // 与主仓 game/src/main.rs:39-49（0.20 已过编译，核实 2026-10-29）。
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
                // 像素美术取样：逻辑 1280×720 在 DPI 1.25 下为物理 1600×900，
                // 64px 瓦片被放大 1.25×——线性过滤会糊边，故 nearest。
                // 依据: `ImagePlugin::default_nearest()` `bevy_image-0.20.0/src/image.rs:209`；
                // tilemap 插件的采样器在 `TilemapRenderingPlugin::finish` 取本插件默认值
                // （`bevy_ecs_tilemap-0.20.0/src/render/mod.rs:153-156`）（核实 2026-10-29）。
                .set(ImagePlugin::default_nearest()),
        )
        // tilemap 渲染插件（教程 Part 3 Step 2）：注册渲染管线/着色器/抽取系统。
        // 依据: `TilemapPlugin` `bevy_ecs_tilemap-0.20.0/src/lib.rs:62`（Plugin impl :64）。
        .add_plugins(TilemapPlugin)
        .add_plugins(brp::BrpPlugin)
        .add_systems(Startup, setup)
        .run()
}

/// P3：tilemap 单实体承载 15×10 网格（教程 Part 3 Step 3–5 的行为规格）。
///
/// 顺序契约（教程 Step 3 注释 + 源码）：先 spawn 空的父实体（拿 `Entity` 作
/// `TilemapId`）→ 逐格 spawn `TileBundle` 子实体并填 `TileStorage` → 最后给父实体
/// insert `TilemapBundle`。
///
/// 依据: `TileBundle { position, texture_index, tilemap_id, ..Default }`
/// `bevy_ecs_tilemap-0.20.0/src/tiles/mod.rs:114-124`；`TilePos { x, y }`
/// `tiles/mod.rs:17-20`；`TileTextureIndex(pub u32)` `tiles/mod.rs:73`；
/// `TilemapId(pub Entity)` `src/map.rs:55`；`TileStorage::empty`/`set`
/// `tiles/storage.rs:32` / `:65`；`TilemapSize` `src/map.rs:72`；
/// `TilemapTileSize` `src/map.rs:214`；`TilemapTexture::Single(Handle<Image>)`
/// `src/map.rs:126`；`TilemapType::Square` `src/map.rs:464-467`；
/// `TilemapAnchor::Center` `src/anchor.rs:13`（居中算式本体 `src/anchor.rs:43`
/// `as_offset`、使用点 `render/prepare.rs:161`）；
/// `TilemapBundle` 别名 `src/lib.rs:115`（`MaterialTilemapBundle<StandardTilemapMaterial>`，
/// 默认材质由 `TilemapRenderingPlugin` 注入 `src/render/mod.rs:122` + `:127-130`）。
/// 非 atlas 默认 feature 下 tilesheet 按 `tile_size` 切图集数组（层序 = 行主序，
/// `render/texture_array_cache.rs:275-315`）——与 P2 的 `TextureAtlasLayout` 索引一致。
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // tilesheet：1472×832 PNG = 23 列 × 13 行 × 64px（教程 Part 2 Step 1 规格）。
    let texture_handle: Handle<Image> = asset_server.load("Tilesheet/towerDefense_tilesheet.png");

    let map_size = TilemapSize { x: COLS, y: ROWS };
    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);
    let path_mid = map_size.y / 2; // 10/2 = 5

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };

            // 三行道路（行 4/5/6，Bevy 2D 为 Y-up：path_mid+1 视觉在上）：
            // 行4 下沿 125/126/127；行5 路面 102/103/104；行6 上沿 79/80/81；其余草 129
            // （与 P2 完全相同的索引集）。
            let tile_index = match (y, x) {
                (r, 0) if r == path_mid - 1 => 125,                  // 左下角
                (r, c) if r == path_mid - 1 && c == COLS - 1 => 127, // 右下角
                (r, _) if r == path_mid - 1 => 126,                  // 下沿
                (r, 0) if r == path_mid => 102,                      // 左端
                (r, c) if r == path_mid && c == COLS - 1 => 104,     // 右端
                (r, _) if r == path_mid => 103,                      // 路面
                (r, 0) if r == path_mid + 1 => 79,                   // 左上角
                (r, c) if r == path_mid + 1 && c == COLS - 1 => 81,  // 右上角
                (r, _) if r == path_mid + 1 => 80,                   // 上沿
                _ => 129,                                            // 草
            };

            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(tile_index),
                        ..Default::default()
                    },
                    MapTile,
                ))
                .id();

            if tile_index != 129 {
                commands.entity(tile_entity).insert(PathTile);
            }

            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        anchor: TilemapAnchor::Center,
        ..Default::default()
    });
}

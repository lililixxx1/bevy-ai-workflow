//! game-td —— M6 塔防（T047-W1 / T048-W1）：外部教程 Part 4–12 的**行为**复刻。
//!
//! 教程（JerryAZR/bevy-tower-defense，MIT）是行为规格、**非实现来源**：其代码为
//! bevy 0.18.1 形态，本工程每个 API 均按 bevy 0.20.0 / bevy_ecs_tilemap 0.20.0
//! 本地源码查证后重写（锚点见使用处注释；对照底稿
//! `docs/evidence/t044-td-spike/w2/api-drift.md` Part 04–12 节）。
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
//! - P7：按教程 07 加敌人寻路——`enemy.rs`（`Enemy` / `PathFollower` /
//!   `MoveSpeed` + 位置式移动 + 终末回收）；单测试敌自 waypoint[0] 出发朝
//!   waypoint[1]（+x）；
//! - P8：按教程 08 加塔放置——`tower.rs`（`Tower` / `TowerTurret` /
//!   `TowerPreview` + `PlacedTowers` / `TowerAtlas`）；
//!   光标悬停预览 + 左键放置挂 `Update`（输入响应），玩法系统留 `FixedUpdate`。
//!   BRP 测试通道 `td.spawn_tower`（无鼠标注入面，放置需确定性入口）；
//! - P9：按教程 09 加索敌 / 伤害 / 死亡回收——敌人 `Health(100)`、
//!   炮塔 `AttackRange(192)` / `Damage(34)` / `AttackTimer(0.5s)`；
//!   `attack_enemies` 挂 `FixedUpdate` 且排在移动之后、回收之前；
//!   `despawn_timed` 清枪口焰（`Update`）；
//! - P10：按教程 10 把单测试敌换成数据驱动波次——
//!   `level.rs` 读 `[enemy_types.*]` / `[[waves]]`，`enemy.rs::build_spawn_schedule`
//!   装载期展开为按时间排序的 `VecDeque<SpawnEvent>`，`spawn_wave_enemies` 每固定步
//!   弹出到期事件（排在移动之前）；`spawn_test_enemy` 退役；
//! - P11：按教程 11 加胜负判定——`BaseLives(5)`、`process_base_reachers`
//!   （终点回收 + 扣命）、`GameOver` 标记资源与 `check_game_state`；
//! - P12（本文件所在阶段）：按教程 12 重构为**状态机**（`GameState`：
//!   LevelSelect → InGame → GameOver → LevelSelect）——关卡装载/卸载挂
//!   `OnEnter/OnExit(InGame)`，玩法系统以 `.run_if(in_state(InGame))` 门控，
//!   `GameEntity` 统一标记 + `cleanup_level` 一次清场，`level_select.rs` /
//!   `game_over.rs` 两个屏内 UI 模块；`Local<bool>` 门闩换成随关卡回收的
//!   `GameFinished` 资源（教程 12 Step 4），`GameOver` 标记退役。
//!   BRP 测试通道 `td.state`（状态读写）以覆盖无键鼠注入的状态迁移；
//! - P13（本文件所在阶段）：按教程 13 加**多关卡**——`assets/levels/level_01..03.toml`
//!   自动发现（`AvailableLevels` / `SelectedLevel` 资源 + `scan_available_levels`），
//!   菜单动态列表 + `1..9` 键映射，`load_level_data` 改读选中路径；
//!   `setup_tower_atlas` 移到 `Startup`（教程 13 Step 7：只装一次）。
//!   BRP 测试通道 `td.level`（关卡列表读取 / 选中项切换）以覆盖无键盘注入的关卡选择；
//! - P14–P17（同批推进，逐部过门禁）：经济（`economy.rs`：金币 / 被动收入 / 赏金 /
//!   建造价 / HUD）、火箭发射器与弹道链（`tower.rs`：弹药 / 追踪弹 / 溅射）、
//!   **数据驱动塔注册表**（`assets/towers.toml` + `TowerRegistry`，教程 16）、
//!   塔选择 dock（教程 17：滚轮 / 数字键 / 槽位点击，`Interaction` 弃用别名按 A'
//!   裁决以 `#[expect(deprecated)]` 落地）。对应测试通道 `td.economy` /
//!   `td.combat` / `td.towers` / `td.tower_select`。
//!
//! 窗口契约（Lead 预置，覆盖教程默认窗口）：1280×720、Windowed、title "game-td"。
//! BRP 常驻见 [`brp`]：`127.0.0.1:15702`（显式回环绑定）+ `td.*` 自定义方法族。

mod brp;
mod economy;
mod enemy;
mod game_over;
mod level;
mod level_select;
mod map;
mod tiling;
mod tower;

use bevy::ecs::system::entity_command::despawn;
use bevy::prelude::*;
// WindowMode 不在 prelude，经 bevy::window 引入（bevy_window-0.20.0/src/window.rs:1338）。
use bevy::window::WindowMode;
use bevy_ecs_tilemap::prelude::*;

use economy::{
    earn_passive_income, spawn_gold_hud, tick_placement_denied, update_gold_hud, Gold,
    STARTING_GOLD,
};
use enemy::{
    build_spawn_schedule, check_game_state, move_enemies, process_base_reachers,
    spawn_wave_enemies, BaseLives, GameFinished,
};
use game_over::{handle_game_over_input, setup_game_over};
use level::{build_map_from_level, load_level, LevelData};
use level_select::{
    handle_level_select_input, scan_available_levels, setup_level_select, AvailableLevels,
    SelectedLevel,
};
use map::{MapLayout, MapTile, PathTile, TileType};
use tiling::{build_rules, TileRules};
use tower::{
    attack_enemies, cycle_tower_on_scroll, despawn_timed, explode_projectiles,
    handle_dock_slot_click, launch_rockets, load_tower_registry, move_projectiles,
    place_tower_on_click, refill_ammo, select_tower_by_key, setup_tower_atlas, setup_tower_dock,
    spawn_placement_preview, update_dock_selection, update_placement_preview, PlacedTowers,
};

/// 单格像素尺寸（教程 Part 2 Step 3 行为规格：64px 格）。
const TILE_SIZE: f32 = 64.0;

/// 游戏状态机（教程 12 Step 1）：三态互斥，`#[default]` = 启动态。
///
/// derive 集是 Bevy 状态机要求（教程 12「Pitfall」：缺 Default 编译错、缺 Hash/Eq
/// 得到深层的 trait-bound 错）。`States` 派生自 `bevy_state::prelude`
/// （`bevy_state-0.20.0/src/lib.rs:77` 起；0.20 抽核维持）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum GameState {
    #[default]
    LevelSelect,
    InGame,
    GameOver,
}

/// 胜负结果（教程 12 Step 5）：`check_game_state` 写、GameOver 屏读。
///
/// `cleanup_level` 刻意**不**移除它——`OnEnter(GameOver)` 要读（教程 12 Step 9
/// 「Critical ordering detail」）。
#[derive(Resource)]
pub enum GameResult {
    Victory,
    Defeat,
}

/// 玩法实体统一标记（教程 12 Step 2）：`cleanup_level` 一条查询全清。
///
/// 瓦片 / 敌人 / 塔 / 预览 / 枪口焰全部挂它；漏挂即在状态切换时泄漏
/// （教程 12 Step 3「Pitfall」）。
#[derive(Component)]
pub struct GameEntity;

/// 屏内 UI 标记（教程 12 Step 2）：菜单 / 结算屏离场时清理，不进 `GameEntity` 面。
#[derive(Component)]
struct ScreenUi;

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
        // 教程 12 Step 1：注册状态枚举 + 初始值（`#[default]` = LevelSelect）+
        // `NextState` 资源；首个 update 触发 `OnEnter(LevelSelect)` 恰一次。
        .init_state::<GameState>()
        // 教程 13 Step 1/8：发现列表空表 + 选中项占位（首个发现项由 scan 写入）。
        .init_resource::<AvailableLevels>()
        .init_resource::<SelectedLevel>()
        // 教程 12 Step 7 + 13 Step 7：相机与塔图集都在 Startup 一次
        // （P13 起图集不再随菜单进出重建——教程 13 Step 7「run exactly once」）。
        // 教程 16：塔注册表 Startup 一次载入（`assets/towers.toml`）。
        .add_systems(
            Startup,
            (spawn_camera, setup_tower_atlas, load_tower_registry).chain(),
        )
        // ---------- LevelSelect ----------
        // 教程 13 Step 8：先扫描关卡、再按发现列表建 UI（顺序倒置 UI 读到空表）。
        .add_systems(
            OnEnter(GameState::LevelSelect),
            (scan_available_levels, setup_level_select).chain(),
        )
        .add_systems(OnExit(GameState::LevelSelect), cleanup_screen_ui)
        .add_systems(
            Update,
            handle_level_select_input.run_if(in_state(GameState::LevelSelect)),
        )
        // ---------- InGame ----------
        // 教程 12 Step 7/10：数据 → 刷怪调度 → 渲染 → 预览幽灵，逐回合重装。
        .add_systems(
            OnEnter(GameState::InGame),
            (
                load_level_data,
                setup_spawn_schedule,
                spawn_tilemap,
                spawn_placement_preview,
                // 教程 14 Step 10：HUD 链尾——读 `load_level_data` 插入的 `Gold`。
                spawn_gold_hud,
                // 教程 17：塔选 dock（底部槽位；读注册表 + 重置选中下标）。
                setup_tower_dock,
            )
                .chain(),
        )
        .add_systems(OnExit(GameState::InGame), cleanup_level)
        // 教程 12 Step 10：玩法固定步长链仅在 InGame 运行（GameOver 时冻结）。
        .add_systems(
            FixedUpdate,
            (
                spawn_wave_enemies,
                move_enemies,
                // 教程 15 Step 10：`attack_enemies` 保留——场上无 `InstantShooter`
                // 实体时是 no-op（P16 数据驱动后即时射击判别器 = `InstantShooter`；
                // 火箭系走弹道链不挂它）。
                attack_enemies,
                // 弹道链（教程 15 Step 10 定序）：补弹 → 发射 → 飞行 → 命中结算。
                refill_ammo,
                launch_rockets,
                move_projectiles,
                explode_projectiles,
                process_base_reachers,
                check_game_state,
                // 教程 14 Step 10：收入挂固定步（随模拟时间缩放）。
                earn_passive_income,
            )
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        // 教程 17 Step「Organizing the systems」：输入类与视觉类分两组注册
        // （纯组织性拆分，无行为变化）。
        .add_systems(
            Update,
            (
                cycle_tower_on_scroll,
                select_tower_by_key,
                handle_dock_slot_click,
            )
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            (
                update_placement_preview,
                update_dock_selection,
                update_gold_hud,
                tick_placement_denied,
                place_tower_on_click,
                despawn_timed,
            )
                .run_if(in_state(GameState::InGame)),
        )
        // ---------- GameOver ----------
        .add_systems(OnEnter(GameState::GameOver), setup_game_over)
        .add_systems(OnExit(GameState::GameOver), cleanup_screen_ui)
        .add_systems(
            Update,
            handle_game_over_input.run_if(in_state(GameState::GameOver)),
        )
        .run()
}

/// 相机（教程 12 Step 7 `spawn_camera`）：Startup 一次，跨菜单 / 玩法 / 结算屏持久。
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 关卡数据装载（教程 12 Step 7 + 13 Step 2）：每次进入 InGame 都重装——
/// 关卡路径读 `SelectedLevel`（教程 13：由 `scan_available_levels` / 菜单输入 /
/// `td.level` 写入），含 `cleanup_level` 会移除的 `PlacedTowers` / `BaseLives`
/// （教程 12 Step 7 注：它们从 main.rs 移到这里，因为每回合都要重建）。
fn load_level_data(mut commands: Commands, selected: Res<SelectedLevel>) {
    let level = load_level(&selected.0);
    let map = build_map_from_level(&level);
    let rules = build_rules();

    commands.insert_resource(map);
    commands.insert_resource(rules);
    commands.insert_resource(level);
    // 教程 11 Step 1 + 12 Step 7：基地 5 命（`insert_resource` 而非 `init_resource`——
    // `i32::default()` 是 0，且不同关卡可有不同起始命数）。
    commands.insert_resource(BaseLives(5));
    // 教程 14 Step 4：每关开局 300 金（`insert_resource` 覆盖 = 干净起点）。
    commands.insert_resource(Gold(STARTING_GOLD));
    // 教程 08 Step 8 + 12 Step 7：占格集合逐回合重建。
    commands.insert_resource(PlacedTowers::default());
}

/// 构建并插入刷怪调度（教程 10 Step 6 `setup_spawn_schedule`）：
/// 装载期一次性展开全部波次（`build_spawn_schedule`），运行期只弹队列。
fn setup_spawn_schedule(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    level: Res<LevelData>,
) {
    let schedule = build_spawn_schedule(&level, &asset_server, &mut texture_atlas_layouts);
    info!(
        "[TD] spawn schedule built: {} events ({} waves, {} enemy types)",
        schedule.events.len(),
        level.waves.len(),
        level.enemy_types.len()
    );
    commands.insert_resource(schedule);
}

/// P4/P5/P6/P7/P12：按 `MapLayout` + `TileRules` 解析视觉索引并建 tilemap
/// （教程 Part 4 Step 4 + 06 Step 6 + 07 Step 1 `spawn_tilemap`）。
///
/// 顺序契约（教程 Part 3 Step 3–5 + 04 Step 4）：先拿父实体 → 逐格 spawn
/// `TileBundle` 并填 `TileStorage` → 最后给父实体 insert `TilemapBundle`。
/// 0.20 锚点同 P3（`TileBundle` `tiles/mod.rs:114-124`、`TileStorage::empty`/`set`
/// `tiles/storage.rs:32`/`:65`、`TilemapAnchor::Center` `anchor.rs:13`）。
///
/// 教程 12 Step 3：tilemap 父实体与每个瓦片实体都挂 `GameEntity`（清场用）。
fn spawn_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map: Res<MapLayout>,
    rules: Res<TileRules>,
) {
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

    let tilemap_entity = commands.spawn(GameEntity).id();
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
                    GameEntity,
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

/// 退出 InGame 时的清场（教程 12 Step 6 `cleanup_level`）：
/// ① 全部 `GameEntity` 实体（瓦片 / 敌人 / 塔 / 预览 / 枪口焰）；
/// ② 全部关卡资源（`GameResult` 刻意保留，供 `OnEnter(GameOver)` 读取）。
///
/// `queue_silenced(despawn())`（0.20 锚：`EntityCommands::queue_silenced`
/// `system/commands/mod.rs:1987`；`despawn()` 命令
/// `system/commands/entity_command.rs:278`；glob 再导出 `system/mod.rs:143`）：
/// 枪口焰子实体与父塔同挂 `GameEntity`，父实体递归 despawn 后子实体命令会失败——
/// 静默忽略正是期望态（教程 12 Step 6「Why queue_silenced?」）。
fn cleanup_level(mut commands: Commands, entities: Query<Entity, With<GameEntity>>) {
    for entity in &entities {
        commands.entity(entity).queue_silenced(despawn());
    }
    commands.remove_resource::<MapLayout>();
    commands.remove_resource::<TileRules>();
    commands.remove_resource::<LevelData>();
    commands.remove_resource::<enemy::SpawnSchedule>();
    commands.remove_resource::<PlacedTowers>();
    commands.remove_resource::<BaseLives>();
    // 教程 14 Step 5：每关干净起点（`load_level_data` 也会覆盖，显式移除表明意图）。
    commands.remove_resource::<Gold>();
    commands.remove_resource::<GameFinished>();
}

/// 屏内 UI 清场（教程 12 Step 6 `cleanup_screen_ui`）：菜单 / 结算屏各在 OnExit 调用。
fn cleanup_screen_ui(mut commands: Commands, query: Query<Entity, With<ScreenUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

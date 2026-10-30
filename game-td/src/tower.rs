//! 塔系统：放置（教程 08）+ 索敌 / 伤害（教程 09）+ 经济门（教程 14）+
//! 火箭发射器 / 弹药 / 投射物 / 溅射（教程 15）+ **数据驱动塔注册表**（教程 16）。
//!
//! 教程（0.18 形态）只是行为规格，本模块每个 API 按 bevy 0.20.0 本地源码重写
//! （对照底稿 `docs/evidence/t044-td-spike/w2/api-drift.md`）；P16 依赖的
//! `serde` / `toml` / `HashMap` 面与漂移表 Part 16「（无新 API，继承前部）」一致
//! ——该行按派工单要求逐条核过：P16 不引入任何 bevy API 新面（`Deserialize` 走
//! `serde`（P13 前已在依赖内），文件读取走 `std::fs`，`toml::from_str` 同
//! `level.rs` 既有用法），故无表外漂移。
//!
//! 设计（教程 08/09/14/15/16 Walkthrough）：
//! - 塔 = **两个实体**：`Tower` 基座（静态）+ 塔顶（独立旋转）；
//! - `TowerPreview`：常驻的两个半透明幽灵，靠 `Visibility` 开关而非 spawn/despawn；
//! - `PlacedTowers`：已占格 `HashMap<[u32; 2], Entity>`（P19 起，教程 19 重构），
//!   O(1) 放置校验（路径 / 已占用都拒绝）与 O(1) 格→塔顶实体查找（射程圈用）；
//! - `TowerAtlas`：预加载的图集句柄资源（教学「Shared handles as resources」）；
//! - P9 `attack_enemies`：单系统完成索敌 → 旋转 → 冷却到期扣血 → 就地 despawn；
//! - P15 `launch_rockets` / `move_projectiles` / `explode_projectiles` / `refill_ammo`：
//!   弹道链（追踪弹 + 三槽弹药 + 溅射 + 爆炸特效）；
//! - P16 **数据驱动**：`assets/towers.toml` 在 `Startup` 一次性载入 `TowerRegistry`
//!   （raw 结构 → 运行期结构，`projectile` 字符串键在装载期解析成
//!   `ProjectileDefinition` 克隆，教程 16「Raw vs. runtime structs」）；
//!   `SelectedTowerType` 选中注册表项（P16 硬编码键 `"rocket"`）；
//! - P17 **塔选择 UI**（教程 17）：注册表改 `Vec`（下标 = 规范塔型 ID），
//!   底部 dock（槽位 / 预览图 / 名称 / 造价 / 选中金边）、滚轮与数字键切换、
//!   槽位点击（`Interaction` 弃用别名，A' 裁决：`#[expect(deprecated)]` 落地）、
//!   预览外观随选中切换。
//!
//! **与教程 16 的一处显式偏离（验证面需要，行为零变化）**：教程把 `Tower` /
//! `TowerTurret` 等标记组件全删（用 `InstantShooter` / `AmmoState` 当判别器）。
//! 本工程保留 `Tower`（基座计数）与 `TowerTurret`（**塔顶实体**通用标记，P15 起
//! 火箭发射器炮管也挂它）两个**零尺寸验证面标记**——BRP 断言面与「塔实体指纹链」
//! （派工单 T048/T049：塔世界坐标 + 塔顶计数 + 敌血量多重集）依赖它们，且
//! `#[type_path = "game_td"]` 反射路径不能随重构漂移（SKILL §6.17）。玩法判别
//! 仍严格按教程：`InstantShooter` / `AmmoState` 决定攻击行为。
//!
//! BRP 面（本工程扩展层，非教程）：[`spawn_tower_entities`] 与 [`is_placeable`]
//! 同时被点击系统与 `td.spawn_tower` handler 复用——BRP 无鼠标注入通道，测试
//! 放置必须走确定性方法面（`td.` 前缀纪律）。
//!
//! P18（教程 18）：放置拆为**一个生产者 + 两个消费者**，以自定义消息
//! [`PlaceTower`] 解耦（`#[derive(Message)]` / `MessageWriter` / `MessageReader`）：
//! - 生产者 [`place_tower_on_click`]：只做输入校验（合法格 + 余额），写消息；
//! - 消费者 [`spawn_tower_from_event`]：读消息 → 生成塔实体；
//! - 消费者 [`deduct_gold_on_placement`]（`economy.rs`）：读消息 → 扣费。
//! 0.20 锚点（漂移表 Part 18，全部摩擦级 0）：
//! `Message` trait `bevy_ecs-0.20.0/src/message/mod.rs:100`；
//! `MessageWriter::write` `message/message_writer.rs:74`；
//! `MessageReader::read` `message/message_reader.rs:44`；
//! `App::add_message` `bevy_app-0.20.0/src/app.rs:427`；
//! `.after()` 定序 `bevy_ecs-0.20.0/src/schedule/config.rs:404`。

use std::collections::HashMap;

use bevy::audio::Volume;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use serde::Deserialize;

use crate::audio::{PlaySound, SoundType};
use crate::economy::{Bounty, Gold, PlacementDenied, DENIED_FLASH_DURATION};
use crate::enemy::{tile_to_world, Enemy, Health, PathFollower};
use crate::map::{MapLayout, TileType};
use crate::GameEntity;

/// 枪口焰 / 爆炸特效共用存续时长（教程 09 Step 3 / 15 Step 8）。
pub const MUZZLE_FLASH_DURATION: f32 = 0.15;

/// 塔注册表文件路径（教程 16；相对进程 CWD = 工程根 `<tree>/game-td`）。
pub const TOWERS_FILE: &str = "assets/towers.toml";

/// 塔基座标记（教程 08 Step 1）：`pub` 供 BRP 断言按 `game_td::Tower` 计数。
///
/// P16 偏离注记：教程删除此标记；本工程保留为**验证面标记**（见模块文档）。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub struct Tower;

/// 塔顶实体标记（教程 08 Step 1，`pub(crate)`）：P15 起兼作「塔顶」验证面标记
/// （瞬时塔炮塔与火箭发射器炮管都挂），玩法判别靠属性组件（见模块文档偏离注记）。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub(crate) struct TowerTurret;

/// 放置预览幽灵标记（教程 08 Step 1，`pub(crate)`）。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub(crate) struct TowerPreview;

/// 所有会射击的塔共有的攻击状态（教程 16「Consolidating components」）：
/// `range` + `timer` 总是一起读写，合并为一件。
#[derive(Component)]
pub(crate) struct TowerAttacker {
    pub range: f32,
    pub timer: Timer,
}

/// 瞬时伤害塔专属状态（教程 16）：伤害 + 枪口焰图集索引；**兼作判别器**
/// （`With<InstantShooter>` 找瞬时塔）。
#[derive(Component)]
pub(crate) struct InstantShooter {
    pub damage: f32,
    pub muzzle_flash_sprite: usize,
}

/// 火箭塔专属状态（教程 16）：补弹计时 + 槽位列表 + 教程外最小记账 `fired`
/// （同 `SpawnSchedule::spawned` 口径，只服务 `td.combat` 快照）；**兼作判别器**。
#[derive(Component)]
pub(crate) struct AmmoState {
    pub regen: Timer,
    pub slots: Vec<Option<Entity>>,
    pub fired: u32,
}

/// 塔型下标（教程 17 `TowerTypeId(pub usize)`）：每个已放置塔存它来自注册表的
/// 哪一项（向量下标 = 规范塔型 ID，教程 17「From string keys to integer indices」）。
///
/// 反射注册（`#[type_path]` 冻结）：断言面按 `game_td::TowerTypeId` 读放置塔的
/// 塔型下标——选择 UI 的「选中切片真的决定放置出的塔型」端到端证据。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub(crate) struct TowerTypeId(pub usize);

/// 追踪弹（教程 15 Step 2 `Projectile`）：组件内存目标 `Entity`。
#[derive(Component)]
pub(crate) struct Projectile {
    pub target: Entity,
    pub target_position: Vec2,
    pub speed: f32,
    pub damage: f32,
    pub splash_radius: f32,
    /// 爆炸特效图集索引（来自 `[projectiles.<id>].explosion_sprite`）。
    pub explosion_sprite: Option<usize>,
}

/// 命中标记（教程 15 Step 2 `Exploding`）：到达目标点、待结算溅射。
#[derive(Component)]
pub(crate) struct Exploding;

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

/// 已占用格（教程 19：`HashSet<[u32; 2]>` → `HashMap<[u32; 2], Entity>`）：
/// 格坐标 → 塔顶实体。射程圈系统按格 O(1) 查实体（教程 19「Refactoring PlacedTowers」），
/// 不必遍历全部塔按距离匹配。
///
/// 写入方随 P19 迁移：占格记账从经济消费者移到生成消费者（教程 19：实体由生成方
/// 建立，格→实体关系也归它）；`td.spawn_tower` 直通道同样在该处入表。
#[derive(Resource, Default)]
pub struct PlacedTowers(pub HashMap<[u32; 2], Entity>);

// ---------------------------------------------------------------------------
// P16：数据驱动塔注册表（教程 16）
// ---------------------------------------------------------------------------

/// 原始注册表（`serde` 直接反序列化形态，教程 16「Raw vs. runtime structs」）。
#[derive(Debug, Deserialize)]
struct TowerRegistryRaw {
    towers: HashMap<String, TowerDefinitionRaw>,
    #[serde(default)]
    projectiles: HashMap<String, ProjectileDefinitionRaw>,
}

/// 单塔原始定义（字段与 TOML 一一对应）。
#[derive(Debug, Deserialize, Clone)]
struct TowerDefinitionRaw {
    name: String,
    description: String,
    base_sprite: usize,
    top_sprite: usize,
    preview_top_sprite: usize,
    cost: u32,
    attack_range: f32,
    attack_cooldown: f32,
    damage: Option<f32>,
    muzzle_flash_sprite: Option<usize>,
    projectile: Option<String>,
    ammo_slot_offsets: Option<Vec<[f32; 2]>>,
    ammo_refill_secs: Option<f32>,
}

/// 单弹体原始定义。
#[derive(Debug, Deserialize, Clone)]
struct ProjectileDefinitionRaw {
    damage: f32,
    speed: f32,
    sprite: usize,
    explosion_sprite: Option<usize>,
    splash_radius: Option<f32>,
}

/// 运行期塔定义（教程 16）：`projectile` 已在装载期解析为定义克隆。
///
/// `key` 是教程 16 HashMap 键在 P17 向量化后的显式保留（教程 17 只留向量下标；
/// 本工程保留键串供 `td.towers` 快照与选择 UI 按键定位——断言面需要稳定标识）。
#[derive(Debug, Clone)]
pub struct TowerDefinition {
    pub key: String,
    pub name: String,
    pub description: String,
    pub base_sprite: usize,
    pub top_sprite: usize,
    pub preview_top_sprite: usize,
    pub cost: u32,
    pub attack_range: f32,
    pub attack_cooldown: f32,
    pub damage: Option<f32>,
    pub muzzle_flash_sprite: Option<usize>,
    pub projectile: Option<ProjectileDefinition>,
    pub ammo_slot_offsets: Option<Vec<[f32; 2]>>,
    pub ammo_refill_secs: Option<f32>,
}

/// 运行期弹体定义（教程 16）。
#[derive(Debug, Clone)]
pub struct ProjectileDefinition {
    pub damage: f32,
    pub speed: f32,
    pub sprite: usize,
    pub explosion_sprite: Option<usize>,
    pub splash_radius: Option<f32>,
}

/// 塔注册表资源（教程 16 HashMap → 教程 17 **向量**）：
/// 向量下标即规范塔型 ID（教程 17「`TowerRegistry` is now a `Vec<TowerDefinition>`」），
/// 下标在会话内稳定（TOML 键序经 HashMap 读入后一次性落到向量）。
#[derive(Resource)]
pub struct TowerRegistry {
    pub towers: Vec<TowerDefinition>,
}

impl TowerRegistry {
    /// 按下标取定义；越界即 panic（教程 16「No validation」口径：配置/下标错误
    /// 显式炸优于静默回落）。
    pub fn get(&self, index: usize) -> &TowerDefinition {
        self.towers
            .get(index)
            .unwrap_or_else(|| panic!("unknown tower index {index}"))
    }

    /// 按键找下标（P17 选择 UI / BRP 的键↔下标换算；找不到返回 `None`）。
    pub fn index_of_key(&self, key: &str) -> Option<usize> {
        self.towers.iter().position(|def| def.key == key)
    }
}

/// 当前选中的塔型下标（教程 17 `SelectedTowerType(pub usize)`）：玩家选择就是一个
/// 数字；`setup_tower_dock` 在每次进关时重置为 0（教程 17 `is_changed()` Pitfall 修法）。
#[derive(Resource)]
pub struct SelectedTowerType(pub usize);

// ---------------------------------------------------------------------------
// P18：自定义消息（教程 18 `PlaceTower`）——生产者 → 两个消费者的广播载荷
// ---------------------------------------------------------------------------

/// 塔放置消息（教程 18「The message type」）：载荷 = 两个消费者各自需要的最小数据。
///
/// `cost` 随消息走，经济消费者因此**不需要**再查注册表（教程 18「Why include cost」）。
/// 0.20 派生面：`#[derive(Message)]`（trait `bevy_ecs-0.20.0/src/message/mod.rs:100`；
/// 派生宏经 `bevy_ecs` prelude 导出，与教程 `use bevy::prelude::*` 形态一致）；
/// 消息类型必须 `.add_message::<PlaceTower>()`（`bevy_app-0.20.0/src/app.rs:427`），
/// 否则 `MessageWriter` / `MessageReader` 运行期报缺注册。
#[derive(Message, Clone, Debug)]
pub struct PlaceTower {
    pub tile: [u32; 2],
    pub world_pos: Vec2,
    pub tower_id: usize,
    pub cost: u32,
}

/// 消息载荷的外部可读快照（BRP `td.events` 用；`[f32; 2]` 便于 JSON 序列化）。
#[derive(Clone, Debug)]
pub struct PlaceTowerRecord {
    pub tile: [u32; 2],
    pub world_pos: [f32; 2],
    pub tower_id: usize,
    pub cost: u32,
}

impl From<&PlaceTower> for PlaceTowerRecord {
    fn from(event: &PlaceTower) -> Self {
        Self {
            tile: event.tile,
            world_pos: [event.world_pos.x, event.world_pos.y],
            tower_id: event.tower_id,
            cost: event.cost,
        }
    }
}

/// 消息链记账（**验证面**资源，非教程；同 `SpawnSchedule::spawned` / `AmmoState::fired`
/// 口径——只服务 `td.events` 快照与断言，不给玩法逻辑增加语义）。
///
/// 字段语义：
/// - `produced`：生产者 [`place_tower_on_click`] 写入的消息数（点击路径）；
/// - `injected`：BRP `td.events` `emit_place` 写入的消息数（测试通道，等价于一次点击事件）；
/// - `spawned` / `gold_applied`：两个消费者各自消费的消息数（同一消息应各 +1，
///   差值即「某个消费者没跑到」）；
/// - `spawn_last` / `gold_last`：各消费者最后消费的载荷快照（逐字段证明载荷穿越消息队列后无损）。
#[derive(Resource, Default)]
pub struct PlacementLog {
    pub produced: u32,
    pub injected: u32,
    pub spawned: u32,
    pub gold_applied: u32,
    pub spawn_last: Option<PlaceTowerRecord>,
    pub gold_last: Option<PlaceTowerRecord>,
}

/// 载入塔注册表（教程 16 `load_tower_registry` + 17 向量化，`Startup` 一次）：
/// 读 TOML → raw 结构 → 把每塔的 `projectile` 字符串键解析成 `ProjectileDefinition`
/// 克隆 → 插入 `TowerRegistry`（向量）+ 默认选中下标 0。
pub fn load_tower_registry(mut commands: Commands) {
    let raw: TowerRegistryRaw = {
        let content = std::fs::read_to_string(TOWERS_FILE)
            .unwrap_or_else(|e| panic!("Failed to read {TOWERS_FILE}: {e}"));
        toml::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {TOWERS_FILE}: {e}"))
    };

    let mut towers: Vec<TowerDefinition> = Vec::new();
    for (id, def_raw) in raw.towers {
        let projectile = def_raw.projectile.as_ref().and_then(|p| {
            raw.projectiles.get(p).cloned().map(|pr| ProjectileDefinition {
                damage: pr.damage,
                speed: pr.speed,
                sprite: pr.sprite,
                explosion_sprite: pr.explosion_sprite,
                splash_radius: pr.splash_radius,
            })
        });

        towers.push(TowerDefinition {
            key: id,
            name: def_raw.name,
            description: def_raw.description,
            base_sprite: def_raw.base_sprite,
            top_sprite: def_raw.top_sprite,
            preview_top_sprite: def_raw.preview_top_sprite,
            cost: def_raw.cost,
            attack_range: def_raw.attack_range,
            attack_cooldown: def_raw.attack_cooldown,
            damage: def_raw.damage,
            muzzle_flash_sprite: def_raw.muzzle_flash_sprite,
            projectile,
                ammo_slot_offsets: def_raw.ammo_slot_offsets,
                ammo_refill_secs: def_raw.ammo_refill_secs,
            },
        );
    }

    info!(
        "[TD] tower registry loaded: {} towers ({:?})",
        towers.len(),
        towers.iter().map(|def| &def.key).collect::<Vec<_>>()
    );
    commands.insert_resource(TowerRegistry { towers });
    commands.insert_resource(SelectedTowerType(0));
}

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

/// 放置合法性（教程 08 Step 4 + 19）：草地 + 未占用。路径 / 越界 / 已占均拒绝。
pub fn is_placeable(map: &MapLayout, placed: &PlacedTowers, tile: [u32; 2]) -> bool {
    let is_grass = map.get(tile[0] as i32, tile[1] as i32) == Some(TileType::Grass);
    is_grass && !placed.0.contains_key(&tile)
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

/// 生成瞬时伤害塔（教程 16 `spawn_instant_tower`）：基座 + 炮塔
/// （`TowerAttacker` 合并射程/冷却，`InstantShooter` 携带伤害与枪口焰索引）。
///
/// P19：返回**塔顶实体**——占格表（`PlacedTowers`）记录的就是它（教程 19 让两个
/// 生成函数都返回 turret 实体）。
fn spawn_instant_tower(
    commands: &mut Commands,
    atlas: &TowerAtlas,
    def: &TowerDefinition,
    tower_index: usize,
    pos: Vec2,
) -> Entity {
    commands.spawn((
        Tower,
        GameEntity,
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: def.base_sprite,
            },
        ),
        Transform::from_xyz(pos.x, pos.y, 2.0),
    ));

    commands
        .spawn((
            TowerTurret,
            GameEntity,
            Sprite::from_atlas_image(
                atlas.texture.clone(),
                TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: def.top_sprite,
                },
            ),
            Transform::from_xyz(pos.x, pos.y, 2.1),
            TowerTypeId(tower_index),
            TowerAttacker {
                range: def.attack_range,
                timer: Timer::from_seconds(def.attack_cooldown, TimerMode::Repeating),
            },
            InstantShooter {
                damage: def.damage.expect("instant tower must have damage"),
                muzzle_flash_sprite: def
                    .muzzle_flash_sprite
                    .expect("instant tower must have muzzle_flash_sprite"),
            },
        ))
        .id()
}

/// 生成火箭发射器（教程 16 `spawn_rocket_launcher`）：基座 + 炮管（`AmmoState`）+
/// 槽位火箭子实体（槽数 = `ammo_slot_offsets.len()`，教程 16「Its length *is* the
/// max ammo capacity」）。P19：返回炮管实体（占格表登记对象）。
fn spawn_rocket_launcher(
    commands: &mut Commands,
    atlas: &TowerAtlas,
    def: &TowerDefinition,
    tower_index: usize,
    pos: Vec2,
) -> Entity {
    let projectile = def
        .projectile
        .as_ref()
        .expect("rocket tower must reference a projectile");
    let offsets = def
        .ammo_slot_offsets
        .as_ref()
        .expect("rocket tower must have ammo_slot_offsets");
    let refill = def
        .ammo_refill_secs
        .expect("rocket tower must have ammo_refill_secs");

    commands.spawn((
        Tower,
        GameEntity,
        Sprite::from_atlas_image(
            atlas.texture.clone(),
            TextureAtlas {
                layout: atlas.layout.clone(),
                index: def.base_sprite,
            },
        ),
        Transform::from_xyz(pos.x, pos.y, 2.0),
    ));

    let turret_entity = commands
        .spawn((
            TowerTurret,
            GameEntity,
            Sprite::from_atlas_image(
                atlas.texture.clone(),
                TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: def.top_sprite,
                },
            ),
            Transform::from_xyz(pos.x, pos.y, 2.1),
            TowerTypeId(tower_index),
            TowerAttacker {
                range: def.attack_range,
                timer: Timer::from_seconds(def.attack_cooldown, TimerMode::Repeating),
            },
            AmmoState {
                regen: Timer::from_seconds(refill, TimerMode::Repeating),
                slots: vec![None; offsets.len()],
                fired: 0,
            },
        ))
        .id();

    let mut slot_entities = vec![None; offsets.len()];
    commands.entity(turret_entity).with_children(|turret_children| {
        for (i, offset) in offsets.iter().enumerate() {
            let slot_entity = turret_children
                .spawn((
                    Sprite::from_atlas_image(
                        atlas.texture.clone(),
                        TextureAtlas {
                            layout: atlas.layout.clone(),
                            index: projectile.sprite,
                        },
                    ),
                    Transform::from_xyz(offset[0], offset[1], 2.2),
                ))
                .id();
            slot_entities[i] = Some(slot_entity);
        }
    });
    commands
        .entity(turret_entity)
        .insert(AmmoState {
            regen: Timer::from_seconds(refill, TimerMode::Repeating),
            slots: slot_entities,
            fired: 0,
        });
    turret_entity
}

/// 按世界坐标生成塔（P18 消息消费者与 P8 `td.spawn_tower` 共用出口）。
///
/// P19：返回塔顶实体（`PlacedTowers` 的 tile → entity 登记值）。
pub fn spawn_tower_at(
    commands: &mut Commands,
    atlas: &TowerAtlas,
    def: &TowerDefinition,
    tower_index: usize,
    pos: Vec2,
) -> Entity {
    if def.damage.is_some() {
        spawn_instant_tower(commands, atlas, def, tower_index, pos)
    } else {
        spawn_rocket_launcher(commands, atlas, def, tower_index, pos)
    }
}

/// 网格坐标版生成（BRP `td.spawn_tower` 使用）：返回（世界坐标, 塔顶实体）。
pub fn spawn_tower_entities(
    commands: &mut Commands,
    atlas: &TowerAtlas,
    def: &TowerDefinition,
    tower_index: usize,
    tile: [u32; 2],
    map_width: f32,
    map_height: f32,
) -> (Vec2, Entity) {
    let pos = tile_to_world(tile, map_width, map_height);
    let turret = spawn_tower_at(commands, atlas, def, tower_index, pos);
    (pos, turret)
}

/// 生成两个常驻预览幽灵（教程 08 Step 5 + 16/17：外观取当前选中塔型）。
pub fn spawn_placement_preview(
    mut commands: Commands,
    atlas: Res<TowerAtlas>,
    registry: Res<TowerRegistry>,
    selected: Res<SelectedTowerType>,
) {
    let def = registry.get(selected.0);
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
        tinted(def.base_sprite),
        Transform::from_xyz(0.0, 0.0, 2.0),
        Visibility::Hidden,
    ));
    commands.spawn((
        TowerPreview,
        GameEntity,
        tinted(def.preview_top_sprite),
        Transform::from_xyz(0.0, 0.0, 2.1),
        Visibility::Hidden,
    ));
}

/// 每帧按悬停更新预览（教程 08 Step 6 + 14 Step 6）：
/// 合法格移动并显示，否则隐藏；显示时按「红闪 > 买得起 > 买不起」三态着色。
pub fn update_placement_preview(
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    map_layout: Res<MapLayout>,
    placed: Res<PlacedTowers>,
    gold: Res<Gold>,
    registry: Res<TowerRegistry>,
    selected: Res<SelectedTowerType>,
    mut preview_q: Query<
        (
            &mut Transform,
            &mut Visibility,
            &mut Sprite,
            Option<&PlacementDenied>,
        ),
        With<TowerPreview>,
    >,
) {
    let (cam, cam_transform) = *camera;

    let Some(tile) = hovered_placeable_tile(&window, cam, cam_transform, &map_layout, &placed)
    else {
        for (_, mut vis, ..) in preview_q.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    };

    let pos = tile_to_world(tile, map_layout.width as f32, map_layout.height as f32);
    let def = registry.get(selected.0);
    let can_afford = gold.0 >= def.cost as f32;

    for (mut transform, mut vis, mut sprite, denied) in preview_q.iter_mut() {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
        *vis = Visibility::Visible;

        // P17（教程 17「Updating the placement preview on selection change」）：
        // 选中变化时换预览外观。教程按 z 排序后取 [0]=基座 / [1]=顶部；
        // 本实现直接按 z 判别（z 2.0 = 基座、2.1 = 顶部）——等价且无需排序断言。
        let is_base = transform.translation.z < 2.05;
        if let Some(atlas_ref) = sprite.texture_atlas.as_mut() {
            atlas_ref.index = if is_base {
                def.base_sprite
            } else {
                def.preview_top_sprite
            };
        }

        if denied.is_some() {
            sprite.color = Color::srgba(1.0, 0.3, 0.3, 0.5); // red flash
        } else if can_afford {
            sprite.color = Color::srgba(0.3, 1.0, 0.3, 0.5); // green
        } else {
            sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.5); // neutral white
        }
    }
}

/// 左键点击放置（教程 18「Refactoring the producer」）：**只做输入校验**——
/// 合法格 + 余额 → 写 [`PlaceTower`] 消息；生成实体与经济记账分别由
/// [`spawn_tower_from_event`] 与 `economy::deduct_gold_on_placement` 消费。
///
/// 与原（P14–P17）单系统的差别：本函数不再持有 `ResMut<Gold>` / `ResMut<PlacedTowers>` /
/// `Res<TowerAtlas>`，也不生成实体——「是否该放」与「告知世界已放」两件事。
/// 买不起仍直接红闪（教程 18「Simplifying the producer」：拒绝反馈留在生产者）。
pub fn place_tower_on_click(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    map_layout: Res<MapLayout>,
    placed: Res<PlacedTowers>,
    gold: Res<Gold>,
    preview_q: Query<Entity, With<TowerPreview>>,
    mut commands: Commands,
    mut place_messages: MessageWriter<PlaceTower>,
    registry: Res<TowerRegistry>,
    selected: Res<SelectedTowerType>,
    mut log: ResMut<PlacementLog>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let (cam, cam_transform) = *camera;

    let Some(tile) = hovered_placeable_tile(&window, cam, cam_transform, &map_layout, &placed)
    else {
        return;
    };

    let def = registry.get(selected.0);

    // 买不起：红闪 0.3s，不放置（教程 14 Step 7；教程 18 保留在生产者）。
    if gold.0 < def.cost as f32 {
        for preview_entity in preview_q.iter() {
            commands
                .entity(preview_entity)
                .insert(PlacementDenied(Timer::from_seconds(
                    DENIED_FLASH_DURATION,
                    TimerMode::Once,
                )));
        }
        return;
    }

    let pos = tile_to_world(tile, map_layout.width as f32, map_layout.height as f32);

    // 教程 18：广播消息——生成与扣费在别处。
    let event = PlaceTower {
        tile,
        world_pos: pos,
        tower_id: selected.0,
        cost: def.cost,
    };
    place_messages.write(event);
    // 验证面记账（`td.events`）：生产者写了一条消息。
    log.produced += 1;
}

/// 生成消费者（教程 18「The spawn consumer」）：读 [`PlaceTower`] 消息 → 生成塔实体。
///
/// 本系统不知道金币 / 输入 / 鼠标；只回答「有人说放塔，我生成」。
/// 架构不变量（教程 18）：每帧最多一条消息——`iter.next()` 两次断言即文档化。
pub fn spawn_tower_from_event(
    mut messages: MessageReader<PlaceTower>,
    mut commands: Commands,
    atlas: Res<TowerAtlas>,
    registry: Res<TowerRegistry>,
    mut placed: ResMut<PlacedTowers>,
    mut log: ResMut<PlacementLog>,
) {
    let mut iter = messages.read();
    let Some(event) = iter.next() else {
        return;
    };
    assert!(
        iter.next().is_none(),
        "only one tower can be placed per frame; the producer should emit at most one PlaceTower message",
    );

    let def = registry.get(event.tower_id);
    // 教程 18：生成不重算世界坐标——消息已携带 `world_pos`。
    let turret = spawn_tower_at(&mut commands, &atlas, def, event.tower_id, event.world_pos);
    // 教程 19：占格记账迁到生成消费者（实体在此诞生，格→实体的关系归它）。
    placed.0.insert(event.tile, turret);
    log.spawned += 1;
    log.spawn_last = Some(PlaceTowerRecord::from(event));
}

/// 索敌 + 旋转 + 冷却扣血 + 赏金 + 就地 despawn（教程 09 Step 4 + 16 重构）：
/// 只匹配**瞬时塔**（`&InstantShooter`）；快照敌人位置释放查询借用，
/// 两条查询以 `Without` 显式互斥（防 B0001）。
pub fn attack_enemies(
    time: Res<Time>,
    atlas: Res<TowerAtlas>,
    mut gold: ResMut<Gold>,
    mut sounds: MessageWriter<PlaySound>,
    mut turrets: Query<
        (Entity, &mut Transform, &mut TowerAttacker, &InstantShooter),
        (With<TowerTurret>, Without<Enemy>),
    >,
    mut enemies: Query<
        (Entity, &Transform, &mut Health, &Bounty),
        (With<Enemy>, With<PathFollower>, Without<TowerTurret>),
    >,
    mut commands: Commands,
) {
    // 快照敌军位置后释放 enemies 查询借用（教程 09「Snapshot pattern」）。
    let enemy_positions: Vec<(Entity, Vec2)> = enemies
        .iter()
        .map(|(e, t, ..)| (e, t.translation.truncate()))
        .collect();

    for (turret_entity, mut turret_transform, mut attacker, instant) in turrets.iter_mut() {
        attacker.timer.tick(time.delta());
        let turret_pos = turret_transform.translation.truncate();

        // 线性扫描最近目标（教程 09 Step 4；小规模足够）。
        let mut nearest: Option<(Entity, f32)> = None;
        for &(entity, pos) in &enemy_positions {
            let dist = turret_pos.distance(pos);
            if dist <= attacker.range && nearest.map_or(true, |(_, best)| dist < best) {
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

        // 冷却到期：扣血、赏金入账、死亡就地回收、枪口焰。
        if attacker.timer.just_finished() {
            // 教程 20 `One-shot laser fire`：先广播音效消息（消费者同帧播放）。
            sounds.write(PlaySound {
                sound: SoundType::LaserFire,
                volume: 1.0,
            });

            if let Ok((_, _, mut health, bounty)) = enemies.get_mut(target) {
                // 教程 14 Step 8：先判活再扣血（防同帧多源伤害重复计赏金）。
                if health.0 > 0.0 {
                    health.0 -= instant.damage;
                    if health.0 <= 0.0 {
                        gold.0 += bounty.0 as f32;
                        commands.entity(target).despawn();
                    }
                }
            }

            // 枪口焰：父实体带 DespawnTimer，两个火苗 sprite 作子实体（教程 09 Step 4）。
            // 教程 12 Step 3：父子同挂 GameEntity——清场时父实体递归 despawn 会让
            // 子实体的 despawn 命令失败，故 cleanup_level 用 queue_silenced（教程 12 Step 6）。
            let texture = atlas.texture.clone();
            let layout = atlas.layout.clone();
            let flash_sprite = instant.muzzle_flash_sprite;
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
                                    index: flash_sprite,
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

/// 发射火箭（教程 15 Step 6 + 16 重构）：索敌 → 炮管旋转 → 冷却到期取首个
/// 有弹槽位 → 槽位 `GlobalTransform` 世界坐标出生 → 销毁槽位火箭 →
/// 按 `TowerTypeId` 查注册表弹体定义 → 生成追踪弹。
pub fn launch_rockets(
    time: Res<Time>,
    atlas: Res<TowerAtlas>,
    registry: Res<TowerRegistry>,
    // 教程 20：弹体挂推进器循环音——音频资产资源注入本系统。
    audio_assets: Res<crate::audio::AudioAssets>,
    mut turrets: Query<
        (
            &mut Transform,
            &mut TowerAttacker,
            &mut AmmoState,
            &TowerTypeId,
        ),
        Without<Enemy>,
    >,
    enemies: Query<
        (Entity, &Transform),
        (With<Enemy>, With<PathFollower>, Without<TowerTurret>),
    >,
    slot_transforms: Query<&GlobalTransform>,
    mut commands: Commands,
) {
    let enemy_positions: Vec<(Entity, Vec2)> = enemies
        .iter()
        .map(|(e, t)| (e, t.translation.truncate()))
        .collect();

    for (mut transform, mut attacker, mut ammo, type_id) in turrets.iter_mut() {
        attacker.timer.tick(time.delta());
        let turret_pos = transform.translation.truncate();

        let mut nearest: Option<(Entity, f32)> = None;
        for &(entity, pos) in &enemy_positions {
            let dist = turret_pos.distance(pos);
            if dist <= attacker.range && nearest.map_or(true, |(_, best)| dist < best) {
                nearest = Some((entity, dist));
            }
        }

        let Some((target, _)) = nearest else {
            continue;
        };

        let Some(target_pos) = enemy_positions
            .iter()
            .find(|(e, _)| *e == target)
            .map(|(_, p)| *p)
        else {
            continue;
        };
        let direction = target_pos - turret_pos;
        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
        transform.rotation = Quat::from_rotation_z(angle);

        if !attacker.timer.just_finished() {
            continue;
        }

        let Some(slot_index) = ammo.slots.iter().position(|slot| slot.is_some()) else {
            continue;
        };
        let Some(slot_entity) = ammo.slots[slot_index] else {
            continue;
        };
        let spawn_pos = slot_transforms
            .get(slot_entity)
            .map(|gt| gt.translation().truncate())
            .unwrap_or(turret_pos);

        let def = registry.get(type_id.0);
        let Some(projectile_def) = def.projectile.as_ref() else {
            continue;
        };

        commands.entity(slot_entity).despawn();
        ammo.slots[slot_index] = None;
        ammo.fired += 1;

        let rocket_angle = direction.y.atan2(direction.x);
        commands.spawn((
            Sprite::from_atlas_image(
                atlas.texture.clone(),
                TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: projectile_def.sprite,
                },
            ),
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, 2.3)
                .with_rotation(Quat::from_rotation_z(rocket_angle)),
            Projectile {
                target,
                target_position: target_pos,
                speed: projectile_def.speed,
                damage: projectile_def.damage,
                splash_radius: projectile_def.splash_radius.unwrap_or(0.0),
                explosion_sprite: projectile_def.explosion_sprite,
            },
            // 教程 20「Looping rocket thruster on the projectile」：循环音作为**组件**
            // 直接挂弹体——弹体 despawn 即声音停止，无需停止消息或对照表。
            AudioPlayer::new(audio_assets.rocket_launch.clone()),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(2.0)),
            GameEntity,
        ));
    }
}

/// 追踪飞行（教程 15 Step 7 `move_projectiles`）：目标存活则刷新目标点
/// （悬垂引用：`get` 返回 Err 时沿用最后已知位置）；朝目标旋转；
/// 剩余距离 ≤ 一步则吸附并挂 `Exploding`。
pub fn move_projectiles(
    time: Res<Time>,
    mut projectiles: Query<
        (Entity, &mut Transform, &mut Projectile),
        (Without<Exploding>, Without<Enemy>),
    >,
    enemies: Query<&Transform, (With<Enemy>, Without<Exploding>, Without<Projectile>)>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut projectile) in projectiles.iter_mut() {
        if let Ok(target_transform) = enemies.get(projectile.target) {
            projectile.target_position = target_transform.translation.truncate();
        }

        let current = transform.translation.truncate();
        let to_target = projectile.target_position - current;
        let distance = to_target.length();
        let step = projectile.speed * time.delta_secs();

        let angle = to_target.y.atan2(to_target.x);
        transform.rotation = Quat::from_rotation_z(angle);

        if distance <= step {
            transform.translation.x = projectile.target_position.x;
            transform.translation.y = projectile.target_position.y;
            commands.entity(entity).insert(Exploding);
        } else {
            let direction = to_target / distance;
            transform.translation.x = current.x + direction.x * step;
            transform.translation.y = current.y + direction.y * step;
        }
    }
}

/// 命中结算（教程 15 Step 8 `explode_projectiles`）：半径内所有敌人扣
/// `projectile.damage`；`health.0 > 0.0` 守卫防同帧多次爆炸重复计赏金；
/// 按弹体定义的 `explosion_sprite` 生成短命特效并销毁弹体。
pub fn explode_projectiles(
    atlas: Res<TowerAtlas>,
    mut gold: ResMut<Gold>,
    mut sounds: MessageWriter<PlaySound>,
    projectiles: Query<(Entity, &Transform, &Projectile), (With<Exploding>, Without<Enemy>)>,
    mut enemies: Query<
        (Entity, &Transform, &mut Health, &Bounty),
        (With<Enemy>, Without<Exploding>),
    >,
    mut commands: Commands,
) {
    for (projectile_entity, transform, projectile) in projectiles.iter() {
        let center = transform.translation.truncate();

        // 教程 20 `One-shot rocket explosion`：先广播爆炸音效（与伤害/特效同帧）。
        sounds.write(PlaySound {
            sound: SoundType::RocketExplosion,
            volume: 0.8,
        });

        for (entity, enemy_transform, mut health, bounty) in enemies.iter_mut() {
            let distance = center.distance(enemy_transform.translation.truncate());
            if distance > projectile.splash_radius {
                continue;
            }
            if health.0 > 0.0 {
                health.0 -= projectile.damage;
                if health.0 <= 0.0 {
                    gold.0 += bounty.0 as f32;
                    commands.entity(entity).despawn();
                }
            }
        }

        // 爆炸特效：短命 Sprite + 通用 DespawnTimer（教程 15 Step 8 第 5 步）。
        if let Some(sprite_index) = projectile.explosion_sprite {
            commands.spawn((
                GameEntity,
                Sprite::from_atlas_image(
                    atlas.texture.clone(),
                    TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: sprite_index,
                    },
                ),
                Transform::from_xyz(center.x, center.y, 2.5),
                DespawnTimer(Timer::from_seconds(MUZZLE_FLASH_DURATION, TimerMode::Once)),
            ));
        }

        commands.entity(projectile_entity).despawn();
    }
}

/// 补弹（教程 15 Step 9 + 16 重构）：`AmmoState.regen` 到期 → 首个空槽 →
/// 按 `TowerTypeId` 取注册表的槽位偏移与弹体图集索引 → 生成火箭子实体回填。
pub fn refill_ammo(
    time: Res<Time>,
    atlas: Res<TowerAtlas>,
    registry: Res<TowerRegistry>,
    mut turrets: Query<(Entity, &TowerTypeId, &mut AmmoState)>,
    mut commands: Commands,
) {
    for (turret_entity, type_id, mut ammo) in turrets.iter_mut() {
        ammo.regen.tick(time.delta());
        if !ammo.regen.just_finished() {
            continue;
        }
        let Some(slot_index) = ammo.slots.iter().position(|slot| slot.is_none()) else {
            continue;
        };

        let def = registry.get(type_id.0);
        let (Some(offsets), Some(projectile_def)) = (def.ammo_slot_offsets.as_ref(), def.projectile.as_ref())
        else {
            continue;
        };
        let Some(offset) = offsets.get(slot_index) else {
            continue;
        };

        let mut new_slot = None;
        commands.entity(turret_entity).with_children(|turret_children| {
            new_slot = Some(
                turret_children
                    .spawn((
                        Sprite::from_atlas_image(
                            atlas.texture.clone(),
                            TextureAtlas {
                                layout: atlas.layout.clone(),
                                index: projectile_def.sprite,
                            },
                        ),
                        Transform::from_xyz(offset[0], offset[1], 2.2),
                    ))
                    .id(),
            );
        });
        ammo.slots[slot_index] = new_slot;
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

// ---------------------------------------------------------------------------
// P19：射程圈可视化（教程 19 gizmos）
// ---------------------------------------------------------------------------
//
// 0.20 锚点（漂移表 Part 19，均摩擦级 0）：
// - `Gizmos` 系统参数（`bevy_gizmos-0.20.0/src/gizmos.rs:143`）——prelude 经
//   `bevy_internal-0.20.0/src/prelude.rs:99-100` re-export；`GizmoPlugin` 随
//   `DefaultPlugins` 内置（`bevy_internal-0.20.0/src/default_plugins.rs:87`；
//   `2d` 默认特性含 `bevy_gizmos_render`，`bevy-0.20.0/Cargo.toml:2706-2724`）；
// - `circle_2d(isometry: impl Into<Isometry2d>, radius, color)`
//   （`bevy_gizmos-0.20.0/src/circles.rs:167`）；`From<Vec2> for Isometry2d`
//   （`bevy_math-0.20.0/src/isometry.rs:195`）——教程 `circle_2d(center Vec2, …)`
//   同形直用。
//
// gizmo 是**每帧重画**的瞬时形状（教程 19「Pitfall」）：系统每帧调用 `circle_2d`，
// 不挂实体。验证面因此记录**绘制参数**（[`RangeGizmoLog`]），`td.gizmos` 可读；
// 参数算式由纯函数承载，`td.gizmos probe` 与绘制系统共用同一函数（防双通道漂移）。

/// 预览圈颜色（教程 19：金色，alpha 0.4）。
pub const PREVIEW_RING_RGBA: [f32; 4] = [1.0, 0.84, 0.0, 0.4];

/// 悬停已放置塔的圈颜色（教程 19：白色，alpha 0.5）。
pub const HOVER_RING_RGBA: [f32; 4] = [1.0, 1.0, 1.0, 0.5];

/// 圈类型（`td.gizmos` 快照可读）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingKind {
    /// 放置预览圈（教程 19：预览可见时每帧画）。
    Preview,
    /// 悬停在已放置塔所在格时画的圈（教程 19：按格 O(1) 查）。
    Hover,
}

/// 一次 `gizmos.circle_2d` 调用的参数化数据——**行为等价面**（教程 19：
/// 圆心 / 半径 / 颜色即行为；gizmo 瞬时，无实体可查）。
#[derive(Clone, Copy, Debug)]
pub struct RingParams {
    pub center: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
    pub kind: RingKind,
}

/// 悬停圈参数（纯函数 + 注入式塔查找）：绘制系统与 `td.gizmos probe` 共用同一算式，
/// 两条通道不会漂移（口径同 `td.spawn_tower` 复用 `is_placeable`）。
pub fn hover_ring_params(
    placed: &PlacedTowers,
    tile: [u32; 2],
    tower_lookup: impl FnOnce(Entity) -> Option<(Vec2, f32)>,
) -> Option<RingParams> {
    let entity = *placed.0.get(&tile)?;
    let (center, radius) = tower_lookup(entity)?;
    Some(RingParams {
        center: [center.x, center.y],
        radius,
        color: HOVER_RING_RGBA,
        kind: RingKind::Hover,
    })
}

/// 预览圈参数（教程 19：半径 = 选中塔型 `attack_range`，圆心 = 预览幽灵位置）。
pub fn preview_ring_params(center: Vec2, radius: f32) -> RingParams {
    RingParams {
        center: [center.x, center.y],
        radius,
        color: PREVIEW_RING_RGBA,
        kind: RingKind::Preview,
    }
}

/// 上一帧绘制记录（**验证面**资源，非教程；同 `SpawnSchedule::spawned` /
/// `AmmoState::fired` 口径——只服务 `td.gizmos` 快照，不给玩法增加语义）。
///
/// `rings` 每帧覆盖（gizmo 语义 = 每帧重画）；`preview_visible` 记录本帧预览
/// 幽灵是否可见（圈只在可见时画，教程 19）。
#[derive(Resource, Default)]
pub struct RangeGizmoLog {
    pub rings: Vec<RingParams>,
    pub preview_visible: bool,
}

/// 每帧画射程圈（教程 19 `draw_tower_ranges`）：
/// ① 光标所在格有已放置塔 → 白色圈（圆心 = 塔位置，半径 = `TowerAttacker.range`）；
/// ② 预览幽灵可见 → 金色圈（圆心 = 幽灵位置，半径 = 选中塔型 `attack_range`）。
///
/// 挂 `Update`（表现层，教程 19 注册在 InGame 的 Update 组）。
pub fn draw_tower_ranges(
    mut gizmos: Gizmos,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    placed: Res<PlacedTowers>,
    towers: Query<(&Transform, &TowerAttacker)>,
    preview: Query<(&Transform, &Visibility), With<TowerPreview>>,
    selected: Res<SelectedTowerType>,
    registry: Res<TowerRegistry>,
    map_layout: Res<MapLayout>,
    mut log: ResMut<RangeGizmoLog>,
) {
    let (cam, cam_transform) = *camera;
    let cursor_world = window
        .cursor_position()
        .and_then(|cursor| cam.viewport_to_world_2d(cam_transform, cursor).ok());

    log.rings.clear();
    log.preview_visible = false;

    // ① 悬停已放置塔：屏幕 → 世界 → 格 → tile→entity O(1) 查（教程 19）。
    if let Some(cursor_pos) = cursor_world {
        if let Some(tile) = world_to_tile(cursor_pos, map_layout.width, map_layout.height) {
            if let Some(ring) = hover_ring_params(&placed, tile, |entity| {
                towers
                    .get(entity)
                    .ok()
                    .map(|(transform, attacker)| (transform.translation.truncate(), attacker.range))
            }) {
                draw_ring(&mut gizmos, ring);
                log.rings.push(ring);
            }
        }
    }

    // ② 预览圈：只有幽灵可见才画（教程 19「only when it is visible」）。
    let def = registry.get(selected.0);
    for (preview_transform, visibility) in preview.iter() {
        if *visibility != Visibility::Visible {
            continue;
        }
        log.preview_visible = true;
        let ring = preview_ring_params(preview_transform.translation.truncate(), def.attack_range);
        draw_ring(&mut gizmos, ring);
        log.rings.push(ring);
    }
}

/// 单圈绘制（`td.` 参数 → `gizmos.circle_2d`；唯一接触 gizmo API 的点）。
fn draw_ring(gizmos: &mut Gizmos, ring: RingParams) {
    gizmos.circle_2d(
        Vec2::new(ring.center[0], ring.center[1]),
        ring.radius,
        Color::srgba(ring.color[0], ring.color[1], ring.color[2], ring.color[3]),
    );
}

// ---------------------------------------------------------------------------
// P17：塔选择 UI（教程 17）
// ---------------------------------------------------------------------------
//
// 0.20 锚点（漂移表 Part 17）：
// - `Node`（`bevy_ui-0.20.0/src/ui_node.rs:485`）、`Val`（`geometry.rs:32`）；
// - `ImageNode::from_atlas_image(image, TextureAtlas)`（`bevy_ui-0.20.0/src/widget/image.rs:99`）；
// - `BorderColor` 四边字段（`ui_node.rs:2603`）与 `BorderColor::all`（`:2627`）——
//   **`Node.border` 必须给宽度，否则边框不渲染**（教程 17 Pitfall）；
// - `Interaction`：0.20 中为 `DeprecatedInteraction` 的弃用别名
//   （`bevy_ui-0.20.0/src/focus.rs:50` 本体 / `:65` 别名；SKILL §6.14），教程 17 原步
//   用法以 `#[expect(deprecated)]` 属性落地（T044 立项 A' 裁决，**不换新 API 重写**）；
// - `MessageReader<MouseWheel>`（`bevy_ecs-0.20.0/src/message/message_reader.rs:34`；
//   `MouseWheel` `bevy_input-0.20.0/src/mouse.rs:214`，字段 `x`/`y` `:218`/`:220`；
//   消息注册 `bevy_input-0.20.0/src/lib.rs:122`）——0.18 的 `EventReader` 已随事件
//   Message 化删除（SKILL §6.13），滚轮读取至此形态；
// - `Res::is_changed`（`bevy_ecs-0.20.0/src/change_detection/params.rs:603`）。

/// dock 根标记（教程 17 `TowerDock`）：挂 `GameEntity`，离场随关卡回收。
#[derive(Component)]
pub(crate) struct TowerDock;

/// dock 槽位（教程 17 `TowerDockSlot(usize)`）：存该槽代表的塔型下标。
///
/// 反射注册（`#[type_path = "game_td"]` 冻结）：U6 约束下 Node/Text 不在 BRP
/// 反射面，UI 态的**可查询侧信道** = 槽位数（`world.query game_td::TowerDockSlot`）
/// + 选中下标资源（`td.tower_select`）；视觉态由截图留证。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub(crate) struct TowerDockSlot(pub usize);

/// 槽位尺寸（教程 17 `DOCK_SLOT_SIZE`）。
const DOCK_SLOT_SIZE: f32 = 80.0;

/// 槽位间距（教程 17 `DOCK_SLOT_GAP`）。
const DOCK_SLOT_GAP: f32 = 8.0;

/// 槽底背景色（教程 17 `DOCK_BG`）。
const DOCK_BG: Color = Color::srgba(0.12, 0.12, 0.12, 0.9);

/// 未选中边框色（教程 17 `DOCK_BORDER_DEFAULT`）。
const DOCK_BORDER_DEFAULT: Color = Color::srgba(0.3, 0.3, 0.3, 1.0);

/// 选中边框色（教程 17 `DOCK_BORDER_SELECTED`：金色）。
const DOCK_BORDER_SELECTED: Color = Color::srgba(1.0, 0.84, 0.0, 1.0);

/// 数字键位表（教程 17：`Digit1..Digit5` → 下标 0..4）。
const DOCK_KEYS: [KeyCode; 5] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
];

/// 生成塔选 dock（教程 17 `setup_tower_dock`）：底部居中的一行槽位，
/// 每槽 = 键号徽标 + 预览图 + 名称 + 造价；槽位挂 `Interaction` 以跟踪悬停/按下。
///
/// `#[expect(deprecated)]`：本函数使用 `Interaction::None`——教程 17 原步，
/// 0.20 现况 = `Interaction` 为弃用别名（`bevy_ui-0.20.0/src/focus.rs:50/:65`；
/// SKILL §6.14「勿新用」）；A' 裁决通道：带属性落地并在此注明，不换 API 重写。
#[expect(
    deprecated,
    reason = "教程 17 原步使用 Interaction；0.20 弃用别名按 T044 A' 裁决以 expect 属性落地"
)]
pub fn setup_tower_dock(
    mut commands: Commands,
    registry: Res<TowerRegistry>,
    atlas: Res<TowerAtlas>,
    mut selected: ResMut<SelectedTowerType>,
) {
    // 教程 17 Pitfall 修法：进关重置选中下标——保证 `update_dock_selection` 的
    // `is_changed()` 在新一局首帧为真（否则全灰无边高亮）。
    selected.0 = 0;

    commands
        .spawn((
            TowerDock,
            GameEntity,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(DOCK_SLOT_GAP),
                ..default()
            },
        ))
        .with_children(|dock| {
            for (index, def) in registry.towers.iter().enumerate() {
                dock.spawn((
                    TowerDockSlot(index),
                    Interaction::None,
                    Node {
                        width: Val::Px(DOCK_SLOT_SIZE),
                        height: Val::Px(DOCK_SLOT_SIZE),
                        border: UiRect::all(Val::Px(2.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                    BackgroundColor(DOCK_BG),
                    BorderColor::all(DOCK_BORDER_DEFAULT),
                ))
                .with_children(|slot| {
                    slot.spawn((
                        Text::new(format!("{}", index + 1)),
                        TextFont {
                            font_size: 12.0.into(),
                            ..default()
                        },
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
                    ));
                    slot.spawn((
                        ImageNode::from_atlas_image(
                            atlas.texture.clone(),
                            TextureAtlas {
                                layout: atlas.layout.clone(),
                                index: def.preview_top_sprite,
                            },
                        ),
                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            ..default()
                        },
                    ));
                    slot.spawn((
                        Text::new(def.name.clone()),
                        TextFont {
                            font_size: 11.0.into(),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    slot.spawn((
                        Text::new(format!("{}g", def.cost)),
                        TextFont {
                            font_size: 11.0.into(),
                            ..default()
                        },
                        TextColor(Color::srgba(0.3, 1.0, 0.3, 1.0)),
                    ));
                });
            }
        });
}

/// 选中变化 → 刷新槽位边框（教程 17 `update_dock_selection`）：
/// `is_changed()` 门控只在玩家真的换塔时遍历；选中槽金色、其余灰色。
pub fn update_dock_selection(
    selected: Res<SelectedTowerType>,
    mut slots: Query<(&TowerDockSlot, &mut BorderColor)>,
) {
    if !selected.is_changed() {
        return;
    }
    for (slot, mut border) in slots.iter_mut() {
        let color = if slot.0 == selected.0 {
            DOCK_BORDER_SELECTED
        } else {
            DOCK_BORDER_DEFAULT
        };
        *border = BorderColor::all(color);
    }
}

/// 滚轮切换（教程 17 `cycle_tower_on_scroll`）：每帧只消费首条 `MouseWheel`；
/// 向下（`y < 0`）下一个、向上（`y > 0`）上一个；两端 clamp 不回绕。
pub fn cycle_tower_on_scroll(
    mut scroll: MessageReader<MouseWheel>,
    registry: Res<TowerRegistry>,
    mut selected: ResMut<SelectedTowerType>,
) {
    let len = registry.towers.len();
    if len == 0 {
        return;
    }
    let Some(ev) = scroll.read().next() else {
        return;
    };

    if ev.y < 0.0 && selected.0 + 1 < len {
        selected.0 += 1;
    } else if ev.y > 0.0 && selected.0 > 0 {
        selected.0 -= 1;
    }
}

/// 数字键直选（教程 17 `select_tower_by_key`）：`1`–`5` → 下标 0–4，越界忽略。
pub fn select_tower_by_key(
    keys: Res<ButtonInput<KeyCode>>,
    registry: Res<TowerRegistry>,
    mut selected: ResMut<SelectedTowerType>,
) {
    for (i, keycode) in DOCK_KEYS.iter().enumerate() {
        if i < registry.towers.len() && keys.just_pressed(*keycode) {
            selected.0 = i;
        }
    }
}

/// 点击槽位选中（教程 17 `handle_dock_slot_click`）：`Changed<Interaction>` 过滤，
/// `Pressed` 即写选中下标（同帧由 `update_dock_selection` 反映到边框）。
///
/// `#[expect(deprecated)]`：`Interaction` 查询与 `Interaction::Pressed` 比对均为
/// 教程 17 原步（0.20 现况：弃用别名 `bevy_ui-0.20.0/src/focus.rs:50/:65`；
/// SKILL §6.14）；A' 裁决通道带属性落地。
#[expect(
    deprecated,
    reason = "教程 17 原步使用 Interaction 三态；0.20 弃用别名按 T044 A' 裁决以 expect 属性落地"
)]
pub fn handle_dock_slot_click(
    slots: Query<(&TowerDockSlot, &Interaction), Changed<Interaction>>,
    mut selected: ResMut<SelectedTowerType>,
) {
    for (slot, interaction) in &slots {
        if *interaction == Interaction::Pressed {
            selected.0 = slot.0;
        }
    }
}

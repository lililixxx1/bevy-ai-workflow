//! BRP 常驻插件（工作流三的通路底座）+ 唯一自定义方法 `td.screenshot`。
//!
//! 监听口径（写死，不暴露 CLI）：`127.0.0.1:15702`。
//! 依据: `RemotePlugin` `bevy_remote-0.20.0/src/lib.rs:572`、
//! `with_method_main(name: impl Into<String>, handler: impl IntoSystem<
//! In<Option<Value>>, BrpResult, M>)` `lib.rs:591`；`RemoteHttpPlugin`
//! `src/http.rs:113`、`with_address(impl Into<IpAddr>)` `http.rs:167`；
//! 默认地址 `DEFAULT_ADDR` = 127.0.0.1、默认端口 `DEFAULT_PORT` = 15702
//! （`http.rs:60` / `:52`）——核实 2026-10-29（对齐 SKILL.md §6.1/§6.2）。
//! 安全约束（AGENTS.md 硬约束 4）：仅回环地址，禁止 0.0.0.0 / 局域网——
//! 即使默认已是回环，这里仍显式绑定，防默认值变化。
//!
//! `td.screenshot`（唯一自定义方法，方法名**不得**带 `game.` 前缀——
//! claim-lint 计数面锚定该前缀于主仓 game 包）：单方法两职能——受理主窗口
//! 帧捕获落盘 PNG、查询捕获完成状态。捕获是**异步**的：handler 只受理请求
//! （spawn 携带 [`Screenshot`] 组件的实体并挂两个 observer），渲染子 app 在
//! 随后帧捕获，完成后以 `ScreenshotCaptured` 实体事件触发 observer——官方
//! `save_to_disk` 同步写 PNG，[`on_captured`] 回填日志（尺寸/状态），实体最后
//! 由 ScreenshotPlugin 的 `clear_screenshots`（First 调度）清理。
//! 因此方法契约是两段式：`{"op": "capture"}` 受理 → 轮询 `{"op": "status"}`
//! 至 `status == "captured"` 后核对文件（PNG 魔数）。
//!
//! params 形态（全部可省；缺省 = 受理一次捕获、默认文件名
//! `td-screenshot-{id}.png`，相对进程 CWD）：
//! - `{}` / 缺省 / `{"path": "x.png"}` → 受理捕获；
//! - `{"op": "status"}` → 只查询，返回 `{pending, entries, next_id}`；
//! - `{"op": "capture", "path": "x.png"}` → 显式受理。
//! 其它 op / 类型一律 -32602（不静默回落）。
//!
//! 依据: `Screenshot(pub RenderTarget)` 组件与 spawn+observer 用法
//! `bevy_render-0.20.0/src/view/window/screenshot.rs:84`；`primary_window()`
//! `:102`；`ScreenshotCaptured { entity, image }`（EntityEvent）`:53`；
//! `save_to_disk(impl AsRef<Path>) -> impl FnMut(On<ScreenshotCaptured>)`
//! （内部 `to_rgb8().save_with_format` 同步落盘）`:145`；Captured 实体清理
//! `clear_screenshots` 定义 `:255`、First 调度注册 `:473-479`（核实 2026-10-29；
//! 注：主仓 `game/src/rpc/screenshot.rs` 模块注释原带同两项 0.19 锚 `:197`/`:416-418`
//! ——G2 前勘误归因：出处是该模块注释而非 SKILL §6.8〔该节现行锚 :53/:145 已是
//! 0.20〕，主仓侧已随 T044 收尾批重锚）。
//!
//! 组件反射注册（SKILL.md §3.4）：`MapTile` / `PathTile` 需被 BRP 查询，
//! 故 derive `Reflect` + `#[reflect(Component)]` 并在 [`BrpPlugin::build`]
//! 显式 `register_type`（`bevy_app-0.20.0/src/app.rs:664`）。
//!
//! 测试通道 `td.spawn_tower`（P8）：BRP 面没有鼠标/键盘注入，塔放置的验证必须
//! 有确定性入口——本方法复用 `tower::is_placeable` + `tower::spawn_tower_entities`
//! （与点击系统同一段校验与生成逻辑，两条通道不会漂移）。方法语义：
//! - params `{"tile": [x, y]}`（网格坐标，必填）；
//! - 放置成功 → `{"placed": true, "tile": [x,y], "pos": [wx, wy]}`；
//! - 不可放置（路径 / 已占用 / 越界 / 未加载关卡）→ `{"placed": false, "reason": …}`；
//! - tile 形态非法 → -32602（不静默回落）。
//!
//! 测试通道 `td.level`（P13）：BRP 面没有键盘注入，关卡选择必须能确定性触发。
//! - `{}` / `{"op":"list"}` → `{"available", "levels": [...], "selected"}`；
//! - `{"op":"select","index":n}` / `{"op":"select","path":"..."}` → 写 `SelectedLevel`；
//! - 非法 index / path → -32602。
//!
//! 测试通道 `td.economy`（P14）：金币 / 常量面只读快照（经济无键鼠注入面，
//! 读数必须能从外部确定性取得）。
//!
//! 测试通道 `td.towers`（P16）：数据驱动塔注册表的只读快照（键 / 数值 / 弹体定义）
//! ——「塔定义真的来自 TOML」的可验证面。
//!
//! 测试通道 `td.tower_select`（P17）：塔选下标读写（U6 侧信道——BRP 无滚轮 / 键盘 /
//! 鼠标点击注入，选择效果经「选中下标 → 放置塔型 → `TowerTypeId`」链验证）。

use bevy::prelude::*;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::{error_codes, BrpError, BrpResult, RemotePlugin};
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use serde::Serialize;
use serde_json::{json, Value};

use crate::map::MapLayout;
use crate::tower::{self, PlacedTowers, TowerAtlas};
use crate::GameState;

/// `td.screenshot` 方法名（唯一自定义方法；前缀 `td.`，非 `game.`）。
pub const TD_SCREENSHOT_METHOD: &str = "td.screenshot";

/// `td.spawn_tower` 方法名（P8 起；`td.` 前缀纪律同前）。
pub const TD_SPAWN_TOWER_METHOD: &str = "td.spawn_tower";

/// `td.wave` 方法名（P10 起：刷怪调度状态查询，只读）。
pub const TD_WAVE_METHOD: &str = "td.wave";

/// `td.game` 方法名（P11 起：基地生命 / 结束标记 / 波次记账快照，只读）。
pub const TD_GAME_METHOD: &str = "td.game";

/// `td.state` 方法名（P12 起：状态机读写——无键鼠注入通道，状态迁移需确定性入口）。
pub const TD_STATE_METHOD: &str = "td.state";

/// `td.level` 方法名（P13 起：关卡发现列表读取 / 选中项切换）。
pub const TD_LEVEL_METHOD: &str = "td.level";

/// `td.economy` 方法名（P14 起：金币 / 常量面只读快照）。
pub const TD_ECONOMY_METHOD: &str = "td.economy";

/// `td.combat` 方法名（P15 起：弹道 / 弹药 / 战斗常量的只读快照）。
pub const TD_COMBAT_METHOD: &str = "td.combat";

/// `td.towers` 方法名（P16 起：数据驱动塔注册表只读快照）。
pub const TD_TOWERS_METHOD: &str = "td.towers";

/// `td.tower_select` 方法名（P17 起：塔选下标读写——无键鼠注入面的确定性通道，
/// U6 约束的「可查询侧信道」）。
pub const TD_TOWER_SELECT_METHOD: &str = "td.tower_select";

pub struct BrpPlugin;

impl Plugin for BrpPlugin {
    fn build(&self, app: &mut App) {
        // 组件反射注册（SKILL.md §3.4）：MapTile / PathTile / TileType 需被 BRP 查询，
        // 显式注册无条件成立且自文档化（`register_type` `bevy_app-0.20.0/src/app.rs:664`）。
        // P5 模块化后类型移入 `crate::map`，但反射类型路径由 `#[type_path = "game_td"]`
        // 冻结在 crate 根形态（见 map.rs 同项注释），BRP 组件名不变。
        app.register_type::<crate::map::MapTile>()
            .register_type::<crate::map::PathTile>()
            .register_type::<crate::map::TileType>()
            // P7：Enemy 标记需被 BRP 查询（p7 断言 Enemy count==1），同一注册纪律。
            .register_type::<crate::enemy::Enemy>()
            // P8：塔三件（基座 / 炮塔 / 预览幽灵）——断言塔计数与占用语义。
            .register_type::<crate::tower::Tower>()
            .register_type::<crate::tower::TowerTurret>()
            .register_type::<crate::tower::TowerPreview>()
            // P9：Health——断言伤害结算与死亡回收（教程 09 Step 1）。
            .register_type::<crate::enemy::Health>()
            // P14：经济面三件——HUD 标记计数与赏金组件读值（教程 14 Step 2/3）。
            .register_type::<crate::economy::GoldHud>()
            .register_type::<crate::economy::Bounty>()
            // P17：塔型下标（放置塔的塔型端到端证据）与 dock 槽位（U6 侧信道计数）。
            .register_type::<crate::tower::TowerTypeId>()
            .register_type::<crate::tower::TowerDockSlot>();
        app.init_resource::<ScreenshotLog>();
        app.add_plugins(
            RemotePlugin::default()
                .with_method_main(TD_SCREENSHOT_METHOD, screenshot_handler)
                .with_method_main(TD_SPAWN_TOWER_METHOD, spawn_tower_handler)
                .with_method_main(TD_WAVE_METHOD, wave_handler)
                .with_method_main(TD_GAME_METHOD, game_handler)
                .with_method_main(TD_STATE_METHOD, state_handler)
                .with_method_main(TD_LEVEL_METHOD, level_handler)
                .with_method_main(TD_ECONOMY_METHOD, economy_handler)
                .with_method_main(TD_COMBAT_METHOD, combat_handler)
                .with_method_main(TD_TOWERS_METHOD, towers_handler)
                .with_method_main(TD_TOWER_SELECT_METHOD, tower_select_handler),
        );
        app.add_plugins(RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST));
        info!("[BRP] listening on 127.0.0.1:15702 (explicit loopback bind)");
        info!(
            "[BRP] custom methods: {TD_SCREENSHOT_METHOD}, {TD_SPAWN_TOWER_METHOD}, \
             {TD_WAVE_METHOD}, {TD_GAME_METHOD}, {TD_STATE_METHOD}, {TD_LEVEL_METHOD}, \
             {TD_ECONOMY_METHOD}, {TD_COMBAT_METHOD}, {TD_TOWERS_METHOD}, \
             {TD_TOWER_SELECT_METHOD}"
        );
    }
}

/// 截图日志（受理即建 pending 行，捕获完成由 observer 补齐）。
#[derive(Resource, Default)]
pub struct ScreenshotLog {
    /// 递增 id（默认文件名序号同源）。
    next_id: u32,
    /// 全部条目（含 pending 与 captured；不清理——验证用）。
    entries: Vec<ScreenshotEntry>,
}

/// 单条截图记录（`td.screenshot` status 响应元素）。
#[derive(Serialize, Clone, Debug)]
pub struct ScreenshotEntry {
    pub id: u32,
    /// 落盘目标路径（原样记录：相对路径以进程 CWD 为准）。
    pub path: String,
    /// `pending`（已受理未完成）/ `captured`（ScreenshotCaptured 已触发）。
    /// 注意 `captured` 只承诺事件已到（含官方 save_to_disk 落盘动作已执行），
    /// 文件核验（存在性/PNG 魔数/尺寸）由证据脚本做——最强证据在文件本身。
    pub status: &'static str,
    /// 捕获图像宽（完成时填）。
    pub width: Option<u32>,
    /// 捕获图像高（完成时填）。
    pub height: Option<u32>,
    /// 受理的 Screenshot 实体（observer 回填定位键；序列化意义为零，跳过）。
    #[serde(skip)]
    entity: Entity,
}

impl ScreenshotLog {
    /// 记一条受理（id 取自 `next_id` 并自增）。
    fn request(&mut self, path: String, entity: Entity) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(ScreenshotEntry {
            id,
            path,
            status: "pending",
            width: None,
            height: None,
            entity,
        });
        id
    }
}

/// 参数解析结果：受理捕获（可带目标路径）或仅查询状态。
enum Op {
    Capture(Option<String>),
    Status,
}

/// 解析 `td.screenshot` 的 params（严格分支，不静默回落）。
fn parse_op(params: Option<&Value>) -> Result<Op, BrpError> {
    let Some(value) = params else {
        return Ok(Op::Capture(None));
    };
    if value.is_null() {
        return Ok(Op::Capture(None));
    }
    let Value::Object(map) = value else {
        return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
    };

    let op = match map.get("op") {
        None | Some(Value::Null) => "capture",
        Some(Value::String(s)) => s.as_str(),
        Some(other) => {
            return Err(invalid_params(&format!("op 须为字符串（收到 {other}）")));
        }
    };

    let path = match map.get("path") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) if s.is_empty() => {
            return Err(invalid_params("path 不能为空字符串"));
        }
        Some(Value::String(s)) => Some(s.clone()),
        Some(other) => {
            return Err(invalid_params(&format!(
                "path 须为字符串（收到 {other}）"
            )));
        }
    };

    match op {
        "capture" => Ok(Op::Capture(path)),
        "status" => Ok(Op::Status),
        other => Err(invalid_params(&format!(
            "未知 op `{other}`（允许 capture / status）"
        ))),
    }
}

/// `td.screenshot` handler：受理一次主窗口捕获 / 查询捕获完成状态（见模块文档）。
pub fn screenshot_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    match parse_op(params.as_ref())? {
        Op::Status => {
            let log = world.resource::<ScreenshotLog>();
            Ok(json!({
                "next_id": log.next_id,
                "pending": log.entries.iter().filter(|e| e.status == "pending").count(),
                "entries": log.entries,
            }))
        }
        Op::Capture(requested) => {
            // 默认文件名需要 id：先读（handler 单线程独占，读后 spawn 前无人改写），
            // spawn 后再经 request() 落账（id 同源）。
            let id_for_name = world.resource::<ScreenshotLog>().next_id;
            let path = requested.unwrap_or_else(|| format!("td-screenshot-{id_for_name}.png"));

            let entity = world
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(std::path::PathBuf::from(path.clone())))
                .observe(on_captured)
                .id();

            let id = world
                .resource_mut::<ScreenshotLog>()
                .request(path.clone(), entity);
            info!("[TD] screenshot #{id} queued -> {path} (entity {entity})");

            Ok(json!({
                "queued": true,
                "id": id,
                "path": path,
                "note": "捕获异步完成：再调 {\"op\":\"status\"} 至 status=captured，再核验 PNG 文件",
            }))
        }
    }
}

/// observer：捕获完成 → 回填日志（尺寸 / 状态翻转）。
/// 与官方 `save_to_disk` 并挂同一实体（两个 observer 的相对次序无契约，
/// 故本条目只记 `captured`，不承诺文件已写——文件核验归证据脚本）。
fn on_captured(event: On<ScreenshotCaptured>, mut log: ResMut<ScreenshotLog>) {
    let entity = event.entity;
    let size = event.image.texture_descriptor.size;
    if let Some(entry) = log.entries.iter_mut().rev().find(|e| e.entity == entity) {
        entry.status = "captured";
        entry.width = Some(size.width);
        entry.height = Some(size.height);
        info!(
            "[TD] screenshot #{} captured ({}x{})",
            entry.id, size.width, size.height
        );
    } else {
        warn!("[TD] ScreenshotCaptured 未匹配到日志条目（entity {entity}）");
    }
}

/// 解析 `td.spawn_tower` 的 `tile` 参数（严格分支，不静默回落）。
fn parse_tile_param(params: Option<&Value>) -> Result<[u32; 2], BrpError> {
    let Some(value) = params else {
        return Err(invalid_params("params 缺 tile（形态 {\"tile\": [x, y]}）"));
    };
    let Some(obj) = value.as_object() else {
        return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
    };
    let Some(Value::Array(arr)) = obj.get("tile") else {
        return Err(invalid_params("tile 须为 [x, y] 数组"));
    };
    if arr.len() != 2 {
        return Err(invalid_params(&format!(
            "tile 须恰含 2 个元素（收到 {}）",
            arr.len()
        )));
    }
    let mut tile = [0u32; 2];
    for (i, item) in arr.iter().enumerate() {
        let Some(n) = item.as_u64() else {
            return Err(invalid_params(&format!(
                "tile[{i}] 须为非负整数（收到 {item}）"
            )));
        };
        let Ok(v) = u32::try_from(n) else {
            return Err(invalid_params(&format!("tile[{i}] 超出 u32（收到 {n}）")));
        };
        tile[i] = v;
    }
    Ok(tile)
}

/// `td.spawn_tower` handler（P8）：确定性放置通道（见模块文档）。
///
/// 复用 `tower::is_placeable` / `tower::spawn_tower_entities`——与点击系统同源，
/// 仅入口不同（`World::commands` 取命令队列，`bevy_ecs-0.20.0/src/world/mod.rs:314`）。
pub fn spawn_tower_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let tile = parse_tile_param(params.as_ref())?;

    // 关卡未加载（P12 起 LevelSelect 态）→ 结构性拒绝而非 panic。
    let (placeable, map_width, map_height, gold_before, tower_index, tower_key, def) = {
        let Some(map) = world.get_resource::<MapLayout>() else {
            return Ok(json!({
                "placed": false,
                "reason": "no level loaded (MapLayout missing)",
                "tile": tile,
            }));
        };
        let Some(registry) = world.get_resource::<crate::tower::TowerRegistry>() else {
            return Ok(json!({
                "placed": false,
                "reason": "tower registry missing",
                "tile": tile,
            }));
        };
        let selected = world.resource::<crate::tower::SelectedTowerType>();
        let tower_index = selected.0;
        let def = registry.get(tower_index).clone();
        let tower_key = def.key.clone();
        let placed = world.resource::<PlacedTowers>();
        let gold = world
            .get_resource::<crate::economy::Gold>()
            .map(|g| g.0)
            .unwrap_or(f32::INFINITY);
        (
            tower::is_placeable(map, placed, tile),
            map.width as f32,
            map.height as f32,
            gold,
            tower_index,
            tower_key,
            def,
        )
    };

    if !placeable {
        return Ok(json!({
            "placed": false,
            "reason": "tile is not an unoccupied grass tile",
            "tile": tile,
        }));
    }

    // 教程 14 Step 7 + 16：买不起即拒绝（价格来自注册表 `def.cost`）。
    let cost = def.cost;
    if gold_before < cost as f32 {
        return Ok(json!({
            "placed": false,
            "reason": "not enough gold",
            "tile": tile,
            "gold": gold_before,
            "cost": cost,
        }));
    }

    world.resource_mut::<crate::economy::Gold>().0 -= cost as f32;
    world.resource_mut::<PlacedTowers>().0.insert(tile);
    let atlas = world.resource::<TowerAtlas>().clone();
    let mut commands = world.commands();
    let pos = tower::spawn_tower_entities(
        &mut commands,
        &atlas,
        &def,
        tower_index,
        tile,
        map_width,
        map_height,
    );
    drop(commands);
    // 立即应用命令队列：返回后实体对已真实入世界（断言读取不等下一帧）。
    world.flush();

    info!(
        "[TD] tower '{tower_key}' placed at tile {tile:?} (world {}, {})",
        pos.x, pos.y
    );
    Ok(json!({
        "placed": true,
        "tile": tile,
        "pos": [pos.x, pos.y],
        "cost": cost,
        "tower_type": tower_key,
        "tower_index": tower_index,
    }))
}

/// `td.economy` handler（P14 起）：金币与常量面的只读快照。
///
/// 响应：`{"available", "gold", "starting_gold", "passive_rate", "tower_cost"}`；
/// 资源缺失（非 InGame 态）→ `available=false` + 各值 null。
/// `tower_cost` 在 P16 数据驱动化后读注册表选中项（价格不再有硬编码常量）。
pub fn economy_handler(In(_params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let gold = world.get_resource::<crate::economy::Gold>().map(|g| g.0);
    let tower_cost = world
        .get_resource::<crate::tower::TowerRegistry>()
        .and_then(|registry| {
            world
                .get_resource::<crate::tower::SelectedTowerType>()
                .map(|selected| registry.get(selected.0).cost)
        });
    Ok(json!({
        "available": gold.is_some(),
        "gold": gold,
        "starting_gold": crate::economy::STARTING_GOLD,
        "passive_rate": crate::economy::PASSIVE_INCOME_RATE,
        "tower_cost": tower_cost,
    }))
}

/// `td.wave` handler（P10）：刷怪调度的只读快照。
///
/// 响应：`{"available": true, "elapsed", "pending", "spawned", "total"}`；
/// 资源缺失（P12 起非 InGame 态被 `cleanup_level` 移除）→ `{"available": false}`。
/// 调度是「已刷 = 总数 − 待刷」的唯一记账源，断言脚本据此对拍事件时间表。
pub fn wave_handler(In(_params): In<Option<Value>>, world: &mut World) -> BrpResult {
    match world.get_resource::<crate::enemy::SpawnSchedule>() {
        Some(schedule) => Ok(json!({
            "available": true,
            "elapsed": schedule.elapsed,
            "pending": schedule.pending(),
            "spawned": schedule.spawned,
            "total": schedule.pending() + schedule.spawned as usize,
        })),
        None => Ok(json!({ "available": false })),
    }
}

/// `td.game` handler（P11 起；P12 换读 `GameFinished`）：胜负与波次记账的只读快照。
///
/// 响应：`{"available", "lives", "game_over", "pending", "spawned", "total", "elapsed"}`；
/// 资源缺失时对应字段为 `null`（P12 起非 InGame 态资源被清理，handler 不得 panic）。
/// `game_over` 由 `GameFinished` 资源的存在性表达（P12 起取代 P11 的 `GameOver` 标记）。
pub fn game_handler(In(_params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let lives = world
        .get_resource::<crate::enemy::BaseLives>()
        .map(|l| l.0);
    let game_over = world
        .get_resource::<crate::enemy::GameFinished>()
        .is_some();
    let wave = world
        .get_resource::<crate::enemy::SpawnSchedule>()
        .map(|s| {
            json!({
                "pending": s.pending(),
                "spawned": s.spawned,
                "total": s.pending() + s.spawned as usize,
                "elapsed": s.elapsed,
            })
        });
    let available = lives.is_some() || wave.is_some();
    Ok(json!({
        "available": available,
        "lives": lives,
        "game_over": game_over,
        "pending": wave.as_ref().map(|w| w["pending"].clone()),
        "spawned": wave.as_ref().map(|w| w["spawned"].clone()),
        "total": wave.as_ref().map(|w| w["total"].clone()),
        "elapsed": wave.as_ref().map(|w| w["elapsed"].clone()),
    }))
}

/// `td.state` 的参数解析结果。
enum StateOp {
    Get,
    Set(GameState),
}

/// 解析 `td.state` 的 params（严格分支）：缺省 / `{"op":"get"}` = 查询；
/// `{"op":"set","state":"InGame"|"LevelSelect"|"GameOver"}` = 请求转移。
fn parse_state_op(params: Option<&Value>) -> Result<StateOp, BrpError> {
    let Some(value) = params else {
        return Ok(StateOp::Get);
    };
    if value.is_null() {
        return Ok(StateOp::Get);
    }
    let Value::Object(map) = value else {
        return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
    };
    let op = match map.get("op") {
        None | Some(Value::Null) => "get",
        Some(Value::String(s)) => s.as_str(),
        Some(other) => {
            return Err(invalid_params(&format!("op 须为字符串（收到 {other}）")));
        }
    };
    match op {
        "get" => Ok(StateOp::Get),
        "set" => {
            let Some(Value::String(name)) = map.get("state") else {
                return Err(invalid_params("set 需要 state 字符串（LevelSelect/InGame/GameOver）"));
            };
            let target = match name.as_str() {
                "LevelSelect" => GameState::LevelSelect,
                "InGame" => GameState::InGame,
                "GameOver" => GameState::GameOver,
                other => {
                    return Err(invalid_params(&format!(
                        "未知 state `{other}`（允许 LevelSelect/InGame/GameOver）"
                    )));
                }
            };
            Ok(StateOp::Set(target))
        }
        other => Err(invalid_params(&format!("未知 op `{other}`（允许 get / set）"))),
    }
}

/// 状态枚举 → 稳定字符串（断言脚本口径，与 Debug 输出一致）。
fn state_name(state: GameState) -> &'static str {
    match state {
        GameState::LevelSelect => "LevelSelect",
        GameState::InGame => "InGame",
        GameState::GameOver => "GameOver",
    }
}

/// `td.state` handler（P12）：状态机读写通道。
///
/// - `{"op":"get"}`（缺省）→ `{"available", "state", "result", "finished"}`；
/// - `{"op":"set","state":…}` → 写 `NextState`（帧末 `StateTransition` 生效），
///   返回 `{"queued": true, "from", "to"}`。
///
/// 为什么需要：BRP 面没有键盘注入，而本阶段起玩法只在 InGame 态运行——
/// 证据脚本必须能确定性地进入/离开各状态（`td.` 前缀纪律，见 M6 选品 §五）。
pub fn state_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let op = parse_state_op(params.as_ref())?;
    let current = world
        .get_resource::<State<GameState>>()
        .map(|s| *s.get());
    match op {
        StateOp::Get => {
            let result = world.get_resource::<crate::GameResult>().map(|r| match r {
                crate::GameResult::Victory => "Victory",
                crate::GameResult::Defeat => "Defeat",
            });
            let finished = world
                .get_resource::<crate::enemy::GameFinished>()
                .is_some();
            Ok(json!({
                "available": current.is_some(),
                "state": current.map(state_name),
                "result": result,
                "finished": finished,
            }))
        }
        StateOp::Set(target) => {
            let Some(current) = current else {
                return Err(invalid_params("状态机未初始化（State<GameState> 缺失）"));
            };
            world.resource_mut::<NextState<GameState>>().set(target);
            info!(
                "[TD] state transition queued: {} -> {}",
                state_name(current),
                state_name(target)
            );
            Ok(json!({
                "queued": true,
                "from": state_name(current),
                "to": state_name(target),
            }))
        }
    }
}

/// `td.level` 的参数解析结果。
enum LevelOp {
    List,
    SelectIndex(usize),
    SelectPath(String),
}

/// 解析 `td.level` 的 params（严格分支）：缺省 / `{"op":"list"}` = 列表；
/// `{"op":"select","index":n}` / `{"op":"select","path":"..."}` = 切换选中关卡。
fn parse_level_op(params: Option<&Value>) -> Result<LevelOp, BrpError> {
    let Some(value) = params else {
        return Ok(LevelOp::List);
    };
    if value.is_null() {
        return Ok(LevelOp::List);
    }
    let Value::Object(map) = value else {
        return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
    };
    let op = match map.get("op") {
        None | Some(Value::Null) => "list",
        Some(Value::String(s)) => s.as_str(),
        Some(other) => {
            return Err(invalid_params(&format!("op 须为字符串（收到 {other}）")));
        }
    };
    match op {
        "list" => Ok(LevelOp::List),
        "select" => {
            let index = map.get("index");
            let path = map.get("path");
            match (index, path) {
                (Some(Value::Number(n)), None) => {
                    let Some(i) = n.as_u64() else {
                        return Err(invalid_params("index 须为非负整数"));
                    };
                    Ok(LevelOp::SelectIndex(i as usize))
                }
                (None, Some(Value::String(s))) => Ok(LevelOp::SelectPath(s.clone())),
                (Some(_), Some(_)) => Err(invalid_params(
                    "index 与 path 只能给一个（二者同给歧义）",
                )),
                _ => Err(invalid_params(
                    "select 需要 index（非负整数）或 path（字符串）",
                )),
            }
        }
        other => Err(invalid_params(&format!("未知 op `{other}`（允许 list / select）"))),
    }
}

/// `td.level` handler（P13）：关卡发现列表读取 / 选中项切换。
///
/// - `{}` / `{"op":"list"}` → `{"available", "levels", "selected"}`；
/// - `{"op":"select",…}` → 写 `SelectedLevel`，返回 `{"selected", "index"}`。
///
/// 为什么需要：BRP 面无键盘注入，而 P13 起关卡由菜单按键选择——证据脚本
/// 必须能确定性地切换关卡（`td.` 前缀纪律，见 M6 选品 §五）。
pub fn level_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let op = parse_level_op(params.as_ref())?;
    let levels: Vec<String> = world
        .get_resource::<crate::level_select::AvailableLevels>()
        .map(|l| l.0.clone())
        .unwrap_or_default();
    let selected = world
        .get_resource::<crate::level_select::SelectedLevel>()
        .map(|s| s.0.clone());

    match op {
        LevelOp::List => Ok(json!({
            "available": !levels.is_empty(),
            "levels": levels,
            "selected": selected,
        })),
        LevelOp::SelectIndex(index) => {
            let Some(path) = levels.get(index).cloned() else {
                return Err(invalid_params(&format!(
                    "index {index} 越界（发现 {} 个关卡）",
                    levels.len()
                )));
            };
            set_selected_level(world, &path)?;
            Ok(json!({ "selected": path, "index": index }))
        }
        LevelOp::SelectPath(path) => {
            let Some(index) = levels.iter().position(|p| *p == path) else {
                return Err(invalid_params(&format!(
                    "关卡 `{path}` 不在发现列表（{:?}）",
                    levels
                )));
            };
            set_selected_level(world, &path)?;
            Ok(json!({ "selected": path, "index": index }))
        }
    }
}

/// 写入 `SelectedLevel`（P13）：资源由 `init_resource` 保证存在。
fn set_selected_level(world: &mut World, path: &str) -> Result<(), BrpError> {
    let mut selected = world.resource_mut::<crate::level_select::SelectedLevel>();
    selected.0 = path.to_string();
    info!("[TD] selected level -> {path}");
    Ok(())
}

/// `td.combat` handler（P15 起；P16 换读 `AmmoState`）：弹道 / 弹药 / 战斗常量面的只读快照。
///
/// 响应：`{"available", "launchers", "projectiles", "exploding", "ammo_filled",
/// "ammo_capacity", "fired", "rocket_damage", "splash_radius", "rocket_speed"}`；
/// 计数为 0 也返回（资源缺失不 panic）。弹药「填充数 + 飞行中」与 `fired` 一起
/// 支撑「发射 → 减弹 → 补弹」链的采样断言。P16 起火箭数值来自注册表 `rocket`
/// 条目（数据驱动，不再有 Rust 常量）。
pub fn combat_handler(In(_params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let mut launchers = 0usize;
    let mut ammo_filled = 0usize;
    let mut ammo_capacity = 0usize;
    let mut fired = 0u64;
    let mut query = world.query::<&crate::tower::AmmoState>();
    for ammo in query.iter(world) {
        launchers += 1;
        ammo_filled += ammo.slots.iter().filter(|slot| slot.is_some()).count();
        ammo_capacity += ammo.slots.len();
        fired += ammo.fired as u64;
    }

    let projectiles = world.query::<&crate::tower::Projectile>().iter(world).count();
    let exploding = world.query::<&crate::tower::Exploding>().iter(world).count();

    let rocket = world
        .get_resource::<crate::tower::TowerRegistry>()
        .and_then(|registry| registry.towers.iter().find(|def| def.key == "rocket"))
        .and_then(|def| def.projectile.as_ref())
        .map(|p| {
            json!({
                "damage": p.damage,
                "speed": p.speed,
                "splash_radius": p.splash_radius,
            })
        });

    Ok(json!({
        "available": true,
        "launchers": launchers,
        "projectiles": projectiles,
        "exploding": exploding,
        "ammo_filled": ammo_filled,
        "ammo_capacity": ammo_capacity,
        "fired": fired,
        "rocket_damage": rocket.as_ref().map(|r| r["damage"].clone()),
        "splash_radius": rocket.as_ref().map(|r| r["splash_radius"].clone()),
        "rocket_speed": rocket.as_ref().map(|r| r["speed"].clone()),
    }))
}

/// `td.towers` handler（P16 起；P17 改向量序）：数据驱动塔注册表的只读快照。
///
/// 响应：`{"available", "selected_index", "selected", "towers": [...]}`，每塔一项含
/// `index` / 键 / 名称 / 数值 / 外观索引 / 弹体定义（`projectile` 为 null 表示瞬时塔）。
/// P17 起注册表是 `Vec`：输出序 = 向量序（= 选择 UI 槽位序 = 数字键下标；
/// 装载序来自 TOML 键的 HashMap 迭代，教程 17 明确「order is arbitrary, but
/// stable for the rest of the session」——断言按 key 反查 index，不假设顺序）。
pub fn towers_handler(In(_params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Some(registry) = world.get_resource::<crate::tower::TowerRegistry>() else {
        return Ok(json!({ "available": false, "selected": null, "selected_index": null, "towers": [] }));
    };
    let selected_index = world
        .get_resource::<crate::tower::SelectedTowerType>()
        .map(|s| s.0);
    let selected = selected_index.map(|i| registry.get(i).key.clone());

    let towers: Vec<Value> = registry
        .towers
        .iter()
        .enumerate()
        .map(|(index, def)| {
            json!({
                "index": index,
                "key": def.key,
                "name": def.name,
                "description": def.description,
                "base_sprite": def.base_sprite,
                "top_sprite": def.top_sprite,
                "preview_top_sprite": def.preview_top_sprite,
                "cost": def.cost,
                "attack_range": def.attack_range,
                "attack_cooldown": def.attack_cooldown,
                "damage": def.damage,
                "muzzle_flash_sprite": def.muzzle_flash_sprite,
                "projectile": def.projectile.as_ref().map(|p| json!({
                    "damage": p.damage,
                    "speed": p.speed,
                    "sprite": p.sprite,
                    "explosion_sprite": p.explosion_sprite,
                    "splash_radius": p.splash_radius,
                })),
                "ammo_slots": def.ammo_slot_offsets.as_ref().map(|o| o.len()),
                "ammo_refill_secs": def.ammo_refill_secs,
            })
        })
        .collect();

    Ok(json!({
        "available": true,
        "selected_index": selected_index,
        "selected": selected,
        "towers": towers,
    }))
}

/// `td.tower_select` 的参数解析结果。
enum TowerSelectOp {
    Get,
    SetIndex(usize),
    SetKey(String),
}

/// 解析 `td.tower_select` 的 params（严格分支）：缺省 / `{"op":"get"}` = 查询；
/// `{"op":"set","index":n}` / `{"op":"set","key":"rocket"}` = 切换选中塔型。
fn parse_tower_select_op(params: Option<&Value>) -> Result<TowerSelectOp, BrpError> {
    let Some(value) = params else {
        return Ok(TowerSelectOp::Get);
    };
    if value.is_null() {
        return Ok(TowerSelectOp::Get);
    }
    let Value::Object(map) = value else {
        return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
    };
    let op = match map.get("op") {
        None | Some(Value::Null) => "get",
        Some(Value::String(s)) => s.as_str(),
        Some(other) => {
            return Err(invalid_params(&format!("op 须为字符串（收到 {other}）")));
        }
    };
    match op {
        "get" => Ok(TowerSelectOp::Get),
        "set" => match (map.get("index"), map.get("key")) {
            (Some(Value::Number(n)), None) => {
                let Some(i) = n.as_u64() else {
                    return Err(invalid_params("index 须为非负整数"));
                };
                Ok(TowerSelectOp::SetIndex(i as usize))
            }
            (None, Some(Value::String(s))) => Ok(TowerSelectOp::SetKey(s.clone())),
            (Some(_), Some(_)) => Err(invalid_params("index 与 key 只能给一个（二者同给歧义）")),
            _ => Err(invalid_params("set 需要 index（非负整数）或 key（字符串）")),
        },
        other => Err(invalid_params(&format!("未知 op `{other}`（允许 get / set）"))),
    }
}

/// `td.tower_select` handler（P17）：塔选下标读写（U6 侧信道）。
///
/// - `{}` / `{"op":"get"}` → `{"selected_index", "key", "len"}`；
/// - `{"op":"set",…}` → 写 `SelectedTowerType`，返回同形快照。
///
/// 为什么需要：BRP 面无键鼠注入，而 P17 的选择由滚轮 / 数字键 / 槽位点击驱动
/// ——证据脚本必须能确定性地切换选中塔型并读取结果；UI 视觉态另由截图留证。
pub fn tower_select_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let op = parse_tower_select_op(params.as_ref())?;
    let Some(registry) = world.get_resource::<crate::tower::TowerRegistry>() else {
        return Ok(json!({ "available": false, "selected_index": null, "key": null, "len": 0 }));
    };
    let len = registry.towers.len();

    match op {
        TowerSelectOp::Get => {}
        TowerSelectOp::SetIndex(index) => {
            if index >= len {
                return Err(invalid_params(&format!(
                    "index {index} 越界（注册表 {} 项）",
                    len
                )));
            }
            world.resource_mut::<crate::tower::SelectedTowerType>().0 = index;
        }
        TowerSelectOp::SetKey(key) => {
            let Some(index) = registry.index_of_key(&key) else {
                return Err(invalid_params(&format!("塔型 `{key}` 不在注册表")));
            };
            world.resource_mut::<crate::tower::SelectedTowerType>().0 = index;
        }
    }

    let selected_index = world.resource::<crate::tower::SelectedTowerType>().0;
    let key = world
        .get_resource::<crate::tower::TowerRegistry>()
        .map(|r| r.get(selected_index).key.clone());
    info!("[TD] tower selected -> index {selected_index} ({key:?})");
    Ok(json!({
        "available": true,
        "selected_index": selected_index,
        "key": key,
        "len": len,
    }))
}

/// 本模块族共用的参数错误（JSON-RPC INVALID_PARAMS -32602）。
fn invalid_params(message: &str) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message: message.to_string(),
        data: None,
    }
}

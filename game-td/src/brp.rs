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
//!
//! 测试通道 `td.events`（P18）：自定义消息链（教程 18 `PlaceTower`）的只读快照 +
//! 确定性发布入口。BRP 无法注入鼠标左键，点击生产者不可直达，故：
//! - `{}` / `{"op":"snapshot"}` → 计数面（produced/injected/spawned/gold_applied）
//!   + 两个消费者最后消费的载荷快照 + 已占格数 / 余额；
//! - `{"op":"emit_place","tile":[x,y],"tower_id":n?}` → 按生产者同一校验
//!   （`tower::is_placeable` + 余额门）写一条 `PlaceTower` 消息（`World::write_message`
//!   `bevy_ecs-0.20.0/src/world/mod.rs:3041`）——两个消费者随后各自消费，
//!   「发布 → 双消费者响应」闭环因此可被确定性断言。
//! - 非法 tile / tower_id 越界 → -32602；不可放置 → `{"queued": false, "reason": …}`。

use bevy::prelude::*;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::{error_codes, BrpError, BrpResult, RemotePlugin};
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
// P22：`World::run_system_once`（一次性运行具名 run 条件本身）——
// trait `RunSystemOnce` 不在 prelude，须显式引入
// （`bevy_ecs-0.20.0/src/system/system.rs:331`）。
use bevy::ecs::system::RunSystemOnce;
use serde::Serialize;
use serde_json::{json, Value};

// `PlaybackMode` 不在 prelude（P20 `td.audio` 播放模式字符串化用）。
use bevy::audio::PlaybackMode;

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

/// `td.events` 方法名（P18 起：自定义消息链快照 + 确定性发布入口）。
pub const TD_EVENTS_METHOD: &str = "td.events";

/// `td.gizmos` 方法名（P19 起：射程圈绘制参数的只读快照 + 单格探测）。
pub const TD_GIZMOS_METHOD: &str = "td.gizmos";

/// `td.audio` 方法名（P20 起：音频实体 / 播放态 / 触发计数的只读快照）。
pub const TD_AUDIO_METHOD: &str = "td.audio";

/// `td.pause` 方法名（P21 起：暂停状态读写——BRP 无键盘注入，Escape 不可直达）。
pub const TD_PAUSE_METHOD: &str = "td.pause";

/// `td.conditions` 方法名（P22 起：具名 run 条件 / 系统集合的可观察面）。
///
/// 为什么需要：P22 是**纯重构**（`run_if` → `configure_sets` + `in_set`），行为面
/// 应与 P21 逐位一致；但「条件真的挂在集合上」需要一个非行为断言面：
/// - `game_is_running` = **直接运行具名条件函数本身**（`World::run_system_once`
///   `bevy_ecs-0.20.0/src/system/system.rs:331`）——不是复述其逻辑；
/// - `ticks` = 三个集合各自成员微系统（`pause::tick_*`）的推进计数——若
///   `configure_sets` 漏挂，计数器在 LevelSelect / 暂停中仍会推进（差异可见）。
pub const TD_CONDITIONS_METHOD: &str = "td.conditions";

/// `td.level_select` 方法名（P23 起：菜单按钮网格 / 焦点 / 交互态的可观察面）。
///
/// 为什么需要：BRP 无鼠标注入，而 P23 的核心是「按钮 + `Interaction` 三态」。
/// - `snapshot` → 按钮结构面（`LevelButton` 下标 / 焦点 / `Interaction` / 背景色）；
/// - `focus` → 写 `FocusedLevel`（键盘导航与视觉系统共享的同一状态）；
/// - `click` → 写目标按钮 `Interaction::Pressed` 后**运行真实的
///   `handle_level_button_click` 一次**（`run_system_once`；`Changed<Interaction>`
///   对该新实例恒真——`function_system.rs:781` 把 `last_run` 置为「MAX 之前」）——
///   鼠标通道的消费者逻辑全走真码路径；
/// - `confirm` → Enter/Space 分支共用的 `apply_level_selection`。
pub const TD_LEVEL_SELECT_METHOD: &str = "td.level_select";

/// `td.input` 方法名（P24 起：逻辑动作注入 + 虚拟光标读写——教程 24 的确定性入口）。
///
/// 为什么需要：BRP 无键鼠 / 手柄注入通道；P24 起全部输入面收敛为
/// `GameAction` + `VirtualCursorPos`，注入逻辑动作即等价于一次真实按键 / 点击，
/// 下游消费者（走格 / 选塔 / 菜单导航 / 暂停 / 放置）全走真码路径。
pub const TD_INPUT_METHOD: &str = "td.input";

/// `td.gamepad` 方法名（P25 起：手柄连接状态 / 绑定表 / 摇杆判别的可观察面）。
///
/// 为什么需要：本机可能无手柄（行为面无法实机断言）——按立项「映射表 + 模拟路径」
/// 落地：结构面（连接数 / 输入模块装配 / 绑定表 / 节拍参数）+ **纯函数探测**
/// （`probe_button` / `probe_stick` 调与读取器同一函数）。
pub const TD_GAMEPAD_METHOD: &str = "td.gamepad";

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
            .register_type::<crate::tower::TowerDockSlot>()
            // P23：菜单按钮下标（按钮网格结构面计数；同 U6 侧信道口径）。
            .register_type::<crate::level_select::LevelButton>();
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
                .with_method_main(TD_TOWER_SELECT_METHOD, tower_select_handler)
                .with_method_main(TD_EVENTS_METHOD, events_handler)
                .with_method_main(TD_GIZMOS_METHOD, gizmos_handler)
                .with_method_main(TD_AUDIO_METHOD, audio_handler)
                .with_method_main(TD_PAUSE_METHOD, pause_handler)
                .with_method_main(TD_CONDITIONS_METHOD, conditions_handler)
                .with_method_main(TD_LEVEL_SELECT_METHOD, level_select_handler)
                .with_method_main(TD_INPUT_METHOD, input_handler)
                .with_method_main(TD_GAMEPAD_METHOD, gamepad_handler),
        );
        app.add_plugins(RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST));
        info!("[BRP] listening on 127.0.0.1:15702 (explicit loopback bind)");
        info!(
            "[BRP] custom methods: {TD_SCREENSHOT_METHOD}, {TD_SPAWN_TOWER_METHOD}, \
             {TD_WAVE_METHOD}, {TD_GAME_METHOD}, {TD_STATE_METHOD}, {TD_LEVEL_METHOD}, \
             {TD_ECONOMY_METHOD}, {TD_COMBAT_METHOD}, {TD_TOWERS_METHOD}, \
             {TD_TOWER_SELECT_METHOD}, {TD_EVENTS_METHOD}, {TD_GIZMOS_METHOD}, \
             {TD_AUDIO_METHOD}, {TD_PAUSE_METHOD}, {TD_CONDITIONS_METHOD}, \
             {TD_LEVEL_SELECT_METHOD}, {TD_INPUT_METHOD}, {TD_GAMEPAD_METHOD}"
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
    let atlas = world.resource::<TowerAtlas>().clone();
    let mut commands = world.commands();
    let (pos, turret) = tower::spawn_tower_entities(
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
    // P19：占格表登记（tile → 塔顶实体，教程 19 HashMap 语义）。
    world
        .resource_mut::<PlacedTowers>()
        .0
        .insert(tile, turret);

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

// ---------------------------------------------------------------------------
// P18：td.events（自定义消息链，教程 18）
// ---------------------------------------------------------------------------

/// `td.events` 的参数解析结果。
enum EventsOp {
    Snapshot,
    EmitPlace {
        tile: [u32; 2],
        tower_id: Option<usize>,
    },
}

/// 解析 `td.events` 的 params（严格分支）：缺省 / `{"op":"snapshot"}` = 快照；
/// `{"op":"emit_place","tile":[x,y],"tower_id":n?}` = 写一条 `PlaceTower` 消息。
fn parse_events_op(params: Option<&Value>) -> Result<EventsOp, BrpError> {
    let Some(value) = params else {
        return Ok(EventsOp::Snapshot);
    };
    if value.is_null() {
        return Ok(EventsOp::Snapshot);
    }
    let Value::Object(map) = value else {
        return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
    };
    let op = match map.get("op") {
        None | Some(Value::Null) => "snapshot",
        Some(Value::String(s)) => s.as_str(),
        Some(other) => {
            return Err(invalid_params(&format!("op 须为字符串（收到 {other}）")));
        }
    };
    match op {
        "snapshot" => Ok(EventsOp::Snapshot),
        "emit_place" => {
            let tile = parse_tile_param(Some(value))?;
            let tower_id = match map.get("tower_id") {
                None | Some(Value::Null) => None,
                Some(Value::Number(n)) => {
                    let Some(i) = n.as_u64() else {
                        return Err(invalid_params("tower_id 须为非负整数"));
                    };
                    Some(i as usize)
                }
                Some(other) => {
                    return Err(invalid_params(&format!(
                        "tower_id 须为非负整数（收到 {other}）"
                    )));
                }
            };
            Ok(EventsOp::EmitPlace { tile, tower_id })
        }
        other => Err(invalid_params(&format!(
            "未知 op `{other}`（允许 snapshot / emit_place）"
        ))),
    }
}

/// 载荷快照 → JSON（`PlaceTowerRecord` 序列化形态；两消费者逐字段对拍用）。
fn record_json(record: &crate::tower::PlaceTowerRecord) -> Value {
    json!({
        "tile": record.tile,
        "world_pos": record.world_pos,
        "tower_id": record.tower_id,
        "cost": record.cost,
    })
}

/// `td.events` handler（P18）：消息链快照 / 确定性发布（见模块文档）。
///
/// 为什么需要：BRP 无鼠标注入，点击生产者不可直达；`emit_place` 复用生产者的
/// 同一校验（`is_placeable` + 余额门）写消息，两个消费者（生成 / 扣费）对同一条
/// 消息各消费一次——计数与载荷快照即「发布 → 响应」闭环的机器可读证据。
pub fn events_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let op = parse_events_op(params.as_ref())?;

    match op {
        EventsOp::Snapshot => Ok(events_snapshot(world)),
        EventsOp::EmitPlace { tile, tower_id } => {
            // 先完成全部只读校验（与 td.spawn_tower 同源谓词），再写消息。
            let (map_width, map_height, tower_index, def, gold_before) = {
                let Some(map) = world.get_resource::<MapLayout>() else {
                    return Ok(json!({
                        "queued": false,
                        "reason": "no level loaded (MapLayout missing)",
                        "tile": tile,
                    }));
                };
                let Some(registry) = world.get_resource::<crate::tower::TowerRegistry>() else {
                    return Ok(json!({
                        "queued": false,
                        "reason": "tower registry missing",
                        "tile": tile,
                    }));
                };
                let tower_index = match tower_id {
                    Some(index) => index,
                    None => world
                        .get_resource::<crate::tower::SelectedTowerType>()
                        .map(|s| s.0)
                        .unwrap_or(0),
                };
                let Some(def) = registry.towers.get(tower_index) else {
                    return Err(invalid_params(&format!(
                        "tower_id {tower_index} 越界（注册表 {} 项）",
                        registry.towers.len()
                    )));
                };
                let def = def.clone();
                let placed = world.resource::<PlacedTowers>();
                if !tower::is_placeable(map, placed, tile) {
                    return Ok(json!({
                        "queued": false,
                        "reason": "tile is not an unoccupied grass tile",
                        "tile": tile,
                    }));
                }
                let gold = world
                    .get_resource::<crate::economy::Gold>()
                    .map(|g| g.0)
                    .unwrap_or(f32::INFINITY);
                (
                    map.width as f32,
                    map.height as f32,
                    tower_index,
                    def,
                    gold,
                )
            };

            if gold_before < def.cost as f32 {
                return Ok(json!({
                    "queued": false,
                    "reason": "not enough gold",
                    "tile": tile,
                    "gold": gold_before,
                    "cost": def.cost,
                }));
            }

            let pos = crate::enemy::tile_to_world(tile, map_width, map_height);
            let written = world.write_message(crate::tower::PlaceTower {
                tile,
                world_pos: pos,
                tower_id: tower_index,
                cost: def.cost,
            });
            if written.is_none() {
                return Err(invalid_params(
                    "PlaceTower 消息队列未注册（add_message 缺失）",
                ));
            }
            world.resource_mut::<crate::tower::PlacementLog>().injected += 1;

            info!(
                "[TD] PlaceTower message injected at tile {tile:?} (tower {tower_index}, cost {})",
                def.cost
            );
            Ok(json!({
                "queued": true,
                "tile": tile,
                "tower_id": tower_index,
                "tower_key": def.key,
                "cost": def.cost,
                "world_pos": [pos.x, pos.y],
            }))
        }
    }
}

/// `td.events` 快照（只读；资源缺失时字段为 null，不 panic）。
fn events_snapshot(world: &World) -> Value {
    let log = world.get_resource::<crate::tower::PlacementLog>();
    let placed = world
        .get_resource::<PlacedTowers>()
        .map(|p| p.0.len())
        .unwrap_or(0);
    let gold = world
        .get_resource::<crate::economy::Gold>()
        .map(|g| g.0);
    match log {
        Some(log) => json!({
            "available": true,
            "produced": log.produced,
            "injected": log.injected,
            "spawned": log.spawned,
            "gold_applied": log.gold_applied,
            "spawn_last": log.spawn_last.as_ref().map(record_json),
            "gold_last": log.gold_last.as_ref().map(record_json),
            "placed_tiles": placed,
            "gold": gold,
        }),
        None => json!({
            "available": false,
            "produced": null,
            "injected": null,
            "spawned": null,
            "gold_applied": null,
            "spawn_last": null,
            "gold_last": null,
            "placed_tiles": placed,
            "gold": gold,
        }),
    }
}

// ---------------------------------------------------------------------------
// P19：td.gizmos（射程圈绘制参数，教程 19）
// ---------------------------------------------------------------------------

/// `td.gizmos` 的参数解析结果。
enum GizmoOp {
    Snapshot,
    Probe { tile: [u32; 2] },
}

/// 解析 `td.gizmos` 的 params（严格分支）：缺省 / `{"op":"snapshot"}` = 上一帧
/// 绘制记录；`{"op":"probe","tile":[x,y]}` = 单格悬停圈参数探测。
fn parse_gizmo_op(params: Option<&Value>) -> Result<GizmoOp, BrpError> {
    let Some(value) = params else {
        return Ok(GizmoOp::Snapshot);
    };
    if value.is_null() {
        return Ok(GizmoOp::Snapshot);
    }
    let Value::Object(map) = value else {
        return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
    };
    let op = match map.get("op") {
        None | Some(Value::Null) => "snapshot",
        Some(Value::String(s)) => s.as_str(),
        Some(other) => {
            return Err(invalid_params(&format!("op 须为字符串（收到 {other}）")));
        }
    };
    match op {
        "snapshot" => Ok(GizmoOp::Snapshot),
        "probe" => Ok(GizmoOp::Probe {
            tile: parse_tile_param(Some(value))?,
        }),
        other => Err(invalid_params(&format!(
            "未知 op `{other}`（允许 snapshot / probe）"
        ))),
    }
}

/// 圈参数 → JSON（`td.gizmos` 响应元素）。
fn ring_json(ring: &crate::tower::RingParams) -> Value {
    let kind = match ring.kind {
        crate::tower::RingKind::Preview => "preview",
        crate::tower::RingKind::Hover => "hover",
    };
    json!({
        "center": ring.center,
        "radius": ring.radius,
        "color": ring.color,
        "kind": kind,
    })
}

/// `td.gizmos` handler（P19）：射程圈绘制参数的只读快照 / 单格探测。
///
/// 为什么需要：gizmo 是每帧重画的瞬时形状（无实体、无 BRP 注入光标通道），
/// 其行为等价面 = **绘制参数**（圆心 / 半径 / 颜色）。`draw_tower_ranges` 每帧把
/// 参数记入 [`crate::tower::RangeGizmoLog`]，`probe` 则对指定格调用绘制系统用的
/// **同一纯函数** `tower::hover_ring_params`——两条通道不会漂移。
pub fn gizmos_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let op = parse_gizmo_op(params.as_ref())?;
    match op {
        GizmoOp::Snapshot => {
            let log = world.get_resource::<crate::tower::RangeGizmoLog>();
            let rings: Vec<Value> = log
                .map(|l| l.rings.iter().map(ring_json).collect())
                .unwrap_or_default();
            Ok(json!({
                "available": log.is_some(),
                "preview_visible": log.map(|l| l.preview_visible),
                // P24：光标高亮格（`draw_cursor_highlight` 写；None = 无虚拟光标）。
                "cursor_highlight": log.and_then(|l| l.cursor_tile),
                "ring_count": rings.len(),
                "rings": rings,
            }))
        }
        GizmoOp::Probe { tile } => {
            // 同一纯函数 + 同一组件口径（`Transform` + `TowerAttacker`）。
            let mut towers = world
                .query::<(&Transform, &crate::tower::TowerAttacker)>();
            let placed = world.get_resource::<PlacedTowers>();
            let ring = placed.and_then(|placed| {
                crate::tower::hover_ring_params(placed, tile, |entity| {
                    towers
                        .get(world, entity)
                        .ok()
                        .map(|(transform, attacker)| (transform.translation.truncate(), attacker.range))
                })
            });
            match ring {
                Some(ring) => Ok(json!({
                    "available": true,
                    "tile": tile,
                    "ring": ring_json(&ring),
                })),
                None => Ok(json!({
                    "available": true,
                    "tile": tile,
                    "ring": null,
                    "reason": "no placed tower on tile",
                })),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// P20：td.audio（音频实体 / 播放态 / 触发计数，教程 20）
// ---------------------------------------------------------------------------

/// `td.audio` handler（P20）：音频结构面只读快照。
///
/// 口径（立项定案：音频非 M6 收官门槛，本部行为等价照做）——不断言可听性、
/// 不断言性能，只断言**可观察结构面**：
/// - `music_entities` / `music_mode` / `music_volume`：关卡音乐实体与 `LOOP`/0.3 配置；
/// - `music_sink`：Bevy 是否已为音乐建 `AudioSink`（无音频设备时该值为 false，环境注记）；
/// - `sfx_entities` / `sfx_sinks`：在场一次性音效实体（`DESPAWN` 播完自回收，采样可遇不可求，
///   故主判据是计数）；
/// - `laser_fire` / `rocket_explosion` / `sfx_spawned` / `last_volume`：音效触发计数
///   （`AudioLog`，消息消费端记账）；
/// - `projectile_audio`：挂循环推进器的在飞弹体数（教程 20 实体耦合模式的直接证据）。
pub fn audio_handler(In(_params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let available = world
        .get_resource::<crate::audio::AudioAssets>()
        .is_some();
    // 先把记账值拷出（释放 World 的不可变借用，后续需建查询）。
    let (laser_fire, rocket_explosion, sfx_spawned, last_volume) =
        match world.get_resource::<crate::audio::AudioLog>() {
            Some(log) => (
                Some(log.laser_fire),
                Some(log.rocket_explosion),
                Some(log.sfx_spawned),
                log.last_volume,
            ),
            None => (None, None, None, None),
        };

    let mut music_entities = 0u32;
    let mut music_mode: Option<&'static str> = None;
    let mut music_volume: Option<f32> = None;
    let mut music_sink = false;
    let mut playback_settings_entities = 0u32;
    let mut sfx_entities = 0u32;
    let mut sfx_sinks = 0u32;
    let mut audio_players_total = 0u32;

    {
        let mut query = world.query::<(
            &AudioPlayer,
            Option<&PlaybackSettings>,
            Option<&crate::audio::BackgroundMusic>,
            Option<&AudioSink>,
        )>();
        for (_player, settings, music_marker, sink) in query.iter(world) {
            audio_players_total += 1;
            if settings.is_some() {
                playback_settings_entities += 1;
            }
            match (music_marker, settings) {
                (Some(_), Some(settings)) => {
                    music_entities += 1;
                    music_mode = Some(match settings.mode {
                        PlaybackMode::Once => "Once",
                        PlaybackMode::Loop => "Loop",
                        PlaybackMode::Despawn => "Despawn",
                        PlaybackMode::Remove => "Remove",
                    });
                    music_volume = Some(settings.volume.to_linear());
                    music_sink |= sink.is_some();
                }
                (None, _) => {
                    sfx_entities += 1;
                    if sink.is_some() {
                        sfx_sinks += 1;
                    }
                }
                _ => {}
            }
        }
    }

    let projectile_audio = {
        let mut query = world
            .query_filtered::<Entity, (With<crate::tower::Projectile>, With<AudioPlayer>)>();
        query.iter(world).count()
    };

    Ok(json!({
        "available": available,
        "audio_players_total": audio_players_total,
        "playback_settings_entities": playback_settings_entities,
        "music_entities": music_entities,
        "music_mode": music_mode,
        "music_volume": music_volume,
        "music_sink": music_sink,
        "sfx_entities": sfx_entities,
        "sfx_sinks": sfx_sinks,
        "projectile_audio": projectile_audio,
        "laser_fire": laser_fire,
        "rocket_explosion": rocket_explosion,
        "sfx_spawned": sfx_spawned,
        "last_volume": last_volume,
        "sinks_supported": music_sink || sfx_sinks > 0,
    }))
}

// ---------------------------------------------------------------------------
// P21：td.pause（暂停状态读写，教程 21 PauseState）
// ---------------------------------------------------------------------------

/// `td.pause` 的参数解析结果。
#[derive(Clone, Copy)]
enum PauseOp {
    Get,
    Set(crate::pause::PauseState),
    Toggle,
}

/// 暂停状态 → 稳定字符串（断言脚本口径，与 Debug 输出一致）。
fn pause_state_name(state: crate::pause::PauseState) -> &'static str {
    match state {
        crate::pause::PauseState::Running => "Running",
        crate::pause::PauseState::Paused => "Paused",
    }
}

/// 解析 `td.pause` 的 params（严格分支）：缺省 / `{"op":"get"}` = 查询；
/// `{"op":"set","state":"Running"|"Paused"}` = 请求转移；`{"op":"toggle"}` = 翻转。
fn parse_pause_op(params: Option<&Value>) -> Result<PauseOp, BrpError> {
    let Some(value) = params else {
        return Ok(PauseOp::Get);
    };
    if value.is_null() {
        return Ok(PauseOp::Get);
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
        "get" => Ok(PauseOp::Get),
        "toggle" => Ok(PauseOp::Toggle),
        "set" => {
            let Some(Value::String(name)) = map.get("state") else {
                return Err(invalid_params("set 需要 state 字符串（Running/Paused）"));
            };
            let target = match name.as_str() {
                "Running" => crate::pause::PauseState::Running,
                "Paused" => crate::pause::PauseState::Paused,
                other => {
                    return Err(invalid_params(&format!(
                        "未知 state `{other}`（允许 Running/Paused）"
                    )));
                }
            };
            Ok(PauseOp::Set(target))
        }
        other => Err(invalid_params(&format!(
            "未知 op `{other}`（允许 get / set / toggle）"
        ))),
    }
}

/// `td.pause` handler（P21）：暂停状态读写通道。
///
/// 为什么需要：BRP 无键盘注入，Escape 不可直达；暂停/恢复的**效果**（实体冻结 /
/// 覆盖层 / 恢复推进）必须能确定性触发与采样（`td.` 前缀纪律）。
/// 守卫与 `toggle_pause` 对齐：只在 `GameState::InGame` 受理（教程 21
/// 「Only works during a level」），否则 -32602；`GameFinished` 已插入时拒绝
/// （教程 21 竞态守卫的同义表达——结算迁移待生效的窗口内不允许暂停）。
pub fn pause_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let op = parse_pause_op(params.as_ref())?;
    let pause_state = world
        .get_resource::<State<crate::pause::PauseState>>()
        .map(|s| *s.get());
    let game_state = world
        .get_resource::<State<GameState>>()
        .map(|s| *s.get());

    let overlay = {
        let mut query = world
            .query_filtered::<Entity, With<crate::pause::PauseOverlay>>();
        query.iter(world).count()
    };

    match op {
        PauseOp::Get => Ok(json!({
            "available": pause_state.is_some(),
            "state": pause_state.map(pause_state_name),
            "game_state": game_state.map(state_name),
            "overlay": overlay,
        })),
        PauseOp::Set(_) | PauseOp::Toggle => {
            let Some(current) = pause_state else {
                return Err(invalid_params(
                    "暂停状态机未初始化（State<PauseState> 缺失）",
                ));
            };
            if game_state != Some(GameState::InGame) {
                return Err(invalid_params(
                    "暂停仅在 InGame 受理（教程 21：Only works during a level）",
                ));
            }
            if world
                .get_resource::<crate::enemy::GameFinished>()
                .is_some()
            {
                return Err(invalid_params(
                    "本局已判定结束（GameFinished）——暂停窗口已关闭",
                ));
            }
            let target = match op {
                PauseOp::Set(target) => target,
                _ => match current {
                    crate::pause::PauseState::Running => crate::pause::PauseState::Paused,
                    crate::pause::PauseState::Paused => crate::pause::PauseState::Running,
                },
            };
            world
                .resource_mut::<NextState<crate::pause::PauseState>>()
                .set(target);
            info!(
                "[TD] pause transition queued: {} -> {}",
                pause_state_name(current),
                pause_state_name(target)
            );
            Ok(json!({
                "queued": true,
                "from": pause_state_name(current),
                "to": pause_state_name(target),
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// P22：td.conditions（具名 run 条件 / 系统集合，教程 22）
// ---------------------------------------------------------------------------

/// `td.conditions` handler（P22）：具名 run 条件与集合计数的可观察面。
///
/// - `{}` / `{"op":"snapshot"}` → `{game_state, pause_state, game_is_running, ticks}`；
/// - 其它 op → -32602。
///
/// `game_is_running` 的真值来自**运行条件函数本身**（`run_system_once`
/// `bevy_ecs-0.20.0/src/system/system.rs:331`；条件经 blanket `SystemCondition`
/// 实现 `schedule/condition.rs:600`）——handler 不复述判断逻辑，避免双实现漂移。
pub fn conditions_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    if let Some(value) = params.as_ref() {
        if !value.is_null() {
            let Value::Object(map) = value else {
                return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
            };
            match map.get("op") {
                None | Some(Value::Null) | Some(Value::String(_)) => {}
                Some(other) => {
                    return Err(invalid_params(&format!("op 须为字符串（收到 {other}）")));
                }
            }
            if let Some(op) = map.get("op").and_then(Value::as_str) {
                if op != "snapshot" {
                    return Err(invalid_params(&format!(
                        "未知 op `{op}`（允许 snapshot）"
                    )));
                }
            }
        }
    }

    let game_state = world
        .get_resource::<State<GameState>>()
        .map(|s| *s.get())
        .map(state_name);
    let pause_state = world
        .get_resource::<State<crate::pause::PauseState>>()
        .map(|s| *s.get())
        .map(pause_state_name);
    // 具名条件即普通系统：`run_system_once` 返回 `Result<bool, RunSystemError>`。
    let game_is_running = match world.run_system_once(crate::pause::game_is_running) {
        Ok(value) => Some(value),
        Err(_) => None,
    };
    let ticks = world.get_resource::<crate::pause::SetTickLog>();
    Ok(json!({
        "game_state": game_state,
        "pause_state": pause_state,
        "game_is_running": game_is_running,
        "ticks": {
            "simulation": ticks.map(|t| t.simulation),
            "interaction": ticks.map(|t| t.interaction),
            "tower_dock": ticks.map(|t| t.tower_dock),
        },
    }))
}

// ---------------------------------------------------------------------------
// P23：td.level_select（菜单按钮网格 / 焦点 / 交互态，教程 23）
// ---------------------------------------------------------------------------

/// `td.level_select` 的参数解析结果。
enum LevelSelectOp {
    /// 只读快照（按钮结构面 + 焦点 + 交互态 + 背景色）。
    Snapshot,
    /// 写焦点下标（键盘导航系统共享的同一状态）。
    Focus(usize),
    /// 走 Enter/Space 分支共用的选中逻辑。
    Confirm,
    /// 写 `Interaction::Pressed` 并运行真实的点击消费者一次。
    Click(usize),
}

/// 解析 `td.level_select` 的 params（严格分支）：缺省 / `{"op":"snapshot"}` = 快照；
/// `{"op":"focus","index":n}` / `{"op":"click","index":n}` / `{"op":"confirm"}`。
fn parse_level_select_op(params: Option<&Value>) -> Result<LevelSelectOp, BrpError> {
    let Some(value) = params else {
        return Ok(LevelSelectOp::Snapshot);
    };
    if value.is_null() {
        return Ok(LevelSelectOp::Snapshot);
    }
    let Value::Object(map) = value else {
        return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
    };
    let op = match map.get("op") {
        None | Some(Value::Null) => "snapshot",
        Some(Value::String(s)) => s.as_str(),
        Some(other) => {
            return Err(invalid_params(&format!("op 须为字符串（收到 {other}）")));
        }
    };
    let index_of = |map: &serde_json::Map<String, Value>| -> Result<usize, BrpError> {
        match map.get("index") {
            Some(Value::Number(n)) => match n.as_u64() {
                Some(i) => Ok(i as usize),
                None => Err(invalid_params("index 须为非负整数")),
            },
            _ => Err(invalid_params("该 op 需要 index（非负整数）")),
        }
    };
    match op {
        "snapshot" => Ok(LevelSelectOp::Snapshot),
        "confirm" => Ok(LevelSelectOp::Confirm),
        "focus" => Ok(LevelSelectOp::Focus(index_of(map)?)),
        "click" => Ok(LevelSelectOp::Click(index_of(map)?)),
        other => Err(invalid_params(&format!(
            "未知 op `{other}`（允许 snapshot / focus / click / confirm）"
        ))),
    }
}

/// `Interaction` 三态 → 稳定字符串（断言脚本口径）。
#[expect(
    deprecated,
    reason = "Interaction 为 0.20 弃用别名；快照口径沿 P17 A' 通道以 expect 属性落地"
)]
fn interaction_name(interaction: &bevy::ui::Interaction) -> &'static str {
    match interaction {
        bevy::ui::Interaction::Pressed => "Pressed",
        bevy::ui::Interaction::Hovered => "Hovered",
        bevy::ui::Interaction::None => "None",
    }
}

/// `td.level_select` handler（P23）：菜单按钮网格的可观察面与三通道确定性入口。
///
/// 守卫：`focus` / `click` / `confirm` 只在 `GameState::LevelSelect` 受理（菜单语义；
/// 与 `navigate_level_select` 的 `.run_if(in_state(LevelSelect))` 对齐），否则 -32602。
/// `click` 通过 `run_system_once` 运行**真实消费者系统**——`Changed<Interaction>` 在
/// 新系统实例上恒真（`bevy_ecs-0.20.0/src/system/function_system.rs:781`），
/// 故「写 Pressed → 消费者响应」闭环可被确定性断言。
#[expect(
    deprecated,
    reason = "Interaction 为 0.20 弃用别名；教程 23 原步消费者按 A' 通道以 expect 属性落地"
)]
pub fn level_select_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let op = parse_level_select_op(params.as_ref())?;
    let game_state = world
        .get_resource::<State<GameState>>()
        .map(|s| *s.get());

    if !matches!(op, LevelSelectOp::Snapshot) && game_state != Some(GameState::LevelSelect) {
        return Err(invalid_params(
            "菜单操作仅在 LevelSelect 受理（教程 23 屏幕语义）",
        ));
    }

    // 三个写通道的共同前置：发现列表 / 选中项 / 焦点资源。
    let len = world
        .get_resource::<crate::level_select::AvailableLevels>()
        .map(|l| l.0.len())
        .unwrap_or(0);

    match op {
        LevelSelectOp::Snapshot => {}
        LevelSelectOp::Focus(index) => {
            if index >= len {
                return Err(invalid_params(&format!("focus index {index} 越界（{len} 关）")));
            }
            world.resource_mut::<crate::level_select::FocusedLevel>().0 = index;
        }
        LevelSelectOp::Confirm => {
            let focused = world.resource::<crate::level_select::FocusedLevel>().0;
            // 多资源同写：`resource_scope` 临时移除 R 并交给闭包 `Mut<R>`，其余 World
            // 仍可用（`bevy_ecs-0.20.0/src/world/mod.rs:2876`）。
            let levels_snapshot = crate::level_select::AvailableLevels(
                world
                    .resource::<crate::level_select::AvailableLevels>()
                    .0
                    .clone(),
            );
            let chosen = world.resource_scope::<crate::level_select::SelectedLevel, _>(
                |world, mut selected| {
                    world.resource_scope::<NextState<GameState>, _>(|_, mut next_state| {
                        crate::level_select::apply_level_selection(
                            focused,
                            &levels_snapshot,
                            &mut selected,
                            &mut next_state,
                        )
                    })
                },
            );
            info!("[TD] level_select confirm -> {chosen:?}");
        }
        LevelSelectOp::Click(index) => {
            if index >= len {
                return Err(invalid_params(&format!("click index {index} 越界（{len} 关）")));
            }
            // ① 输入源替代：目标按钮 Pressed、其余按钮清回 None（避免旧按下态干扰）。
            let mut found = false;
            {
                let mut query = world.query_filtered::<
                    (&crate::level_select::LevelButton, &mut bevy::ui::Interaction),
                    With<crate::level_select::LevelButton>,
                >();
                for (btn, mut interaction) in query.iter_mut(world) {
                    if btn.0 == index {
                        *interaction = bevy::ui::Interaction::Pressed;
                        found = true;
                    } else if *interaction != bevy::ui::Interaction::None {
                        *interaction = bevy::ui::Interaction::None;
                    }
                }
            }
            if !found {
                return Err(invalid_params(&format!(
                    "click index {index} 找不到对应按钮实体（UI 未生成？）"
                )));
            }
            // ② 运行真实消费者（选中 + 请求 InGame）。
            if world
                .run_system_once(crate::level_select::handle_level_button_click)
                .is_err()
            {
                return Err(invalid_params("点击消费者运行失败（系统参数校验未过）"));
            }
            info!("[TD] level_select click -> button {index}");
        }
    }

    // ---- 快照（三条写通道后同样返回，便于一次调用即读结果）----
    let focused_index = world
        .get_resource::<crate::level_select::FocusedLevel>()
        .map(|f| f.0);
    let selected = world
        .get_resource::<crate::level_select::SelectedLevel>()
        .map(|s| s.0.clone());
    let mut buttons: Vec<Value> = Vec::new();
    {
        let mut query = world.query_filtered::<
            (
                &crate::level_select::LevelButton,
                &bevy::ui::Interaction,
                &BackgroundColor,
            ),
            With<crate::level_select::LevelButton>,
        >();
        for (btn, interaction, bg) in query.iter(world) {
            let color = bg.0.to_srgba();
            buttons.push(json!({
                "index": btn.0,
                "interaction": interaction_name(interaction),
                "focused": Some(btn.0) == focused_index,
                "color": [color.red, color.green, color.blue, color.alpha],
            }));
        }
    }
    buttons.sort_by_key(|b| b["index"].as_u64().unwrap_or(u64::MAX));
    Ok(json!({
        "available": game_state == Some(GameState::LevelSelect),
        "game_state": game_state.map(state_name),
        "levels": world
            .get_resource::<crate::level_select::AvailableLevels>()
            .map(|l| l.0.clone())
            .unwrap_or_default(),
        "focused": focused_index,
        "selected": selected,
        "buttons": buttons,
    }))
}

// ---------------------------------------------------------------------------
// P24：td.input（逻辑动作注入 + 虚拟光标，教程 24）
// ---------------------------------------------------------------------------

/// `td.input` 的参数解析结果。
enum InputOp {
    /// 读虚拟光标（缺省）。
    Cursor,
    /// 写虚拟光标（格坐标，需关卡已加载）。
    SetCursor([u32; 2]),
    /// 写一条 `GameAction` 消息（物理输入的统一出口）。
    Emit(crate::input::GameAction),
}

/// 解析 `td.input` 的 params（严格分支）：缺省 / `{"op":"cursor"}` = 读光标；
/// `{"op":"set_cursor","tile":[x,y]}` = 写光标；`{"op":"emit","action":"Confirm"}`
/// （`SelectTower`/`SelectLevel` 需 `index`）= 注入一条逻辑动作。
fn parse_input_op(params: Option<&Value>) -> Result<InputOp, BrpError> {
    let Some(value) = params else {
        return Ok(InputOp::Cursor);
    };
    if value.is_null() {
        return Ok(InputOp::Cursor);
    }
    let Value::Object(map) = value else {
        return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
    };
    let op = match map.get("op") {
        None | Some(Value::Null) => "cursor",
        Some(Value::String(s)) => s.as_str(),
        Some(other) => {
            return Err(invalid_params(&format!("op 须为字符串（收到 {other}）")));
        }
    };
    match op {
        "cursor" => Ok(InputOp::Cursor),
        "set_cursor" => Ok(InputOp::SetCursor(parse_tile_param(Some(value))?)),
        "emit" => {
            let Some(Value::String(name)) = map.get("action") else {
                return Err(invalid_params("emit 需要 action 字符串"));
            };
            let index = match map.get("index") {
                None | Some(Value::Null) => None,
                Some(Value::Number(n)) => {
                    let Some(i) = n.as_u64() else {
                        return Err(invalid_params("index 须为非负整数"));
                    };
                    Some(i as usize)
                }
                Some(other) => {
                    return Err(invalid_params(&format!(
                        "index 须为非负整数（收到 {other}）"
                    )));
                }
            };
            match crate::input::action_from_spec(name, index) {
                Some(action) => Ok(InputOp::Emit(action)),
                None => Err(invalid_params(&format!(
                    "未知 action `{name}`（或缺少 index）"
                ))),
            }
        }
        other => Err(invalid_params(&format!(
            "未知 op `{other}`（允许 cursor / set_cursor / emit）"
        ))),
    }
}

/// `td.input` handler（P24）：逻辑动作注入 + 虚拟光标读写。
///
/// 为什么需要：BRP 无键鼠 / 手柄注入通道，而 P24/P25 的全部行为面都经
/// `GameAction` / `VirtualCursorPos` 汇合——注入逻辑动作即等价于「按下键盘 / 点击 /
/// 推手柄」，且消费者全走真码路径（`nudge_virtual_cursor` / `select_tower_by_key` /
/// `navigate_level_select` / `toggle_pause` / `place_tower_on_click`）。
pub fn input_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let op = parse_input_op(params.as_ref())?;

    // 响应体同时给「动作面」与「光标面」：一次调用即可确认注入结果。
    let mut emitted: Option<String> = None;
    match op {
        InputOp::Cursor => {}
        InputOp::SetCursor(tile) => {
            let Some(map) = world.get_resource::<MapLayout>() else {
                return Err(invalid_params("set_cursor 需要已加载关卡（MapLayout 缺失）"));
            };
            if tile[0] >= map.width || tile[1] >= map.height {
                return Err(invalid_params(&format!(
                    "tile {:?} 越界（地图 {}x{}）",
                    tile, map.width, map.height
                )));
            }
            let width = map.width;
            let height = map.height;
            if let Some(mut cursor) = world.get_resource_mut::<crate::tower::VirtualCursorPos>() {
                cursor.0 = Some(tile);
            } else {
                world.insert_resource(crate::tower::VirtualCursorPos(Some(tile)));
            }
            info!("[TD] input set_cursor -> {:?} (map {width}x{height})", tile);
        }
        InputOp::Emit(action) => {
            let name = crate::input::action_name(&action);
            if world.write_message(action).is_none() {
                return Err(invalid_params(
                    "GameAction 消息队列未注册（InputPlugin 未装配？）",
                ));
            }
            info!("[TD] input emit -> {name}");
            emitted = Some(name);
        }
    }

    let cursor = world
        .get_resource::<crate::tower::VirtualCursorPos>()
        .and_then(|c| c.0);
    let placeable = match (
        cursor,
        world.get_resource::<MapLayout>(),
        world.get_resource::<crate::tower::PlacedTowers>(),
    ) {
        (Some(tile), Some(map), Some(placed)) => {
            Some(crate::tower::tile_is_placeable(map, placed, tile).is_some())
        }
        _ => None,
    };
    Ok(json!({
        "cursor": cursor,
        "cursor_available": world.get_resource::<crate::tower::VirtualCursorPos>().is_some(),
        "placeable": placeable,
        "emitted": emitted,
    }))
}

// ---------------------------------------------------------------------------
// P25：td.gamepad（手柄连接 / 绑定表 / 摇杆判别的可观察面，教程 25）
// ---------------------------------------------------------------------------

/// `td.gamepad` 的参数解析结果。
enum GamepadOp {
    /// 连接状态 + 绑定表 + 摇杆节拍参数。
    Snapshot,
    /// 单键映射探测（调用 `input::map_gamepad_button` 同一函数）。
    ProbeButton(bevy::input::gamepad::GamepadButton),
    /// 摇杆向量探测（调用 `input::stick_to_action` 同一函数）。
    ProbeStick(Vec2),
}

/// 解析 `td.gamepad` 的 params（严格分支）：缺省 / `{"op":"snapshot"}`；
/// `{"op":"probe_button","button":"DPadUp"}`；
/// `{"op":"probe_stick","x":0.5,"y":0.0}`。
fn parse_gamepad_op(params: Option<&Value>) -> Result<GamepadOp, BrpError> {
    let Some(value) = params else {
        return Ok(GamepadOp::Snapshot);
    };
    if value.is_null() {
        return Ok(GamepadOp::Snapshot);
    }
    let Value::Object(map) = value else {
        return Err(invalid_params(&format!("params 须为对象（收到 {value}）")));
    };
    let op = match map.get("op") {
        None | Some(Value::Null) => "snapshot",
        Some(Value::String(s)) => s.as_str(),
        Some(other) => {
            return Err(invalid_params(&format!("op 须为字符串（收到 {other}）")));
        }
    };
    match op {
        "snapshot" => Ok(GamepadOp::Snapshot),
        "probe_button" => {
            let Some(Value::String(name)) = map.get("button") else {
                return Err(invalid_params("probe_button 需要 button 字符串"));
            };
            match crate::input::gamepad_button_from_name(name) {
                Some(button) => Ok(GamepadOp::ProbeButton(button)),
                None => Err(invalid_params(&format!(
                    "未知 button `{name}`（允许标准 19 键变体名）"
                ))),
            }
        }
        "probe_stick" => {
            let read = |key: &str| -> Result<f32, BrpError> {
                match map.get(key) {
                    Some(Value::Number(n)) => n
                        .as_f64()
                        .map(|v| v as f32)
                        .ok_or_else(|| invalid_params(&format!("{key} 须为有限数"))),
                    _ => Err(invalid_params(&format!("probe_stick 需要 {key} 数值"))),
                }
            };
            let x = read("x")?;
            let y = read("y")?;
            if !x.is_finite() || !y.is_finite() {
                return Err(invalid_params("x / y 须为有限数"));
            }
            Ok(GamepadOp::ProbeStick(Vec2::new(x, y)))
        }
        other => Err(invalid_params(&format!(
            "未知 op `{other}`（允许 snapshot / probe_button / probe_stick）"
        ))),
    }
}

/// `td.gamepad` handler（P25）：手柄输入的结构面与纯函数探测面。
///
/// 环境限制（本机无手柄）：**行为面**无法用实机断言——按立项「映射表 + 模拟路径
/// 断言替代」落地：
/// - `gamepads` = 已连接手柄实体数（`Query<&Gamepad>`；无硬件时为 0）；
/// - `gamepad_input_module` = bevy_input 的 `gamepad` 特性是否装配
///   （`Messages<GamepadEvent>` 资源存在 = `InputPlugin` 在 `#[cfg(feature = "gamepad")]`
///   下注册过该消息，`bevy_input-0.20.0/src/lib.rs:143-144`）；
/// - `bindings` = [`crate::input::GAMEPAD_BINDINGS`] 表（按钮 → 动作）；
/// - `probe_button` / `probe_stick` = 调用与读取器**同一纯函数**（`map_gamepad_button` /
///   `stick_to_action`）——映射与死区 / 主导轴判别可被硬件无关地断言；
/// - `stick` = 节拍参数（死区 0.3 / 重复 150ms，教程 25 Step 3 原文值）。
pub fn gamepad_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    use bevy::input::gamepad::Gamepad;

    let op = parse_gamepad_op(params.as_ref())?;

    match op {
        GamepadOp::ProbeButton(button) => Ok(json!({
            "button": crate::input::gamepad_button_name(button),
            "action": crate::input::map_gamepad_button(button)
                .map(|a| crate::input::action_name(&a)),
        })),
        GamepadOp::ProbeStick(stick) => Ok(json!({
            "stick": [stick.x, stick.y],
            "action": crate::input::stick_to_action(stick)
                .map(|a| crate::input::action_name(&a)),
        })),
        GamepadOp::Snapshot => {
            let gamepads = {
                let mut query = world.query_filtered::<Entity, With<Gamepad>>();
                query.iter(world).count()
            };
            // `gamepad` 特性装配的证据 = `GamepadEvent` 消息队列资源存在
            // （`add_message` 在 `#[cfg(feature = "gamepad")]` 块内，bevy_input-0.20.0/src/lib.rs:143）。
            let gamepad_input_module =
                world.contains_resource::<bevy::ecs::message::Messages<bevy::input::gamepad::GamepadEvent>>();
            let bindings: Vec<Value> = crate::input::GAMEPAD_BINDINGS
                .iter()
                .map(|(button, action)| {
                    json!({
                        "button": crate::input::gamepad_button_name(*button),
                        "action": crate::input::action_name(action),
                    })
                })
                .collect();
            Ok(json!({
                "available": true,
                "gamepads": gamepads,
                "gamepad_input_module": gamepad_input_module,
                "stick": {
                    "dead_zone": crate::input::STICK_DEAD_ZONE,
                    "repeat_secs": crate::input::STICK_REPEAT_SECS,
                },
                "bindings": bindings,
            }))
        }
    }
}

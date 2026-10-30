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

use bevy::prelude::*;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::{error_codes, BrpError, BrpResult, RemotePlugin};
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use serde::Serialize;
use serde_json::{json, Value};

/// `td.screenshot` 方法名（唯一自定义方法；前缀 `td.`，非 `game.`）。
pub const TD_SCREENSHOT_METHOD: &str = "td.screenshot";

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
            .register_type::<crate::enemy::Enemy>();
        app.init_resource::<ScreenshotLog>();
        app.add_plugins(
            RemotePlugin::default()
                .with_method_main(TD_SCREENSHOT_METHOD, screenshot_handler),
        );
        app.add_plugins(RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST));
        info!("[BRP] listening on 127.0.0.1:15702 (explicit loopback bind)");
        info!("[BRP] custom methods: {TD_SCREENSHOT_METHOD}");
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

/// 本模块族共用的参数错误（JSON-RPC INVALID_PARAMS -32602）。
fn invalid_params(message: &str) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message: message.to_string(),
        data: None,
    }
}

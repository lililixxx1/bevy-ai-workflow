//! [`game.screenshot`] / [`game.screenshot_log`]：主窗口帧捕获与落盘
//! （M2 验收②的截图环节——落地后全链不再依赖 OS 级窗口截屏）。
//!
//! 捕获是**异步**的：handler 只受理请求（spawn 携带 [`Screenshot`] 组件的
//! 实体并挂两个 observer），渲染子 app 在随后帧捕获，完成后以
//! `ScreenshotCaptured` 实体事件触发该实体上的 observer——官方 `save_to_disk`
//! 同步写 PNG，本模块的 [`on_captured`] 补记日志（尺寸/帧号/状态翻转），
//! 实体最后由 ScreenshotPlugin 的 `clear_screenshots`（First 调度）清理。
//! 因此方法契约是两段式：`game.screenshot` 受理 → 工具端轮询
//! `game.screenshot_log` 至 `status == "captured"` 后核对文件（PNG 魔数）。
//!
//! 路径口径：`params.path` 可省（默认 `screenshot-{id}.png`，进程 CWD 相对）。
//! 监听面安全（AGENTS.md 硬约束 4）：BRP 仅回环，本方法与 bevy 官方
//! `save_to_disk` 同信任域（本地进程可写任意路径），不另设路径白名单。
//!
//! 依据: `Screenshot(pub RenderTarget)` 组件与 spawn+observer 用法（含 doctest）
//! `bevy_render-0.19.1/src/view/window/screenshot.rs:68-101`；`ScreenshotCaptured`
//! `EntityEvent`（字段 entity / image）`:49-53`；`primary_window()` `:98`；
//! `save_to_disk(impl AsRef<Path>) -> impl FnMut(On<ScreenshotCaptured>)`
//! （内部 `to_rgb8().save_with_format` 同步落盘）`:134`；Captured 实体清理
//! `clear_screenshots` 定义 `:197`、First 调度注册 `:416-418`；官方示例
//! `bevy-0.19.1/examples/window/screenshot.rs:26-27`；`EntityWorldMut::observe`
//! `bevy_ecs-0.19.1/src/world/entity_access/world_mut.rs:1966`（核实 2026-09-26）。

use bevy::prelude::*;
use bevy::remote::{error_codes, BrpError, BrpResult};
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use serde::Serialize;
use serde_json::{json, Value};

/// 截图日志（`game.screenshot` 受理即建 pending 行，捕获完成由 observer 补齐）。
#[derive(Resource, Default)]
pub struct ScreenshotLog {
    /// 递增 id（默认文件名序号同源）。
    next_id: u32,
    /// 全部条目（含 pending 与 captured；不清理——查证用）。
    entries: Vec<ScreenshotEntry>,
}

/// 单条截图记录（`game.screenshot_log` 响应元素）。
#[derive(Serialize, Clone, Debug)]
pub struct ScreenshotEntry {
    pub id: u32,
    /// 落盘目标路径（原样记录：相对路径以进程 CWD 为准）。
    pub path: String,
    /// `pending`（已受理未完成）/ `captured`（ScreenshotCaptured 已触发）。
    /// 注意 `captured` 只承诺事件已到（含官方 save_to_disk 落盘动作已执行），
    /// 文件核验（存在性/PNG 魔数）由工具端做——最强证据在文件本身。
    pub status: &'static str,
    /// 受理时模拟帧号（SimStats.frame_count；缺资源为 null）。
    pub frame_requested: Option<u64>,
    /// 完成时模拟帧号。
    pub frame_captured: Option<u64>,
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
    fn request(&mut self, path: String, frame_requested: Option<u64>, entity: Entity) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries.push(ScreenshotEntry {
            id,
            path,
            status: "pending",
            frame_requested,
            frame_captured: None,
            width: None,
            height: None,
            entity,
        });
        id
    }
}

/// `game.screenshot` handler：受理一次主窗口捕获（异步，见模块文档）。
pub fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    // params 形态校验显式报错（对齐 run_tests 的严格分支，不静默回落默认路径）。
    let requested = match params.as_ref() {
        None | Some(Value::Null) => None,
        Some(Value::Object(map)) => match map.get("path") {
            None | Some(Value::Null) => None,
            Some(Value::String(s)) if s.is_empty() => {
                return Err(super::invalid_params("path 不能为空字符串"));
            }
            Some(Value::String(s)) => Some(s.clone()),
            Some(other) => {
                return Err(super::invalid_params(&format!(
                    "path 须为字符串（收到 {other}）"
                )));
            }
        },
        Some(other) => {
            return Err(super::invalid_params(&format!(
                "params 须为对象（收到 {other}）"
            )));
        }
    };

    // 默认文件名需要 id：先读（handler 单线程独占，读后 spawn 前无人改写），
    // spawn 后再经 request() 落账（id 同源）。
    let id_for_name = world.resource::<ScreenshotLog>().next_id;
    let path = requested.unwrap_or_else(|| format!("screenshot-{id_for_name}.png"));
    let frame_requested = world
        .get_resource::<crate::sim::SimStats>()
        .map(|s| s.frame_count);

    let entity = world
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(std::path::PathBuf::from(path.clone())))
        .observe(on_captured)
        .id();

    let id = world
        .resource_mut::<ScreenshotLog>()
        .request(path.clone(), frame_requested, entity);
    info!("[RPC] screenshot #{id} queued -> {path} (entity {entity})");

    Ok(json!({
        "queued": true,
        "id": id,
        "path": path,
        "note": "捕获异步完成：轮询 game.screenshot_log 至 status=captured，再核验 PNG 文件",
    }))
}

/// `game.screenshot_log` handler：全部截图条目 + pending 计数。
pub fn log_handler(In(_params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let log = world.resource::<ScreenshotLog>();
    Ok(json!({
        "next_id": log.next_id,
        "pending": log.entries.iter().filter(|e| e.status == "pending").count(),
        "entries": log.entries,
    }))
}

/// observer：捕获完成 → 回填日志（尺寸 / 完成帧号 / 状态翻转）。
/// 与官方 `save_to_disk` 并挂同一实体（两个 observer 的相对次序无契约，
/// 故本条目只记 `captured`，不承诺文件已写——文件核验归工具端）。
fn on_captured(
    event: On<ScreenshotCaptured>,
    mut log: ResMut<ScreenshotLog>,
    stats: Option<Res<crate::sim::SimStats>>,
) {
    let entity = event.entity;
    let size = event.image.texture_descriptor.size;
    let frame_captured = stats.map(|s| s.frame_count);
    if let Some(entry) = log.entries.iter_mut().rev().find(|e| e.entity == entity) {
        entry.status = "captured";
        entry.frame_captured = frame_captured;
        entry.width = Some(size.width);
        entry.height = Some(size.height);
        info!(
            "[RPC] screenshot #{} captured ({}x{}, frame {:?})",
            entry.id, size.width, size.height, entry.frame_captured
        );
    } else {
        warn!("[RPC] ScreenshotCaptured 未匹配到日志条目（entity {entity}）");
    }
}

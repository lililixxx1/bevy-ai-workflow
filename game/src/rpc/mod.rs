//! 游戏专属 BRP 自定义方法（自研件核心，意向文档 §5.3 / §10 第 3 条）。
//!
//! 方法集（全部经 [`RemotePlugin::with_method_main`] 注册进主 app，[`brp`] 模块组装）：
//! - [`RUN_TESTS_METHOD`]（[`run_tests`]）：按任务测试集验收清单在游戏进程内
//!   逐条断言，响应附世界快照供工具端做跨调用断言（M2 验收③）；
//! - [`SCREENSHOT_METHOD`] / [`SCREENSHOT_LOG_METHOD`]（[`screenshot`]）：主窗口
//!   帧异步捕获 + 落盘 + 完成状态查询（M2 验收②的截图环节）；
//! - [`LAUNCH_LEVEL_METHOD`]（[`launch_level`]）：关卡加载——清场 + 按关卡定义
//!   生成单位 + 写关卡状态资源（M4 A 案系统席①，T032 落地——意向文档 §5.3
//!   规划三方法自此齐全）。
//!
//! handler 形态（SKILL.md §6.2 首次仓库内落地）：`fn(In(params): In<Option<Value>>,
//! world: &mut World) -> BrpResult`，由 bevy_remote 经 `world.run_system_with`
//! 独占执行——handler 内可安全直改世界（无调度上下文，SKILL.md §3.3 认可的
//! `&mut World` 场景）。
//! 依据: `with_method_main(name: impl Into<String>, handler: impl IntoSystem<
//! In<Option<Value>>, BrpResult, M>)` `bevy_remote-0.19.1/src/lib.rs:591-599`；
//! 独占执行 `world.run_system_with(id, message.params)` `lib.rs:1501`；内置
//! handler 同形态（`&mut World`）`builtin_methods.rs:1058-1061`；`BrpError` 公开
//! 字段 `lib.rs:1304-1312`；`error_codes::INVALID_PARAMS` `lib.rs:1387` 起
//! （核实 2026-09-26）。
//!
//! [`brp`]: crate::brp

pub mod launch_level;
pub mod run_tests;
pub mod screenshot;
pub mod suites;

use bevy::prelude::*;
use bevy::remote::{error_codes, BrpError};

/// 本模块族共用的参数错误（JSON-RPC INVALID_PARAMS -32602）。
pub(crate) fn invalid_params(message: &str) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message: message.to_string(),
        data: None,
    }
}

/// `game.run_tests` 方法名（内置方法用 `world.`/`registry.` 前缀，游戏专属用 `game.`）。
pub const RUN_TESTS_METHOD: &str = "game.run_tests";
/// `game.screenshot` 方法名。
pub const SCREENSHOT_METHOD: &str = "game.screenshot";
/// `game.screenshot_log` 查询方法名（捕获是异步的，落盘结果经此轮询）。
pub const SCREENSHOT_LOG_METHOD: &str = "game.screenshot_log";
/// `game.launch_level` 方法名（关卡加载，M4 A 案系统席①）。
pub const LAUNCH_LEVEL_METHOD: &str = "game.launch_level";

/// 自定义方法的资源底座（截图日志；`run_tests` 无状态）。
pub struct GameRpcPlugin;

impl Plugin for GameRpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<screenshot::ScreenshotLog>();
    }
}

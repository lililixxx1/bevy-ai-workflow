//! Rust Hotpatching Windows 独立冒烟（意向文档 §10 第 6 条 / §5.3）。
//!
//! 运行方式（对齐 bevy 0.19.1 官方 `examples/ecs/hotpatching_systems.rs` 的说明，
//! 已核实 0.19.1 仍为 `bevy/hotpatching` feature + dx CLI）：
//! ```sh
//! dx serve --hot-patch
//! ```
//!
//! 冒烟判定：修改 [`tick_system`] 中的 `TAG`/步长后保存，dx 触发热补丁，
//! **进程不重启**的前提下输出行为发生变化。frame 计数连续（不归零）是
//! 「未重启进程」的直接证据——`Local` 状态存活于 ECS，热补只替换函数体。
//!
//! 插件选择：`MinimalPlugins` + 显式 [`HotPatchPlugin`]（无窗口/无 wgpu 依赖，
//! 冒烟只需逻辑热替换；HotPatchPlugin 负责 `connect_subsecond()` 连接 dx CLI）。

use bevy::app::hotpatch::HotPatchPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default(), HotPatchPlugin))
        .add_systems(Update, tick_system)
        .run();
}

/// 每帧计数，每 30 帧打印一行带 tag 的输出。
fn tick_system(mut frame: Local<u32>) {
    *frame += 1;
    // 以下是热补目标行：dx serve --hot-patch 保存后应立即生效，无需重启。
    const TAG: &str = "AFTER"; // 热补丁变更行（原值 BEFORE）
    if *frame % 10 == 0 { // 热补丁变更行（原值 30）：输出频率应变为 3 倍
        info!("[hotpatch-smoke] frame={} tag={TAG}", *frame);
    }
}

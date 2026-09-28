//! BRP 常驻插件（工作流三的通路底座，意向文档 §5.3）+ 游戏专属方法组装点。
//!
//! 监听口径（写死，不暴露 CLI）：`127.0.0.1:15702`。
//! 依据: RemotePlugin/RemoteHttpPlugin 挂载形态
//! `bevy-0.19.1/examples/remote/server.rs:18-19`；`with_address(impl Into<IpAddr>)`
//! 签名 `bevy_remote-0.19.1/src/http.rs:167-170`；默认地址常量 `DEFAULT_ADDR`
//! = 127.0.0.1、默认端口 `DEFAULT_PORT` = 15702（`src/http.rs:52-60`）。
//! 安全约束（AGENTS.md 硬约束 4）：仅回环地址，禁止 0.0.0.0 / 局域网——
//! 即使默认已是回环，这里仍显式绑定，防默认值变化。
//!
//! 自定义方法（自研件，意向文档 §10 第 3 条）在本插件的 RemotePlugin 上
//! 组装（SKILL.md §3.2「集中管理」）：`game.run_tests` / `game.screenshot` /
//! `game.screenshot_log` / `game.launch_level`（M4 A 案系统席①，T032——
//! 意向文档 §5.3 规划三方法自此齐全），handler 与资源见 [`crate::rpc`]。

use bevy::prelude::*;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::RemotePlugin;

use crate::rpc;

pub struct BrpPlugin;

impl Plugin for BrpPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(rpc::GameRpcPlugin);
        app.add_plugins(
            RemotePlugin::default()
                .with_method_main(rpc::RUN_TESTS_METHOD, rpc::run_tests::handler)
                .with_method_main(rpc::SCREENSHOT_METHOD, rpc::screenshot::handler)
                .with_method_main(rpc::SCREENSHOT_LOG_METHOD, rpc::screenshot::log_handler)
                .with_method_main(rpc::LAUNCH_LEVEL_METHOD, rpc::launch_level::handler),
        );
        app.add_plugins(
            RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST),
        );
        info!("[BRP] listening on 127.0.0.1:15702 (explicit loopback bind)");
        info!(
            "[BRP] custom methods: {} / {} / {} / {}",
            rpc::RUN_TESTS_METHOD,
            rpc::SCREENSHOT_METHOD,
            rpc::SCREENSHOT_LOG_METHOD,
            rpc::LAUNCH_LEVEL_METHOD
        );
    }
}

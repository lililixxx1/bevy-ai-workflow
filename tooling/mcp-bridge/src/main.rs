//! mcp-bridge 可执行入口：stdio MCP 循环（窗口前置增强 T041 / A1）。
//!
//! 启动时只做一件事——校验 BRP 目标地址（默认 `127.0.0.1:15702`，环境变量
//! `MCP_BRIDGE_BRP_ADDR` 可覆盖，主机限 `127.0.0.1` / `localhost` 字面）；
//! 非法地址打印 stderr 并以退出码 2 结束。随后直接进入 stdio 循环，**不探测
//! BRP**——游戏未启动时桥进程存活，逐调用返回 bridge_error（会话自连形态的
//! 健壮性要求）。
//!
//! stdout 只写合法 MCP 消息；诊断信息一律走 stderr（MCP stdio 传输约束）。

use std::io::{self, BufReader};
use std::process::ExitCode;

use mcp_bridge::{BrpAddr, HttpBrp, Session};

fn main() -> ExitCode {
    let addr = match BrpAddr::from_env() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("mcp-bridge 启动失败：{e}");
            return ExitCode::from(2);
        }
    };
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut session = Session::new(HttpBrp::new(addr));
    match session.run(BufReader::new(stdin.lock()), &mut stdout.lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mcp-bridge stdio 循环失败：{e}");
            ExitCode::from(1)
        }
    }
}

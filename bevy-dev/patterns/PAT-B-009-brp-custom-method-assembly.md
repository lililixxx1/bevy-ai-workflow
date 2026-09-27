# PAT-B-009-brp-custom-method-assembly

### PAT-B-009：BRP 自定义方法组装——`with_method_main` 链 + 错误码复用的受理侧纪律

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（bevy_remote 自定义方法组装面）
- 标签：`BRP` `with_method_main` `BrpError` `INVALID_PARAMS` `自定义方法` `命名前缀`
- 源码出处：`game/src/brp.rs:26-34`（组装）+ `game/src/rpc/mod.rs:29-43`（错误码复用 + 方法名常量）

**场景**：给常驻游戏加游戏专属 RPC 方法（验收断言 / 截图受理等），要求：注册面集中一处、方法名可检索、参数校验错误与 BRP 协议错误体系一致（JSON-RPC INVALID_PARAMS）。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/brp.rs` 与 `game/src/rpc/mod.rs` 的逐字删减——去缩进；块 2 末行尾注由 :38 doc 注释改写，系本条目添加）：

```rust
// 抄自 game/src/brp.rs:26-34（插件组装点集中一处）：
app.add_plugins(
    RemotePlugin::default()
        .with_method_main(rpc::RUN_TESTS_METHOD, rpc::run_tests::handler)
        .with_method_main(rpc::SCREENSHOT_METHOD, rpc::screenshot::handler)
        .with_method_main(rpc::SCREENSHOT_LOG_METHOD, rpc::screenshot::log_handler),
);
app.add_plugins(
    RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST),
);

// 抄自 game/src/rpc/mod.rs:30-36 + :39-43（错误码复用 + 命名前缀纪律）：
pub(crate) fn invalid_params(message: &str) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message: message.to_string(),
        data: None,
    }
}
pub const RUN_TESTS_METHOD: &str = "game.run_tests"; // 内置 world./registry. 前缀，游戏专属用 game.
```

要点：①handler 统一形态 `fn(In(params): In<Option<Value>>, world: &mut World) -> BrpResult`——bevy_remote 经 `world.run_system_with` 独占执行，handler 内可安全直改世界（`with_method_main` 签名 `bevy_remote-0.19.1/src/lib.rs:591-599`，核实 2026-09-26）；②监听面与组装面分离：`RemoteHttpPlugin::default().with_address(Ipv4Addr::LOCALHOST)` 显式回环绑定（即使默认同值，防默认漂移——AGENTS 硬约束 4）；③参数错误统一走模块级 `invalid_params()`（复用 `error_codes::INVALID_PARAMS` = -32602，不手写魔数）；④方法名 `&'static str` 常量集中定义 + `game.` 前缀区隔内置方法。

**为什么**：注册集中使方法清单一眼可审（与 `[BRP] custom methods` 启动日志互证）；错误码复用让工具端按 JSON-RPC 标准分支处理；常量名避免字符串散落拼写漂移。

**验证证据**：
- 代码为 M2（T019）已验证任务代码的逐字删减，**删减后未单独重新编译**；
- 运行验证：三方法 BRP 实测全通（`game.run_tests` 5 断言 ×6 调用、`game.screenshot` PNG 魔数+IHDR 断言、`game.screenshot_log` 轮询——`docs/evidence/m2-rpc.md` + `m2-rpc/`，2026-09-26）；`rpc.discover` 收录三方法。

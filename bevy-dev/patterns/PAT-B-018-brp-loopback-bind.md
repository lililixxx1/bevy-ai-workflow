# PAT-B-018-brp-loopback-bind

### PAT-B-018：BRP 安全绑定——`RemoteHttpPlugin::default().with_address(Ipv4Addr::LOCALHOST)` 显式回环

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（bevy_remote 监听面配置）
- 标签：`RemotePlugin` `RemoteHttpPlugin` `with_address` `127.0.0.1` `回环` `安全约束`
- 源码出处：`game/src/brp.rs:23-41`（插件组装 + 显式回环绑定 + 方法注册）

**场景**：游戏常驻开放 BRP（BRP 无鉴权）——必须把监听面钉死在回环地址，且不因上游默认值变化而漂移。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/brp.rs:23-34` 的逐字删减——保留安全注释原文）：

```rust
impl Plugin for BrpPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(rpc::GameRpcPlugin);
        app.add_plugins(
            RemotePlugin::default()
                .with_method_main(rpc::RUN_TESTS_METHOD, rpc::run_tests::handler)
                .with_method_main(rpc::SCREENSHOT_METHOD, rpc::screenshot::handler)
                .with_method_main(rpc::SCREENSHOT_LOG_METHOD, rpc::screenshot::log_handler),
        );
        app.add_plugins(
            RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST),
        );
    }
}
```

要点：①即使 bevy_remote 默认已是 127.0.0.1:15702，仍**显式** `with_address(Ipv4Addr::LOCALHOST)`——把安全约束写进代码而非依赖默认值（默认漂移即静默暴露）；②监听口径写死、不暴露 CLI（监听面不做成可配项，防误配 0.0.0.0）；③自定义方法集中在一个 `RemotePlugin::default()` 上链式注册（`game.` 前缀命名空间与内置 `world.`/`registry.` 区分）。

**为什么**：BRP 协议无鉴权——任何能连上端口的一方即可读写世界；唯一防线是监听面。显式绑定 + 不可配置化使「安全约束」成为代码评审可见的一行。

**验证证据**：
- 代码为 T003/T019 已验证任务代码的逐字删减，删减后未单独重新编译；
- 运行验证：全部 BRP 证据（m1-phase2 12 任务、m2-rpc、ts-*）均经 127.0.0.1:15702；监听回环侧证见 docs/evidence/m2-rpc/（进程/门禁日志）。

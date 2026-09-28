# PAT-B-017-in-process-test-suites

### PAT-B-017：进程内测试套件——`SuiteFn = fn(&mut World) -> Vec<TestCase>` 注册表 + 无净副作用约定

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（bevy_ecs World 直测 + BRP handler 组合；注册表形态引擎无关）
- 标签：`测试套件` `&mut World` `注册表` `无净副作用` `run_tests` `自恢复`
- 源码出处：`game/src/rpc/suites/mod.rs:46-67`（类型 + all/lookup/names 注册表）+ `:8`（无净副作用约定）

**场景**：把「任务测试集」做成游戏进程内的可枚举套件：BRP 方法 `game.run_tests` 按名执行，套件以 `&mut World` 直测判定语义，返回结构化断言供工具端裁决——避免每条断言一次 BRP 往返。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/rpc/suites/mod.rs:46-67` 的逐字删减）：

```rust
/// 套件函数形态（约定无净副作用：只读或写后自恢复，见模块文档）。
pub type SuiteFn = fn(&mut World) -> Vec<TestCase>;

/// 全部套件 (名称, 入口)。
pub fn all() -> Vec<(&'static str, SuiteFn)> {
    vec![
        ("ts-01", ts01::run),
        ("ts-02", ts02::run),
        // ……
        ("ts-12", ts12::run),
    ]
}

/// 按名查找套件。
pub fn lookup(name: &str) -> Option<(&'static str, SuiteFn)> {
    all().into_iter().find(|(n, _)| *n == *name)
}

/// 可用套件名清单（错误 message 用）。
pub fn names() -> Vec<String> {
    all().into_iter().map(|(n, _)| n.to_string()).collect()
}
```

要点：①`fn(&mut World) -> Vec<TestCase>` 纯函数指针——注册表可静态枚举、BRP 错误信息可列出全部合法名；②**无净副作用约定**：写通路型套件以 ECS 直写复测判定语义，断言完自恢复（计数/资源值/初值还原）——套件可重复执行、不污染后续任务；③`TestCase { name, pass, detail }`——`pass=false` 不是协议错误（HTTP 层仍 200），汇总裁决归工具端（退出码绑定，task-runner --expect-* 体系）。

**为什么**：`&mut World` 直测绕过序列化往返（快且能测写通路语义）；无净副作用使 12 套件可任意次序/任意次组合执行——回归集的前提。

**验证证据**：
- 代码为 T019/T020 已验证任务代码的逐字删减，删减后未单独重新编译；
- 运行验证：M2 验收③ TS-01 两阶段 5/5×6 调用退出码绑定（docs/evidence/m2-rpc.md）；M1 phase2 十套件重跑全 PASS 且每任务收尾残留核对 clean（docs/evidence/m1-phase2/summary.txt）。

# 任务台账

- 依据：意向文档 §4（M3 ③「任务台账连续记录」）、§10 第 2 条（骨架就绪后立即启用，从第 1 个任务起逐条记录）。
- 角色：M3 增值证明（返工率对比）与升级窗口工时量化的**唯一数据源**，任何任务开始前不可缺位。
- 纪律：任务收尾前完成登记；耗时以分钟近似（挂钟时间，含等待编译）；返工以「首次判定未过后的重做次数」计。

## Schema

| 字段 | 说明 |
|---|---|
| 编号 | 自增，`T001` 起 |
| 任务 | 任务名；标准化任务标注 taskset 条目号 |
| 类型 | `骨架` / `文档` / `代码` / `验证` / `工具` / `升级迁移` / `资产回写` / `其他` |
| 一次通过 | `是` / `否`。代码类任务口径（M1 定义，§4）：首次提交即过 `cargo check` 且 demo 首次运行满足验收清单，期间无人工改代码 |
| 返工次数 | 首次判定后的重做次数（0 = 一次通过） |
| 耗时 | 分钟（近似） |
| 原因 | 返工/失败原因；一次通过者记录关键偏差或环境问题 |
| 证据 | commit hash / 命令与结果摘录 / 日志或截图路径 |

## 记录

| 编号 | 任务 | 类型 | 一次通过 | 返工次数 | 耗时 | 原因 | 证据 |
|---|---|---|---|---|---|---|---|
| T001 | 仓库骨架搭建与台账启用（意向文档 §10 第 1、2 条 + §6） | 骨架 | 是 | 0 | ≈20 min | 通过=无/无审核返工（审核已通过）。过程记一次环境级偏差（不计返工，代码零返工）：首次 `cargo check --workspace` 因环境 rustc 1.94.0 低于 bevy 0.19.1 MSRV 1.95.0（`bevy-0.19.1/Cargo.toml:14`）在解析阶段被拒，修复 = `rustup update stable` → 1.98.1，重跑通过；教训入库 PIT-M-001（含管道吞退出码风险）。另：曾拟用 `bevy::VERSION`，查本地源码发现 0.19.1 无此常量（容器 crate 仅 `pub use bevy_internal::*`），未造成编译失败，main.rs 注释留痕 | 提交 d8ecd72 / cd7fd32 / 3fb1098 / db813ed + 本收尾提交；`cargo check --workspace` REAL_EXIT=0（1m44s）；`cargo test --doc -p docs` REAL_EXIT=0（2 passed）；`cargo run -p game` REAL_EXIT=0 |
| T002 | bevy-dev/SKILL.md v0 撰写 + patterns/pitfalls 条目骨架（意向文档 §10 第 3 条 + §5.2） | 文档 | 是 | 0 | ≈50 min | 通过=按任务书五项覆盖成文，无返工。关键查证记录：①bevy_remote 不在本地 registry（骨架未依赖它），按「仓库外临时 crate cargo fetch」拉取 0.19.1 源码核实（fetch REAL_EXIT=0，未动本仓库 manifest，且使 bevy_remote 源码常驻本地 registry 供后续任务复用）；②修正种子信息一处——`RemotePlugin::with_method` 在 0.19.1 为私有 fn（lib.rs:611），公开 API 是 `with_method_main`/`with_method_render`（lib.rs:591/601），已写入 SKILL.md §6.2 并标注「种子信息也错，源码为准」；③BRP 默认监听实测为 127.0.0.1:15702（主）/15703（render）（http.rs:52-60）；④发现 crate 文档请求示例漏 `jsonrpc` 字段、与反序列化器（lib.rs:1145-1146）不符，记入 §6.4。未做（无代码变更，不适用 cargo check 门禁）：SKILL.md 中代码片段为官方示例同形态，均标注「未在本仓库过编译，首次写入 game/ 时须过 check」 | 提交 97973fb + 本收尾提交；源码引用清单见 SKILL.md §6（bevy-0.19.1 与 bevy_remote-0.19.1 行号引用共 20+ 处） |

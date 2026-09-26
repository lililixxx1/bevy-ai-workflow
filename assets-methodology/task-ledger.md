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
| T002 | bevy-dev/SKILL.md v0 撰写 + patterns/pitfalls 条目骨架（意向文档 §10 第 3 条 + §5.2） | 文档 | 否 | 1 | ≈70 min | 通过=无/审核返工（v0.1 修复后审核已通过）。首版按任务书五项覆盖成文，审核（第 1 轮）判三处事实失准，返工 1 次修复（未降约束、未删证据）：①§6.2 内置方法清单漏 `schedule.list`/`schedule.graph`（共 23 个常量，初版 grep 按 `_METHOD` 后缀过滤漏检了无后缀的 `BRP_SCHEDULE_LIST`/`BRP_SCHEDULE_GRAPH`，builtin_methods.rs:102/108；注册 lib.rs:779/784）；②§6.2 `with_watching_method_render` 行号 :641→:642（641 是 `#[inline]` 属性行）；③§0.3/§6.6 `rust-version` 行号 :13→:14（与 T001 一致；初版从 sed 输出数行时偏移一行）。关键查证记录：bevy_remote 不在本地 registry，按「仓库外临时 crate cargo fetch」拉取 0.19.1 源码核实（fetch REAL_EXIT=0，未动本仓库 manifest，且使 bevy_remote 源码常驻本地 registry）；修正种子信息一处——`RemotePlugin::with_method` 在 0.19.1 为私有 fn（lib.rs:611），公开 API 是 `with_method_main`/`with_method_render`（lib.rs:591/601）；BRP 默认监听实测 127.0.0.1:15702/15703（http.rs:52-60）；crate 文档请求示例漏 `jsonrpc` 字段、与反序列化器（lib.rs:1145-1146）不符。教训（方法论级，防再犯机制已内嵌 SKILL.md §6.2 检索提示）：行号引用应逐条 `grep -n` 直读而非从区间输出目测推数；枚举「全集」时不得按命名后缀过滤 | 提交 97973fb（首版）+ bf9bae4（首版收尾）+ 67233e0（返工）+ 本收尾提交；源码引用清单见 SKILL.md §6 |
| T003 | M1 第一役：相机漫游+N 动态实体 demo + BRP 常驻 + 帧率基线 + 首批任务测试集（意向文档 §10 第 5 条） | 代码 | 否 | 1 | ≈110 min | 通过=终版全部门禁过/无审核返工（审核已通过，返工 0）。返工 1 次系代码执行期返工（非审核返工，如实保留：M1「一次通过率」口径的数据源）。首次 check 失败（返工 1 次，8 个编译错误三类：①spawn_batch 要求迭代器 'static，闭包借用系统参数 E0373/E0521→move 闭包+值拷出；②PresentMode/WindowMode 不在 prelude E0433→bevy::window:: 显式导入；③f32 无 Eq 的低级失误）；修复后 check/test/doctest/release 全绿。demo 首次运行即满足 BRP 验收（11/11 断言 PASS，含解析式运动数学断言）。查证收获：0.19 事件 API 为 MessageWriter<AppExit>（非 EventWriter）；BRP 实测三形态（Vec3=[x,y,z] 数组、Camera 在 bevy_camera crate、远程 mutate 帧边界竞态）——taskset 初版预写形态被实测推翻后修正。基线双组：vsync 组全 60.0（锁帧无区分度）→ 增补 --no-vsync 参数与第二组数据（534.7→113.8 阶梯清晰），文档同步义务履行（docs/fps-baseline.md + brp-smoke.md 入 doctest 封装）。坑入库：PIT-B-001/002/003 + PIT-M-002（SplitMix64 参考向量凭记忆写错，先独立实现验证后发现，补做失败复现过门禁）；模式入库 PAT-B-001（BRP 常驻最小集成） | 提交见本收尾系列；BRP 断言 docs/evidence/brp-assert-result.txt（11/11 PASS）；基线 docs/fps-baseline.md + docs/evidence/bench-n*.log ×10；taskset TS-01~12 |
| T004 | BRP 通路验证与 bevy_brp_mcp 兼容性实测（意向文档 §10 第 4 条 + §5.3） | 验证 | 否 | 1 | ≈60 min | 通过=全链跑通有存证（9 个内置 BRP 方法 + bevy_brp_mcp 0.22.7 三轮会话 19 项工具调用，每个含请求/响应原文；游戏进程收尾确认杀净）。返工 1 次为请求/探针参数形态错误：①curl 首写 `params:{}`（list_components）与复数 `resources`（get_resources）报 -32602——实为 0.19.1 schema 收紧的真实发现，转 PIT-B-004 入库；②MCP 探针 v1 把 entity id 传成字符串被服务端 u64 校验拦截，v2 修正后全链成功。关键结论：BRP 默认 127.0.0.1:15702（http.rs:52,60）；rpc.discover 23 方法；BRP 依赖反射注册属实（register_type 四类型+camera 一类型全可查可改）；bevy_brp_mcp 0.22.7（依赖 bevy ^0.19.1）与 0.19.1 游戏协议级兼容，extras 系 14 工具需游戏加装 bevy_brp_extras（环境性预期失败，报错原文存证）；上游 GitHub 仓库已归档迁移 bevy_brp workspace，crates.io 渠道正常 | `tooling/brp-logs/`（01–09 方法存证、10-* 三轮 MCP transcript、11-mcp-tools-report.md、验证报告.md）；PIT-B-004 |

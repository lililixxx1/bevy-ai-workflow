# bevy-ai-workflow

**一套「AI 长期驻场」的 Bevy 游戏开发工作流**——主产出是工作流本身与两层知识资产；游戏只是试金石（逼出真实约束、喂真实教训），不是产品野心。

## 这是什么

用三条工作流把 AI 开发闭环钉在编译器与运行时事实上：

1. **AI 写文档**（知识生产）——`docs/` 的 Markdown 经 `include_str!` 薄封装纳入 rustdoc，`cargo test --doc` 门禁全绿才可合并：编译器当裁判，过滤 AI 幻觉。
2. **AI 写代码**（受 skill 约束）——锁版本（bevy 0.19）+ 禁凭记忆写 API（不确定必查源码/官方文档）+ 每次变更过 `cargo check` + 错题本/模式库只在验证后入库（Voyager 原则：未过验证禁止入库）。
3. **AI 操控运行中的游戏**（验证闭环）——官方 `bevy_remote`（BRP，JSON-RPC over HTTP，仅回环）+ 自研游戏专属 RPC（`run_tests` / `screenshot` / `launch_level` / 战斗指令等 7 个 `game.*` 方法）+ 进程内断言套件，状态断言优先、截图辅助。

两层知识资产：`assets-methodology/`（引擎无关、可带走的方法论层）+ `bevy-dev/`（版本绑定的 Bevy 特定层，随升级窗口整体迁移）。

## 仓库结构

```text
game/                  # 锁版本的 Bevy 游戏工程（试金石：网格回合制战术小品）
docs/                  # 工作流一产出，经 include_str! 纳入 doctest 门禁
assets-methodology/    # 方法论层（SOP / 任务台账 / taskset / 错题本 / 模式 / schema）
bevy-dev/              # Bevy 特定层（SKILL.md / patterns/ / pitfalls.md）
tooling/               # 自研件：task-runner（BRP 判定驱动）/ claim-lint（计数门禁）等
```

## 快速上手

```bash
cargo check                 # 每次代码变更的门禁
cargo test --doc -p docs    # docs/ 的 doctest 门禁（命令本体；作者本机以降优先级+限 4 线程形态运行，脚本见 docs/evidence/m3-assets/batch-g/run-doc-gate.ps1——该脚本含作者本机路径，仅作形态参照）
cargo run -p claim-lint     # 计数与宣称的机器门禁（13 条断言钉在真值再计算上）
cargo run --release -p game # 运行游戏（BRP 监听 127.0.0.1:15702）
```

## 核心文档

- [`Bevy-AI开发意向文档.md`](./Bevy-AI开发意向文档.md)——最高上下文：定位、三条工作流、里程碑、范围边界（M1–M4 已完成，窗口前置增强进行中）。
- [`AGENTS.md`](./AGENTS.md)——仓库硬约束（版本锁 / BRP 仅回环 / 平台 PC）与常用命令。
- [`bevy-dev/SKILL.md`](./bevy-dev/SKILL.md)——Bevy 0.19 开发纪律与已核实事实速查（含源码行号锚点）。
- [`assets-methodology/task-ledger.md`](./assets-methodology/task-ledger.md)——任务台账（T001 起逐条：类型 / 一次通过 / 返工 / 耗时 / 原因 / 证据），M3 增值证明与升级窗口工时量化的唯一数据源。
- [`docs/pre-window-plan.md`](./docs/pre-window-plan.md)——窗口前置增强定案（claim-lint / game.snapshot / MCP 薄桥）。

## 纪律亮点

- **未过验证禁止入库**：模式条目须过编译 + 运行验证；错题条目须在标注版本复现（反例代码 `compile_fail` 围栏由 doctest 机器断言）。
- **宣称钉在真值上**：`claim-lint` 把错题/模式/taskset/台账/方法/套件计数全部断言到再计算结果，负控自证留痕。
- **证据自含**：门禁日志必须带 `REAL_EXIT` 行；任务证据目录全程留痕。
- **0.20 升级窗口演练**：版本号只允许在升级窗口变动，迁移成本由 taskset 回归量化。

## License

MIT OR Apache-2.0 双许可，任选其一：[LICENSE-MIT](./LICENSE-MIT) · [LICENSE-APACHE](./LICENSE-APACHE)。向本仓库提交的贡献默认按同一双许可授权，无需附加条款。

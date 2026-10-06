# AGENTS.md

## 项目定位

搭建并验证一套「AI 长期驻场」的 Bevy 游戏开发工作流。**主产出是工作流本身与两层知识资产**，游戏只是试金石（逼出真实约束、喂真实教训），不是产品野心。任何任务的优先级判断都以这一标尺为准。

**最高上下文：[`Bevy-AI开发意向文档.md`](./Bevy-AI开发意向文档.md)（v0.3.3）**。改动仓库结构、里程碑、版本策略、范围边界等敏感区域前，必须先读该文档对应章节；与其冲突时以该文档为准。

## 当前状态（2026-09-30）

- M1–M4 已完成（台账 T001–T037；M4 验收见 `docs/m4-acceptance.md`）。骨架早已就绪：`game/`（bevy 0.19 锁版本）、`docs/`（doctest 门禁内）、`assets-methodology/`、`bevy-dev/`、`tooling/` 均在——仓库实况以 git log 与台账为准，勿依赖本节快照。
- **已暂停**：**窗口前置增强**序列（`docs/pre-window-plan.md`；grill 七项定案 2026-09-29）：T038 claim-lint、T039 开源准备与发布、T040 `game.snapshot`（含 T038 审核轮移交的代码级 S1/S3/S5 加固）均已完成过审；**T041 MCP 薄桥已冻结**（2026-09-30 owner 开工 grill：序列暂停、万阵 M0 预验证优先——新独立私有仓，全职三周自 2026-10-05 起；M0 后或 0.20 窗口时重启，任务卡存续 plan §二）。0.20 窗口开启时的冻结规则不变（先迁移后恢复）。
- **M5 试金石·万阵已收官**（2026-10-06 立项 → 2026-10-07 收官）：选品备忘录定本 v1.0（`docs/m5-game-selection.md`）全量落地——T017~T025 九卡交付（taskset ≤14 计 9/14 零触顶；系统席位 10/10；game.* 方法面 6→7 计数门禁留痕）；可玩性三合一判定 21/21（配置-观战-实验核心循环 + 跨模式确定性逐位一致 + m5-all 判定面 12/12）；观战档 10k 性能验收 PASS（avg 304.15/1% low 197.22 量级——以万阵仓判定档为准）；**M0 条件账 C-1 随席位 9 干净窗口关账**；M0 残余账 12 项处置映射终核在档。验收判定档与逐任务台账在万阵私有仓 `docs/evidence/m5/README.md`（跨仓纪律：引用不复制正文）；本仓侧无新任务——T041 保持冻结按需解冻（解冻 = 主动决策非自动触发），0.20 冻结规则不变；下一身份决策（试金石→下一步）归 owner。
- **已开源**（2026-09-29，owner 决策）：GitHub 公开仓 `lililixxx1/bevy-ai-workflow`，MIT OR Apache-2.0 双许可，全历史公开；「保持可公开态」纪律持续有效（扫描档 `docs/evidence/pre-window/open-source-scan.txt`）。

## 目标仓库结构（意向文档 §6，已定案）

```
game/                  # 锁版本的 Bevy 游戏工程（试金石）
docs/                  # 工作流一产出，经 include_str! 纳入 doctest 门禁
assets-methodology/    # 方法论层（引擎无关，可带走）
├── sop.md / task-ledger.md / taskset/ / pitfalls.md / patterns.md / pitfalls-schema.md
bevy-dev/              # Bevy 特定层（版本绑定，随升级窗口整体迁移）
├── SKILL.md / patterns/ / pitfalls.md
tooling/               # 自研件：游戏专属 RPC（launch_level / run_tests / screenshot）
```

## 硬约束（违反即返工）

1. **版本锁**：`Cargo.toml` 中 `bevy = "0.19"`，`Cargo.lock` 入库。版本号变更只允许发生在升级窗口（0.20 起的例行迁移演练），平时任何 PR 不得动它。
2. **禁凭记忆写 Bevy API**：不确定的 API 必须查 docs.rs 对应版本或官方 examples。Bevy 每 3-5 个月破坏性变更，训练语料滞后，记忆不可信。
3. **每次代码变更必须过 `cargo check`**；涉及运行时行为的变更必须走工作流三（BRP 验证闭环）。
4. **BRP 安全**：仅监听 `127.0.0.1`，禁止绑定 `0.0.0.0` 或局域网地址（BRP 无鉴权）。
5. **平台仅 PC（Windows）**，语言 Rust（最新 stable）。不做移动端、不做网络多人、不做编辑器。

## 常用命令（骨架就绪后）

- `cargo check` — 每次代码变更的门禁
- `cargo test --doc` — `docs/` 的 Markdown 经 `include_str!` 薄封装纳入 crate 文档后的 doctest 门禁；长运行示例标 `no_run` 并注明理由，禁止无理由 `ignore`
- `cargo run -p claim-lint` — 计数与宣称的机器门禁（错题/模式/taskset/台账/方法/套件/doctest 计数钉在真值再计算上；实况变化须同步 `tooling/claim-lint/src/main.rs` 的 `expect` 表，T038 起；首批 13 条受管面，扩展随任务增补）
- `cargo run --release` — 运行游戏（帧率基线采集有固定口径：release 构建、固定分辨率/vsync、记录 CPU/GPU/驱动、固定随机种子）

## 知识资产纪律

- **未过验证的条目禁止入库**（Voyager 原则）：
  - 模式条目（正例）：代码过编译 + 运行验证后才可入库；
  - 错题条目（反例）：失败须在标注版本上复现（附最小复现）+ 修复方案过编译与运行验证；反例代码统一用 ```` ```compile_fail ```` 标记。
- 每次踩坑在**同一任务收尾前、最迟下次会话开始前**回写错题本，并标注通用性分级（引擎特定 → `bevy-dev/`；方法论级 → `assets-methodology/`）。分层判定标准：换引擎后是否仍成立。
- 两层资产不掺私货（密钥/机器路径/内部黑话），**保持随时可公开态**。
- 行为、接口、版本相关的变更必须同步更新 `docs/` 或模式库；纯重构、格式化、补丁版本升级豁免。

## 任务台账

骨架就绪后立即启用，**从第 1 个任务起逐条记录**：类型 / 一次通过 / 返工次数 / 耗时 / 原因。它是 M3 增值证明与升级窗口工时量化的唯一数据源，任何任务开始前不可缺位。台账位于 `assets-methodology/task-ledger.md`。

## 术语口径

- **知识资产** = 两层资产（方法论层 + Bevy 特定层）；**自研件** = `tooling/` 下的代码产出；**工作流** = 三条流程（AI 写文档 / AI 写代码 / AI 操控运行中的游戏做验证）。

# AGENTS.md

## 项目定位

搭建并验证一套「AI 长期驻场」的 Bevy 游戏开发工作流。**主产出是工作流本身与两层知识资产**，游戏只是试金石（逼出真实约束、喂真实教训），不是产品野心。任何任务的优先级判断都以这一标尺为准。

**最高上下文：[`Bevy-AI开发意向文档.md`](./Bevy-AI开发意向文档.md)（v0.3.3）**。改动仓库结构、里程碑、版本策略、范围边界等敏感区域前，必须先读该文档对应章节；与其冲突时以该文档为准。

## 当前状态（2026-10-30）

- M1–M4 已完成（台账 T001–T037；M4 验收见 `docs/m4-acceptance.md`）。骨架早已就绪：`game/`（bevy 0.20 锁版本）、`docs/`（doctest 门禁内）、`assets-methodology/`、`bevy-dev/`、`tooling/` 均在——仓库实况以 git log 与台账为准，勿依赖本节快照。
- **0.20 升级窗口已开启并完成迁移**（2026-10-24 owner 拍板开工；Bevy 0.20.0 发布于 2026-10-08）：**T042 迁移完成**——版本锁 `bevy = "0.20"`（Cargo.lock 0.20.0；MSRV 1.97.1，工具链 1.98.1 达标）、代码面零改动过全门禁（check 0 错 0 警 / 单测 20/20 / **任务测试集 13/13 套件 45 断言全绿 = 基线恢复** / doctest 77/0/17 = 基线恢复）、BRP 运行时证据在档（discover 31、截图 1600x900、snapshot 契约）；SKILL v0.15 全批重锚 + PIT-B-015 跨版本反转修订（条目过期检测首例实战）。**意向文档 §4 完成判定三件套齐备，窗口关闭**（帧率基线重采非判定项、fps-baseline.md 已注记）。
- **窗口前置增强序列**（`docs/pre-window-plan.md`；grill 七项定案 2026-09-29）：T038 claim-lint、T039 开源准备与发布、T040 `game.snapshot`、**T041 MCP 薄桥**均已完成——T041（2026-10-29）：`tooling/mcp-bridge`（stdio MCP → BRP HTTP 薄转发，零 bevy 依赖，仅 serde_json + std）+ 仓根 `.mcp.json` 注册；工具面 12 = 8 `game.*` + 4 只读内置，白名单外 -32602 拒转发；driver C1–C8 全链路 78 断言 78/78 REAL_EXIT=0（`docs/evidence/t041-mcp-bridge/`）；同批 **T040 S2 注勘误**（`params.flatten()` 归一 params:null，详见 pre-window-plan §二）+ SKILL v0.16（§6.15）。**T043（B2 机器可读待重核清单）已完成 2026-10-29（同日立项批 + 执行批）**：`docs/recheck-registry.md` 87 条目三表（51 PIT-B + 21 PAT-B + SKILL §6.1–6.15）四字段机器可读核对表，内嵌 doctest 自校（计数 == claim-lint 字面量镜像 + 分型词表校验）挂 doctest 门禁（**DOCTEST 基线 77→78**，负控红绿留痕）；G2 通过（0 必改）；逐条目行号级重核执行归窗口空闲档/下一版本窗口。**pre-window 序列至此全部收官**。下一版本窗口开启时冻结规则不变（先迁移后恢复）。
- **M6 塔防试跑 spike（T044）已完成 2026-10-30，四条判定全过**（`docs/evidence/t044-td-spike/verdict.md`；方向 owner 2026-10-28 拍板：JerryAZR/bevy-tower-defense 26 部 MIT 教程 = 外部路线图/行为规格，主仓 `game-td/`，先试跑再立项）：① tilemap 兼容 ✓——`game-td/` 新工程（workspace 第 4 成员，bevy 0.20 + bevy_ecs_tilemap 0.20.0 锁内仅增一包）P1 红方块→P2 图集网格→P3 单 draw call tilemap，P2↔P3 像素级等价 99.96%、P3 双截图逐位一致；② 卡模板纪律 ✓——全 API 按 0.20 源码锚点重写（实测三部 17.8min vs 档位中值 120min）；③ BRP 面 ✓——discover 24 = 23 内置 + `td.screenshot` 两段式（`td.` 前缀规避 claim-lint `game.` 锚）；④ 26 部外推 = **执行带 17~27h 中值 ≈22h**。双批 G2 完整轮通过（W1 0 必改 + 5 建议全落实）。随批：SKILL v0.17（§6.16 tilemap + §6.14 Interaction/Button 弃用口径修正）、recheck-registry 16 节联动、附录 A 安全阀定量边界 + D 两条款（树 mtime 假红、共享 target 烘焙路径污染——claim-lint 已改运行时 cwd 优先）、game/src 0.19 注锚清扫 8 处（余 25 处立 T045）。
- **M6 塔防已立项（2026-10-30，T046 立项批）**：owner 获准全量立项（外推执行带 17~27h 中值 ≈22h 为决策输入）+ grill 四题定案——①扩展层 = **验证面扩展**（每波次 BRP 状态断言锚 + `td.*` 方法随部增补 + m6-all 终局判定面；玩法内容不越教程 26 部）；②收官判定 = **四件套**（26 部行为等价对照全绿〔W2 漂移表为对照底稿〕+ m6-all BRP 判定面全绿 + 无 P0 崩溃 + 台账/taskset 纪律无断档；性能/音频非门槛）；③体量 = **日历周 ≤3（自 2026-10-30）+ 系统席位 ≤10 + taskset 新增 ≤10**（触顶即 owner 裁决，防失控条款照沿）；④T045 清扫卡随首波次并行。产出：`docs/m6-game-selection.md` 定本 v1.0（§8 五条对照含实时制诚实注记 + 系统席位表 + td.* 方法面）+ 意向文档 **v0.3.5**（§4 M6 行 + §6 结构图 game-td/ + §6 版本锁 0.19→0.20 勘误〔T042 漏同步补正〕+ §8 M6 选品段）。P26 DynamicScene 裁弃（L1-2）、P17/P23 `#[expect(deprecated)]` 通道（A'）沿 T044 裁决。**本仓下一动作 = M6 26 部执行·首波次**（T047 起，按波次派工复用三层协议；W2 漂移表为对照底稿、`trees/m6-recon/` 教程源续用至 M6 收官；T045 清扫卡随首波次并行）。
- **M5 试金石·万阵已收官**（2026-10-06 立项 → 2026-10-07 收官）：选品备忘录定本 v1.0（`docs/m5-game-selection.md`）全量落地——T017~T025 九卡交付（taskset ≤14 计 9/14 零触顶；系统席位 10/10；game.* 方法面 6→7 计数门禁留痕）；可玩性三合一判定 21/21（配置-观战-实验核心循环 + 跨模式确定性逐位一致 + m5-all 判定面 12/12）；观战档 10k 性能验收 PASS（avg 304.10/1% low 197.22 量级——以万阵仓判定档为准，基线采自 0.19.1）；**M0 条件账 C-1 随席位 9 干净窗口关账**；M0 残余账 12 项处置映射终核在档。验收判定档与逐任务台账在万阵私有仓 `docs/evidence/m5/README.md`（跨仓纪律：引用不复制正文）；本仓侧无新任务；身份决策 C1（收官归档→回归工作流主线）已执行。
- **已开源**（2026-09-29，owner 决策）：GitHub 公开仓 `lililixxx1/bevy-ai-workflow`，MIT OR Apache-2.0 双许可，全历史公开；「保持可公开态」纪律持续有效（扫描档 `docs/evidence/pre-window/open-source-scan.txt`）。

## 目标仓库结构（意向文档 §6，已定案）

```
game/                  # 锁版本的 Bevy 游戏工程（试金石）
game-td/               # 塔防试金石工程（M6；bevy_ecs_tilemap 0.20.0 锁内）
docs/                  # 工作流一产出，经 include_str! 纳入 doctest 门禁
assets-methodology/    # 方法论层（引擎无关，可带走）
├── sop.md / task-ledger.md / taskset/ / pitfalls.md / patterns.md / pitfalls-schema.md
bevy-dev/              # Bevy 特定层（版本绑定，随升级窗口整体迁移）
├── SKILL.md / patterns/ / pitfalls.md
tooling/               # 自研件：游戏专属 RPC（launch_level / run_tests / screenshot）
```

## 硬约束（违反即返工）

1. **版本锁**：`Cargo.toml` 中 `bevy = "0.20"`，`Cargo.lock` 入库。版本号变更只允许发生在升级窗口（下一大版本起的例行迁移演练；0.19→0.20 先例 = T042），平时任何 PR 不得动它。
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

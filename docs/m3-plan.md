# M3 执行计划（沉淀复用）

- 适用 Bevy 版本：0.19（`Cargo.lock` 解析 0.19.1；本文为流程计划文档，不含 API 示例代码）。
- 日期：2026-09-27（Block A 起草）。状态：随各块推进更新 §七执行记录。
- 依据：意向文档 §4 M3（①-④）、§5.2（资产入库门禁）、§6（分层）、§7（资产不增值风险行）；owner 2026-09-27 指令「开始 M3」。
- M1/M2 终审状态：M1 有验收建议=通过待确认（`docs/m1-acceptance.md`）；M2 无独立验收结论文档，三项验收证据分存 `tooling/brp-logs/验证报告.md`（①兼容性）与 `docs/evidence/m2-rpc.md`（②全链、③自动判定），待 owner 终审。owner 未另作批示即指令开工 M3，本计划按「M1/M2 以既有证据为准继续」推进；若 owner 另有裁决，以裁决为准并回写本行。
- 日历时间盒：未设（意向文档 §4 里程碑执行规则：时间盒待 owner 终审时设定）；块序执行，owner 可随时叫停。

## 一、验收映射与块划分

M3 成功标准（意向文档 §4，逐条可验收）：

| 条目 | 标准 | 本计划落点 |
|---|---|---|
| ① | 方法论层 + Bevy 层结构落地（§6） | Block B：结构对照核对 + 两处内容缺口收口（sop 操作步骤回填、方法论模式首批入库） |
| ② | Bevy 层错题本 ≥ 50 条、模式库 ≥ 20 条，全部过验证门禁（§5.2） | Block A 先落地门禁机制升级；Block C–G 五批生产（方法学见 §三） |
| ③ | 任务台账连续记录 | 全程：每块必登（T021 起）；已核 T001–T020 无断档（2026-09-27） |
| ④ | 台账返工 ≥1 任务抽 ≥3 重跑，返工率显著下降（口径以台账实测定标） | Block H：取样与量化口径**本文件 §四预注册**，C–G 完成后执行 |

块划分（每块独立台账条目 + plan-code-reviewer 审核 + 提交，沿 M1/M2 先例）：

| 块 | 内容 | 状态 |
|---|---|---|
| A | M3 启动：工作树修复（陌生重跑存档+恢复）+ 本计划落档 + 错题本 doctest 门禁升级（pitfalls.md 纳入 `cargo test --doc`，存量 4 条适配）+ SKILL.md v0.6 同步 | 已完成（2026-09-27，ad4953e） |
| B | ① 收口：sop.md 三工作流操作步骤回填（M1/M2 实战提炼）+ `assets-methodology/patterns.md` 首批方法论模式（≥5 条 PAT-M）+ schema 过程型模式验证口径补注 + §6 结构对照表 | 已完成（2026-09-27） |
| C | ② 批次一：ECS 查询与调度域 | 已完成（2026-09-27） |
| D | ② 批次二：事件 / observer / State 域 | 已完成（2026-09-27） |
| E | ② 批次三：反射 / BRP 深水区 | 已完成（2026-09-27） |
| F | ② 批次四：资产 / 场景 / 时间 / 输入域 | 未开始 |
| G | ② 批次五：渲染 / 窗口 / UI / 数学域 | 未开始 |
| H | ④ 执行：T003/T004/T005 净室重跑（§四） | 未开始 |
| I | M3 验收文档（`docs/m3-acceptance.md`，①-④ 分列证据 + 局限声明）+ 终审材料 | 未开始 |

> 域清单为初排，批次内按探查实况调整次序与内容（某域无可入库条目时如实记 0 并换域，不硬凑）；调整回写本表。

## 二、现状基线（快照时点 = Block A，2026-09-27 提交 72de272 + ad4953e；Block B 后最新计数见 §五对照表）

| 资产 | 现状 | M3 目标 | 缺口 |
|---|---|---|---|
| bevy-dev/pitfalls.md（PIT-B） | 4 条（PIT-B-001..004） | ≥ 50 | +46 |
| bevy-dev/patterns/（PAT-B） | 1 条（PAT-B-001） | ≥ 20 | +19 |
| assets-methodology/pitfalls.md（PIT-M） | 7 条（PIT-M-001..007） | （无硬数） | 按实沉淀 |
| assets-methodology/patterns.md（PAT-M） | 0 条（骨架） | 结构落地（Block B 首批 ≥5） | 内容空 |
| bevy-dev/SKILL.md | v0.5（Block A 升 v0.6） | 随 ② 持续增补事实速查 | — |
| taskset | TS-01..12（判定记录两阶段齐） | 新任务按需入集 | — |
| 台账 | T001..T020 连续无断档 | 连续记录 | — |

④ 取样池（台账返工 ≥1，实测定标）：T002(1) / T003(1) / T004(2) / T005(1) / T018(1) / T019(2) / T020(4)，共 7 条。

## 三、② 生产方法学（Bevy 层 ≥50 错题 / ≥20 模式）

### 3.1 三条产出渠道

1. **新任务真实失败**（首选坑源）：继续以小功能任务推进试金石（每批可含 1 个 taskset 候选任务，如 `launch_level` RPC、UI 面板、状态机暂停菜单），失败即坑、可运行代码即模式候选。与 M1 期完全同口径——不为了让台账好看而绕开真实首试。
2. **系统性 API 域探查**（主力渠道）：每批取一域（§一块 C–G），按固定时序探查——
   1. **凭记忆首试**：按训练语料直觉写该域典型用法（正是错题本要防的对象），`cargo check` / 运行，失败原文与真实退出码归档（`docs/evidence/m3-assets/batch-*/`）；
   2. **查证修正**：按 SKILL.md §2.1 查本地源码 / docs.rs 精确版本，改出正确形态；
   3. **双重验证**：修复形态过 `cargo check` + 运行验证（行为级走 BRP / run_tests 套件 / task-runner 口径）；
   4. **入库**：失败 + 根因 + 修复 + 证据齐 → 按模板追加 pitfalls.md；修复中可复用的真实代码 → 落进 game/tooling 后按 PAT-B 模板入库。
   **诚实性红线（违反即证据造假）**：时序不可倒置——先查源码再「构造失败」不构成错题；每条反例必须来自真实首试或真实任务失败。条目须过通用性判定（bevy-specific：换引擎不再成立；纯 Rust 语言级错误不属 Bevy 错题，不充数）。凭记忆首试代码限探查用临时载体（探针文件 / 临时 crate），不进 `game/`、`tooling/` 生产路径；查证修正后的形态过门禁后才可落地为生产代码。
3. **既有已验证代码的模式化**（PAT-B 补充渠道）：从 `game/src/`（sim / camera / rng / cli / bench / rpc 三方法 + 11 套件）与 `tooling/task-runner` 挖掘已过编译 + 运行验证的真实代码入库，证据指向原任务（PAT-B 模板「源码出处」字段即为此设；删减后未重新编译的按模板注释要求注明）。

### 3.2 门禁机制升级（Block A 落地）

- `bevy-dev/pitfalls.md` 经 `include_str!` 纳入 `docs` crate（`docs/src/lib.rs`），`cargo test --doc -p docs` 成为错题反例的**机器断言**——兑现 `assets-methodology/pitfalls-schema.md` 已写明的「doctest 断言其编译失败」口径（此前该口径对本文件仅为标记约定，无机器执行）。
- 围栏约定（写入 pitfalls.md 文件头，对其后全部条目生效）：
  - `rust,compile_fail` 围栏：反例，doctest 断言编译必败；
  - `rust,ignore` 围栏：非编译载体的反例（附理由），或非 self-contained 的修复片段（附理由 + 完整代码出处）；
  - `rust` / `rust,no_run` 围栏：self-contained 修复正例（可测即测；长运行标 no_run 附理由，沿 `docs/doc-conventions.md`）。
- 升级窗口红利：0.20 迁移时若某反例在新版本能编译，`cargo test --doc` 立即红——错题本自动过期检测，服务意向文档 §6「升级视为独立里程碑」。
- `patterns/` 与 `SKILL.md` 维持不入 doctest（片段多为删减形态），门禁仍为逐条证据（模板口径不变）。

### 3.3 批次规格

- 每批目标：PIT-B 8–12 条 + PAT-B 3–5 条（五批合计预期 ≈50 错题 / ≈20 模式，含存量 4/1）；实际以探查结果为准，禁止凑数——某域探完不足预期属正常，缺口由后续域 / 新任务补。
- 每批交付：pitfalls.md / patterns 增量 + 探查证据归档 + 台账条目（类型=资产回写）+ plan-code-reviewer 审核 + 提交。探查过程轮（attempt）的失败日志与当轮源码快照一并归档（Block C 审计建议：attempt 留存升级为规格，下批起执行）。
- 运行时验证沿用 M2 口径：`game.run_tests` 套件 / task-runner / curl 直调，按 SKILL.md §4.2；常驻游戏进程一律后台任务托管（PIT-M-007）。

## 四、④ 取样与量化口径（预注册，Block H 执行前不得修改）

> 预注册目的：防事后挑样 / 改口径。执行结果无论正负如实入 `docs/m3-acceptance.md`。

1. **选样（锁定）**：T003（代码）、T004（验证）、T005（验证）——理由：三条均 M1 期任务，重跑语义可复原；类型覆盖代码 / 验证；返工原因与资产因果链最强（T003 首败三类 → PIT-B-001/002；T004 探针参数形态错 → PIT-B-004 + SKILL §6.2/§6.4；T005 导入与格式捕获错 → SKILL §6.5 + 日志事实）。T018/T019/T020 为 M1/M2 收尾与工具任务，重跑=重做已存在代码，污染不可控，不入选（取舍理由如实记录于此）。T002 重写 SKILL.md 的语义被 v0.5 既有文件污染（重跑产物即现存文件），同样不入选。
2. **重跑口径（净室）**：
   - 工作区：scratch（`.zcode/` 下或仓库外临时目录），禁止读 `game/src`、`tooling/` 既有实现与 `docs/evidence/` 旧证据（裁判材料）；
   - 注入材料：AGENTS.md 纪律章节（硬约束 / 知识资产纪律 / 任务台账 / 术语——不含「当前状态」节，避免过时状态误导净室执行者）+ `bevy-dev/` 全部（SKILL.md + pitfalls.md + patterns/）+ `assets-methodology/` 全部 + 该任务原始需求 / 验收清单原文（从台账与意向文档复原，与原始口径的差异显式标注）。taskset/ 对 T003 属其自身历史交付物（T003 即首批测试集作者）——重跑需求书裁剪为 demo 主体（相机漫游 + N 动态实体 + BRP 常驻 + 基线），测试集产出部分不重做（与 T018 排除同逻辑），裁剪差异在 H 块需求书显式标注；
   - 判定：与原判定同口径（首次 `cargo check` 过 + 首次运行满足验收清单 = 一次通过；返工 = 首判未过后的重做次数，台账 schema，**含独立审核轮引发的重做**——与台账 T004/T019/T020 先例一致；原基线 4 次中即含 T004 的审核返工 1 次，重跑同口径对称，防止对照系统性偏向新结果）；
   - 局限（验收文档必注明）：同一 AI 模型执行、仓库历史可能经训练语料泄漏——不声称完美净室，仅声称「流程净室」（执行上下文与材料注入受控）。
3. **量化口径（定标，依台账实测）**：原三条任务合计返工 4 次（1+2+1，均值 1.33/任务）。**判定「返工率显著下降」= 重跑合计返工 ≤ 1 且单任务返工 ≤ 1**（降幅 ≥ 75%）。合计 ≥ 2 → 未达显著，如实记录负结果并走意向文档 §7「资产不增值」处置（以台账定位失效环节：条目不准 / 检索不到 / 注入时机），由 owner 裁决返工 / 降标准 / 终止。验收文档除判定结果外**逐任务列出原始与重跑的返工数据**，owner 可以任意口径复核（避免单一阈值结论的事后争议面）。预注册时间戳凭据 = 本文件所在 git 提交 hash，Block H 报告须引用。
4. **时点**：Block H 在 C–G（资产充实）完成后执行——增值假设检验要求先有资产后有对照。
5. **取样前不重做**（意向文档 §4 M3④）：M1 失败任务以 M1 验收时点快照为准——本仓 M1 期 12 个 taskset 任务全部一次通过（无失败任务），该约束自然满足；T003/T004/T005 为工程任务（非 taskset 条目），M1 验收后未重做过，重跑即首次复做。

## 五、①③ 的收口与核对

- ①（Block B，2026-09-27 已完成）：
  - `sop.md` 三工作流「操作步骤」回填为 M1/M2 实战提炼（工作流一：doctest 门禁操作链；工作流二：查证 → 写码 → check → 转工作流三；工作流三：后台托管启动 → 冒烟 → 状态断言优先 → 自动判定 → 杀进程收尾），状态由「骨架」升「实战回填 v1」；
  - `assets-methodology/patterns.md` 首批 **7 条** PAT-M（001 真实退出码判定 / 002 两阶段判定 / 003 字面值绑定断言 / 004 无净副作用套件 / 005 常驻进程后台托管 / 006 摘录逐字回对 raw / 007 驱动脚本绝对锚定）——全部过程型，验证证据逐条指向实际执行任务与留痕（PIT-M-001/004/005/006/007 的正例面配对入条）；
  - `pitfalls-schema.md` 补注模式条目分型：代码型 = 编译 + 运行验证（不变）；过程型 = ≥1 任务实际执行并留痕；
  - §6 结构对照表（逐项核对，2026-09-27）：

| 意向文档 §6 条目 | 实际状态 | 核对 |
|---|---|---|
| `game/` 锁版本游戏工程 | bevy = "0.19"（workspace 继承）+ Cargo.lock 入库 | ✓ |
| `docs/`（include_str! doctest 门禁） | `docs/src/lib.rs` 封装 8 个 Markdown 模块（含 bevy-dev/pitfalls.md，Block A 起） | ✓ |
| `assets-methodology/sop.md` | 实战回填 v1（本块） | ✓ |
| `assets-methodology/task-ledger.md` | T001..T024 连续（E 块 T025 随提交落入） | ✓ |
| `assets-methodology/taskset/` | TS-01..12 + README（含回归口径注意事项） | ✓ |
| `assets-methodology/pitfalls.md` | PIT-M-001..007 | ✓ |
| `assets-methodology/patterns.md` | PAT-M-001..007（本块首批） | ✓ |
| `assets-methodology/pitfalls-schema.md` | schema + 过程型分型补注（本块） | ✓ |
| `bevy-dev/SKILL.md` | v0.9（C §6.10 / D §6.11 / E §6.12） | ✓ |
| `bevy-dev/patterns/` | PAT-B-001..010（C +4 / D +3 / E +3→实际 +2，批次三新增 009–010）+ README + 模板 | ✓ |
| `bevy-dev/pitfalls.md` | PIT-B-001..025（C +9 / D +7 / E +5，探针双重验证）+ doctest 门禁（Block A 起） | ✓ |
| `tooling/`（自研件） | task-runner / hotpatch-smoke（crate）+ brp-logs（证据目录，workspace exclude）；run_tests / screenshot 以 game 侧 BRP 自定义方法落地（`game/src/rpc/`，SKILL §3.6 定案）、launch_level 未建（§3.1 候选任务） | ✓ |
| 两层不掺私货 | 无密钥 / 机器路径硬编码（`<reg>` 类以「实测为准」写法） | ✓ |

  **结论：意向文档 §6 结构全部落地，① 的内容缺口（sop 步骤、方法论模式）本块收口——M3 ① 达成（待 M3 验收文档复核确认）。**
- ③（全程）：台账每块必登；M3 验收时核 T001..T0xx 连续无断档、字段齐备。

## 六、风险与对策（M3 特有）

| 风险 | 对策 |
|---|---|
| 资产不增值（④ 未达显著） | §四预注册口径如实记录负结果 → 意向文档 §7 处置路径（定位失效环节，owner 裁决）；负结果本身入方法论层资产 |
| 为凑 50/20 硬造条目 | §3.1 诚实性红线 + 每批独立审核（审核方核对失败原文与证据归档）；不足即如实记录，走 §4 未达标处置 |
| 批量探查稀释条目质量 | 每条过完整门禁（复现 + 根因 + 修复 + 双重验证），宁少勿滥；批次目标是区间不是死数 |
| doctest 门禁升级拖慢迭代 | compile_fail / ignore 围栏无运行负担（compile_fail 仅需编译失败一次）；workspace check 增量在秒级（T016/T017 实测 0.57s / 0.52s） |
| 重跑净室被上下文污染 | §四局限声明；重跑在独立会话执行、上下文显式最小化（仅注入材料） |

## 七、执行记录

- **Block A（2026-09-27）**：工作树修复——提交 72de272 之后一次未经记录的 `run-all.sh` 重跑（本日 13:29）覆盖了 `docs/evidence/m1-phase2/` 下 54 个证据文件，且新 summary 丢失 round2 取代 round1 的出处注记；判定结论未变（12 任务退出码全 0），但至少 ts-11 的测量数值族与提交态不可比（存档实测 brp-avg 99.90 / [BENCH] avg_fps 95.7，当次 vsync 未实际生效——非 60fps 基线族），重跑产物非等值证据；处置：全量存档至会话内部区（`.zcode/`，gitignore，不入库）后 `git restore` 恢复提交态，待 owner 知悉。本计划落档（§四预注册 ④ 口径；审核后补返工计数含审核轮、注入边界细则）；pitfalls.md 纳入 doctest 门禁（存量 4 条按围栏约定适配）；SKILL.md v0.6（门禁覆盖面同步 + §6.5 Hotpatching 冒烟结论回写 + 断言计数 77→78 勘误×2）；pitfalls-schema.md 补 doctest 适用面注记；台账 T021。
- （后续块逐条追加）
- **Block B（2026-09-27）**：① 收口——sop.md 三工作流操作步骤实战回填（骨架 → v1）；patterns.md 首批 7 条过程型 PAT-M（全部过过程型门禁：≥1 任务执行留痕）；pitfalls-schema.md 模式条目分型补注（代码型门禁不变）；§五 结构对照表 13 项全 ✓，**M3 ① 达成**。审核与门禁结果见台账 T022。
- **Block C（2026-09-27）**：② 批次一（ECS 查询与调度域）——仓库外探针 crate「凭记忆首试」（10 编译级 + 4 运行时假设，时序合规：r1 失败原文先归档后查源码）；产出 **PIT-B-005..013 共 9 条**（编译级 5 + 运行时 4，全部过「复现+根因（源码行号）+修复（check/运行验证）」门禁，compile_fail 反例已入 doctest 机器断言）+ **PAT-B-002..005 共 4 条**（spawn_batch owned 迭代器 / Startup 链式定序+auto_insert_apply_deferred / 确定性 PRNG 单一抽取落点 / 解析式运动 BRP 可复算，均出自本仓已验证代码）；无坑确认 3 项独立列出（iter_many、Res::is_changed、run_if 捕获闭包）+ configure_sets 并入 PIT-B-009 行。**事实发现（含一次勘误）：bevy 0.19.1 门面 default=[2d,3d,ui,audio] 本身不含 multi_threaded，但经 2d/3d/ui→default_platform（Cargo.toml:2762-2774，:2768）传递启用——本仓 game/docs 实际跑 MultiThreadedExecutor（cargo tree -e features 实证）**；本行初版误判「默认单线程」，独立审核以 cargo tree 纠正，已同步 SKILL §6.10 与 PIT-B-006 勘误注。计数：PIT-B 13/50、PAT-B 5/20。审核与门禁见台账 T023。
- **Block D（2026-09-27）**：② 批次二（事件 / observer / State 域）——探针两轮首试：r1 轮 10 编译探针失败 3 函数 4 处（P4/P6/P9）+ 4 运行时假设 3 panic（R1 未注册消息 / R3、R4 缺 StatesPlugin）；r1b 轮追加 3 个「先验摇摆」探针全失败（P11 iter_mut / P12 On<Message<T>> / P13 register_event）——两轮失败原文均先归档后查源码（时序合规）。r2 正解：编译面 `World::trigger`/`Commands::trigger`、`On<Add, T>`、`MessageMutator::read()` 全过 check；运行时正解 r1c/r2b/r3c/r4c 四探针 REAL_EXIT=0。产出 **PIT-B-014..020 共 7 条**（编译级 4：缓冲事件 API 全面不存在 / `On<Add<T>>`→第二泛型 / `iter_mut`→`read` / `On<Message<T>>` 不存在；运行时 3：未注册消息 panic / StatesPlugin 缺装 panic+不在 prelude / 双缓冲语义误读）+ **PAT-B-006..008 共 3 条**（定时自退出采集 / 反射 Event+observer 资源翻转 / run_if 门控+chain）。**架构级发现：Event（纯触发，无缓冲）与 Message（双缓冲队列，不可 trigger）分流总纲；State 转换内部也走 Message 系（init_state 即 add_message::<StateTransitionEvent> + write_message 初始 entered 转换，bevy_state/src/app.rs:99-112）；StatesPlugin 不在 prelude，完整路径 `bevy::state::app::StatesPlugin`**；语义确认（无错不入错题本，进 SKILL §6.11）：初始 OnEnter 恰一次（R3c）、NextState::set 后一次 update() 完成转换（R4c）；R2b 双缓冲语义两面处置——误读面以 PIT-B-020 入错题本、正面语义（错峰可读）进 SKILL §6.11。过程诚实记录两笔：①lib.rs P12 注释初版「Message 是 derive 宏」与 E0782 原文（found a trait）不符——修正注释后重跑 check 并重做最终快照（证据—源码一致性优先）；②r2 轮首写 import 凭 prelude 惯例触发 E0425——该失败真实发生于查证后（会话内即时输出），为满足「失败须归档」纪律以复现实验重跑留痕（`probe-import-prelude-check.log`：临时注释正确 import → check REAL_EXIT=101 → 恢复）。计数：PIT-B 20/50、PAT-B 8/20。**独立审核（plan-code-reviewer）：有条件通过 → 4 必改 + 9 建议 + 4 备注全数落实**（必改：PAT-B-006 注记「逐字删减+两处行内注解为条目添加」+ 区间 58-108、PAT-B-008「未删减但系 build 链中段片段不独立编译」、PIT-B-015/SKILL observers.rs 行号 :149→:152/:144→:143/区间 142-152、本行 R2b 分类措辞两面化；建议：batch-d/provenance.md 出处页（E0425 复现实验时序）、event/mod.rs 引文补全尾句+去链接化标注+区间 :16-18、摘录统一 `...` 省略标记、messages.rs/sub_app.rs 行号微精化、幂等归属 contains_resource 守卫、014/015/017 fence 补观察者计数断言、020 ignore fence 补证据指向、PAT-B-008 标签行修复、PAT-B-007 doc 段注记改准确；备注：探针 main.rs 注释 :108-113→:107-112 勘误 + check 重跑 + 快照重制）。门禁（审核后终跑）：`cargo check --workspace` REAL_EXIT=0；`cargo test --doc -p docs` REAL_EXIT=0（19 passed / 15 ignored）。审核与门禁见台账 T024。
- **Block E（2026-09-27）**：② 批次三（反射 / BRP 深水区）——探针 crate bevy-probe-e（target 缓存与 probe-d 共享）「凭记忆首试」：r1 轮 lib 5 失败（clone_value / from_world bounds / #[reflect_value] 属性不存在 / get_with_short_name / TypeRegistry 非 Resource）→ 注释隔离后 r1b 轮 bin 6 失败（E0277 ×3 + E0599 ×3，含**裸 World 无 register_type**）——两轮失败原文均先归档后查源码（时序合规）。r2 正解 + 两笔**修正轮真实发现**（如实留痕）：①#[reflect(opaque)] derive 要求 Clone（E0277 help 明示）；②RwLock 守卫内联 &arc.read() 传参 E0716。运行时四探针：r1c 首断言「注册 1 型得 1 型」被证伪（实测 20 型——**register 注册依赖闭包**，r1c 失败轮留档）→ r1c2 按闭包语义复测 PASS；r2c（ReflectSerializer 输出形态：type_path 外包裹+字段内层，BRP 响应同源）/ r3（apply 深拷贝）/ r4c（短名查）PASS。产出 **PIT-B-021..025 共 5 条** + **PAT-B-009..010 共 2 条**（BRP 自定义方法组装 / 异步截图受理-轮询契约，源码 brp.rs + rpc/）。SKILL v0.9 §6.12。**doctest 机器断言再抓两笔真发现**（首跑失败轮补录 `gate-doc-test-attempt1.log`，修复后复跑 28 passed）：①仅 `derive(Component, Reflect)` 不注册 `ReflectComponent` 数据——须 `#[reflect(Component)]` 属性（并入 PIT-B-022）；②`reflect_clone` 归属 `PartialReflect`（首版 doctest E0599 真因 = 漏 import，独立审核实测纠正；opaque 可克隆须 `#[reflect(Clone)]`，否则运行期 NotImplemented——并入 PIT-B-023）。计数：PIT-B 25/50、PAT-B 10/20。审核与门禁见台账 T025。

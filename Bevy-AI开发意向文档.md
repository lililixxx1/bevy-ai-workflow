# 意向文档：AI 驱动的 Bevy 游戏开发工作流

- 版本：v0.3.3（开源决策与编号勘误版）
- 日期：2026-09-29
- 状态：方向已定，关键决策已 grill 定案，plan-code-reviewer 审核必改项已修订
- 本文档用途：作为项目的最高层上下文，喂给一切参与本项目的 AI 工具（ZCode / Claude Code / Cursor / MCP agent 等），约束它们的目标、边界与纪律。
- 用词口径：**知识资产** = §6 两层资产；**自研件** = `tooling/` 下的代码产出；**工作流** = 三条流程本身。

---

## 1. 一句话定位与主产出定义

搭建并验证一套「AI 长期驻场」的 Bevy 游戏开发工作流。**主产出是工作流本身与它沉淀的知识资产**——三条工作流（AI 写文档 / AI 写代码 / AI 操控运行中的游戏做验证）+ 两层知识资产（引擎无关的方法论层 + Bevy 特定层）。游戏是**试金石**而非产品野心：它的作用是把工作流逼到真实约束下验证有效性，并持续给知识资产喂真实教训。

判定项目成败的隐含标尺：游戏成了 + 资产沉淀了 = 双赢；游戏没成但资产沉淀了 = 项目仍成立；两头都空 = 失败。因此 M1-M3 的验收全部围绕工作流与资产设计，不围绕游戏好不好玩。

## 2. 背景与动机

### 2.1 为什么是 Bevy
- 性能潜力强：纯 Rust + ECS，GPU 驱动渲染（0.16 起）。官方以 Activision Caldera 场景做基准：帧时间较 0.15 提升约 3 倍（移动版 RTX 4090 上 10.16ms ≈ 100fps），数万对象的大场景 CPU 开销普遍降低 3 倍以上——CPU 密集场景（海量实体模拟）是其甜点区。
- 架构对 AI 天然友好：万物皆代码。0.19 的 BSN 场景系统当前以 `bsn!` 宏形态存在——场景是类型安全、可 diff、可编译验证的 Rust 代码，本就是 AI 擅长操作的形态；官方规划的 `.bsn` 纯文本资产格式（可 diff、可版本控制、免编译）尚未发布，预计随后续版本落地，届时再引入。
- 零成本零锁定：MIT/Apache 双协议，无分成、无授权费、无黑盒。
- **AI 工具生态是洼地**（对照 Godot 阵营 11+ 个严肃工具、Unity 侧 9500+ star 的 MCP for Unity；Bevy 侧 2026-09-28/29 四项目实测：≥3 条 BRP 桥 + 若干非 BRP 的 MCP 面，但全部为早期 / 低星（≤72★）/ 单作者项目，明细见 §3）——洼地意味着自建的工作流与资产有稀缺价值，也意味着什么都得自己动手，与「生态要自己建」的判断互相印证。（v0.3.2 修订：原「仅有的两个早期 MCP」系 2026-09-26 时点表述，已被调研证伪。）

### 2.2 已知短板（不回避，写进风险清单管理）
- 无官方编辑器（社区原型 Jackdaw 被官方六周年博文评为 "dangerously functional"；官方编辑器连 baseline 都未落地，保守估计仍需 1-2 年）。
- API 每 3-5 个月破坏性变更，AI 训练语料滞后且总量少，文档稀疏——这正是本项目要靠「锁版本 + 编译器门禁 + 错题本」对冲的核心矛盾，也是知识资产分两层的根本原因（特定层注定随版本作废，方法论层保值）。
- 社区 2026 年实行严格 no-AI 政策，且据官方 2026-08 博文该政策正因执法困难与副作用被重写（新政策未定）。不会等来官方 AI 生态——独立仓库自建自用无障碍，向上游提代码则遵守届时政策。

### 2.3 为什么现在做
三个前置件在过去三个版本里陆续落地：GPU 驱动渲染（0.16，2025-04）→ Rust Hotpatching 初步集成（0.17，2025-09）→ BSN 场景系统（0.19，2026-06）；加上官方 `bevy_remote`（BRP，JSON-RPC over HTTP，内置 `world.query` / `world.spawn_entity` / `world.mutate_components` / `registry.schema` 等完整方法集）。前置件齐了，「AI 操控 Bevy」从设想变成了工程问题。**已定案：立即锁 0.19 起步，不等 0.20；0.20（预计携带 `.bsn`）发布即触发首个升级窗口。**

## 3. 同类方案调研对照（2026-09-26）

**结论先行：桥不稀缺，闭环 + 知识资产才稀缺。** 各阵营已把「MCP 桥接引擎」卷成红海，但没有人把「文档流 + 代码流 + 验证流」连同**可带走的知识资产**做成一个整体闭环——这是本项目的差异化位置。（2026-09-29 注：Bevy 侧桥规模实测升为 ≥3 条 BRP 桥 + 非 BRP MCP 面——前半句仅规模更新、方向不变，后半句结论被 4 个实测样本反而加强。）

| 方案 | 是什么 | 对本项目的含义 |
|---|---|---|
| [bevy_brp](https://github.com/natepiano/bevy_brp) workspace（含 `bevy_brp_mcp`，2026-09 归档迁入） | BRP 工具 workspace：MCP 封装 + extras（截图/输入，需游戏侧加装 bevy_brp_extras）；72★、2,532 commit、活跃；crates.io `bevy_brp_mcp` 0.22.7（依赖 bevy ^0.19.1） | **通用桥复用对象**（已定案）。T004 实测 0.22.7 ↔ bevy 0.19.1 协议级兼容（32 次 tools/call 无一协议性失败）；仓库 license 未检出，引代码前须核许可 |
| [bevy_mcp](https://github.com/Nub/bevy_mcp) | BRP→MCP 单作者桥：13 工具、bevy 0.18、2 commit，2026-02 起休眠 | 佐证「桥不稀缺」；无额外借鉴面 |
| [Plinth](https://github.com/Luminary-Analytics/plinth) | Bevy 门面 + agent 工具链：MCP 薄桥（7 工具）/ 精确 tick 步进 / 稳定 id 快照；pre-0.1、休眠约 3 月、crate 为 stub；MIT OR Apache-2.0 | **机制层最高价值参照**（稳定 id 快照与 MCP 薄桥形态；机制引用不引代码）；不作依赖 |
| [SpawnForge](https://github.com/Tristan578/project-forge) | 浏览器 Bevy/WASM 引擎 + React 编辑器 + MCP 面（**非 BRP**：本机 relay + 页内桥）；9★ 活跃；380 命令注册 vs 2 proven、外部 MCP 生产不通；BSL 1.1（non-production，2030-02-11 转 Apache-2.0） | 方法论参照：把宣称钉在机器门禁上（能力矩阵）；反面教材：宣称 > 实证 |
| [Axiom](https://github.com/cats2333/bevy_ai_editor) | egui 桌面编辑器 + BRP 写通道（组件代理技法：Reflect 组件 + `Added<T>` handler）；13★、休眠约 7 月；声明 MIT/Apache 但仓库无 LICENSE 文件 | 组件代理作 0.20 自定义方法签名变更时的逃生通道预案；编辑器路线不翻案（§9） |
| [bevy-agent](https://github.com/jbuehler23/bevy-agent) | CLI 单次 LLM 生成 Rust：无编译回灌 / 无运行时验证 / 版本硬编码 0.12；12★、废弃 | 反面教材：无验证闭环的代码生成（本项目工作流二的严格子集） |
| [Jackdaw](https://github.com/jbuehler23/jackdaw) | 社区编辑器原型（520★、活跃、非 AI） | 佐证 §2.2「dangerously functional」评价与官方编辑器空窗 |
| [bevy_debugger_mcp](https://github.com/ladvien/bevy_debugger_mcp) | Claude Code + MCP + BRP 调试器，11 个工具，有测试但 vibe coded | 备选与参照，其工具清单可借鉴 |
| [bevy 官方讨论 #20042](https://github.com/bevyengine/bevy/discussions/20042) | 社区提案：游戏内可插拔 MCP Server | 佐证「AI 操控 Bevy」方向获社区关注；也提醒未来官方/半官方方案可能吸收自建件 |
| Godot / Unity MCP 阵营 | Godot 11+ 工具（Godot-MCP、tomyud1/godot-mcp、Godot MCP Pro 162 工具、Fennara 的诊断+截图+patch-and-rerun）；Unity 侧 MCP for Unity 9500+ star，同团队产品已上架 Godot 官方商店 | 同类能力的成熟参照系：场景/脚本操作、截图、跑测试、运行时错误反馈、patch-and-rerun 是业界收敛的能力面；自研件应对标此能力面设计，勿低于水位 |
| [Voyager](https://arxiv.org/abs/2305.16291)（NVIDIA/Caltech） | LLM 终身学习 agent：自动课程 + **技能库**（可执行代码、embedding 索引、成功才入库）+ 环境反馈迭代 | 知识资产的总范式：只收验证过的、条目化、可检索、可组合。本项目的模式库/错题本是其在开发域的手工版，纪律直接吸收：**未过验证的条目禁止入库** |
| [GameLogicBench](https://arxiv.org/html/2609.21562) / TITAN / Orak / GameGen-Verifier | 学界：LLM 游戏测试与自动验证基准（72 个 Godot 任务、每 tick 断言规则、关键点分解 + 运行时状态注入） | 两条直接吸收：①**状态注入验证 > 纯截图验证**（截图当辅助证据）——run_tests 自研件的设计依据；②**任务基准化**思路——任务测试集既量一次通过率，又当版本升级后的回归集，量化迁移成本 |

## 4. 总体目标与成功标准

| 阶段 | 目标 | 可验收的成功标准 |
|---|---|---|
| M1 飞轮跑通 | AI 能写出可编译的 Bevy 代码，且指标可复测 | 建立 ≥ 10 个标准化任务的测试集（每任务含需求描述 + BRP 可断言的验收清单）；「一次通过」= AI 首次提交即过 `cargo check` 且 demo 首次运行满足验收清单，期间无人工改代码；一次通过率 > 70%（N=10 时即 ≥ 8/10，样本量限制需在结论中注明）。任务测试集本身入库成为资产 |
| M2 闭环验证 | AI 能操控运行中的游戏 | ①bevy_brp_mcp 对 0.19 兼容性验证通过（不兼容则完成修补/fork）；②由单条脚本或单个 agent 会话完整执行并留存日志与截图（此即「全程无人工点击」的可核查定义）：spawn 实体 → 改组件 → 截图 → 读回状态断言——截图在自研 `screenshot` RPC 落地前允许 OS 级窗口截屏，其余步骤必须经 BRP/MCP 完成；③`tooling/` 至少 `run_tests` + `screenshot` 可用，并以 ≥ 1 个任务的验收清单跑通一次全自动判定 |
| M3 沉淀复用 | 两层知识资产成型且可证明增值 | ①方法论层 + Bevy 层结构落地（见 §6）；②Bevy 层错题本 ≥ 50 条、模式库 ≥ 20 条，全部过验证门禁（§5.2）；③任务台账连续记录（类型 / 一次通过 / 返工次数 / 耗时 / 原因）；④从台账中返工次数 ≥ 1 的任务（含 M1-M3 期间；M1 失败任务以 M1 验收时点的首次结果快照为准，且 M3 ④ 取样前不得重做）抽取 ≥ 3 个重跑（不足 3 个则如实记录实际数量），返工率显著下降（量化口径以台账实测定标） |
| M4 试金石 | 用这套工作流交付一款按选品标准选定的 PC 游戏 | ①按 §8 选品标准完成选品备忘录（含：可玩性定义、体量上限、占位美术/音效来源策略）；②「可玩交付」= 核心循环闭环 + 无 P0 崩溃 + 核心任务回归全绿；③开发全程台账无断档，游戏核心系统以任务形式补入测试集——M4 本身就是对工作流的终极测试 |

**里程碑执行规则：**
- §10 的序号是**依赖序而非日历序**，M2 的准备工作可与 M1 并行（如 BRP 通路验证、桥兼容性实测）。
- **升级窗口（0.20 起）的完成判定**：任务测试集回归通过率恢复到升级前基线 + doctest 全绿 + 迁移成本（工时/返工数）记录完毕，三者齐备才算窗口关闭。
- **未达标处置**：任何里程碑验收不过禁止默认放行——须 owner 明示裁决：返工 / 降标准（留痕记入变更记录） / 终止。里程碑的日历时间盒待 owner 终审时设定。

## 5. 范围：三条工作流

### 5.1 工作流一：AI 写文档（知识生产）
- 文档的每段示例代码以 doctest 形式验证：`docs/` 的 Markdown 经 `include_str!` 薄封装纳入 crate 文档，`cargo test --doc` 通过才可合并（编译器当裁判，过滤 AI 幻觉）。需要打开窗口/长运行的示例标 `no_run` 并注明理由；禁止无理由的 `ignore`。
- 文档一律标注适用的 Bevy 版本号；版本升级时由 AI 对照官方 migration guide 批量修订，修订后重新过 doctest 验证。
- 产出物沉淀在 `docs/` 下，与 skill 共享同一套代码模式库。

### 5.2 工作流二：AI 写代码（受 skill 约束）
- 维护 `bevy-dev` skill（Bevy 特定层）+ 方法论层 SOP，核心内容：版本纪律、架构约定、代码模式库（只收录本项目编译通过的真实代码）、错题本、验证流程。
- 纪律：AI 禁止凭训练记忆写 Bevy API，不确定必须查 docs.rs 对应版本或官方 examples；每次代码变更必须过 `cargo check`，涉及运行时行为必须走工作流三验证。
- 资产入库门禁（Voyager 原则，按条目类型分型）：
  - **模式条目**（正例）：代码过编译 + 运行验证后才可入库；
  - **错题条目**（反例）：失败须在标注版本上复现（附最小复现）+ 修复方案过编译与运行验证后才可入库；条目内的反例代码统一用 ```` ```compile_fail ```` 标记（语义不符时用 `ignore` 并附理由）。
  - **分型口径细化注记（2026-09-28，M3 实践）**：反例载体为非代码时（wire 协议请求、运行时行为观察——无编译面可断言），错题条目以 `text` 围栏收录原始请求与报错原文并锚定证据文件（先例 `bevy-dev/pitfalls.md` PIT-B-051：BRP `world.mutate_components` 对不存在实体裸 panic），不适用 `compile_fail`/`ignore`——本条为上述「语义不符时用 `ignore`」口径的运行时载体扩展。围栏约定全表见 `bevy-dev/pitfalls.md` 文件头（M3 起错题本纳入 doctest 机器断言）。
- 每次踩坑在**同一任务收尾前、最迟下次会话开始前**回写错题本，并标注通用性分级（引擎特定 / 方法论级），分级决定条目落入哪层（§6）。

### 5.3 工作流三：AI 操控 Bevy（验证闭环）
- 技术底座：官方 `bevy_remote`（BRP）常驻游戏（`RemotePlugin` + `RemoteHttpPlugin`）+ **通用桥复用 bevy_brp_mcp / bevy_brp workspace**（已定案，不自建通用桥；fork 仅在其实测不满足时触发）。**例外（v0.3.2，2026-09-29 grill 定案）：自有 `game.*` 方法的窄白名单 MCP 薄转发（stdio→BRP，工具面白名单窄且可枚举、不含任意 RPC 载荷与写通道）属允许的自研形态**——自研件 `tooling/mcp-bridge`（T041）：复用 BRP 传输而非重造，不是通用桥。
- **安全约束：BRP 仅监听回环地址（127.0.0.1），禁止绑定 0.0.0.0 或局域网地址**（BRP 无鉴权，暴露即等于交出游戏操控权）；远程调试需求如出现，单独立项评估。
- 分三步走：
  1. 人工用 curl/脚本直调 BRP 内置方法（`world.query` / `world.spawn_entity` / `world.mutate_components` / `registry.schema`），验证通路；
  2. 接入 bevy_brp_mcp，验证其对 0.19 的兼容性（M2 首个任务；不兼容则修补/fork）；截图暂用 OS 级窗口截屏过渡；
  3. 自研游戏专属 RPC（`RemotePlugin::with_method` 注册 `launch_level` / `run_tests` / `screenshot`）——这是自研件的核心（M2 验收③）：`run_tests` 按任务测试集的验收清单逐条断言，设计吸收「状态注入优先、截图作辅助证据」（§3 学界结论）。
- Rust Hotpatching（0.17+，基于 subsecond）现状限制：仅支持热补 ECS system 函数，实验特性，Windows 成熟度未经验证。M1 期间独立冒烟：通过则纳入「改逻辑不重启」流程；不通过则回退「重启进程验证」（桥上有进程管理，重启成本可控）。
  - **冒烟结论（2026-09-26）：通过，纳入「改逻辑不重启」流程**。bevy 0.19.1 机制经本地源码核实延续 0.17 路线（`bevy/hotpatching` feature → bevy_ecs 引入 subsecond 0.7.0-rc.0；DefaultPlugins 在该 feature 下自动附带 HotPatchPlugin），运行方式 `dx serve --hot-patch`（dioxus-cli 0.7.10）。Windows 10 19044 实测：两次热补丁（字符串常量变更 / 控制流 `%30→%10` 变更）各约 1.1s 生效，frame 计数全程连续单调（749460→749490、1194210→1398200+，进程未重启）而输出行为已变更。适用边界沿用上述限制：仅 system 函数体内的改动；结构变更（新增/删除 system、非 system 代码、依赖变更）仍走重启。证据：`tooling/hotpatch-smoke/`（冒烟 crate + evidence/hotpatch-smoke-run.log，台账 T005）。

## 6. 仓库结构与知识资产分层（已定案）

```
repo/
├── game/                  # 锁版本的 Bevy 游戏工程（试金石）
├── docs/                  # 工作流一的产出（经 include_str! 纳入 doctest 门禁）
├── assets-methodology/    # ★ 方法论层（引擎无关，任何"编译器+运行时反馈"栈可带走）
│   ├── sop.md             # 三条工作流操作手册
│   ├── task-ledger.md     # 任务台账 schema 与记录
│   ├── taskset/           # 标准化任务测试集（需求+验收断言模板）
│   ├── pitfalls.md        # 方法论级错题条目（引擎无关的教训落点）
│   ├── patterns.md        # 方法论级模式条目落点
│   └── pitfalls-schema.md # 错题条目 schema（含通用性分级字段）
├── bevy-dev/              # ★ Bevy 特定层（版本绑定，随升级窗口整体迁移）
│   ├── SKILL.md           # 版本纪律、架构约定
│   ├── patterns/          # 模式库（只收编译+运行验证过的真实代码）
│   └── pitfalls.md        # Bevy/版本特定错题本
└── tooling/               # 自研件：游戏专属 RPC（launch_level/run_tests/screenshot）等
```

- 技术硬约束：`Cargo.toml` 中 `bevy = "0.19"`（0.x 语义下等价于锁定 0.19.x 系列），`Cargo.lock` 入库；版本号变更仅允许发生在升级窗口。语言为 Rust（Bevy MSRV 及以上的最新 stable）；主力 agent 为 ZCode / Claude Code / Cursor 任一，MCP 作为工具接入协议；平台仅 PC（Windows）。
- 仓库纪律：**行为、接口、版本相关的变更必须同步更新 docs/ 或模式库**；纯重构、格式化、补丁版本升级豁免文档同步义务（防止为合规制造垃圾文档）。
- 分层纪律：一条教训落入哪层，由「换引擎后是否仍成立」判定；两层都不掺私货（密钥/机器路径/内部黑话），**保持随时可公开态**（是否真公开延至 M3 后决策，已定案）。
- 版本升级（0.20 起）是 Bevy 特定层的例行迁移演练：migration guide 批量修订 + 全量重编译 + 任务测试集回归重跑，迁移成本以台账量化——这本身就是方法论层数据。

## 7. 风险与对策

| 风险 | 等级 | 对策 |
|---|---|---|
| Bevy 每 3-5 个月破坏性更新，特定层资产集体过时 | 高 | 锁版本；升级视为独立里程碑；方法论层/特定层分离让主产出保值；迁移成本经任务测试集回归量化 |
| AI 幻觉（编造 API） | 高 | 编译器门禁 + doctest + BRP `registry.schema` 运行时自查 + 错题本前置注入 |
| **资产不增值**（M3 增值证明失败：错题本/模式库未带来返工率下降） | 高 | 以任务台账定位失效环节（条目不准？检索不到？注入时机不对？），调整资产形态或注入策略；仍无效则如实记录负结果，并修订 §1 的成败标尺表述——负结果本身也是方法论层资产 |
| bevy_brp_mcp 停滞或版本兼容滞后 | 中 | M2 实测兼容性；不兼容即修补/fork——届时自建需求已明确，沉没成本最低；触发 fork 即承担其长期维护与每个升级窗口的同步成本（计入台账） |
| 无编辑器导致关卡/场景制作效率低 | 中 | `bsn!` 宏（场景即代码）+ 旧 `.scn.ron` + 程序化生成优先；`.bsn` 落地后迁移；中期可尝试 Jackdaw |
| Rust Hotpatching 实验特性（仅 system、Windows 成熟度未知） | 中 | M1 先独立冒烟；失败即回退「重启进程验证」，不阻塞主流程 |
| 游戏试金石选错品类（自动验证闭环吃不上力） | 中 | 按 §8 选品标准执行：离散回合制、确定性模拟、BRP 可断言；避开实时动作/物理手感与强叙事演出 |
| 社区 AI 政策不确定（no-AI 正被重写） | 低 | 不依赖上游生态；独立仓库自建；向上游贡献时如实披露并遵守届时政策 |

## 8. 游戏选品标准（试金石逻辑，已定案：定标准不锁品类）

选品逻辑：**什么品类最能锻炼并验证这套工作流**，而非最想玩什么。硬标准：

1. 离散/回合制逻辑优先，确定性模拟（可种子重放）。
2. 游戏状态可经 BRP 查询并断言（闭环吃得动）；验收以状态断言为主、截图为辅。
3. 避开：实时动作 / 物理手感调优（人工手感验收占比高）、强叙事演出（人工审美验收占比高）。
4. 中小体量——它是试金石，不是产品野心。
5. 达标候选（示例，非封闭清单）：回合制策略、roguelike、解谜、离散 tick 模拟经营。

M2 闭环跑通后、动工 M4 前，从达标品类中选定并写选品备忘录。备忘录必须固化三件事：**可玩性定义**（M4 验收②的展开）、**体量上限**（周数或系统数，防失控）、**占位美术/音效来源策略**（本工作流不覆盖美术生产，须提前定来源）。

## 9. 明确不做的事（当前版本）

- 不做移动端——不适配、不打包、不设评估里程碑；未来若想法变化按新立项处理。
- 不自建通用 MCP 桥（通用面复用 bevy_brp_mcp / bevy_brp workspace；fork 仅在其实测不满足时触发）。**边界（v0.3.2，2026-09-29 grill 定案）：自有 `game.*` 方法的窄白名单 MCP 薄转发属允许的自研形态**（见 §5.3 修订与 `docs/pre-window-plan.md` T041）——白名单窄且可枚举、无任意 RPC 载荷与文件/shell 写通道。
- 不等官方编辑器，不做编辑器二次开发。
- 不做网络多人功能（除非选品强制要求，届时单独立项评审）。
- 不向 Bevy 上游承诺维护职责。
- **已决策开源（2026-09-29，owner）**：GitHub 公开仓 `bevy-ai-workflow`，MIT OR Apache-2.0 双许可，全历史公开；「保持可公开态」纪律持续有效（两层资产不掺私货）。开源准备与发布 = 台账 T039。

## 10. 下一步（批准后立即执行；序号为建议次序，前置/并行关系以各条内注为准）

1. 建仓库骨架：按 §6 两层结构初始化（`game/` 锁 0.19 + `assets-methodology/` + `bevy-dev/` + `docs/` + `tooling/`）。
2. 任务台账启用：骨架就绪后立即生效，从第 1 个任务起记录（类型 / 一次通过 / 返工次数 / 耗时 / 原因）——这是 M3 增值证明与升级窗口工时量化的数据源头，任何后续任务开始前不可缺位。
3. 写 `bevy-dev/SKILL.md` v0：版本纪律 + 查证流程（禁凭记忆写 API）+ 架构约定——M1 第一役的前置，冷启动不能是空 skill。
4. BRP 通路验证（可与第 5 项并行）：curl 直调内置方法确认通路 → 接入 bevy_brp_mcp 实测 0.19 兼容性（M2 首战）。
5. M1 第一役：AI 按 skill 约束写「相机漫游 + N 个动态实体」demo（N 从 1,000 阶梯升至 50,000），首批 ≥ 10 个标准化任务进测试集。任务判定两阶段机制：M1 期间由人工对照验收清单判定并留存运行证据（日志/截图）；BRP 与 `run_tests` 就绪后将全部任务的验收清单转为脚本断言并**重跑校准**旧数据，统一口径。
   帧率基线采集口径（缺任一项则数据不可比，禁止省略）：release 构建（或专用 bench profile）；固定分辨率、窗口模式、vsync 设置；记录 CPU/GPU/驱动版本；demo 场景固定随机种子。基线数据与口径说明同文入库。
6. Hotpatching 冒烟（M1 期间完成，可与第 5 项并行）：subsecond 在 Windows/本项目结构下独立验证，结论回写 §5.3。

---

## 变更记录

- **v0.3.3（2026-09-29，开源决策 + 任务编号勘误）**：①§9「当前不决策开源与否」改写为**已决策开源**（owner 裁决四项参数：GitHub 公开仓 `bevy-ai-workflow` / MIT OR Apache-2.0 双许可 / 全历史公开 / 外部调研件不收编）：新增 LICENSE + LICENSE-MIT + LICENSE-APACHE + 根 README，开源准备与发布 = 台账 T039（含可公开态终审：受管文件零机器路径/零敏感信息、git 全历史统一 bot 身份）；②任务编号勘误——开源任务插队 T039，`game.snapshot` T039→**T040**、MCP 薄桥 T040→**T041**（§5.3/§9 与 `docs/pre-window-plan.md` 内引用同步）；③v0.3.2 entry 中外部调研件的本机路径引用按「不收编」定案改为「未入库」措辞。
- **v0.3.2（2026-09-29，同类项目调研增补 + 桥政策边界修订，七项 grill 定案）**：①§2.1「Bevy 侧仅有的两个早期 MCP」失实表述改写（2026-09-28/29 四项目实测：≥3 条 BRP 桥 + 非 BRP MCP 面，全部早期/低星（≤72★）/单作者）；②§3 表格以 bevy_brp workspace 行**继任合并**原 bevy_brp_mcp 行（含 T004 兼容实测结论与许可标注）并增补 6 行（bevy_mcp / Plinth / SpawnForge / Axiom / bevy-agent / Jackdaw），结论句追加规模更新注记（「桥不稀缺，闭环 + 知识资产才稀缺」后半句保留并加强）；③§5.3/§9 同步修订：自有 `game.*` 方法的窄白名单 MCP 薄转发列为允许的自研形态（解锁 MCP 薄桥任务 `tooling/mcp-bridge`），通用桥复用对象更新为 bevy_brp_mcp / bevy_brp workspace；④关键事实信源增补。同批 grill 七项定案（详见 `docs/pre-window-plan.md`）：A 级三项批准立项（台账标「窗口前置增强」，不立 M5）；B4 精确 tick 步进维持记录项；B1 端口收口不做记档；优化方案归档 docs/ 挂 doctest 门禁。依据：外部调研件《Bevy-AI同类项目调研报告》（未入库，2026-09-28/29，公开面核读 + 独立复核 + plan-code-reviewer 终审 5 必改全落实；v0.3.3 勘误：原文本机路径引用已按「不收编」定案改为此措辞）。
- **2026-09-28（M3 §5.2 分型口径细化注记）**：M3 Block H 净室重跑产出运行时行为型错题（PIT-B-051：BRP `world.mutate_components` 对不存在实体裸 panic 击穿进程——反例载体为 wire 请求与 panic 栈，非代码），§5.2「资产入库门禁（Voyager 原则，按条目类型分型）」追加带日期注记（原文未动）：非代码载体错题以 `text` 围栏 + 证据锚定入库。M3 验收结论见 `docs/m3-acceptance.md`。
- **2026-09-26（Hotpatching 冒烟结论回写，§10 第 6 条）**：冒烟通过，§5.3 Hotpatching 条目末尾追加带日期结论注记（原文未动）；新增 `tooling/hotpatch-smoke/`（最小冒烟 crate + 运行证据）；修复根 Cargo.toml 的 `tooling/*` glob 成员误吸入 `tooling/brp-logs`（无 Cargo.toml 的证据目录）导致 workspace 解析失败的问题（加 `exclude`）。
- **v0.3.1（2026-09-26）**：plan-code-reviewer 审核（裁决：有条件通过）后修订，6 项必改全部落实：B1 M1「一次通过」定义与两阶段判定机制、§10 依赖序修正；B2 M3 ④ 样本池改为「台账返工 ≥1 任务」并规定 M1 失败快照不可重做；B3 M2 截图口径与「无人工点击」可核查定义；B4 自研件 run_tests/screenshot 纳入 M2 验收③；B5 错题/模式门禁分型（反例 `compile_fail` 标记）+ 方法论级条目落点（assets-methodology/pitfalls.md、patterns.md）；B6 帧率基线采集口径。同时采纳审核建议：S1 版本锁写法更正（`"0.19"` + Cargo.lock，`=` 精确锁不适用于 x 范围）；S2 新增「资产不增值」风险与未达标处置规则、升级窗口完成判定；S3 M4 可玩性定义与选品备忘录固化项；S4 doctest 机制指定（include_str! 薄封装、no_run 规范）；S5 「24 小时回写」改为可执行的会话口径；S6 PR 文档纪律收窄（豁免纯重构/格式化）；S7 BRP 仅监听回环地址；G1 补 SKILL.md v0 冷启动步骤；G2 里程碑执行规则（依赖序/并行/时间盒待定）；G3 游戏核心系统入测试集；G4 变更记录补注 v0.2 的 M5 移动端评估里程碑已随移动端移除而删除；G5 统一「知识资产/自研件/工作流」用词口径。复核（裁决：通过）后处理两条非阻断备注：台账 schema 补「耗时」字段（升级窗口的工时量化有出处）；台账启用上移至骨架后首位、序号措辞改为「建议次序，前置/并行以各条内注为准」、Hotpatching 冒烟标注可并行。
- **v0.3（2026-09-26）**：目的重定调（owner 决策）——主产出 = 工作流 + 两层知识资产，游戏降为试金石，移动端全部移除（v0.2 曾补设的 M5 移动端评估里程碑一并删除）。新增 §3 调研对照（bevy_brp_mcp/bevy_debugger_mcp/Godot-Unity 阵营/Voyager/GameLogicBench 等）。五项决策经 grill 定案落地：①M2 复用 bevy_brp_mcp，不自建通用桥，游戏专属 RPC 自研；②知识资产两层结构（§6）；③游戏定选品标准不锁品类（§8）；④私有起步保持可公开态，开源延后 M3 再议；⑤立即 0.19 起步，0.20 即首个升级窗口。里程碑重构：任务测试集入库为资产、迁移成本量化、M4 改为试金石定位。吸收 Voyager「验证过才入库」纪律与学界「状态注入 > 纯截图」验证结论。
- **v0.2（2026-09-26）**：事实核查修订——①BSN 论据更正（0.19 仅 `bsn!` 宏，`.bsn` 未发布）；②版本归因更正（GPU-driven=0.16、Hotpatching=0.17）；③性能数字替换为官方 Caldera 基准；④补齐 M5；⑤成功标准量化；⑥工作流三截图落地顺序修正；⑦Hotpatching 限制与回退；⑧决策点 0；⑨no-AI 政策更新；⑩doctest 门禁；⑪工具清单加入 ZCode。
- **v0.1（2026-09-26）**：初稿。

## 关键事实信源

- Bevy 0.19 发布文（2026-06-19，BSN 仅宏形态）：<https://bevy.org/news/bevy-0-19>
- docs.rs `bevy::scene`（.bsn 格式 not yet released）：<https://docs.rs/bevy/latest/bevy/scene/index.html>
- Bevy 0.17 发布文（Rust Hotpatching / subsecond / 仅 system）：<https://bevy.org/news/bevy-0-17>
- Bevy 0.16 发布文（GPU-driven / Caldera 基准数字）：<https://bevy.org/news/bevy-0-16>
- docs.rs `bevy_remote`（BRP 方法集 `world.*` / `registry.schema` / `with_method`）：<https://docs.rs/bevy_remote/latest/bevy_remote/>
- Bevy 六周年博文（Jackdaw / no-AI 政策重写中 / .bsn 即将入 main）：<https://bevy.org/news/bevys-sixth-birthday>
- bevy_brp_mcp（BRP 的 MCP 封装，本项目通用桥复用对象）：<https://mcpservers.org/servers/natepiano/bevy_brp_mcp>
- bevy_brp workspace（bevy_brp_mcp 2026-09 归档迁入处；license 未检出，引代码前须核）：<https://github.com/natepiano/bevy_brp>
- 同类项目实测四项（2026-09-28/29 调研，公开面核读 + 独立复核）：Plinth <https://github.com/Luminary-Analytics/plinth> · SpawnForge <https://github.com/Tristan578/project-forge> · Axiom <https://github.com/cats2333/bevy_ai_editor> · bevy-agent <https://github.com/jbuehler23/bevy-agent>
- bevy_debugger_mcp（备选参照）：<https://github.com/ladvien/bevy_debugger_mcp>
- bevy 官方讨论：游戏内可插拔 MCP 提案：<https://github.com/bevyengine/bevy/discussions/20042>
- Voyager（技能库范式）：<https://arxiv.org/abs/2305.16291>
- GameLogicBench（任务基准化 / 每 tick 断言）：<https://arxiv.org/html/2609.21562>

---

*本文档由 AI 协助起草，人类 owner 审批后生效；后续重大方向变更需更新版本号并注明变更原因。*

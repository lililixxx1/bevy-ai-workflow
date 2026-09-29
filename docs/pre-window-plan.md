# 窗口前置增强方案（定案版）

- 适用 Bevy 版本：**0.19**（本方案三项任务全部落在 0.19 上；版本升级时对照官方 migration guide 批量修订后重过 doctest）。
- 状态：**owner 已批准**（2026-09-29，grill 七项定案）；T038 立项批次执行中。
- 依据：外部调研报告《Bevy-AI同类项目调研报告》（`q-novel/`，2026-09-28/29，四项目公开面核读 + 独立复核 + plan-code-reviewer 终审 5 必改全落实）；意向文档 v0.3.2（本次同批修订）。
- 定位：M4 收官（T037，344d002）之后、0.20 升级窗口开启之前的**窗口前置增强（pre-window）**序列——不立新里程碑（意向文档无 M5 定义，grill 定案不设立），台账类型标「窗口前置增强」。

## 一、七项定案（grill，2026-09-29）

| # | 决策点 | 定案 |
|---|---|---|
| 1 | 批准范围 | A 级三项全批：T038 计数门禁 → T039 `game.snapshot` → T040 MCP 薄桥 |
| 2 | 里程碑归属 | 不立 M5，台账标「窗口前置增强（pre-window）」序列 |
| 3 | 意向文档 §9/§5.3 | 授权修订（合并为一次修订、一个变更记录 entry，即 v0.3.2），T040 前置解除 |
| 4 | §2.1/§3 增补幅度 | 按 v2 全量：7 行增补 + `bevy_brp_mcp` 行继任合并 + 结论句保留后半句 |
| 5 | B4 精确 tick 步进（原 A3） | **维持记录项**：窗口前不立项；0.20 迁移期间确需 tick 粒度定位时再按路径①评估（基线重采列为显式成本） |
| 6 | B1 端口收口 | **不做，记档**：硬约束 4（仅回环）已满足；将来分发或共享机器运行时重开 |
| 7 | 方案归档 | 归档 `docs/`（本文件）并纳入 doctest 门禁 |

## 二、A 级三项（已批准立项）

### T038 A2：计数与宣称的机器门禁（claim-lint）

- **动因**：台账实测「计数/口径」是审核返工第一大来源——T002（内置方法清单漏 2 条）、T004（rpc.discover 方法数笔误 27→23）、T020（断言计数 77→78，P1-1 必改）、T023（引文/口径）、T029（汇总表误植）、T034（注册表 13→12）、T037（26 份门禁日志零份自含 REAL_EXIT，终审必改）；外部教训同源（SpawnForge 380 命令注册 vs 2 proven、营销写 350）。
- **落点**：`tooling/claim-lint/`（**crate 成员**，随 `tooling/*` glob 入 workspace——规避 PIT-M-003 非 crate 目录复发；零第三方依赖，纯 std 文本处理）。
- **claims 表（首批 9 条真值断言）**：PIT-B 条数（`bevy-dev/pitfalls.md` `^### PIT-B-0` 标题计数 −1，模板行 `PIT-B-00X`）；PIT-M 条数（`assets-methodology/pitfalls.md` `^### PIT-M-` 计数，无模板行）；PAT-B 文件数；taskset `ts-*.md` 文件数；台账任务行数（行首 `| T0##`）；`game.*` 方法数（`game/src/rpc/mod.rs` `pub const …_METHOD` 锚定）；in-process 套件数（`game/src/rpc/suites/mod.rs` `all()` 注册表 `("ts-` 计数）；BRP 方法总数（23 内置 + game.\* 静态推算）；doctest 门禁计数（运行时真值，取自门禁日志，日志缺失时 SKIP 并注明）。
- **一致性断言**：受管文件自述头（两份 pitfalls 的「当前：N 条」、patterns/README）必须与真值相等；行内 `<!-- claim-lint:ignore 理由 -->` 豁免历史性数字，豁免清单本身入审核面。
- **联动规则**：每新增 `game.*` 方法同步 claims 表（T039 落地 30→31；B4 若将来落地 31→32）。
- **验收**：全绿 + 负控自证（临时改一处自述数字必须红，留痕后复原）+ 门禁登记（SKILL §4.1 / AGENTS.md 常用命令 / tooling/README.md）。

### T039 A4：稳定 id 快照 `game.snapshot`

- **动因**：T032/T034/T035/T036 反复出现「实体号变化属预期、用业务键绕行」（PIT-B-010 迭代序纪律）——固化为一次性契约。
- **设计底座**：收编 T035 已验证的驱动侧重放快照字段集（业务键布阵 + HP + 行动标记 + winner/phase + rng_state）为正式 RPC 方法。键 = `GridPos(x,y)`（`game/src/battle.rs` 按序/寻址）+ `Wanderer::index`（`game/src/sim.rs`，确定性口径锚点）；~~UnitId~~（全仓 0 命中，系调研报告失实，已在复核中剔除）。
- **字段来源**：phase/winner/回合数 ∈ `BattleState`；paused ∈ `SimConfig`——实现时写明各字段来源资源，防混编。
- **精度口径**：`rng_state`（u64）在任何 JS 工具端沿 PIT-M-009 处理（字符串化或排除出严格相等；沿 TS-16「f64 渲染分辨率下一致」限定先例，不得倒退）。
- **验收**：`rpc.discover` 在列（30→31，claim-lint 同步）；新套件断言与逐字段 `world.query` 等价、与 T035 字段集一致；同 seed 跨进程两次启动快照逐字节一致（实体号排除口径不变）；既有 12 套件 + driver 级回归全绿、0 panic。

### T040 A1：MCP 薄桥（stdio→BRP 转发）

- **前提**：T004 实测 `bevy_brp_mcp 0.22.7 ↔ bevy 0.19.1` 协议级兼容（32 次 tools/call：19 成功、13 失败无一为协议不兼容）——缺口不是可行性是「没接」。意向文档 v0.3.2 已修订 §5.3/§9 放开「自有方法窄白名单薄转发」。
- **落点**：`tooling/mcp-bridge/`（crate 成员，**不链 bevy**，SKILL §3.6；依赖面沿 task-runner 口径：仅 serde_json + std TCP）。
- **形态**：stdio MCP（`initialize`/`ping`/`tools/list`/`tools/call`，capabilities `{"tools":{}}`，`protocolVersion` 回显），每工具 = 一次到 `127.0.0.1:15702` 的 HTTP POST；**不新增监听端口**（仅出站连接，硬约束 4 无衰减）；工具面白名单：7 个 `game.*` + 只读 BRP 子集（`rpc.discover` / `world.query` / `world.get_components` / `registry.schema`）——**排除 `world.mutate_components`**（PIT-B-051 进程击穿面）等写通道。MCP stdio 帧格式以规范与 Plinth `mcp.rs` 参照核实，禁凭记忆（SKILL §2）。
- **判定基准（机器面）**：由 MCP agent 驱动，判定 = `game.run_tests` 返回的 pass/fail 字段 + taskset 断言清单逐条 + 错误码原文——agent 只作驱动，不作判定。
- **验收**：全链路会话留痕（沿 T004 transcript 先例）；tools/list 输出与白名单清单**逐项相等**；错误码透传干净（`-32602` 原样到达）；依赖面零新增。

## 三、B/C/D 级处置

- **B1 端口收口：不做，记档**（定案 6）——现状 dev/release 均开回环 15702 已满足硬约束 4；改启动口径的扰动（TS-11 release bench 脚本改造 + 口径勘误）大于单用户 PC 上的边际安全增益；重开条件：游戏分发或共享机器运行。
- **B2 机器可读迁移待重核清单：0.20 窗口开启时立项（T042）**——51 条 PIT-B + 20 条 PAT-B + SKILL §6.1–6.14 转结构化核对表（条目 id / 验证方式 / 上版结论 / 期望结果），`docs/` 文档挂 doctest 门禁（沿 `m3-plan.md` 先例）；直接服务意向文档 §4 升级窗口完成判定三件套。A2/A4/A1 落地后此清单回归能力更强。
- **B3 关卡数据文件先验：开题时取用，零动作**——Axiom 的坑（BRP spawn `SceneRoot` 受 `Handle` 反射限制；解法 = 资产 base64 内联 + 游戏侧去重落盘）与 Plinth 的方案（`*.scene.json` schema + validator 毫秒级诊断，刻意把 agent 挡在 Rust 编译外）；将来 NL→关卡让 LLM 输出本项目自己的关卡 schema，不给自由代码。
- **B4 精确 tick 步进：维持记录项**（定案 5）——终审实证：仓库无 `FixedUpdate`/`Time<Fixed>`/`Time<Virtual>` 承接面，调度仅 `Startup`+`Update`，`SimStats.tick` 为「每未暂停帧 +1」非定步，暂停是 `run_if` 门；战斗域（TS-16 的 78 步重放）无任何 `Time` 依赖，「重放到 tick 粒度」对战斗域不成立。路径①（引入定步骨架）牵动 TS-01 #3 / TS-02 / TS-07 / TS-11 / TS-16 四组已验证基线重采；路径②（仅 sim 域 advance 钩子）零结构改动但非 Plinth 形态。0.20 迁移期间确需 tick 粒度定位时再评估，基线重采列为显式成本。
- **C 级（仅记录）**：C1 输入注入非缺口（规则面 RPC 验证严格优于；表现层验收时再评估且须新增通道）；C2 `.mcp.json` 注册形态 T040 落地时采纳；C3 组件代理通道（Reflect 组件 + `Added<T>`）作 0.20 自定义方法签名变更时的逃生通道预案；C4 生成文件 md5 入台账边际价值，需要时补。
- **D 级（实施期红线）**：D1 不引入绕过门禁的 AI 直写通道；D2 版本号只在根 `Cargo.toml`（A1 文案不得写死 0.19）；D3 工具面不为命令数好看扩面；D4 未过验证不入库（A2 是其机制化）；D5 不做编辑器；D6 桥不同时给任意 RPC 载荷与文件/shell 写；D7 四项目代码一律不作来源（SpawnForge BSL 1.1 non-production、2030-02-11 转 Apache-2.0；Axiom 无 LICENSE 文件；bevy-agent 仅 MIT 文件；Plinth 双许可），`bevy_brp`/`bevy_mcp` 仓库 license 未检出（2026-09-29 在线亲验），引代码前须核许可。

## 四、执行序与时序红线

```text
T038 A2 claim-lint（首个用例 = 意向文档 v0.3.2 与本方案的口径一致性）
  → T039 A4 game.snapshot（收编 T035 快照字段集）
    → T040 A1 MCP 薄桥（工具面收录 game.snapshot）
      → 0.20 窗口开启：T042 B2 待重核清单（A2/A4/A1 全部转为窗口回归工具）
```

- **窗口时序预案**：0.20 发布是外部事件（0.19 发布于 2026-06-19）。**窗口开启即冻结未完成的 A 级任务**——先迁移后恢复；已落地项转为窗口回归工具反哺迁移。
- 每任务沿 M4 既定流程：立项 taskset 断言清单 → 实现 → `cargo check` 0 警告 → 套件绿 → 0 panic → 证据留痕（自含 `REAL_EXIT` 行）→ 台账两阶段判定 → plan-code-reviewer 审核。
- 若 0.20 在 A 级完成前发布：冻结、先迁移、后恢复（本方案三项全部锚定 0.19 语义）。

## 五、文书增量（随任务落地）

1. 意向文档 v0.3.2（**已完成**，commit c34eb89）：§2.1/§3/§5.3/§9 + 变更记录 + 信源。
2. 本方案归档 `docs/` + doctest（**本文件**）。
3. T038：claim-lint 登记 SKILL §4.1 门禁清单、AGENTS.md 常用命令、`tooling/README.md`。
4. T039/T040 落地后：SKILL §6 事实速查新增条目（带本地源码行号 + 核实日期），变更记录升版。
5. PAT-M「宣称-实证对照」「计数机器门禁」两条：**必须等 A2 实际执行留痕后才可入库**（`pitfalls-schema.md` 过程型门禁：≥1 任务实际执行并留痕）——之前只在台账「原因」列记「待验证教训」。
6. 台账 T038–T040（及窗口期 T042）按现有体例记录，返工如实计。

## 六、审核记录

| 轮次 | 裁决 | 说明 |
|---|---|---|
| v1 终审（plan-code-reviewer，2026-09-29） | 有条件通过（5 必改 + 8 建议 + 8 备注） | 5B 全落实后出定案版：B1 UnitId 失实剔除、B2 tick 步进降级 B4、B3 §9/§5.3 走修订、B4 §2.1 定位纠正、B5 claim-lint crate 形态 |
| owner grill（2026-09-29） | 七项定案（见 §一） | 本文件即定案产物 |

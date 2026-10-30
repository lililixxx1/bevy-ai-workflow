# T052-W1 · M6 塔防 m6-all 收官判定 · 证据总账

- 任务：T052-W1（M6 塔防 m6-all 终局判定卡 = **收官节点卡**；grill 定案② 四件套 + 登记项七条）
- 树：`<tree>` = `trees/m6/w6`（基线 = `<main-repo>` `d6d7609` 的 git archive 纯净导出；无 `.git`，物理隔离）
- 日期：2026-10-31
- 判定面脚本：`docs/evidence/t052-m6-all/m6-all.mjs`（独立可跑；agent 只作驱动，判定行与总判定由脚本计算生成）
- 轮次说明：首轮判定面（run1–run3 + p0probe-run1）后经 **G1 U1 裁 (a)** 加固 `setup_game_over`，返工轮 = **run4 + p0probe-run2 + 四闸 b 档**；本档为两轮合并总账（失败轮/首轮档一律保留）。

## 〇、终局结论（先说结论）

| 件 | 判定 | 依据 |
| --- | --- | --- |
| 件一 · m6-all BRP 判定面全绿 | **PASS** | 终轮 `m6-all-run4.log`：**PASS=483 / FAIL=0 / SKIP=2 / TOTAL=485，REAL_EXIT=0**（SKIP = 可选登记项⑥ 溅射聚怪签名，见 §四）；首轮 `run3` 同值同判定 |
| 件二 · 26 部行为等价对照全绿 | **PASS** | `w2-final-check.md`（26 部逐部核对表；148 行分类机器统计与底稿/宣告三方对齐） |
| 件三 · 无 P0 崩溃 | **PASS**（G1 U1 裁 (a) 修复+复验后） | 首轮负控 `p0probe-run1.log` 曾命中 1 处击穿（`td.state set GameOver` 缺 `GameResult` → `Res<GameResult>` 校验失败 → 进程 panic 退出，**原档保留**）；修复 = `setup_game_over` 改 `Option<Res<GameResult>>`（None → 空屏 + info 标记）→ 返工轮 `p0probe-run2.log` **9/9 PASS**（进程存活 / `td.state get`==GameOver / `result`==null / `finished`==false / 日志无 panic / skip 标记在档）；主跑 `run4` 全程无 panic |
| 件四 · 台账 / taskset 纪律无断档 | **PASS** | `discipline-check.md`（台账 T044–T051 8/8 在档；taskset 17 全绿；返工数 T047=2/T048=4/T049=2/T050=5/T051=7 与台账原文一致） |

**m6-all 总判定 = PASS（四件套 4/4）**。首轮唯一阻塞点（件三 P0）已按 G1 U1 裁 (a) 修复并独立复验；U2/U3/U4 裁决口径见 §五，返工自述见 §十一。

## 一、判定面结构与用法

```
docs/evidence/t052-m6-all/
├── m6-all.mjs              # 判定面（相位：screen / l1 / l2 / l3 / chain / p0probe / all）
├── toml-level-expand.mjs   # 独立 TOML 网格展开器（自 t051 档原样复制，未改）
├── m6-all-run4.log         # ★ 判定输出档（件一终轮：逐项 PASS/FAIL + SUMMARY + M6ALL_JUDGEMENT + REAL_EXIT）
├── m6-all-run4.game.log    # 同轮游戏进程日志（stdout+stderr；panic 扫描面）
├── p0probe-run2.log        # ★ 件三负控（返工轮：U1 修复后 9/9 PASS）
├── p0probe-run2.game.log   # 同轮游戏日志（含 skip 标记行）
├── p0probe-run1.log / p0probe-run1.game.log   # 首轮负控：命中击穿（FAIL）+ panic 原始栈（保留）
├── w2-final-check.md       # 件二
├── discipline-check.md     # 件四
├── m6-all-build.log            # 首轮 release 构建（game-td）
├── m6-all-build-run2.log       # 返工轮 release 构建（U1 修复后重建）
├── gate-check-workspace.log / gate-test-workspace.log / gate-doc-test-t052.log /
│   gate-claim-lint-t052.log / process-check.log / scope-check.log              # 首轮门禁（保留）
├── gate-check-workspace-t052b.log / gate-test-workspace-t052b.log /
│   gate-doc-test-t052b.log / gate-claim-lint-t052b.log / process-check-t052b.log /
│   scope-check-t052b.log                                                       # ★ 返工轮门禁（终态）
├── m6-all-shot-l1-level_01--run4.png / m6-all-shot-l2-level_02--run4.png /
│   m6-all-shot-l3-level_03--run4.png / m6-all-shot-chain-v-level_01--run4.png /
│   m6-all-shot-victory-run4.png                                               # ★ run4 截图（逐相位）
├── m6-all-run1.log / m6-all-run1.game.log      # 失败轮（经济体单采样竞态，见 §八）
├── m6-all-run2.log / m6-all-run2.game.log      # 中间绿档（截图同名覆盖瑕疵轮）
├── m6-all-run3.log / m6-all-run3.game.log      # 首轮判定绿档（件一）
├── m6-all-shot.png / m6-all-shot-run2.png / m6-all-shot-victory-run2.png       # 上述轮次截图（保留）
└── dev-*.log / m6-all-dev.game.log             # 分段调试轮（脚本校准，非判定轮）
```

用法（树根执行 = 脚本 CWD；游戏进程 CWD = `<tree>/game-td`；**游戏存活期禁一切 cargo**）：

```
# 1) 先构建 release exe（本卡每次换构建前先 taskkill 核零残留）
$ CARGO_TARGET_DIR=<main-repo>/target cargo build --release -p game-td -j 3
# 2) 直启 exe（BEVY_ASSET_ROOT 必设，SKILL §6.18；缺省 = 静默空图）
$ cd <tree>/game-td && BEVY_ASSET_ROOT=<tree>/game-td <main-repo>/target/release/game-td.exe \
      > <tree>/docs/evidence/t052-m6-all/m6-all-run4.game.log 2>&1 &
# 3) 等 BRP 就绪后跑判定面（run4 实测 ≈133 s：19:50:29Z → 19:52:42Z）
$ node docs/evidence/t052-m6-all/m6-all.mjs --phase all --expect-discover 41 \
      --game-log docs/evidence/t052-m6-all/m6-all-run4.game.log \
      --shot ../docs/evidence/t052-m6-all/m6-all-shot.png \
      --shot-victory ../docs/evidence/t052-m6-all/m6-all-shot-victory.png
# 4) 件三负控（独立进程，脚本不并进 all；修复后为「存活面回归」）
$ node docs/evidence/t052-m6-all/m6-all.mjs --phase p0probe \
      --game-log docs/evidence/t052-m6-all/p0probe-run2.game.log
# 5) taskkill //F //IM game-td.exe 并核零残留
```

相位说明：`screen`（方法面/发现/开局卫生/错误路径）· `l1|l2|l3`（单关：核心循环 + 双模式终局 + ENTITY_DIGEST 采样窗）· `chain`（登记项② 状态机全链）· `p0probe`（件三：带外转移存活面）· `all`（screen + l1 + l2 + l3 + chain）。

## 二、断言计数与总判定（run4，机器生成）

```
SUMMARY: phase=all PASS=483 FAIL=0 SKIP=2 TOTAL=485
M6ALL_JUDGEMENT=PASS
REAL_EXIT=0
```

| 段 | PASS | FAIL | SKIP | 面 |
| --- | --- | --- | --- | --- |
| `screen` + `input-channel` | 17 | 0 | 0 | discover 41 / 18 `td.*` 方法面 / 3 关发现 / LevelSelect 卫生 / **18 条错误路径**（-32602×17 + -32601×1）/ 进程存活 / 日志无 panic / `td.input emit` 端到端冒烟 |
| `l1[level_01]` | 113 | 0 | 1 | 核心循环 + victory（零漏）+ defeat + ENTITY_DIGEST 采样窗 |
| `l2[level_02]` | 108 | 0 | 0 | 同上（无冻结 ENTITY_DIGEST → 只做独立复算） |
| `l3[level_03]` | 109 | 0 | 1 | 同上 + rocket 弹道/弹药面（launchers=1 / ammoCap=3 / fired≥1 / projectile≥1） |
| `chain-v[level_01]` | 72 | 0 | 0 | 登记项② 第一段：victory |
| `chain`（状态机链） | 10 | 0 | 0 | 回菜单（cleanup + `GameResult` 保留契约）→ 进 level_02（全新资源 + 指纹）→ 说明项 |
| `chain-d[level_02]` | 26 | 0 | 0 | 登记项② 第二段：另一关再终局（Defeat） |
| 相位间状态迁移（leaveToMenu） | 16 | 0 | 0 | 每次 `td.state set LevelSelect` 的 from/to 断言（登记项① 逐次留痕） |
| `chain-l2` 进关 + 地图面 | 10 | 0 | 0 | level_02 逐格 TileType 对拍 + GRID_DIGEST + 无残留塔 |
| `final` | 2 | 0 | 0 | 日志全程无 panic / 进程存活 |
| **合计** | **483** | **0** | **2** | — |

件三负控（独立进程）：`p0probe-run2.log` = **PASS=9 / FAIL=0 / SKIP=0，REAL_EXIT=0**。

关键实测值（run4）：
- victory 用时：level_01 **18.875s** / level_02 **18.500s** / level_03 **18.828s** / chain 相位 **18.906s**（均 `< lastSpawn + 末怪走完全程`，**零漏**：`lives stayed 5` 四轮全 PASS）；
- defeat 用时：level_01 **6.781s** / level_02 **10.813s** / level_03 **11.250s** / chain 相位 **10.797s**（均 ≥ 第 5 只漏怪时刻；`spawned == 事件表期望` 无调度失步）；
- 经济体独立复算账（`gold == 300 + 3.0·t + 25·kills − Σcost`）共 12 个采样点全部 PASS，一致样本残差 0.09–0.14 金；一致采样在本轮触发 1 次（l3 cp1：首样本 `alive=5 > spawned=4` = **刷怪竞态**，第二次 diff=0.0938）；
  - [取证补录 2026-10-31 · G2 M2 采纳] 恒等式中 bounty 系数 `25` 为脚本侧常量，与关卡 TOML `bounty = 25` 耦合——TOML 值另有 defeat 相位独立断言（bounty==25）兜底，改 TOML 值时判定面**响亮失败**而非假绿（失败方向安全）。注记入 README 而不改 `m6-all.mjs` 本体（保持脚本版本与 run4 判定档一致）。
- `td.*` 错误路径 18/18 返回预期码且进程存活（`-32601` 锚 `bevy_remote-0.20.0/src/lib.rs:1398`；`-32602` 锚 `:1401`）。

## 三、判定面覆盖对照（派工单要求 → 落点）

| 派工单要求 | 落点（`m6-all.mjs` / 日志行） |
| --- | --- |
| 核心循环：开局状态 → 波次推进 → 经济闭环 → 塔放置 → 敌人推进与击杀 → digest | 每关 victory 轮：`opening:*`（lives 5 / 300+3t / total==事件数）→ `cp1..cp3 spawned == 独立事件表` → `economy ledger`（初始金币/被动/击杀 bounty/放置扣费四要素合成一条恒等式）→ `place rapid @[...]` + 3 条负例（占格 / 路径 / 越界）+ **余额拒绝** → `cp* towers on field` + `combat` 面 → `GRID_DIGEST == registry` + `ENTITY_DIGEST == T048 baseline` |
| victory / defeat 双模式终局（每关各一次） | 每关各一轮：没塔跑 defeat（零伤害零赏金 → 漏 5 只判负）；3 座 rapid 起步 + 赏金机会性补塔跑 victory（run4 实况：l1 = 4 rapid + 1 rocket / l2 = 6 rapid / l3 = 5 rapid + 1 rocket / chain = 5 rapid），**零漏**断言 `lives stayed 5` |
| 登记项② victory→level_select→另一关→再终局 | `chain` 相位：level_01 victory → 菜单（cleanup 断言 + `GameResult` 保留契约）→ level_02（lives/gold/地图/指纹全新）→ defeat → `result flipped Victory → Defeat` |
| 登记项① `td.state set` 面纳入并注记 | 每次进/出关都断言 `{queued, from, to}`（本档 §二 单列 16 条）；记：**set 面 = BRP 无键鼠注入的判定面必要替代**（T048 U1 裁 A），本轮 16 次转移全部按 from/to 断言、无一次隐式跳转 |
| 登记项③ O1·U9 采样窗 / spawn 延迟 / 长路径通用口径 | 统一为「**事件表独立复算 + elapsed 检查点**」：脚本内 `parseLevelEvents()` 从 TOML 独立展开事件表（时间/血量/速度/bounty），每关 3 个 `elapsed` 检查点断言「spawned == 事件表中 t ≤ elapsed 的条数」与 `pending == total − spawned`；victory/defeat 用时用「末刷时刻 / 第 5 只漏怪时刻」代数上界断言（非手数值） |
| 登记项⑥ U3 溅射聚怪（可选 ≤20min） | `splashSignature()` 探针（两敌同时恰为满血−50 且间距 <60px）→ **2 处 SKIP**（l1/l3 battle 各一次）；U3 裁「SKIP 维持、非收官门槛」，见 §四-6 与 §五 U3 |
| 登记项⑦ `td.input` 注入通道评估 | §四-7 建议口径已被 G1 采纳为**定案口径**（U2）；脚本内保留一次 `emit Right` 端到端冒烟 |
| 无 P0：td.* 错误路径不击穿进程 | `screen` 段 18 条错误路径 + `p0probe`（返工轮 9/9） + `final process alive` |

## 四、登记项 ①–⑦ 处置（逐条）

1. **① `td.state` set 面**：纳入判定面（每关进出 + chain 12 跳共 16 次 from/to 断言）。注记：`set` 是 BRP 无键鼠注入下的状态机驱动面（T048 U1 裁 A），本轮全部终局迁移均由 `set` 驱动；`GameOver` 的**生成路径**仍走游戏内 `check_game_state`（非 set），故 set 只作「观察面驱动」，不承担结果赋值。
2. **② victory 全链重测**：`chain` 相位已落（victory → 菜单 → level_02 → defeat → result 翻转）；额外断言「菜单期 `GameResult` 保留（汇总屏契约）+ `GameFinished` 已移除（新局门闩重置）+ 地图/塔/敌人零残留 + `td.wave`/`td.economy` 不可用」。
3. **③ 采样窗口径**：见 §三 表格行；口径落字 = 「期望值一律由脚本从 TOML 独立复算；采样点以 `elapsed` 为锚（不用 sleep 计时）；长路径关卡用 `pathPx/speed` 代数式给终局用时上界」。
4. **④ 见件二**（`w2-final-check.md`）。
5. **⑤ ts06.rs 注锚收紧**：`game/src/rpc/suites/ts06.rs:15-17` 的 `World::get_entity` 原注锚 `world/mod.rs:951` 经本地 `bevy_ecs-0.20.0` 源码实核**已漂移**——`:951` 落在**下一函数** `get_entity_mut` 的文档块；`get_entity` 签名在 **`:934`**、返回类型在 **`:937`**（`Result<F::Ref<'_>, EntityNotSpawnedError>`）。已改为精确锚并注记漂移来源（0.19→0.20 行号残留）。前后对照：

   ```
   改前：//! （world/mod.rs:951，`Result` 形态，Err 即不存在）（核实 2026-09-27）。
   改后：//! （bevy_ecs-0.20.0/src/world/mod.rs:934 函数签名，`:937` 返回
         //! `Result<_, EntityNotSpawnedError>`，Err 即不存在）（0.20 实核 2026-10-31：原注锚
         //! `world/mod.rs:951` 落在**下一函数** `get_entity_mut` 的文档块内，系 0.19→0.20
         //! 行号漂移残留——本行收紧到签名行）。
   ```
   语义未变（`get_entity` 仍为 Result 形态、Err 即不存在），故只改注释、不改代码。
6. **⑥ U3 溅射聚怪（可选，时间盒 ≤20min）**：探针已实现并随 l1/l3 两轮 battle 跑（rocket 实际开火：`fired≥7`、`projectile≥1`），但**签名未观测到** → 2 处 SKIP（不判 FAIL）。原因如实记：敌军按固定间隔 spawn 且同速段不聚团，异速追及点（如 level_01 的 runner 追及 soldier）在 3–4 座塔的 DPS 下存活窗口过短，20min 时间盒内无法稳定构造「两敌同时处于 60px 内且各挨恰好一次溅射」的采样。**G1 U3 裁：SKIP 维持**（登记⑥ 本为可选，与性能/音频非门槛同族，不另派卡）。
7. **⑦ `td.input` 注入通道评估**：两条通道实测语义：
   - **A（`td.input emit GameAction`）**＝写真消息队列，消费者在**下一帧**读；端到端但**非同步**，且不校验「由谁消费」；
   - **B（写状态 + `run_system_once` 直跑消费者）**＝`td.level_select click` / `td.pause` 的做法：同步、确定性、单次调用即见下游效应；但**绕过消息投递**（测消费者逻辑，不测投递）。
   **G1 U2 裁：采纳建议口径**——判定面**主用写状态 + `run_system_once`（确定性锚点）**，每类动作保留 ≥1 条 `emit GameAction` 端到端冒烟。本卡脚本已按此实现（smoke 一条 + 其余走写状态/BRP 直通道）。

## 五、上报项处置（U1–U4，含 G1 裁决）

- **U1（首轮阻塞 → 已裁 (a) → 已修 → 已复验）**：`td.state set GameOver` 在 `GameResult` 未建立时击穿进程。
  - 根因（首轮亲核）：正路（`check_game_state` 在 FixedUpdate 链末 `insert_resource` → 命令 sync point 落地 → `OnEnter(GameOver)` 读取）永不缺席；但 `td.state` 的文档**显式允许** `GameOver`（`game-td/src/brp.rs:597` / `:619` / `:627` / `:649`），带外转移击穿进程踩 M5 红线。
  - **G1 裁 (a)**：`setup_game_over` 改 `Option<Res<GameResult>>`，`None` → 不 spawn 结算屏 + `info!` 标记（fail-survivable 且非 fail-silent）；正路行为零变化。已落（diff 摘要见 §十一）。
  - 复验：`p0probe-run2.log` 9/9 PASS（存活 / state GameOver / result null / finished false / 无 panic / skip 标记）。
- **U2（判据口径，已裁定案）**：采纳「判定面主用写状态 + `run_system_once`；每类动作 ≥1 条 `emit GameAction` 冒烟」为本仓定案口径（见 §四-7）。
- **U3（可选扩展，已裁 SKIP 维持）**：溅射多目标签名不列入 M6 收官门槛，不另派卡；仅本档 §四-6 注记。
- **U4（文档同步，G1 裁定由 Lead 收获批落字）**：`docs/m6-game-selection.md` §五 补「m6-all 判定面 = 纯脚本侧组合既有 18 方法」一句——本卡不动（已按要求让出写面）。

## 六、门禁（首轮 a 档 + 返工轮 b 档，全绿；首轮档一律保留、另名不覆盖）

**返工轮（终态，b 档）**：

```
1) cargo check --workspace -j 3                       → REAL_EXIT=0，0 警告 0 错误   （gate-check-workspace-t052b.log）
2) cargo test --workspace -j 3 -- --test-threads=3    → REAL_EXIT=0；game 20/20、mcp-bridge 15/15、
                                                        doctest 78 passed / 0 failed / 17 ignored（209.73s）
                                                                                    （gate-test-workspace-t052b.log）
3) cargo test --doc -p docs -j 3 -- --test-threads=3  → 78/0/17，REAL_EXIT=0（198.96s）  （gate-doc-test-t052b.log；
                                                        先产本档再跑 claim-lint——recheck-registry 使用说明④）
4) cargo run -p claim-lint                            → 13 条断言 PASS 13 / FAIL 0 / SKIP 0，REAL_EXIT=0；
                                                        doctest-gate 选中 gate-doc-test-t052b.log（78/0/17）
                                                                                    （gate-claim-lint-t052b.log）
5) 游戏进程收尾零残留                                  → tasklist | grep -iE 'game-td|game.exe|rustc|cargo'
                                                        零命中（GREP_EXIT=1）        （process-check-t052b.log）
```

**首轮（a 档，保留）**：`gate-check-workspace.log` / `gate-test-workspace.log` / `gate-doc-test-t052.log` / `gate-claim-lint-t052.log` / `process-check.log` —— 与 b 档同值（首轮改动的唯一 Rust 侧文件是 ts06 注释；b 档覆盖 U1 修复后的终态）。

所有 cargo 命令 `-j 3`、测试 `--test-threads=3` 透传、`CARGO_TARGET_DIR=<main-repo>/target`（复用既有产物；对齐 T049–T051 口径）。**游戏进程存活期未起任何 cargo**：每轮流程 = 构建 → 起 exe → 判定 → taskkill → 核零残留 → 下一动作。

## 七、指纹链与截图链

### 7.1 GRID_DIGEST（三关，run4 一次过）

| 关卡 | digest | 期望来源 |
| --- | --- | --- |
| level_01 | `70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07` | T047 冻结基线 |
| level_02 | `3edf8dffc55b743933c8c4b8484f50855b8e7d193495d5f4c2536f96ea8a4f6d` | T049 P13 登记 |
| level_03 | `840298ef16970017e1916a573a5333759bea8782a959509d365c12fc9dcd76f7` | T049 P13 登记 |

### 7.2 ENTITY_DIGEST（无害塔 + elapsed ∈ [2.2,3.0)）

| 关卡 | digest（run4） | 判定 |
| --- | --- | --- |
| level_01（塔 `[2,0]`） | `1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23` | == T048 冻结基线（逐位一致）+ == 独立复算 |
| level_02（塔 `[7,0]`，dist=256） | `47c5f8bb19be3c5469fd0cf7f622f53d235a6e17782673a6bc2596a388645f84` | == 独立复算（该关无冻结基线） |
| level_03（塔 `[14,9]`，dist=326.3） | `7c587b5b2f24b8fded0021a135819d5270e23edbc4d7fdfb8ea1ce8b83527018` | 同上 |

### 7.3 截图链（md5，PNG 均验 1600×900 + 非空）

| 文件（run4 终态） | md5 | bytes | 判定 |
| --- | --- | --- | --- |
| `m6-all-shot-l1-level_01--run4.png` | `1e1c9f7acfe6dc5234603db91b8958f0` | 89156 | level_01 在局（cp1 窗口，塔 + 敌在路） |
| `m6-all-shot-l2-level_02--run4.png` | `ffb4ba0e2bdeb27276653acf3775ff01` | 109294 | level_02 在局（L 形路径） |
| `m6-all-shot-l3-level_03--run4.png` | `409abad4b931c4f3eed6b533763b3bfa` | 141409 | level_03 在局（之字形路径） |
| `m6-all-shot-chain-v-level_01--run4.png` | `fd0221027cc65415e61b2635b46320d8` | 87713 | chain 相位 level_01 在局 |
| `m6-all-shot-victory-run4.png` | `68ee729b6980d5e001391ce6b1a4a652` | 48928 | 胜利结算屏（静态屏；与 run2/run3 同值 → 跨轮确定性） |

在局截图 md5 逐轮不同属预期（敌方位置 / 交战帧不定）；本卡判定面 = 断言计数 + digest + 经济账，截图为视觉留证。**瑕疵披露**：run4 沿用了与 run3 相同的 `--shot` 参数，四张同名在局档被覆盖（run3 的 md5 仍在 `m6-all-run3.log` 的 `SCREENSHOT` 行，文件不可复原）；run4 完成后已把文件改名加 `-run4` 后缀以明示轮次（见 §八）。

## 八、跑档披露（失败轮 / 中间轮 / 调试轮 / 返工轮全留，禁覆盖）

| 轮 | 结果 | 归因与处置 |
| --- | --- | --- |
| dev 分段轮 ×8（`dev-*.log`；游戏日志 `m6-all-dev.game.log`） | 全部为脚本校准轮 | ① l1 dev run1：2 FAIL（终局后查 Tower 计数 = 0；defeat 的 lives 采样错过最后一次扣命）→ 断言设计 bug，改「检查点内核对塔数」+「≥4 次扣命观测」；**该轮仅控制台执行、未成档**（如实记）；② l3 dev run1：rocket 从未落下（金币优先花在 rapid 上）→ 资源优先级 bug；③ l3 dev run3：rocket 位与 rapid 撞车 → 拒绝刷屏；④ l3 dev run4：`rocketTile=undefined` → `-32602` 抛错 → 改 `bestCoverTile` 排除占位；⑤ 其余（l1 run2/run3、l2 run1、l3 run5、chain run1）为绿档 |
| **run1**（`m6-all-run1.log`） | PASS=470 / **FAIL=1** / SKIP=2（473），REAL_EXIT=1 | **失败轮留档**：`l2 cp1 economy ledger diff=25.28` —— 单次采样撞上「在飞击杀」（三次独立 BRP 读之间敌人死亡，bounty +25 已入账而 alive/spawned 仍是击杀前），**驱动侧非原子性**，非游戏缺陷。处置：经济体改为**一致采样重采**（最多 3 次、全档披露 attempt、恒等式阈值不变 1.0 金） |
| **run2**（`m6-all-run2.log`） | PASS=483 / FAIL=0 / SKIP=2（485），REAL_EXIT=0 | 绿档，但披露一处取证瑕疵：4 个胜利相位共用同一 `--shot` 路径 → 截图覆盖式同名。处置：截图路径按相位加后缀后重跑；run2 及其截图原样留档 |
| **run3**（`m6-all-run3.log`） | PASS=483 / FAIL=0 / SKIP=2（485），REAL_EXIT=0 | 首轮判定绿档（件一） |
| **p0probe-run1**（独立进程） | PASS=4 / **FAIL=1**（5），REAL_EXIT=1 | **件三首轮判定 = FAIL**：命中「`td.state set GameOver` 缺 `GameResult` → 进程击穿」；进程按预期退出（收尾核零残留）。按「崩了 = FAIL 不是重跑」如实判定，触发 G1 U1 裁 (a) |
| **run4**（`m6-all-run4.log`） | **PASS=483 / FAIL=0 / SKIP=2（485），REAL_EXIT=0** | ★ 返工轮终轮（U1 修复后全量重跑）。一致采样触发 1 次（`l3 cp1`：首样本 `alive=5 > spawned=4` = 刷怪竞态，第二次 diff=0.0938，attempt 序列在档） |
| **p0probe-run2**（独立进程） | **PASS=9 / FAIL=0 / SKIP=0（9），REAL_EXIT=0** | ★ 件三复验：进程存活 + state GameOver + result null + finished false + 日志无 panic + skip 标记在档。进程存活留于 GameOver 态，随后 taskkill 核零残留 |
| 已知 INFO（非断言） | — | chain 相位最后一次机会性补塔落在终局迁移之后，返回 `no level loaded (MapLayout missing)`（该分支只读不写世界，无副作用）；`td.combat` 的 `rapid` 面 `launchers=0` 属设计（rapid 为瞬时塔、无 `AmmoState`），rocket 面才有 `launchers/ammo/fired/projectile` |
| 截图覆盖瑕疵（返工轮） | — | run4 沿用了 run3 的同名 `--shot` 路径 → run3 四张在局档被覆盖（run3 md5 仍在 `m6-all-run3.log`）。处置：run4 文件改名加 `-run4` 后缀；本档如实披露，后续轮次一律按轮次命名（教训登记，见 §十一） |

## 九、写面清单与零依赖声明

- **新增（证据面）**：`docs/evidence/t052-m6-all/**`（脚本 + 两轮断言档 + 件二/件四 + 两轮门禁档 + 截图 + 失败/调试轮）。
- **修改（3 处）**：
  1. `game/src/rpc/suites/ts06.rs`（登记项⑤：`world/mod.rs:951` → `:934/:937`，仅 doc 注释）；
  2. `game-td/src/game_over.rs`（G1 U1 裁 (a)：`setup_game_over` 参数 `Option<Res<GameResult>>` + None 空屏 + `info!` 标记 + 模块/函数注释补句，见 §十一 diff）；
  3. 本证据目录内的判定面脚本与文档（同一路径演进，轮次以档名区分）。
- **未动**：`Cargo.toml` / `Cargo.lock` / `game-td/Cargo.toml`（内容 strip-CR 后 SAME，零依赖变更）、`tooling/**`、`assets-methodology/**`（台账/taskset 由主会话收获批登记）、`bevy-dev/**`、`docs/` 其余文件、主仓任何路径。
- 写面自检：首轮 `scope-check.log` + 返工轮 `scope-check-t052b.log` —— 树 vs 主仓 `d6d7609` 差异**仅** `docs/evidence/t052-m6-all/`（新增目录）+ `game/src/rpc/suites/ts06.rs` + `game-td/src/game_over.rs`；主仓 `git status --porcelain` 空（`<main-repo>` 未被写入）。

## 十、与既有形态的偏离（口径说明）

1. **判定面按「相位复用」组织**：`l1/l2/l3` 每相位内部依次跑「victory → defeat → ENTITY_DIGEST」三轮——判定面不依赖绝对 entity id、也不断言进程级累计计数（K5 口径），同进程顺序跑不互相污染，且把「零 panic」覆盖到完整链路。
2. **经济账用一条恒等式**（初始金币 + 被动 + 赏金 − 花费）替代逐项孤立采样：一次钉住四个经济要素，期望值全由脚本侧常量与 TOML 推出。
3. **victory 严格要求零漏**（`lives stayed 5`）：派工单「victory = 零漏清波」照字面落地；defeat 轮要求「零伤害 + 零击杀 + 零塔」以证明判负确由漏怪致。
4. **一致性重采**（经济账 / 血量多重集）：对「多次独立 BRP 读 + 在飞击杀/刷怪」的驱动侧非原子性，采用「重采到自洽 + 全档披露 attempt」，不放宽阈值。
5. **p0probe 与 `all` 分进程**：探针会改变状态机（进入 GameOver 且无结算屏），与判定面混跑会干扰后续相位；分进程也保证「击穿与否」独立可判。
6. **截图按相位加后缀**（`withTag`），但**未按轮次加后缀**（run4 覆盖 run3 同名档）——已披露并在返工轮后改名补正。

## 十一、G1 返工轮记录（U1 裁 (a)）

**派工依据**：G1 前置裁决（Lead 亲核 `game_over.rs:19` / `brp.rs` `td.state` 文档行 / `p0probe-run1.game.log` 全读）。

**修复 diff 摘要**（`game-td/src/game_over.rs`，strip-CR 真实差异 = 2 处）：

```
- pub fn setup_game_over(mut commands: Commands, result: Res<GameResult>) {
+ /// T052 U1：`result` 为 `Option`——BRP 带外转移（`td.state set GameOver`）可绕过
+ /// `check_game_state` 的写入顺序，此时 `GameResult` 缺席；缺席即空屏（不 spawn）+
+ /// info 标记，避免测试通道击穿进程（M5 红线）。正路（游戏内胜负）恒为 `Some`。
+ pub fn setup_game_over(mut commands: Commands, result: Option<Res<GameResult>>) {
+     let Some(result) = result else {
+         info!("[TD] setup_game_over skipped: no GameResult (out-of-band state transition)");
+         return;
+     };
      let message = match *result { … };

 模块注释（第 3–6 行）：保留教程 12 Step 9「Critical ordering detail」顺序契约原文；
 追加「**T052 U1 起**：BRP 带外转移下 GameResult 可能缺席，参数改 Option——None 时
 不 spawn 结算屏、留一行 [TD] info 标记（fail-survivable 且非 fail-silent）；正路契约不变」。
 「若清理链删掉 GameResult，本屏 Res<GameResult> 会 panic」一句同步改为「结算屏就无文案可显示」。
```

**脚本侧**（`m6-all.mjs`）：`p0probe` 断言由「是否击穿（期望 no panic + trace）」翻转为**存活面回归**（存活 / state GameOver / result null / finished false / 无 panic / skip 标记在档，共 9 条）；卫生清理 = 删 `ledgerSampleConsistent` 未用的 `label` 形参、`scanGameLog` 去掉死分支 `expectPanic`。

**复验链**：重建 release exe（`m6-all-build-run2.log`，REAL_EXIT=0，32.82s）→ `run4` 全量 483/0/2 绿 → `p0probe-run2` 9/9 绿 → 四闸 b 档全绿 → 进程零残留 → 写面自检（`scope-check-t052b.log`）。

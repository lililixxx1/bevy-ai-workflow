# TS-14 证据：战斗核心（回合机/移动/攻击/敌方 AI/胜负）

- 日期：2026-09-28。启动：`./target/release/game.exe --count 0 --seed 20260926`（release 构建 `gate-build-release.log` REAL_EXIT=0）。
- 驱动：`drive-ts14.js`（node 内嵌贪心玩家驱动器，全量请求/响应经 BRP；逐条断言 + 全量 transcript/replay 落档）。
- 判定：**12/12 PASS（运行时 11/11 退出码 0 + 日志面 #12）**，`drive-ts14-out.txt`（run3 干净整轮）。

## 逐条判定（run3，单进程干净整轮）

| # | 判定 | 关键证据（摘自 transcript，逐字） |
|---|---|---|
| 1 | PASS | 未加载关卡 `get_resources(game::battle::BattleState)` → `error.code=-23501`（资源实例不存在口径，与 level.rs 注释/T032 审核结论一致） |
| 2 | PASS | `launch(1,seed=7)` 响应五键形态与 TS-13 一致（响应未加键）；`BattleState` 初值 `phase=0,turn=1,winner=-1,goal_kind=0,goal_x=-1,goal_y=-1,seed=7,rng_state=7,ai_kind=0` 逐位 |
| 3 | PASS | `(1,1)→(1,3)` 响应 `{"from_x":1,"from_y":1,"phase":0,"to_x":1,"to_y":3,"winner":-1}`；query 三组件 `(1,3)` team0 `moved=true` |
| 4 | PASS | 已移动/超范围/占用/越界四探针全 `-32602`；探针后布阵与 `BattleState` 与 #3 后逐位一致（无副作用） |
| 5 | PASS | 距离>1/无单位/友方三探针全 `-32602` + 无副作用 |
| 6 | PASS | `end_turn` 响应 `turn=2`、`enemy_actions` 3/3（全员有记录，kind ∈ 四类）；玩家 `moved` 复位（#3 单位再移动实证成功）；`rng_state=15755400384260045000 != 7`（随机档抽取推进）。

  > **勘误（R1 审核 B2，2026-09-28）**：上值系 node `JSON.parse` 的 f64 渲染值；**精确 u64 = 15755400384260043846**（curl 直写原文 + 独立复算 7+3·0x9E3779B97F4A7C15 mod 2^64 双证，`u64-precision-probe/03-battle-raw.json` + `04-node-precision-demo.txt`；错题入库 PIT-M-009——本表其余数值摘自 node transcript 的 u64 字段同理为渲染值，小整数与布尔/坐标不受影响） |
| 7 | PASS | 贪心驱动完整对局：**7 回合玩家胜（歼灭）**，终局 query 敌方 0 行；对局全程零非预期 error（预期 error 仅 #4/#5/#8 探针）；每步状态快照 30 条入 `ts-14-replay.jsonl` |
| 8 | PASS | 胜利后 move/attack/end_turn 三探针全 `-32602`（`"message":"终局（winner=0），不可再操作；重开请 game.launch_level"`）；随后 `rpc.discover` 正常应答（进程存活） |
| 9 | PASS | `relaunch(1,seed=7)`：`BattleState` 全量复位（`phase=0,turn=1,winner=-1,rng_state=7`）+ 6 行布阵 |
| 10 | PASS | 关卡 3：`goal_kind=1,goal=(8,4),ai_kind=1,units_spawned=9`；两步 `{"from_x":0,"from_y":4,"to_x":4,"to_y":4,...,"winner":-1}` → `{"from_x":4,"from_y":4,"phase":2,"to_x":8,"to_y":4,"winner":0}`（占点即胜，敌方存活 5 亦可） |
| 11 | PASS | `rpc.discover` 30 方法（27+3）：`game.move_unit`/`game.attack`/`game.end_turn` 在列 |
| 12 | PASS | `run3-game.log` 无 `panicked`；`[BRP] custom methods:` 行列全 7 自研方法；`[BRP] listening on 127.0.0.1:15702`（回环绑定） |

## 过程偏差（如实记）

- **P1（驱动脚本笔误，不计游戏侧返工）**：run1 的 #10 断言字面值误写 `units_spawned===8`（关卡 3 实为 3+6=9）——游戏侧两步移动响应全对（transcript 摘录见上），修正字面值后过。run1 全量 raw 以 `-run1-miscount` 后缀存档。
- **P2（判定语义进程绑定）**：修正后未重启进程即在同进程复跑（run2），#1「未加载关卡 → -23501」因 `BattleState` 已被 run1 写入而 FAIL——该断言语义绑定**新进程首次**，非游戏侧缺陷。重启进程后 run3 干净整轮 12/12。
- **P3（run2 证据未独立存档）**：run2 的 transcript/replay 与判定输出被 run3 同名覆盖（未另存）——run2 结论（11/12，仅 #1 状态污染）以本节文字为准；run1/run3 raw 均完整存档。
- **P4（门禁返工，计台账返工①②）**：`cargo test -p game` 首轮 E0369 ×2（`UnitView` 缺 `PartialEq`，断言用 `assert_eq!(unit_at(..), None)`）；二轮 E0499 ×2（测试内 `w.get_mut` 参数中嵌套 `unit_at(&mut w)` 双重可变借用）——两处测试代码修复后 r3 20/20。按 T003/T019 先例（首次 test 失败即返工）计 2 次。
- **范围内调整（如实记）**：`cargo check` 首轮 REAL_EXIT=0 但 3 处 dead_code 警告（`PHASE_ENEMY`/`AI_GREEDY`/`GoalDef::Reach` 未用）——处置：`PHASE_ENEMY` 落为 end_turn 瞬态真实写入、AI 常量经 launch `debug_assert` 引用、**关卡 3 数据表（含占点目标）自 T034 计划提前并入本任务**（胜负两型机制本任务已全量实现，占点无数据可验即真缺口；T034 转为进程内战斗回归套件）。r2 复跑 0 警告。

## 门禁

- `cargo check --workspace`：r1 REAL_EXIT=0（3 警告）→ 修警告后 r2 REAL_EXIT=0（0 警告）。
- `cargo test -p game`：r1/r2 REAL_EXIT=101（见 P4）→ r3 REAL_EXIT=0（**20 passed / 0 failed**：存量 11 + battle 新 8 + rng 新 1）。
- `cargo build --release -p game`：REAL_EXIT=0。
- doctest 门禁未变（docs/ 无涉本任务的门禁内文档）——终版双门禁复跑见台账 T033 行与 `gate-check-final.log` / `gate-doc-test-final.log`。

## 过程偏差（终审补记）

> **勘误（终审 B1，2026-09-28）**：T033 起本批门禁日志未自含 `REAL_EXIT` 行（bash 侧回显未落盘，T031/T032 口径回退）——成文所记退出码以当次 bash 回显为准；自 T037 收口起恢复自含，`docs/evidence/m4-acceptance/gate-check-final2.log` / `gate-doc-test-final2.log`（77/0/17）为自含复跑档，终审核立独立复跑同绿。

## 文件清单

`drive-ts14.js`（驱动+断言）、`drive-ts14-out.txt`（run3 判定输出）、`ts-14-transcript.jsonl`（210 行全量请求/响应原文）、`ts-14-replay.jsonl`（30 步状态快照）、`ts-14-transcript-run1-miscount.jsonl` / `ts-14-replay-run1-miscount.jsonl`（run1 存档）、`run1-game.log` / `run2-kill.log` / `run3-game.log` / `run3-kill.log`、`gate-check-r1/r2.log`、`gate-test-r1/r2/r3.log`、`gate-build-release.log`。

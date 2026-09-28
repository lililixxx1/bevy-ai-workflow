# TS-14：战斗核心——回合机 / 移动 / 攻击 / 敌方 AI / 胜负（M4 A 案系统席②④⑤⑥⑦）

- 难度：高
- 前置：TS-13（launch_level 数据面）；M4 选品定本（docs/m4-game-selection.md v1.0 §二 A 案系统席②④⑤⑥⑦）
- 类型：功能（M4 核心系统入库：`game::battle::{BattleState, ActionFlags}` 规则面 + `game.move_unit` / `game.attack` / `game.end_turn` 三指令通路）

## 需求描述

实现并验证 M4 战斗核心（规则面 `game/src/battle.rs`：回合机阵营轮转、网格占用、曼哈顿移动、贴身整数伤害、敌方 AI 两档（种子随机/贪心）、胜负判定两型（歼灭/占点）；`launch_level` 扩战斗态初值 + 可选 seed 参数）。全部离散结算同步发生于 BRP 独占执行内；确定性纪律：业务键序遍历（PIT-B-010）+ `SplitMix64` 整数取数（RNG 进度入 `BattleState.rng_state` 跨请求续跑）。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 0 --seed 20260926`。驱动：node 内嵌贪心玩家驱动器（读态 → 计算移动/攻击 → 逐条发指令），全量请求/响应原文落 `ts-14-transcript.jsonl`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.get_resources` | `{"resource":"game::battle::BattleState"}` | 未加载关卡：`error.code == -23501`（RESOURCE_ERROR，资源实例不存在——非 null，`builtin_methods.rs:617-637` 口径） |
| 2 | `game.launch_level` | `{"level":1,"seed":7}` | 响应六字段与 TS-13 #2 形态一致（响应不加键）；`get_resources` 读 `BattleState`：`phase=0, turn=1, winner=-1, goal_kind=0, goal_x=-1, goal_y=-1, seed=7, rng_state=7, ai_kind=0` 逐位 |
| 3 | `game.move_unit` | `(1,1)→(1,3)`（距离 2 ≤ 3，终点空闲） | `result.winner==-1 && result.phase==0`；query 三组件确认 `(1,3)` 属 team0 且 `ActionFlags.moved==true` |
| 4 | `game.move_unit` ×4（错误面） | 已移动（同单位再动）/ 超范围（`(2,1)→(2,8)`）/ 占用（`(2,1)→(1,3)`）/ 越界（`(2,1)→(9,1)`） | 全部 `error.code == -32602`；随后布阵与 `BattleState` 与 #3 后逐位一致（无副作用） |
| 5 | `game.attack` ×3（错误面） | 距离>1（`(1,3)` 打 `(5,7)`）/ 目标无单位（`(1,3)` 打 `(4,4)`）/ 友方（`(1,3)` 打 `(2,1)`） | 全部 `-32602`；无副作用 |
| 6 | `game.end_turn` | `{}` | `result.turn==2 && result.phase==0 && result.winner==-1`；`enemy_actions` 数组长度 == 当轮敌方存活数且每条 `kind ∈ {attack,move,move_attack,pass}`；玩家 `moved` 复位（#3 单位可再移动实证）；`BattleState.rng_state != 7`（随机档抽取推进） |
| 7 | 贪心驱动完整对局 | 关卡 1（seed=7）逐回合：每玩家单位趋近/贴身攻击最低血，`end_turn` 收口，上限 30 回合 | `winner==0 && phase==2` 终局；query 敌方 0 行；**对局全程驱动日志零非预期 error**（预期 error 仅 #4/#5/#8 探针）；每步后状态快照（业务键排序布阵+HP+BattleState）入 `ts-14-replay.jsonl` |
| 8 | 终局封锁 ×3 | 胜利后 move / attack / end_turn | 全部 `-32602`；进程存活（后续请求正常应答） |
| 9 | `game.launch_level` | 再 `{"level":1,"seed":7}` | `BattleState` 全量复位（`phase=0, turn=1, winner=-1, rng_state==7`）；布阵 6 行业务键集合与 TS-13 #3 一致 |
| 10 | 占点型关卡 3 | `{"level":3,"seed":7}` → 驱动 `(0,4)` 单位直奔 `(8,4)` | `BattleState.goal_kind==1 && goal_x==8 && goal_y==4 && ai_kind==1`；中途移动响应 `winner==-1`；踏上 `(8,4)` 那次移动响应 `winner==0 && phase==2`（占点即胜，敌方存活亦可） |
| 11 | `rpc.discover` | 无 params | 方法数 27→30：新增 `game.move_unit` / `game.attack` / `game.end_turn` 三方法在列 |
| 12 | 游戏日志 | 全程 | 无 `panicked`；`[BRP] custom methods` 行列全 7 个自研方法（进程无崩溃面） |

## 判定记录

- 判定者/日期/结论：执行 agent，2026-09-28，**通过（12/12 PASS：运行时 11/11 退出码 0 + 日志面 #12——无 panic、7 自研方法、无进程残留）**。核心闭环实证：关卡 1 贪心驱动 7 回合歼灭制胜、关卡 3 两步占点制胜、终局封锁与全错误面 `-32602` 无副作用、同进程 relaunch 全量复位。驱动侧偏差三笔如实记（`docs/evidence/ts-14-brp.md` P1-P3：#10 字面值笔误 8→9 / run2 同进程状态污染 #1 / run2 证据被覆盖）；门禁返工 2 次计台账 T033（单测首轮 E0369、二轮 E0499）。范围内调整：关卡 3 数据表自 T034 提前并入（dead_code 处置 + 占点型无数据可验即真缺口），T034 转为进程内战斗回归套件。
- 佐证：`rpc.discover` 30 方法（27+3）；`BattleState.rng_state` 跨请求续跑实证（seed=7 → 15755400384260045000）；确定性纪律（业务键序 + SplitMix64 整数取数）另由 `cargo test -p game` 进程内单测 8 条锁定（含「同布阵异生成序同种子 → 同状态」的迭代序直接检验）。
- 证据：`docs/evidence/ts-14-brp.md`（逐条判定 + 过程偏差 + 门禁）；`docs/evidence/ts-14/`（驱动脚本、210 行全量 transcript、30 步 replay 快照、run1 存档、运行/杀进程/门禁日志）；台账 T033。

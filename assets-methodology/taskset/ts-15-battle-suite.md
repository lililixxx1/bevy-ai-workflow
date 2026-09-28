# TS-15：战斗规则进程内回归套件（game.run_tests "ts-14"）

- 难度：中
- 前置：TS-14（第一阶段 BRP 驱动 12/12）；M1 两阶段判定先例（T020）
- 类型：功能（M4 核心系统任务化：战斗规则进 `game.run_tests` 套件族，成为快速回归面）

## 需求描述

将 TS-14 战斗核心转为游戏进程内套件 `ts-14`（`game/src/rpc/suites/ts14.rs`：6 断言——初值/移动收拒/击杀歼灭终局封锁/同世界同种子双跑确定性（含 RNG 续跑面）/占点即胜/快照恢复），沿用套件族「无净副作用」约定（入场快照三组件 + 资源原值或缺失，出场重生/还原/移除）。规则函数与 BRP handler 同一 `&mut World` 通路（ECS 直写等价，M1 校准口径）。

## 验收清单（BRP 断言）

启动方式：`./target/release/game.exe --count 0 --seed 20260926`（战斗域独立于 demo 域；全 11 旧套件的校准口径回归归 TS-16 战役回归任务）。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `game.run_tests` | `{"suite":"ts-14"}` | `total=6, failed=0`（六断言名与 TS-14 清单对应见套件文件头） |
| 2 | `game.run_tests` ×2 | 同进程第二次 | 仍 `failed=0`（连跑自洽——无净副作用约定的直接实证） |
| 3 | `game.launch_level` + `game.run_tests` | `{"level":1,"seed":7}` 后跑套件 | 套件仍 `failed=0`；**套件后** query 布阵 6 行业务键集合与加载后逐位一致、`BattleState` 回初值（`turn=1/winner=-1/rng_state==7`——套件的战斗未污染局外状态，快照/恢复含 RNG 状态面） |
| 4 | `game.run_tests` | `{"suite":"ts-99"}` | `-32602`；message 可用清单含 `ts-14`（注册表扩展可发现） |
| 5 | 游戏日志 | 全程 | 无 `panicked`（套件不 panic 纪律——断言失败走 `pass=false`，规则函数 `Err` 走断言 false 而非 expect） |

## 判定记录

- 判定者/日期/结论：执行 agent，2026-09-28，**通过（4/4 运行时 + 日志面 #5：0 panic、进程无残留，退出码 0）**。#3 是快照/恢复的强验证：加载态（关卡 1、seed=7）下套件打完自己的战斗后，世界布阵逐位还原且 `rng_state==7` 未被推进（套件内 RNG 消耗被恢复覆盖）。门禁返工 2 次计台账 T034（套件首轮 E0382×2 `saved.level.map` 移动后借用 + 二轮 E0308×2 `Option<&T>` 比较形态），修复后 check 0 警告 + 单测 20/20。
- 佐证：套件断言明细 `battle_state_initial / move_accept_and_reject / attack_kill_annihilate_and_lock / end_turn_twin_deterministic / reach_goal_wins_on_move / world_restored_after_suite`；注册表共 **12** 套件名（ts-01..03,05..12 + ts-14——初稿「13」系计数笔误，R1 审核 S1 勘误，与 suites::all() 及错误 message 实测一致）。
- 证据：`docs/evidence/ts-15-brp.md`；`docs/evidence/ts-15/`（驱动脚本、transcript、运行/杀进程/门禁日志）；台账 T034。

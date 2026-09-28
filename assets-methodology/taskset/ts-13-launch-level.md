# TS-13：关卡加载 game.launch_level（M4 A 案系统席①）

- 难度：低
- 前置：TS-05（world.spawn_entity 写通路先例）；M4 选品定本（docs/m4-game-selection.md v1.0）
- 类型：功能（M4 核心系统入库：`game.launch_level` + 关卡数据面 `game::level::{Unit, GridPos, LevelState}`）

## 需求描述

实现并验证 M4 首个核心系统：关卡加载。新增 `game::level` 模块（关卡定义常量表 + `Unit`/`GridPos` 组件 + `LevelState` 资源，全整数数值、纯数据无随机——同关卡号同一布阵）与 `game.launch_level` 自研 RPC（清场 → 按定义序生成 → 写状态资源；错误一律 `-32602` Result 路径不 panic）。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 0 --seed 20260926`（`--count 0`：不生成 wanderer 群，关卡域独立验证）。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.query` | `data.components=["game::level::Unit"]` | 0 行（启动无默认关卡） |
| 2 | `game.launch_level` | `{"level":1}` | `result == {level:1, name:"first-contact", width:9, height:9, units_spawned:6}` |
| 3 | `world.query` | `data.components=["game::level::GridPos","game::level::Unit"]` | 6 行；按业务键 `(team,x,y)` 排序后 `hp/move_range/attack` 逐位等于关卡 1 定义（PIT-B-010：不依赖迭代序与实体号） |
| 4 | `game.launch_level` ×2 | 连续两次 `{"level":1}` | 两次响应 `units_spawned == 6`；第二次后 query 仍 6 行且业务键集合与 #3 逐位一致（幂等：清场后重生，实体号允许变化） |
| 5 | `game.launch_level` | `{"level":2}` | `units_spawned == 9`；query 9 行业务键集合逐位等于关卡 2 定义（重载清场 6→9） |
| 6 | `game.launch_level` | `{"level":99}`（错误路径） | `error.code == -32602`；随后 `world.get_resources` 读 `game::level::LevelState` 仍为关卡 2 态（错误调用无副作用）；再 query 仍 9 行——**进程存活，自研方法错误路径不击穿进程**（备忘录 A 案可玩性定义验收面） |

## 判定记录

- 判定者/日期/结论：执行 agent，2026-09-28，**通过（第一阶段脚本判定，curl + node 逐条断言，6/6 PASS 退出码 0）**。游戏侧行为全程正确（含错误路径 -32602 与进程存活面）；如实记两笔**驱动侧**重发（不改判游戏侧结论，详见 `docs/evidence/ts-13-brp.md` 过程偏差 P2/P3）：断言 6 的 `get_resources` 首发误写复数形态（-32602 missing field `resource`，错误原文留档）后按 ts-03 既有形态修正；断言脚本首版 #2 被 serde_json 字母序键序绊倒（假阴性）改键序无关判等。任务返工口径见台账 T032（首次启动 `--count 0` 被 CLI 拒 → 放开下界，计 1 次返工）。
- 佐证：`rpc.discover` 27 方法（内置 23 + 自研 4）含 `game.launch_level`；重复 launch 幂等（实体号变化属预期，断言按业务键 `(team,x,y)`，PIT-B-010）。
- 证据：`docs/evidence/ts-13-brp.md`（curl 命令原文 + 响应原文摘录 + 逐条判定 + 过程偏差四笔）；原始响应/运行日志/门禁日志 `docs/evidence/ts-13/`；台账 T032。

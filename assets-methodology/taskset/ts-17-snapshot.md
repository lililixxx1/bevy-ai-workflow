# TS-17：稳定 id 快照 game.snapshot（窗口前置增强 T040 / A4）

- 难度：中
- 前置：TS-13/TS-14（关卡与战斗域——快照 battle 面的字段来源）；TS-16（T035 驱动侧快照字段集——本条目的收编来源）
- 类型：功能（自研 RPC 方法 + 进程内回归套件；`docs/pre-window-plan.md` §二 T040 卡）

## 需求描述

把 T035 已验证的驱动侧重放快照字段集收编为正式 RPC 方法 `game.snapshot`：以业务键为稳定 id 的只读世界状态快照——battle 面（units 按 `(x, y)` 业务键序 `{x,y,team,hp,moved,attacked}` + `turn/winner/phase/rng_state` ∈ `BattleState`，未加载关卡 = `null`）+ sim 面（`paused` ∈ `SimConfig` + wanderers 按 `Wanderer::index` 键序 `{index,origin,velocity,phase}` 确定性初值四元组）。**实体号零出现在快照面**；T035 快照的 `level`/`op` 系驱动侧记账，不入。约束：不引入时间依赖字段（`Transform`/`SimStats`——「跨进程逐字节一致」的面在时间无关初值上才成立）；`rng_state`（u64）在 JS 工具端沿 PIT-M-009 口径（字符串化或排除出严格相等，不倒退 TS-16 限定先例）；handler 全程 `Result` 路径不 panic。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 100 --seed 20260926`（后台，sim 面非空），BRP 端点 `http://127.0.0.1:15702`。

| # | 面与方法 | 要点 | 断言 |
|---|---|---|---|
| 1 | `rpc.discover` | 方法清单 | 含 `game.snapshot`；总数 31 = 23 内置 + 8 `game.*`（claim-lint `brp-total`/`game-methods` 联动） |
| 2 | `game.snapshot`（未加载关卡） | 冷启动先调（params 缺省） | `battle == null`（资源缺失不自造默认值）；`sim.paused` 为布尔 |
| 3 | `game.snapshot` ↔ `world.query`/`world.get_resources` 逐字段等价 | `game.launch_level {"level":1,"seed":20260928}` 后 | units/wanderers 逐字段逐序相等（业务键序）；turn/winner/phase/rng_state == `BattleState` 序列化值；paused == `SimConfig`（等价面在 < 2⁵³ 值域内严格相等，PIT-M-009） |
| 4 | 进程内套件 ts-17 | `game.run_tests {"suite":"ts-17"}` | 全 PASS：字段集==T035 / 逐字段等价 / 键序 / 同 seed 重 launch 快照一致**且实体号实际变化** / `battle=null` 分支 / 无净副作用 |
| 5 | 同 seed 跨进程两次启动 | 冷启动进程 A 与 B（同 `--count --seed`）各 launch(1, 20260928) 后 snapshot | 快照**原始响应体**（固定 JSON-RPC id）逐字节相等（`diff -q` 无输出；不经 JS 数值面，实体号不在快照面） |
| 6 | 回归 + 日志面 | 既有 12 套件 + ts-17 全跑 + 双进程 game 日志 | 13 套件全 PASS（failed=0）；0 panic（日志无 `panicked`）；收尾无 game.exe 残留 |

## 判定记录

- 判定者/日期/结论：执行 agent，2026-09-29，**通过（6/6 PASS）**。① discover 总 31 = 23 内置 + 8 `game.*`（含 game.snapshot）；② 未 launch 时 `battle=null`、`sim.paused=false`（wanderers=100）+ params 负控 -32602；③ launch(1, seed=20260928) 后双面逐字段等价（units 6 行 / battle 四字段 turn=1·winner=-1·phase=0·rng_state=20260928 / paused / wanderers 100 行四元组）；④ ts-17 套件 6/6（进程内，含「重 launch 快照一致且实体号换血」正证）；⑤ A/B 冷启动快照**原始响应体**逐字节相等（md5 `479388c3908c8a70939ea5f4dff6cdf6` 双侧同值，16756 字节）；⑥ 13 套件全 PASS + 双进程 0 panic + 收尾无 game.exe 残留。两处过程偏差如实记：首轮 `cargo check` 一处编译错误（`world.resource::<T>().cloned()` 应为 `.clone()`），即改即过；首轮驱动把套件排在 launch 之后，ts-08 相位 A 默认 90 断言假红（radius=14）——根因 = 表现层关卡视角收拢 + 该断言隐含「进程未加载过关卡」前提，修驱动顺序（套件先跑）后 13/13 全绿，坑入库 **PIT-M-010**（复现档 run-b-probe-attempt1-pitm010.log）。均计入台账 T040 返工口径。
- 证据：`docs/evidence/ts-17-brp.md`（逐条判定 + 摘要原文）；`docs/evidence/ts-17/`（snapshot-probe.js 驱动器 + A/B 双进程日志 + 快照原始体 ×2 + md5/diff + 13 套件复跑 + PIT-M-010 复现档 + claim-lint t040 档）；台账 T040。

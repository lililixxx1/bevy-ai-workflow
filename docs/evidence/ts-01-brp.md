# TS-01 断言证据：实体数量参数化与计数核对（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-01-entity-count.md`（4 条断言，逐条执行）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo 基线（本次**零改码**）
- 启动命令（与清单一致）：`cargo run --release -p game -- --count 500 --seed 20260926`（后台）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），首次探测即就绪；游戏日志横幅
  `[CONFIG] game demo | count=500 seed=20260926 bench_secs=0 vsync=on(AutoVsync)`、
  `[SIM] spawned 500 wanderers | seed=20260926`（`docs/evidence/ts-01/run1-game.log`）
- 原始响应文件：本目录 `ts-01/` 下 `a1-*.json` / `a2-*.txt` / `a3-*.txt` / `a4-*.json`；大响应（500 行 query）存原文文件，此处摘录首末行

## 断言 1：`world.query` Wanderer 计数 == 500 —— PASS

命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":101,"params":{"data":{"components":["game::sim::Wanderer"]}}}'
```

响应摘录（原文 500 行，全量见 `ts-01/a1-query-wanderer.raw.json`；`result` 为数组）：

```json
首行: {"components":{"game::sim::Wanderer":{"index":0,"origin":[59.20360565185547,0.800000011920929,40.68254470825195],"phase":0.6825083494186401}},"entity":4294966885}
末行: {"components":{"game::sim::Wanderer":{"index":499,"origin":[-85.36661529541016,0.800000011920929,45.38034439086914],"phase":5.907473087310791}},"entity":4294966386}
```

判定：`result` 数组长度 == 500（node 计数 `result_len=500`，无 `error` 字段）。**通过**。

## 断言 2：`world.get_resources` SimConfig 回读 —— PASS

命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":102,"params":{"resource":"game::sim::SimConfig"}}'
```

响应原文：

```json
{"jsonrpc":"2.0","id":102,"result":{"value":{"bench_secs":0.0,"entity_count":500,"paused":false,"seed":20260926}}}
```

判定：`entity_count == 500` 且 `seed == 20260926` 且 `paused == false`。**通过**。

## 断言 3：`world.get_resources` SimStats tick 推进 —— PASS

命令原文（两次，间隔 2s ≥ 1s）：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":103,"params":{"resource":"game::sim::SimStats"}}'
sleep 2
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":104,"params":{"resource":"game::sim::SimStats"}}'
```

响应原文（两次完整响应）：

```json
读1: {"jsonrpc":"2.0","id":103,"result":{"value":{"avg_fps":88.17662026638628,"elapsed_secs":46.07029781979509,"fps_1s":60.12245268067289,"frame_count":4142,"tick":4142}}}
读2: {"jsonrpc":"2.0","id":104,"result":{"value":{"avg_fps":86.8955234848406,"elapsed_secs":48.170046421000734,"fps_1s":60.11129655829322,"frame_count":4268,"tick":4268}}}
```

> **勘误（2026-09-26，M1 独立审计发现）**：读 2 的 `fps_1s` 摘录初版写作 `60.11129322958322`（末段数字誊写失真），raw（`ts-01/a3-stats-tick2.raw.txt`）为 `60.11129655829322`，已改正。tick 判定不受影响。

判定：`tick1 = 4142 > 0`；间隔 2s 后 `tick2 = 4268 > tick1`。**通过**。

## 断言 4：`world.query` Wanderer + option Velocity，行内双组件与初值确定性 —— PASS

请求形态依据：`params.data.option` 字段核实于本地源码 `bevy_remote-0.19.1/src/builtin_methods.rs:383-396`
（`BrpQuery { components, option: ComponentSelector, has }`；`ComponentSelector::Paths(Vec<String>)`
`builtin_methods.rs:840-871`）——清单写法与 0.19.1 源码一致，未放宽。

命令原文（进程 1）：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":105,"params":{"data":{"components":["game::sim::Wanderer"],"option":["game::sim::Velocity"]}}}'
```

响应摘录（全量见 `ts-01/a4-query-wanderer-velocity-run1.raw.json`）：

```json
index=0 行: components 同时含 "game::sim::Wanderer" 与 "game::sim::Velocity"，
            Velocity.linear = [1.8184853792190552, 0, 0.6655197739601135]
```

- 主句判定：500 行中 500 行同时含 Wanderer 与 Velocity 值（`rows_with_both=500`）。**通过**。
- 初值确定性（清单括注「同 seed 下 `linear` 向量跨进程一致」）：杀进程后以**完全相同参数**
  （`--count 500 --seed 20260926`）二次启动（进程 2，日志 `ts-01/run2-game.log`），重发同一命令
  （`id:201`，响应存 `ts-01/a4-query-wanderer-velocity-run2.raw.json`），按 `Wanderer.index`
  对齐后逐位比较两进程全部 500 个 `linear` 向量：
  `compared_indices=500, mismatches=0`（index 0 两进程均为
  `[1.8184853792190552, 0, 0.6655197739601135]`；比较脚本内联于会话，提取数据存
  `ts-01/a4-linear-run1.json`）。**通过**。
  说明：进程 2 是断言 4 括注的**跨进程对照测量**（同 seed 第二次启动），非断言失败后的重跑；
  断言 1–4 在进程 1 首次执行即全部通过，其结果未被重跑替换。

## 汇总与口径说明

- 结论：**4/4 断言首次执行全部通过，一次通过，返工 0**。
- 通用前置：`cargo check --workspace` REAL_EXIT=0（本任务零改码，仍按 taskset README 公共前置执行；
  日志 `ts-01/cargo-check.log`）。
- 游戏收尾：两次运行均以 `taskkill /F /IM game.exe` 终止，run 日志末行 `REAL_EXIT=1`
  为强杀的预期退出码（非游戏故障）；收尾确认无 `game.exe` 残留。
- 与清单口径的偏差：无。补充两点测量事实——①断言 3 的 tick 初值 4142 是就绪探测后
  首次断言时刻的真实值（进程存活约 46s，期间在跑就绪轮询与断言 1/2），不违反
  「tick > 0」口径；②`avg_fps`/`fps_1s` 字段与本任务断言无关，仅随原文一并留痕。

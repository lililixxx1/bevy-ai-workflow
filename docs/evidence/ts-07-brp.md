# TS-07 断言证据：远程改写实体运动参数（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-07-mutate-velocity.md`（4 条断言，逐条执行，含第 4 条数学复算断言，容差 0.1）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo，本次**零改码核对型任务**（任务需求即「不改动代码」）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 100 --seed 20260926`；
  实际执行 `cargo build --release -p game`（REAL_EXIT=0，`cargo-build-release.exit`）后直接运行同一产物
  `./target/release/game.exe --count 100 --seed 20260926`（后台）——与 `cargo run --release` 构建并运行的
  是同一二进制同一参数，口径等价（TS-01~06 同款注记）。横幅：
  `[CONFIG] game demo | count=100 seed=20260926 max_speed=3 bench_secs=0 vsync=on(AutoVsync) | bevy 0.19 (workspace locked)`（`ts-07/run1-game.log`）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`；向量按 `[x,y,z]` 数组形态传）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），1 次即就绪（`ts-07/ready-discover.json`）
- 原始文件：本目录 `ts-07/` 下各 `a*.raw.json` / `run1-game.log` / `run1-game.exit` /
  `cargo-check.*` / `cargo-build-release.*`
- 数学参考值独立复算：node（f64，`Math.sin`）按 `game/src/sim.rs:229-232` 公式独立实现计算，
  **未抄 demo 日志/响应中的位置值充当参考**（PIT-M-002 口径）；demo 侧为 f32，f32/f64 差异 ≪ 容差

## 门禁（公共前置）

- `cargo check --workspace`：**REAL_EXIT=0**（`ts-07/cargo-check.exit`；增量 0.60s；零改码任务仍按公共前置执行）
- `cargo build --release -p game`：REAL_EXIT=0（断言所用即此产物，源码零变更）

## 请求形态查证（写断言前，非事后）

- `world.mutate_components` 参数结构 `BrpMutateComponentsParams { entity: Entity, component: String,
  path: String, value: Value }`（`bevy_remote-0.19.1/src/builtin_methods.rs:286-302`）——清单
  `entity` / `component="game::sim::Velocity"` / `path="linear"` / `value=[0.0,0.0,0.0]` 与源码一致；
  handler `process_remote_mutate_components_request`（`:1164-1221`）经 `reflect_path(path)` 定位字段、
  `TypedReflectDeserializer` 按 `Velocity.linear` 的类型（`Vec3`）反序列化 value——即 glam serde 的
  `[x,y,z]` 数组形态（TS-05 / taskset README 注记），传对象会报 `expected a sequence of 3 f32 values`。
- 成功响应 `Ok(Value::Null)`（`:1220-1221`）即 `result: null`；实体/组件/路径任一不存在均走
  `component_error`（`error.code = -23402`），断言 3 只需「无 error」。
- `world.get_resources` 单资源形态：`params={"resource":"..."}`，响应 `result.value`（TS-04 已核实
  `builtin_methods.rs:303-316`；本次实测 `{"value":{...}}` 一致）。
- `world.get_components` / `world.query` / Entity 位串数字回填：TS-05/TS-06 已核实并实测（本次同形态复用）。
- `Transform` 全路径 `bevy_transform::components::transform::Transform`（`docs/brp-smoke.md` 实测形态结论 4）。

## 断言 1：`world.query` Wanderer+Velocity 取 index==0 行 —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":701,"params":{"data":{"components":["game::sim::Wanderer","game::sim::Velocity"]}}}'
```

响应原文摘录（全量 100 行见 `ts-07/a1-query-wanderer-velocity.raw.json`，index==0 行原文）：

```json
{"components":{"game::sim::Velocity":{"linear":[1.8184853792190552,0,0.6655197739601135]},"game::sim::Wanderer":{"index":0,"origin":[59.20360565185547,0.800000011920929,40.68254470825195],"phase":0.6825083494186401}},"entity":4294966884}
```

判定：`result` 数组行数 **100**，无 `error`；按 `index === 0` 检索得唯一行：
**e = 4294966884**，`origin = [59.20360565185547, 0.800000011920929, 40.68254470825195]`，
`phase = 0.6825083494186401`，`linear_v0 = [1.8184853792190552, 0, 0.6655197739601135]`。
（origin/phase 与 TS-05/TS-06 存档同 seed 同 count 逐位一致——跨进程确定性侧证，非清单断言。）**通过**。

## 断言 2：`world.get_resources` SimStats 记 t1 —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":702,"params":{"resource":"game::sim::SimStats"}}'
```

响应原文（全量，`ts-07/a2-stats-t1.raw.json`）：

```json
{"jsonrpc":"2.0","id":702,"result":{"value":{"avg_fps":95.0255032351912,"elapsed_secs":37.093132213223726,"fps_1s":59.93656912889091,"frame_count":3591,"tick":3591}}}
```

判定：无 `error`；**t1 = elapsed_secs = 37.093132213223726**（tick=3591）。**通过**。

## 断言 3：`world.mutate_components` 归零 Velocity.linear 响应无 error —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.mutate_components","id":703,"params":{"entity":4294966884,"component":"game::sim::Velocity","path":"linear","value":[0.0,0.0,0.0]}}'
```

响应原文（全量，`ts-07/a3-mutate-velocity.raw.json`）：

```json
{"jsonrpc":"2.0","id":703,"result":null}
```

判定：无 `error`；`result: null` 与源码 `Ok(Value::Null)`（`builtin_methods.rs:1220-1221`）一致；
value 按 `[0.0,0.0,0.0]` Vec3 数组形态传入一次成功（无 `-23402` 序列化拒绝）。**通过**。

## 断言 4：t2 + Transform 数学复算（容差 0.1）—— PASS

按清单顺序**先取 t2、背靠背再取 Transform**（两次请求间无额外动作，最小化帧边界偏移）。
命令原文（顺序执行）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":704,"params":{"resource":"game::sim::SimStats"}}'
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_components","id":705,"params":{"entity":4294966884,"components":["bevy_transform::components::transform::Transform"],"strict":true}}'
```

响应原文（全量，`ts-07/a4-stats-t2.raw.json` / `a4-transform.raw.json`）：

```json
{"jsonrpc":"2.0","id":704,"result":{"value":{"avg_fps":78.85256592245521,"elapsed_secs":67.20759729179554,"fps_1s":59.97970915937358,"frame_count":5398,"tick":5398}}}
{"jsonrpc":"2.0","id":705,"result":{"bevy_transform::components::transform::Transform":{"rotation":[0.0,0.0,0.0,1.0],"scale":[1.0,1.0,1.0],"translation":[59.20360565185547,1.5426182746887207,40.68254470825195]}}}
```

独立复算（node f64，公式 `p(t) = origin + linear*t + Y*0.8*sin(2π*0.7*t + phase)`，取自
`game/src/sim.rs:229-232`；`linear` 已归零 ⇒ 漂移项消失；参考向量由公式+断言 1 的 origin/phase + 本条
实测 t2 现算，非抄任何现成值）：

```text
t2       = 67.20759729179554
actual   = [59.20360565185547, 1.5426182746887207, 40.68254470825195]
expected = [59.20360565185547, 1.4586626098243025, 40.68254470825195]
           （expected.y = 0.800000011920929 + 0.8*sin(2π*0.7*67.20759729179554 + 0.6825083494186401)）
abs_err  = [0, 0.08395566486441819, 0]
norm_err = 0.083956
```

判定：三分量 |Δ| 均 ≤ 0.1（x/z **恰为 0**——速度归零后水平漂移项精确消失，位置回到
`origin.x`/`origin.z` 逐位相同）；‖Δ‖ = **0.083956 ≤ 0.1**。y 分量 0.0840 的残差来自 t2 读取与
Transform 读取之间的 O(1) 帧边界偏移（y 项斜率上限 0.8·2π·0.7 ≈ 3.517 m/s，0.084/3.517 ≈ 23.9ms
≈ 1-2 帧@60fps，与 `docs/brp-smoke.md` 形态结论 3 的 mutate/读取帧边界竞态同源，非公式失准）。**通过**。

## 非清单侧证：注入值驻留（Velocity 回读）

命令原文与响应原文（全量，`ts-07/a5-velocity-readback.raw.json`，断言 4 完成后执行）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_components","id":706,"params":{"entity":4294966884,"components":["game::sim::Velocity"],"strict":true}}'
```

```json
{"jsonrpc":"2.0","id":706,"result":{"game::sim::Velocity":{"linear":[0.0,0.0,0.0]}}}
```

`linear` 严格 `=== [0.0,0.0,0.0]`：断言 3 的状态注入写入并驻留（写 → 存 → 读闭环）。

## 汇总与口径说明

- 结论：**4/4 断言首次执行全部通过；一次通过，返工 0；零改码**（`cargo check` 仍按公共前置执行且 REAL_EXIT=0）。
- 与清单口径的偏差：**无**。断言未放宽；启动命令、请求形态（Vec3 `[x,y,z]` 数组、`path:"linear"`、
  `strict:true`）、判定顺序（先 t2 后 Transform）、容差 0.1 均按清单原文执行。数学参考值独立复算，
  未使用 demo 输出值充当参考（PIT-M-002 纪律）。
- 工具级注记（非断言失败，原始响应全程完好）：断言 2 首次提取时我的 node 取值表达式按
  `result['game::sim::SimStats']` 写（get_resources 单资源实为 `result.value`），报 TypeError 后改
  正确路径提取——curl 请求/响应本身一次成功，不构成任务返工（TS-06 同款「工具级偏差」记录口径）。
- 断言 1~4 间存在分钟级墙钟间隔（t1=37.09s → t2=67.21s）：清单未规定断言执行节奏，间隔只影响
  t 的取值不影响公式（位置是 t 的解析函数）；断言 4 内部两次读取严格背靠背（见上）。
- 游戏收尾：`taskkill /F /IM game.exe`（PID 15856）终止，`tasklist | grep game.exe` 确认无残留；
  `run1-game.exit` REAL_EXIT=1 为强杀预期退出码（非游戏故障，TS-01~06 同款注记）。
- 无代码/行为/接口变更，无文档同步义务触发；零踩坑（mutate_components 请求形态按 §2 源码查证后
  一次写对；数学断言独立复算流程与 brp-smoke #11 口径一致）。

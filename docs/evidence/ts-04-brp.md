# TS-04 断言证据：远程暂停/恢复模拟（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-04-pause-resume.md`（5 条断言，逐条执行）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo，本次**零改码核对型任务**（验证既有 `SimConfig.paused` 语义的 BRP 操控闭环）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 1000 --seed 20260926`；
  实际执行 `cargo build --release -p game`（REAL_EXIT=0）后直接运行同一产物
  `./target/release/game.exe --count 1000 --seed 20260926`（后台）——与 `cargo run --release` 构建并运行的
  是同一二进制同一参数，口径等价（TS-01/02/03 同款注记）。横幅：
  `[CONFIG] game demo | count=1000 seed=20260926 max_speed=3 bench_secs=0 vsync=on(AutoVsync)`（`ts-04/run1-game.log`）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），首次轮询即就绪（`ts-04/ready-discover.json`）
- 原始文件：本目录 `ts-04/` 下各 `a*.raw.json` / `a1-timing.txt`（采样时刻戳）/ `run1-game.log` /
  `run1-game.exit` / `cargo-check.*` / `cargo-build-release.*`
- **全程单一游戏进程**（PID 8484）：暂停与恢复全部经 BRP `world.mutate_resources` 完成，
  无重启进程、无改码（任务约束满足）

## 门禁（公共前置）

- `cargo check --workspace`：**REAL_EXIT=0**（`ts-04/cargo-check.exit`；增量 0.59s；零改码任务仍按公共前置执行）
- `cargo build --release -p game`：REAL_EXIT=0（断言所用即此产物，源码与 TS-03 提交一致无变更）

## 请求形态查证（写断言前，非事后）

`world.mutate_resources` 参数结构 `BrpMutateResourcesParams { resource: String, path: String,
value: Value }`，成功响应为 `result: null`（`bevy_remote-0.19.1/src/builtin_methods.rs:303-316` 结构体、
`:1250-1255` 解构、handler 末尾 `Ok(Value::Null)`；核实 2026-09-26）——与清单「`resource` / `path` /
`value`」写法一致，断言「响应无 error」即核验响应不含 `error` 字段且 `result` 为 null。

## 采样间隔证明（`ts-04/a1-timing.txt` 时间戳，单位 epoch 秒）

| 断言 | 配对采样点 | 间隔 |
|---|---|---|
| 1 | T1_START 1790429343.316 → T2_END 1790429344.931 | 1.615s（≥1s） |
| 3 | T3_START 1790429391.348 → T4_END 1790429392.964 | 1.616s（≥1s） |
| 4 | Q1_DONE 1790429414.421 → Q2_DONE 1790429416.038 | 1.616s（≥1s） |
| 5 | T5_START 1790429434.065 → T6_END 1790429435.681 | 1.616s（≥1s） |

（断言之间的更大空隙是执行 agent 两次工具调用间的处理延迟，不影响断言本身。）

## 断言 1：运行中——`tick2 > tick1` —— PASS

命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":401,"params":{"resource":"game::sim::SimStats"}}'
# ≥1s 后同方法 id=402
```

响应原文（`ts-04/a1-tick1.raw.json` / `a1-tick2.raw.json`）：

```json
{"jsonrpc":"2.0","id":401,"result":{"value":{"avg_fps":107.57026294280392,"elapsed_secs":25.73116430779919,"fps_1s":59.9964062152677,"frame_count":2822,"tick":2822}}}
{"jsonrpc":"2.0","id":402,"result":{"value":{"avg_fps":104.53873621922529,"elapsed_secs":27.34729449870065,"fps_1s":60.03063727540422,"frame_count":2919,"tick":2919}}}
```

判定：`tick1=2822 → tick2=2919`，`tick2 > tick1`（间隔 1.615s，≈60 fps 推进 97 tick）。
**通过**。

## 断言 2：BRP 置 `paused=true` 响应无 error —— PASS

命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.mutate_resources","id":403,"params":{"resource":"game::sim::SimConfig","path":"paused","value":true}}'
```

响应原文（全量，`ts-04/a2-mutate-pause.raw.json`）：

```json
{"jsonrpc":"2.0","id":403,"result":null}
```

判定：无 `error` 字段，`result:null` 与源码预期一致。**通过**。

## 断言 3：已冻结——`tick4 - tick3 <= 2` —— PASS

命令原文：同断言 1 的 `world.get_resources`（id=404 / 405，间隔 1.616s）。

响应原文（`ts-04/a3-tick3.raw.json` / `a3-tick4.raw.json`）：

```json
{"jsonrpc":"2.0","id":404,"result":{"value":{"avg_fps":75.73172392258785,"elapsed_secs":57.76350509049371,"fps_1s":59.986053242621075,"frame_count":5704,"tick":4744}}}
{"jsonrpc":"2.0","id":405,"result":{"value":{"avg_fps":75.38619994559691,"elapsed_secs":57.76350509049371,"fps_1s":60.026897954568675,"frame_count":5801,"tick":4744}}}
```

判定：`tick4 - tick3 = 4744 - 4744 = 0 <= 2`（容差 2 帧内；本次差 0）。旁证三重：
`elapsed_secs` 两采样逐位相同（57.76350509049371，模拟时间冻结）；`frame_count` 5704→5801
照常推进（渲染循环未停，停的只是模拟）；`fps_1s` 仍 ≈60。**通过**。

## 断言 4：暂停期间实体 Transform 逐位不变 —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":406,"params":{"data":{"components":["game::sim::Wanderer","bevy_transform::components::transform::Transform"]}}}'
# ≥1s 后同方法 id=407
```

响应原文摘录（全量各 1000 行见 `ts-04/a4-query1.raw.json` / `a4-query2.raw.json`，各 342816 字节；
`result` 数组首行原文）：

```json
{"components":{"bevy_transform::components::transform::Transform":{"rotation":[0.0,0.0,0.0,1.0],"scale":[1.0,1.0,1.0],"translation":[164.24569702148438,0.5861022472381592,79.12529754638672]},"game::sim::Wanderer":{"index":0,"origin":[59.20360565185547,0.800000011920929,40.68254470825195],"phase":0.6825083494186401}},"entity":4294966884}
```

判定（node 逐行严格 `===` 比较两次采样的 `translation` 三分量，输出原文）：
`rows: 1000 1000 | bitwise-same: 1000 diff: 0`——两次采样（间隔 1.616s）**全部 1000 个实体**
Transform.translation 逐位相同，首个一致实体即上引首行（entity 4294966884，index 0，
translation `[164.24569702148438,0.5861022472381592,79.12529754638672]`）。
远超「任一实体相同」的清单要求。**通过**。

## 断言 5：恢复后 tick 恢复增长 —— PASS

命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.mutate_resources","id":408,"params":{"resource":"game::sim::SimConfig","path":"paused","value":false}}'
# 随后 world.get_resources SimStats（id=409 / 410，间隔 1.616s）
```

响应原文（`ts-04/a5-mutate-resume.raw.json` / `a5-tick5.raw.json` / `a5-tick6.raw.json`）：

```json
{"jsonrpc":"2.0","id":408,"result":null}
{"jsonrpc":"2.0","id":409,"result":{"value":{"avg_fps":69.8611741684588,"elapsed_secs":57.86490469099954,"fps_1s":59.78262439942127,"frame_count":8267,"tick":4750}}}
{"jsonrpc":"2.0","id":410,"result":{"value":{"avg_fps":69.7244393172734,"elapsed_secs":59.480607588309795,"fps_1s":60.05197744762752,"frame_count":8364,"tick":4847}}}
```

判定：恢复 mutate 无 error；`tick5=4750 → tick6=4847`（间隔 1.616s，+97 tick）恢复增长，
`elapsed_secs` 同步推进（57.865 → 59.481）。收尾旁证：`world.get_resources` SimConfig
（`a5-simconfig-final.raw.json`）回读 `paused:false`（`{"bench_secs":0.0,"entity_count":1000,"max_speed":3.0,"paused":false,"seed":20260926}`）。
冻结→恢复的 tick 连续性自洽：断言 3 冻结时 tick=4744，暂停期间保持 4744，恢复后从 4745 起继续
（tick5=4750 = 恢复后 6 帧）。**通过**。

## 汇总与口径说明

- 结论：**5/5 断言首次执行全部通过；一次通过，返工 0；零改码**。
- 与清单口径的偏差：**无**。断言未放宽；启动命令、请求形态、容差（断言 3 容差 2 帧，实测差 0）均按清单原文执行。
- 游戏收尾：`taskkill /F /IM game.exe`（PID 8484）终止，`tasklist | grep game.exe` 确认无残留；
  `run1-game.exit` REAL_EXIT=1 为强杀预期退出码（非游戏故障，TS-01/02/03 同款注记）。
- 无行为/接口变更，无文档同步义务触发；零踩坑（`mutate_resources` 参数形态按 §2 查证后一次写对）。

# TS-10 断言证据：事件驱动的状态变更（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-10-trigger-event.md`（4 条断言，逐条执行）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo，**改码任务**（新增 `game::sim::PauseRequested` 事件 + observer 翻转
  `SimConfig.paused`，唯一改码文件 `game/src/sim.rs`）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 100 --seed 20260926`；
  实际执行 `cargo build --release -p game`（REAL_EXIT=0，`ts-10/cargo-build-release.*`）后直接运行同一产物
  `./target/release/game.exe --count 100 --seed 20260926`（后台）——与 `cargo run --release` 构建并运行的
  是同一二进制同一参数，口径等价（TS-01..09 同款注记）。横幅：
  `[CONFIG] game demo | count=100 seed=20260926 max_speed=3 bench_secs=0 vsync=on(AutoVsync)`
  （`ts-10/run1-game.log`）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），约 2s 即就绪
  （`ts-10/ready-discover.json`，含 `world.trigger_event` 方法项）
- 原始文件：本目录 `ts-10/` 下各 `a*.raw.json` / `a3-timing.txt` / `run1-game.log` /
  `run1-game.exit` / `cargo-check-first.*` / `cargo-test-game.*` / `cargo-build-release.*`
- **全程单一游戏进程**（PID 22332）：两次事件触发全部经 BRP `world.trigger_event` 完成
  （任务约束「trigger_event 是唯一允许的触发通道」满足），无重启进程
- 游戏收尾：`taskkill /F /IM game.exe`（PID 22332）终止，`tasklist | grep game.exe` 确认无残留；
  `run1-game.exit` REAL_EXIT=1 为强杀预期退出码（非游戏故障，TS-01..09 同款注记）

## 门禁（公共前置 + 改码任务口径）

- **首次** `cargo check --workspace`（改码后第一次，计入「一次通过」口径）：**REAL_EXIT=0**
  （`ts-10/cargo-check-first.exit`；增量 1.52s）
- `cargo test -p game`：REAL_EXIT=0，**11/11** 通过（新增单测
  `pause_requested_event_flips_paused` 首次运行即过；原 10 项不受影响）
- `cargo build --release -p game`：REAL_EXIT=0（9.01s）
- 终态复验（证据文件成文前发现一处**注释级行号引用失准**——`Ok(Value::Null)` 引 1516 应为
  1514、handler 区间末行 1517 应为 1516——修正属纯注释改动，按 T007 先例以最终源码重跑
  check + test + build 全绿（`ts-10/cargo-check-final.*` / `cargo-test-game-final.*`（11/11）/
  `cargo-build-release-final.*`，均 REAL_EXIT=0），并用**最终二进制**完整重跑 4/4 断言
  （run2，见下节）确认断言产物与最终源码一致。修正发生在首次 check 之后、不影响任何判定，
  首次 check 与首次 demo 断言记录不因此作废。

## 请求形态查证（写码/写断言前，非事后）

- `world.trigger_event` 参数结构 `BrpTriggerEventParams { event: String, value: Option<Value> }`
  （`bevy_remote-0.19.1/src/builtin_methods.rs:327-333`）；`value` 可省——handler 对空载荷以
  `DynamicStruct::default()`（0 字段）经 `ReflectFromReflect::from_reflect` 构造后
  `ReflectEvent::trigger`（`builtin_methods.rs:1481-1516`；默认 trigger fn =
  `from_reflect_with_fallback::<E>` + `world.trigger`，`bevy_ecs-0.19.1/src/reflect/event.rs:125-137`）。
  与清单「`value` 形态以实测/schema 为准，可空」一致，本次断言 2/4 均省略 `value`。
- handler 校验面：类型未注册报 `Unknown event type: ...`（`builtin_methods.rs:1492`）、
  缺 `ReflectEvent` 数据报 `Event ... is not reflectable`（`:1497`）——本类型
  `register_type::<PauseRequested>()` + `#[reflect(Event, Serialize, Deserialize)]` 双条件满足。
- observer 形态（首次使用前查证）：`on_pause_requested(_: On<PauseRequested>, mut config: ResMut<SimConfig>)`，
  `On<E>` + `ResMut` 组合逐字对齐官方示例 `bevy-0.19.1/examples/ecs/observers.rs:142`（`on_add_mine`）；
  注册 `App::add_observer`（`bevy_app-0.19.1/src/app.rs:1474`）；`World::trigger` 同步运行
  observer（`bevy_ecs-0.19.1/src/observer/mod.rs:63`）——故翻转在 HTTP 响应返回前已生效。

## 断言 1：初始态 `paused == false` —— PASS

命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":1001,"params":{"resource":"game::sim::SimConfig"}}'
```

响应原文（全量，`ts-10/a1-simconfig-initial.raw.json`）：

```json
{"jsonrpc":"2.0","id":1001,"result":{"value":{"bench_secs":0.0,"entity_count":100,"max_speed":3.0,"paused":false,"seed":20260926}}}
```

判定：`paused == false`（node 严格 `===`），且 `entity_count==100` / `seed==20260926` 与启动参数一致。**通过**。

## 断言 2：`world.trigger_event` 触发 `PauseRequested` 响应无 error —— PASS

命令原文（`value` 省略，unit 事件无载荷）：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.trigger_event","id":1002,"params":{"event":"game::sim::PauseRequested"}}'
```

响应原文（全量，`ts-10/a2-trigger-1.raw.json`）：

```json
{"jsonrpc":"2.0","id":1002,"result":null}
```

判定：无 `error` 字段，`result:null` 与源码 `Ok(Value::Null)`（`builtin_methods.rs:1514`）一致。**通过**。

## 断言 3：等待 ≥0.5s 后 `paused == true`（事件副作用已生效）—— PASS

命令原文：同断言 1 的 `world.get_resources`（id=1003）；触发（断言 2 完成）与回读之间
`sleep 1`——间隔实测 1.123s（`ts-10/a3-timing.txt` 时间戳 1790433749.444 → 1790433750.567，
epoch 秒，覆盖 sleep + 请求往返），≥0.5s 满足。

响应原文（全量，`ts-10/a3-simconfig-after-trigger1.raw.json`）：

```json
{"jsonrpc":"2.0","id":1003,"result":{"value":{"bench_secs":0.0,"entity_count":100,"max_speed":3.0,"paused":true,"seed":20260926}}}
```

判定：`paused == true`——事件经 observer 翻转已生效。旁证：游戏日志
（`ts-10/run1-game.log:32`）`INFO game::sim: [SIM] PauseRequested observed -> paused=true`
（14:42:17.244，与请求时刻一致）。**通过**。

## 断言 4：再触发一次 `paused == false`（翻转语义）—— PASS

命令原文：`world.trigger_event`（id=1004，同断言 2 形态）+ `sleep 1` +
`world.get_resources`（id=1005，同断言 1 形态）。

响应原文（全量，`ts-10/a4-trigger-2.raw.json` / `a4-simconfig-after-trigger2.raw.json`）：

```json
{"jsonrpc":"2.0","id":1004,"result":null}
{"jsonrpc":"2.0","id":1005,"result":{"value":{"bench_secs":0.0,"entity_count":100,"max_speed":3.0,"paused":false,"seed":20260926}}}
```

判定：第二次触发无 error，`paused` 回 `false`——翻转语义成立（false→true→false 两步闭环）。
旁证：`run1-game.log:63` `[SIM] PauseRequested observed -> paused=false`（14:42:47.542）。**通过**。

## 汇总与口径说明

- 结论：**4/4 断言首次执行全部通过；一次通过，返工 0；改码（唯一改码文件 `game/src/sim.rs`）**。
- 终态复验（run2，最终二进制 `--count 100 --seed 20260926`，PID 18832）：同一 4 断言序列
  重跑结果与 run1 逐条一致（`paused` false → 触发 null → true → 触发 null → false；
  `ts-10/run2-a*.raw.json`、`run2-game.log` 两条 observer 日志）——断言产物与最终源码
  （含行号修正后的注释）一致。run2 是一致性复验而非重跑挑结果（run1 首次即全过）。

## 与清单口径的偏差

- **无**。断言未放宽；启动命令、请求形态（`value` 省略）、≥0.5s 等待（实测 1.123s）
  均按清单原文或清单明示的「以实测/schema 为准」执行。
- 行为/接口变更的文档同步：`game/src/sim.rs` 模块注释已同步（暂停通道补 `PauseRequested`
  事件；反射注册类型清单「六个」→「七个」）；`docs/brp-smoke.md` 系带日期首役实录不追改
  （T008 先例）。
- 工具级偏差 1（非返工，门禁运行前自查发现并修正）：编辑器字符串替换时误吞
  `max_speed_only_scales_speed_draw_order_untouched` 测试的三层收尾花括号（old_string 界定
  含文件尾部），下一编辑即修复；发生在首次 `cargo check` 之前，未产生任何失败的门禁/断言记录。
- 无新踩坑入库（observer/事件 API 全部按 §2.1 查证后一次写对；`world.trigger_event`
  空载荷通路与源码推演一致）。

# TS-03 断言证据：新增 BRP 可见 Resource SimMetadata（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-03-new-resource.md`（2 条断言，逐条执行）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo，本次**改码任务**（新增 `game::sim::SimMetadata` Resource：`version`/`entity_count`/`seed`，Startup 一次性填充，此后只读）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 250 --seed 7`；
  实际执行 `cargo build --release -p game`（REAL_EXIT=0）后直接运行同一产物
  `./target/release/game.exe --count 250 --seed 7`（后台）——与 `cargo run --release` 构建并运行的
  是同一二进制同一参数，口径等价（TS-02 同款注记）。横幅：
  `[CONFIG] game demo | count=250 seed=7 max_speed=3 bench_secs=0 vsync=on(AutoVsync)`（`ts-03/run1-game.log`）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），首次轮询即就绪（`ts-03/ready-discover.json`，
  返回 `{"info":{"title":"Bevy Remote Protocol","version":"0.19.1"},...}`）
- 原始文件：本目录 `ts-03/` 下 `a1-list-resources.raw.json` / `a2-get-simmetadata.raw.json` /
  `run1-game.log` / `run1-game.exit` 及各 cargo 门禁日志（`cargo-check-first.*` / `cargo-test-game.*` /
  `cargo-build-release.*`）

## 门禁（一次通过口径的关键测量点）

- **首次 `cargo check --workspace`（改码完成后）：REAL_EXIT=0**（`ts-03/cargo-check-first.exit`；
  增量 1.46s，`Finished dev profile`，覆盖全部改动——仅 `game/src/sim.rs` 一个文件）。
- `cargo test -p game`：REAL_EXIT=0，**10 passed / 0 failed**（既有单测无一破坏）。
- `cargo build --release -p game`：REAL_EXIT=0（断言所用即此产物）。

## 实现口径（与任务约束逐条对应）

- 字段：`version: String`（编译期 `env!("CARGO_PKG_VERSION")` → "0.1.0"，即 `game/Cargo.toml` 的
  version）/ `entity_count: u32` / `seed: u64`；Startup 系统一次性填充，此后无任何系统写入
  （「不引入运行时可变状态」满足——全插件仅 `init_metadata` 一个写者，且仅 Startup 运行）。
- derive 形态逐字对齐官方 BRP 示例 Resource（`bevy-0.19.1/examples/remote/server.rs:68-70`）：
  `#[derive(Resource, Reflect, Serialize, Deserialize, ...)]` +
  `#[reflect(Resource, Serialize, Deserialize)]`；插件 build 中显式
  `init_resource::<SimMetadata>()` + `register_type::<SimMetadata>()`（SKILL.md §3.4 强制形态）。
- 响应形态查证（写断言前，非事后）：`world.list_resources` 的 `result` 为排序后的
  `Vec<String>`（有 `ReflectResource` 数据的已注册类型全路径，`bevy_remote-0.19.1/src/
  builtin_methods.rs:1413-1432`，`BrpListResourcesResponse = Vec<String>` 于 `:520`）——与实测一致。

## 断言 1：`world.list_resources` 结果数组包含 `"game::sim::SimMetadata"` —— PASS

命令原文（清单要求「无 params」，请求体即不含 params 字段）：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.list_resources","id":301}'
```

响应原文摘录（全量见 `ts-03/a1-list-resources.raw.json`；`result` 共 45 项，排序数组，此处摘首尾与目标项）：

```json
{"jsonrpc":"2.0","id":301,"result":["bevy_a11y::AccessibilityRequested", … ,"game::camera::CameraRig","game::sim::SimConfig","game::sim::SimMetadata","game::sim::SimStats"]}
```

判定（node 严格断言输出原文）：`A1: isArray=true | contains game::sim::SimMetadata = true | total=45`。
**通过**。

## 断言 2：`world.get_resources` SimMetadata 三字段值 —— PASS

命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":302,"params":{"resource":"game::sim::SimMetadata"}}'
```

响应原文（全量，`ts-03/a2-get-simmetadata.raw.json`）：

```json
{"jsonrpc":"2.0","id":302,"result":{"value":{"entity_count":250,"seed":7,"version":"0.1.0"}}}
```

判定（node 严格断言输出原文）：
`A2: version="0.1.0" (=== "0.1.0": true) | entity_count=250 (===250: true) | seed=7 (===7: true)`。
三字段全中：`version == "0.1.0"`（字符串相等，非前缀）、`entity_count == 250`（严格 ===）、`seed == 7`。**通过**。

运行日志侧旁证（`ts-03/run1-game.log`，ANSI 剥离后）：
`INFO game::sim: [SIM] metadata ready | version=0.1.0 | entity_count=250 | seed=7`（Startup 填充实际发生）。

## 汇总与口径说明

- 结论：**2/2 断言首次执行全部通过；一次通过，返工 0**。首次 `cargo check --workspace` REAL_EXIT=0。
- 游戏收尾：`taskkill /F /IM game.exe`（PID 16576）终止，`tasklist | grep game.exe` 确认无残留；
  `run1-game.exit` REAL_EXIT=1 为强杀预期退出码（非游戏故障，TS-01/02 同款注记）。
- 与清单口径的偏差：**无**。断言未放宽；启动命令、请求形态、字段值均按清单原文执行。
- 唯一改码文件：`game/src/sim.rs`（新类型 + 插件注册 + Startup 填充系统 + 模块注释「四个类型」→「五个类型」，
  履行行为/接口同步义务；`docs/brp-smoke.md` 为带日期的首役实录而非活文档，无需追改）。

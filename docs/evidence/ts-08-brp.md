# TS-08 断言证据：相机轨道参数远程调整（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-08-camera-rig.md`（4 条断言，逐条执行；容差 0.1）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo，本次**零改码核对型任务**（任务需求即「不改动代码」）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 1000 --seed 20260926`；
  实际执行 `cargo build --release -p game`（REAL_EXIT=0，`ts-08/cargo-build-release.exit`）后直接运行同一产物
  `./target/release/game.exe --count 1000 --seed 20260926`（后台）——与 `cargo run --release` 构建并运行的
  是同一二进制同一参数，口径等价（TS-01~07 同款注记）。横幅：
  `[CONFIG] game demo | count=1000 seed=20260926 max_speed=3 bench_secs=0 vsync=on(AutoVsync) | bevy 0.19 (workspace locked)`（`ts-08/run1-game.log`）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），1 次即就绪（`ts-08/ready-discover.json`，
  `result.info.version == "0.19.1"`）
- 原始文件：本目录 `ts-08/` 下各 `a*.raw.json` / `run1-game.log` / `run1-game.exit` /
  `cargo-check.*` / `cargo-build-release.*`
- 距离口径：断言的「translation 到 XZ 原点距离」= `hypot(x, z)`（node f64 复算）。
  相机轨道是解析函数（`game/src/camera.rs:58-65`：`pos = (R·cos(ω·t+φ), H, R·sin(ω·t+φ))`），
  XZ 距离恒等于 `radius`，与时刻 t 无关——demo 侧 f32 三角函数复数引入的偏差实测 ~1e-7 量级，远小于容差。

## 门禁（公共前置）

- `cargo check --workspace`：**REAL_EXIT=0**（`ts-08/cargo-check.exit`；增量 0.58s；零改码任务仍按公共前置执行）
- `cargo build --release -p game`：REAL_EXIT=0（断言所用即此产物，源码零变更）

## 请求形态查证（写断言前，非事后）

- `world.mutate_resources` 参数结构本次直读本地源码复核：`BrpMutateResourcesParams { resource: String,
  path: String, value: Value }`（`bevy_remote-0.19.1/src/builtin_methods.rs:308-320`，struct 起始 308 行、
  三字段止于 320 行）——清单 `resource="game::camera::CameraRig"` / `path="radius"` / `value=60.0`
  与源码一致。handler `process_remote_mutate_resources_request`（`:1229` 起）经 `reflect_path` 定位字段、
  按 `f32` 反序列化标量 value；成功响应 `result: null`（TS-04 已核实同款形态，本次实测一致）。
- `world.query` 的 `params.data.components` 形态：TS-01 起多次核实（`builtin_methods.rs:383-396`
  ComponentSelector::Paths）并实测复用。
- `Camera` 全路径 `bevy_camera::camera::Camera`、`Transform` 全路径
  `bevy_transform::components::transform::Transform`：taskset README 实测注记 + `docs/brp-smoke.md`
  形态结论 4（本次断言 1 响应键名再证）。

## 断言 1：`world.query` Camera+Transform，恰 1 行，XZ 距离 ≈ 90.0（容差 0.1）—— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":801,"params":{"data":{"components":["bevy_camera::camera::Camera","bevy_transform::components::transform::Transform"]}}}'
```

响应原文摘录（全量见 `ts-08/a1-query-camera-transform.raw.json`，Camera 对象略，Transform 原文）：

```json
{"jsonrpc":"2.0","id":801,"result":[{"components":{"bevy_camera::camera::Camera":{...},"bevy_transform::components::transform::Transform":{"rotation":[-0.22851867973804474,-0.1007450744509697,-0.02378268539905548,0.9680206775665283],"scale":[1.0,1.0,1.0],"translation":[-18.532459259033203,45.0,88.0712661743164]}},"entity":4294966886}]}
```

判定：`result` 数组行数 **恰 1**，无 `error`；translation 为 `[x,y,z]` 数组形态（与清单括注一致）；
XZ 距离 = `hypot(-18.532459259033203, 88.0712661743164)` = **89.999999842972**，
|d−90| = 1.57e-7 ≤ 0.1（残差为 f32 三角函数累积舍入，远小于容差）。**通过**。

## 断言 2：`world.get_resources` CameraRig，radius==90.0 且 height==45.0 —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":802,"params":{"resource":"game::camera::CameraRig"}}'
```

响应原文（全量，`ts-08/a2-get-camerarig.raw.json`）：

```json
{"jsonrpc":"2.0","id":802,"result":{"value":{"angular_speed":0.10000000149011612,"height":45.0,"phase":0.0,"radius":90.0}}}
```

判定：无 `error`；node 严格 `===` 判定 **radius === 90.0、height === 45.0**（`angular_speed` 的
0.10000000149011612 为 f32→f64 序列化形态，非断言项）。与 `CameraRig::default()`
（`game/src/camera.rs:27-36`）一致。**通过**。

## 断言 3：`world.mutate_resources` radius→60.0，响应无 error —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.mutate_resources","id":803,"params":{"resource":"game::camera::CameraRig","path":"radius","value":60.0}}'
```

响应原文（全量，`ts-08/a3-mutate-radius.raw.json`）：

```json
{"jsonrpc":"2.0","id":803,"result":null}
```

判定：无 `error`；`result: null` 与源码成功响应形态一致（TS-04 同款）。**通过**。

## 断言 4：`world.query` 同 #1，XZ 距离 ≈ 60.0（容差 0.1；下一帧起生效，失败则等 0.2s 重查）—— PASS

命令原文（与断言 1 同形，id=804；mutate 后立即执行，**首次查询即通过，0.2s 重查条款未触发**）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":804,"params":{"data":{"components":["bevy_camera::camera::Camera","bevy_transform::components::transform::Transform"]}}}'
```

响应原文摘录（全量见 `ts-08/a4-query-camera-transform.raw.json`，Transform 原文）：

```json
{"bevy_transform::components::transform::Transform":{"rotation":[-0.2987634539604187,0.31090784072875977,0.10363595932722092,0.8962902426719666],"scale":[1,1,1],"translation":[37.15515899658203,45,47.111507415771484]}}
```

判定：行数恰 1，无 `error`；XZ 距离 = `hypot(37.15515899658203, 47.111507415771484)` =
**59.999999758730**，|d−60| = 2.41e-7 ≤ 0.1。相机已在半径 60 的新轨道上（相位自洽侧证：由两次
translation 反解轨道角 θ=atan2(z,x)，θ1=1.778196 → θ2=0.903006，Δθ=5.407995 rad，除以
`angular_speed`=0.1 rad/s 得 54.1s，与断言 1→4 的实际墙钟间隔一致；XZ 距离是 `radius` 的解析
恒等式、与时刻/相位无关，`game/src/camera.rs:58-65`；`orbit_camera` 系统每帧以 `rig.radius` 现算，
BRP 写入的 Resource 值下一帧即被消费，无重试需要）。**通过**。

## 非清单侧证：注入值驻留（CameraRig 回读）

命令原文与响应原文（全量，`ts-08/a5-camerarig-readback.raw.json`，断言 4 完成后执行）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":805,"params":{"resource":"game::camera::CameraRig"}}'
```

```json
{"jsonrpc":"2.0","id":805,"result":{"value":{"angular_speed":0.10000000149011612,"height":45.0,"phase":0.0,"radius":60.0}}}
```

`radius` 严格 `=== 60.0`：断言 3 的状态注入写入并驻留（写 → 存 → 读闭环；`orbit_camera` 只读
`Res<CameraRig>`，不改写 Resource，回读值即注入值）。

## 汇总与口径说明

- 结论：**4/4 断言首次执行全部通过；一次通过，返工 0；零改码**（`cargo check` 仍按公共前置执行且 REAL_EXIT=0）。
- 与清单口径的偏差：**无**。断言未放宽；启动命令（`--count 1000 --seed 20260926`）、组件全路径、
  mutate 三字段形态、容差 0.1 均按清单原文执行；断言 4 的 0.2s 重查条款未触发（首查即过）。
- 游戏收尾：`taskkill /F /PID 6372`（game.exe）终止，`tasklist` 确认无残留；
  `run1-game.exit` REAL_EXIT=1 为强杀预期退出码（非游戏故障，TS-01~07 同款注记）。
- 无代码/行为/接口变更，无文档同步义务触发；零踩坑（mutate_resources 形态按 §2 源码直读复核后
  一次写对；距离断言不依赖时刻 t，规避帧边界竞态面）。

# TS-06 断言证据：BRP 销毁实体（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-06-despawn-entity.md`（5 条断言，逐条执行）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo，本次**零改码核对型任务**（任务需求即「不改动代码」）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 100 --seed 20260926`；
  实际执行 `cargo build --release -p game`（REAL_EXIT=0）后直接运行同一产物
  `./target/release/game.exe --count 100 --seed 20260926`（后台）——与 `cargo run --release` 构建并运行的
  是同一二进制同一参数，口径等价（TS-01~05 同款注记）。横幅：
  `[CONFIG] game demo | count=100 seed=20260926 max_speed=3 bench_secs=0 vsync=on(AutoVsync)`（`ts-06/run1-game.log`）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），1 次即就绪（`ts-06/ready-discover.json`）
- 原始文件：本目录 `ts-06/` 下各 `a*.raw.json` / `run1-game.log` / `run1-game.exit` /
  `cargo-check.*` / `cargo-build-release.*`
- 断言用响应字段均以 node 严格 `===` / `!!` 判定（计数、entity 比较无浮点歧义）

## 门禁（公共前置）

- `cargo check --workspace`：**REAL_EXIT=0**（`ts-06/cargo-check.exit`；增量 0.56s；零改码任务仍按公共前置执行）
- `cargo build --release -p game`：REAL_EXIT=0（断言所用即此产物，源码与 TS-05 提交一致无变更）

## 请求形态查证（写断言前，非事后）

- `world.despawn_entity` 参数结构 `BrpDespawnEntityParams { entity: Entity }`
  （`bevy_remote-0.19.1/src/builtin_methods.rs:186-191`）——清单 `entity=e` 与源码一致；
  handler `get_entity_mut(world, entity)?.despawn()`（`:1334-1344`），成功返回 `Ok(Value::Null)`
  即 `result: null`；实体不存在时经 `get_entity_mut`（`:1811-1818`）返回
  `BrpError::entity_not_found`（`src/lib.rs:1317-1324`，message 形如 `Entity {index}v{gen} not found`）。
- `world.get_components` handler 在组件反射**之前**先 `get_entity(world, entity)?`
  （`builtin_methods.rs:604`）——实体已不存在时同样报 `entity_not_found`，与请求的 components 无关。
- 错误码常量 `ENTITY_NOT_FOUND = -23401`（`src/lib.rs:1408-1409`，Bevy 应用错误段）——
  清单断言 4/5 只要求「error 存在」并把具体码留待实测，预期即 -23401（下文以实测回填）。
- `world.query` 参数结构 `BrpQueryParams { data: BrpQuery, filter, strict }`、
  `BrpQuery.components: Vec<String>`（`:383-387`）——与 TS-01/05 已核实形态一致。

## 断言 1：`world.query` Wanderer 基线 `n0 == 100`，取实体 `e` —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":601,"params":{"data":{"components":["game::sim::Wanderer"]}}}'
```

响应原文摘录（全量 100 行见 `ts-06/a1-query-wanderer.raw.json`，`result` 数组首行原文）：

```json
{"components":{"game::sim::Wanderer":{"index":0,"origin":[59.20360565185547,0.800000011920929,40.68254470825195],"phase":0.6825083494186401}},"entity":4294966884}
```

判定：`result` 数组行数 **100**，无 `error` 字段。`n0 = 100`；取首行实体 **e = 4294966884**（index 0）。**通过**。

## 断言 2：`world.despawn_entity` `entity=e` 响应无 error —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.despawn_entity","id":602,"params":{"entity":4294966884}}'
```

响应原文（全量，`ts-06/a2-despawn-entity.raw.json`）：

```json
{"jsonrpc":"2.0","id":602,"result":null}
```

判定：无 `error`；`result: null` 与源码 `Ok(Value::Null)`（`builtin_methods.rs:1343`）一致。**通过**。

## 断言 3：`world.query` 行数 == 99 且不含 `e` —— PASS

命令原文（同断言 1）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":603,"params":{"data":{"components":["game::sim::Wanderer"]}}}'
```

响应原文摘录（全量 99 行见 `ts-06/a3-query-after-despawn.raw.json`，`result` 数组首行原文——
注意 query 行序按 archetype 迭代序返回，不按 `index` 排序，销毁后首行为 index 99）：

```json
{"components":{"game::sim::Wanderer":{"index":99,"origin":[-51.02488708496094,0.800000011920929,30.711864471435547],"phase":3.308243989944458}},"entity":4294966785}
```

判定：无 `error`；行数 **99**；全表扫描 `entity === 4294966884` 无匹配；99 个 `index` 两两不重复且
**恰缺 index 0**（被销毁的正是断言 1/2 所取的行），与断言 1/2 计数自洽（100 − 1 = 99）。**通过**。

## 断言 4：`world.get_components` `entity=e`, `strict=true` 响应 error 存在 —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_components","id":604,"params":{"entity":4294966884,"components":["game::sim::Wanderer"],"strict":true}}'
```

响应原文（全量，`ts-06/a4-get-components-dead.raw.json`）：

```json
{"jsonrpc":"2.0","id":604,"error":{"code":-23401,"message":"Entity 411v0 not found"}}
```

判定：`error` 存在。**实测错误码 = -23401**（`ENTITY_NOT_FOUND`，`bevy_remote-0.19.1/src/lib.rs:1409`；
get_components handler 先 `get_entity` 后反射，`builtin_methods.rs:604`，与断言预期路径一致）；
message `Entity 411v0 not found` 为 Entity 的 Display 形态（index 411 + generation 0，
位串 4294966884 的组成部分），非实体号原文回显。**通过**。

## 断言 5：`world.despawn_entity` 重复销毁 响应 error 存在 —— PASS

命令原文（与断言 2 同参重发）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.despawn_entity","id":605,"params":{"entity":4294966884}}'
```

响应原文（全量，`ts-06/a5-despawn-again.raw.json`）：

```json
{"jsonrpc":"2.0","id":605,"error":{"code":-23401,"message":"Entity 411v0 not found"}}
```

判定：`error` 存在；code 同为 **-23401**（重复销毁与查已死实体走同一 `entity_not_found` 路径，
`get_entity_mut` `builtin_methods.rs:1811-1818`）。**通过**。

## 汇总与口径说明

- 结论：**5/5 断言首次执行全部通过；一次通过，返工 0；零改码**（`cargo check` 仍按公共前置执行且 REAL_EXIT=0）。
- 断言 4 清单括注「具体错误码以实测为准并回填本条」：实测 **-23401**。按本次任务纪律（清单差异/回填不改动
  清单表行本身），该值记录于本证据文件与任务文件「判定记录」节，清单表行未改写。
- 与清单口径的偏差：**无**。断言未放宽；启动命令、请求形态（`entity` 数字原样回填、`strict=true`）、
  判定值均按清单原文执行。断言 4 请求中 `components` 清单未点名，取 `["game::sim::Wanderer"]`
  （任务上下文组件；源码核实实体不存在时先于组件解析报错，该选择不影响断言结果）。
- 跨进程确定性侧证（非清单断言）：本次 a1 与 TS-05 存档 `ts-05/a1-query-wanderer.raw.json` 同
  seed=20260926 同 count=100 逐 index 比对 100/100 行 origin+phase 逐位一致（despawn 前基线）。
  顺带发现 `docs/evidence/ts-05-brp.md` 断言 1 的「首行原文」摘录数字与其自身 raw 文件首行不符
  （raw 首行即本次的 index 0 行；文案摘录笔误，不影响 TS-05 判定与 TS-06，未改动该文件）。
- 游戏收尾：`taskkill /F /IM game.exe`（PID 24824）终止，`tasklist | grep game.exe` 确认无残留；
  `run1-game.exit` REAL_EXIT=1 为强杀预期退出码（非游戏故障，TS-01~05 同款注记）。
- 无代码/行为/接口变更，无文档同步义务触发；零踩坑（despawn/get_components 的请求形态与错误码
  均按 §2 查证后一次写对；断言 4/5 的错误码实测与源码常量 -23401 相互印证）。

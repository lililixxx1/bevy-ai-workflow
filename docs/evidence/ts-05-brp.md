# TS-05 断言证据：BRP 生成实体并回读（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-05-spawn-entity.md`（4 条断言，逐条执行）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo，本次**零改码核对型任务**（任务需求即「不改动代码」）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 100 --seed 20260926`；
  实际执行 `cargo build --release -p game`（REAL_EXIT=0）后直接运行同一产物
  `./target/release/game.exe --count 100 --seed 20260926`（后台）——与 `cargo run --release` 构建并运行的
  是同一二进制同一参数，口径等价（TS-01/02/03/04 同款注记）。横幅：
  `[CONFIG] game demo | count=100 seed=20260926 max_speed=3 bench_secs=0 vsync=on(AutoVsync)`（`ts-05/run1-game.log`）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），约 2s 即就绪（`ts-05/ready-discover.json`，
  23 方法清单与 SKILL.md §6.2 一致）
- 原始文件：本目录 `ts-05/` 下各 `a*.raw.json` / `run1-game.log` / `run1-game.exit` /
  `cargo-check.*` / `cargo-build-release.*`
- 断言用响应字段均以 node 严格 `===` 判定（JSON 数字无浮点舍入歧义；实测 1.0/0.0/2.0 序列化为
  `1`/`0`/`2`，严格相等成立）

## 门禁（公共前置）

- `cargo check --workspace`：**REAL_EXIT=0**（`ts-05/cargo-check.exit`；增量 0.55s；零改码任务仍按公共前置执行）
- `cargo build --release -p game`：REAL_EXIT=0（断言所用即此产物，源码与 TS-04 提交一致无变更）

## 请求形态查证（写断言前，非事后）

- `world.spawn_entity` 参数结构 `BrpSpawnEntityParams { components: HashMap<String, Value> }`
  （`bevy_remote-0.19.1/src/builtin_methods.rs:172-177`）——清单 `components={"game::sim::Velocity":{...}}`
  与源码一致；Vec3 传 `[x,y,z]` 数组形态（glam serde，taskset README 实测注记）。
- `world.get_components` 参数结构 `BrpGetComponentsParams { entity: Entity, components: Vec<String>,
  strict: bool }`（`builtin_methods.rs:118-141`）——清单 `entity` / `components` / `strict=true` 与源码一致。
- `world.query` 参数结构 `BrpQueryParams { data: BrpQuery, filter: BrpQueryFilter, strict }`；
  `BrpQuery.components: Vec<String>`（`:383-387`）、`BrpQueryFilter.without: Vec<String>`（`:409-418`）
  ——清单 `data.components` 与 `filter={"without":[...]}` 与源码一致。
- `Entity` 的 JSON 形态 = u64 位串数字（`bevy_ecs-0.19.1/src/entity/mod.rs:633-641`，
  `serialize_u64(self.to_bits())`）——spawn 响应的 `result.entity` 数字可原样回填 get_components 的
  `params.entity`，无字符串化（T004 PIT 探针教训反向印证）。

## 断言 1：`world.query` Wanderer 基线 `n0 == 100` —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":501,"params":{"data":{"components":["game::sim::Wanderer"]}}}'
```

响应原文摘录（全量 100 行见 `ts-05/a1-query-wanderer.raw.json`，`result` 数组首行原文）：

```json
{"components":{"game::sim::Wanderer":{"index":0,"origin":[-70.7808609008789,0.800000011920929,-63.557125091552734],"phase":1.2848815917968752}},"entity":4294966884}
```

判定：`result` 数组行数 **100**，无 `error` 字段。`n0 = 100`。**通过**。

## 断言 2：`world.spawn_entity` 返回有效实体号 `e` —— PASS

命令原文（Vec3 以 `[x,y,z]` 数组传入，清单括注口径）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.spawn_entity","id":502,"params":{"components":{"game::sim::Velocity":{"linear":[1.0,0.0,2.0]}}}}'
```

响应原文（全量，`ts-05/a2-spawn-entity.raw.json`）：

```json
{"jsonrpc":"2.0","id":502,"result":{"entity":4294966777}}
```

判定：无 `error`；`result.entity = 4294966777`，类型为整数（u64 位串，与断言 1 首行实体 4294966884
同量级、同代际段），记 **e = 4294966777**。**通过**。

## 断言 3：`world.get_components` 回读 `linear == [1.0, 0.0, 2.0]` —— PASS

命令原文（`strict=true` 按清单；entity 为断言 2 的数字原样回填）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_components","id":503,"params":{"entity":4294966777,"components":["game::sim::Velocity"],"strict":true}}'
```

响应原文（全量，`ts-05/a3-get-components.raw.json`）：

```json
{"jsonrpc":"2.0","id":503,"result":{"game::sim::Velocity":{"linear":[1.0,0.0,2.0]}}}
```

判定：无 `error`（`strict=true` 下无组件缺失/无效）；`linear` 三分量与 `[1.0, 0.0, 2.0]`
**严格相等**（node `===` 逐位比较，JSON 的 `1.0`/`0.0`/`2.0` 解析后与目标逐位一致）。
注入值经完整「写 → 存 → 读」回路无失真。**通过**。

## 断言 4：`world.query` 无 Wanderer 的 Velocity 行 `== 1` 且 entity == e —— PASS

命令原文：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":504,"params":{"data":{"components":["game::sim::Velocity"]},"filter":{"without":["game::sim::Wanderer"]}}}'
```

响应原文（全量，`ts-05/a4-query-without-wanderer.raw.json`）：

```json
{"jsonrpc":"2.0","id":504,"result":[{"components":{"game::sim::Velocity":{"linear":[1.0,0.0,2.0]}},"entity":4294966777}]}
```

判定：无 `error`；`result` 数组**恰好 1 行**，行内 `entity = 4294966777 == e`——全游戏唯一的
「有 Velocity 无 Wanderer」实体即本次 spawn 注入的新实体（既有 100 个 Wanderer 全部同时持有
Velocity，被 `without` 过滤排除，与断言 1/2 计数自洽：100 个 Wanderer 各带 Velocity + 1 个新实体
= 无过滤 Velocity 查询应得 101 行，filter 排除 100 行后余 1 行）。**通过**。

## 汇总与口径说明

- 结论：**4/4 断言首次执行全部通过；一次通过，返工 0；零改码**（`cargo check` 仍按公共前置执行且 REAL_EXIT=0）。
- 与清单口径的偏差：**无**。断言未放宽；启动命令、请求形态（含 Vec3 数组形态与 `strict=true`）、
  判定值均按清单原文执行。
- 游戏收尾：`taskkill /F /IM game.exe`（PID 24044）终止，`tasklist | grep game.exe` 确认无残留；
  `run1-game.exit` REAL_EXIT=1 为强杀预期退出码（非游戏故障，TS-01~04 同款注记）。
- 无代码/行为/接口变更，无文档同步义务触发；零踩坑（三个方法的请求形态均按 §2 查证后一次写对）。

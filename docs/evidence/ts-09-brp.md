# TS-09 断言证据：新组件类型注册与 schema（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-09-type-schema.md`（4 条断言，逐条执行）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo，本次**改码任务**（新增 `game::sim::Tagged { tag: String }` 组件 + 显式
  反射注册 + Startup 给 `index < 10` 的 Wanderer 打标）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 100 --seed 20260926`；
  实际执行 `cargo build --release -p game`（REAL_EXIT=0）后直接运行同一产物
  `./target/release/game.exe --count 100 --seed 20260926`（后台）——与 `cargo run --release` 构建
  并运行的是同一二进制同一参数，口径等价（TS-02/03 同款注记）。横幅：
  `[CONFIG] game demo | count=100 seed=20260926 max_speed=3 bench_secs=0 vsync=on(AutoVsync)`（`run1-game.log`）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），首次轮询即就绪（`ready-discover.json`，
  `{"info":{"title":"Bevy Remote Protocol","version":"0.19.1"},...}`）
- 原始文件：本目录 `ts-09/` 下各 `*.raw.json` / 运行日志 / cargo 门禁日志

## 门禁（一次通过口径的关键测量点）

- **首次 `cargo check --workspace`（改码完成后）：REAL_EXIT=0**（`cargo-check-first.exit`；
  增量 1.37s，覆盖全部改动——仅 `game/src/sim.rs` 一个文件）。
- `cargo test -p game`：REAL_EXIT=0，**10 passed / 0 failed**（既有单测无一破坏）。
- `cargo build --release -p game`：REAL_EXIT=0（断言所用即此产物）。

## 实现口径（与任务约束逐条对应）

- 新组件 `Tagged { tag: String }`（`game/src/sim.rs:127`），derive 形态对齐官方 BRP 示例
  Component（`bevy-0.19.1/examples/remote/server.rs:89-91`）+ 一个容器属性
  `#[reflect(Component, Serialize, Deserialize, no_auto_register)]`（`sim.rs:126`）。
  `no_auto_register` 是任务约束「注册必须显式（register_type），不依赖 reflect_auto_register
  自动链」的落点：它使**显式 `register_type::<Tagged>()`（`sim.rs:146`）成为唯一注册通路**。
  依据（本地源码核实 2026-09-26）：
  - 属性语义：`bevy_reflect_derive-0.19.1/src/lib.rs:329-335`（"opt-out of the automatic
    reflect type registration"）；
  - 标注后 derive 不发 `inventory::submit`：`bevy_reflect_derive-0.19.1/src/impls/common.rs:174-177`；
  - 行为 doctest（标注后 `register_derived_types` 不含该类型）：
    `bevy_reflect-0.19.1/src/lib.rs:4044-4057`；
  - 为什么必须有它：0.19.1 默认 feature 链 `default → 2d/3d → default_app →
    reflect_auto_register`（`bevy-0.19.1/Cargo.toml:2742-2754`、`:2586-2592`），`App::new`
    在该 feature 下用 `new_with_derived_types()` 初始化类型注册表（`bevy_app-0.19.1/src/
    app.rs:117-119`），自动注册**所有非泛型** `#[derive(Reflect)]` 类型（`bevy_ecs-0.19.1/
    src/reflect/mod.rs:55-63`、`bevy_reflect-0.19.1/src/type_registry.rs:125-131`，Windows 在
    支持平台列表）——不 opt-out 时注释 `register_type` 该类型仍可见，断言 4 反证无从成立。
- 打标系统 `tag_first_ten`（`sim.rs:237`）：Startup 链 `(init_metadata, spawn_swarm,
  tag_first_ten).chain()`（`sim.rs:147`，§3.3 显式排序）；tag 值取确定性形态 `wanderer-{index}`。
  Commands 跨系统可见依据：`auto_insert_apply_deferred` 构建通道默认开启
  （`bevy_ecs-0.19.1/src/schedule/schedule.rs:1629` 默认 true），对「上游含 Deferred 参数
  （Commands 即是）且存在排序依赖」的边自动插入 ApplyDeferred
  （`bevy_ecs-0.19.1/src/schedule/auto_insert_apply_deferred.rs:13-17` 文档）。
- 「不得改动既有组件定义」：既有五类型（SimConfig/SimStats/SimMetadata/Wanderer/Velocity）
  定义零改动；改动仅新增类型 + build 注册行 + Startup 链 + 模块注释「五个类型」→「六个类型」
  （`sim.rs:17`，履行行为/接口同步义务）。
- 请求形态查证（写断言前，非事后）：`registry.schema` params 为 `BrpJsonSchemaQueryFilter
  { without_crates, with_crates, type_limit }`——只能按 **crate 名**过滤，无按类型路径的参数
  （`bevy_remote-0.19.1/src/builtin_methods.rs:427-449`）；`world.query` 的
  `BrpQueryParams { data: BrpQuery, filter: BrpQueryFilter, strict }`，filter 与 data 平级
  （`:152-165`），`BrpQueryFilter.with: Vec<String>`（`:408-423`）——清单写法与源码一致、未放宽。

## 断言 1：`world.list_components` 数组含 `"game::sim::Tagged"` —— PASS

命令原文（清单要求「无 params」，请求体即不含 params 字段）：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.list_components","id":901}'
```

响应原文摘录（全量见 `ts-09/a1-list-components.raw.json`；`result` 共 312 项排序数组，此处摘 game crate 相关项）：

```json
{"jsonrpc":"2.0","id":901,"result":[ … ,"game::camera::CameraRig","game::sim::SimConfig","game::sim::SimMetadata","game::sim::SimStats","game::sim::Tagged","game::sim::Velocity","game::sim::Wanderer", … ]}
```

判定（node 严格断言输出原文）：`A1: isArray=true | contains game::sim::Tagged = true | total=312`。
**通过**。

## 断言 2：`registry.schema` 该类型 schema 含字段 `tag`（string 类型）—— PASS

清单括注「参数形态以 rpc.discover 返回的方法描述为准」——**实测 `rpc.discover` 对
`registry.schema` 的方法描述为 `"params": []`（未提供参数 schema）**，原文
（`ts-09/a2-discover-registry-schema.txt`，取自 `ready-discover.json`）：

```json
{
 "name": "registry.schema",
 "params": []
}
```

描述为空时按纪律回退**本地源码**定形态：`BrpJsonSchemaQueryFilter` 只能按 crate 名过滤
（`builtin_methods.rs:427-449`），故用最贴近「过滤该类型」的合法参数 `{"with_crates":["game"]}`，
再在返回 map 中按类型路径键定位。命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"registry.schema","id":902,"params":{"with_crates":["game"]}}'
```

响应原文（摘 `result["game::sim::Tagged"]`，全量见 `ts-09/a2-registry-schema-game.raw.json`）：

```json
{"additionalProperties":false,"componentInfo":{"isSendAndSync":true,"mutable":true,"storageType":"Table"},"crateName":"game","kind":"Struct","modulePath":"game::sim","properties":{"tag":{"type":{"$ref":"#/$defs/alloc::string::String"}}},"reflectTypes":["Deserialize","Component","Serialize"],"required":["tag"],"shortPath":"Tagged","type":"object","typePath":"game::sim::Tagged"}
```

判定（node 严格断言输出原文）：`A2: has game::sim::Tagged = true`；schema **含字段 `tag`**
（`properties.tag` 存在且列入 `required`），字段类型为 **string**——0.19.1 的 schema 导出中
字段类型经 `$ref` 引用表达（`#/$defs/alloc::string::String`，生成处
`bevy_remote-0.19.1/src/schemas/json_schema.rs:102-105` + `:437-443`），而
`alloc::string::String` 在类型映射表中归为 `SchemaType::String`（`json_schema.rs:427-429`）。
**侧证**（全量 schema 查 `alloc::string::String` 自身条目，`ts-09/a2-registry-schema-full.raw.json`）：

```json
{"crateName":"alloc","kind":"Value","modulePath":"alloc::string","reflectTypes":["Deserialize","Default","Serialize"],"shortPath":"String","type":"string","typePath":"alloc::string::String"}
```

其 `"type":"string"` 直接印证 $ref 目标即 string 类型。**通过**（未放宽：字段存在 + 类型为 string
两个条件均以实测响应判定）。

## 断言 3：`world.query` Tagged + filter with Wanderer 行数 == 10 —— PASS

命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":904,"params":{"data":{"components":["game::sim::Tagged"]},"filter":{"with":["game::sim::Wanderer"]}}}'
```

响应原文摘录（全量见 `ts-09/a3-query-tagged.raw.json`；恰 10 行，首行原文）：

```json
{"components":{"game::sim::Tagged":{"tag":"wanderer-0"}},"entity":4294966883}
```

判定（node 严格断言输出原文）：`A3: isArray=true | rows=10 (expect 10)`；10 行 tag 值恰为
`["wanderer-0", … , "wanderer-9"]`（index 0..9 全中，index ≥ 10 无一混入；count=100 下
Wanderer 共 100 实体，filter.with 语义生效）。运行日志侧旁证（`run1-game.log`，ANSI 剥离后）：
`INFO game::sim: [SIM] tagged 10 wanderers (index < 10)`。**通过**。

## 断言 4：反证——注释 `register_type::<Tagged>()` 后类型对 BRP 不可见 —— PASS

流程：临时注释 `game/src/sim.rs` 的 `.register_type::<Tagged>()` 行 → `cargo check
--workspace` REAL_EXIT=0（`a4-cargo-check-noreg.exit`）+ `cargo build --release -p game`
REAL_EXIT=0（`a4-cargo-build-noreg.exit`）→ 以**同参数**重启游戏（`run2-noreg-game.log`）→
重复断言 1。

**重复 #1 命令原文与响应摘录**（全量见 `ts-09/a4-list-components-noreg.raw.json`）：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.list_components","id":911}'
```

```json
{"jsonrpc":"2.0","id":911,"result":[ … ,"game::sim::SimStats","game::sim::Velocity", … ]}
```

判定（node 严格断言输出原文）：`A4-1: isArray=true | contains game::sim::Tagged = false |
total=311`——总数 312→311、game crate 条目由 7 减为 6，**`list_components` 不含该路径**（清单主
分支原样满足）。**通过**。

**「或 #3 报错」分支同测**（超出清单的完备性取证）：
- #3 原请求（strict 缺省 false）重复：返回 `{"jsonrpc":"2.0","id":912,"result":[]}`——未知组件
  在非 strict 模式下被跳过而非报错（源码行为：`builtin_methods.rs:1843-1852`，strict 分支才报错）。
- strict 变体（`ts-09/a4-query-noreg-strict.raw.json`）：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":913,"params":{"data":{"components":["game::sim::Tagged"]},"filter":{"with":["game::sim::Wanderer"]},"strict":true}}'
```

```json
{"jsonrpc":"2.0","id":913,"error":{"code":-23402,"message":"Component `game::sim::Tagged` isn't registered or used in the world"}}
```

错误码 -23402 = `COMPONENT_ERROR`（`bevy_remote-0.19.1/src/lib.rs:1412`）；query 路径的报错
文案是 "isn't registered or used in the world"（`builtin_methods.rs:1843-1848`），清单括注引用的
"Unknown component type" 文案属 get_components 路径（`:686`/`:1185`）——两分支同证「未注册类型
BRP 拿不到」（SKILL.md §3.4/§6.3）。此为**文案注记，非结果偏差**（主分支已原样通过）。

**恢复后复验**（清单要求「验证后恢复代码」）：取消注释 → `cargo check --workspace` REAL_EXIT=0
（`restore-cargo-check.exit`）+ release 重建 REAL_EXIT=0（`restore-cargo-build.exit`）→ 第三次
启动同参数复测：`list_components` 复含 `game::sim::Tagged`（total=312，
`run3-restore-list-components.raw.json`）、断言 3 同请求复得 10 行
（`run3-restore-query.raw.json`）——恢复态与首验态逐项一致。

## 汇总与口径说明

- 结论：**4/4 断言首次执行全部通过；一次通过，返工 0**。首次 `cargo check --workspace` REAL_EXIT=0。
- 游戏收尾：三次运行（run1/run2 反证/run3 复验）均 `taskkill /F` 收尾，每次 `tasklist | grep
  game.exe` 确认无残留；`run*-game.exit` REAL_EXIT=1 均为强杀预期退出码（TS-01 起同款注记）。
- 与清单口径的偏差：**无（断言未放宽）**。两处如实注记：①断言 2 的参数形态按清单指示查
  `rpc.discover`，实测其对 registry.schema **不给参数描述**（params:[]），按纪律回退本地源码定
  形态（`with_crates` crate 级过滤 + 响应 map 按类型路径定位），断言本身（含字段 tag、string
  类型）原样执行；②断言 4 括注的错误文案 "Unknown component type" 在 query 路径实测为同义的
  "isn't registered or used in the world"（错误码同 -23402，两文案源码行号均已注），主分支
  （list_components 不含该路径）原样通过。
- 设计决策记录（防误读为「为过断言改口径」）：`#[reflect(no_auto_register)]` 在**写码前**即按
  源码查证定案（见「实现口径」节），它是任务约束「不依赖 reflect_auto_register 自动链」的直接
  落点——没有它，0.19.1 默认 feature 链会自动注册 Tagged，断言 4 的「未注册不可见」将无从观测
  （这正是 SKILL.md §3.4 所述自动链的适用条件与局限）。
- 唯一改码文件：`game/src/sim.rs`（新类型 + 注册行 + Startup 打标系统 + 模块注释计数更新）。
- 工具级偏差 1（非返工，4 条断言的请求/响应均一次成功）：run3 复验时一处便利性复查请求的
  shell 字面值多打一个 `}`（服务端 -32600 trailing characters 拒收），修正后重发成功；该请求
  属恢复态复查，非清单断言（清单断言已在 run1 全部通过）。

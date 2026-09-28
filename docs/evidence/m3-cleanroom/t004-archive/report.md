# T004 净室重跑报告（BRP 通路验证·curl 直调·裁剪版）

- 执行日期：2026-09-28
- 执行者：净室重跑独立执行者（无 prior 会话上下文，仅依据 req-t004.md + INJECTED-agents-discipline.md + 注入知识资产行事）
- 被验对象：`cleanroom/t003/` demo 产物（bevy 0.19.1 + bevy_remote，BRP 显式绑定 127.0.0.1:15702）
- 工作区：`cleanroom/t004/`（本报告 + logs/ 193 个证据文件）
- 二进制复用说明：`t003/target/release/demo.exe`（mtime 09-28 15:26）晚于全部源码/manifest 最后修改（≤15:17），原样复用未重建（零构建，资源纪律满足）。

## 判定结论

**一次通过（首判全链跑通有存证）**。16 个内置方法成功调用存证（≥8 达标，含 4 必测）、`rpc.discover` 原文与方法计数（23）留档、6 组错误探针报错原文留档（≥3 达标）、进程收尾杀净无残留。**返工次数 = 0**（口径理由见 §五）。

需求书裁剪项如实标注：**bevy_brp_mcp 接入部分不可执行**（重跑环境无该 MCP server 配置），不在本重跑判定面内；原任务该半的协议级兼容结论不因重跑改变。

## 一、逐轮时间线

进程形态：demo 由长生命周期后台任务托管（PAT-M-005/PIT-M-007），两段进程（15 探针致进程崩溃后重启一次，见 R15 行）。BRP 全程仅 127.0.0.1:15702。

| 轮 | 时刻 | 内容 | 结果 | curl 退出码 | 证据（logs/ 前缀） |
|---|---|---|---|---|---|
| R00a | 16:19 | 启动 demo（--count 1000 --seed 20260926），+3s 存活检查 | PID 20784 存活 | — | alive-check-2s.log / demo-run.log |
| R00b | 16:19 | rpc.discover 探活（首试） | 200，**23 个方法** | 0 | discover-poll-1.{req,json,err,exit} |
| R02 | 16:20:31 | 探针：未知方法名 `world.no_such_method` | -32601 `Method ... not found` | 0 | 02-* |
| R03 | 16:20:31 | 探针：缺 `jsonrpc` 字段 | **-32600** `missing field \`jsonrpc\`` | 0 | 03-* |
| R04 | 16:20:31 | world.query Wanderer+Velocity | 200，1000 行（== --count） | 0 | 04-* |
| R05 | 16:20:31 | 探针：query strict=true 未注册类型 | -23402 `... isn't registered or used in the world` | 0 | 05-* |
| R06 | 16:20:47 | world.spawn_entity（Velocity{linear:[1,0,2]}） | 200，entity=4294965877 | 0 | 06-* / probe-entity.txt |
| R07 | 16:20:47 | 探针：spawn params {} | -32602 `missing field \`components\`` | 0 | 07-* |
| R08 | 16:20:48 | world.get_components（strict） | 200，linear === [1.0,0.0,2.0]（写→存→读零失真） | 0 | 08-* |
| R09 | 16:20:48 | 探针：get strict 未注册类型 | -23402 `Unknown component type` | 0 | 09-* |
| R10 | 16:21:04 | world.insert_components（Wanderer→E） | 200，result:null | 0 | 10-* |
| R11 | 16:21:04 | 探针：insert 不存在实体 | -23401 `Entity 4294843839v0 not found` | 0 | 11-* |
| R12 | 16:21:04 | world.remove_components（Wanderer 离 E） | 200，result:null | 0 | 12-* |
| R13 | 16:21:04 | 探针：remove 实体上不存在的组件（OrbitCamera） | **200，result:null（幂等成功，不报 -23403）** | 0 | 13-* |
| R14 | 16:21:04 | world.mutate_components（E.linear→[3,0,0]） | 200，result:null | 0 | 14-* |
| R15 | 16:21:04 | 探针：mutate 不存在实体 | **进程崩溃**：空响应，curl 退出码 52（Empty reply）；demo panic 于 bevy_remote builtin_methods.rs:1194 `Entity not yet spawned`，进程退出码 101 | 52 | 15-* / 901-* / demo-run.log 尾部 |
| R901 | 16:21+ | 崩溃后存活复核 rpc.discover | 连接拒绝（exit 7，0 字节）——证实进程确已死亡 | 7 | 901-* |
| R00c | 16:22 | 重启 demo（同参数），+4s 存活 + discover 复核 | PID 23892 存活，discover 200 | 0 | alive-check-2-restart.log / 902-* / demo-run2.log |
| R16 | 16:23:15 | 重立探针实体 E2（spawn） | entity=4294965877（与首进程同号，确定性分配） | 0 | 16-* / probe-entity2.txt |
| R17 | 16:23:16 | world.mutate_components 成功（E2.linear→[3,0,0]） | 200，result:null | 0 | 17-* |
| R18 | 16:23:16 | mutate 读回验证（get_components strict） | linear === [3.0,0.0,0.0]（注入驻留闭环） | 0 | 18-* |
| R19 | 16:23:16 | 探针：remove 不存在实体 | -23401 | 0 | 19-* |
| R20 | 16:23:16 | 探针：remove 未注册类型 | -23402 | 0 | 20-* |
| R21 | 16:23:32 | spawn 空组件实体 F（reparent 载体） | entity=4294965876 | 0 | 21-* / parent-entity.txt |
| R22 | 16:23:32 | world.reparent_entities（E2→F） | 200，result:null | 0 | 22-* |
| R23 | 16:23:32 | 探针：reparent 自父（F→F） | -23404 `Cannot reparent Entity 1419v0 to itself` | 0 | 23-* |
| R24 | 16:23:32 | reparent 解除（无 parent 键，Option 缺省路径） | 200，result:null | 0 | 24-* |
| R25 | 16:23:32 | 探针：despawn 不存在实体 | -23401 | 0 | 25-* |
| R26 | 16:23:32 | world.despawn_entity（清理 E2） | 200，result:null | 0 | 26-* |
| R27 | 16:23:33 | world.despawn_entity（清理 F） | 200，result:null | 0 | 27-* |
| R28 | 16:23:46 | world.list_components（无 params 全量） | 200，**311 项**（含 demo::* 6 类型） | 0 | 28-* |
| R29 | 16:23:46 | 探针：list_components `params:{}` | -32602 `missing field \`entity\``（params 存在性语义） | 0 | 29-* |
| R30 | 16:23:46 | world.list_resources（无 params） | 200，44 项（含 demo::* 3 资源） | 0 | 30-* |
| R31 | 16:23:47 | world.get_resources（SimStats） | 200，result.value（tick 单调推进，见 R44） | 0 | 31-* |
| R32 | 16:23:47 | 探针：get_resources 复数形态 `resources:[...]` | -32602 `missing field \`resource\``（0.19.1 字段为单数） | 0 | 32-* |
| R33 | 16:24:02 | world.mutate_resources（CameraRig.radius→75.0） | 200，result:null | 0 | 33-* |
| R34 | 16:24:02 | mutate_resources 验证（get_resources） | radius === 75.0（写→存→读闭环） | 0 | 34-* |
| R35 | 16:24:02 | mutate_resources 还原（radius→90.0，无净副作用） | 200，result:null | 0 | 35-* |
| R36 | 16:24:02 | 探针：mutate_resources 未知资源 | -23501 `Unknown resource type` | 0 | 36-* |
| R37 | 16:24:23 | registry.schema（with_crates:["demo"]） | 200，43 类型（demo 6 型全含，Wanderer schema：index u32/origin Vec3/phase f32） | 0 | 37-* |
| R38 | 16:24:23 | 探针：schema 过滤器错型（with_crates:123） | -32602 `invalid type: integer \`123\`, expected a sequence` | 0 | 38-* |
| R39 | 16:24:23 | schedule.list（无 params） | 200，schedule_labels 15 + empty 2 + unavailable 2（RemoteLast/Main 运行中暂不可描述） | 0 | 39-* |
| R40 | 16:24:23 | schedule.graph（schedule_label:"Update"） | 200，含 demo::sim::SimStats 组件依赖与冲突表 | 0 | 40-* |
| R41 | 16:24:23 | 探针：schedule.graph 未知 label | -23501 `Schedule with label=NoSuchSchedule not found...` | 0 | 41-* |
| R42 | 16:24:23 | 探针（覆盖外）：trigger_event 未知事件 | -23501 `Unknown event type: \`demo::sim::NoSuchEvent\`` | 0 | 42-* |
| R43 | 16:24:54 | 终态核验：query Wanderer without Velocity | 200，**0 行**（全部写操作净零） | 0 | 43-* |
| R44 | 16:24:54 | 终态核验：SimStats | tick=7399、elapsed=123.9s、fps_1s=60.0（运行中） | 0 | 44-* |
| R99 | 16:25 | 收尾：taskkill //F //IM demo.exe（PID 23892）+ tasklist 复核 | TASKKILL_EXIT=0；复核 findstr 无结果（AFTER_EXIT=1=未找到，即无残留） | — | residue-check-final.log |

## 二、验收清单逐项判定

### a) rpc.discover 方法清单（原文 + 计数）——通过

- 原文：`logs/discover-poll-1.json`（1181 字节，JSON-RPC result.methods 全量）。
- **方法计数 = 23**（node 解析 `result.methods[].length` 实测）。
- 与源码常量表一一对应（builtin_methods.rs:45-111 恰 23 条 `*_METHOD`/`BRP_SCHEDULE_*`/`RPC_DISCOVER_METHOD` 常量；全量经 `RemotePlugin::default()` → `add_default_methods` 注册，lib.rs:688-803，Default impl :791-803）。重跑实测清单与源码面完全一致，无自定义方法（demo 未注册任何自定义方法，符合 t003 源码 brp.rs 形态）。

### b) ≥8 个内置方法成功调用存证——通过（实测 16 个）

| # | 方法 | 成功证据 | 错误面证据（探针） |
|---|---|---|---|
| 1 | rpc.discover | discover-poll-1.json | 03（协议层缺 jsonrpc → -32600）；02（未知方法 → -32601） |
| 2 | world.query ★必测 | 04（1000 行）、43（0 行终态） | 05（strict 未注册 → -23402） |
| 3 | world.spawn_entity ★必测 | 06/16/21 | 07（params {} → -32602） |
| 4 | world.mutate_components ★必测 | 14/17 + 读回 18 | 15（坏实体 → **进程崩溃**，curl 52） |
| 5 | registry.schema ★必测 | 37（demo 6 型 schema） | 38（过滤器错型 → -32602） |
| 6 | world.get_components | 08/18 | 09（strict 未注册 → -23402） |
| 7 | world.insert_components | 10 | 11（坏实体 → -23401） |
| 8 | world.remove_components | 12 | 19（坏实体 → -23401）、20（未注册 → -23402）、13（不存在组件 → null 幂等） |
| 9 | world.despawn_entity | 26/27 | 25（坏实体 → -23401） |
| 10 | world.reparent_entities | 22/24 | 23（自父 → -23404） |
| 11 | world.list_components | 28（311 项） | 29（params {} → -32602） |
| 12 | world.list_resources | 30（44 项） | 无参数错误面：handler 忽略 params（builtin_methods.rs:1414-1416），协议层探针 02/03 覆盖 |
| 13 | world.get_resources | 31/34/44 | 32（复数形态 → -32602） |
| 14 | world.mutate_resources | 33/35（+验证 34、还原 35） | 36（未知资源 → -23501） |
| 15 | schedule.list | 39 | 无参数错误面：handler 忽略 params（builtin_methods.rs:1722），协议层探针覆盖 |
| 16 | schedule.graph | 40 | 41（未知 label → -23501） |

（覆盖外补充探针：42 trigger_event 未知事件 → -23501。四必测方法全部命中且各带成功+错误双档。）

### c) 错误探针报错原文留档（≥3 组）——通过（实测 6 组，报错原文均在 logs/）

1. **协议层**（02/03）：`{"code":-32601,"message":"Method \`world.no_such_method\` not found"}`；`{"code":-32600,"message":"missing field \`jsonrpc\`"}`。
2. **params 形态**（07/29/32/38）：`-32602` missing field `components`/`entity`/`resource`、invalid type: integer `123`, expected a sequence。
3. **实体语义**（11/19/25）：`-23401` `Entity 4294843839v0 not found`（错误消息用 Entity Display 形态 index+gen，非位串回显）。
4. **组件语义**（05/09/20）：`-23402`，query/get 两路径文案不同（"isn't registered or used in the world" vs "Unknown component type"）。
5. **资源/调度语义**（36/41/42）：`-23501` Unknown resource type / Schedule ... not found / Unknown event type。
6. **自父语义**（23）：`-23404` `Cannot reparent Entity 1419v0 to itself`。
7. **破坏性语义**（15，重大发现）：`world.mutate_components` 对不存在实体**无优雅错误分支**——handler 直接 `world.entity_mut(entity)`（builtin_methods.rs:1194），panic "Entity not yet spawned" 经 `process_remote_requests` 系统未被捕获，**整个游戏进程退出（101）**，客户端侧表现为 curl 退出码 52（Empty reply from server）+ 后续连接拒绝（901 探针 exit 7）。原文见 demo-run.log 尾部 panic 栈。

另记一条**非报错语义观察**（R13）：remove_components 移除实体上不存在的已注册组件 → `result:null` 幂等成功，不触发 -23403。

### d) 游戏进程收尾杀净——通过

`taskkill //F //IM demo.exe`（MSYS 双斜杠，PIT-M-005）终止 PID 23892，`tasklist | findstr demo` 复核无输出（exit 1 = 未找到）。原文 `logs/residue-check-final.log`。首段进程（PID 20784）系 R15 探针致其自身 panic 退出（退出码 101，demo-run.log 存证），非本执行者遗留。

## 三、API 查证清单（本地 registry 源码，禁凭记忆纪律）

来源：`~/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/bevy_remote-0.19.1/src/`（核实 2026-09-28，本重跑逐条亲查）：

| 事实 | 出处（文件:行） |
|---|---|
| 23 个方法名常量（含 schedule.list/graph 无 `_METHOD` 后缀） | builtin_methods.rs:45-111 |
| 请求必须含 `"jsonrpc":"2.0"`；缺失报 missing_field | lib.rs:1146（值非 "2.0" 报 invalid_value :1114-1118；method 必填 :1148） |
| BrpGetComponentsParams{entity, components, strict 默认 false} | builtin_methods.rs:118-133 |
| BrpGetResourcesParams 单数字段 `resource:String` | builtin_methods.rs:140-144 |
| BrpQueryParams{data, filter, strict}；BrpQuery{components,option,has}；BrpQueryFilter{without,with} | builtin_methods.rs:152-167 / 380-401 / 408-423 |
| BrpSpawnEntityParams{components: HashMap<String,Value>} | builtin_methods.rs:172-177 |
| BrpInsertComponentsParams / BrpRemoveComponentsParams | builtin_methods.rs:226-240 / 198-208 |
| BrpMutateComponentsParams{entity,component,path,value} | builtin_methods.rs:286-302 |
| BrpMutateResourcesParams{resource,path,value} | builtin_methods.rs:308-320 |
| BrpReparentEntitiesParams{entities, parent: Option 缺省} | builtin_methods.rs:259-273 |
| BrpDespawnEntityParams{entity} | builtin_methods.rs:189-192 |
| BrpListComponentsParams{entity}（params 为 Some 时必填） | builtin_methods.rs:277-280 + handler :1378-1390（params.map(parse).transpose()?） |
| list_resources / schedule_list 忽略 params | builtin_methods.rs:1414-1416 / 1722 |
| registry.schema：params None→默认过滤器；Some→BrpJsonSchemaQueryFilter | builtin_methods.rs:1668-1673 |
| with_crates 过滤仅在 crate_name 为 Some 时生效（无 crate 名类型直通） | builtin_methods.rs:1681-1687（实测 43 = 21 过滤命中 + 22 无名直通） |
| BrpTriggerEventParams{event, value: Option} | builtin_methods.rs:327-333；未知事件报 resource_error :1490-1494 |
| BrpScheduleGraphParams{schedule_label} | builtin_methods.rs:374-380 |
| insert/remove/despawn/reparent 用 get_entity_mut → -23401 | builtin_methods.rs:1128 / 1301-1303 / 1342 / 1356-1357（get_entity_mut 定义 :1811，entity_not_found lib.rs:1317） |
| **mutate_components 唯一裸 panic 路径：world.entity_mut(entity)** | builtin_methods.rs:1194（panic 实证 demo-run.log） |
| reparent 自父检查先于 add_child → -23404 | builtin_methods.rs:1360-1366 |
| 错误码全集：-32700/-32600/-32601/-32602/-32603/-23401/-23402/-23403/-23404/-23501/-23502 | lib.rs:1392-1424 |
| 全部内置方法随 RemotePlugin::default() 注册 | lib.rs:688-803（Default :791-803） |

## 四、与注入知识资产的呼应/差异 + 独立性界限自评

### 呼应（重跑实测与注入资产一致）

- SKILL §6.1/§6.4：端点 127.0.0.1:15702、请求须含 `"jsonrpc":"2.0"` 与 `id` —— 实测一致（03 探针反向证实缺 jsonrpc 即拒）。
- SKILL §6.2：内置方法 23 个 —— rpc.discover 实测 23，逐名对上。
- PIT-B-004：params 存在性语义（`{}` → -32602 missing field `entity`）、get_resources 单数字段 —— 29/32 探针独立复现同型报错。
- 台账 T004 行（原任务记录）：rpc.discover 23 方法、curl 直调内置方法通路 —— 与本重跑一致。
- PAT-B-018/PAT-B-001（回环显式绑定）：demo 源码形态照此，实测监听确为回环（curl 全程打 127.0.0.1）。
- SKILL §6.9：despawn 坏实体 -23401 判定语义 —— 25 探针一致。

### 差异/新增（本重跑的净新观察，注入资产未记载）

1. **mutate_components 坏实体 → 进程崩溃**（builtin_methods.rs:1194 裸 `world.entity_mut`）：注入资产（PIT-B-004/SKILL §6.2/§6.9/台账 T004 行）均未记载此破坏性语义；原任务 9 方法存证是否触发过不可考（禁读其原始证据）。这是本次重跑对「错误语义」面的实质增量——BRP 错误探针存在**能杀死游戏进程**的形态，工具侧（如未来 task-runner）须避免对 mutate 传未验证实体号。
2. remove_components 对实体上不存在的已注册组件 → 幂等 null（不报 -23403）：注入资产未记载。
3. 缺 jsonrpc 的 wire 错误码为 **-32600**（INVALID_REQUEST）：SKILL §6.4 只写了"报 missing_field("jsonrpc")"未给码。
4. registry.schema 的 with_crates 过滤对无 crate 名类型（元组等）不生效（builtin_methods.rs:1681-1687 的 `if let Some(crate_name)`）：注入资产未记载。
5. schedule.graph 未知 label / trigger_event 未知事件的具体错误码均为 -23501 及文案：SKILL §6.7 提过 "Unknown event type" 文案但未给码。
6. 错误消息中实体呈 Display 形态 `4294843839v0`（台账 T011 行有同形记载，属 T011 而非 T004 结论，如实区分）。

### 独立性界限自评（注入资产中含原任务结论本身的条目）

本次重跑是**知情重跑**而非盲重跑，以下注入材料直接或间接含有原 T004 任务的结论，构成先验信息：

- `assets-methodology/task-ledger.md` **T004 行**：原任务的原始记录（9 个内置方法 + 32 次 MCP 调用 + 返工 2 次 + "rpc.discover 23 方法"）。本重跑的方法计数预期由此预知。
- `bevy-dev/pitfalls.md` **PIT-B-004**：由原 T004 执行期失败直接转化（复现证据指向原任务产物 tooling/brp-logs/07-/08-）。本重跑的 29/32 探针设计与预期错误形态受其影响（尽管我先行源码查证复认了 handler 行为）。
- `bevy-dev/SKILL.md` **§6.1/§6.2/§6.4**：源于 T002 的源码核实（T004 前置任务）与 T019 回写，含端点/23 方法/请求体字段——预知了通路参数形态，压低了"请求形态首试失败"的概率。
- `bevy-dev/patterns/PAT-B-001/PAT-B-018`：主仓 game/（T003/T019 产物）代码形态，本重跑未直接使用其代码但复用了其结论（回环绑定）。
- `assets-methodology/patterns.md` PAT-M-005 / `pitfalls.md` PIT-M-005/PIT-M-007：进程托管与 taskkill 转义方法论（非 T004 专属结论，但直接决定本次执行形态）。

净新观察（上节 1-6 项，尤其 mutate 崩溃语义）不受上述先验污染——它们不在任何注入材料中，由本次探针首次触发并归档。结论：本重跑验证了「注入知识资产足以支撑一次通过的重跑」，但不能声称对 23 方法计数与 params 语义的**首次发现权**；破坏性错误语义一条为本重跑独立增量。

## 五、返工次数自认定：0

口径（需求书）：返工 = 首判未过后的重做次数；错误探针是验收项不是失败轮。

- 全部 16 个方法的成功调用**均首试通过**（无一例"请求形态写错需修正重试"）；判定所需证据首轮采集齐备 → 首判全链跑通有存证 = 一次通过。
- R15 的进程崩溃是**故意错误探针的观察结果**（验收项 c），不是成功调用失败；其后果（重启 demo 一次）为探针语义的自然代价，非重做。崩溃原文、空响应（curl 52）、死亡复核（901，exit 7）三重留档，未删改。
- R13 的 null 语义同理（探针观察，非失败）。

## 六、证据文件清单（logs/，共 193 个文件）

- 进程与生命周期：alive-check-2s.log、alive-check-2-restart.log、demo-run.log（首进程，含 R15 panic 栈）、demo-run2.log（次进程）、residue-check-final.log（收尾杀净）
- 逐调四件套 `NN-*.{req,json,err,exit}`：discover-poll-1、02-05、06-09、10-15、16-20、21-27、28-32、33-36、37-42、43-44、901-902（.req 为 curl --data 串原文，.exit 含 REAL_EXIT）
- 实体号中转：probe-entity.txt、probe-entity2.txt、parent-entity.txt
- 命令流水：commands.log（每条 curl 带时刻戳与完整命令原文）

## 七、执行纪律核对

- 后台任务托管常驻进程（两段）✓；BRP 仅 127.0.0.1 ✓（demo 显式 with_address(LOCALHOST)，curl 全程回环）
- 一次只跑一个重负载命令 ✓（零构建，复用二进制；curl 轮询轻量）
- 未修改主仓与 t003 任何文件 ✓（仅运行其产物）
- 禁区未读 ✓（未触碰主仓 game/、tooling/、docs/、.git/、bevy-probe-*、.zcode/、cleanroom/t005/）
- 失败/意外原文未删改 ✓（15 号空响应与崩溃日志原样保留）

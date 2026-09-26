# bevy_brp_mcp 0.22.7 工具级实测报告

- 日期：2026-09-26（UTC 11:30–11:40 三轮会话）
- 被测服务端：`bevy_brp_mcp.exe` 0.22.7（`cargo install bevy_brp_mcp --version 0.22.7 --locked`，编译 7m05s，
  其依赖 `bevy ^0.19.1`——crates.io API 实查）
- 被控游戏：同 `验证报告.md`（bevy 0.19.1，127.0.0.1:15702，PID 16984）
- 驱动方式：MCP stdio（JSON-RPC over stdin/stdout），自研探针脚本三轮会话；
  每轮全部请求/响应原文见 `10-mcp-session{,2,3}-transcript.jsonl`，结构化摘要见同前缀 `.summary.json`
- 仓库状态说明：GitHub `natepiano/bevy_brp_mcp` 已归档（ARCHIVE_NOTICE.md 原文：moved to
  `natepiano/bevy_brp` workspace）；crates.io 的 0.22.7 仍为活跃发布渠道（2026-09-23 发布）。
  本报告实测的是 crates.io 0.22.7 二进制。

## 工具清单（tools/list 实测）

共 **47 个工具**（第一轮会话 `tools/list` 原文，`10-mcp-session-transcript.jsonl`；分类经脚本对
tools/list 逐名重数，2026-09-26 返工核正）：world 核心（`world_*` 非 watch）16 个、协议元
（rpc_discover / registry_schema）2 个、应用管理（brp_status / brp_launch / brp_list_bevy /
brp_shutdown / brp_all_type_guides / brp_list_agent_tools / brp_type_guide / brp_execute）8 个、
extras 桥（`brp_extras_*`）14 个、watch 4 个（world_get_components_watch /
world_list_components_watch / brp_list_active_watches / brp_stop_watch）、
日志 3 个（brp_list_logs / brp_read_log / brp_delete_logs）。16+2+8+14+4+3 = **47** ✅。

## 逐工具实测结果

「兼容」判据：调用成功且结果与原生 BRP（curl 直连同方法）语义一致。
下表为**工具 × 场景**的合并视图（19 行）；按原始调用次数的完整统计见表后 §「按原始调用次数统计」
（三轮共 32 次 tools/call，同工具多次调用各计一次）。

| 工具 | 调用参数（要点） | 结果 | 与 bevy 0.19.1 兼容 | 证据 |
|------|------------------|------|--------------------|------|
| `brp_status` | `{app_name:"game", port:15702}` | ✅ `Process 'game' (PID: 16984) is running with BRP enabled on port 15702` | ✅ | s2 |
| `rpc_discover` | `{port:15702}` | ✅ `Discovered 23 methods`（与原生 rpc.discover 一致：23 个） | ✅ | s1 |
| `world_spawn_entity` | `{components:{game::sim::Wanderer:{index:7777,origin:[4,5,6],phase:1.25},game::sim::Velocity:{linear:[0,0,0]}}}` | ✅ `Spawned entity 4294965876` | ✅（Vec3 数组形式与原生一致） | s1 |
| `world_get_components` | `{entity:4294965876, components:["game::sim::Wanderer"]}` | ✅ 读到组件 | ✅ | s2 |
| `world_mutate_components` | `{entity:4294965876, component:"game::sim::Wanderer", path:"index", value:8888}` | ✅ `Mutated ... Wanderer`，读回 index=8888 | ✅（字段级 path 形态，与 0.19 BrpMutateComponentsParams 匹配） | s2 |
| `world_query` | `{data:{components:["game::sim::Wanderer"]}, filter:{with:["game::sim::Velocity"]}}` | ✅ `Found 1000 entities`（entity_count=1000, component_count=2000） | ✅（注意：MCP 只回计数元数据，不回传全量行——1000 行原文需 curl 直连取，见 03 文件） | s2 |
| `world_list_components`（实体） | `{entity:4294965876}` | ✅ 2 个（Wanderer+Velocity） | ✅ | s2 |
| `world_list_components`（全量） | `{}`（不传 entity） | ✅ 310 个注册组件 | ✅（MCP 正确适配 0.19 的 params 存在性语义：省略 params 走全量分支，builtin_methods.rs:1378-1400） | s2 |
| `world_get_resources` | `{resource:"game::sim::SimStats"}`（单数） | ✅ tick/elapsed 实时值 | ✅（单数 `resource` 字段，与 0.19 BrpGetResourcesParams 匹配） | s1 |
| `world_list_resources` | `{port:15702}` | ✅ 44 个资源 | ✅ | s3 |
| `registry_schema` | `{with_crates:["game"]}` | ✅ 42 schemas | ✅（与 curl 直连完全一致：5 个 game 类型 + 37 个无 crate 名原语类型） | s1 |
| `brp_execute` | `{method:"world.get_resources", params:{resource:"game::sim::SimConfig"}}` | ✅ 读到 SimConfig（entity_count=1000, seed=20260926） | ✅（任意 BRP 方法逃生舱） | s2 |
| `brp_type_guide` | `{types:["game::sim::Wanderer"]}` | ✅ 2525 字节格式指导（spawn/insert 示例 + mutation path） | ✅ | s3 |
| `brp_all_type_guides` | `{crates:["game"]}` | ✅ `Discovered schemas for all 310 registered type(s)` | ✅（注意 message 报全量 310，crates 过滤在该响应 message 中未体现——使用时以 result 数据为准） | s3 |
| `brp_extras_screenshot` | `{path:"...png", port:15703}` | ❌ `Method \`brp_extras/screenshot\` not found. This method requires the bevy_brp_extras crate ...` | ⚠️ 环境性失败（游戏未装 bevy_brp_extras，非 0.19 协议不兼容） | s2 |
| `brp_extras_send_keys` | `{keys:["Space"], port:15703}` | ❌ 同上（`requires ... BrpExtrasPlugin (error -32601)`） | ⚠️ 环境性失败（同上） | s2 |
| `brp_list_agent_tools` | `{port:15702}` | ❌ `Method \`brp_extras/agent_tools\` not found ...` | ⚠️ 环境性失败（同上） | s3 |
| 错误处理探针 | 错误端口 15799 / 已删实体二次 despawn | ❌ 干净透传：HTTP connection failed 原文 / `Entity 1419v0 not found (error -23401)` | ✅（错误码与原文透传，agent 可读） | s2 |
| 参数校验探针 | 探针侧参数格式错误共 8 次（见下节统计）：entity 传字符串 `"4294965876"`（5 次，`Invalid parameter format ... invalid type: string ..., expected u64`）、brp_status 缺 `app_name`（1 次）、brp_extras_screenshot 缺 `path`（1 次，s1 版）、brp_type_guide 缺 `types`（1 次，s2 版） | ❌ 均被 MCP 参数校验拦截，报错原文自解释 | ✅（MCP 侧严格校验；客户端须传数字而非字符串、必填字段不可缺） | s1+s2 |

（证据列：s1 = `10-mcp-session-transcript.jsonl`，s2 = `10-mcp-session2-transcript.jsonl`，s3 = `10-mcp-session3-transcript.jsonl`；均配 `.summary.json`）

## 按原始调用次数统计（2026-09-26 返工核正）

对三轮 transcript 的 32 次 `tools/call` 逐条解析（解析脚本重跑于返工时，与首轮会话原文比对一致）：

| 分类 | 次数 | 明细 |
|------|------|------|
| 成功 | **19** | rpc_discover、world_spawn_entity、world_query×2、world_get_resources×2、registry_schema、brp_status、world_get_components×2（读+读回）、world_mutate_components、world_list_components×2（实体+全量）、world_despawn_entity、brp_execute、brp_type_guide×2（Wanderer+Velocity）、brp_all_type_guides、world_list_resources |
| 探针参数格式错误（我方之错，MCP 校验拦截） | **8** | s1 探针 v1：brp_status 缺 `app_name`（1）、entity 传字符串致 u64 校验失败（5：get_components×2 / mutate_components / list_components / despawn_entity）、brp_extras_screenshot 缺 `path`（1）；s2：brp_type_guide 缺 `types`（1，v3 补测成功） |
| 故意错误探针（验证错误透传） | **2** | 错误端口 15799 的 world_get_resources（HTTP connection failed 原文）；已删实体二次 despawn（`Entity 1419v0 not found (error -23401)`） |
| extras 环境性失败（游戏未装 bevy_brp_extras，预期内） | **3** | brp_extras_screenshot、brp_extras_send_keys、brp_list_agent_tools |
| **合计** | **32** | 19 成功 + 13 失败 |

## 结论

1. **bevy_brp_mcp 0.22.7 与 bevy 0.19.1 游戏协议级兼容**：32 次 `tools/call` 中 19 次成功；13 次失败
   无一为协议不兼容——8 次为我方探针参数格式错误（被 MCP 校验正确拦截，报错原文自解释）、
   2 次为故意的错误处理探针（错误码与原文干净透传）、3 次为游戏侧未安装 `bevy_brp_extras` 的
   环境性预期失败。探针侧失败恰恰验证了 MCP 的参数校验与错误透传质量。
2. MCP 对 0.19 的三处收紧 schema 均已正确适配（list_components 无参全量 / get_resources 单数 /
   mutate_components 字段级 path）——对照 `验证报告.md` §1.1，这些恰是裸写 curl 易踩的坑，
   MCP 工具层已抹平。
3. 两个使用注意：`world_query` 大结果集只回计数元数据不回全量行（需要原始行时用 curl 直连或
   `brp_execute`）；extras 系工具（screenshot/send_keys/type_text 等 14 个）要求游戏加装
   `bevy_brp_extras` crate——若工作流三需要截图/输入注入能力，后续任务应评估引入该依赖。
4. 供应链状态：上游 GitHub 仓库已归档迁移至 `natepiano/bevy_brp` workspace，crates.io 发布渠道
   正常且活跃（0.22.7 为 2026-09-23 发布）。锁定建议：`--version 0.22.7 --locked`，升级随升级窗口走。

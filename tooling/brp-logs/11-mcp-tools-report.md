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

共 **47 个工具**（第一轮会话 `tools/list` 原文，`10-mcp-session-transcript.jsonl`）：
BRP 核心（world_* / registry_schema / rpc_discover）20 个、应用管理（brp_*）12 个、
extras 桥（brp_extras_*）14 个、watch 2 个（world_get_components_watch / world_list_components_watch）、
日志 3 个（brp_list_logs / brp_read_log / brp_delete_logs）。

## 逐工具实测结果

「兼容」判据：调用成功且结果与原生 BRP（curl 直连同方法）语义一致。

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
| 参数校验探针 | entity 传字符串 `"4294965876"` | ❌ `Invalid parameter format for 'GetComponentsParams': invalid type: string ..., expected u64` | ✅（MCP 侧严格校验；客户端必须传数字而非字符串） | s1 |

（证据列：s1 = `10-mcp-session-transcript.jsonl`，s2 = `10-mcp-session2-transcript.jsonl`，s3 = `10-mcp-session3-transcript.jsonl`；均配 `.summary.json`）

## 结论

1. **bevy_brp_mcp 0.22.7 与 bevy 0.19.1 游戏协议级兼容**：本轮 19 项调用中 14 项直接成功，
   3 项失败均为游戏侧未安装 `bevy_brp_extras` 的环境性预期失败（报错原文自解释），2 项失败为我方探针
   传错参数类型（MCP 校验拦截，行为正确）。未发现任何协议不兼容。
2. MCP 对 0.19 的三处收紧 schema 均已正确适配（list_components 无参全量 / get_resources 单数 /
   mutate_components 字段级 path）——对照 `验证报告.md` §1.1，这些恰是裸写 curl 易踩的坑，
   MCP 工具层已抹平。
3. 两个使用注意：`world_query` 大结果集只回计数元数据不回全量行（需要原始行时用 curl 直连或
   `brp_execute`）；extras 系工具（screenshot/send_keys/type_text 等 14 个）要求游戏加装
   `bevy_brp_extras` crate——若工作流三需要截图/输入注入能力，后续任务应评估引入该依赖。
4. 供应链状态：上游 GitHub 仓库已归档迁移至 `natepiano/bevy_brp` workspace，crates.io 发布渠道
   正常且活跃（0.22.7 为 2026-09-23 发布）。锁定建议：`--version 0.22.7 --locked`，升级随升级窗口走。

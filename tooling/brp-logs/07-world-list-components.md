# world.list_components 实测存证

- 时间(UTC): 2026-09-26T11:14:13Z  目标: http://127.0.0.1:15702
- 依据: `BRP_LIST_COMPONENTS_METHOD` = builtin_methods.rs:66
- 用途: 验证 BRP 操作组件依赖反射注册（game::sim::* 四类型须在列）

## curl 命令

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":3,"method":"world.list_components","params":{}}'
```

## 响应原文

```json
{"jsonrpc":"2.0","id":3,"error":{"code":-32602,"message":"missing field `entity`"}}
```

## 修正重测（带 entity，2026-09-26T11:15:43ZZ）

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":9,"method":"world.list_components","params":{"entity":4294965877}}'
```

```json
{"jsonrpc":"2.0","id":9,"result":["game::sim::Velocity","game::sim::Wanderer"]}
```

- 结论: 0.19.1 中 list_components 必须指定 entity，无参全量清单改用 registry.schema（无 params）获得。

## 二次修正（2026-09-26）：params 完全省略才是全量清单的正确用法

- 复核源码：\`process_remote_list_components_request\` 中 \`params.map(parse).transpose()?\`（builtin_methods.rs:1378-1385）——\`params:{}\` 会因缺 entity 报错，但**完全不发 params**（None）走全量分支。MCP 的 world_list_components 不传 entity 即此形态（10-mcp-session2 实测返回 310）。

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":14,"method":"world.list_components"}'
```

```json
{"jsonrpc":"2.0","id":14,"result":["bevy_a11y::AccessibilityRequested","bevy_a11y::ManageAccessibilityUpdates","bevy_animation::AnimatedBy","bevy_animation::AnimationPlayer","bevy_animation::AnimationTargetId","bevy_animation::graph::AnimationGraphHandle","bevy_animation::transition::AnimationTransitions","bevy_anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening","bevy_anti_alias: ...（共 310 项，完整原文见下）
mponents::DynamicWorldRoot","bevy_world_serialization::components::WorldAssetRoot","game::camera::CameraRig","game::sim::SimConfig","game::sim::SimStats","game::sim::Velocity","game::sim::Wanderer"]}
```

# world.mutate_components 实测存证

- 时间(UTC): 2026-09-26T11:15:43Z  目标: http://127.0.0.1:15702  实体: 4294965877（由 world.spawn_entity 创建）
- 依据: `BRP_MUTATE_COMPONENTS_METHOD` = builtin_methods.rs:69；params BrpMutateComponentsParams{entity, component(完整路径), path(字段路径), value}（builtin_methods.rs:286-302）——0.19 为按字段路径单点改写，非整组件覆盖

## curl 命令

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":7,"method":"world.mutate_components","params":{"entity":4294965877,"component":"game::sim::Wanderer","path":"index","value":12345}}'
```

## 响应原文

```json
{"jsonrpc":"2.0","id":7,"result":null}
```

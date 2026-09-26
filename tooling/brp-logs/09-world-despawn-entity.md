# world.despawn_entity 实测存证（清理本批 spawn 的实体 4294965877）

- 时间(UTC): 2026-09-26T11:15:43Z  目标: http://127.0.0.1:15702
- 依据: `BRP_DESPAWN_COMPONENTS_METHOD` = builtin_methods.rs:60；BrpDespawnEntityParams{entity}

## curl 命令

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":11,"method":"world.despawn_entity","params":{"entity":4294965877}}'
```

## 响应原文

```json
{"jsonrpc":"2.0","id":11,"result":null}
```

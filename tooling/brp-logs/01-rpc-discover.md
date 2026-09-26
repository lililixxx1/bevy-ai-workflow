# rpc.discover 实测存证

- 时间(UTC): 2026-09-26T11:14:13Z  目标: http://127.0.0.1:15702  game: release, count=1000, seed=20260926
- 依据: 方法名常量 `RPC_DISCOVER_METHOD` = bevy_remote-0.19.1/src/builtin_methods.rs:111

## curl 命令

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"rpc.discover","params":{}}'
```

## 响应原文

```json
{"jsonrpc":"2.0","id":1,"result":{"info":{"title":"Bevy Remote Protocol","version":"0.19.1"},"methods":[{"name":"world.trigger_event","params":[]},{"name":"world.spawn_entity","params":[]},{"name":"schedule.list","params":[]},{"name":"schedule.graph","params":[]},{"name":"world.get_resources","params":[]},{"name":"world.list_components+watch","params":[]},{"name":"world.query","params":[]},{"name":"world.list_resources","params":[]},{"name":"world.despawn_entity","params":[]},{"name":"world.mutate_resources","params":[]},{"name":"world.observe+watch","params":[]},{"name":"world.insert_resources","params":[]},{"name":"world.write_message","params":[]},{"name":"world.get_components+watch","params":[]},{"name":"world.reparent_entities","params":[]},{"name":"world.remove_components","params":[]},{"name":"world.mutate_components","params":[]},{"name":"world.insert_components","params":[]},{"name":"world.list_components","params":[]},{"name":"rpc.discover","params":[]},{"name":"world.get_components","params":[]},{"name":"world.remove_resources","params":[]},{"name":"registry.schema","params":[]}],"openrpc":"1.3.2","servers":[{"name":"Server","url":"127.0.0.1:15702"}]}}
```

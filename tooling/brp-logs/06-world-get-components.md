# world.get_components 实测存证（mutate 读回验证）

- 时间(UTC): 2026-09-26T11:15:43Z  目标: http://127.0.0.1:15702  实体: 4294965877
- 依据: `BRP_GET_COMPONENTS_METHOD` = builtin_methods.rs:45；BrpGetComponentsParams{entity, components: Vec<完整路径>}（builtin_methods.rs:118-135）
- 预期: Wanderer.index 已从 9999 变为 12345（mutate 生效证据）；origin 为 [1,2,3] 序列形式（spawn 教训的镜像证据）

## curl 命令

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":8,"method":"world.get_components","params":{"entity":4294965877,"components":["game::sim::Wanderer","game::sim::Velocity"]}}'
```

## 响应原文

```json
{"jsonrpc":"2.0","id":8,"result":{"components":{"game::sim::Velocity":{"linear":[0.10000000149011612,0.0,0.0]},"game::sim::Wanderer":{"index":12345,"origin":[1.0,2.0,3.0],"phase":0.5}},"errors":{}}}
```

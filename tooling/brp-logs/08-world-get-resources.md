# world.get_resources 实测存证

- 时间(UTC): 2026-09-26T11:14:13Z  目标: http://127.0.0.1:15702
- 依据: `BRP_GET_RESOURCE_METHOD` = builtin_methods.rs:78；params 字段名以 BrpGetResourcesParams 反序列化实测为准

## curl 命令

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":4,"method":"world.get_resources","params":{"resources":["game::sim::SimStats","game::sim::SimConfig"]}}'
```

## 响应原文

```json
{"jsonrpc":"2.0","id":4,"error":{"code":-32602,"message":"missing field `resource`"}}
```

## 修正重测（单数字段 resource，2026-09-26T11:15:43ZZ）

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":10,"method":"world.get_resources","params":{"resource":"game::sim::SimStats"}}'
```

```json
{"jsonrpc":"2.0","id":10,"result":{"value":{"avg_fps":60.00361542898816,"elapsed_secs":232.86748746130615,"fps_1s":60.0038520505677,"frame_count":13968,"tick":13968}}}
```

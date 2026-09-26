# world.spawn_entity 实测存证

- 时间(UTC): 2026-09-26T11:15:20Z  目标: http://127.0.0.1:15702
- 依据: `BRP_SPAWN_ENTITY_METHOD` = builtin_methods.rs:51；params BrpSpawnEntityParams{components: HashMap<完整类型路径, Value>}（builtin_methods.rs:172-188）

## 首次尝试（Vec3 用 map 形式）——失败，原文存证

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":5,"method":"world.spawn_entity","params":{"components":{"game::sim::Wanderer":{"index":9999,"origin":{"x":1.0,"y":2.0,"z":3.0},"phase":0.5},"game::sim::Velocity":{"linear":{"x":0.1,"y":0.0,"z":0.0}}}}}'
```

```json
{"jsonrpc":"2.0","id":5,"error":{"code":-23402,"message":"game::sim::Velocity is invalid: invalid type: map, expected a sequence of 3 f32 values"}}
```

## 教训: Vec3 经 reflect 序列化为 [x,y,z] 序列，不接受 {x,y,z} map（错误原文见上）

## 修正后 curl 命令

```sh
curl -s -X POST http://127.0.0.1:15702 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":6,"method":"world.spawn_entity","params":{"components":{"game::sim::Wanderer":{"index":9999,"origin":[1.0,2.0,3.0],"phase":0.5},"game::sim::Velocity":{"linear":[0.1,0.0,0.0]}}}}'
```

## 响应原文

```json
{"jsonrpc":"2.0","id":6,"result":{"entity":4294965877}}
```

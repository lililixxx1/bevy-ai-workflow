# TS-06：BRP 销毁实体

- 难度：低
- 前置：TS-05
- 类型：功能（核对型：world.despawn_entity 通路与错误面）

## 需求描述

不改动代码。经 BRP 销毁一个既有实体，核对计数减少与二次销毁的报错行为。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 100 --seed 20260926`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.query` | `data.components=["game::sim::Wanderer"]` | `n0 == 100`；取任一行 `entity` 记为 `e` |
| 2 | `world.despawn_entity` | `entity=e` | 响应无 error |
| 3 | `world.query` | 同 #1 | 行数 == 99 且不含 `e` |
| 4 | `world.get_components` | `entity=e`, `strict=true` | 响应 `error` 存在（实体已不存在，具体错误码以实测为准并回填本条） |
| 5 | `world.despawn_entity` | `entity=e`（重复销毁） | 响应 `error` 存在 |

## 判定记录

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）

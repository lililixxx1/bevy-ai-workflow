# TS-05：BRP 生成实体并回读

- 难度：低
- 前置：TS-01
- 类型：功能（核对型：world.spawn_entity / world.get_components 通路）

## 需求描述

不改动代码。经 BRP 生成一个带自定义组件的新实体并回读其组件值，验证「注入状态可回读」的写通路。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 100 --seed 20260926`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.query` | `data.components=["game::sim::Wanderer"]` | 记基线数量 `n0 == 100` |
| 2 | `world.spawn_entity` | `components={"game::sim::Velocity":{"linear":[1.0,0.0,2.0]}}`（**Vec3 序列化为 `[x,y,z]` 数组**——glam serde 形态，2026-09-26 实测；传对象报 `expected a sequence of 3 f32 values`） | 响应 `result.entity` 返回有效实体号；记为 `e` |
| 3 | `world.get_components` | `entity=e`, `components=["game::sim::Velocity"]`, `strict=true` | `linear == [1.0, 0.0, 2.0]` |
| 4 | `world.query` | `data.components=["game::sim::Velocity"], filter={"without":["game::sim::Wanderer"]}` | 行数 == 1 且行内 entity == `e`（新实体无 Wanderer） |

## 判定记录

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）

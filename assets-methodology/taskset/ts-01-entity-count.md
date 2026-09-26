# TS-01：实体数量参数化与计数核对

- 难度：低
- 前置：无（demo 基线即被测对象）
- 类型：功能（本任务为**核对型**：验证既有能力的 BRP 断言通路，同时练习口径书写）

## 需求描述

以不同 `--count` 启动 demo，核对 BRP 查询到的实体数与注入值一致，并把「启动配置 → BRP 可见状态」的对应关系固化为断言步骤（两阶段判定的第一阶段样例）。

约束：不得改动 `game/` 代码（本任务是核对既有行为）；断言须使用 BRP 状态查询，不允许只看进程日志。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 500 --seed 20260926`（后台），BRP 端点 `http://127.0.0.1:15702`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.query` | `params.data.components=["game::sim::Wanderer"]` | 响应 `result` 数组长度 == 500 |
| 2 | `world.get_resources` | `params.resource="game::sim::SimConfig"` | `entity_count == 500` 且 `seed == 20260926` 且 `paused == false` |
| 3 | `world.get_resources` | `params.resource="game::sim::SimStats"` | `tick > 0`（模拟已每帧推进）；间隔 ≥1s 再查一次，`tick2 > tick1` |
| 4 | `world.query` | `params.data.components=["game::sim::Wanderer"], params.data.option=["game::sim::Velocity"]` | 每行同时含 Wanderer 与 Velocity 值（初值确定性：同 seed 下 `linear` 向量跨进程一致） |

## 判定记录

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）

# TS-12：批量实体操作一致性

- 难度：中
- 前置：TS-05、TS-06
- 类型：功能（核对型：多次 BRP 写操作后世界状态一致性）

## 需求描述

不改动代码。对同一运行中的进程连续执行 10 次成对操作（despawn 一个 Wanderer + spawn 一个带 `Velocity` 的新实体），核对每一步后世界计数与筛选计数保持一致（总实体数的净变化、Wanderer 计数递减、无 Wanderer 但有 Velocity 的实体数递增）。

约束：全部经 BRP；不得重启进程。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 100 --seed 20260926`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.query` ×2 | Wanderer 计数；`filter={"without":["game::sim::Wanderer"]}` + `data.components=["game::sim::Velocity"]` 计数 | 初始 `w0 == 100`、`v0 == 0` |
| 2 | 循环 10 次：`world.query`（取一个 Wanderer）→ `world.despawn_entity` → `world.spawn_entity`（带 Velocity） | — | 每次操作均无 error |
| 3 | `world.query` ×2 | 同 #1 | `w1 == 90`、`v1 == 10`（净变化精确一致） |
| 4 | `world.get_resources` | `resource="game::sim::SimStats"` | `tick` 在整个操作期间持续增长（批量操作不阻塞模拟） |

## 判定记录

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）

# TS-07：远程改写实体运动参数

- 难度：高
- 前置：TS-01、TS-05
- 类型：功能（核对型：world.mutate_components + 解析式运动的可预测断言）

## 需求描述

不改动代码。经 BRP 改写一个 Wanderer 实体的 `Velocity.linear`，并利用 demo 的解析式运动口径（`game/src/sim.rs`：`p(t) = origin + linear*t + Y*0.8*sin(2π*0.7*t + phase)`）精确复算改写后的位置——这是「状态注入 + 数学断言」的完整样例。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 100 --seed 20260926`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.query` | `data.components=["game::sim::Wanderer","game::sim::Velocity"]` | 取 `index == 0` 的行，记 `entity=e`、`origin`、`linear_v0` |
| 2 | `world.get_resources` | `resource="game::sim::SimStats"` | 记 `t1 = elapsed_secs` |
| 3 | `world.mutate_components` | `entity=e`, `component="game::sim::Velocity"`, `path="linear"`, `value=[0.0,0.0,0.0]`（Vec3 数组形态，见 TS-05 注） | 响应无 error |
| 4 | `world.get_resources` + `world.get_components` | 先取 `elapsed_secs = t2`，再取 `e` 的 `bevy_transform::components::transform::Transform` | `translation ≈ [origin.x, 0.8*sin(2π*0.7*t2+phase)+origin.y, origin.z]`（速度已归零，漂移项消失；实测口径容差 0.1——见 `docs/brp-smoke.md` 数学断言，err 实测 ≈0.06） |

## 判定记录

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）

# TS-10：事件驱动的状态变更

- 难度：高
- 前置：TS-04
- 类型：功能

## 需求描述

定义事件类型 `PauseRequested`（`game::sim::PauseRequested`，Reflect + Serialize + Deserialize），添加一个 observer（或响应事件的 system）——收到该事件时翻转 `SimConfig.paused`。BRP `world.trigger_event` 是唯一允许的触发通道。

约束：事件类型与 observer 的 API 形态首次使用前按 SKILL.md §2.1 查证 `bevy_ecs-0.19.1/src/`（observer/消息系统在 0.19 的准确 API 不得凭记忆写）；事件必须可被反射注册（BRP 需要按全路径构造事件）。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 100 --seed 20260926`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.get_resources` | `resource="game::sim::SimConfig"` | `paused == false`（初始态） |
| 2 | `world.trigger_event` | `event="game::sim::PauseRequested"`（`value` 形态以实测/schema 为准，可空） | 响应无 error |
| 3 | `world.get_resources` | 同 #1（等待 ≥0.5s） | `paused == true`（事件副作用已生效） |
| 4 | `world.trigger_event` + `world.get_resources` | 再触发一次 | `paused == false`（翻转语义） |

## 判定记录

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）

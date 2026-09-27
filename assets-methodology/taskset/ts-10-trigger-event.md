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

- 判定者/日期/结论：执行 agent，2026-09-26，**通过**（两阶段判定第一阶段：人工对照；4/4 断言
  首次执行全 PASS，首次 `cargo check --workspace` REAL_EXIT=0，一次通过、返工 0）。
  实现：`game/src/sim.rs` 新增 `PauseRequested`（unit 事件，`Event + Reflect + Serialize +
  Deserialize` + `#[reflect(Event, Serialize, Deserialize)]`，显式 `register_type`）+
  observer `on_pause_requested`（`On<PauseRequested>` + `ResMut<SimConfig>` 翻转 `paused`，
  `App::add_observer` 注册）；BRP `world.trigger_event`（省略 `value`）为唯一触发通道。
- 证据：`docs/evidence/ts-10-brp.md`（curl 命令原文 + 响应原文摘录 + 逐条判定 + 请求形态
  源码查证 + 终态复验 run2）；原始响应/运行日志/门禁日志 `docs/evidence/ts-10/`。
- 第二阶段（脚本判定，M1 校准重跑）：**PASS**（第二轮），2026-09-27 — 套件 `ts-10`
  （3 断言×2 调用）：初始 paused=false → `world.trigger` 触发后 **true** → 再触发
  **false**（翻转语义 + 套件净零；trigger 在 BRP handler 上下文同步执行 observer，
  与 BRP trigger_event 语义一致，第一阶段已证反射构造通路）。**首跑 FAIL 如实记录**：
  套件内两次 trigger 的 `second` 布尔实参交叉错位（断言名与期望错配），世界翻转语义
  自始正确（round1 详情可证 false→true→false 净零）；修复后复跑 3/3×2 全过退出码 0
  （round1 存档 `m1-phase2/ts-10/runner-round1.txt`，计入台账 T020 返工）。证据
  `docs/evidence/m1-phase2.md` §三/§四 + `docs/evidence/m1-phase2/ts-10/`。

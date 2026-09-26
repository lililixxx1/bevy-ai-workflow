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

- 判定者/日期/结论：执行 agent，2026-09-26 — **一次通过**（人工对照阶段；4/4 断言在进程 1 首次执行全部通过，返工 0，零改码）
- 证据：`docs/evidence/ts-01-brp.md`（curl 命令原文 + 响应原文摘录 + 判定；原始响应与运行日志在 `docs/evidence/ts-01/`；断言 4 的 `option` 字段形态核实于 `bevy_remote-0.19.1/src/builtin_methods.rs:383-396`）
- 第二阶段（run_tests 脚本）：**PASS**，2026-09-26 — `task-runner run --suite ts-01
  --expect-count 500 --expect-seed 20260926` 全自动判定：套件 5/5 断言通过 + 字面值
  绑定（经 `world.get_resources` 核对 entity_count/seed 与清单字面值一致）+ 跨调用
  `snapshot.tick` 递增通过，退出码 0（负例 `--expect-seed 1` 正确失败退出 1）。证据
  `docs/evidence/m2-rpc.md` 断言组 B/E（raw `m2-rpc/taskrunner-run1.txt` /
  `taskrunner-run2.txt` / `taskrunner-run2-negative.txt`，台账 T019）。进程内套件与本
  清单的对应：`game/src/rpc/suites/ts01.rs` 模块文档（断言 1/2 套件内为配置自洽口径，
  清单字面值由工具端 `--expect-*` 绑定；清单 #4 括注的「跨进程一致」为测量口径，进程内
  以初值重抽一致断言等价替代；断言 3 的第二采样由工具端两连调比对 `snapshot.tick` 完成）。

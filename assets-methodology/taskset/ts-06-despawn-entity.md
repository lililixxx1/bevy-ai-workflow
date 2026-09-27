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

> 断言 4 实测回填（2026-09-26 测量后按任务原文「回填本条」补齐）：错误码 **-23401**（`ENTITY_NOT_FOUND`，`bevy_remote-0.19.1/src/lib.rs:1409`；message 为 Entity Display 形态，如 `Entity 411v0 not found`）。测量期纪律为不改清单文件，回填延至测量完成、经 M1 审计确认后执行（见判定记录与 `docs/evidence/ts-06-brp.md`）。
| 5 | `world.despawn_entity` | `entity=e`（重复销毁） | 响应 `error` 存在 |

## 判定记录

- 判定者/日期/结论：执行 agent，2026-09-26，**通过（第一阶段人工对照）**——5/5 断言首次执行全部 PASS，零改码（`cargo check --workspace` REAL_EXIT=0 仍按公共前置执行）。断言 4 实测错误码 **-23401**（`ENTITY_NOT_FOUND`，`bevy_remote-0.19.1/src/lib.rs:1409`；message `Entity 411v0 not found`），断言 5 重复销毁同为 -23401——按本次任务纪律该回填值记录于证据文件与本节，清单表行未改写。
- 证据：`docs/evidence/ts-06-brp.md`（curl 命令原文 + 响应原文摘录 + 逐条判定）；原始响应/运行日志/门禁日志 `docs/evidence/ts-06/`。
- 第二阶段（脚本判定，M1 校准重跑）：**PASS**，2026-09-27 — 套件 `ts-06`（2 断言×2
  调用）：`despawn_removes_entity`（探针=真实 Wanderer 语义、index=u32::MAX 与真实
  序号隔离：spawn 后 101==100+1 → despawn 返回 true → 计数回 100 → `get_entity`
  Err；ECS 直写复测判定语义，BRP 通路第一阶段已证）、`double_despawn_fails`
  （重复 despawn 返回 **false**——BRP 侧即 -23401 ENTITY_NOT_FOUND 的进程内等价；
  终态计数==基线无残留）。字面值 100/20260926 + tick 递增。退出码 0。证据
  `docs/evidence/m1-phase2.md` §三 + `docs/evidence/m1-phase2/ts-06/`（台账 T020）。

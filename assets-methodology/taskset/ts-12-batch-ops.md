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

- 判定者/日期/结论：执行 agent，2026-09-26（两阶段之第一阶段：人工对照）。4/4 断言通过（w0=100/v0=0
  → 10 次成对操作 30 响应全无 error → w1=90/v1=10 净变化精确一致 → tick 5754<5834<5913 严格递增）；
  单进程（PID 23924）无重启，零改码，一次通过。
- 证据：`docs/evidence/ts-12-brp.md`（curl 命令原文 + 响应原文摘录 + 逐条判定）；原始响应/运行日志/门禁
  日志 `docs/evidence/ts-12/`。
- 第二阶段（脚本判定，M1 校准重跑）：**PASS**，2026-09-27 — 套件 `ts-12`（4 断言×2
  调用）：基线 w0=100==entity_count、v0=0；10 轮成对操作（spawn 探针 + despawn
  **剩余最小 index** 的真实 Wanderer——确定性取序，强于第一阶段依赖 archetype
  迭代序）全部成功；终态 w1=90==w0−10、v1=10==v0+10 净变化精确一致；恢复后
  w=100/v=0（探针清理 + 五组件快照重生，index/初值逐位还原，实体号允许变化——
  回归口径禁断言绝对 entity id）。#4 tick 递增由工具端两连调完成（246→308）。
  退出码 0。证据 `docs/evidence/m1-phase2.md` §三 + `docs/evidence/m1-phase2/ts-12/`
  （台账 T020）。round2（2026-09-27，审核 P1-3 修复后最终证据）：快照由五组件补
  为全组件（含 Tagged——最小 index 恰为 index<10 打标实体，旧版重生丢 Tagged 致
  组件集不等价），恢复断言加 Tagged 计数核对，实测 Tagged=10==t0(10)；
  `saturating_sub` 防下溢 + w0≥10 前置；4/4×2 调用全过退出码 0，tick 427→489。

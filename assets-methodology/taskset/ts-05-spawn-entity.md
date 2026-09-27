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

- 判定者/日期/结论：执行 agent，2026-09-26，**通过（第一阶段人工对照）**——4/4 断言首次执行全部通过，零改码，一次通过（返工 0）。e = 4294966777，`linear` 回读严格等于 `[1.0, 0.0, 2.0]`，无 Wanderer 的 Velocity 行恰 1 行且 entity == e。
- 证据：`docs/evidence/ts-05-brp.md`（curl 命令原文 + 响应原文摘录 + 逐条判定）；原始响应/运行日志/门禁日志 `docs/evidence/ts-05/`；台账 T010。
- 第二阶段（脚本判定，M1 校准重跑）：**PASS**，2026-09-27 — 套件 `ts-05`（3 断言×2
  调用）：`spawn_velocity_readback`（ECS 直写 spawn 后**同帧回读** linear==
  [1.0,0.0,2.0] 逐位；BRP 反射通路第一阶段已证，此处复测判定语义）、
  `velocity_only_isolation`（Velocity-only 计数 1 == 基线 0+1，等价清单 #4 的
  without 筛选）、`world_restored_after_suite`（探针 despawn 后计数回 0——套件
  无净副作用约定）。字面值 100/20260926 + tick 递增。退出码 0。证据
  `docs/evidence/m1-phase2.md` §三 + `docs/evidence/m1-phase2/ts-05/`（台账 T020）。
  round2（2026-09-27，审核 P2-1/P2-8 修复后最终证据）：断言 3→4——增
  `wanderer_baseline_matches_config`（清单 #1 进程内自洽：计数 100==entity_count），
  恢复改无条件 despawn；4/4×2 调用全过退出码 0，tick 411→473。

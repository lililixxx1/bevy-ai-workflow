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

- 判定者/日期/结论：执行 agent，2026-09-26 / **通过（人工对照第一阶段）**——4/4 断言首次执行全部 PASS；零改码（cargo check --workspace REAL_EXIT=0 仍按公共前置执行）；一次通过、返工 0。断言 4 数学复算：‖Δ‖=0.083956 ≤ 0.1（x/z 误差恰为 0，y 残差 0.0840 系 t2 与 Transform 两读间 O(1) 帧边界偏移，斜率上限 3.517 m/s × ≈23.9ms）。
- 证据：`docs/evidence/ts-07-brp.md`（curl 命令原文 + 响应原文摘录 + 独立复算过程 + 逐条判定）；原始响应/运行日志/门禁日志 `docs/evidence/ts-07/`
- 第二阶段（脚本判定，M1 校准重跑）：**PASS**，2026-09-27 — 套件 `ts-07`（3 断言×2
  调用）：`index0_located`（entity 419v0，origin/linear_v0/phase 记录）；
  `velocity_write_accepted_and_restored`（改写 linear→[0,0,0] 同帧回读逐位受理 +
  还原 v0 受理——即写即还原无净副作用；清单 #3 的 BRP mutate 通路第一阶段已证）；
  `transforms_match_analytic_formula`（**100 实体 Transform 与
  wanderer_translation(w,v,elapsed_secs) 复算逐位一致，最大偏差 0e0**——强于第一阶段
  跨帧采样容差 0.1 的口径：进程内 update_stats/move_swarm 同帧同条件 chain，BRP
  handler 帧间读到的 elapsed 即写 Transform 所用 t；改写的跨帧生效版本属第一阶段
  测量口径）。退出码 0。证据 `docs/evidence/m1-phase2.md` §三 +
  `docs/evidence/m1-phase2/ts-07/`（台账 T020）。

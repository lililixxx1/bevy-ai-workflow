# TS-11：阶梯性能回归验证

- 难度：中
- 前置：TS-01；基线数据 `docs/fps-baseline.md`
- 类型：验证

## 需求描述

不改代码。按基线口径（release、1280x720 窗口化、AutoVsync、种子 20260926、warmup 2s）以 `--count 50000 --bench-secs 10` 采集一次，对照基线同阶梯数据核对性能未回归（阈值：本机 avg_fps ≥ 基线值 × 0.8；跨机器不可比时只验证采集链路完整并如实记录）。

约束：采集期间不运行其他重负载任务；判定以 `[BENCH]` 汇总行与 BRP 读到的 `SimStats.avg_fps` 双源一致为准。

## 验收清单（BRP 断言 + 日志）

启动方式：`cargo run --release -p game -- --count 50000 --seed 20260926 --bench-secs 10`（后台，日志重定向文件）。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.get_resources` | `resource="game::sim::SimStats"`（进程退出前采样） | `avg_fps > 0` 且 `fps_1s > 0`（采集链路活） |
| 2 | `world.get_resources` | 同上 | `tick > 0`（50000 实体下模拟仍每帧推进） |
| 3 | 日志 | `[BENCH]` 汇总行 | 行内 `n=50000`、`avg_fps` 与 #1 读值量级一致（±10%）；warmup_secs=2.0 |
| 4 | 基线对照 | `docs/fps-baseline.md` 同阶梯 | avg_fps ≥ 基线 × 0.8（同机）或如实记录「跨机器不可比」 |

## 判定记录

- 判定者/日期/结论：执行 agent，2026-09-26，**通过**（4/4 断言首次采集运行全部通过；一次通过，返工 0，零改码）。实测 N=50000 AutoVsync：[BENCH] avg_fps=60.0 与 BRP 读 `SimStats.avg_fps=60.003` 双源一致（相对偏差 0.005%，远小于 ±10%）；基线同阶梯 vsync 组 60.0（同机），比值 1.000 ≥ 阈值 0.8，无回归。
- 证据：`docs/evidence/ts-11-brp.md`（curl 命令原文 + 响应原文 + [BENCH] 行 + 基线对照 + 工具级偏差如实记录）；原始文件 `docs/evidence/ts-11/`（run1 日志/退出码/轮询留痕/评估器输出）。
- 第二阶段（脚本判定，M1 校准重跑）：**PASS**，2026-09-27 — 套件 `ts-11`（2 断言×2
  调用，运行中）：avg_fps=59.695>0 且 fps_1s>0、tick>0 且实体 50000==启动值；工具端
  `bench-judge`（进程自退出后）四查全过：[BENCH] 行 `n=50000 warmup_secs=2.0
  measure_secs=8.016 measure_frames=476 avg_fps=59.4`，n 字面 PASS、warmup 口径
  PASS、双源一致 |59.4−59.695|/59.695=0.49%≤10% PASS、基线 59.4 ≥ 60×0.8=48 PASS
  （vsync 组同机口径；no-vync 组对照建议仍见 m1-acceptance §四第 5 条）。run 与
  bench-judge 退出码均 0。证据 `docs/evidence/m1-phase2.md` §三 +
  `docs/evidence/m1-phase2/ts-11/`（brp-avg.txt 为 BRP 读值留档，台账 T020）。

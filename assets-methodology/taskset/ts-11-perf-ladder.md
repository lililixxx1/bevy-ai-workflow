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

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）

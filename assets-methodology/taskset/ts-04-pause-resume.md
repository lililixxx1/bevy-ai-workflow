# TS-04：远程暂停/恢复模拟

- 难度：中
- 前置：TS-01
- 类型：功能（核对型：验证既有 paused 语义的 BRP 操控闭环——「状态注入 > 截图」的最小样例）

## 需求描述

不改动代码。通过 BRP 把 `SimConfig.paused` 置 true（暂停）再置回 false（恢复），核对模拟冻结/恢复的证据链完整。

约束：暂停/恢复必须全部经 BRP 完成（重启进程或改代码判定无效）；两次采样间须等待可观测的时间差（≥1 秒）。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 1000 --seed 20260926`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.get_resources` | `resource="game::sim::SimStats"` | 记 `tick1`；等待 ≥1s 再查，`tick2 > tick1`（运行中） |
| 2 | `world.mutate_resources` | `resource="game::sim::SimConfig"`, `path="paused"`, `value=true` | 响应无 error |
| 3 | `world.get_resources` | `resource="game::sim::SimStats"` | 记 `tick3`；等待 ≥1s 再查，`tick4 - tick3 <= 2`（已冻结；**容差 2 帧**——mutate 请求生效点与采样点之间存在帧边界竞态，实测会差恰好 1 tick，见 `docs/brp-smoke.md`） |
| 4 | `world.query` | `data.components=["game::sim::Wanderer","bevy_transform::components::transform::Transform"]` | 暂停期间两次采样（间隔 ≥1s），任一实体 Transform.translation 逐位相同 |
| 5 | `world.mutate_resources` | `path="paused"`, `value=false` | 恢复后再查 SimStats，`tick` 恢复增长 |

## 判定记录

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）

# TS-02：速度上限配置化

- 难度：中
- 前置：TS-01
- 类型：功能

## 需求描述

把写死在 `game/src/sim.rs` 的速度上限常量 `MAX_SPEED` 改为 `SimConfig` 的字段 `max_speed: f32`（默认 3.0，保持现口径不变），实体初速度生成改用该字段。

约束：遵守 SKILL.md（查证流程、组件设计）；`SimConfig` 新字段需带 `#[serde(default)]` 兼容旧序列化；CLI 增加 `--max-speed F`（可选，默认 3.0）；不改变 SplitMix64 的抽取顺序语义（除速度缩放外初值逐位不变——`origin` 与方向向量生成序列必须保持不变）。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 1000 --seed 20260926 --max-speed 9.0`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.get_resources` | `resource="game::sim::SimConfig"` | `max_speed == 9.0` |
| 2 | `world.query` | `data.components=["game::sim::Velocity"]` | 采样 ≥100 个实体：每个 `linear` 模长 ∈ (0, 9.0]；其中至少 1 个模长 > 3.0（新上限生效） |
| 3 | `world.query` | `data.components=["game::sim::Wanderer"]` | `origin` 字段与 `--max-speed 3.0`（默认）同 seed 同 count 运行的 `origin` 逐位一致（抽取顺序未破坏） |

## 判定记录

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）

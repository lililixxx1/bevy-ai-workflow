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

- 判定者/日期/结论：执行 agent（第一阶段人工对照），2026-09-26——**通过**。3/3 断言首次执行全部 PASS
  （进程 2 为断言 3 清单要求的默认上限对照口径，非重跑）；改码后首次 `cargo check --workspace` REAL_EXIT=0；
  一次通过、返工 0。实现口径注记：`#[serde(default)]` 约束按带路径形态
  `#[serde(default = "default_max_speed")]` 落地（缺字段默认 3.0 而非 0.0，依「默认 3.0，保持现口径不变」同句约束）。
- 证据：`docs/evidence/ts-02-brp.md`（curl 命令原文 + 响应原文摘录 + 逐条判定）；原始响应/运行日志/门禁日志
  `docs/evidence/ts-02/`。
- 第二阶段（脚本判定，M1 校准重跑）：**PASS**，2026-09-27 — 套件 `ts-02`（2 断言×2
  调用）：`velocity_within_configured_cap` 1000 实体模长全量 ∈ (0,9]（越界 0，最大
  8.9986）、>默认上限 3.0 的 675 个 ≥1（全量扫描强于清单「采样 ≥100」）；
  `origin_invariant_to_max_speed` 1000 实体 origin/phase 与默认上限重抽逐位一致
  （不符 0；清单 #3 跨进程对照以进程内重抽等价替换，`sim::draw_initial` 单一落点）。
  字面值：count 1000 / seed 20260926 / max_speed 9.0（--expect-max-speed）。退出码 0。
  证据 `docs/evidence/m1-phase2.md` §三 + `docs/evidence/m1-phase2/ts-02/`（台账 T020）。

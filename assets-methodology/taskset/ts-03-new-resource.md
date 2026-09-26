# TS-03：新增 BRP 可见 Resource

- 难度：低
- 前置：TS-01
- 类型：功能

## 需求描述

新增 Resource `SimMetadata`（模块路径定为 `game::sim::SimMetadata`），字段：`version: String`（crate 版本，取 `env!("CARGO_PKG_VERSION")`）、`entity_count: u32`、`seed: u64`。Startup 时填充。

约束：derive 形态与注册方式逐字对齐官方 BRP 示例（`Component/Resource + Reflect + Serialize + Deserialize`，`#[reflect(Resource, Serialize, Deserialize)]` + 插件 build 中 `register_type`，SKILL.md §3.4）；不引入运行时可变状态。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 250 --seed 7`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.list_resources` | 无 params | 结果数组包含 `"game::sim::SimMetadata"` |
| 2 | `world.get_resources` | `resource="game::sim::SimMetadata"` | `version == "0.1.0"`、`entity_count == 250`、`seed == 7` |

## 判定记录

- 判定者/日期/结论：执行 agent，2026-09-26，**两阶段第一阶段（人工对照）通过**——2/2 断言首次执行
  全部 PASS，首次 `cargo check --workspace` REAL_EXIT=0，一次通过、返工 0
- 证据：`docs/evidence/ts-03-brp.md`（curl 命令原文 + 响应原文摘录 + 逐条判定）；
  原始响应/运行日志/门禁日志 `docs/evidence/ts-03/`

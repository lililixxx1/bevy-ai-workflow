# TS-09：新组件类型注册与 schema

- 难度：中
- 前置：TS-03
- 类型：功能

## 需求描述

新增组件 `Tagged { tag: String }`（模块路径 `game::sim::Tagged`），按 SKILL.md §3.4 注册反射，并给 `index < 10` 的 Wanderer 实体加上该组件。

约束：注册必须显式（`register_type`），不依赖 `reflect_auto_register` 自动链；不得改动既有组件定义。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 100 --seed 20260926`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.list_components` | 无 params | 数组含 `"game::sim::Tagged"` |
| 2 | `registry.schema` | 过滤该类型（参数形态以 `rpc.discover` 返回的方法描述为准） | 返回的 schema 含字段 `tag`（string 类型） |
| 3 | `world.query` | `data.components=["game::sim::Tagged"], filter={"with":["game::sim::Wanderer"]}` | 行数 == 10 |
| 4 | 反证：临时注释掉 `register_type::<Tagged>()` 编译运行 | 重复 #1 | `list_components` 不含该路径（或 #3 报 `Unknown component type`）——验证「未注册类型 BRP 拿不到」（SKILL.md §3.4；验证后恢复代码） |

## 判定记录

- 判定者/日期/结论：执行 agent，2026-09-26 / **通过**（两阶段第一阶段人工对照；4/4 断言首次执行
  全 PASS，一次通过、返工 0；首次 `cargo check --workspace` REAL_EXIT=0）
- 证据：`docs/evidence/ts-09-brp.md`（curl 命令原文 + 响应原文摘录 + 逐条判定 + 反证/恢复全过程）；
  原始响应/运行日志/门禁日志 `docs/evidence/ts-09/`。两处如实注记（非放宽）：①`rpc.discover` 对
  `registry.schema` 不给参数描述（params:[]），参数形态按纪律回退本地源码（`BrpJsonSchemaQueryFilter.
  with_crates` crate 级过滤）；②断言 4 括注的 "Unknown component type" 文案在 query 路径实测为
  "isn't registered or used in the world"（同码 -23402），主分支（list_components 不含该路径）
  原样通过。实现按约束用 `#[reflect(..., no_auto_register)]` 使显式注册成为唯一通路（详见证据
  文件「实现口径」节；SKILL.md §3.4 已增补该已验证事实）。

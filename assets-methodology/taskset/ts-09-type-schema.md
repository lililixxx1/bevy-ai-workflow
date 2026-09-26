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

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）

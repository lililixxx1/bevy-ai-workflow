# G2 · 完整轮审核档（T047 M6 首波次 P4-7 节点卡）

日期 2026-10-30（晚）；审核者 plan-code-reviewer（**完整轮**——节点卡适用，含独立复算）；对象 = 树 `<tree>` 批次 T047-W1R 全量产出（game-td P4-7 + assert-td.mjs 断言面 + 证据档）。

> **档位注记**：本档为 Lead 对审核者报告的归档转录——审核者原报告全文在会话档未落盘，此处如实转录其结论、阻塞清单、独立复算数值与 Minor 原文，未增删判定语义。

## 结论

**通过**（0 阻塞 / 0 必改 / 1 Minor 非阻塞）。

## 审核面

- **①–⑬ 全 PASS**（逐项对照证据档行号引用，见审核者原报告）：验收 ①–⑬ 与 assert-td.mjs 五 phase 断言档一一对位——p4-assert2.log 21/21、p5-assert.log 21/21、p6-assert.log 21/21、p6b-assert.log 12/12、p7-assert.log 25/25，REAL_EXIT 均 0。
- **G1 裁决披露面复核**：U1–U4 / O1–O3 各自披露与档面一致（含 p6b 修复前首跑 10/12 被重跑覆盖之披露采信、p4-shot.png 1×1 对照件保留、变体 TOML 不入 assets/levels）。
- **代码面行为等价对照**（vs 教程行为规格）：tiling 内角/边缘规则（:137-154）、level 宽 3 展开与拐角（:82-143）、enemy 移动快照/吸附/不结转余量（:63-144）逐段核证等价；`#[type_path = "game_td"]` 必要性成立（bevy_remote 按完整类型路径查注册表）。

## 独立复算（审核者亲跑，非采信 worker 档）

| 项 | 结果 |
| --- | --- |
| toml-path-count.mjs 变体重跑（57/93/150） | EXIT=0 |
| toml-path-count.mjs 缺省重跑（45/105/150） | EXIT=0 |
| 截图 md5 链 ×5 | 全部一致 |
| 算术复核（速度 76.8 / +1.56% / 4.667s 窗） | 成立 |

## Minor（非阻塞——收获批已采纳）

- `game-td/src/map.rs:17`（原行号）注释把 `builtin_methods.rs:1183` 引作 `world.query` 的专属查表点——实际 `get_with_type_path` 调用点散布各 handler（`:1183` 属 mutate 路径），query 主路径另走 `get_component_ids`（`:857-940`）；建议措辞泛化。
- **处置**：收获批已改（「`bevy_remote-0.20.0` 多处经 `get_with_type_path` 解析，调用点散布 `builtin_methods.rs` 各 handler——如 `:1183`/`:1915` 等，非查询单点专属」）；复验 = Lead 对本地 registry 0.20.0 源码 grep `get_with_type_path` 命中**十处**（:1183/:1202/:1257/:1490/:1528/:1594/:1915/:1935/:1978/:2002）≥2 ✓ + `cargo check -p game-td`（随收获批主仓 workspace check 门禁覆盖，见 gate-check-main-t047.log）。

## 披露

- 可选重起游戏复跑 phase 断言：审核者未做（以档面证据 + 独立复算 + 代码面核证为判定面），如实披露不补录。

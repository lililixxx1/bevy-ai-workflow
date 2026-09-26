# PAT-B-XXX-<slug>（模板文件，复制后改名使用，勿直接填写本文件）

<!--
门禁（缺一不得入库，见 assets-methodology/pitfalls-schema.md）：
1. 代码过编译：cargo check --workspace 真实退出码 0；
2. 运行验证：过 BRP 闭环或等价运行验证（SKILL.md §4.2），证据写在下方「验证证据」。
代码必须来自本仓库 game/ 或 tooling/ 的真实代码（允许删减到最小可独立理解，
但不得引入未编译过的改写；删减后若未重新编译，须在「验证证据」中注明）。
-->

### PAT-B-XXX：<标题——一句话可检索的结论>

- 日期：YYYY-MM-DD
- 适用版本：bevy 0.19.x（按 Cargo.lock 实际解析版本填写）
- 分型：模式
- 通用性分级：bevy-specific（若换引擎仍成立 → 移交 assets-methodology/patterns.md）
- 标签：（检索关键词，空格分隔）
- 源码出处：`game/src/...:行号` 或 `tooling/.../...:行号`（真实代码所在地）

**场景**：<什么情况下用这条模式>

**做法**（已过 cargo check + 运行验证）：

```rust
// 真实代码（或其最小删减），保持可独立理解
```

**为什么**：<一两句原理/依据；引用源码处写明 路径:行号（核实 YYYY-MM-DD）>

**验证证据**：
- `cargo check --workspace` → REAL_EXIT=0（YYYY-MM-DD）
- 运行验证：<命令 + 断言结果摘录，如 BRP world.query 响应>

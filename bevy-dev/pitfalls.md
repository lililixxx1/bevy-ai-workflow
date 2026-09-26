# Bevy 特定层错题本

- 收录：通用性分级为 `bevy-specific`（换引擎后不再成立）的错题条目——API/版本/引擎行为相关的坑。
- 入库门禁与条目字段定义：见 [assets-methodology/pitfalls-schema.md](../assets-methodology/pitfalls-schema.md)（失败须在标注版本复现 + 修复过编译与运行验证；**未过验证禁止入库**）。
- 追加方式：按下方模板在文件末尾追加，`PIT-B-XXX` 三位自增（从 001 起），并在本行更新条数。
- 反例代码统一用 ```` ```compile_fail ```` 标记（doctest 断言其编译失败）；语义不符时用 `ignore` 并附理由（先例见 assets-methodology/pitfalls.md PIT-M-001 的 shell 命令处理）。
- 当前：0 条（骨架初始状态，2026-09-26）。

---

<!-- ===== 条目格式骨架（入库时复制本块并填写，勿留空字段）=====

### PIT-B-00X：<标题——一句话教训>

- 日期：YYYY-MM-DD
- 适用版本：bevy 0.19.x（按 Cargo.lock 实际解析版本；失败复现所在版本）
- 分型：错题
- 通用性分级：bevy-specific（若换引擎仍成立 → 移交 assets-methodology/pitfalls.md）
- 标签：（检索关键词）

**现象**：<失败原文 / 最小复现步骤；报错摘录须保留关键行>

**最小复现**：

```compile_fail
// 反例代码（在上方标注版本上复现失败；语义不符时改用 `ignore` 并在此说明理由）
```

**根因**：<一段话；引用源码处写明 路径:行号（核实 YYYY-MM-DD）>

**修复**（已过 cargo check + 运行验证）：

```rust
// 正例代码
```

**验证证据**：
- 复现：<命令与失败输出摘要>（YYYY-MM-DD）
- `cargo check --workspace` → REAL_EXIT=0
- 运行验证：<命令 + 断言结果摘录>

===== 骨架结束 ===== -->

# 错题条目 schema（含通用性分级字段）

依据：意向文档 §5.2「资产入库门禁」（Voyager 原则：**未过验证的条目禁止入库**）。

## 条目分型

### 模式条目（正例）

- 定义：本项目验证过的、可复用的正确做法/代码。
- 入库门禁：代码**过编译**（`cargo check`）+ **运行验证**通过后才可入库。
- 落点：Bevy 特定 → `bevy-dev/patterns/`；方法论级 → `assets-methodology/patterns.md`。

### 错题条目（反例）

- 定义：踩坑记录（错误做法、误解、失败路径）及其修复。
- 入库门禁（两条都过才入库）：
  1. 失败须在**标注版本上复现**——附最小复现代码或失败原文；
  2. 修复方案**过编译与运行验证**。
- 落点：Bevy 特定 → `bevy-dev/pitfalls.md`；方法论级 → `assets-methodology/pitfalls.md`。
- 反例代码统一用 ` ```compile_fail ` 标记（doctest 断言其编译失败）；语义不符时用 `ignore` 并附理由。
  - 适用面注记（2026-09-27）：`bevy-dev/pitfalls.md` 已纳入 `cargo test --doc -p docs`（经 `docs/src/lib.rs`），其 compile_fail 反例为**机器断言**；方法论层 `assets-methodology/pitfalls.md` 未入门禁，仍为标记约定。

## 通用性分级字段

| 分级值 | 判定 | 落点 |
|---|---|---|
| `bevy-specific` | 换引擎后不再成立（API/版本/引擎行为相关） | `bevy-dev/` |
| `methodology` | 换引擎后仍成立（流程/验证/协作纪律相关） | `assets-methodology/` |

判定标准：**换引擎后该教训是否仍然成立**（意向文档 §6 分层纪律）。

## 条目字段

| 字段 | 必填 | 说明 |
|---|---|---|
| id | 是 | 错题：`PIT-B-001`（bevy）/ `PIT-M-001`（方法论）；模式：`PAT-B-001` / `PAT-M-001`；各自独立递增 |
| 标题 | 是 | 一句话教训（可检索） |
| 适用版本 | bevy 条目必填 | 如 `bevy 0.19.1`；方法论条目可省 |
| 日期 | 是 | `YYYY-MM-DD` |
| 分型 | 是 | `模式` / `错题` |
| 通用性分级 | 是 | `bevy-specific` / `methodology` |
| 现象/失败原文 | 错题必填 | 最小复现或失败原文（报错摘录、命令与输出） |
| 根因 | 是 | 一段话 |
| 修复/正例 | 是 | 过编译+运行验证的代码；反例部分用 `compile_fail` 标记 |
| 验证证据 | 是 | 验证命令与结果（如 `cargo check` 通过、`cargo test --doc` 通过、BRP 断言输出） |
| 标签 | 否 | 便于检索的关键词 |

## 模板

### 错题条目模板

```markdown
### PIT-B-00X：<标题>

- 日期：YYYY-MM-DD
- 适用版本：bevy 0.19.x
- 分型：错题
- 通用性分级：bevy-specific（或 methodology → 落 assets-methodology/pitfalls.md）
- 标签：

**现象**：<失败原文 / 最小复现步骤>

**最小复现**：

```compile_fail
// 反例代码（在标注版本上复现失败）
```

**根因**：<一段话>

**修复**（已过 cargo check + 运行验证）：

```rust
// 正例代码
```

**验证证据**：<命令与结果>
```

### 模式条目模板

```markdown
### PAT-B-00X：<标题>

- 日期：YYYY-MM-DD
- 适用版本：bevy 0.19.x
- 分型：模式
- 通用性分级：bevy-specific（或 methodology）
- 标签：

**场景**：<何时用>

**做法**（已过 cargo check + 运行验证）：

```rust
// 真实代码
```

**验证证据**：<命令与结果>
```

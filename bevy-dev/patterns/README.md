# patterns/（Bevy 特定层模式库）

- 收录：通用性分级为 `bevy-specific` 的模式条目（正例）——**本项目编译通过且运行验证过的真实代码**（意向文档 §6；入库门禁见 [assets-methodology/pitfalls-schema.md](../../assets-methodology/pitfalls-schema.md)）。
- 条目形式（SKILL.md v0 定案）：**一条一文件**，命名 `PAT-B-XXX-<slug>.md`（XXX 三位自增，从 001 起；slug 用小写短横线英文），从 [_TEMPLATE.md](./_TEMPLATE.md) 复制起手。
- 入库步骤：
  1. 确认门禁已过：代码过 `cargo check` + 运行验证（BRP 闭环，SKILL.md §4.2）；
  2. 复制 `_TEMPLATE.md` → 改名 `PAT-B-XXX-slug.md` → 填完全部必填字段（模板内注释一并删除）；
  3. 在下方索引表加一行；
  4. 反例形态的教训不进本目录——走 `bevy-dev/pitfalls.md`。
- 当前：0 条（骨架初始状态，2026-09-26）。

## 索引

| id | 标题 | 适用版本 | 文件 | 验证证据（命令与结果） |
|---|---|---|---|---|
| （暂无条目） | | | | |

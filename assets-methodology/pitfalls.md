# 方法论级错题本（引擎无关）

- 收录：通用性分级为 `methodology`（换引擎后仍成立）的错题条目。
- 入库门禁与条目模板：见 [pitfalls-schema.md](./pitfalls-schema.md)（未过验证的条目禁止入库）。
- 当前：1 条。

---

### PIT-M-001：依赖 MSRV 高于本地工具链时 cargo 在编译前直接拒绝；管道转发会吞掉真实退出码

- 日期：2026-09-26
- 适用版本：方法论级（无引擎版本要求；涉及 cargo 的 MSRV 感知解析与 shell 管道语义）
- 分型：错题
- 通用性分级：methodology（换引擎/换依赖后同样成立）
- 标签：`MSRV` `工具链` `退出码` `shell管道`

**现象**：首次 `cargo check --workspace` 失败（发生在依赖下载完成后、任何编译开始前）：

```text
error: rustc 1.94.0 is not supported by the following packages:
  bevy@0.19.1 requires rustc 1.95.0
  bevy_ecs@0.19.1 requires rustc 1.95.0
Either upgrade rustc or select compatible dependency versions with
`cargo update <name>@<current-ver> --precise <compatible-ver>`
```

同时，该命令当时以 `cargo check --workspace 2>&1 | tail -30` 形式后台执行，任务通知报「exit code 0」——退出码来自管道末端的 `tail`，真实失败险些被漏判。

**最小复现**（反例命令；shell 命令无法用 `compile_fail` 标记，以文本形式给出，语义不符理由如前）：

```text
# 工具链 rustc 1.94.0 + 依赖 bevy 0.19.1（rust-version = "1.95.0"）时：
cargo check --workspace        # 必然报上述 error，未编译任何包
cargo check --workspace | tail -30; echo $?   # 反例：$? 是 tail 的退出码 0，不是 cargo 的
```

**根因**：

1. 本地 stable 工具链（rustc 1.94.0，2026-03 发布）落后于 bevy 0.19.1 声明的 `rust-version = "1.95.0"`（本地源码 `bevy-0.19.1/Cargo.toml` 第 14 行）；cargo 的 MSRV 感知解析在解析阶段即拒绝，报错不涉及任何业务代码。
2. POSIX 管道的退出码取自最后一个命令；`cargo ... | tail` 的失败会被 `tail` 的 0 覆盖。

**修复**（已过验证）：

1. `rustup update stable`（1.94.0 → 1.98.1）——符合仓库纪律「Rust 为 Bevy MSRV 及以上的最新 stable」（意向文档 §6）；不采用 `cargo update --precise` 降级 bevy，因版本锁已定案 0.19.x 系列。
2. 长时命令的成败判定一律看真实退出码：`cargo check ... > log 2>&1; echo REAL_EXIT=$?`，或用 `${PIPESTATUS[0]}`。

**验证证据**：

- 修复后 `cargo check --workspace`：`REAL_EXIT=0`（Checking game/docs 均过，1m44s）；
- `cargo test --doc -p docs`：`REAL_EXIT=0`（2 passed; 0 failed）。

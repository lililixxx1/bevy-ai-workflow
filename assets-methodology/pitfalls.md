# 方法论级错题本（引擎无关）

- 收录：通用性分级为 `methodology`（换引擎后仍成立）的错题条目。
- 入库门禁与条目模板：见 [pitfalls-schema.md](./pitfalls-schema.md)（未过验证的条目禁止入库）。
- 当前：3 条。

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

### PIT-M-002：测试参考向量不得凭记忆书写——算法常数的「正确感」也是幻觉

- 日期：2026-09-26
- 适用版本：方法论级（无引擎版本要求；实例为 SplitMix64 参考向量，坑本身与语言/引擎无关）
- 分型：错题
- 通用性分级：methodology（换引擎/换语言后仍成立：任何「我应该记得这个值」的常量、哈希、编码、签名都是同一坑）
- 标签：`参考向量` `记忆不可信` `单元测试` `幻觉`

**现象**：为 SplitMix64（seed=0）写单测参考向量时，前两个值凭训练记忆直接写出 `0xE220A87510B23965` / `0x6C789E6AA1B965F4`。独立实现（node BigInt，同算法常数）计算的真实值为 `0xE220A8397B1DCDAF` / `0x6E789E6AA1B965F4`——前 16 进制位相同（`0xE220`/`0x06C4` 级别的开头碰对），主体错误。若未先验证直接提交，测试会以「看似可信的错误期望值」污染回归集。

**最小复现**（Rust 测试断言；`compile_fail` 语义不符——代码可编译，失败在断言，故用 `ignore` 并附理由：本块还原的是**运行期断言失败**，非编译失败）：

```rust,ignore
// 反例：期望值凭记忆书写（真实输出是 0xE220A8397B1DCDAF）
let mut r = SplitMix64::new(0);
assert_eq!(r.next_u64(), 0xE220_A875_10B2_3965);  // 运行期 panic
```

实测复现（2026-09-26，`cargo test -p game splitmix64_reference`）：

```text
assertion `left == right` failed
  left: 16294208416658607535    (真实输出)
 right: 16294208672571210085    (记忆期望值)
test result: FAILED. 0 passed; 1 failed; ... REAL_EXIT=101
```

**根因**：训练记忆对「曾经见过的高熵常量」只保留模糊印迹，输出时以貌似合理的十六进制填充细节——与 API 幻觉同源，只是对象从签名换成了数值。开头几位碰对反而提高了蒙混概率。

**修复**（已过 cargo test）：

1. 用**独立实现**（另一语言/另一份代码）计算参考向量后再写断言；本例用 node BigInt 按公开算法常数独立计算；
2. 在测试注释中注明向量来源与计算方式，供升级窗口复核。

```rust
// 正例（节选自 game/src/rng.rs，已过 cargo test -p game）
// 参考向量由独立实现（node BigInt，同算法常数）计算，防手误。
let mut r = SplitMix64::new(0);
assert_eq!(r.next_u64(), 0xE220_A839_7B1D_CDAF);
assert_eq!(r.next_u64(), 0x6E78_9E6A_A1B9_65F4);
assert_eq!(r.next_u64(), 0x06C4_5D18_8009_454F);
```

**验证证据**：

- 复现：记忆向量版本 `cargo test -p game splitmix64_reference` → FAILED，REAL_EXIT=101（输出见上，2026-09-26）；
- 修复：独立计算向量替换后同命令 → `test result: ok. 1 passed`（全量 `cargo test -p game` 8 passed）。

### PIT-M-003：workspace 成员 glob 会把非 crate 目录吸入解析链——证据目录与 crate 目录不能共用一个命名空间

- 日期：2026-09-26
- 适用版本：方法论级（无引擎版本要求；实例为 Cargo workspace glob 成员，坑模式适用于任何「构建系统按目录隐式发现成员」的栈）
- 分型：错题
- 通用性分级：methodology（换引擎后同型坑仍在：工程工具按目录约定隐式注册成员时，非代码目录混入即破坏构建）
- 标签：`workspace` `glob 成员` `目录职责混放` `证据管理`

**现象**：T004 把 BRP 验证证据放在 `tooling/brp-logs/`（纯日志目录，无 Cargo.toml）。根 workspace 用 `members = ["tooling/*"]` glob。T005 在 `tooling/hotpatch-smoke/` 新建 crate 后，**任何** cargo 命令（包括 `cargo check -p hotpatch-smoke`）在解析阶段即失败，尚未编译任何代码：

```text
error: failed to load manifest for workspace member `...\tooling\brp-logs`
referenced via `tooling/*` by workspace at `...\bevy-ai-workflow\Cargo.toml`

Caused by:
  failed to read `...\tooling\brp-logs\Cargo.toml`

Caused by:
  系统找不到指定的文件。 (os error 2)
```

**最小复现**（shell 语义；`compile_fail` 不适用——失败在 cargo 解析而非 rustc 编译，故用 `ignore` 并附理由：还原的是**构建系统报错**，非编译失败）：

```text,ignore
# 结构：根 Cargo.toml 含 members=["tooling/*"]，tooling/brp-logs/ 存在但无 Cargo.toml
cargo check -p <任何成员>
# → error: failed to load manifest for workspace member `tooling/brp-logs`（os error 2）
```

**根因**：`members` glob 把每个直接子目录都视为候选成员，候选必须可解析为 package（含 Cargo.toml）；解析失败发生在 workspace 装配阶段，先于目标选择，故 `-p` 无法绕开。深层原因是**目录职责混放**：证据资产目录（长期入库、会被 glob 吸入）与 crate 目录（构建系统成员）共用 `tooling/` 命名空间。

**修复**（已过编译与运行验证）：

1. 根 workspace 加 `exclude = ["tooling/brp-logs"]`（exclude 优先于 glob 匹配）；
2. 规约：`tooling/` 下只放 crate；证据目录放 crate 内部（如 `tooling/hotpatch-smoke/evidence/`，非 `tooling/*` 直接子级，不入 glob 命名空间）或 docs 侧。

```toml
# 正例（根 Cargo.toml，修复后 cargo check --workspace 通过）
[workspace]
resolver = "2"
members = ["game", "docs", "tooling/*"]
exclude = ["tooling/brp-logs"]
```

**验证证据**：

- 复现：`cargo check -p hotpatch-smoke` → 上述报错（2026-09-26）；
- 修复：加 exclude 后 `cargo check -p hotpatch-smoke` → `Finished dev profile ... in 0.78s`；`cargo check --workspace` 亦通过；冒烟全流程（dx serve --hot-patch 两次热补丁）正常执行。

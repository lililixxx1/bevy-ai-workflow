# 方法论级错题本（引擎无关）

- 收录：通用性分级为 `methodology`（换引擎后仍成立）的错题条目。
- 入库门禁与条目模板：见 [pitfalls-schema.md](./pitfalls-schema.md)（未过验证的条目禁止入库）。
- 当前：5 条。

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

### PIT-M-004：证据摘录不得徒手誊写——「从 raw 复制值进成文」本身就是失真源

- 日期：2026-09-26
- 适用版本：方法论级（无引擎版本要求；实例为 BRP 响应摘录，坑模式适用于任何「原始记录 → 成文证据」的搬运）
- 分型：错题
- 通用性分级：methodology（换引擎/换工具后同样成立：日志、截图 OCR、测量读数的手工誊写同型）
- 标签：`证据摘录` `誊写失真` `验证闭环`

**现象**：M1 测量 12 任务的证据成文中，独立审计做全量数字扫描（证据文件中所有 ≥7 位数字/高精度小数逐一对回 raw 目录文本），发现 2 处摘录失真：

1. `docs/evidence/ts-05-brp.md` 断言 1 的「raw 首行原文」——`origin`/`phase` 值在 raw 中不存在（誊写时从记忆/其他行拼出）；该失真由后续任务的执行 agent 交叉发现并披露，但文件未即时修正；
2. `docs/evidence/ts-01-brp.md` 断言 3 读 2 的 `fps_1s`——末段 8 位数字誊错（`60.11129322958322` vs raw `60.11129655829322`）。

两处均不影响断言判定（判定以 raw 为准），但若下游把摘录当数据源引用（如汇总报告、置信计算），失真会无声扩散。

**最小复现**（流程性反例；`compile_fail` 不适用——非代码缺陷，故用 `ignore` 并附理由：还原的是**成文流程缺陷**，非编译/运行失败）：

```text,ignore
# 反例流程：人/agent 凭「刚看过 raw」的印象把响应值敲进 Markdown
# （高精度浮点与长位串的短时记忆不可靠，开头几位碰对率高，末段最易错）
1. cat raw/a1.json | 肉眼扫一遍
2. 打开 evidence.md，凭记忆键入 origin:[-70.78…, -63.55…]   # 末段失真
# 正例流程：摘录一律从 raw 文件复制粘贴，或用脚本抽取；定稿前逐字回对
```

**根因**：短时记忆对高熵数值（长浮点、位串、十六进制）只保留前缀印迹，末段以貌似合理值填充——与 PIT-M-002（参考向量凭记忆书写）同源，对象从「算法常数」换到「实测响应」。交叉发现（后续任务对同一 raw 的独立摘录逐位比对）能兜住一部分，但兜不全（本批 12 任务仅 1 处被顺带发现）。

**修复**（已落实并验证）：

1. 摘录纪律入 `assets-methodology/taskset/README.md` 公共前置：**摘录必须逐字回对 raw 原文后才可定稿**；
2. 已发现的 2 处按 raw 逐字改正并加带日期勘误注（ts-01/ts-05 证据文件）；
3. 审计方法沉淀：对成文证据做「全量数字扫描对回 raw」可作为独立审计的标准动作（本次即由该扫描发现）。

**验证证据**：

- 复现：M1 审计全量数字扫描报告（2026-09-26，plan-code-reviewer）——12 份证据中不匹配 raw 的仅上述 2 处，其余高精度数字均为可独立复算的派生值；
- 修复后复扫：2 处勘误落实后，摘录与 raw 一致（勘误注保留审计轨迹）。

### PIT-M-005：类 Unix shell 调 Windows 原生命令时，斜杠开关会被 MSYS 路径转换破坏

- 日期：2026-09-26
- 适用版本：方法论级（无引擎版本要求；实例为 Git Bash 调 taskkill，坑模式适用于任何「POSIX 风格 shell 调用另族原生命令」的组合）
- 分型：错题
- 通用性分级：methodology（换引擎后同样成立：进程收尾/系统调用与 shell 族无关）
- 标签：`Git Bash` `MSYS 路径转换` `taskkill` `进程收尾`

**现象**：运行时验证收尾执行 `taskkill /F /IM game.exe`，命令失败且**目标进程未被终止**：

```text
错误: 无效参数/选项 - 'F:/'。
键入 "TASKKILL /?" 以了解用法。
```

`/F` 被 MSYS 当作 POSIX 路径转换为 `F:/`（盘符根），taskkill 收到的是非法开关。若只看「命令已执行」不核对退出码与进程清单，残留进程会占用 BRP 端口与 GPU，静默污染下一次运行时验证。

**最小复现**（shell 语义；`compile_fail` 不适用——非编译失败，故用 `ignore` 并附理由：还原的是 **shell 参数转换失败**）：

```text,ignore
# Git Bash（MSYS）下：
taskkill /F /IM game.exe     # → 错误: 无效参数/选项 - 'F:/'，进程未终止
taskkill //F //IM game.exe   # 双斜杠转义后正常
```

**根因**：MSYS 的启发式路径转换把以单个 `/` 开头的参数视作 POSIX 路径并映射为 Windows 路径（`/F` → `F:/`）；`//` 开头不被视为路径，原样传递后在 Win32 侧等价单斜杠开关。

**修复**（已过验证）：

1. 单斜杠开关改双斜杠（`taskkill //F //IM game.exe`），或环境变量 `MSYS_NO_PATHCONV=1` 一次性关闭转换；
2. 收尾纪律：强杀后必须 `tasklist | grep <进程>` 核对无残留（本仓库 SKILL.md §4.2 已有「验证完杀干净游戏进程」，本条补上「怎么杀才杀得掉」）。

**验证证据**：

- 复现：2026-09-26，`taskkill /F /IM game.exe` → 上述报错原文，`tasklist` 仍见 game.exe（PID 6720）；
- 修复：`taskkill //F //IM game.exe` → `成功: 已终止进程 "game.exe"，其 PID 为 6720`，复查 tasklist 无残留。
- 既有同因先例：T007（TS-02 执行期）与 `docs/evidence/ts-02-brp.md:96` 已记录「taskkill /F /IM 被路径转义吞参，改 MSYS_NO_PATHCONV=1」——本条是该坑的正式入库（含双斜杠等价修复形态）。（勘误注 2026-09-26：初版误把 TS-11/TS-12 当旁证——两任务收尾均为 PID 定点 taskkill 且核对无残留，属不同例；经独立审核指正后改正。）

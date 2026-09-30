# 方法论级错题本（引擎无关）

- 收录：通用性分级为 `methodology`（换引擎后仍成立）的错题条目。
- 入库门禁与条目模板：见 [pitfalls-schema.md](./pitfalls-schema.md)（未过验证的条目禁止入库）。
- 当前：11 条（2026-09-30 增 PIT-M-011——doctest 门禁负载敏感假红三档对照入库；2026-09-29 增 PIT-M-010——T040 驱动顺序假红复现入库）。

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

---

### PIT-M-006：长脚本按 `$0` 相对形态推断仓库根，层级差一导致全部产物落错目录

- 日期：2026-09-27
- 适用版本：方法论级（bash 脚本路径语义，与引擎无关）
- 分型：错题
- 通用性分级：methodology（换引擎/换语言后同类翻车同样成立）
- 标签：`bash` `路径锚定` `CWD` `驱动脚本`

**现象**：M1 校准重跑驱动脚本首版以 `cd "$(dirname "$0")/../.."` 锚定仓库根。脚本实际位于仓库根下**三级**目录（`docs/evidence/m1-phase2/`），上溯两级落在 `<repo>/docs/`——此后所有相对路径（summary、逐任务证据目录、`./target/release/game.exe`）全部写去 `docs/docs/...` 杂散树；游戏从未启动，BRP 就绪探测 20s 超时，11 任务全军覆没。最迷惑处：脚本本身继续"正常"输出（echo/tee 到 stdout 依旧打印），错误被 `> 重定向` 吞进错误目录里的日志文件，控制台无任何报错。

**最小复现**（shell 语义；`compile_fail` 不适用——非编译失败，故用 `ignore` 并附理由：还原的是 **CWD 锚定层级错误**；存档 `docs/evidence/m1-phase2/pit-m006-repro.txt`）：

```text,ignore
# 脚本位于 <repo>/docs/evidence/m1-phase2/（根下三级）：
cd -- "$(dirname "$0")/../.." && pwd    # → <repo>/docs      ← 错（差一级）
cd -- "$(dirname "$0")/../../.." && pwd # → <repo>           ← 对
```

**根因**：按「脚本相对自身定位」的直觉写上溯层级时，把脚本所在目录本身当成了一级；且 `$0` 的相对/绝对形态随调用方式变化，锚定又叠加依赖调用方 CWD 的隐含假设。

**修复**（已过验证）：

1. 锚定改为「先取脚本真实目录再上溯」，不依赖调用方 CWD：`SCRIPT_DIR=$(cd -- "$(dirname -- "$0")" && pwd); REPO=$(cd -- "$SCRIPT_DIR/../../.." && pwd)`，此后所有路径一律 `$REPO/...` 绝对引用；
2. 锚定后立即做一次可观测自检（如 `mkdir -p "$ROOT"` 建 summary 前置文件），首行失败当场暴露而不是 11 任务后复盘。

**验证证据**：

- 复现：2026-09-27，`bash -x` 逐行追到 `cd docs/evidence/m1-phase2/../..` 后 `mkdir -p docs/evidence/m1-phase2/ts-01` 实建 `<repo>/docs/docs/...`，`game.log` 内容为 `./target/release/game.exe: No such file or directory`（杂散树已清理）；
- 修复：同日重跑（绝对锚定版）12 任务全部真实执行，产物落位正确（`docs/evidence/m1-phase2/summary.txt`）。

---

### PIT-M-007：前台短命调用里 `&` 起的常驻子进程数秒内被回收——跨调用存活的进程必须由长生命周期后台任务托管

- 日期：2026-09-27
- 适用版本：方法论级（本仓库 AI 驻场工作流的执行环境语义；与引擎无关）
- 分型：错题
- 通用性分级：methodology（换引擎后同样成立；对任何「工具调用即进程组」的 agent 执行环境有效）
- 标签：`进程生命周期` `后台任务` `驱动脚本` `运行时验证`

**现象**：在前台工具调用内 `./game.exe ... &` 启动常驻游戏（BRP 服务），主命令 sleep 期间游戏日志正常推进（[STATS] t=3.0s），随后进程消失——同调用内 tasklist 已查无 game.exe，端口连接被拒。游戏"看起来启动成功、日志健康"，但活不过几秒，后续判定全部踩空。

**根因**：前台工具调用按「调用结束即清理」管理其进程组/作业对象，调用内 `&` 起的分离子进程不获豁免（甚至在调用进行中即被回收）；需要跨调用、跨分钟存活的常驻进程（游戏 + BRP 监听）必须挂在一个长生命周期的宿主下。

**修复**（已过验证）：

1. 常驻进程由**后台任务**托管：要么整个驱动脚本作为一个后台任务运行（其 `&` 子进程随宿主存活，M1 校准重跑 12 任务实测），要么游戏单独以后台任务启动、判定调用随后单独执行；
2. 驱动脚本内加**存活快速失败**：启动 +2s `tasklist` 查进程，早夭即判失败并带日志末行入汇总，不空耗 20s 就绪探测。

**验证证据**：

- 复现：2026-09-27，前台调用内 `game.exe &` → 日志止于 t=3.0s、tasklist 无进程、curl 连接拒绝；
- 修复：同日驱动脚本以后台任务运行，其内 11 个游戏子进程各自存活至任务完成（BRP 就绪 + 套件判定 + taskkill 收尾全链成功，`docs/evidence/m1-phase2/summary.txt` 全绿）。

---

### PIT-M-008：重负载 doctest 门禁默认按核数并发拉起 rustc——不限流不限优先级可把宿主机打满卡死；退出码须走 `$LASTEXITCODE` 直通链

- 日期：2026-09-28
- 适用版本：方法论级（rustdoc doctest 并发模型与进程优先级语义，任意工具链版本；agent 执行环境）
- 分型：错题
- 通用性分级：methodology（换引擎/换语言后的重负载构建门禁同样成立）
- 标签：`doctest 门禁` `资源治理` `并发限流` `进程优先级` `退出码`

**现象**：`cargo test --doc -p docs`（94 个 doctest，其中十余个运行型 fence 各需独立编译+链接 bevy）作为后台任务裸跑：rustdoc 按可用并行度（本机 12 逻辑核）同时进入十余个 doctest 的编译/链接段，全核占满 + 内存峰值，桌面系统失去响应，只能硬重启（一次卡死复现；同并发形态另有一次因会话中断被杀，两份截断日志同型）。另有次生坑：为治理资源占用而写的门禁启动器，两版都在**退出码**上失真——`Start-Process -PassThru` 的进程对象经 `Wait-Process -Id` 后 `$p.ExitCode` 取到 `null`（`exit $null` → 0），换 `$p.WaitForExit()` 则在该对象上不阻塞立即返回（本机 `cargo` 实为 rustup shim，进程对象行为不可靠），外层随即把假的 `REAL_EXIT=0` 写进日志，产出「`test result: FAILED` 但 REAL_EXIT=0」的自相矛盾证据。

**根因**：

1. rustdoc 的 doctest 执行线程池默认 = `available_parallelism`，而每个运行型 doctest 的「执行」含一次完整 rustc 编译+链接（重依赖 crate 下链接是主要开销，单进程 1–2GB 内存 + 多核代码生成）；后台任务对资源占用无任何约束，Windows 上后台编译与前台 UI 同优先级抢调度；
2. 证据链的退出码是最脆弱一环：`Start-Process -PassThru` 返回的进程对象在 PowerShell 5.1 下 `ExitCode`/`WaitForExit()` 行为依进程形态（直接 exe vs shim 代理链）不稳定，任何「进程对象取码」方案都比「前台执行 + `$LASTEXITCODE` 直通」多一截不可靠面。

**修复**（已过验证）：

1. 启动器先自降优先级再前台执行：PowerShell 内 `(Get-Process -Id $PID).PriorityClass = 'BelowNormal'`（后续 cargo/rustdoc/rustc 子进程全部继承），再 `& cargo test --doc ...`——即使 CPU 仍被占满，前台/UI 调度上抢得过它，系统不失响应；
2. `--test-threads 4` 显式限并发（并发链接的内存峰值压到 ~6–8GB 量级，31.8GB 内存下安全；代价是墙钟拉长约 50%）；
3. 退出码 `exit $LASTEXITCODE` 直通 cargo 退出码，文件重定向交外层 shell 完成（与既往 UTF-8 门禁日志同格式），全程不碰进程对象。

**验证证据**：

- 复现：2026-09-28，无限流两轮日志均截断于 ≥12 个 doctest 并行编译/链接段（`docs/evidence/m3-assets/batch-g/gate-doc-test-attempt1-interrupted.log` 会话中断、`gate-doc-test-attempt2-killed-reboot.log` 伴随系统卡死重启）；退出码失真两例见 `gate-doc-test-attempt3-first-fail.log`（FAILED + REAL_EXIT=0 矛盾）与 `gate-doc-test-attempt4-note.txt`（WaitForExit 不阻塞致日志污损）；
- 修复：限流形态（BelowNormal + 4 线程）连续三轮运行期间系统全程可交互；attempt5 干净全绿（77 passed / 0 failed / 17 ignored，REAL_EXIT=0，`gate-doc-test.log`）；启动器最终形态与演化史见 `run-doc-gate.ps1` 头注。

### PIT-M-009：BRP/HTTP 响应中的 u64 在 JS 工具端经 `JSON.parse` 降精度——f64 渲染值不得当精确基线

- 日期：2026-09-28
- 适用版本：方法论级（IEEE 754 f64 只有 53 bit 尾数；任意 >2^53 整数经 JS Number 往返即失真，与引擎/协议无关）
- 分型：错题
- 通用性分级：methodology（换引擎、换 RPC 框架后同样成立）
- 标签：`u64` `JSON 精度` `JS Number` `证据链` `重放基线`

**现象**：BRP `world.get_resources` 返回的 `BattleState.rng_state`（u64，本例 ≈1.58e19 > 2^53）——curl 直写文件的**原始字节**为 `15755400384260043846`；同一响应在 node 驱动里 `await res.json()` 解析后 `JSON.stringify` 回写 transcript 变成 `15755400384260045000`。成文证据（任务书/台账/证据文档）凡摘自 node 侧的该值，字面上全是 f64 渲染值而非真值（M4 T033 首版三处中招，R1 审核以独立复算 + 活体 curl 原文抓出）。

**根因**：JS `Number` 是 IEEE 754 双精度浮点，安全整数域 `Number.MAX_SAFE_INTEGER = 2^53-1`；`JSON.parse` 把整数字符串按 Number 解析即最近舍入，`JSON.stringify` 再渲染成十进制——往返两端一致但都不等于原始 u64。serde_json/BRP 侧输出本身是精确的，失真只发生在 JS 工具链内部。

**修复**（已过验证）：

1. 凡 u64 量级字段（种子/RNG 状态/大计数）需要**字面基线**时：curl 直写文件留原始字节为权威证据，成文摘录以该文件为准；
2. JS 侧需要精确比较时用文本级提取（`raw.match(/"rng_state":(\d+)/)`）或 BigInt，不经 `JSON.parse` 中转；
3. 仅做「两侧等值」判定（如同种子重放 diff）时，f64 渲染下的比对仍强（随机分歧落入同一 double 桶概率 ≈2^-53）——但成文措辞须写明「f64 渲染分辨率下一致」，不得写「逐位一致」。

**验证证据**（2026-09-28，`docs/evidence/ts-14/u64-precision-probe/`）：`03-battle-raw.json`（curl 原文，rng_state=15755400384260043846）+ `04-node-precision-demo.txt`（同一文件 `JSON.parse` → 15755400384260045000，精度丢失=true）+ `04-node-precision-demo.txt`（独立复算 7+3·0x9E3779B97F4A7C15 mod 2^64 = 15755400384260043846，与原文逐位吻合——失败复现与修复口径双重验证，node BigInt）。游戏侧行为全程正确（bevy_remote/serde_json 输出精确），错题属工具链侧。

### PIT-M-010：进程内套件的默认值断言隐含进程史前提——调度驱动的表现层副作用只看「跨帧持久」的状态，驱动顺序改变即可假红

- 日期：2026-09-29
- 适用版本：方法论级（断言依赖「进程此前发生过什么」而非仅被测单元；任何带调度循环/帧边界的引擎与任何跨套件共享进程的回归设计同样成立）
- 分型：错题
- 通用性分级：methodology
- 标签：`测试套件` `进程史` `驱动顺序` `回归` `隐含前提`

**现象**：T040 驱动器在单进程内「先 `game.launch_level(1)` 做快照等价断言、后连跑 13 套件回归」，ts-08（相机轨道两连调状态机）相位 A 的「radius == 90.0（`CameraRig::default` 字面）」断言假红——实测 radius=14（棋盘视角）。同套件在「每任务独立进程/进程内无持久关卡」的 run-all 口径下从未红过（T020/T035 十余年任务回归全绿）。

**根因**：两层耦合。① ts-08 的默认值断言隐含「进程未加载过关卡」前提——M1 时代设计，T036 表现层落地后 `launch_level` 的 `LevelState` 跨帧持久会被 Update 调度里的相机收拢系统观察到（`game/src/present.rs`，radius→14）；② 表现层副作用由**调度系统在帧边界**施加，而套件在 BRP handler 独占执行内**原子完成**——套件内部即便 launch+还原（如 ts-17），中间态永不跨帧、表现层无从观察。故「套件自身无净副作用」与「套件前有人 launch 过」是两个命题：前者原子成立，后者才是假红源。

**修复**（已过验证）：

1. 回归驱动把套件执行排在任何**跨帧持久**的世界变更（launch/mutate_resources 等）之前，或每套件独立进程（run-all 口径）——默认值断言的进程史前提由驱动侧保证并成文注明；
2. 新写套件时把「默认值/初始值断言」的进程史前提写进套件文档（ts-08 文档现补注：相位 A 前提 = 进程未加载过关卡）；
3. 判定「假红 vs 真回归」先查进程史：失败断言的期望值是否是**默认字面值**，若是，先问「本进程此前是否发生过会改写它的持久变更」。

**验证证据**（2026-09-29，`docs/evidence/ts-17/`）：失败复现 = `run-b-probe-attempt1-pitm010.log`（launch→套件序：ts-08 failed=1，其余 12 套件全过——精确圈定进程史耦合而非代码回归）；修复复跑 = `run-b-probe.log`（套件先于 launch：13/13 套件全 PASS，ts-08 3/3）；原子性正证 = `atomicity-check.log`（REAL_EXIT=0：先跑 ts-17（内含 launch+还原）隔 1.5s 再跑 ts-08——相位 A 仍见 radius==90（|Δ|=0.000e0），表现层确实观察不到套件内中间态）。

---

### PIT-M-011：编译/长用例类门禁对机器负载敏感——并发重编译可致跨任务假红与挂死，门禁须空闲独占 + 限并行

- 日期：2026-09-30
- 适用版本：方法论级（任何含长运行用例的门禁套件与任何并发构建场景，与引擎无关）
- 分型：错题
- 通用性分级：methodology
- 标签：`门禁` `负载敏感` `并发编译` `假红` `挂死` `量测纪律`

**现象**：T041 冻结标注（零代码、零围栏变更，仅 Markdown 文本）的 doctest 门禁复跑连续两轮翻红：一轮 9 FAILED 但**无失败明细、无汇总行**（执行 worker 被会话中断连带杀除、进程树受损截断）；另一轮**失败集合完全漂移**（773/953/2584/2726 ≠ 前轮 1021–1453/2110）、多条「running for over 60 seconds」后整体挂死（日志停滞 8 分钟无新行）。同一代码在空闲独占 + `-j 3` 下 77 passed / 0 failed / 17 ignored 全绿（325.7s）。

**根因**：门禁中的长运行 doctest（bevy App 级用例）在默认并行下多个进程同时驻留，与另一仓库的 bevy 全量编译并发时 CPU/内存饱和——长用例超时判 FAILED（哪批超时逐轮随机，故失败集合漂移）、编译/测试子进程死锁挂死；被杀执行的半截运行再留下截断日志，放大「像真回归」的误判。**失败集合逐轮不同是环境性抖动的签名；真回归每轮失败同一批。**

**修复**（已过验证）：

1. 门禁/量测一律**空闲独占**执行，禁止与其他仓 cargo 编译并发（与帧率量测口径同纪律）；
2. cargo 并行限 3（编译 `-j 3` + doctest `--test-threads 3`；owner 2026-09-30 指令，兼顾内存上限）；
3. 判定「假红 vs 真回归」的顺序：先比对失败集合是否逐轮漂移 → 再查机器负载与并发进程 → 最后才怀疑代码；
4. 长门禁命令不交由可被会话中断连带杀除的后台执行者独跑（中断连锁曾二次致害：截断日志 + rustup 工具链安装被中断损坏，见 wanzhen 仓台账 T001）。

**验证证据**（2026-09-30，`docs/evidence/pre-window/`）：负载下红 = `gate-doc-test-t041-freeze.log`（9 FAILED 无汇总行：并发 bevy 编译 + worker 被杀截断）与 `gate-doc-test-t041-rerun.log`（失败集合漂移 + 挂死停滞：同因并发编译）；空闲独占绿 = `gate-doc-test-t041-rerun2.log`（`-j 3 -- --test-threads 3`：77 passed / 0 failed / 17 ignored，REAL_EXIT=0，finished in 325.74s）。三档日志与两批失败集合对照在案。

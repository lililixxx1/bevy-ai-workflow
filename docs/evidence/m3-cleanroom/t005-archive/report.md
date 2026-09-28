# 净室重跑 T005 报告：Rust Hotpatching（bevy 0.19 / subsecond）Windows 独立冒烟

- 执行者：净室独立执行者（无 prior 会话上下文，仅依据 req-t005.md + INJECTED-agents-discipline.md + 注入知识资产行事）
- 执行日期：2026-09-28（UTC）
- 工作区：`C:\Users\Administrator\Desktop\ccc\cleanroom\t005\`
- 环境：Windows 10 19044 / rustc 1.98.1（≥ bevy 0.19.1 MSRV 1.95.0）/ cargo 1.98.1 / dioxus-cli (dx) 0.7.10 / bevy 0.19.1 + subsecond 0.7.10（Cargo.lock 解析值）
- **总结论：冒烟通过。bevy 0.19 Hotpatching 在 Windows/本项目结构下可用——两次热补丁（字符串常量变更、控制流变更）均生效，帧计数全程连续单调、banner 仅出现一次（进程未重启），补丁生效时延 ~1.4s 量级，进程收尾零残留。首次 `cargo check` 通过 + 首次运行即达验收 → 一次通过，返工 0 次。**

## 一、逐轮时间线（UTC；退出码均为真实退出码，日志自含 REAL_EXIT 行或后台任务状态）

| # | 时间（约） | 内容 | 结果 | 退出码 | 证据 |
|---|---|---|---|---|---|
| r00 | 07:51:48 | 环境核查：rustc/cargo/dx 版本、t005 至盘根无父级 Cargo.toml（workspace 隔离扫描）、registry 内 bevy 0.19.1 全家 + subsecond 存在 | PASS | 0 | `logs/r00-environment.log` |
| r01 | 07:52:13 | 源码查证①：hotpatching feature 链（bevy→bevy_internal→bevy_app/bevy_ecs→subsecond）、DefaultPlugins 的 cfg 附带、`bevy_app/src/hotpatch.rs` 全文、官方示例 `examples/ecs/hotpatching_systems.rs` | PASS | 0 | `logs/r01-verify-sources.log` |
| r01b | 07:52:43 | 源码查证②：subsecond 版本要求 `"0.7.0-rc.0"`（registry 现存 0.7.10）、hotpatch 模块导出、bevy_log prelude 内容、MinimalPlugins/RunMode 定位 | PASS | 0 | `logs/r01b-verify-imports.log` |
| r01c | 07:52:57 | 源码查证③：MinimalPlugins 构成（default_plugins.rs:163-170） | PASS | 0 | `logs/r01c-verify-minimal.log` |
| r01d | 07:53:08 | 源码查证④：RunMode 枚举与 run_loop doc 示例 | PASS | 0 | `logs/r01d-verify-runner.log` |
| r01e | 07:53:43 | 源码查证⑤：bevy::prelude 组成、`Local` 在 bevy_ecs prelude、LogPlugin 不在 prelude | PASS | 0 | `logs/r01e-verify-prelude.log` |
| r01f | 07:54:00 | 源码查证⑥：bevy_log 在默认 feature 链 | PASS | 0 | `logs/r01f-verify-default-features.log` |
| r01g | 07:55:16 | 源码查证⑦：`run_loop(Duration)` 签名、ScheduleRunnerPlugin derive(Default) | PASS | 0 | `logs/r01g-verify-runner2.log` |
| r01h | 07:55:32 | 源码查证⑧：bevy_app prelude 明细（PluginGroup 在、ScheduleRunnerPlugin 不在） | PASS | 0 | `logs/r01h-verify-app-prelude.log` |
| 建档 | 07:56–07:57 | 写 `hotpatch-smoke/Cargo.toml` + `src/main.rs`（设计见文件头注释；无窗口 MinimalPlugins + 显式 HotPatchPlugin + LogPlugin + 8ms run_loop 心跳） | — | — | `hotpatch-smoke/Cargo.toml`、`hotpatch-smoke/src/main.rs` |
| **r02** | 07:57–07:59 | **首次 `cargo check`（一次通过判定点）** | **PASS：1m56s Finished，REAL_EXIT=0，0 编译警告**（grep 命中的 2 处 "warning" 实为依赖 crate 名 `warnings-macro`/`warnings`） | **0** | `logs/r02-cargo-check-1.log` |
| run | 07:59:01–08:08:32 | `dx serve --hot-patch` 后台托管运行：dx 全量构建 272.36s，app 于 08:03:35.370 启动（banner pid=10004），持续心跳至 08:08:32（frame=33770），全程 ~4 分 57 秒后被收尾终止 | PASS（按计划终止；后台任务状态 killed 为收尾动作，非失败） | n/a（常驻进程） | `logs/run/hotpatch-run.log` |
| 试验态 | 08:04:2x | 补丁 1 编辑脚本首试：assert 护栏发现模式 `label = "BEFORE"` 在文件中出现 2 次（代码 + 文档注释），**写盘前安全中止，文件未被修改**（dx 无感知，不构成一次补丁尝试） | 中止（未落盘） | 脚本异常退出 | `logs/run/patch1-edit-marker.log` |
| p1 | 08:04:32.174 | 热补丁 1 落盘：`label = "BEFORE"` → `"AFTER"`（唯一代码行模式，python 毫秒级时间戳同命令落盘） | 生效：08:04:33.572521 首行 AFTER（frame=6570） | 0 | `logs/run/patch1-edit-marker-v2.log`、`logs/run/hotpatch-run.log` |
| p2 | 08:07:29.670 | 热补丁 2 落盘：`if *frame % 30 == 0` → `% 10`（控制流变更） | 生效：08:07:31.029918 起 +10 帧间距（frame=26740） | 0 | `logs/run/patch2-edit-marker.log`、`logs/run/hotpatch-run.log` |
| parse | 08:07:5x | 全量解析运行日志：1422 行心跳、帧号严格单调、banner 计数、两次转轮边界提取 | PASS | 0 | `logs/run/parse-transition.log` |
| 收尾 | 08:08:50 | TaskStop 终止 dx 后台任务（连带进程树）；taskkill 双保险（RC=128 = 已无可杀）；tasklist 复查 dx/hotpatch/cargo/rustc/dioxus 全无 | PASS：零残留 | 0 | `logs/residue-check-final.log` |
| 终览 | 08:09:23 | 运行日志尾部快照 + app 级 WARN/ERROR/panic 扫描（0 条嫌疑；日志中每行前的 "ERROR" 为 dx 转发子进程输出的自有标签，app 实际级别为行内 INFO） | PASS | 0 | `logs/run/final-tail-and-scan.log` |

## 二、验收清单逐项判定（口径 = req-t005.md）

### a) 冒烟 crate 构建/运行成功 — **PASS**

- `cargo check` 首次即过：`Finished dev profile ... in 1m 56s` + `REAL_EXIT=0`（`logs/r02-cargo-check-1.log` 末行）。
- `dx serve --hot-patch` 构建并启动：`Build completed successfully in 272.36s, launching app!`（`logs/run/hotpatch-run.log`），app banner `hotpatch-smoke app started ... pid=10004` 于 08:03:35.370 出现，心跳持续至 08:08:32.787（frame=33770）。

### b) 两次热补丁各自生效证据（前后输出原文 + 帧连续性） — **PASS**

以下摘录自 `logs/run/hotpatch-run.log` 原始行（日志内保留 ANSI 转义码原文；此处为去码后的逐字内容，字段顺序未动）：

**热补丁 1（字符串常量 BEFORE → AFTER）**：

```text
332.37s ... 2026-09-28T08:04:33.304108Z  INFO hotpatch_smoke: hotpatch smoke heartbeat frame=6540 label="BEFORE"   ← 末行 BEFORE
332.64s ... 2026-09-28T08:04:33.572521Z  INFO hotpatch_smoke: hotpatch smoke heartbeat frame=6570 label="AFTER"    ← 首行 AFTER
332.90s ... 2026-09-28T08:04:33.835622Z  INFO hotpatch_smoke: hotpatch smoke heartbeat frame=6600 label="AFTER"
```

帧连续性：6540 → 6570（+30，正常帧间距，无归零、无跳变）。

**热补丁 2（控制流 `% 30` → `% 10`，打印节奏变更）**：

```text
2026-09-28T08:07:30.670049Z ... frame=26700 label="AFTER"   ← 间距 30 帧/行（~4 行/秒）
2026-09-28T08:07:30.940008Z ... frame=26730 label="AFTER"   ← 末行 30 帧间距
2026-09-28T08:07:31.029918Z ... frame=26740 label="AFTER"   ← 首行 10 帧间距（~12.5 行/秒）
2026-09-28T08:07:31.116902Z ... frame=26750 label="AFTER"
2026-09-28T08:07:31.205713Z ... frame=26760 label="AFTER"
```

帧连续性：26730 → 26740（无缝续接，分支条件变更即时生效于后续帧）。

**进程未重启的全局证据**（`logs/run/parse-transition.log`）：
- 全部 1422 行心跳帧号 **严格单调递增：30 → 33770 = True**（跨两次补丁无任何重置）；
- 启动 banner（`app started ... pid=10004`）全程**仅出现 1 次**（该行设计为进程重启即重现）；
- dx 侧两次均为 `Hot-patching: src\main.rs took ...`，无 rebuild/relaunch 全量重启消息。

### c) 补丁生效时延量级 — **PASS**

| 补丁 | 编辑落盘时刻（UTC，ms 级） | 行为变更首行时刻（tracing 时间戳） | 编辑→行为变更 | dx 自报 rebuild+patch |
|---|---|---|---|---|
| 1（字符串常量） | 08:04:32.174 | 08:04:33.572521 | **≈ 1.40 s** | 1191 ms |
| 2（控制流） | 08:07:29.670 | 08:07:31.029918 | **≈ 1.36 s** | 1186 ms |

（编辑时刻来自 python 写盘同命令时间戳 `logs/run/patch1-edit-marker-v2.log` / `patch2-edit-marker.log`；差值含 dx 文件监视器检测延迟，dx 自报值为其检测后的纯 rebuild+apply 耗时，两者口径不同、互为印证。量级结论：**秒级（~1.1–1.4s）**，与原任务回写结论同量级。）

### d) 子进程收尾杀净并核对 — **PASS**

TaskStop 终止 dx 后台任务（进程树连带终止）；pre-kill tasklist 已无 dx/hotpatch/cargo/rustc；`taskkill //F //IM dx.exe` 返回 RC=128（"没有找到进程" = 无可杀）；post-kill 复查 `dx.exe|hotpatch*|cargo|rustc|dioxus` 前缀全无 → **零残留**（`logs/residue-check-final.log`）。注：早前 `tasklist | grep dx` 命中的 `flpidx.exe` 为无关系统进程（子串误匹配），未触碰。

## 三、API 查证清单（全部按本地 registry 源码核实，`<reg>` = `~/.cargo/registry/src/rsproxy.cn-e3de039b2554c837`，核实日期 2026-09-28）

| # | 查证事实 | 出处（`<reg>` 相对路径:行号） | 用在哪 |
|---|---|---|---|
| 1 | `hotpatching` feature 存在且转接 bevy_internal | `bevy-0.19.1/Cargo.toml:2802` | `Cargo.toml` 的 `features = ["hotpatching"]` |
| 2 | feature 作用域 = bevy_app/hotpatching + bevy_ecs/hotpatching | `bevy_internal-0.19.1/Cargo.toml:282-285` | 报告（作用域结论） |
| 3 | bevy_ecs 侧经 subsecond 实现：`hotpatching = ["dep:subsecond"]`，版本要求 `"0.7.0-rc.0"` | `bevy_ecs-0.19.1/Cargo.toml:70, 211-213` | 报告（本次解析为 0.7.10，semver 兼容漂移，见 §四） |
| 4 | DefaultPlugins 在该 feature 下自动附带 HotPatchPlugin（故 MinimalPlugins 需显式添加） | `bevy_internal-0.19.1/src/default_plugins.rs:96-97` | main.rs 插件组装决策 + 注释 |
| 5 | HotPatchPlugin 定义与机制（connect_subsecond 连 dx CLI、HotPatched 消息、Last 调度回写） | `bevy_app-0.19.1/src/hotpatch.rs:16-17`（plugin）、build 布线 :20-46 | main.rs 显式 `.add_plugins(HotPatchPlugin)` |
| 6 | hotpatch 模块导出路径（cfg(feature="hotpatching") pub mod） | `bevy_app-0.19.1/src/lib.rs:40-41` | `use bevy::app::hotpatch::HotPatchPlugin` |
| 7 | 运行口径 `dx serve --hot-patch`（官方示例证实；"All systems are automatically hot patchable"） | `bevy-0.19.1/examples/ecs/hotpatching_systems.rs:4-6, 16` | 运行命令；补丁只改 system 函数体的设计依据 |
| 8 | LogPlugin 不在 prelude（bevy_log prelude 仅 tracing 宏与 *_once） | `bevy_log-0.19.1/src/lib.rs:35-40`（prelude）、`:218`（pub struct LogPlugin） | `use bevy::log::LogPlugin` |
| 9 | bevy::prelude 组成（app/ecs prelude + MinimalPlugins；log prelude 随 bevy_log feature） | `bevy_internal-0.19.1/src/prelude.rs:3-10` | `use bevy::prelude::*` 可用面判定 |
| 10 | `Local` 在 bevy_ecs prelude | `bevy_ecs-0.19.1/src/lib.rs:97` | heartbeat 系统参数 |
| 11 | MinimalPlugins 构成（TaskPool/FrameCount/Time/ScheduleRunner） | `bevy_internal-0.19.1/src/default_plugins.rs:163-170` | 无窗口方案选型 |
| 12 | `ScheduleRunnerPlugin` **不在** bevy_app prelude；`run_loop(Duration)` 签名；RunMode::Loop 语义 | `bevy_app-0.19.1/src/lib.rs:59-69`（prelude 清单无它）、`src/schedule_runner.rs:64`、`:20-29` | `use bevy::app::ScheduleRunnerPlugin` + `.set(run_loop(Duration::from_millis(8)))` |
| 13 | bevy_log 在默认 feature 链 | `bevy-0.19.1/Cargo.toml:2751` | 默认 feature 下 `bevy::log` 路径可达判定 |
| 14 | MSRV 1.95.0 | `bevy-0.19.1/Cargo.toml:14`（SKILL §6.6 速查引用复核） | 开场工具链核对（rustc 1.98.1 ≥） |

## 四、与注入知识资产的呼应/差异标注（如实）

**帮到的条目**：

- `bevy-dev/SKILL.md` §6.5（Hotpatching 速查）：feature 链、`dx serve --hot-patch`、无窗口 MinimalPlugins+显式 HotPatchPlugin+LogPlugin 方案、两条编译教训（LogPlugin 导入路径；inline format 对包装类型需解引用）——本次全部按本地源码**独立重新核实**（§三清单，行号基本重合）后才落码，直接促成首次 check 一次通过。
- `bevy-dev/SKILL.md` §0.5 / `assets-methodology/patterns.md` PAT-M-001（真实退出码判定 + 后台托管轮询）、PAT-M-005（常驻进程后台任务托管 + 强杀 + 残留核对）：本次 dx serve 后台托管与收尾全照此执行。
- `assets-methodology/pitfalls.md` PIT-M-005（Git Bash 下 taskkill 需 `//F //IM` 双斜杠）、PIT-M-001（管道吞退出码）：收尾命令与全部日志的 REAL_EXIT 形态直接受益。
- PIT-M-003（workspace glob 吸入非 crate 目录）：促成 r00 的 workspace 隔离扫描 + Cargo.toml 空 `[workspace]` 表防御。
- PAT-M-006（摘录逐字回对 raw）：§二摘录均自 `logs/run/hotpatch-run.log` 原文抽取（python 解析），仅去 ANSI 码、未改字段顺序。

**必须披露的独立性界限**：注入的 SKILL.md §6.5 与台账 T005 行**本身即原任务结论的回写**（SKILL v0.6 变更记录注明），等于预先提供了原任务的技术路线与两条编译教训——本次重跑的独立性受限于此，实际增量在于：全部 API 事实按本地源码带行号重新核实、独立复刻 crate 并重新产出全部运行证据、以及发现下述版本漂移。重跑结论与原结论一致，不能视为完全独立的无偏复证。

**需求书未覆盖、自行决定处**：

1. **运行节奏**：用 `ScheduleRunnerPlugin::run_loop(8ms)`（~125fps）固定心跳节奏（原任务为默认无界循环，帧号达百万级；本次 ~3.4 万）。帧率数字与原证据**不可直接对比**，不影响任何验收项。
2. **重启检测面**：增设 Startup banner 带 PID 的启动横幅（重现即重启），与帧连续性互证。
3. **编辑时间戳方法**：python 写盘同命令打毫秒级 UTC 时间戳（工具调用间隙会引入秒级误差，故不用 Edit 工具夹时间戳的方案）。
4. **subsecond 版本漂移**：bevy_ecs 要求 `"0.7.0-rc.0"`（caret），本地 registry 现解析为 **0.7.10**（原任务时点为 0.7.0-rc.0）。semver 兼容、冒烟行为无差异，但复现环境与原任务非逐位同版。

**其他如实记录**：本任务不涉及 BRP（冒烟本体无需远程操控通路），BRP 安全约束（仅 127.0.0.1）本任务 N/A。

## 五、自认定返工次数及口径

- **判定口径**（req-t005.md）：冒烟目标达成 + 进程收尾干净 = 一次通过；返工 = 首判未过后的重做次数；另按预注册口径，为写代码做的 API 探查尝试轮失败标试验态、不计返工但如实列出。
- **自认定：一次通过（是），返工 0 次。** 首次 `cargo check` REAL_EXIT=0 零警告；首次运行（dx serve --hot-patch）即完成两次热补丁并全达验收 a–d。
- **试验态尝试（未计入返工，如实列出 3 笔）**：
  1. 补丁 1 编辑脚本首试被 assert 护栏中止（模式在文档注释重复出现，计数 2≠1），**写盘前中止、文件零变更**（`logs/run/patch1-edit-marker.log`）——不构成对 crate 代码或判定轮的失败；
  2. 就绪轮询循环的 `grep -c ... || echo 0` 在无匹配时输出两行 "0" 致 shell 比较 warning（`integer expression expected`）——轮询本身正常命中（~270s 出心跳），纯脚本装饰性瑕疵；
  3. 补丁 1 转轮定位首用的 grep 模式 `label="BEFORE"` 被 ANSI 转义码干扰返回空——只读探查，随即改用 python 解析（`logs/run/parse-transition.log`）获得权威证据，判定未受影响。

## 六、证据文件清单（均在 `C:\Users\Administrator\Desktop\ccc\cleanroom\t005\`）

| 文件 | 内容 |
|---|---|
| `logs/r00-environment.log` | 环境核查（版本/隔离扫描/registry 清单） |
| `logs/r01-verify-sources.log` | feature 链 + HotPatchPlugin + 官方示例查证 |
| `logs/r01b-verify-imports.log` | subsecond 版本要求 / 导出路径 / bevy_log prelude 查证 |
| `logs/r01c-verify-minimal.log` | MinimalPlugins 构成查证 |
| `logs/r01d-verify-runner.log` | RunMode / run_loop 查证 |
| `logs/r01e-verify-prelude.log` | bevy::prelude 覆盖面查证 |
| `logs/r01f-verify-default-features.log` | 默认 feature 链含 bevy_log 查证 |
| `logs/r01g-verify-runner2.log` | run_loop 签名 + ScheduleRunnerPlugin Default 查证 |
| `logs/r01h-verify-app-prelude.log` | bevy_app prelude 明细查证 |
| `logs/r02-cargo-check-1.log` | 首次 cargo check（REAL_EXIT=0，1m56s） |
| `logs/run/hotpatch-run.log` | dx serve --hot-patch 全程运行日志（1608 行，288KB；含 dx 构建消息、banner、全部心跳、两次 Hot-patching 消息） |
| `logs/run/patch1-edit-marker.log` | 补丁 1 首试中止原文（试验态） |
| `logs/run/patch1-edit-marker-v2.log` | 补丁 1 落盘时间戳（08:04:32.174） |
| `logs/run/patch2-edit-marker.log` | 补丁 2 落盘时间戳（08:07:29.670） |
| `logs/run/parse-transition.log` | 全量解析：帧单调性 / banner 计数 / 两次转轮边界 |
| `logs/residue-check-final.log` | 收尾杀进程 + 零残留核对 |
| `logs/run/final-tail-and-scan.log` | 运行日志尾部快照 + 告警扫描（0 嫌疑行） |
| `hotpatch-smoke/Cargo.toml`、`hotpatch-smoke/src/main.rs` | 冒烟 crate 本体（含查证出处注释） |
| `hotpatch-smoke/Cargo.lock` | 版本锁定（bevy 0.19.1 / subsecond 0.7.10） |

# T045 · game/src 0.19→0.20 注锚清扫 · 证据档

- 任务：T045（`game/src` 0.19 注锚清扫收尾；随 M6 首波次并行卡）
- 本档生成单：**T045-W1R2**（第二次续做；补位与死亡留痕见 §4）
- 隔离树：基线 commit `d035230` 导出（树内无 `.git`，物理隔离；下文命令一律在树根执行，树根路径以 `<TREE>` 记）
- 档内文件：

| 文件 | 内容 |
| --- | --- |
| `check.log` | 门禁一 `cargo check --workspace -j 3`：命令全文 + 原始输出 + `REAL_EXIT` |
| `test.log` | 门禁二 `cargo test --workspace -j 3 -- --test-threads=3`：命令全文 + 原始输出 + `REAL_EXIT` |
| `README.md` | 本档：索引 + 判定行 + 清扫面统计 + 偏差登记 + 补位留痕 |

**判定行（结论）**

- 清扫面：`0.19.1` 注锚残留 **0 命中**（命令与原始输出见 §1.1，退出码 1 = 无匹配）
- 门禁一（check）：`REAL_EXIT=0`，0 错 0 警（变更 crate `game` 实际重检，`Finished` 3.08s，`warning` 行 = 0）
- 门禁二（test）：`REAL_EXIT=0`，113 passed / 0 failed / 17 ignored（10 个 `test result` 段机器求和）
- 偏差登记：**零登记**（§3）

---

## §1 清扫面统计（复验）

本机 `grep` 实现为 ugrep 7.8.4，`-rn` / `-rl` / `-ro` / `-rc` 语义同 GNU grep 同名选项；所有计数均为机器计数（禁手数），行数经两条独立路径互证（§1.2）。

### 1.1 零残留证明：`0.19.1` 命中 0

```text
$ grep -rn "0\.19\.1" game/src
（无输出）
REAL_EXIT=1

$ grep -rn "0\.19\.1" game/src | wc -l
0
$ grep -rl "0\.19\.1" game/src | wc -l
0
```

### 1.2 `0.20.0` 注锚现状（机器计数）

```text
$ grep -rn "0\.20\.0" game/src | wc -l
41
$ grep -rl "0\.20\.0" game/src | wc -l
17
$ grep -ro "0\.20\.0" game/src | wc -l
42
```

逐文件行数（`grep -rc "0\.20\.0" game/src --include=*.rs`，已滤 `:0` 项；逐文件求和 = `-rn | wc -l` = 41，双路径互证）：

```text
game/src\bench.rs:3
game/src\brp.rs:2
game/src\camera.rs:1
game/src\level.rs:1
game/src\main.rs:3
game/src\present.rs:1
game/src\rpc\launch_level.rs:1
game/src\rpc\mod.rs:1
game/src\rpc\screenshot.rs:3
game/src\rpc\suites\ts03.rs:3
game/src\rpc\suites\ts05.rs:1
game/src\rpc\suites\ts06.rs:1
game/src\rpc\suites\ts07.rs:1
game/src\rpc\suites\ts08.rs:1
game/src\rpc\suites\ts09.rs:2
game/src\rpc\suites\ts10.rs:1
game/src\sim.rs:15
```

口径注：派工单声明 T045 重写面为 **14 文件 31 处**（Lead 已核验），与上表 **17 文件 41 行 42 处**口径不同，不互推、不构成矛盾——前者 = 本次 0.19.1→0.20.0 重写动作计数；后者 = 清扫后现存全部 `0.20.0` 字样（含 T045 前既有注锚：`bench.rs` / `rpc/launch_level.rs` / `rpc/screenshot.rs` 三个文件不在派工单 14 文件清单内；其中 `bench.rs`、`rpc/screenshot.rs` 的注锚自带「0.20 复核 2026-10-29」既有标注）。

残留 `0.20.0` 字样全部位于注释文本内——唯一出现在代码行的 `bench.rs:68` 亦为行尾 `//` 注释内的注锚，非代码标识符/字面量：

```text
$ grep -rn "0\.20\.0" game/src | grep -vE ':[0-9]+:\s*(//|///|//!)'
game/src\bench.rs:68:    let dt = time.delta_secs_f64(); // 依据: bevy_time-0.20.0/src/time.rs:290（0.20 同行号复核 2026-10-29；初核 2026-09-26）
```

### 1.3 抽样 3 个文件目检 + 本地 0.20.0 源码锚实核

注锚形态 = 注释文本（`///` doc 注释或行尾 `//`），格式为「被引用项 +（`crate-0.20.0/文件:行号` + 语义摘要 + 核实日期）」，与既有 doc 注释风格一致。抽样 `sim.rs` / `rpc/suites/ts03.rs` / `brp.rs` 三个文件，下表每行一个锚点，逐点回本地 registry 0.20.0 源码核对（`<cargo-registry>` = 本机 cargo registry src 根，路径前缀按可公开态占位）：

| 抽样注锚 | 指向 | 实核（行号 + 原文） |
| --- | --- | --- |
| `game/src/sim.rs:145-147` | `bevy_remote-0.20.0/src/builtin_methods.rs:327-333`（`BrpTriggerEventParams`） | :327 `pub struct BrpTriggerEventParams {`；:331 `pub event: String,`；:333 `pub value: Option<Value>,` — 一致 |
| `game/src/rpc/suites/ts03.rs:12` | `bevy_ecs-0.20.0/src/reflect/mod.rs:36`（`AppTypeRegistry` 为 Resource） | :35 `#[derive(Resource, Clone, Default)]`；:36 `pub struct AppTypeRegistry(pub TypeRegistryArc);` — 一致 |
| `game/src/rpc/suites/ts03.rs:13` | `bevy_reflect-0.20.0/src/type_registry.rs:448`（`TypeRegistry::get`） | :448 `pub fn get(&self, type_id: TypeId) -> Option<&TypeRegistration> {` — 一致 |
| `game/src/brp.rs:5-6` | `bevy_remote-0.20.0/src/http.rs:167-170`（`with_address` 签名） | :167 `pub fn with_address(mut self, address: impl Into<IpAddr>) -> Self {` — 一致 |
| `game/src/brp.rs:6-7` | `bevy_remote-0.20.0/src/http.rs:52-60`（`DEFAULT_ADDR` / `DEFAULT_PORT`） | :52 `pub const DEFAULT_PORT: u16 = 15702;`；:60 `pub const DEFAULT_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));` — 一致 |

（实核命令形如 `grep -n "pub struct BrpTriggerEventParams" <cargo-registry>/bevy_remote-0.20.0/src/builtin_methods.rs`；上表各锚点命中行号与注锚所记行号一一对应。）

---

## §2 门禁判定

| 门禁 | 命令（前置 `CARGO_TARGET_DIR=<shared-target>`） | 判定 | 关键计数 |
| --- | --- | --- | --- |
| 一：构建 + 静态检查 | `cargo check --workspace -j 3` | `REAL_EXIT=0`；**0 错 0 警** | 变更 crate 实际重检（`Checking game v0.1.0`），`Finished` 3.08s；`warning` 行 = 0 |
| 二：单测 + doctest | `cargo test --workspace -j 3 -- --test-threads=3` | `REAL_EXIT=0`；**0 failed** | `game` 20/20；`mcp_bridge`（lib）15/15；`Doc-tests docs` 78 passed / 17 ignored；合计 113 passed / 0 failed / 17 ignored（10 段机器求和）；`error` 行 = 0、`warning` 行 = 0 |

逐 crate 明细（自 `test.log`）：

- 单测段：`claim-lint` 0 测试；`docs`（lib）0 测试；`game`（bin）**20 passed**；`game_td`（bin）0 测试；`hotpatch_smoke`（bin）0 测试；`mcp_bridge`（lib）**15 passed**、`mcp_bridge`（bin）0 测试；`task_runner`（bin）0 测试。
- doctest 段：`Doc-tests docs` **78 passed / 0 failed / 17 ignored**（255.70s）；`Doc-tests mcp_bridge` 0 测试。
- `claim-lint` / `task-runner` 源码无 `#[test]` 项（两 crate 的门禁形态是各自 CLI 运行，不在本卡门禁范围）。

取证注记（口径披露，已录数据零改动）：

1. 门禁一首跑（18:17）全量缓存命中——输出仅 `Finished ... in 1.29s`，无 `Checking` 行；缓存命中态下 cargo 不重发告警，无法据以观察「0 警」。
2. 故先执行 `cargo clean -p game`（`Removed 751 files, 5.8GiB total`；**仅 game 包产物**，实测共享 target 内 bevy rmeta 315 / rlib 162 留存，bevy 依赖未失效），再重跑门禁一 → `Checking game v0.1.0` + `Finished` 3.08s + 0 warning 行，即 `check.log` 定稿档。
3. 门禁二（`test.log`）运行于上述 clean 之前；本卡未改动 `game/src` 任何源码，源码面与最终态同一。复跑者需先重建 `game` 包单测产物（其依赖 rmeta/rlib 均缓存留存），再执行同一命令即可复现。
4. 门禁二观测时长：doctest 段 255.70s（`--test-threads=3` 钉死，未触发 10 分钟硬超时，无中断续跑）。

---

## §3 SOURCE-DEVIATION-NOTE

**零登记**（本卡偏差条目数 = 0，附录 C 上限 3 条未触及）：

- 清扫动作限于注释 / doc 注锚文本层面，无接口、行为、依赖、版本锁改动；本卡写范围内未补改任何 `game/src` 源文件。
- 本卡无源镜像移植面（附录 C 降格适用），无上游缺陷登记面。
- 登记格式（本项目沿用，附录 C）：`SOURCE-DEVIATION-NOTE <编号>: <文件:行> 源=<行为> 本侧=<行为> 理由=<...> 裁决=<编号>` —— 本次无条目。

---

## §4 补位事实留痕

T045 执行经两段死亡；本单 T045-W1R2 为**第二次续做**（续做，非重做；树内进度全程保留）：

| # | 时间（2026-10-30） | 事实 | 处置 |
| --- | --- | --- | --- |
| 1 | 10:45 | 宿主机整机冻结硬重启（commit 顶满致系统级冻结，机制见 PROJECT-APPENDIX §A「cargo 段全局互斥」条：doctest 多 ×rustc + 页文件颠簸，事件日志 6008 意外关机 10:45:35） | `test.log` 首跑在 doctest 段被截断——末段为 `... has been running for over 60 seconds` 告警行，无 `test result` 收尾行、无 `REAL_EXIT` → **作废** |
| 2 | 11:09 | 用户「晚上再开工」停工指令，续做实例被停止 | 留下 `.check.raw.log` / `.test.raw.log` 两份截断临时档（后者止于 doctest `... ok` 行，无收尾行）→ **作废**；本单开工时（18:16）删除。截断档不作证据，死亡事实以本段留痕 |
| 3 | 18:16 起 | 本单 T045-W1R2 续做 | 复用树内既有进度：`game/src` 31 处注锚重写（Lead 已核验）+ 10:42 绿档 check；重跑两门禁、补齐本档后收工 |

说明：

- 时间点 10:45 / 11:09 取自档内文件 mtime（原 `test.log` mtime 10:45、`.test.raw.log` mtime 11:09）与派工单记录。
- 两段死亡期间树内源码未被二次改动：本单复验 `0.19.1` 仍 0 命中、两门禁全绿，按收割判定「进树续做至绿」处置，未丢弃重来。

---

## §5 档内保真与可公开态

- `check.log` / `test.log` 仅做**路径前缀替换**（附录 G「确需清理机器路径前缀时只改前缀，命中行号与内容原样留痕」）：共享 target 绝对路径前缀 → `<shared-target>`；树路径前缀 → `<TREE>`。替换前含绝对路径行数：check 1 行、test 8 行（本档写作时逐行扫描实测）；替换后复查 = 0 行。行号、构件 hash（如 `claim_lint-8910f178cf797c61.exe`）、`test result` 行与计时原样未动。
- 本档正文零机器绝对路径：registry 路径以 `<cargo-registry>` 占位，树根以 `<TREE>` 占位；命令中 `CARGO_TARGET_DIR` 值以 `<shared-target>` 占位书写（先例 T041 证据档）。
- 自含性：两门禁首行即完整命令（含 `CARGO_TARGET_DIR` 占位与全部限流参数 `-j 3` / `--test-threads=3`），只看本档即可独立复跑。

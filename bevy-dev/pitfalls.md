# Bevy 特定层错题本

- 收录：通用性分级为 `bevy-specific`（换引擎后不再成立）的错题条目——API/版本/引擎行为相关的坑。
- 入库门禁与条目字段定义：见 [assets-methodology/pitfalls-schema.md](../assets-methodology/pitfalls-schema.md)（失败须在标注版本复现 + 修复过编译与运行验证；**未过验证禁止入库**）。
- 追加方式：按下方模板在文件末尾追加，`PIT-B-XXX` 三位自增（从 001 起），并在本行更新条数。
- 反例代码统一用 `rust,compile_fail` 围栏标记（doctest 断言其编译失败）；语义不符时用 `rust,ignore` 并附理由（先例见 assets-methodology/pitfalls.md PIT-M-001 的 shell 命令处理）。
- **doctest 门禁（M3 起，2026-09-27）**：本文件经 `docs/src/lib.rs` include_str! 纳入 `cargo test --doc -p docs`。围栏约定（对其后全部条目生效）：`rust,compile_fail`=反例（机器断言编译必败）；`rust,ignore`=非编译载体反例（附理由）或非 self-contained 修复片段（附理由 + 完整代码出处）；`rust`/`rust,no_run`=self-contained 修复正例（长运行标 no_run 附理由）。升级窗口换版本后反例如能编译，doctest 立即红——条目自动过期检测。
- **非代码载体反例（2026-09-28 增补）**：wire 协议请求 / 运行时行为观察类坑无编译面可断言，以 `text` 围栏收录原始请求与报错原文并锚定证据文件（先例 PIT-B-051；口径出处 = 意向文档 §5.2「分型口径细化注记」2026-09-28）。
- 批次探查条目（M3 §3.1）的证据面：探针 crate 在仓库外不入 workspace，其「修复过编译」以探针 check 日志（归档 `docs/evidence/m3-assets/batch-*/`）为证据面，不要求 `cargo check --workspace`。
- 当前：51 条（2026-09-26 起；2026-09-27 M3 批次一 005–013 证据 batch-c/；批次二 014–020 证据 batch-d/；批次三 021–025 证据 batch-e/；批次四 026–030 证据 batch-f/；2026-09-28 批次五 031–050 证据 docs/evidence/m3-assets/batch-g/；同日 Block H 净室重跑净新发现 051 证据 docs/evidence/m3-cleanroom/）。

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

### PIT-B-001：`Commands::spawn_batch` 要求迭代器 `'static`——闭包借用系统参数必然编译失败

- 日期：2026-09-26
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Bevy Commands 队列的生命周期约束与 ECS 系统参数借用语义的组合）
- 标签：`spawn_batch` `Commands` `生命周期` `E0373` `E0521`

**现象**：在 Startup system 里用 `(0..count).map(|i| { ...引用 rng/mesh/handle/config... }).collect()` 形态的迭代器调 `commands.spawn_batch(iter)`，报 E0373 ×N + E0521（2026-09-26 实测原文摘录）：

```text
error[E0373]: closure may outlive the current function, but it borrows `rng`, which is owned by the current function
   --> game\src\sim.rs:136:45
    |
136 |     let iter = (0..config.entity_count).map(|index| {
    |                                             ^^^^^^^ may outlive borrowed value `rng`
error[E0521]: borrowed data escapes outside of function
    |
110 |     config: Res<SimConfig>,
    |     ------
    |     has type `bevy::bevy_ecs::change_detection::Res<'1, SimConfig>`
155 |     commands.spawn_batch(iter);
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ argument requires that `'1` must outlive `'static`
```

**最小复现**：

```rust,ignore
// 语义性反例（E0373 的核心是借用语义而非纯语法，无法用 compile_fail 精确断言场景，故标 ignore 附理由）：
fn spawn_swarm(mut commands: Commands, config: Res<SimConfig>) {
    let mut rng = MyRng::new(1);
    let iter = (0..config.entity_count).map(|i| MyBundle::new(&mut rng, i));
    commands.spawn_batch(iter); // ← 迭代器借用了函数局部 `rng`，而 spawn_batch 要求 'static
}
```

**根因**：`Commands::spawn_batch` 的签名约束 `I: IntoIterator + Send + Sync + 'static`（`bevy_ecs-0.19.1/src/system/commands/mod.rs:587-593`）——命令队列延迟到帧末 ApplyDeferred 阶段执行，迭代器里的闭包必须自带全部数据；而系统参数（`Res`/`ResMut`）与函数局部变量的引用只在系统函数体内有效，`Res<'1>` 非 `'static`。

**修复**（已过 cargo check + 运行验证）：

1. 闭包改 `move`，把 mesh/material 句柄、PRNG、配置字段**拷出为 owned 值**后 move 进闭包（handle 是 `Arc` 内部指针，`clone()` 代价可忽略）；
2. `Res` 不能 move——只拷贝需要的标量字段（`let seed = config.seed;`）。

```rust,ignore
// 片段（形态示意，不独立编译；完整已验证实现见 game/src/sim.rs 首役代码，T003）：
let seed = config.seed;
let mut rng = SplitMix64::new(seed);
let iter = (0..config.entity_count).map(move |index| {
    // 全部使用 move 进来的 owned 数据
});
commands.spawn_batch(iter);
```

**验证证据**：
- 复现：初版 `cargo check --workspace --all-targets` → 8 errors（E0277/E0373×4/E0433×2/E0521），REAL_EXIT=101（2026-09-26，`/tmp/check1.log`）；
- 修复后：`cargo check --workspace --all-targets` REAL_EXIT=0、0 warnings；
- 运行验证：`--count 1000` 启动后 BRP `world.query` Wanderer 计数 == 1000（`docs/evidence/brp-assert-result.txt` #3），`--count 50000` 基线采集 spawn 成功——批量生成真实生效。

### PIT-B-002：`PresentMode`/`WindowMode` 不在 prelude——须从 `bevy::window::` 显式导入

- 日期：2026-09-26
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Bevy prelude 的导出面）
- 标签：`prelude` `Window` `PresentMode` `导入路径` `E0433`

**现象**：`use bevy::prelude::*;` 后直接写 `mode: WindowMode::Windowed` 与 `present_mode: PresentMode::AutoVsync`，报 E0433 ×2（2026-09-26 实测）：

```text
error[E0433]: cannot find type `WindowMode` in this scope
  --> game\src\main.rs:38:23
   |
38 |                 mode: WindowMode::Windowed,
   |                       ^^^^^^^^^^ use of undeclared type `WindowMode`
error[E0433]: cannot find type `PresentMode` in this scope
  --> game\src\main.rs:39:31
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn f(window: Window) -> PresentMode {
    window.present_mode // E0433: PresentMode 不在 prelude
}
```

**根因**：0.19 prelude 含 `Window`/`WindowPlugin` 但**不含** `PresentMode`/`WindowMode`（后者定义于 `bevy_window-0.19.1/src/window.rs:1219/1338`，经 `pub use window::*`（lib.rs:33）从 `bevy::window` 可达）。官方窗口示例自己也是显式导入：`bevy-0.19.1/examples/window/window_settings.rs:8`。

**修复**（已过 cargo check + 运行验证）：`use bevy::window::{PresentMode, WindowMode};`

**验证证据**：
- 复现：E0433 原文见上（初版 check，2026-09-26）；
- 修复后 `cargo check --workspace --all-targets` REAL_EXIT=0；运行验证见基线采集（两种 present mode 均真实生效：vsync 组锁 60fps、no-vsync 组 113.8~534.7fps）。

### PIT-B-003：BRP 引用内置类型凭旧版本印象写全路径——`world.query` 静默返回空，不报错

- 日期：2026-09-26
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（类型路径 + BRP 非严格模式行为）
- 标签：`BRP` `world.query` `类型路径` `bevy_camera` `静默失败`

**现象**：BRP 断言相机时按 0.16 时代印象写 `data.components=["bevy_render::camera::Camera", ...]`——请求**成功返回**但结果数组为空（0 行），下游取 `qcam[0]` 得 undefined、`hypot(undefined,...)` 得 NaN。无任何错误提示（`strict` 默认 false 时未知组件被跳过而非报错；0.19 中 Camera 实际在 `bevy_camera::camera::Camera`——bevy_camera crate）。

**最小复现**（运行时行为，非编译失败；语义不符 compile_fail，标 ignore 附理由：本坑是 HTTP 请求返回空结果，无编译期载体）：

```rust,ignore
// 游戏运行中（BRP 127.0.0.1:15702）：
// POST world.query {"data":{"components":["bevy_render::camera::Camera"]}}
// → {"result":[]}   ← 0 行，不报错（真实路径是 bevy_camera::camera::Camera）
```

实测原文（2026-09-26，node 断言脚本）：`qcam` 为空数组 → `TypeError: Cannot read properties of undefined` → 修正路径后 `PASS | camera orbit radius~=90 | len=1 dist=90.00`。

**根因**：①类型随版本跨 crate 搬迁（0.19 相机体系移入 bevy_camera），训练语料滞后；②BRP `world.query` 默认非严格模式对未注册/未匹配组件静默跳过（`bevy_remote-0.19.1/src/builtin_methods.rs`，`strict` 字段默认 false）——错误路径不产生 error 响应，只有空结果。

**修复**（已过运行验证）：引用任何 bevy 内置类型前，先 `world.list_components` 实测全路径（本机实测 Camera = `bevy_camera::camera::Camera`）；对「必须存在」的组件断言加 `strict: true`，让未知类型显式报错而非静默空结果。

**验证证据**：
- 复现：错误路径 query 返回空 + 下游 NaN/undefined（2026-09-26 断言脚本首跑）；
- 修复：`world.list_components` 实测路径 + strict 后 `PASS | camera orbit radius~=90`（`docs/evidence/brp-assert-result.txt` #9）。

---

### PIT-B-004：BRP 0.19.1 的 params「存在性语义」——传 `{}` 与完全省略 params 行为不同；`world.get_resources` 字段是单数

- 日期：2026-09-26
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（BRP 方法签名随版本收紧）
- 标签：`BRP` `params` `world.list_components` `world.get_resources` `schema收紧`

**现象**：两个实测失败原文（2026-09-26，curl 对 127.0.0.1:15702）：
1. `world.list_components` 传 `"params":{}` → `{"code":-32602,"message":"missing field \`entity\`"}`；而完全省略 params 才走「全量注册组件」分支（实测返回 310 个）。
2. `world.get_resources` 传 `"params":{"resources":["..."]}`（复数、按 0.16 时代直觉）→ `{"code":-32602,"message":"missing field \`resource\`"}`；0.19.1 字段为单数 `resource: String`。

**最小复现**（运行时行为，非编译失败；语义不符 compile_fail，标 ignore 附理由：本坑是 HTTP JSON-RPC 参数反序列化失败，无编译期载体）：

```rust,ignore
// 游戏运行中（BRP 127.0.0.1:15702）：
// POST {"method":"world.list_components","params":{}}      → -32602 missing field `entity`
// POST {"method":"world.list_components"}                  → 200，310 个组件（正确全量用法）
// POST {"method":"world.get_resources","params":{"resources":["game::sim::SimStats"]}} → -32602 missing field `resource`
// POST {"method":"world.get_resources","params":{"resource":"game::sim::SimStats"}}    → 200 成功
```

**根因**：0.19.1 各方法 handler 形如 `In<Option<Value>>` + `params.map(parse).transpose()?`（`bevy_remote-0.19.1/src/builtin_methods.rs:1378-1385`，核实 2026-09-26）：params 为 `Some` 时必须完整满足 `BrpXxxParams` 反序列化（`{}` 缺必填字段即 -32602）；`None`（整个 params 字段省略或 null）才有独立的全量分支。`get_resources` 的参数结构在 0.19.1 是单数 `resource: String`（builtin_methods.rs:140-145），与旧资料复数数组形态不同。

**修复**（已过运行验证）：写 BRP 请求前逐方法核对本地源码的 `BrpXxxParams` 结构；「全量清单」类调用完全省略 params；资源读取用单数 `resource` 字段。bevy_brp_mcp 0.22.7 已正确适配这些语义（可作为参数形态的活参考）。

**验证证据**：
- 复现：`tooling/brp-logs/07-world-list-components.md`（-32602 原文 + 省略 params 后 310 个）与 `08-world-get-resources.md`（-32602 原文 + 单数修正后成功）；
- 运行验证：同两文件的修正重测段（2026-09-26，curl 原文存证）。

---

<!-- ==================== M3 批次一（ECS 查询与调度域，2026-09-27） ====================
探查方法：仓库外探针 crate（Desktop/ccc/bevy-probe-c，源码快照 docs/evidence/m3-assets/
batch-c/probe-src-final/）「凭记忆首试 → check 失败原文归档 → 查 0.19.1 源码修正 →
双重验证」（m3-plan §3.1 时序，失败原文见 probe-r1-check.log REAL_EXIT=101）。
编译级 5 条（005-009）+ 运行时 4 条（010-013，断言失败=假设证伪）。 -->

### PIT-B-005：`Query` 单实体访问族——`single()`/`single_mut()` 返回 `Result`，`get_single` 不存在

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Query API 面随版本翻转）
- 标签：`Query` `single` `get_single` `Result` `QuerySingleError` `E0609` `E0599`

**现象**：按旧语料直觉「`single()` = 不满足即 panic 的直接值、`get_single()` = Result 变体」书写，两个实测失败（2026-09-27，probe-r1-check.log）：

```text
error[E0609]: no field `0` on type `std::result::Result<&Pos, QuerySingleError>`
  --> src\lib.rs:27:7
   |
27 |     p.0
   |       ^ unknown field
   |
help: one of the expressions' fields has a field of the same name
   |
27 |     p.unwrap().0
   |       +++++++++
...
error[E0599]: no method named `get_single` found for struct `bevy::bevy_ecs::system::Query<'world, 'state, D, F>` in the current scope
  --> src\lib.rs:92:7
   |
92 |     q.get_single().unwrap().0
   |       ^^^^^^^^^^
   |
help: there is a method `single` with a similar name
   |
92 -     q.get_single().unwrap().0
92 +     q.single().unwrap().0
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

#[derive(Component)]
struct Health(f32);

fn f(q: Query<&Health>) -> f32 {
    let h = q.single(); // 0.19.1 返回 Result<&Health, QuerySingleError>，非直接值
    h.0 // E0609：Result 无字段 0
}
```

**根因**：0.19.1 中 `Query::single(&self) -> Result<ROQueryItem<'_, 's, D>, QuerySingleError>`（`bevy_ecs-0.19.1/src/system/query.rs:2097`）与 `single_mut -> Result`（`:2126`）；`get_single` 在 query.rs 全文 **0 处**（grep 核实 2026-09-27）——旧语料的「panic 变体 / Result 变体」分工已不存在，panic 语义由调用方 `.expect()` 表达（官方示例 `query.rs:2078-2089` 用 match 处理 `QuerySingleError`）。

**修复**（已过 cargo check + 运行验证）：`q.single().map(|p| p.0).unwrap_or(0.0)` 或 `.expect()` / match。

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-c/probe-r1-check.log`（E0609 + E0599 原文，REAL_EXIT=101，2026-09-27）；
- 修复：`probe-r2-check.log` REAL_EXIT=0（正解形态 p1c）。

### PIT-B-006：`QueryParIter` 不实现任何迭代器 trait——`par_iter()` 无 `.count()/.map()`，并行面是固有方法 `for_each`

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（QueryParIter 接口面）
- 标签：`par_iter` `QueryParIter` `ParallelIterator` `multi_threaded` `E0599` `feature`

**现象**：`q.par_iter().count()` 报 E0599（2026-09-27，probe-r1-check.log）：

```text
error[E0599]: `QueryParIter<'_, '_, &Pos, ()>` is not an iterator
  --> src\lib.rs:37:18
   |
37 |     q.par_iter().count()
   |                  ^^^^^ `QueryParIter<'_, '_, &Pos, ()>` is not an iterator
   |
  ::: ...\bevy_ecs-0.19.1\src\query\par_iter.rs:16:1
   |
16 | pub struct QueryParIter<'w, 's, D: IterQueryData, F: QueryFilter> {
   | ----------------------------------------------------------------- doesn't satisfy `QueryParIter<'_, '_, &Pos, ()>: Iterator`
   |
   = note: the following trait bounds were not satisfied:
           `QueryParIter<'_, '_, &Pos, ()>: Iterator`
           which is required by `&mut QueryParIter<'_, '_, &Pos, ()>: Iterator`
```

**关键证伪**：启用 `multi_threaded` feature 后**同样报错**（probe-r2-check-attempt1.log，2026-09-27）——trait 缺失与 feature 无关，初步归因「缺 feature」不成立。（补注：本仓默认链本已启用该 feature，见根因勘误；r1/r2 均处启用态，「trait 缺失」由源码无 impl 独立支持。）

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

#[derive(Component)]
struct Pos(f32);

fn f(q: Query<&Pos>) -> usize {
    q.par_iter().count() // QueryParIter 无 Iterator/ParallelIterator impl，任何 feature 下都编译失败
}
```

**根因**：`QueryParIter` 的并行接口是**固有方法** `for_each` / `for_each_init`（`par_iter.rs:42/:77`），内部按 `multi_threaded` feature 与 `ComputeTaskPool` 线程数自动选并行/串行实现（`:86-:120` cfg 分支）——不实现 `Iterator`，也没有 rayon `ParallelIterator` impl（grep 无）。feature 面（2026-09-27 勘误，独立审核纠正初判）：门面 `default = ["2d","3d","ui","audio"]`（`bevy-0.19.1/Cargo.toml:2742-2747`）本身不含 `multi_threaded`，但 `2d`（`:2586-2592`）/`3d`（`:2605-2611`）/`ui`（`:2884-2890`）均含 `default_platform`（`:2762-2774`），后者含 `multi_threaded`（`:2768`）——**默认链实际传递启用**（`cargo tree -e features` 实证）；「feature 关闭时 `for_each` 退化为串行 fold」仅在显式 `default-features = false` 自组 feature 时才会发生。

**修复**（已过 cargo check）：用固有 `for_each`（真并行需显式加 feature）：

```rust,ignore
// 片段（形态示意；完整已验证形态见 docs/evidence/m3-assets/batch-c/probe-src-final/src/lib.rs p3c）：
use std::sync::atomic::{AtomicUsize, Ordering};
let n = AtomicUsize::new(0);
q.par_iter().for_each(|_| {
    n.fetch_add(1, Ordering::Relaxed);
});
n.load(Ordering::Relaxed)
```

**验证证据**：
- 复现：`probe-r1-check.log` + `probe-r2-check-attempt1.log`（带 feature 二次证实，均 REAL_EXIT=101）；
- 修复：`probe-r2-check.log` REAL_EXIT=0（p3c，for_each + AtomicUsize 形态）。

### PIT-B-007：命令入队是 `Commands::queue`——`add` 不存在

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Commands API 改名史）
- 标签：`Commands` `queue` `add` `闭包命令` `E0599`

**现象**：按 0.11 时代印象写 `commands.add(closure)` 报 E0599（2026-09-27，probe-r1-check.log）：

```text
error[E0599]: no method named `add` found for struct `bevy::bevy_ecs::system::Commands<'w, 's>` in the current scope
  --> src\lib.rs:42:14
   |
42 |     commands.add(|world: &mut World| {
   |     ---------^^^ method not found in `bevy::bevy_ecs::system::Commands<'_, '_>`
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn f(mut commands: Commands) {
    commands.add(|world: &mut World| { // E0599：Commands 无 add 方法（0.19.1 是 queue）
        let _ = world.entities().len();
    });
}
```

**根因**：0.19.1 的公开入队方法是 `Commands::queue(&mut self, command: impl Command)`（`bevy_ecs-0.19.1/src/system/commands/mod.rs:641`；另有 `queue_handled`/`queue_silenced` `:694/:703`，EntityCommands 侧 `queue` `:1956`）；`add` 在 0.13 前后已改名，`EntityCommands::add_observer`（`:1200`）等新方法不是它的后继。

**修复**（已过 cargo check）：`commands.queue(|world: &mut World| { ... })`——闭包 `FnOnce(&mut World)` 实现 `Command` trait，形态不变只改名。

**验证证据**：
- 复现：`probe-r1-check.log`（E0599 原文，REAL_EXIT=101）；
- 修复：`probe-r2-check.log` REAL_EXIT=0（p4c，queue + 闭包形态编译通过）。

### PIT-B-008：可变组合迭代不吃 for——`iter_combinations_mut` 须用 `fetch_next()`

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（QueryCombinationIter 的 trait 实现面）
- 标签：`iter_combinations_mut` `QueryCombinationIter` `fetch_next` `ReadOnlyQueryData` `E0277`

**现象**：`for pair in q.iter_combinations_mut::<2>()` 报 E0277（2026-09-27，probe-r1-check.log）：

```text
error[E0277]: the trait bound `&mut Vel: ReadOnlyQueryData` is not satisfied
  --> src\lib.rs:50:17
   |
50 |     for pair in q.iter_combinations_mut::<2>() {
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `ReadOnlyQueryData` is not implemented for `&mut Vel`
   |
   = help: the following other types implement trait `ReadOnlyQueryData`:
             &Archetype
             &T
             ...
   = note: `ReadOnlyQueryData` is implemented for `&Vel`, but not for `&mut Vel`
   = note: required for `QueryCombinationIter<'_, '_, &mut Vel, (), 2>` to implement `Iterator`
   = note: required for `QueryCombinationIter<'_, '_, &mut Vel, (), 2>` to implement `IntoIterator`
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

#[derive(Component)]
struct Vel(f32);

fn f(mut q: Query<&mut Vel>) -> usize {
    let mut n = 0;
    for _pair in q.iter_combinations_mut::<2>() { // E0277：可变数据的组合迭代器不实现 IntoIterator
        n += 1;
    }
    n
}
```

**根因**：`Query::iter_combinations_mut` 存在（`bevy_ecs-0.19.1/src/system/query.rs:813`，bound 仅 `D: IterQueryData`），但返回的 `QueryCombinationIter` 的 `Iterator/IntoIterator` impl **仅对 `ReadOnlyQueryData` 成立**（错误注记明示）；可变数据的官方形态是 `while let Some([mut a1, mut a2]) = combinations.fetch_next()`（官方示例 `query.rs:796-806`，`fetch_next` 定义 `:802`，StreamingIterator 风格）。

**修复**（已过 cargo check + 运行验证）：

```rust,ignore
// 片段（完整已验证形态见 probe-src-final/src/lib.rs p5c；mut 绑定是必填语法非可选）：
let mut it = q.iter_combinations_mut::<2>();
while let Some([mut a, b]) = it.fetch_next() {
    a.0 += b.0;
}
```

**验证证据**：
- 复现：`probe-r1-check.log`（E0277 原文）；
- 修复：`probe-r2-check.log` REAL_EXIT=0（p5c；首版漏 `mut` 绑定的 E0596 修法痕迹在 `probe-r2-check-attempt1.log`）。

### PIT-B-009：`add_systems` 首参只收 `ScheduleLabel`——系统入集用 `.in_set()`

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（App 装配 API 签名）
- 标签：`add_systems` `SystemSet` `ScheduleLabel` `in_set` `configure_sets` `E0277`

**现象**：`app.add_systems(MySets::A, system)` 报 E0277（2026-09-27，probe-r1-check.log）：

```text
error[E0277]: the trait bound `MySets: ScheduleLabel` is not satisfied
   --> src\lib.rs:87:21
    |
 87 |     app.add_systems(MySets::A, p6_two_mut_queries);
    |         ----------- ^^^^^^^^^ unsatisfied trait bound
    |         |
    |         required by a bound introduced by this call
    |
help: the trait `ScheduleLabel` is not implemented for `MySets`
   ...
    = note: consider annotating `MySets` with `#[derive(ScheduleLabel)]`
    ...
note: required by a bound in `bevy::bevy_app::App::add_systems`
   --> ...evy_app-0.19.1\srcpp.rs:323:24
    |
321 |     pub fn add_systems<M>(
    |            ----------- required by a bound in this associated function
322 |         &mut self,
323 |         schedule: impl ScheduleLabel,
    |                        ^^^^^^^^^^^^^ required by this bound in `App::add_systems`
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum MySets {
    A,
}

fn work() {}

fn f(app: &mut App) {
    app.add_systems(MySets::A, work); // E0277：首参须 ScheduleLabel（Update/FixedUpdate/自定义 label）
}
```

**根因**：0.19.1 `App::add_systems<M>(&mut self, schedule: impl ScheduleLabel, ...)`（`bevy_app-0.19.1/src/app.rs:321-323`）——首参只能是调度标签；`SystemSet` 的进图通道是 `configure_sets`（可加 `.chain()`，r1 探针实证编译通过）+ 系统侧 `.in_set(MySets::A)`（`bevy_ecs-0.19.1/src/schedule/config.rs:322`（ScheduleConfigs）/`:493`（IntoScheduleConfigs））。

**修复**（已过 cargo check）：`app.add_systems(Update, work.in_set(MySets::A));`（configure_sets 形态不变）。

**验证证据**：
- 复现：`probe-r1-check.log`（E0277 原文）；
- 修复：`probe-r2-check.log` REAL_EXIT=0（p9c）。

### PIT-B-010：query 迭代序不保证——组件迁移触发表内 swap_remove 行重排（实测 [3,2,1]）

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（archetype/table 存储语义）
- 标签：`迭代序` `archetype` `swap_remove` `确定性` `运行时`

**现象**：三实体按序 spawn（A=1,2,3），给首个实体 `insert` 新组件后 query 全序——记忆预期 `[1,2,3]`，实测 `[3,2,1]`（2026-09-27，probe-run-r1.log，REAL_EXIT=101）：

```text
R1 实际迭代序 = [3.0, 2.0, 1.0]
assertion `left == right` failed: 记忆预期：迭代序 == spawn 序
  left: [3.0, 2.0, 1.0]
 right: [1.0, 2.0, 3.0]
```

**最小复现**（运行时行为，非编译失败；标 ignore 附理由）：

```rust,ignore
use bevy::prelude::*;

#[derive(Component)] struct A(f32);
#[derive(Component)] struct B(f32);

let mut world = World::new();
let e1 = world.spawn(A(1.0)).id();
let _e2 = world.spawn(A(2.0)).id();
let _e3 = world.spawn(A(3.0)).id();
world.entity_mut(e1).insert(B(9.0)); // e1 迁入 (A,B) archetype，原表行 swap_remove
let mut q = world.query::<&A>();
let order: Vec<f32> = q.iter(&world).map(|a| a.0).collect();
assert_eq!(order, vec![1.0, 2.0, 3.0]); // panic：实际 [3.0, 2.0, 1.0]
```

**根因**：官方文档明示「Iteration order is not guaranteed」（`bevy_ecs-0.19.1/src/system/query.rs:654` 等 8 处：654/685/723/762/792/824/1154/1183；bevy_ecs 全域 13 处）。机制：实体按 archetype 分表存储；`insert` 新组件把实体迁入新 archetype，原表行以 **swap_remove** 退出（`storage/table/mod.rs:226` `swap_remove_unchecked`、`:253` `entities.swap_remove`）——尾行补位使表内次序重排（本例 e1 让出的首行由尾行 e3 顶替 → [3,2]，新表 (A,B) 追加在后 → [1]）；跨表顺序同样无契约。仓库旁证：TS-12 实测 despawn 恰先取 index 0..9（带 Tagged 的独立 archetype 迭代序居首，docs/evidence/ts-12-brp.md）。

**修复**（已过运行验证）：任何依赖次序的逻辑按**业务键**排序/寻址（本仓先例：`Wanderer::index` 组件定位实体，taskset README 回归口径第 1 条），不依赖迭代序：

```rust,ignore
let mut sorted = raw_order.clone();
sorted.sort_by(|x, y| x.partial_cmp(y).unwrap()); // → [1,2,3] 确定（完整代码 probe-src-final/src/main.rs r1c_iteration_order_corrected；probe-run-r1c.log PASS）
```

**验证证据**：
- 复现：`probe-run-r1.log`（panic 原文，REAL_EXIT=101）；
- 修复：`probe-run-r1c.log`（实测序 [3,2,1] 如实打印 + 排序后断言 PASS，REAL_EXIT=0）。

### PIT-B-011：`Changed<T>` 首帧全量命中——包含 Query 首次运行之前的变更

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（change detection 的 tick 语义）
- 标签：`Changed` `Added` `change_detection` `首帧` `运行时`

**现象**：spawn 10 个实体后首帧跑 `Query<&A, Changed<A>>` 计数——记忆预期 0（「没人改过」），实测 10（2026-09-27，probe-run-r2.log，REAL_EXIT=101）：

```text
R2 首帧 Changed<A> 计数 = 10
assertion `left == right` failed: 记忆预期：首帧 Changed 计数 0
  left: 10
 right: 0
```

**最小复现**（运行时行为，标 ignore 附理由）：

```rust,ignore
use bevy::prelude::*;

#[derive(Component)] struct A(f32);
#[derive(Resource, Default)] struct Seen(usize);

fn count_changed(mut seen: ResMut<Seen>, q: Query<&A, Changed<A>>) {
    seen.0 = q.iter().count();
}

let mut world = World::new();
world.init_resource::<Seen>();
for i in 0..10 { world.spawn(A(i as f32)); }
let mut sched = Schedule::default();
sched.add_systems(count_changed);
sched.run(&mut world); // 首帧：Changed 命中全部 10（spawn 写入发生在 Query 首跑之前）
assert_eq!(world.resource::<Seen>().0, 0); // panic：实际 10
```

**根因**：官方文档明示 `Changed` 过滤器「**includes changes that happened before the first time this `Query` was run**」（`bevy_ecs-0.19.1/src/query/filter.rs:886-896`，`Added` 同型 `:663-670`）——change tick 自 genesis 起算，从未运行过的 Query 其 `last_run` 落后于此前一切写入（含 spawn 时的初始写入），首帧全部视为变更。

**修复**（已过运行验证）：语义上把首帧当「初始快照」处理——需要「真实增量」时跳过首帧，或用 `Added<T>`/显式版本号区分；修正预期后实测首帧 10 → 第二次运行 0（`probe-run-r2c.log`，REAL_EXIT=0，两值均断言通过）。

**验证证据**：
- 复现：`probe-run-r2.log`（REAL_EXIT=101）；
- 修复：`probe-run-r2c.log`（首帧 10、第二次 0 双断言 PASS，REAL_EXIT=0）。

### PIT-B-012：跨系统访问冲突不 panic——默认任意序执行，确定性须显式 `chain`

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（调度器歧义处理策略）
- 标签：`调度` `歧义` `ambiguity` `chain` `确定性` `运行时`

**现象**：两个系统都 `Query<&mut A>` 且不加排序——记忆预期（旧语料印象）「调度构建/运行 panic」，实测**不 panic**、双系统都执行（2026-09-27，probe-run-r3.log，REAL_EXIT=0）：

```text
R3 两个冲突系统无序共存：未 panic，Seen=11，A=12
```

**最小复现**（运行时行为，标 ignore 附理由）：

```rust,ignore
use bevy::prelude::*;

#[derive(Component)] struct A(f32);
#[derive(Resource, Default)] struct Seen(usize);

fn writer_one(mut seen: ResMut<Seen>, mut q: Query<&mut A>) {
    for mut a in &mut q { a.0 += 1.0; }
    seen.0 += 1;
}
fn writer_two(mut seen: ResMut<Seen>, mut q: Query<&mut A>) {
    for mut a in &mut q { a.0 += 10.0; }
    seen.0 += 10;
}

let mut world = World::new();
world.init_resource::<Seen>();
world.spawn(A(1.0));
let mut sched = Schedule::default();
sched.add_systems((writer_one, writer_two)); // ← 无 chain：冲突但放行
sched.run(&mut world); // 不 panic；先后顺序无保证
```

**根因**：访问冲突的强制检查只发生在**同一系统**的参数之间（→ PIT-B-013 B0001）；**跨系统**冲突属调度歧义（ambiguity），默认不检测不报错——歧义配置只是可选项（`bevy_ecs-0.19.1/src/schedule/schedule.rs:48-49` `ignored_scheduling_ambiguities` 仅为「需要报告时忽略哪些」的配置面），执行顺序在执行器实现间无契约。

**修复**（已过运行验证）：有依赖就显式定序 `(writer_one, writer_two).chain()`——5 轮后 A=56=1+5×(1+10) 逐位确定（`probe-run-r3c.log`；顺带实录：修正断言首版误写 55 忘初始值，被确定性实测打脸改 56——chain 的确定性反被意外实证，失败轮存档 `probe-run-r3c-attempt1.log`）。

**验证证据**：
- 复现：`probe-run-r3.log`（不 panic 原文输出）；
- 修复：`probe-run-r3c.log`（chain 定序断言 PASS，REAL_EXIT=0）。

### PIT-B-013：同系统冲突 Query 编译放行、首帧 panic B0001——正解 `ParamSet` / `Without`

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（SystemParam 初始化期冲突检查）
- 标签：`B0001` `ParamSet` `Without` `访问冲突` `运行时`

**现象**：同一系统两个 `Query<&mut A>`——**编译通过**（r1 探针实证，编译期不拦截），系统首次运行时 panic（2026-09-27，probe-run-r4.log，REAL_EXIT=101）：

```text
thread 'main' panicked at ...bevy_ecs-0.19.1/src/query/state.rs:216:13:
error[B0001]: Query<&mut A> in system probe_c::two_mut accesses component(s) A in a way that conflicts
with a previous system parameter. Consider using `Without<T>` to create disjoint Queries or merging
conflicting Queries into a `ParamSet`. See: https://bevy.org/learn/errors/b0001
```

**最小复现**（运行时 panic，非编译失败；标 ignore 附理由）：

```rust,ignore
use bevy::prelude::*;

#[derive(Component)] struct A(f32);

fn two_mut(mut a: Query<&mut A>, mut b: Query<&mut A>) { // ← 编译通过
    for mut x in &mut a { x.0 += 1.0; }
    for mut x in &mut b { x.0 += 2.0; }
}

let mut world = World::new();
world.spawn(A(1.0));
let mut sched = Schedule::default();
sched.add_systems(two_mut);
sched.run(&mut world); // ← 此处 panic：error[B0001]（state.rs:216）
```

**根因**：同系统参数间的访问冲突检查发生在**系统初始化**（QueryState 初始化路径 `bevy_ecs-0.19.1/src/query/state.rs:210-218` 的 panic，错误文案自带 `Without<T>`/`ParamSet` 建议）——编译期 derive 不拦截（r1 编译通过实证），闸门推迟到首个 run，冷启动测试不全的项目会拖到线上才炸。

**修复**（已过运行验证）：`ParamSet` 显式串行化访问（或用 `With`/`Without` 过滤器证明不相交）：

```rust,ignore
fn two_mut_set(mut set: ParamSet<(Query<&mut A>, Query<&mut A>)>) {
    for mut x in set.p0().iter_mut() { x.0 += 1.0; }
    for mut x in set.p1().iter_mut() { x.0 += 2.0; }
}
// probe-run-r4c.log：正常运行，A=4（1+1+2），REAL_EXIT=0
```

**验证证据**：
- 复现：`probe-run-r4.log`（B0001 panic 原文，REAL_EXIT=101）；
- 修复：`probe-run-r4c.log`（ParamSet 形态 PASS，REAL_EXIT=0）。

### PIT-B-014：缓冲事件 API 全面不存在——`write_event` / `read_event` / `register_event` / `add_event` 均已移除

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Event/Message 架构分流的 API 面）
- 标签：`Event` `Message` `write_event` `read_event` `register_event` `E0599` `trigger` `observer`

**现象**：按 EventReader 时代记忆写「事件缓冲」调用——`World::write_event` / `World::read_event` / `Commands::write_event` 全部 E0599（2026-09-27，`docs/evidence/m3-assets/batch-d/probe-r1-check.log`，REAL_EXIT=101 共 4 错含 P6）：

```text
error[E0599]: no method named `write_event` found for mutable reference `&mut bevy::bevy_ecs::world::World` in the current scope
...
error[E0599]: no method named `read_event` found for mutable reference `&mut bevy::bevy_ecs::world::World` in the current scope
...
error[E0599]: no method named `write_event` found for struct `bevy::bevy_ecs::system::Commands<'w, 's>` in the current scope
```

追加轮又实证注册侧旧称同样不存在：`app.register_event::<T>()` E0599（`probe-r1b-check.log`）。

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

#[derive(Event)]
struct Exploded {
    power: f32,
}

// EventReader 时代印象的「缓冲事件 API」在 0.19 全部不存在：
fn buffered_event_api_gone(world: &mut World, app: &mut App, mut commands: Commands) {
    world.write_event(Exploded { power: 1.0 });    // E0599
    let _n = world.read_event::<Exploded>();       // E0599
    commands.write_event(Exploded { power: 2.0 }); // E0599
    app.register_event::<Exploded>();              // E0599（add_event 同样不存在）
}
```

**根因**：0.19 的 Event 是**纯 observer 触发**、无缓冲存储——官方定义「To make an Event happen, you trigger it on a World using `World::trigger` or via a Command using `Commands::trigger`. This causes any Observer watching for that Event to run _immediately_, as part of the `World::trigger` call.」（`bevy_ecs-0.19.1/src/event/mod.rs:16-18`，核实 2026-09-27；引文去除 rustdoc 链接标记，文字逐字）。缓冲需求整体迁往 Message 系（`Messages<M>` 资源 + Writer/Reader/Mutator），注册入口 `app.add_message::<T>()`（`bevy_app-0.19.1/src/sub_app.rs:390`，实体为 `MessageRegistry::register_message` 调用 :395；幂等来自 `contains_resource::<Messages<T>>()` 守卫 :394-396）。World 侧对已注册消息有 `write_message`（`world/mod.rs:3015`），但**没有**任何 `*_event` 缓冲方法。

**修复**（已过编译 + 运行验证）：触发用 `trigger`；需要缓冲/多读者错峰读就改用 Message 系（见 PIT-B-018 注册坑、PIT-B-020 双缓冲语义）：

```rust
use bevy::prelude::*;

#[derive(Event)]
struct Exploded {
    power: f32,
}

#[derive(Resource, Default)]
struct Fired(usize);

let mut world = World::new();
world.init_resource::<Fired>();
world.add_observer(|_: On<Exploded>, mut fired: ResMut<Fired>| {
    fired.0 += 1;
});
world.trigger(Exploded { power: 1.0 }); // World 侧；Commands 侧为 commands.trigger(...)
assert_eq!(world.resource::<Fired>().0, 1); // 同步执行：trigger 返回前观察者已跑
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-d/probe-r1-check.log`（E0599 ×3，REAL_EXIT=101）、`probe-r1b-check.log`（register_event E0599）；
- 修复：`docs/evidence/m3-assets/batch-d/probe-r2-check.log`（`World::trigger` / `Commands::trigger` 形态编译通过，REAL_EXIT=0）；本条目正例 fence 含断言「trigger 后观察者恰执行一次」（doctest 机器断言）。

### PIT-B-015：生命周期观察者过滤写成 `On<Add<T>>`——正解是第二泛型 `On<Add, T>`

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（observer 泛型签名）
- 标签：`observer` `生命周期` `On` `Add` `Remove` `E0107` `Bundle`

**现象**：组件添加/移除的观察者按直觉写 `On<Add<T>>` 泛型过滤，E0107（2026-09-27，`docs/evidence/m3-assets/batch-d/probe-r1-check.log`）：

```text
error[E0107]: struct takes 0 generic arguments but 1 generic argument was supplied
   --> src\lib.rs:59:29
    |
 59 | fn p6_on_add_observer(_: On<Add<Health>>) {}
    |                             ^^^-------- help: remove the unnecessary generics
...
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

#[derive(Component)]
struct Health(f32);

// 生命周期过滤的「泛型组件」形态——Add 不是泛型：
fn on_add_wrong(_: On<Add<Health>>) {} // E0107
```

**根因**：`Add` 是无泛型的事件载荷结构体（`bevy_ecs-0.19.1/src/lifecycle.rs:337` `pub struct Add { pub entity: Entity }`）；组件过滤走 `On` 的**第二泛型** `B: Bundle`（`observer/system_param.rs:38` `pub struct On<'w, 't, E: Event, B: Bundle = ()>`）。官方示例即此形态（`examples/ecs/observers.rs:142` `fn on_add_mine(add: On<Add, Mine>, ...)`，:152 `On<Remove, Mine>`）。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Health(f32);

#[derive(Resource, Default)]
struct Fired(usize);

fn on_add_right(add: On<Add, Health>, mut fired: ResMut<Fired>) {
    fired.0 += 1;
    let _who = add.entity; // 载荷字段直读（官方 observers.rs:143 用法）
}

let mut world = World::new();
world.init_resource::<Fired>();
world.add_observer(on_add_right);
world.spawn(Health(100.0)); // spawn 即触发
assert_eq!(world.resource::<Fired>().0, 1);
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-d/probe-r1-check.log`（E0107 原文）；
- 修复：`docs/evidence/m3-assets/batch-d/probe-r2-check.log`（`On<Add, Health>` 编译通过，REAL_EXIT=0）；本条目正例 fence 含断言「spawn 后观察者恰执行一次」（doctest 机器断言）。

### PIT-B-016：`MessageMutator` 没有 `iter_mut`——可变迭代也是 `.read()`

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Message 系 API 命名）
- 标签：`MessageMutator` `iter_mut` `read` `E0599` `消息`

**现象**：消息可变访问按迭代器惯例写 `m.iter_mut()`，E0599（2026-09-27，`docs/evidence/m3-assets/batch-d/probe-r1b-check.log`）：

```text
error[E0599]: no method named `iter_mut` found for struct `bevy::bevy_ecs::message::MessageMutator<'w, 's, M>` in the current scope
   --> src\lib.rs:106:16
    |
106 |     for d in m.iter_mut() {
    |                ^^^^^^^^ method not found in `bevy::bevy_ecs::message::MessageMutator<'_, '_, Damage>`
...
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

#[derive(Message)]
struct Damage {
    amount: f32,
}

fn mutate_wrong(mut m: MessageMutator<Damage>) {
    for d in m.iter_mut() { // E0599：MessageMutator 无 iter_mut
        d.amount += 1.0;
    }
}
```

**根因**：`MessageMutator` 的 API 面只有 `read`（`message_mutator.rs:67`，产可变项的迭代器）/ `read_with_id`（:72）/ `par_read`（并行版）——「迭代未读过的消息并推进游标」统一叫 `read`，不按容器惯例分 `iter`/`iter_mut`。`MessageReader::read` 同名同构（只读）。

**修复**（已过编译验证；系统形态运行验证随探针 r2 编译面）：

```rust
use bevy::prelude::*;

#[derive(Message)]
struct Damage {
    amount: f32,
}

fn mutate_right(mut m: MessageMutator<Damage>) {
    for d in m.read() { // read() 产可变项；游标推进，重复调用不重读
        d.amount += 1.0;
    }
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-d/probe-r1b-check.log`（E0599 原文）；
- 修复：`docs/evidence/m3-assets/batch-d/probe-r2-check.log`（`.read()` 形态编译通过，REAL_EXIT=0）；本条目正例 fence 即 doctest 编译验证。

### PIT-B-017：`On<Message<T>>` 不存在——缓冲消息没有 observer 形态（事件/消息分流结论）

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Event/Message 架构边界）
- 标签：`On` `Message` `Event` `E0782` `observer` `分流`

**现象**：想用观察者监听缓冲消息，写 `On<Message<Damage>>`，E0782（2026-09-27，`docs/evidence/m3-assets/batch-d/probe-r1b-check.log`）：

```text
error[E0782]: expected a type, found a trait
   --> src\lib.rs:112:34
    |
112 | fn p12_on_message_observer(_: On<Message<Damage>>) {}
    |                                  ^^^^^^^^^^^^^^^
...
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

#[derive(Message)]
struct Damage {
    amount: f32,
}

// 给缓冲消息找 observer 形态：Message<Damage> 非类型（裸 Message 被解析为 trait）
fn observe_message_wrong(_: On<Message<Damage>>) {} // E0782
```

**根因**：0.19 的架构分流——**Event = 触发（trigger + observer，无缓冲）；Message = 缓冲队列（Writer/Reader/Mutator，不可 trigger）**。`On<E>` 要求 `E: Event`（`observer/system_param.rs:38`），Message 载荷不实现 Event，故不存在任何「观察消息」的监听形态；裸 `Message` 是 trait 名（与 derive 宏同名），`Message<Damage>` 连类型都不是。

**修复**（已过编译 + 运行验证）：按需求选边——要「观察/响应」用 Event；要「缓冲、多读者、隔帧可读」用 Message 系：

```rust
use bevy::prelude::*;

#[derive(Event)]
struct Damaged {
    amount: f32,
}

#[derive(Resource, Default)]
struct Fired(usize);

fn observe_event(dmg: On<Damaged>, mut fired: ResMut<Fired>) {
    fired.0 += 1;
    let _amount = dmg.event().amount;
}

let mut world = World::new();
world.init_resource::<Fired>();
world.add_observer(observe_event);
world.trigger(Damaged { amount: 1.0 });
assert_eq!(world.resource::<Fired>().0, 1);
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-d/probe-r1b-check.log`（E0782 原文）；
- 修复：本条目正例 fence 含断言「Event 侧观察恰执行一次」（doctest 机器断言）；消息侧缓冲读法见 PIT-B-018/020。

### PIT-B-018：未注册消息直接挂 `MessageWriter`——运行时 panic「Message not initialized」

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Message 注册机制）
- 标签：`Message` `MessageWriter` `add_message` `Messages` `注册` `运行时`

**现象**：EventReader 时代印象「事件队列随用随有」，不注册 `Messages<T>` 直接给系统挂 `MessageWriter`——编译通过，系统首次运行 panic（2026-09-27，`docs/evidence/m3-assets/batch-d/probe-run-r1.log`，REAL_EXIT=101）：

```text
thread 'TaskPool (1)' (2500) panicked at ...bevy_ecs-0.19.1\src\error\handler.rs:130:1:
Encountered an error in system `probe_d::writer_only`: Parameter `MessageWriter<Damage>::messages` failed validation: Message not initialized
If this is an expected state, wrap the parameter in `Option<T>` and handle `None` when it happens, or wrap the parameter in `If<T>` to skip the system when it happens.
...
```

**最小复现**（运行时 panic，非编译失败；标 ignore 附理由——panic 载体完整脚本见证据目录）：

```rust,ignore
use bevy::prelude::*;

#[derive(Message)]
struct Damage {
    amount: f32,
}

fn writer_only(mut w: MessageWriter<Damage>) {
    w.write(Damage { amount: 1.0 });
}

let mut world = World::new(); // ← 未注册 Messages<Damage>
let mut sched = Schedule::default();
sched.add_systems(writer_only);
sched.run(&mut world); // ← panic：Message not initialized
```

**根因**：`MessageWriter` 的系统参数校验要求 `Messages<T>` 资源已存在；注册入口是 `app.add_message::<T>()`（`bevy_app-0.19.1/src/sub_app.rs:390-399`，实体 `MessageRegistry::register_message` 调用 :395，幂等来自 `contains_resource` 守卫 :394-396）。裸 `World`/`Schedule` 环境等价做法是 `world.init_resource::<Messages<T>>()`（探针 R1c 实证单凭资源初始化即可通过校验）。报错文案自带两条容错出路：`Option<T>` 参数或 `If<T>` 跳过。

**修复**（已过运行验证）：

```rust
use bevy::prelude::*;

#[derive(Message)]
struct Damage {
    amount: f32,
}

#[derive(Resource, Default)]
struct Count(usize);

fn writer_once(mut w: MessageWriter<Damage>) {
    w.write(Damage { amount: 1.0 });
}

fn reader(mut c: ResMut<Count>, mut r: MessageReader<Damage>) {
    c.0 += r.read().count();
}

let mut world = World::new();
world.init_resource::<Messages<Damage>>(); // 注册（App 侧等价：app.add_message::<Damage>()）
world.init_resource::<Count>();
let mut sched = Schedule::default();
sched.add_systems((writer_once, reader).chain());
sched.run(&mut world);
assert_eq!(world.resource::<Count>().0, 1);
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-d/probe-run-r1.log`（panic 原文，REAL_EXIT=101）；
- 修复：`docs/evidence/m3-assets/batch-d/probe-run-r1c.log`（「R1c 注册后消息：单帧读到 1 条」，REAL_EXIT=0）；本条目正例 fence 即 doctest 运行验证。

### PIT-B-019：`init_state` 前未装 `StatesPlugin` 运行时 panic——且 `StatesPlugin` 不在 prelude

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（States 插件依赖 + 导出面）
- 标签：`StatesPlugin` `init_state` `StateTransition` `prelude` `E0425` `运行时`

**现象**（双重坑）：① 裸 `App::new()`（无 DefaultPlugins）直接 `init_state::<S>()`——panic（2026-09-27，`docs/evidence/m3-assets/batch-d/probe-run-r3.log`，REAL_EXIT=101）：

```text
thread 'main' (25028) panicked at ...bevy_state-0.19.1\src\app.rs:102:67:
The `StateTransition` schedule is missing. Did you forget to add StatesPlugin or DefaultPlugins before calling init_state?
...
```

② 补装插件时按惯例写 `use bevy::prelude::*` 就用——`StatesPlugin` **不在 prelude**，E0425（`docs/evidence/m3-assets/batch-d/probe-import-prelude-check.log` ×2 处，REAL_EXIT=101）：

```text
error[E0425]: cannot find value `StatesPlugin` in this scope
   --> src\main.rs:165:21
    |
165 |     app.add_plugins(StatesPlugin);
...
```

**最小复现**（运行时 panic，非编译失败；标 ignore 附理由——panic 载体完整脚本见证据目录）：

```rust,ignore
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
}

let mut app = App::new(); // ← 未装 StatesPlugin / DefaultPlugins
app.init_state::<GameState>(); // ← panic：StateTransition schedule is missing
```

**根因**：`init_state` 依赖 `StateTransition` schedule，由 `StatesPlugin` 建立且**必须先装**（`bevy_state-0.19.1/src/app.rs:96-103`：`warn_if_no_states_plugin_installed` 先警告，随后 `get_schedule_mut(StateTransition).expect(...)` :102-103 panic）；`insert_state`/`add_computed_state`/`add_sub_state` 同款检查（:129/:165/:195）。导出面上 `bevy::state` 模块存在（`bevy_internal/src/lib.rs:93-94` `bevy_state as state`），但 `bevy_state` 的 prelude 清单不含 `StatesPlugin`（`bevy_state/src/lib.rs:77-99`）——完整路径 `bevy::state::app::StatesPlugin`（`bevy_state/src/lib.rs:55` `pub mod app` + `app.rs:330`）。`DefaultPlugins` 内含它，故走默认插件链的项目感知不到。

**修复**（已过运行验证）：

```rust
use bevy::prelude::*;
use bevy::state::app::StatesPlugin; // 不在 prelude——完整路径

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
}

#[derive(Resource, Default)]
struct Fired(usize);

fn on_enter_menu(mut c: ResMut<Fired>) {
    c.0 += 1;
}

let mut app = App::new();
app.add_plugins(StatesPlugin); // 必须先于 init_state
app.init_resource::<Fired>();
app.init_state::<GameState>();
app.add_systems(OnEnter(GameState::Menu), on_enter_menu);
app.update();
assert_eq!(app.world().resource::<Fired>().0, 1); // 首个 update() 触发初始 OnEnter（语义见 SKILL §6.11）
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-d/probe-run-r3.log`（panic 原文 REAL_EXIT=101）、`probe-run-r4.log`（同款 panic）、`probe-import-prelude-check.log`（E0425 ×2，REAL_EXIT=101；复现实验：临时注释正确 import 重跑 check 后恢复，时序见 m3-plan §七 D 行）；
- 修复：`docs/evidence/m3-assets/batch-d/probe-run-r3c.log` / `probe-run-r4c.log`（装插件后 PASS，REAL_EXIT=0）；本条目正例 fence 即 doctest 运行验证。

### PIT-B-020：消息生命周期不是「单帧清空」——双缓冲 + 读者游标，新读者隔帧仍可读

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Messages 双缓冲实现）
- 标签：`Messages` `双缓冲` `MessageReader` `生命周期` `update` `语义`

**现象**：认知预期「消息单帧生命周期：下一帧清零」。单读者实测 `(run1 读 1 条, run2 再读 0 条)` 表面吻合（`docs/evidence/m3-assets/batch-d/probe-run-r2.log`），但这是**游标推进**不是清空——换「新读者隔一帧再读」就暴露真相：仍能读到 1 条（`probe-run-r2b.log`，REAL_EXIT=0）：

```text
R2 消息生命周期：run1 读 1 条，run2 再读 0 条
R2b 双缓冲错峰：reader_a 第 1 帧读 1 条，reader_b 第 2 帧读 1 条
```

**最小复现**（语义反例，非报错载体；标 ignore 附理由——完整对照脚本见证据目录）：

```rust,ignore
// 误读推理：run2 读 0 →「消息被清空」。
// 实际：同一 MessageReader 实例游标已推进到 1，第二帧无新消息可读。
// 对照证据（完整脚本见证据目录）：docs/evidence/m3-assets/batch-d/probe-run-r2.log
// （单读者 (1, 0)）与 probe-run-r2b.log（双读者错峰 (1, 1)）。
```

**根因**：`Messages<M>` 是**双缓冲**资源（`bevy_ecs-0.19.1/src/message/messages.rs:95-102`：`messages_a` 存最旧存活消息（字段 :98）、`messages_b` 存新消息（字段 :100））；`update()` 每帧一次交换并清最旧缓冲（:193-196，doc 原文「Swaps the message buffers and clears the oldest message buffer. In general, this should be called once per frame/update」）。每条消息因此**至少存活到下一次 update 之后**；各 `MessageReader` 持独立游标、互不影响——同读者第二帧读 0 是游标语义，新读者（游标 0）隔帧照读。推论：错峰/延迟读取是设计内行为，不是竞态。

**修复**（已过运行验证；按真实语义设计读者，勿按「单帧清空」直觉）：

```rust
use bevy::prelude::*;

#[derive(Message)]
struct Damage {
    amount: f32,
}

#[derive(Resource, Default)]
struct Count(usize);

fn writer_once(mut w: MessageWriter<Damage>) {
    w.write(Damage { amount: 1.0 });
}

fn reader(mut c: ResMut<Count>, mut r: MessageReader<Damage>) {
    c.0 += r.read().count();
}

let mut world = World::new();
world.init_resource::<Messages<Damage>>();
world.init_resource::<Count>();

let mut run_a = Schedule::default();
run_a.add_systems((writer_once, reader).chain());
run_a.run(&mut world);
let first = world.resource::<Count>().0; // 读者 A（实例 1）：读 1 条

world.resource_mut::<Messages<Damage>>().update(); // 帧边界：交换双缓冲

let mut run_b = Schedule::default();
run_b.add_systems(reader); // 新 Schedule = 新系统实例 = 新 MessageReader（游标 0）
run_b.run(&mut world);
let second = world.resource::<Count>().0 - first;

assert_eq!((first, second), (1, 1)); // 新读者隔帧仍读到：消息活过一次 update()
```

**验证证据**：
- 复现（误读对照）：`docs/evidence/m3-assets/batch-d/probe-run-r2.log`（单读者 (1, 0)）；
- 修复（语义实证）：`docs/evidence/m3-assets/batch-d/probe-run-r2b.log`（双读者错峰 (1, 1)，REAL_EXIT=0）；本条目正例 fence 即 doctest 运行验证。
### PIT-B-021：反射克隆 `clone_value` 已移除——正解 `reflect_clone`（Result 语义）

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Reflect trait API 迁移）
- 标签：`Reflect` `clone_value` `reflect_clone` `E0599` `反射`

**现象**：按旧语料写 `v.clone_value()` 克隆反射值，E0599（2026-09-27，`docs/evidence/m3-assets/batch-e/probe-r1-check.log`）：

```text
error[E0599]: no method named `clone_value` found for reference `&(dyn bevy::bevy_reflect::Reflect + 'static)` in the current scope
...
```

**最小复现**：

```rust,compile_fail
use bevy::reflect::Reflect;

#[derive(Reflect, Clone)]
struct Hp(f32);

fn clone_wrong(v: &dyn Reflect) -> Box<dyn Reflect> {
    v.clone_value() // E0599：clone_value 已移除
}
```

**根因**：反射克隆整体迁往 `reflect_clone`——`fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError>`（`bevy_reflect-0.19.1/src/reflect.rs:312`，核实 2026-09-27）；另有 `reflect_clone_and_take::<T>`（:321）。`clone_value` 在 bevy_reflect 0.19.1 全库无实现（全文 0 处；bevy_animation 有同名非 Reflect 方法，不相干）。Result 语义的边界：不可克隆字段（如非 Clone 依赖）返回 `ReflectCloneError` 而非 panic。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::prelude::*;

#[derive(Reflect, Clone, Debug, PartialEq)]
struct Hp(f32);

let v = Hp(3.0);
let cloned: Box<dyn Reflect> = v.reflect_clone().expect("Clone 型可克隆");
assert_eq!(*cloned.downcast_ref::<Hp>().unwrap(), Hp(3.0));
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-e/probe-r1-check.log`（E0599 原文）；
- 修复：`docs/evidence/m3-assets/batch-e/probe-r2-check.log`（p3c 形态编译通过，REAL_EXIT=0）；本条目正例 fence 含克隆后下转断言（doctest 机器断言）。

### PIT-B-022：`ReflectComponent::from_world` bounds 不满足——正解从注册表 `data::<ReflectComponent>()` 取，insert 已改三参

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（ECS 反射组件 API）
- 标签：`ReflectComponent` `from_world` `insert` `E0277` `反射` `BRP同源`

**现象**：按旧语料 `ReflectComponent::from_world(world)` 构造再 insert，编译报 bounds（2026-09-27，`docs/evidence/m3-assets/batch-e/probe-r1-check.log`）：

```text
error[E0599]: the associated function or constant `from_world` exists for struct `ReflectComponent`, but its trait bounds were not satisfied
...
```

**最小复现**：

```rust,compile_fail
use bevy::ecs::reflect::ReflectComponent;
use bevy::prelude::*;

#[derive(Component, Reflect)]
struct Marker;

fn insert_wrong(world: &mut World) {
    let rc = ReflectComponent::from_world(world); // E0599（bounds 版）：不可用形态
}
```

**根因**：0.19 的标准路径是**从注册表取类型数据**：`TypeRegistration::data::<ReflectComponent>()`（BRP `get_component`/`insert` 内部同源）；`insert` 签名已改为三参 `(entity: &mut EntityWorldMut, component: &dyn PartialReflect, registry: &TypeRegistry)`（`bevy_ecs-0.19.1/src/reflect/component.rs:153-159`，核实 2026-09-27）——组件值以 `PartialReflect` 传入且须携带注册表。**且组件须带 `#[reflect(Component)]` 属性**（仅 `derive(Component, Reflect)` 不注册该数据，运行时 `data::<ReflectComponent>()` 得 None——`component.rs:14`「When a user adds the `#[reflect(Component)]` attribute」；本条正例首版漏此属性，doctest 运行断言抓出后补，失败轮见门禁日志）。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::ecs::reflect::{AppTypeRegistry, ReflectComponent};
use bevy::prelude::*;

// 第二坑（doctest 机器断言抓出）：缺 #[reflect(Component)] 属性时
// data::<ReflectComponent>() 运行时返回 None（静默，expect 才 panic）——
// 该属性才注册 FromType<ReflectComponent> 数据（component.rs:14 文档；
// 本仓 sim.rs:96/:108/:127 全部组件同款实证）。
#[derive(Component, Reflect, Clone, Debug, PartialEq)]
#[reflect(Component)]
struct Marker(f32);

let mut world = World::new();
world
    .get_resource_or_insert_with::<AppTypeRegistry>(Default::default)
    .write()
    .register::<Marker>();
let entity = world.spawn_empty().id();
let value = Marker(7.0);
let arc = world.resource::<AppTypeRegistry>().0.clone();
let reg = arc.read(); // 守卫显式绑定（内联 &arc.read() 会 E0716 提前 drop）
let data = reg
    .get(std::any::TypeId::of::<Marker>())
    .and_then(|r| r.data::<ReflectComponent>())
    .expect("Marker 已注册");
data.insert(&mut world.entity_mut(entity), &value, &reg);
assert_eq!(world.get::<Marker>(entity), Some(&Marker(7.0)));
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-e/probe-r1-check.log`（bounds 版 E0599 原文）；
- 修复：`docs/evidence/m3-assets/batch-e/probe-r2-check.log`（p4c 形态编译通过，REAL_EXIT=0）；本条目正例 fence 含插入后读回断言（doctest 机器断言）。

### PIT-B-023：不透明反射属性 `#[reflect_value]` 已改名 `#[reflect(opaque)]`——且要求类型实现 `Clone`

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Reflect derive 属性面）
- 标签：`reflect_value` `reflect(opaque)` `不透明反射` `Clone` `属性`

**现象**：旧属性名 `#[reflect_value]` 不再存在（2026-09-27，`docs/evidence/m3-assets/batch-e/probe-r1-check.log`）：

```text
error: cannot find attribute `reflect_value` in this scope
...
```

改名后首试又踩附带 bound：`#[reflect(opaque)]` derive 要求 `Clone`（r2 修正轮实测 E0277，help 明示 `consider annotating with #[derive(Clone)]`）。

**最小复现**：

```rust,compile_fail
use bevy::reflect::Reflect;

#[derive(Reflect)]
#[reflect_value] // 属性不存在（0.19 已改名）
struct Wrap(f32);
```

**根因**：不透明反射属性现名 `#[reflect(opaque)]`（`bevy_reflect_derive-0.19.1/src/container_attributes.rs:394` 文档；用例见 `bevy_reflect/src/lib.rs` `#[cfg(test)] mod tests` 内 :3635/:4020，核实 2026-09-27）。三笔事实（独立审核以临时 crate 实测复核）：①opaque derive 要求 `Clone`（bound，E0277）；②`reflect_clone` 是 **PartialReflect** 的方法（`reflect.rs:101` trait 起、:312 定义，默认实现返回 `Err(ReflectCloneError::NotImplemented)`）——调用它须 `use PartialReflect`，仅 `use Reflect` 报 E0599（首版 doctest 失败的真因即漏此 import，help 明示「trait PartialReflect which provides reflect_clone is implemented but not in scope」）；③opaque 变体默认未实现 reflect_clone（运行期 NotImplemented），可克隆须再标 `#[reflect(Clone)]`（该属性生成实现 `container_attributes.rs:621-637`）；官方测试用例走 `data.apply(&patch)` 通路（opaque 的 `PartialReflect::apply` 经 `impls/opaque.rs:92-104` try_apply→Clone）。

**修复**（已过编译 + 运行验证；官方 doctest 同款 apply 通路）：

```rust
use bevy::reflect::{PartialReflect, Reflect};

#[derive(Reflect, Clone, Debug, PartialEq)] // opaque 要求 Clone（E0277 help 明示）
#[reflect(opaque)]
struct WrapC(f32);

let mut data = WrapC(1.0);
data.apply(&WrapC(2.0)); // PartialReflect::apply（须 use PartialReflect；
                         // reflect_clone 亦可，但须 #[reflect(Clone)] 才非 NotImplemented）
assert_eq!(data, WrapC(2.0));
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-e/probe-r1-check.log`（属性不存在原文）+ r2 修正轮 E0277 原文（`probe-r2-attempt1.log` 补录）+ 首版 doctest E0599 漏 import 失败轮（`gate-doc-test-attempt1.log` 补录）；
- 修复：`docs/evidence/m3-assets/batch-e/probe-r2-check.log`（REAL_EXIT=0）；本条目正例 fence 即 doctest 编译+运行验证。

### PIT-B-024：注册表资源面重排——`TypeRegistry` 非 Resource、裸 `World` 无 `register_type`、短名查改名 `get_with_short_type_path`

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（类型注册表的资源形态与访问协议）
- 标签：`TypeRegistry` `AppTypeRegistry` `register_type` `get_with_short_type_path` `E0277` `E0599` `RwLock`

**现象**：四连坑（2026-09-27，`docs/evidence/m3-assets/batch-e/probe-r1-check.log` + `probe-r1b-check.log`）：

```text
error[E0277]: `TypeRegistry` is not a `Resource`
...
error[E0599]: no method named `register_type` found for struct `bevy::bevy_ecs::world::World` in the current scope
...
error[E0599]: no method named `get_with_short_name` found for reference `&TypeRegistry` in the current scope
...
error[E0599]: no method named `len` found for reference `&TypeRegistry` in the current scope
...
```

**最小复现**：

```rust,compile_fail
use bevy::reflect::TypeRegistry;
use bevy::prelude::*;

fn registry_wrong(world: &World) -> usize {
    world.resource::<TypeRegistry>().len() // E0277：TypeRegistry 不是 Resource（也无 len）
}
```

**根因**：Resource 是包装器 `AppTypeRegistry(pub TypeRegistryArc)`（`bevy_ecs-0.19.1/src/reflect/mod.rs:35-41`，`#[derive(Resource)]` + Deref→`TypeRegistryArc`，核实 2026-09-27）；`TypeRegistryArc` 内 `Arc<RwLock<TypeRegistry>>`，访问须经 `.read()`/`.write()` 守卫（`bevy_reflect/src/type_registry.rs:569/:574`）。`register_type` 只在 App 侧（`bevy_app/src/app.rs:677`，写 `AppTypeRegistry`）；裸 `World` 须手动 `get_resource_or_insert_with::<AppTypeRegistry>` 后 `write().register::<T>()`（`TypeRegistry::register` :201）。短名查现名 `get_with_short_type_path`（:467）；`TypeRegistry` 无 `len`（计数用 `iter().count()` :547）。附带通用注意（Rust 层，不单独立条）：守卫须显式绑定变量——内联 `&arc.read()` 传参会 E0716 临时值提前 drop。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::ecs::reflect::AppTypeRegistry;
use bevy::prelude::*;
use bevy::reflect::Reflect;

#[derive(Reflect)]
struct Cfg(f32);

let mut world = World::new();
world
    .get_resource_or_insert_with::<AppTypeRegistry>(Default::default)
    .write()
    .register::<Cfg>();
let hit = world
    .resource::<AppTypeRegistry>()
    .read()
    .get_with_short_type_path("Cfg")
    .is_some();
assert!(hit); // App 上等价：app.register_type::<Cfg>() 后同样 read() 可查
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-e/probe-r1-check.log`（P9 E0277）+ `probe-r1b-check.log`（bin 侧 E0277 ×3 / E0599 ×3）；
- 修复：`docs/evidence/m3-assets/batch-e/probe-r2-check.log`（REAL_EXIT=0）+ `probe-run-r1c2.log`（注册+短名查 PASS）；本条目正例 fence 即 doctest 机器断言。

### PIT-B-025：`register::<T>` 注册的是**类型依赖闭包**——「注册 1 型得 1 型」预期被证伪

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（注册表递归注册语义）
- 标签：`register` `依赖闭包` `TypeRegistry` `list_components` `计数`

**现象**：运行时断言「注册 MyStruct 后 registry 恰 1 型」被证伪——实测 **20 型**（2026-09-27，`docs/evidence/m3-assets/batch-e/probe-run-r1c.log`，REAL_EXIT=101）：

```text
R1c 裸 World 经 AppTypeRegistry 注册后收录 20 型
...
thread 'main' (10024) panicked at src\main.rs:52:5:
assertion `left == right` failed: 注册后恰收录 1 型
  left: 20
 right: 1
...
```

**最小复现**（运行时语义反例，非编译载体；标 ignore 附理由——完整脚本与失败轮见证据目录）：

```rust,ignore
// 误读预期：register::<MyStruct>() 只登记 MyStruct 一型。
// 对照证据：docs/evidence/m3-assets/batch-e/probe-run-r1c.log（断言 ==1 失败，
// 实测 20）与 probe-run-r1c2.log（按依赖闭包语义复测 PASS）。
```

**根因**：`TypeRegistry::register::<T>` 在登记 T 后递归调用 `T::register_type_dependencies(self)`（`bevy_reflect-0.19.1/src/type_registry.rs:201-207`；doc :166「will also recursively register any type dependencies」，核实 2026-09-27）。依赖闭包层层展开：MyStruct → Inner/f32/String → std primitives 全家桶。推论：BRP `registry.list_components` 的可见集合同样含依赖型；对注册数做精确断言须按闭包口径，或改断言 `contains(TypeId)`。

**修复**（已过运行验证；按闭包语义断言）：

```rust
use bevy::ecs::reflect::AppTypeRegistry;
use bevy::prelude::*;
use bevy::reflect::Reflect;

#[derive(Reflect)]
struct Inner(f32);

#[derive(Reflect)]
struct Outer(Inner, String);

let mut world = World::new();
world
    .get_resource_or_insert_with::<AppTypeRegistry>(Default::default)
    .write()
    .register::<Outer>();
let n = world.resource::<AppTypeRegistry>().read().iter().count();
assert!(n > 1, "依赖闭包一并注册（Outer→Inner/String→primitives）");
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-e/probe-run-r1c.log`（断言失败原文，REAL_EXIT=101）；
- 修复：`docs/evidence/m3-assets/batch-e/probe-run-r1c2.log`（闭包语义复测 PASS「收录 20 型（含依赖闭包），MyStruct 在册 = true」，REAL_EXIT=0）；本条目正例 fence 即 doctest 机器断言。
### PIT-B-026：`EventReader` 类型整体不存在——事件读取全面 Message 化（含 `AssetEvent`）

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Event/Message 迁移的读取侧收尾）
- 标签：`EventReader` `AssetEvent` `MessageReader` `E0425` `资产` `事件`

**现象**：按旧语料给系统挂 `EventReader<AssetEvent<Image>>`，E0425——**类型本身没了**（2026-09-27，`docs/evidence/m3-assets/batch-f/probe-r1-check.log`）：

```text
error[E0425]: cannot find type `EventReader` in this scope
  --> src\lib.rs:17:38
   |
17 | fn f1_asset_event_reader(mut events: EventReader<AssetEvent<Image>>) {
   |                                      ^^^^^^^^^^^ not found in this scope
...
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn read_asset_events(mut events: EventReader<AssetEvent<Image>>) { // E0425：类型不存在
    for _e in events.read() {}
}
```

**根因**：0.19 的缓冲事件体系全面 Message 化——`EventReader` 类型在 bevy_ecs 0.19.1 中已删除（`pub struct EventReader` 全库 0 处）；`AssetEvent<A>` 直接 `#[derive(Message)]`（derive `bevy_asset-0.19.1/src/event.rs:49`、enum :50；同文件 `AssetLoadFailedEvent` :9-10、`UntypedAssetLoadFailedEvent` :27-28 同族，核实 2026-09-27）。读取侧统一 `MessageReader`（与 PIT-B-014/017 的分流总纲相接：写侧 `*_event` API 移除是同一迁移的另一半）。

**修复**（已过编译验证）：

```rust
use bevy::prelude::*;

fn read_asset_events(mut events: MessageReader<AssetEvent<Image>>) {
    for _e in events.read() {}
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-f/probe-r1-check.log`（E0425 原文）；
- 修复：`docs/evidence/m3-assets/batch-f/probe-r2-check.log`（f1c 形态编译通过，REAL_EXIT=0）；本条目正例 fence 即 doctest 编译验证。

### PIT-B-027：`Handle` 无 `clone_weak`——弱语义位是 `Handle::Uuid` 变体（且 `AssetId::Uuid` 是 struct variant）

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（Handle 重构后的形态）
- 标签：`Handle` `clone_weak` `Uuid` `AssetId` `E0599` `资产`

**现象**：按旧语料 `handle.clone_weak()`，E0599（2026-09-27，`docs/evidence/m3-assets/batch-f/probe-r1-check.log`）：

```text
error[E0599]: no method named `clone_weak` found for reference `&bevy::bevy_asset::Handle<bevy::bevy_image::Image>` in the current scope
  --> src\lib.rs:23:12
   |
23 |     handle.clone_weak()
   |            ^^^^^^^^^^
...
help: there is a method `clone` with a similar name
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn weak_wrong(handle: &Handle<Image>) -> Handle<Image> {
    handle.clone_weak() // E0599：clone_weak 不存在
}
```

**根因**：0.19 的 `Handle` 是 enum——`Handle::Strong(Arc<StrongHandle>)`（持活资产）/ `Handle::Uuid(Uuid, PhantomData)`（跨运行稳定标识，drop 不释放资产）（`bevy_asset-0.19.1/src/handle.rs:134-141`，核实 2026-09-27）。「不持活」的语义位是 **Uuid 变体**而非 clone 方法；运行时 `load` 默认得 Strong + `AssetId::Index`，Uuid 变体仅显式注册时存在。附带形态细节：`AssetId::Uuid` 是 **struct variant**（`id.rs:40` `Uuid { uuid: Uuid }`，非 tuple variant——match 须 `AssetId::Uuid { uuid }`，r2 修正轮实测）。

**修复**（已过编译验证）：

```rust
use bevy::asset::AssetId;
use bevy::prelude::*;

fn uuid_handle(handle: &Handle<Image>) -> Option<Handle<Image>> {
    match handle.id() {
        AssetId::Uuid { uuid } => Some(Handle::Uuid(uuid, std::marker::PhantomData)),
        _ => None, // 运行时 load 默认 Index id，无 Uuid 弱形态
    }
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-f/probe-r1-check.log`（E0599 原文）+ r2 修正轮 E0164（tuple 形态 match 失败——复现式补录 `probe-r2-attempt1.log`）；
- 修复：`docs/evidence/m3-assets/batch-f/probe-r2-check.log`（REAL_EXIT=0）；本条目正例 fence 即 doctest 编译验证。

### PIT-B-028：`DynamicScene` / `SceneRoot` / `DynamicSceneRoot` 全部移除——0.19 场景系统重构为 BSN 范式

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（场景系统代际更替）
- 标签：`DynamicScene` `SceneRoot` `bsn!` `spawn_scene` `场景` `E0425` `重构`

**现象**：按旧语料写 `DynamicScene::from_world(&world)`、`commands.spawn(SceneRoot(handle))`——类型全部 E0425（2026-09-27，`docs/evidence/m3-assets/batch-f/probe-r1-check.log`，共 4 处）：

```text
error[E0425]: cannot find type `DynamicScene` in this scope
...
error[E0425]: cannot find function, tuple struct or tuple variant `SceneRoot` in this scope
...
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn scene_wrong(commands: &mut Commands, scene: Handle<DynamicScene>) {
    commands.spawn(SceneRoot(scene)); // E0425：两者均不存在
}
```

**根因**：bevy_scene 0.19 **整体重构**为 BSN（Bevy Scene Notation）体系：prelude 导出面为 `bsn, bsn_list, on, template_value, CommandsSceneExt, EntityCommandsSceneExt, EntityWorldMutSceneExt, ..., WorldSceneExt`（`bevy_scene-0.19.1/src/lib.rs:900-906`）——`DynamicScene`/`SceneRoot`/`DynamicSceneRoot` 在 crate 内无定义（`pub struct DynamicScene` 全库 0 处，核实 2026-09-27）。新范式 = `World::spawn_scene(bsn! { ... })`（`WorldSceneExt::spawn_scene`，`spawn.rs:56`，返回 `Result<EntityWorldMut, SpawnSceneError>`；依赖未就绪用 `queue_spawn_scene`）。语法要点（r2 修正轮实测）：项 = 裸组件名 + **行分隔无逗号**（`crate::Ping(1.0),` 报「unexpected token」）；场景内组件需 `Clone + Default`（E0277 ×2：`FromTemplate` blanket impl 要求 `Clone + Default + Unpin`，`bevy_ecs-0.19.1/src/template.rs:390/:404`——Default 缺口由 2026-09-28 审核复现补录发现，见 `probe-r2-attempt1.log`）。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::prelude::*;
use bevy::scene::prelude::*;

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
struct Marker(f32);

let mut app = App::new();
app.add_plugins((
    bevy::app::TaskPoolPlugin::default(),
    bevy::asset::AssetPlugin::default(),
    bevy::scene::ScenePlugin,
));
app.register_type::<Marker>();
let world = app.world_mut();
let n0 = world.query::<&Marker>().iter(world).count();
let spawned = world
    .spawn_scene(bsn! {
        Marker(1.0)
    })
    .expect("场景应可解析");
let _ = spawned.id();
let world = app.world_mut();
let n1 = world.query::<&Marker>().iter(world).count();
assert_eq!(n1 - n0, 1); // BSN 单实体场景即时生成
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-f/probe-r1-check.log`（E0425 ×4）+ r2 修正轮两笔（bsn 逗号语法、Clone/Default bound——复现式补录 `probe-r2-attempt1.log`）；
- 修复：`docs/evidence/m3-assets/batch-f/probe-r2-check.log`（REAL_EXIT=0）+ `probe-run-r3c.log`（「BSN spawn 前后 Ping 计数 0 -> 1」，REAL_EXIT=0）；本条目正例 fence 即 doctest 机器断言。

### PIT-B-029：`Time<Virtual>` 缩放改名 `set_relative_speed`——且负值/非有限直接 panic

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_time API 命名）
- 标签：`Time` `Virtual` `set_scale` `set_relative_speed` `E0599` `缩放`

**现象**：按旧语料 `time.set_scale(2.0)`，E0599（2026-09-27，`docs/evidence/m3-assets/batch-f/probe-r1-check.log`）：

```text
error[E0599]: no method named `set_scale` found for mutable reference `&mut bevy::bevy_time::Time<bevy::bevy_time::Virtual>` in the current scope
  --> src\lib.rs:40:10
   |
40 |     time.set_scale(2.0);
   |          ^^^^^^^^^ method not found in `&mut bevy::bevy_time::Time<bevy::bevy_time::Virtual>`
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn scale_wrong(time: &mut Time<Virtual>) {
    time.set_scale(2.0); // E0599：已改名
}
```

**根因**：现名 `set_relative_speed`（f32 版 `bevy_time-0.19.1/src/virt.rs:188`、f64 版 :201）——语义为「相对系统时钟的推进速率」；getter `relative_speed` :148。**panic 语义**：负值或非有限直接 assert（:202-203「tried to go infinitely fast」/「tried to go back in time」），不是 Err。同族 `pause()`/`unpause()`/`delta()` 名称未变（r1 编译通过实证）。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::prelude::*;

let mut app = App::new();
app.add_plugins(MinimalPlugins);
app.update();
let mut v = app.world_mut().resource_mut::<Time<Virtual>>();
v.set_relative_speed(2.0);
assert_eq!(v.relative_speed(), 2.0);
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-f/probe-r1-check.log`（E0599 原文）；
- 修复：`docs/evidence/m3-assets/batch-f/probe-run-r2.log`（「relative_speed=2：virtual=0.001508800 real=0.000754400 比值=2.000」，REAL_EXIT=0；探针断言容差 |比值−2|<0.3、打印三位小数）；本条目正例 fence 即 doctest 机器断言。
- doctest 首跑失手留痕（`gate-doc-test-attempt1.log` E0596）：fix fence 初稿写 `let v`（非 mut）——`resource_mut` 返回 `Mut<T>`，经 DerefMut 调 `&mut self` 方法要求绑定本身为 mut；改 `let mut v` 后通过。该 Rust 层约束已并入 fence 本体。

### PIT-B-030：`AccumulatedMouseMotion` 不在 prelude + `load_folder` 的 `&str` 参数要求 `'static`

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（输入资源与资产服务器的导出面/签名）
- 标签：`AccumulatedMouseMotion` `prelude` `load_folder` `E0425` `E0521` `鼠标` `资产`

**现象**：两笔独立坑（2026-09-27，`docs/evidence/m3-assets/batch-f/probe-r1-check.log`）：

```text
error[E0425]: cannot find type `AccumulatedMouseMotion` in this scope
  --> src\lib.rs:55:28
   |
55 | fn f8_mouse_motion(mouse: &AccumulatedMouseMotion) -> Vec2 {
   |                            ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope
...
error[E0521]: borrowed data escapes outside of function
81 |     let _ = server.load_folder(path);
   |             ^^^^^^^^^^^^^^^^^^^^^^^^
   |             argument requires that `'1` must outlive `'static`
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn read_mouse_wrong(mouse: &AccumulatedMouseMotion) -> f32 { // E0425：不在 prelude
    mouse.delta.x
}
```

**根因**：①`bevy_input` 的 prelude 导出 `Axis, ButtonInput` 及 gamepad/keyboard/按钮/触摸项（`MouseButton` :61、`TouchInput`/`Touches` :65 等）——`AccumulatedMouseMotion`（`mouse.rs:218`，字段 `delta: Vec2`）须完整路径 `bevy::input::mouse::AccumulatedMouseMotion`（`bevy_input lib.rs:47-66` prelude 清单，核实 2026-09-27）；②`AssetServer::load_folder(path: impl Into<AssetPath<'a>>)`（`server/mod.rs:1115`）返回 `Handle<LoadedFolder>`——但 `&str` → `AssetPath` 的借用转换实测要求 `'static`（E0521），传字面量（满足 'static）或 `AssetPath::from(owned_string)` / `AssetPath::parse("dir").into_owned()`。

**修复**（已过编译验证）：

```rust
use bevy::input::mouse::AccumulatedMouseMotion;

fn read_mouse(mouse: &AccumulatedMouseMotion) -> f32 {
    mouse.delta.x
}

fn folder(server: &bevy::prelude::AssetServer) -> bevy::asset::Handle<bevy::asset::LoadedFolder> {
    server.load_folder("some/dir") // &'static str 字面量满足 'static
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-f/probe-r1-check.log`（E0425 + E0521 原文）；
- 修复：`docs/evidence/m3-assets/batch-f/probe-r2-check.log`（f8c/f13c 形态编译通过，REAL_EXIT=0）；本条目正例 fence 即 doctest 编译验证。
### PIT-B-031：`*Bundle` 整包类型全面移除（渲染/UI 域）——required components 裸组件 spawn

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bundle→required components 迁移的渲染/UI 面）
- 标签：`Camera3dBundle` `SpriteBundle` `TextBundle` `NodeBundle` `VisibilityBundle` `E0422` `E0433` `required_components`

**现象**：按 0.14-0.15 语料写整包 spawn，五类 Bundle 全部 E0422/E0433（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`）：

```text
error[E0422]: cannot find struct, variant or union type `Camera3dBundle` in this scope
  --> src\lib.rs:11:20
   |
11 |     commands.spawn(Camera3dBundle {
   |                    ^^^^^^^^^^^^^^ not found in this scope
```

（SpriteBundle/TextBundle/VisibilityBundle 同型 E0433；NodeBundle 为 E0422，原文同日志。）

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn spawn_camera_wrong(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0),
        ..default()
    }); // E0422：Camera3dBundle 不存在
}
```

**根因**：0.15 起 bundle 体系废除、0.19 沿用 required components 范式——`Camera3d` 等标记组件自带必需组件自动插入，spawn 只需标记组件 + 想定制的字段（本仓 `game/src/camera.rs:57-61` 即现行形态：`Camera3d::default()` + `Transform`）。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::prelude::*;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G1 Camera3dBundle/G13 NodeBundle E0422、G9 VisibilityBundle/G11 SpriteBundle/G12 TextBundle E0433 原文）；
- 修复：探针 g1c/g11c/g12c/g13c/g9c 编译通过（`probe-r2-check.log` + `probe-r2b-check-final.log` REAL_EXIT=0）；相机 spawn 形态运行验证 `probe-run-r5.log`（R2 forward dot=1）+ 本仓 `game/src/camera.rs` 同形态在跑。

### PIT-B-032：`Handle<Mesh>` / `Handle<StandardMaterial>` 直挂不是 Bundle——须 `Mesh3d` / `MeshMaterial3d` 包装组件

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（网格/材质组件位）
- 标签：`Handle` `Mesh3d` `MeshMaterial3d` `E0277` `PBR`

**现象**：0.14 记忆把 Handle 直接进 spawn 元组（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`）：

```text
error[E0277]: `(Handle<Mesh>, Handle<StandardMaterial>, Transform)` is not a `Bundle`
   --> src\lib.rs:25:20
    |
 25 |     commands.spawn((mesh, mat, Transform::default()));
    |              ----- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ invalid `Bundle`
    |              |
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn spawn_mesh_wrong(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mat = materials.add(StandardMaterial::default());
    commands.spawn((mesh, mat, Transform::default())); // E0277：Handle 不是组件
}
```

**根因**：Handle 泛型不实现 Component；网格/材质组件位是包装类型 `Mesh3d(Handle<Mesh>)` / `MeshMaterial3d<M: Material>(Handle<M>)`（本仓 `game/src/sim.rs:214-215`、`bevy-0.19.1/examples/3d/wireframe.rs:64` 同形态）。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::prelude::*;

fn spawn_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mat = materials.add(StandardMaterial::default());
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::default(),
    ));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G2 E0277 原文）；
- 修复：探针 g2c 编译通过（`probe-r2-check.log` REAL_EXIT=0）；本仓 `game/src/sim.rs:204-238` 同形态 50000 实体运行验证（ts-11 证据链）。
- 关联：`ResMut` 参数调 `Assets::add`（`&mut self`）要求参数绑定声明 `mut`——G2 修正轮同日志 `probe-r2-attempt1.log` E0596 三连，纯 Rust 层约束。

### PIT-B-033：相机配置外置伴生组件三连——`hdr` 字段→`Hdr` 组件、`target` 字段→`RenderTarget` 组件、`Msaa` 资源→per-Camera 组件

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_camera 配置模型重组）
- 标签：`Camera` `Hdr` `RenderTarget` `Msaa` `伴生组件` `E0560` `E0277`

**现象**：三笔独立旧形态同批失败（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`）：

```text
error[E0560]: struct `bevy::bevy_camera::Camera` has no field named `hdr`
  --> src\lib.rs:49:13
   |
49 |             hdr: true,
   |             ^^^ `bevy::bevy_camera::Camera` does not have this field
   |
   = note: available fields are: `viewport`, `order`, `is_active`, `computed`, `output_mode` ... and 4 others
```

（`target` 字段 E0560 同型（:243）；`insert_resource(Msaa::Sample4)` E0277「`bevy::bevy_render::view::Msaa` is not a `Resource`」原文同日志 :41。）

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn msaa_wrong(app: &mut App) {
    app.insert_resource(Msaa::Sample4); // E0277：Msaa 是组件不是资源
}

fn hdr_wrong(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera { hdr: true, ..default() }, // E0560：无 hdr 字段
        Transform::default(),
    ));
}
```

**根因**：相机配置从「结构体字段/全局资源」整体外置为**伴生组件**：
- HDR = 单元组件 `Hdr`（`bevy_camera-0.19.1/src/components.rs:89`，中间 HDR 纹理开关）；
- 渲染目标 = `RenderTarget` 组件（`camera.rs:890-892`，`#[derive(Component)]`；变体 `Window(WindowRef)` / `Image(ImageRenderTarget)`（非裸 Handle 包装）/ `TextureView` / `None { size }`；`Camera` 结构体现存字段见 E0560 帮助文本：`viewport/order/is_active/computed/output_mode...`）；
- MSAA = per-Camera 组件 `Msaa`（`bevy_render-0.19.1/src/view/mod.rs:240-246`，文档「Component for configuring the number of samples ... for a Camera」；**默认相机自动插入 `Msaa(Sample4)`**——探针运行时实测）。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::prelude::*;
use bevy::camera::{Hdr, RenderTarget};

fn msaa_hdr_target(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Msaa::Sample8,          // per-Camera 组件
        Hdr,                    // HDR 单元组件
        RenderTarget::Window(bevy::window::WindowRef::Primary), // 伴生组件
        Transform::default(),
    ));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G4/G5/G6/G19 原文）；
- 修复：探针 g4c/g5c/g19c 编译通过（`probe-r2-check.log` REAL_EXIT=0）；
- 运行时语义：`probe-run-r5.log` R3「camera Msaa=Sample4 samples=4」——默认相机带自动插入的 Msaa(Sample4)；探查过程两轮误判（先以为「无组件」后坐实「延迟 commands」）全程留痕 `probe-run-r2/r3/r4.log`。

### PIT-B-034：`StandardMaterial.emissive` 是 `LinearRgba` 非 `Color`——且 `Color` 不实现 `Mul<f32>`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（PBR 材质字段类型）
- 标签：`StandardMaterial` `emissive` `LinearRgba` `E0369` `自发光`

**现象**：0.15 记忆「emissive: Color * 强度」双错（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`）：

```text
error[E0369]: cannot multiply `bevy::bevy_color::Color` by `{float}`
  --> src\lib.rs:32:46
   |
32 |         emissive: Color::srgb(0.0, 1.0, 0.0) * 2.0,
   |                   -------------------------- ^ --- {float}
   |                   |
   |                   bevy::bevy_color::Color
```

（emissive 字段类型面由源码核实：`bevy_pbr-0.19.1/src/pbr_material.rs:92` 为 `LinearRgba`，与 `Color` 乘法错误同日志并存。）

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn emissive_wrong(mut materials: ResMut<Assets<StandardMaterial>>) {
    let _ = materials.add(StandardMaterial {
        emissive: Color::srgb(0.0, 1.0, 0.0) * 2.0, // E0369：Color 无 *f32
        ..default()
    });
}
```

**根因**：`emissive` 字段类型是 `LinearRgba`（`bevy_pbr-0.19.1/src/pbr_material.rs:92`）；`Color` 枚举不实现 `Mul<f32>`（0.19 移除）。强度语义 = 直接写高数值（cd/m²，物理单位），官方例 `examples/3d/bloom_3d.rs:39` `emissive: LinearRgba::rgb(0.0, 0.0, 150.0)`。`LinearRgba` 实现了向量运算（乘标量逐分量）。

**修复**（已过编译验证）：

```rust
use bevy::prelude::*;

fn emissive(mut materials: ResMut<Assets<StandardMaterial>>) {
    let _ = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(0.0, 1.0, 0.0) * 2.0, // LinearRgba 支持 *f32
        ..default()
    });
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G3 E0369 原文）；
- 修复：探针 g3c 编译通过（`probe-r2-check.log` REAL_EXIT=0）；形态出处 `examples/3d/bloom_3d.rs:39`。

### PIT-B-035：`AmbientLight` 从全局资源变为相机组件——全局环境光走 `GlobalAmbientLight`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_light 环境光架构）
- 标签：`AmbientLight` `GlobalAmbientLight` `E0277` `环境光`

**现象**：`insert_resource(AmbientLight{..})`（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`）：

```text
error[E0277]: `bevy::bevy_light::AmbientLight` is not a `Resource`
   --> src\lib.rs:74:25
    |
 74 |       app.insert_resource(AmbientLight {
   |  _________---------------^
   |         |
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn ambient_wrong(app: &mut App) {
    app.insert_resource(AmbientLight {
        brightness: 500.0,
        ..default()
    }); // E0277：AmbientLight 不是 Resource
}
```

**根因**：架构重组——`AmbientLight` 是挂在**相机**上的组件且 `#[require(Camera)]`（`bevy_light-0.19.1/src/ambient_light.rs:11`，文档句「can be added to a camera to override `GlobalAmbientLight`, which is the default」在 :8）；全局默认 = `GlobalAmbientLight` 资源（字段 `color/brightness/affects_lightmapped_meshes`，Default `WHITE/80.0/true` :82-90；用法 `examples/3d/skybox.rs:91-92` insert_resource）。

**修复**（已过编译验证）：

```rust
use bevy::prelude::*;

fn ambient(app: &mut App, mut commands: Commands) {
    // 每相机覆盖：
    commands.spawn((
        Camera3d::default(),
        AmbientLight { brightness: 500.0, ..default() },
        Transform::default(),
    ));
    // 全局默认：
    app.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 80.0,
        ..default()
    });
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G7 E0277 原文）；
- 修复：探针 g7c 编译通过（`probe-r2-check.log` REAL_EXIT=0）；修正轮实测 `GlobalAmbientLight` 缺 `affects_lightmapped_meshes` 字段 E0063（`probe-r2-attempt1.log`）。

### PIT-B-036：`Color` 具名常量只剩 `WHITE`/`BLACK`/`NONE`——具名色在 `palettes::css`（`Srgba` + `Color::from`）

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_color 导出面）
- 标签：`Color` `palettes` `css` `Srgba` `E0599` `颜色`

**现象**：`Color::GRAY`/`Color::RED`/`Color::GREEN` 全部 E0599（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log` 与 `probe-r1-bin-only-check.log`）：

```text
error[E0599]: no variant, associated function, or constant named `GRAY` found for enum `bevy::bevy_color::Color` in the current scope
  --> src\lib.rs:86:27
   |
86 |             color: Color::GRAY,
   |                           ^^^^ variant, associated function or constant not found in `bevy::bevy_color::Color`
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn gray_wrong() -> Color {
    Color::GRAY // E0599：具名常量已收窄
}
```

**根因**：`Color` 自身常量仅 `WHITE`/`BLACK`/`NONE`（`bevy_color-0.19.1/src/color.rs:503-510`）；完整具名色板在 `bevy::color::palettes::css`（`Srgba` 常量，css.rs），经 `Color::from(css::GRAY)` 或 `.into()` 进入 `Color`（官方例 `examples/3d/wireframe.rs:64` `Color::from(RED)`、`borders.rs` `BackgroundColor(MAROON.into())`）。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::color::palettes::css;
use bevy::prelude::*;

fn gray() -> Color {
    Color::from(css::GRAY)
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G8 GRAY）+ `probe-r1-bin-only-check.log`（RED/GREEN）；
- 修复：探针 g8c/g25c 编译通过（`probe-r2-check.log` REAL_EXIT=0）；运行时 `probe-run-r5.log` Gizmos 用 `Color::from(css::RED/GREEN)` 连续 6 帧绘制。

### PIT-B-037：UI 样式模型重构——`Style` 结构体不存在，样式字段直接长在 `Node` 上 + `px()`/`percent()` 单位函数

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_ui 布局模型）
- 标签：`Node` `Style` `px` `percent` `E0422` `UI` `布局`

**现象**：`NodeBundle { style: Style { ... } }` 双双不存在（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`）：

```text
error[E0422]: cannot find struct, variant or union type `NodeBundle` in this scope
error[E0422]: cannot find struct, variant or union type `Style` in this scope
   --> src\lib.rs:179:16
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn node_wrong(mut commands: Commands) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            ..default()
        },
        ..default()
    }); // E0422：NodeBundle 与 Style 均不存在
}
```

**根因**：0.19 UI 重构——**`Style` 的字段整体并入 `Node`**：`Node { width, height, margin, flex_wrap, border, align_items, justify_content, ... }`（`examples/ui/styling/borders.rs:95-99` 实形）；单位用自由函数 `px(T)` / `percent(T)`（`bevy_ui-0.19.1/src/geometry.rs:541/:558`）。`Node` 在 prelude。

**修复**（已过编译验证）：

```rust
use bevy::prelude::*;

fn node(mut commands: Commands) {
    commands.spawn(Node {
        width: percent(100.0),
        height: px(40.0),
        ..default()
    });
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G13 双 E0422 原文）；
- 修复：探针 g13c 编译通过（`probe-r2-check.log` REAL_EXIT=0）；`docs/evidence/m3-assets/batch-g/probe-src-final/lib.rs` g13c 形态。

### PIT-B-038：UI 文本/图像组件更名与多段文本形态——`TextStyle` 拆分、`UiImage`→`ImageNode`、`TextSection`→子实体 `TextSpan`、`TextAlignment`→`Justify`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_ui/bevy_text 组件面）
- 标签：`TextStyle` `TextFont` `TextColor` `UiImage` `ImageNode` `TextSection` `TextSpan` `TextAlignment` `Justify` `UI`

**现象**：四笔更名/形态迁移同批暴露（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`、`probe-r1c-lib-check.log`）：

```text
error[E0422]: cannot find struct, variant or union type `TextStyle` in this scope
error[E0599]: no associated function or constant named `from_sections` found for struct `bevy::bevy_ui::widget::Text` in the current scope
error[E0433]: cannot find type `TextSection` in this scope
error[E0433]: cannot find type `TextAlignment` in this scope
```

（`UiImage` E0433 见 `probe-r1-lib-check.log` :195。）

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn text_wrong(mut commands: Commands) {
    commands.spawn(TextBundle::from_section(
        "hello",
        TextStyle { font_size: 30.0, ..default() }, // E0422：TextStyle 不存在
    )); // TextBundle 亦不存在（PIT-B-031）
    commands.spawn((Node::default(), UiImage::default())); // E0433：UiImage→ImageNode
}
```

**根因**：①`TextStyle` 拆为 `TextFont { font, font_size: FontSize, .. }` + `TextColor(pub Color)` 两组件（`examples/ui/text/text.rs:37-43`；`TextFont::from_font_size(f32)` 便捷构造仍在）；②`UiImage` 更名 `ImageNode`（`bevy_ui lib.rs:70` widget 导出清单）；③多段文本改为**子实体**：根 `Text::new` + `.with_child((TextSpan::new("..."), TextFont{..}, TextColor(..)))`（`bevy_text text.rs:193` `pub struct TextSpan(pub String)`；官方例 text.rs:69-76/126-134）；④对齐 `TextAlignment` → `Justify` 枚举经 `TextLayout::justify(Justify::Center)`（text.rs:46）。

**修复**（已过编译验证）：

```rust
use bevy::prelude::*;

fn text(mut commands: Commands) {
    // 单段：
    commands.spawn((
        Text::new("hello"),
        TextFont::from_font_size(30.0),
        TextColor(Color::WHITE),
    ));
    // 多段（子实体 TextSpan）：
    commands
        .spawn(Text::new("root"))
        .with_child((
            TextSpan::new(" span"),
            TextFont::from_font_size(14.0),
            TextColor(Color::WHITE),
        ));
    // 图像节点：
    commands.spawn((Node::default(), ImageNode::default()));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G12/G15）+ `probe-r1c-lib-check.log`（G41/G42）；
- 修复：探针 g12c/g24/g15c/g41c/g42c 编译通过（`probe-r2-check.log` + `probe-r2b-check-final.log` REAL_EXIT=0）；
- 无坑面如实记：`Text::new + TextFont::from_font_size + TextColor` 的 0.15 记忆形态在 0.19 一次写对（G24 r1 编译通过）。

### PIT-B-039：`BorderColor` 从单一 Color 变每边字段 `{ top, right, bottom, left }`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_ui 边框组件）
- 标签：`BorderColor` `E0423` `边框` `UI`

**现象**：`BorderColor(Color::WHITE)` 元组形态（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1b-lib-check.log`）：

```text
error[E0423]: expected function, tuple struct or tuple variant, found struct `BorderColor`
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn border_wrong(mut commands: Commands) {
    commands.spawn((
        Node::default(),
        BorderColor(Color::WHITE), // E0423：BorderColor 是字段结构体非元组
    ));
}
```

**根因**：`BorderColor` 是每边一色结构体 `{ top: Color, right: Color, bottom: Color, left: Color }`（`bevy_ui-0.19.1/src/ui_node.rs:2256-2260`）；`impl<T: Into<Color>> From<T>` 提供全边同色捷径（:2263-2266，`BorderColor::all`）。

**修复**（已过编译验证）：

```rust
use bevy::prelude::*;

fn border(mut commands: Commands) {
    // 每边异色：
    commands.spawn((
        Node::default(),
        BorderColor { top: Color::WHITE, ..default() },
    ));
    // 全边同色捷径：
    commands.spawn((Node::default(), BorderColor::from(Color::WHITE)));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1b-lib-check.log`（G29 E0423 原文）；
- 修复：探针 g29c 编译通过（`probe-r2b-check-final.log` REAL_EXIT=0）。

### PIT-B-040：`Window.cursor` 字段移除——光标选项是独立 `CursorOptions` 组件

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_window 光标配置）
- 标签：`Window` `CursorOptions` `光标` `E0560` `E0422`

**现象**：`Window { cursor: Cursor { visible: false } }`（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`）：

```text
error[E0560]: struct `bevy::bevy_window::Window` has no field named `cursor`
   --> src\lib.rs:213:9
```

（`Cursor` 类型 E0422 同日志 ：213。）

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn cursor_wrong() -> Window {
    Window {
        title: "x".into(),
        cursor: Cursor { visible: false, ..default() }, // E0560 + E0422
        ..default()
    }
}
```

**根因**：光标行为配置外置为**独立组件** `CursorOptions`（`bevy_window-0.19.1/src/window.rs:752`），挂窗口实体，不占 `Window` 字段。`Window` 现存公有字段：`present_mode/mode/position/resolution/title/name/composite_alpha_mode/resize_constraints/resizable/enabled_buttons/decorations/transparent/focused/...`（window.rs:164-330 区域）——`title` 仍是公有 String 字段（运行时直读实证 `probe-run-r5.log` R1）。

**修复**（已过编译验证）：

```rust
use bevy::prelude::*;
use bevy::window::CursorOptions;

fn cursor(mut commands: Commands) {
    commands.spawn((
        Window::default(),
        CursorOptions { visible: false, ..default() },
    ));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G16 双错原文）；
- 修复：探针 g16c 编译通过（`probe-r2-check.log` REAL_EXIT=0）。
### PIT-B-041：窗口域 prelude 缺口——`WindowMode` / `MonitorSelection` / `PrimaryWindow` 须完整路径

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy prelude 导出面）
- 标签：`WindowMode` `MonitorSelection` `PrimaryWindow` `prelude` `E0425` `窗口`

**现象**：`use bevy::prelude::*` 下三类型 E0425（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log` 与 `probe-r1-bin-only-check.log`）：

```text
error[E0425]: cannot find type `PrimaryWindow` in this scope
```

（`WindowMode`/`MonitorSelection` E0425/E0433 原文见同两日志。）

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn mode_wrong() -> WindowMode { // E0425：不在 prelude
    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
}

fn query_wrong(windows: Query<&Window, With<PrimaryWindow>>) -> usize { // E0425
    windows.iter().count()
}
```

**根因**：`bevy_window` 的 prelude 面不含这三者；须 `use bevy::window::{WindowMode, MonitorSelection, PrimaryWindow};`（本仓 `game/src/main.rs:22` 先例：`use bevy::window::{PresentMode, WindowMode};`）。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

fn query(windows: Query<&Window, With<PrimaryWindow>>) -> usize {
    windows.iter().count()
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G17）+ `probe-r1-bin-only-check.log`（PrimaryWindow ×2）；
- 修复：探针 g17c + main 完整路径编译通过（`probe-r2-check.log` REAL_EXIT=0）；运行时 `probe-run-r5.log` R1 以 `With<PrimaryWindow>` 查询直读窗口实测。

### PIT-B-042：渲染域 prelude 缺口清单——`Bloom`/`Exposure`/`Hdr`/`Skybox`/`Wireframe`/`RenderTarget`/`Tonemapping`/`DebandDither`/`FocusPolicy`/`RenderLayers`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy prelude 导出面）
- 标签：`prelude` `Bloom` `Exposure` `Skybox` `Wireframe` `RenderTarget` `Tonemapping` `DebandDither` `FocusPolicy` `RenderLayers` `E0433` `E0432`

**现象**：渲染/后处理相关类型十项在 prelude 全部 E0433/E0432（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`、`probe-r1b-lib-check.log`、`probe-r1c-lib-check.log`、`probe-r2b-check.log`）：

```text
error[E0433]: cannot find type `Bloom` in this scope
error[E0433]: cannot find type `Exposure` in this scope
error[E0433]: cannot find type `Skybox` in this scope
error[E0433]: cannot find type `Tonemapping` in this scope
error[E0432]: unresolved import `bevy::render::view::RenderLayers`
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn bloom_wrong(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Bloom::NATURAL, Transform::default())); // E0433
}
```

**根因**：现行路径（逐项本地源码核实 2026-09-28）：
- `bevy::post_process::bloom::Bloom`（`bevy_post_process-0.19.1/src/bloom/settings.rs:33`，`#[require(Hdr)]`）；
- `bevy::camera::Hdr`（`bevy_camera components.rs:89`）；`bevy::camera::Exposure`（camera.rs:232）；`bevy::camera::RenderTarget`（camera.rs:892）；
- `bevy::light::Skybox`（`bevy_light probe.rs:229`）；
- `bevy::pbr::wireframe::{Wireframe, NoWireframe, WireframeColor, WireframePlugin, ...}`（`examples/3d/wireframe.rs:13-16`）；
- `bevy::core_pipeline::tonemapping::{Tonemapping, DebandDither}`（mod.rs:119/:383）；
- `bevy::ui::FocusPolicy`（`bevy_ui focus.rs:108`）；
- `RenderLayers` 在 `bevy::camera::visibility`（`render_layers.rs:20`；旧路径 `bevy::render::view::` 已失效，E0432 两轮实证，正解 `examples/2d/pixel_grid_snap.rs:4`）；
- `WindowRef` 定义在 `bevy::window`（bevy_camera 仅 use）。

**修复**（已过编译验证）：

```rust
use bevy::camera::{Exposure, Hdr};
use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

fn camera_stack(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Bloom::NATURAL,
        Exposure::INDOOR,
        Tonemapping::TonyMcMapface,
        RenderLayers::layer(1),
        Transform::default(),
    ));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/`（G6/G25/G26/G27/G19 覆盖 `probe-r1-lib-check.log`；G32/G33/G40 覆盖 `probe-r1b-lib-check.log`；G44 两轮 E0432：旧路径 `bevy::render::view::` 见 `probe-r1c-lib-check.log`、修正轮误路径 `bevy::camera::` 根见 `probe-r2b-check.log`）；
- 修复：探针 g6c/g26c/g27c（`probe-r2-check.log`）与 g32c/g33c/g40c/g44c（`probe-r2b-check-final.log`）编译通过，REAL_EXIT=0。

### PIT-B-043：光照阴影字段更名 `shadows_enabled` → `shadow_maps_enabled`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_light 字段名）
- 标签：`PointLight` `DirectionalLight` `shadow_maps_enabled` `E0560` `阴影`

**现象**：两种光源同字段 E0560（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`）：

```text
error[E0560]: struct `bevy::bevy_light::PointLight` has no field named `shadows_enabled`
  --> src\lib.rs:112:13
   |
112 |             shadows_enabled: true,
   |             ^^^^^^^^^^^^^^^ unknown field
   |
help: a field with a similar name exists
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn shadow_wrong(mut commands: Commands) {
    commands.spawn((
        PointLight { shadows_enabled: true, ..default() }, // E0560
        Transform::default(),
    ));
}
```

**根因**：现名 `shadow_maps_enabled: bool`（`bevy_light-0.19.1/src/point_light.rs:70`；`directional_light.rs:93`，默认 `false` :152）；另有独立 `contact_shadows_enabled` 字段（point_light.rs:73 附近）。本仓基线口径「DirectionalLight 默认无阴影」不变（`game/src/sim.rs:199-201` 注释）。

**修复**（已过编译验证）：

```rust
use bevy::prelude::*;

fn shadow(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default(),
    ));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G22/G23 E0560 原文）；
- 修复：探针 g22c/g23c 编译通过（`probe-r2-check.log` REAL_EXIT=0）。

### PIT-B-044：`Exposure::indoor()` 方法已成关联常量 `Exposure::INDOOR`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_camera 曝光 API）
- 标签：`Exposure` `INDOOR` `常量` `E0433`

**现象**：`Exposure::indoor()` 调用（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`，E0433 因 Exposure 不在 prelude 先行拦截；方法名迁移由源码核实坐实）：

```text
error[E0433]: cannot find type `Exposure` in this scope
  --> src\lib.rs:139:42
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn exposure_wrong(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Exposure::indoor(), Transform::default()));
    // E0433：不在 prelude；引 bevy::camera::Exposure 后 indoor() 亦不存在（是常量）
}
```

**根因**：`Exposure` 在 `bevy::camera`（camera.rs:232，字段 `ev100: f32`）；预设是关联**常量**：`SUNLIGHT`(15.0)/`OVERCAST`(12.0)/`INDOOR`(7.0)/`BLENDER`(9.7，默认)（camera.rs:238-253 + Default :279）；方法面仅 `from_physical_camera` / `exposure()`。

**修复**（已过编译验证）：

```rust
use bevy::camera::Exposure;
use bevy::prelude::*;

fn exposure(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Exposure::INDOOR, Transform::default()));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G26 E0433 原文）；
- 修复：探针 g26c 编译通过（`probe-r2-check.log` REAL_EXIT=0）。

### PIT-B-045：`Skybox` 双变化——`image` 是 `Option<Handle>` + 新增 `rotation: Quat` 字段

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_light 天空盒）
- 标签：`Skybox` `Option` `rotation` `E0063` `E0433`

**现象**：路径 E0433 之外，修正轮再踩两笔 E0063（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log` 与 `probe-r2-attempt1.log`）：

```text
error[E0063]: missing field `rotation` in initializer of `Skybox`
```

**最小复现**：

```rust,compile_fail
use bevy::camera::Hdr;
use bevy::light::Skybox;
use bevy::prelude::*;

fn skybox_wrong(mut commands: Commands, mut assets: ResMut<Assets<Image>>) {
    let image = assets.add(Image::default());
    commands.spawn((
        Camera3d::default(),
        Skybox { image, brightness: 1000.0 }, // E0063：缺 rotation；image 须 Option
        Transform::default(),
    ));
}
```

**根因**：`Skybox` 字段三件 `image: Option<Handle<Image>>`（:235）/ `brightness: f32`（:240）/ `rotation: Quat`（:245）（`bevy_light-0.19.1/src/probe.rs:229-245`）；官方形态 `examples/3d/skybox.rs:81-84`（`image: Some(handle)`）。

**修复**（已过编译验证）：

```rust
use bevy::light::Skybox;
use bevy::prelude::*;

fn skybox(mut commands: Commands, mut assets: ResMut<Assets<Image>>) {
    let image = assets.add(Image::default());
    commands.spawn((
        Camera3d::default(),
        Skybox {
            image: Some(image),
            brightness: 1000.0,
            rotation: Quat::IDENTITY,
        },
        Transform::default(),
    ));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G27 E0433）+ `probe-r2-attempt1.log`（E0063 rotation）；
- 修复：探针 g27c 编译通过（`probe-r2-check.log` REAL_EXIT=0）。

### PIT-B-046：`Wireframe` 单元组件化 + 颜色分离 `WireframeColor`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_pbr 线框）
- 标签：`Wireframe` `WireframeColor` `E0422` `线框`

**现象**：`Wireframe { color }` 构造 + prelude 双错（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`）：

```text
error[E0422]: cannot find struct, variant or union type `Wireframe` in this scope
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn wireframe_wrong(mut commands: Commands) {
    commands.spawn((Wireframe { color: Color::RED },)); // E0422：不在 prelude 且无 color 字段
}
```

**根因**：`Wireframe` 是**单元组件**（开关语义，三态：不挂/`Wireframe`/`NoWireframe`）；颜色经独立组件 `WireframeColor { color: Color }`（`bevy_pbr-0.19.1/src/wireframe.rs:845-846`）；全在 `bevy::pbr::wireframe` 模块，另有 `WireframePlugin`、全局 `WireframeConfig` 资源（`examples/3d/wireframe.rs:13-16/:34-37`）。

**修复**（已过编译验证）：

```rust
use bevy::color::palettes::css;
use bevy::pbr::wireframe::{Wireframe, WireframeColor};
use bevy::prelude::*;

fn wireframe(mut commands: Commands) {
    commands.spawn((
        Wireframe,
        WireframeColor { color: Color::from(css::RED) },
    ));
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（G25 E0422 原文）；
- 修复：探针 g25c 编译通过（`probe-r2-check.log` REAL_EXIT=0）。

### PIT-B-047：`Dir3::new` 返回 `Result<Dir3, InvalidDirectionError>`——表面「编译通过」在返回位才暴露

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_math 方向类型构造面）
- 标签：`Dir3` `Result` `E0308` `方向` `构造器`

**现象**：r1 轮即以 E0308 暴露（g20 探针带显式返回位，30 错之一，`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log` :304-317）；r2 修正轮 attempt1 因 g20 尚未修而同错复发（`probe-r2-attempt1.log` :40-52，引文如下）：

```text
error[E0308]: mismatched types
   --> src\lib.rs:289:5
    |
288 | pub fn g20_dir3(v: Vec3) -> Dir3 {
    |                             ---- expected `bevy::bevy_math::Dir3` because of return type
289 |     Dir3::new(v)
    |     ^^^^^^^^^^^^ expected `Dir3`, found `Result<Dir3, InvalidDirectionError>`
```

**最小复现**：

```rust,compile_fail
use bevy::math::{Dir3, Vec3};

fn dir3_wrong(v: Vec3) -> Dir3 {
    Dir3::new(v) // E0308：new 返回 Result
}
```

**根因**：`Dir3::new(value: Vec3) -> Result<Self, InvalidDirectionError>`（`bevy_math-0.19.1/src/direction.rs:563`）；同族 `new_unchecked`（:572，panic 语义由调用方担）、`new_and_length -> Result<(Self, f32), _>`（:587）、`from_xyz -> Result`（:596）。Dir2 同构（:153/:162）。**探查方法佐证**：g20 的探针函数带显式返回位，签名变化在 r1 首试轮即被 E0308 抓住——表达式级探查（如 `let _ = Dir3::new(v);`）会漏掉此类变化，探针函数应带显式返回位。

**修复**（已过编译验证）：

```rust
use bevy::math::{Dir3, Vec3};

fn dir3(v: Vec3) -> Option<Dir3> {
    Dir3::new(v).ok() // Result 显式处理；或 .expect(..) / new_unchecked
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log`（r1 首试 E0308 原文，:304-317）+ `probe-r2-attempt1.log`（修正轮未及修而复发，:40-52）；
- 修复：探针 g20c 编译通过（`probe-r2-check.log` REAL_EXIT=0）。

### PIT-B-048：`Query::single()` 返回 `Result`——`forward()` 等链式调用直接 E0599

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_ecs 查询 API 签名）
- 标签：`Query` `single` `Result` `E0599` `查询`

**现象**：`query.single().forward()` 链式调用（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1-bin-only-check.log`）：

```text
error[E0599]: no method named `forward` found for enum `std::result::Result<T, E>` in the current scope
  --> src\main.rs:58:18
   |
 58 |     let fwd = gt.forward();
   |                  ^^^^^^^ method not found in `std::result::Result<&bevy::bevy_transform::components::GlobalTransform, QuerySingleError>`
   |
note: the method `forward` exists on the type `&bevy::bevy_transform::components::GlobalTransform`
```

**最小复现**：

```rust,compile_fail
use bevy::prelude::*;

fn forward_wrong(query: Query<&GlobalTransform, With<Camera3d>>) -> f32 {
    let gt = query.single(); // 0.19：返回 Result
    gt.forward().z // E0599：Result 无 forward
}
```

**根因**：`Query::single(&self) -> Result<ROQueryItem<'_, 's, D>, QuerySingleError>`（`bevy_ecs-0.19.1/src/system/query.rs:2097`）——0.19 将「非唯一即 panic」改为显式 Result；须 `.expect(..)`/`.unwrap()`/`?` 解包（或 `single_mut` 同构）。错误帮助文本会精确指出方法存在于解包后类型上。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::prelude::*;

fn forward(query: Query<&GlobalTransform, With<Camera3d>>) -> f32 {
    let gt = query.single().expect("唯一相机");
    gt.forward().z
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1-bin-only-check.log`（forward/translation E0599 原文）；
- 修复：main r2 形态编译 + 运行验证（`probe-r2-check.log` + `probe-run-r5.log` R2，dot=1 断言通过）。

### PIT-B-049：Startup 同 schedule 内查询刚 spawn 的实体必空——commands 延迟应用（运行时 NoEntities 双踩）

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_ecs 延迟命令语义；与 PAT-B-003 正例面配对）
- 标签：`Commands` `延迟` `Startup` `NoEntities` `运行时` `调度`

**现象**：探针把「spawn 相机」与「查询相机」并排挂 Startup，查询 panic（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-run-r2.log` 与 `probe-run-r3.log` 两轮同型）：

```text
thread 'Compute Task Pool (5)' (25748) panicked at src\main.rs:72:31:
唯一相机: NoEntities("bevy_ecs::system::query::Query<'_, '_, &bevy_render::view::Msaa, ...With<bevy_camera::components::Camera3d>>")
```

**最小复现**（语义型，`ignore` 理由：panic 须真实 App 运行，doctest 内以正例 fence 表达对照）：

```rust,ignore
use bevy::prelude::*;

fn spawn(mut commands: Commands) {
    commands.spawn(Camera3d::default()); // 延迟：本 schedule 内未生效
}

// 同在 Startup 的查询系统拿到 NoEntities（运行时 panic 于 .single().expect()）。
```

**根因**：`Commands` 是**延迟写**——spawn 排队到下一个 apply_deferred 同步点才进 World；同 schedule 内排在后面的系统查不到（Block C 已建正例：`.chain()` + `auto_insert_apply_deferred` 在链间自动插同步点，PAT-B-003）。本条是它的运行时错题面：探针自身复踩两次（r2/r3），并把「默认相机无 Msaa 组件」误判为根因，r4/r5 才分离两因——**先排掉调度因素再下组件语义结论**的教训一并入库。

**修复**（已过编译 + 运行验证）：

```rust
use bevy::prelude::*;

fn spawn_later(mut app: App) {
    app.add_systems(Startup, spawn)
        .add_systems(Update, |query: Query<&Camera3d>| {
            assert_eq!(query.iter().count(), 1); // Update 时 Startup 命令已应用
        });
}

fn spawn(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-run-r2.log` + `probe-run-r3.log`（NoEntities 两轮原文）；
- 修复：`probe-run-r4.log`/`probe-run-r5.log`（查询移至 Update 后相机可见，R3 断言通过，REAL_EXIT=0）；正例面 PAT-B-003（`bevy-dev/patterns/PAT-B-003-*.md`）。

### PIT-B-050：`Val` 枚举变体重组——`Undefined` 移除，新增视口单位 `Vw`/`Vh`/`VMin`/`VMax`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题
- 通用性分级：bevy-specific（bevy_ui 几何单位）
- 标签：`Val` `Undefined` `Vw` `Vh` `视口单位` `E0599` `UI`

**现象**：`Val::Undefined`（2026-09-28，`docs/evidence/m3-assets/batch-g/probe-r1c-lib-check.log`）：

```text
error[E0599]: no variant, associated function, or constant named `Undefined` found for enum `bevy::bevy_ui::Val` in the current scope
```

**最小复现**：

```rust,compile_fail
use bevy::ui::Val;

fn val_wrong() -> Val {
    Val::Undefined // E0599：变体已移除
}
```

**根因**：现行变体 `Auto / Px(f32) / Percent(f32) / Vw(f32) / Vh(f32) / VMin(f32) / VMax(f32)`（`bevy_ui-0.19.1/src/geometry.rs:32-60`）——`Undefined` 语义并入 `Auto`（「按上下文自动」），并新增四种**视口相对单位**（VMin/VMax = 视口较小/较大边）；便捷构造函数 `px(T)`/`percent(T)`（:541/:558）。

**修复**（已过编译验证）：

```rust
use bevy::ui::Val;

fn val() -> Val {
    Val::Vw(50.0) // 视口宽 50%
}
```

**验证证据**：
- 复现：`docs/evidence/m3-assets/batch-g/probe-r1c-lib-check.log`（G43 E0599 原文）；
- 修复：探针 g43c 编译通过（`probe-r2b-check-final.log` REAL_EXIT=0）；`Val::Px`/`Val::Percent`/`Val::Auto` 未变（G37 r1 编译通过，`probe-r1b-lib-check.log` 无 G37 错）。

### PIT-B-051：BRP `world.mutate_components` 对不存在实体直接 panic 杀死整个游戏进程——内置写方法中唯一无优雅错误分支

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：错题（运行时行为；反例载体为 wire 请求与 panic 栈，非编译面——无 compile_fail 反例，用 `text` 围栏）
- 通用性分级：bevy-specific（bevy_remote handler 实现细节）
- 标签：`BRP` `world.mutate_components` `entity_mut` `panic` `进程崩溃` `-23401` `可用性` `净室重跑`

**现象**：M3 Block H 净室重跑 T004 错误探针首触发（2026-09-28；摘录 `docs/evidence/m3-cleanroom/t004-crash-evidence.txt`，裁判源码级复核 `docs/evidence/m3-cleanroom/judge-t004.md` §二）：

```text
{"jsonrpc":"2.0","method":"world.mutate_components","id":15,
 "params":{"entity":123456,"component":"demo::sim::Velocity","path":"linear","value":[1.0,0.0,0.0]}}
→ curl 退出码 52（Empty reply，响应 0 字节）
thread 'main' panicked at ...bevy_remote-0.19.1\src\builtin_methods.rs:1194:28:
Entity not yet spawned: The entity with ID 4294843839v0 is not spawned
Encountered a panic in system `bevy_remote::process_remote_requests`
（进程退出码 101；死后后续 BRP 调用连接拒绝，curl exit 7）
```

**根因**：handler 取组件反射前裸调 `world.entity_mut(entity)`（`builtin_methods.rs:1194`，`.reflect_mut(world.entity_mut(entity))`）——实体不存在时 `entity_mut` 直接 panic（非 Result 路径），panic 在 `process_remote_requests` 系统内未被捕获，沿 Main 调度击穿整个进程。对照同文件其余写方法的坏实体分支均温和（经 `get_entity_mut` → `-23401 Entity ... not found`，函数定义 `:1811`）：insert `:1129`、remove `:1304`、despawn `:1341`、reparent 挂父/解除 `:1359`/`:1370`——净室重跑 3 个对照探针（insert/remove/despawn，档 11-*/19-*/25-*）实测同码；reparent 据源码同型（其探针为自父 -23404，未做坏实体探针）。

**修复**（调用方防御；引擎侧属上游缺陷——正确修法应与同类方法一致改走 `get_entity_mut`）：

1. 工具/断言侧**禁止向 `world.mutate_components` 透传未验证实体号**：先 `world.get_components`（strict）或 `world.query` 确认目标存在，再发起 mutate；
2. 无预验证环境须有失败重启兜底；客户端侧坑指纹 = curl **52（空响应）+ 后续 7（拒连）**组合，服务端日志 panic 于 builtin_methods.rs。

**验证证据**（净室重跑 T004，2026-09-28；全档指针见 `docs/evidence/m3-cleanroom/t004-crash-evidence.txt`）：
- 复现：坏实体 mutate → 进程崩溃三重留档（请求原文 / curl 52 + 0 字节响应 / demo-run.log panic 栈，进程 101 + 死后拒连 exit 7）；
- 守卫序列运行验证：对先验证存在的实体 mutate 成功（200 result:null）+ 读回 `linear == [3,0,0]` 注入驻留（净室档 14-*/17-*/18-*）；
- 源码对照行由裁判 sed 亲验（judge-t004.md §二）；净新性声明：该语义不在任何注入资产（SKILL §6.1/6.2/6.4/6.9、PIT-B-004、台账 T004 行）中，原任务是否曾触发不可考（净室禁读原证据）。

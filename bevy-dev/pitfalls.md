# Bevy 特定层错题本

- 收录：通用性分级为 `bevy-specific`（换引擎后不再成立）的错题条目——API/版本/引擎行为相关的坑。
- 入库门禁与条目字段定义：见 [assets-methodology/pitfalls-schema.md](../assets-methodology/pitfalls-schema.md)（失败须在标注版本复现 + 修复过编译与运行验证；**未过验证禁止入库**）。
- 追加方式：按下方模板在文件末尾追加，`PIT-B-XXX` 三位自增（从 001 起），并在本行更新条数。
- 反例代码统一用 `rust,compile_fail` 围栏标记（doctest 断言其编译失败）；语义不符时用 `rust,ignore` 并附理由（先例见 assets-methodology/pitfalls.md PIT-M-001 的 shell 命令处理）。
- **doctest 门禁（M3 起，2026-09-27）**：本文件经 `docs/src/lib.rs` include_str! 纳入 `cargo test --doc -p docs`。围栏约定（对其后全部条目生效）：`rust,compile_fail`=反例（机器断言编译必败）；`rust,ignore`=非编译载体反例（附理由）或非 self-contained 修复片段（附理由 + 完整代码出处）；`rust`/`rust,no_run`=self-contained 修复正例（长运行标 no_run 附理由）。升级窗口换版本后反例如能编译，doctest 立即红——条目自动过期检测。
- 批次探查条目（M3 §3.1）的证据面：探针 crate 在仓库外不入 workspace，其「修复过编译」以探针 check 日志（归档 `docs/evidence/m3-assets/batch-*/`）为证据面，不要求 `cargo check --workspace`。
- 当前：13 条（2026-09-26 起；2026-09-27 M3 批次一新增 005–013，探查证据 `docs/evidence/m3-assets/batch-c/`）。

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

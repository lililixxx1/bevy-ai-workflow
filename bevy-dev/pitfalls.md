# Bevy 特定层错题本

- 收录：通用性分级为 `bevy-specific`（换引擎后不再成立）的错题条目——API/版本/引擎行为相关的坑。
- 入库门禁与条目字段定义：见 [assets-methodology/pitfalls-schema.md](../assets-methodology/pitfalls-schema.md)（失败须在标注版本复现 + 修复过编译与运行验证；**未过验证禁止入库**）。
- 追加方式：按下方模板在文件末尾追加，`PIT-B-XXX` 三位自增（从 001 起），并在本行更新条数。
- 反例代码统一用 ```` ```compile_fail ```` 标记（doctest 断言其编译失败）；语义不符时用 `ignore` 并附理由（先例见 assets-methodology/pitfalls.md PIT-M-001 的 shell 命令处理）。
- 当前：4 条（2026-09-26 起）。

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

```rust
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

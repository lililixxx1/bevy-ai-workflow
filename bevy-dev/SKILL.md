# bevy-dev skill v0.5 —— Bevy 0.19 开发纪律与约定（Bevy 特定层）

- 适用版本：**bevy 0.19**（当前 `Cargo.lock` 解析为 0.19.1；本文件全部 API 事实按 0.19.1 本地源码核实，核实日期 2026-09-26）。
- 层归属：Bevy 特定层（`bevy-dev/`），随升级窗口整体迁移（意向文档 §6）。引擎无关的纪律在 `assets-methodology/sop.md`，本文件不重复。
- 读者：任何在本仓库写/改 Bevy 代码、做 BRP 验证或回写本层资产的 AI agent。读本文即受其约束。
- 引用口径：`<reg>` 指 `~/.cargo/registry/src/` 下的 registry 目录（本机实测为 `rsproxy.cn-e3de039b2554c837`；换机/换镜像后以 `ls ~/.cargo/registry/src/` 实际结果为准）。形如 `bevy-0.19.1/Cargo.toml:2695` 的引用均相对该目录。
- **时效声明：第 6 节事实速查在 bevy 版本号变更（升级窗口）后全部作废，必须逐条重核再更新。**

## 0. 任务开场核对（每条都做，按序执行）

1. 读 `AGENTS.md`、本文件、`assets-methodology/sop.md`；涉及仓库结构、里程碑、范围边界的决策，先读《Bevy-AI开发意向文档.md》对应章节。
2. 确认版本未被擅动：根 `Cargo.toml` 的 `bevy = "0.19"`（`Cargo.toml:9`）与 `Cargo.lock` 一致；发现版本号在本仓库被改动且非升级窗口任务 → 停止并上报。
3. 确认工具链 ≥ MSRV：`rustc -V` ≥ 1.95.0（`rust-version` 声明于 `bevy-0.19.1/Cargo.toml:14`；低于此 cargo 在解析阶段即拒绝，见 `assets-methodology/pitfalls.md` PIT-M-001）。
4. 台账 `assets-methodology/task-ledger.md` 中上一任务已完整登记（缺 → 先补记再开工）。
5. 长时命令（cargo 全量编译等）后台跑 + 轮询，成败判定看**真实退出码**（`... > log 2>&1; echo REAL_EXIT=$?`），禁止管道末端判码（PIT-M-001 教训）。

## 1. 版本纪律（硬约束，违反即返工）

1. **版本锁**：`bevy` 的版本号只允许出现在根 `Cargo.toml` 的 `[workspace.dependencies]`（现状 `Cargo.toml:9`）。成员（`game/`、`tooling/*`、`docs/`）一律 `bevy = { workspace = true }`（现状 `game/Cargo.toml:9`），**禁止**在成员 manifest 里写 bevy 版本号。
2. **feature 追加在成员层**：需要 bevy feature 时（如 BRP 的 `bevy_remote`）在对应成员写 `{ workspace = true, features = ["bevy_remote"] }`；提交信息必须注明动了哪个 feature。feature 是版本敏感面，视同接口变更（见 §4.4）。
3. **Cargo.lock 入库**：禁止 gitignore；提交前 `git status` 确认 lock 变更与代码同提交。
4. **版本号变更仅限升级窗口**（0.20 起的例行迁移演练，意向文档 §4/§6）。平时任何提交不得动 bevy 版本号——包括「顺手升级补丁版本」与「临时降级绕 MSRV」（后者正确解法是升工具链，见 PIT-M-001）。
5. 工具链保持 Bevy MSRV 及以上的最新 stable（意向文档 §6）。

## 2. 查证流程（禁凭记忆写 API）

**原则：训练记忆中的任何 Bevy API 一律视为未验证。**「我记得有这个方法/参数/默认值」不构成依据；每个不确定符号都要走本节流程后才能进代码。

### 2.1 查证优先级（依序降级，前者优先）

1. **本地依赖源码**（最权威——它就是本次编译的目标代码）：`~/.cargo/registry/src/<reg>/<crate>-0.19.1/`。常用落点：
   - `bevy-0.19.1/src/`（facade 与重导出）、`bevy-0.19.1/examples/`（随包发布的官方示例）；
   - `bevy_ecs-0.19.1/src/`、`bevy_app-0.19.1/src/`（App/Plugin/schedule）、`bevy_reflect-0.19.1/src/`、`bevy_remote-0.19.1/src/`、`bevy_state-0.19.1/src/`（均已在本地 registry 核实存在）；
   - feature 定义在各 crate 的 `Cargo.toml [features]`（如 `bevy-0.19.1/Cargo.toml:2585` 起）。
   - 依赖源码未拉取时：在**仓库外**建临时 crate 声明该依赖并 `cargo fetch`（先例见台账 T002 证据），**禁止**为此改动本仓库 manifest。
2. **docs.rs 对应版本**：URL 必须带精确版本号（如 `docs.rs/bevy_remote/0.19.1/`），**禁用 `latest`**——它指向未来版本，正是要防的语料污染。
3. **官方 examples**：优先本地 `bevy-0.19.1/examples/`（与编译目标同版本），其次 GitHub 仓库对应 tag。

### 2.2 留痕规则（查证结论必须可追溯）

- 写进代码的查证结论：在使用处以注释留 `// 依据: <源码路径:行号 或 docs.rs URL>（核实 YYYY-MM-DD）`；同一事实多处使用时至少每个文件一处。
- 查证结论与记忆/任务种子信息冲突时：以源码为准，并把修正记入台账该任务的「原因/证据」列；构成踩坑的按 §4.3 回写错题本。
- 本文第 6 节是已核实结论的集中沉淀；引用速查条目可免重复留痕，但版本一变即失效（见时效声明）。

### 2.3 「已核实」≠「已验证」

- 读源码/docs.rs 只算**已核实**（签名、feature 名、默认值可信度提高）。
- **已验证** = 该 API 实际写进 `game/`/`tooling/` 并通过 `cargo check`（运行时行为另走 §4.2）。模式/错题条目的门禁更高，见 §4.3。
- 本文件与 `patterns/` 的代码片段**不进 doctest 门禁**（`docs/src/lib.rs:12` 只封装 `docs/doc-conventions.md`）。因此本层片段必须：要么逐字抄自官方示例/已验证条目并注明出处行号，要么显式标注「未过编译，首次使用须过 check」。

## 3. 架构约定

### 3.1 工程组织

- workspace 只有三类成员：`game/`（可执行，游戏本体/试金石）、`docs/`（doctest 门禁 crate）、`tooling/*`（自研件，游戏外进程）。新 crate 先问归属；不属于任何一类 → 上报，不塞进 `game/`。
- `game/src/` 目录定案（M1 第一役，2026-09-26）：`main.rs`（只组装插件）+ 按功能一模块一插件——`cli.rs`（参数解析）/ `rng.rs`（确定性 PRNG，纯逻辑）/ `sim.rs`（模拟组件+Resource+system 聚合）/ `camera.rs` / `brp.rs` / `bench.rs`（帧率采集）。新功能优先新模块+新插件，不在旧模块里堆系统；纯逻辑（无 Plugin）可独立小模块。

### 3.2 插件组织

- 功能单元 = `Plugin`（trait 方法签名 `fn build(&self, app: &mut App)`，`bevy_app-0.19.1/src/plugin.rs:57-59`）。一组紧密相关的组件 + Resource + system 打成一个自定义 Plugin；`main.rs` 只组装插件，不直接堆系统（官方组装形态参照 `bevy-0.19.1/examples/remote/server.rs:16-23`）。
- 插件依赖的 Resource 在自己的 `build` 里 `init_resource`，不得假设「别的插件恰好插过」。
- BRP 常驻插件（`RemotePlugin` + `RemoteHttpPlugin`）在 game 侧单独成一个模块/插件组，集中管理监听地址（§6.1）。

### 3.3 system 拆分

- 一 system 一事：一个 system 只读写一个领域的数据；参数表混入多个不相关领域的读写 → 拆。
- 副作用走 `Commands`（帧末延迟合并）；当帧必须生效的才用 `&mut World` 直改，且仅限无调度上下文的场景（BRP 自定义方法 handler 即属此类，§6.2）。
- 系统间有顺序依赖时用显式 ordering（`.before/.after` 或 SystemSet）声明，禁止依赖「注册顺序碰巧对」。

### 3.4 组件设计（含 BRP 反射注册约定，M1 起强制）

- 数据放组件，纯每帧可推导的量不落组件。
- **凡需被 BRP 查询/操控的组件或 Resource：derive `Reflect` + 对应 `#[reflect(...)]`，并在插件 build 中显式 `app.register_type::<T>()`**（`register_type` 签名：`bevy_app-0.19.1/src/app.rs:677`）。derive 形态逐字对齐官方 BRP 示例（`bevy-0.19.1/examples/remote/server.rs:69-70` Resource、`:89-91` Component）：

  ```rust
  use bevy::prelude::*;
  use serde::{Deserialize, Serialize};

  #[derive(Component, Reflect, Serialize, Deserialize)]
  #[reflect(Component, Serialize, Deserialize)]
  pub struct Velocity {
      pub linear: Vec2,
  }

  // 插件 build 中：
  // app.register_type::<Velocity>();
  ```

  （片段抄自上述官方示例同形态，**未在本仓库过编译**；首次写入 `game/` 时必须过 `cargo check`。）
- 为什么显式 `register_type` 仍强制：0.19 默认 feature 链（`default` → `2d`/`3d` → `default_app` → `reflect_auto_register`，`bevy-0.19.1/Cargo.toml:2742-2754`）会经 `inventory` 自动注册**非泛型**的 `#[derive(Reflect)]` 类型（`bevy_ecs-0.19.1/src/reflect/mod.rs:58-63`；`bevy_reflect-0.19.1/src/type_registry.rs:126-131`，Windows 在支持平台列表）；但**泛型类型不覆盖、`default-features = false` 即失效**。显式注册无条件成立且自文档化。
- 需要显式注册成为**唯一**通路时（如验证「未注册类型 BRP 拿不到」的反证实验），用容器属性 `#[reflect(..., no_auto_register)]` 退出自动链：属性语义 `bevy_reflect_derive-0.19.1/src/lib.rs:329-335`，标注后不发 `inventory::submit`（`bevy_reflect_derive-0.19.1/src/impls/common.rs:174-177`）。**已验证**（TS-09/T014，2026-09-26：标注 + 显式注册 → BRP 可见；注释显式注册 → `list_components` 立即不含该类型，312→311）。
- 未注册类型 BRP 拿不到：解析走 `AppTypeRegistry`，未注册报 `Unknown component type: ...`（`bevy_remote-0.19.1/src/builtin_methods.rs:608-613, 686`）。

### 3.5 Resource / State 使用边界

- Resource：全局唯一、与实体无关的状态（配置、随机源、资产句柄缓存）。禁止把实体数据放 Resource；Resource 里存 `Entity` 引用时必须处理悬垂（实体可能已被 despawn）。
- State：流程阶段（加载 → 菜单 → 对局……），只用于「阶段切换 + 进入/退出批量启停逻辑」。与阶段无关的单系统启停用普通 run condition，不为此造 State。
- State API 细节（derive 形态、调度集名称）**首次使用前**按 §2.1 查证 `bevy_state-0.19.1/src/`，禁止凭记忆写。

### 3.6 tooling 边界

- `tooling/` 下自研件是**游戏外**进程（RPC 客户端、测试驱动、截图工具），不链接 bevy 进游戏进程。游戏内新能力一律走 BRP 自定义方法（注册路径见 §6.2），这是自研件 `launch_level` / `run_tests` / `screenshot` 的唯一扩展通道（意向文档 §5.3）。

## 4. 验证流程

### 4.1 每次代码变更的门禁（无例外）

- `cargo check --workspace` 通过（真实退出码；全量编译可能超 10 分钟，后台跑 + 轮询）。失败即修，禁止带错提交。
- 涉及 `docs/` 变更：另跑 `cargo test --doc -p docs`，全绿才合并（意向文档 §5.1）。

### 4.2 运行时行为 → 工作流三（BRP 闭环）

- 判定：改动影响「运行起来才看得见」的行为（渲染、调度时机、交互、性能）→ 必须运行验证，不允许只凭 check 通过收工（AGENTS.md 硬约束 3）。
- 路径：`cargo run --release -p game`（后台）→ BRP over HTTP `http://127.0.0.1:15702`（事实依据 §6.1）→ **状态断言优先，截图辅助**（意向文档 §3 学界结论）。验证完杀干净游戏进程。
- M2 前（bevy_brp_mcp 与 `run_tests` 未就绪）用 curl 直调做通路冒烟与断言。冒烟首选无参方法 `rpc.discover`（方法集见 §6.2）：

  ```bash
  curl -s http://127.0.0.1:15702 \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"rpc.discover\",\"id\":1}"
  ```

  （请求体字段要求见 §6.4；缺 `"jsonrpc":"2.0"` 会被拒——这是实测过的源码行为，别按直觉省略。）
- 断言结果留痕：命令与响应摘录进台账「证据」列；截图/日志文件路径也记入证据列（run_tests 落地后按其口径统一）。

### 4.3 资产入库门禁（细则见 `assets-methodology/pitfalls-schema.md`，此处为本层执行要点）

- **模式条目**（正例）：代码过编译 + 运行验证后才可入库 → `bevy-dev/patterns/`，一条一文件，命名 `PAT-B-XXX-<slug>.md`，从 [`patterns/_TEMPLATE.md`](./patterns/_TEMPLATE.md) 复制起手，并在 `patterns/README.md` 索引表加行。
- **错题条目**（反例）：失败须在标注版本上复现（最小复现代码或失败原文）+ 修复方案过编译与运行验证，才可入库 → 追加进 [`bevy-dev/pitfalls.md`](./pitfalls.md)（内联模板已放好，id 形如 `PIT-B-001` 递增）。反例代码统一 ```` ```compile_fail ````；语义不符（如 shell 命令、行为差异）用 `ignore` 并附理由。
- **通用性分层**：换引擎后该教训仍成立 → `assets-methodology/`；不再成立 → `bevy-dev/`。判定拿不准 → 两边都不放，先在台账「原因」列记「待分层」，不要硬塞。
- **回写时机**：同一任务收尾前、最迟下次会话开始前（意向文档 §5.2）。
- **门禁未过 = 不入库**：宁可台账留「待验证教训」条目，不写半真半假的资产条目（Voyager 原则）。

### 4.4 文档同步义务

- 行为、接口、版本相关的变更必须同步更新 `docs/` 或模式库；纯重构、格式化、补丁版本升级豁免（意向文档 §6）。

## 5. 与 assets-methodology/ 的分工

| 内容 | 落点 | 判定 |
|---|---|---|
| Bevy API 用法、版本坑、引擎行为、feature 组合、查证结论 | `bevy-dev/`（本文件 + patterns/ + pitfalls.md） | 换引擎后不再成立 |
| 为什么要查证、如何留痕的**流程本身**；任务台账；错题 schema；任务测试集 | `assets-methodology/` | 换引擎后仍成立 |

- 本文件只写 Bevy 绑定的内容。写规则时自问：换到 Godot/Unity + 对应验证栈，这条还成立吗？成立 → 移到方法论层，这里只留链接。
- 两层都不掺私货（密钥、机器路径、内部黑话），保持随时可公开态（意向文档 §6）。`<reg>` 这类本机相关值用「以 `ls` 实测为准」的写法，不硬编码。

## 6. 已核实事实速查（bevy 0.19.1，2026-09-26 全部按 `<reg>` 本地源码核实）

> ⚠️ 升级窗口动版本号后，本节全部条目作废，逐条重核后更新并改此处日期。

### 6.1 BRP 常驻与监听

- 启用：成员依赖 `{ workspace = true, features = ["bevy_remote"] }`（feature 定义 `bevy-0.19.1/Cargo.toml:2695`：`bevy_remote = ["bevy_internal/bevy_remote"]`）。
- 游戏里挂两个插件：`RemotePlugin::default()` + `RemoteHttpPlugin::default()`（官方示例 `examples/remote/server.rs:18-19`；`RemotePlugin` 定义 `bevy_remote-0.19.1/src/lib.rs:572`，`RemoteHttpPlugin` 定义 `src/http.rs:113`）。
- 默认监听 `127.0.0.1`（`DEFAULT_ADDR`，`src/http.rs:60`）；主 app 端口 **15702**（`src/http.rs:52`），render 子 app 端口 15703（`src/http.rs:57`，启用 bevy_render 时）。
- 默认已是回环，但游戏代码仍**显式绑定**：`RemoteHttpPlugin::default().with_address(...)`（builder 方法 `with_address`/`with_port`/`with_headers` 见 `src/http.rs:164-212`）。安全约束（意向文档 §5.3）：仅 127.0.0.1，禁止 0.0.0.0/局域网。传参用 `Ipv4Addr::LOCALHOST`（std 的 `Into<IpAddr>` 转换；此防御性写法尚未在本仓库过编译，首次写入 game/ 时须过 check）。

### 6.2 BRP 方法集与自定义方法

- 内置方法共 **23** 个（`RemotePlugin::default()` 经 `add_default_methods` 全量注册，Default impl `src/lib.rs:791-803`；方法名常量表 `src/builtin_methods.rs:45-111`）：
  `world.get_components` / `world.query` / `world.spawn_entity` / `world.insert_components` / `world.remove_components` / `world.despawn_entity` / `world.reparent_entities` / `world.list_components` / `world.mutate_components` / `world.get_components+watch` / `world.list_components+watch` / `world.get_resources` / `world.insert_resources` / `world.remove_resources` / `world.mutate_resources` / `world.list_resources` / `world.trigger_event` / `world.write_message` / `world.observe+watch` / `registry.schema` / `schedule.list` / `schedule.graph` / `rpc.discover`。
  检索提示：常量名大多带 `_METHOD` 后缀，但 `schedule.list` / `schedule.graph` 的常量是 `BRP_SCHEDULE_LIST` / `BRP_SCHEDULE_GRAPH`（`src/builtin_methods.rs:102` / `:108`，**无** `_METHOD` 后缀）——grep 常量表时勿按后缀过滤（v0.1 前车之鉴）。两方法随 `bevy_dev_tools`（bevy_remote 的**非 optional** 依赖，`Cargo.toml:76-78`）默认可用，注册处 `src/lib.rs:779` / `:784`；`rpc.discover` 注册于 `src/lib.rs:719`。
- 自定义方法（自研件 `launch_level`/`run_tests`/`screenshot` 的注册通道）公开 API：**`with_method_main` / `with_method_render`**（`src/lib.rs:591` / `:601`）；watch 变体 `with_watching_method_main` / `with_watching_method_render`（`:632` / `:642`）。
- ⚠️ **常见误写：`with_method`**——0.19.1 中它是私有 `fn`（`src/lib.rs:611`，无 `pub`），外部不可调用。任务种子信息或旧资料出现 `RemotePlugin::default().with_method(...)` 时按本条纠正（本条即查证纪律的实例：种子信息也错，源码为准）。
- handler 形态：`impl IntoSystem<In<Option<Value>>, BrpResult, M>`（`src/lib.rs:591-599`，name 参数为 `impl Into<String>`），即 `fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult`；`BrpResult<T = Value> = Result<T, BrpError>`（`src/lib.rs:1428`），`Value` = `serde_json::Value`。handler 经 `world.run_system_with(id, message.params)` 独占执行（`src/lib.rs:1501`）——handler 内可直改世界（§3.3 认可的 `&mut World` 场景）。`BrpError` 三字段全公开（code/message/data，`lib.rs:1304-1312`），错误码复用 `error_codes` 常量模块（`lib.rs:1387` 起，如 `INVALID_PARAMS` = -32602）。**已验证**（T019，2026-09-26：`game/src/rpc/` 三方法注册运行，证据 `docs/evidence/m2-rpc.md`）。自定义方法会进 `rpc.discover` 方法清单（与内置方法同域，实测）。

### 6.3 BRP 与反射注册

- BRP 一切组件/资源操作经 `AppTypeRegistry` 解析（`src/builtin_methods.rs:608-613` get_components、`:626-630` resources）；未注册类型报 `Unknown component type: ...`（`:686`）/ `Unknown component: ...`（`:698`）。
- 自动注册的适用条件与局限见 §3.4（默认 feature 链 + 非泛型才自动；显式 `register_type` 无条件）。

### 6.4 BRP 请求体（JSON-RPC 2.0 over HTTP POST）

- 请求必须含 `"jsonrpc": "2.0"`（缺失报 `missing_field("jsonrpc")`，`src/lib.rs:1145-1146`；值非 "2.0" 报 invalid_value，`:1114-1118`）、`method`（字符串、大小写敏感，crate 文档 `:38`）、`id`（任意 JSON，响应原样带回）；`params` 按方法可省。
- 响应：成功含 `result`，失败含 `error`（`code`/`message` 恒在，`:65-90`）。
- 注意：crate 文档自己的请求示例（`src/lib.rs:15-25`）**漏了 `jsonrpc` 字段**，与反序列化器实际行为不符——文档也可能错，编译目标（源码）才是最终依据。

### 6.5 Hotpatching（0.19.1 现状，未验证部分如实标注）

- feature 存在：`hotpatching = ["bevy_internal/hotpatching"]`（`bevy-0.19.1/Cargo.toml:2802`），作用域为 `bevy_app/hotpatching` + `bevy_ecs/hotpatching` 两个下游 feature（`bevy_internal-0.19.1/Cargo.toml:282-285`）——即只覆盖 ECS/app 层。
- 运行方式（dx CLI）、Windows 成熟度、0.19 相对 0.17 的变化：**未在本仓库验证**。M1 冒烟任务（意向文档 §10 第 6 条）出结论前，不得把 Hotpatching 写进任何流程假设。

### 6.6 工具链

- bevy 0.19.1 MSRV = rustc **1.95.0**（`bevy-0.19.1/Cargo.toml:14`，`rust-version` 字段）。

### 6.7 事件与 observer（BRP `world.trigger_event` 通路）

- 事件定义：`#[derive(Event, Reflect, Serialize, Deserialize)]` + 容器属性
  `#[reflect(Event, Serialize, Deserialize)]`——`#[reflect(Event)]` 注册 `ReflectEvent` 数据
  （`bevy_ecs-0.19.1/src/reflect/event.rs:34-36` 文档），缺它时 BRP 报 `Event ... is not
  reflectable`、类型未注册报 `Unknown event type`（`bevy_remote-0.19.1/src/builtin_methods.rs:1492/1497`）。
  BRP 端按全路径 `event` + 可选 `value` 触发（`BrpTriggerEventParams`，`builtin_methods.rs:327-333`）；
  省略 `value` 时 handler 以 `DynamicStruct::default()` 经 `ReflectFromReflect` 构造
  （`builtin_methods.rs:1481-1516`；derive(Reflect) 自动注册 `ReflectFromReflect`，
  `bevy_reflect_derive-0.19.1/src/registration.rs:32`）——**0 字段（unit）事件无条件成功**，
  带字段事件走 TypedReflectDeserializer 需完整形态。
- observer 形态（0.19 系统参数为 `On<E>`，非旧语料的 `Trigger<E>`）：
  `fn on_x(_: On<E>, mut r: ResMut<R>)` + `app.add_observer(on_x)`
  （官方示例 `bevy-0.19.1/examples/ecs/observers.rs:142`；`App::add_observer`
  `bevy_app-0.19.1/src/app.rs:1474`）。`World::trigger` **同步**执行匹配 observer
  （`bevy_ecs-0.19.1/src/observer/mod.rs:63`）——BRP `trigger_event` 的 HTTP 响应返回前
  副作用已生效，紧接着的 `get_resources` 必读到新值。
- **已验证**（TS-10/T015，2026-09-26：unit 事件 `game::sim::PauseRequested` + observer 翻转
  `SimConfig.paused`，经 BRP 空载荷触发两连翻转，false→true→false 闭环；单测用
  `World::add_observer` + `World::trigger` 直证，`observer/mod.rs:55/63`）。

### 6.8 Screenshot 捕获管线（`bevy::render::view::screenshot`）

- 导入路径 `bevy::render::view::screenshot` 的实体文件是 `bevy_render-0.19.1/src/view/window/screenshot.rs`（经 `view/mod.rs:10` 的 `pub use window::*` 重导出）——按 `view/screenshot.rs` 路径 grep 会落空。
- 用法是**组件实体 + observer**（不是挂到相机上）：`world.spawn(Screenshot::primary_window()).observe(save_to_disk(path))`；完成是异步实体事件 `ScreenshotCaptured { entity, image }`（`screenshot.rs:49`），本机实测受理后 +3 帧完成。同一实体可并挂多个 observer（写盘 + 日志回填），相对次序无契约。
- `save_to_disk(path: impl AsRef<Path>) -> impl FnMut(On<ScreenshotCaptured>)`（`screenshot.rs:134`）：observer 内**同步**写盘（`to_rgb8` + `save_with_format`，格式按扩展名）。捕获实体完成后由 ScreenshotPlugin 的 `clear_screenshots`（First 调度）清理——完成轮询须经自有日志资源，不能靠查实体。
- 截图为**物理分辨率**：逻辑 1280x720 + DPI 1.25 的本机实得 1600x900——尺寸断言以物理分辨率为准。
- **已验证**（T019，2026-09-26：`game.screenshot` / `game.screenshot_log` 经此管线，PNG 魔数 + IHDR 尺寸与日志回填互证，证据 `docs/evidence/m2-rpc.md`）。

### 6.9 BRP 内省的进程内等价 API（`run_tests` 套件用，0.19.1 本地源码核实 2026-09-27）

- BRP `world.list_components` / `world.list_resources` 的数据源是 **AppTypeRegistry 中带 `ReflectComponent` / `ReflectResource` 数据的类型**（`builtin_methods.rs:1377-1400` 与 `:1414-1424` 两 handler 同构判定）——进程内套件直查同一注册表即等价：`world.resource::<AppTypeRegistry>().read().get(TypeId::of::<T>())` 再 `.data::<ReflectComponent>()`（`AppTypeRegistry` 定义 `bevy_ecs/src/reflect/mod.rs:36`；`TypeRegistry::get` `bevy_reflect/src/type_registry.rs:423`、`TypeRegistration::data` `:677`）。
- BRP `registry.schema` 的同源等价：`TypeRegistration::type_info() -> &'static TypeInfo`（`type_registry.rs:635`）→ `TypeInfo::Struct(&StructInfo)` → `StructInfo::field(name) -> Option<&NamedField>`（`structs.rs:165`）→ `NamedField::type_info()`（`fields.rs:58`）+ `TypeInfo::is::<T>()`（`type_info.rs:295`）可断言「字段存在且类型为 T」。
- `World::despawn(entity) -> bool`：实体不存在时 **warn + 返回 false**（不 panic；`world/mod.rs:1598-1605`）——进程内等价 BRP 侧 -23401 `ENTITY_NOT_FOUND` 的判定语义。存在性检查用 `World::get_entity`（`Result` 形态，`:951`，Err 即不存在）。
- `World::trigger`（`observer/mod.rs:63`）在 `run_tests` handler（`world.run_system_with` 上下文）内**实测同步执行 observer**（ts-10 套件：触发即翻转 `paused`，两次触发净零；与 §6.7 的 BRP `world.trigger_event` 行为一致）。
- 通用形态：`World::spawn` 返回 `EntityWorldMut`（`.id()` 取实体号，`world_mut.rs:181`），spawn/despawn/get_mut 直写**同帧生效**（无 Commands 延迟）——套件内「写→读→还原」无需跨帧。
- **已验证**（T020，2026-09-27：11 套件 77 断言经此组 API 落地并全绿，证据 `docs/evidence/m1-phase2.md`）。

---

## 变更记录

- **v0.5（2026-09-27）**：新增 §6.9 BRP 内省的进程内等价 API（list_components/list_resources 的注册表判定源、schema 的 TypeInfo/StructInfo 同源等价、`World::despawn` warn+false 语义、`World::trigger` 在 run_tests handler 内同步执行、spawn 直写同帧生效——T020 十一套件 77 断言实测）。
- **v0.4（2026-09-26）**：§6.2 handler 形态从「未过编译」改标**已验证**（T019 三方法落地；补 `run_system_with` 独占执行、`BrpError` 公开字段、`error_codes` 复用、`rpc.discover` 收录自定义方法四条实测事实）；新增 §6.8 Screenshot 捕获管线（导入路径与实体文件错位、组件实体+observer 形态、异步 +3 帧完成、物理分辨率口径，均 T019 实测）。

- **v0.3（2026-09-26）**：新增 §6.7 事件与 observer（TS-10 运行时验证 `world.trigger_event`
  空载荷触发 + `On<E>` observer 翻转资源后回写，未改既有约束）。

- **v0.2（2026-09-26）**：§3.4 增补 `#[reflect(no_auto_register)]` 条目（TS-09 运行时验证「显式注册唯一通路」后回写，未改既有约束）。

- **v0.1（2026-09-26）**：审核返工修复三处事实失准（未降低约束、未删证据）：①§6.2 内置方法清单漏 `schedule.list` / `schedule.graph`——常量名不带 `_METHOD` 后缀（`builtin_methods.rs:102/108`）致初版 grep 按后缀过滤漏检，补齐为 23 个并附注册行号（`lib.rs:719/779/784`）；②§6.2 `with_watching_method_render` 行号 `:641` → `:642`（641 是 `#[inline]` 属性行）；③§0.3 与 §6.6 的 `rust-version` 行号 `:13` → `:14`（与台账 T001 一致）。
- **v0（2026-09-26）**：首次成文（意向文档 §10 第 3 条 + §5.2）。五节纪律（版本/查证/架构/验证/分层）+ 事实速查；全部 API 事实按本地源码 `<reg>` 核实并带行号引用。核实过程中的两处修正：①种子信息中的 `RemotePlugin::with_method` 实为私有（§6.2）；②BRP 默认监听确认为 127.0.0.1:15702（§6.1）。bevy_remote 源码经仓库外临时 crate `cargo fetch` 拉取核实（未动本仓库 manifest）。

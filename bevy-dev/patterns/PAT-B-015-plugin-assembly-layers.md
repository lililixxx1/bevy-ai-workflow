# PAT-B-015-plugin-assembly-layers

### PAT-B-015：App 插件组装分层——窗口口径 → 领域插件(带配置) → 通路插件 → 工具插件

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（bevy_app 组装次序与配置传递形态）
- 标签：`App` `add_plugins` `插件组装` `SimConfig` `分层`
- 源码出处：`game/src/main.rs:25-60`（args→config→组装→run 全链）

**场景**：小型可验证 demo 的 main：CLI 参数先物化为领域配置结构，再按「引擎默认(带窗口定制) → 领域 → 相机 → RPC 通路 → 基准工具」分层组装，一次 `.run()`。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/main.rs:25-60` 的逐字删减）：

```rust
let args = cli::CliArgs::parse();

let sim_config = sim::SimConfig {
    entity_count: args.count,
    seed: args.seed,
    max_speed: args.max_speed,
    bench_secs: args.bench_secs,
    paused: false,
};

App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin { /* 见 PAT-B-011 */ ..default() }))
    .add_plugins(sim::SimPlugin { config: sim_config })
    .add_plugins(camera::OrbitCameraPlugin)
    .add_plugins(brp::BrpPlugin)
    .add_plugins(bench::BenchPlugin::default())
    .run()
```

要点：①领域插件吃**配置结构体**（`SimPlugin { config }`）而非散参——配置类型本身可 Reflect/serde（升级窗口回归面）；②通路插件（BrpPlugin）自带「方法注册 + 监听绑定」全部细节（PAT-B-018）；③工具插件（BenchPlugin）默认无操作，`--bench-secs` 才激活自退出计时——同一 main 支撑交互/基准两形态；④组装顺序即可读性：引擎层→领域→观察→通路→工具。

**为什么**：main 不写任何系统逻辑，只做「配置物化 + 组装」——所有行为归宿各自插件，BRP/单测/基准可分插件复用。

**验证证据**：
- 代码为 T003（M1 首役）已验证任务代码的逐字删减，删减后未单独重新编译；
- 运行验证：全部 12 任务以本组装运行（docs/evidence/m1-phase2/summary.txt 12 任务退出码全 0）；bench 形态 ts-11 自退出 REAL_EXIT=0。

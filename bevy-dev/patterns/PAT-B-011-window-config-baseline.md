# PAT-B-011-window-config-baseline

### PAT-B-011：窗口口径配置——`DefaultPlugins.set(WindowPlugin)` + CLI 直通 PresentMode

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（bevy_winit/bevy_window 插件配置形态）
- 标签：`WindowPlugin` `Window` `PresentMode` `WindowMode` `vsync` `基线口径`
- 源码出处：`game/src/main.rs:22`（PresentMode/WindowMode 完整路径引入）+ `:42-55`（Window 设置）

**场景**：帧率基线采集要求固定窗口口径（分辨率/窗口模式/vsync 开关可切换），且口径由 CLI 参数驱动，不重编译即可切换 vsync。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/main.rs:22/:42-55` 的逐字删减——保留依据注释原文，未改写）：

```rust
// PresentMode/WindowMode 不在 prelude，经 bevy::window 引入：
use bevy::window::{PresentMode, WindowMode};

App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "bevy-ai-workflow demo".into(),
            resolution: (1280, 720).into(),
            mode: WindowMode::Windowed,
            present_mode: if args.no_vsync {
                PresentMode::AutoNoVsync
            } else {
                PresentMode::AutoVsync
            },
            ..default()
        }),
        ..default()
    }))
```

要点：①`DefaultPlugins.set(WindowPlugin{..})` 是改主窗口配置的正规位；②`(w, h).into()` 直接得 `WindowResolution`；③vsync 走 `PresentMode::AutoVsync/AutoNoVsync`（均「处处支持」），由 CLI bool 直通——同一二进制覆盖两种基线口径；④`Window.title` 是公有 String 字段，运行时 `Query<&Window, With<PrimaryWindow>>` 可直读（探针实测，batch-g probe-run-r5.log R1）。

**为什么**：帧率基线五要素（release/分辨率/vsync/CPU-GPU-驱动/种子）中窗口口径占其二；vsync 可切换使「vsync 组 vs no-vsync 组」对照实验不用维护两份构建。

**验证证据**：
- 代码为 T003（M1 首役）已验证任务代码的逐字删减，删减后未单独重新编译（原形态在 `cargo check --workspace` 内）；
- 运行验证：M1 全部 12 任务在此窗口口径下执行（docs/evidence/m1-phase2/）；vsync 双口径对照见 ts-11（60.0 vs 无上限对照，docs/evidence/ts-11-brp.md）。

//! M1 第一役 demo：相机漫游 + N 个动态实体 + BRP 常驻（意向文档 §10 第 5 条）。
//!
//! 模块一览：[`cli`]（参数）、[`rng`]（确定性 PRNG）、[`sim`]（实体群与解析式运动）、
//! [`camera`]（轨道相机）、[`brp`]（127.0.0.1:15702 常驻）、[`bench`]（帧率采集）。
//!
//! 基线口径：release 构建、1280x720 窗口化、AutoVsync、种子 20260926——
//! 全量口径与阶梯数据见 `docs/fps-baseline.md`。

mod bench;
mod brp;
mod camera;
mod cli;
mod rng;
mod sim;

use bevy::prelude::*;
// PresentMode/WindowMode 不在 prelude，经 bevy::window 引入
//（依据: bevy-0.19.1/examples/window/window_settings.rs:8；bevy_window-0.19.1
// src/lib.rs:33 `pub use window::*`，核实 2026-09-26）。
use bevy::window::{PresentMode, WindowMode};

fn main() -> AppExit {
    let args = cli::CliArgs::parse();
    println!("{}", args.banner());

    let sim_config = sim::SimConfig {
        entity_count: args.count,
        seed: args.seed,
        max_speed: args.max_speed,
        bench_secs: args.bench_secs,
        paused: false,
    };

    App::new()
        // 窗口口径（基线五要素之二）：1280x720 窗口化 + vsync 开关（默认 AutoVsync，
        // `--no-vsync` 切换 AutoNoVsync）。
        // 依据: WindowPlugin/Window 设置形态 `bevy-0.19.1/examples/window/window_settings.rs:18-23`；
        // WindowResolution 默认 1280x720（bevy_window-0.19.1/src/window.rs:895-912）、
        // PresentMode 枚举（window.rs:1219 起，AutoVsync/AutoNoVsync 均「处处支持」）（核实 2026-09-26）。
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
        .add_plugins(sim::SimPlugin { config: sim_config })
        .add_plugins(camera::OrbitCameraPlugin)
        .add_plugins(brp::BrpPlugin)
        .add_plugins(bench::BenchPlugin::default())
        .run()
}

//! M3 Block F 探针（资产 / 场景 / 时间 / 输入域）——「凭记忆首试」载体。
//!
//! r1 轮结论（2026-09-27，REAL_EXIT=101，10 错；失败原文归档
//! docs/evidence/m3-assets/batch-f/probe-r1-check.log，r1 源码快照同目录）：
//! - 失败：EventReader 类型不存在（F1）、Handle 无 clone_weak（F2）、
//!   DynamicScene/SceneRoot 不存在（F3/F4/F12——0.19 场景系统整体重构为
//!   BSN/SceneComponent）、Time<Virtual> 无 set_scale（F5）、
//!   AccumulatedMouseMotion 不在 prelude（F8）、load_folder 参数需 'static（F13）。
//! - 编译通过（记忆写对，无坑）：Time pause/unpause/delta（F5 部分）、
//!   Time<Real>::delta_secs_f64（F6）、ButtonInput::just_pressed（F7）、
//!   load::<Image> turbofish（F9）、Assets::contains（F10）、
//!   Timer::new+TimerMode::Repeating（F11）。
//! 注：lib 失败阻断 bin 编译，main.rs 首试形态仅以 r1 快照存档（未及单独
//! 归档 bin 报错即进入查证——流程瑕疵，如实记）。
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
struct Ping(f32);

// F1【r1 失败】EventReader 类型整体不存在（0.19 事件读取全面 Message 化）。
// fn f1_asset_event_reader(mut events: EventReader<AssetEvent<Image>>) {
//     for _e in events.read() {}
// }

/// F1c（→PIT-B-026）：AssetEvent 是 Message（bevy_asset event.rs:47
/// `#[derive(Message)] pub enum AssetEvent<A>`），读取用 MessageReader。
fn f1c_asset_event_reader(mut events: MessageReader<AssetEvent<Image>>) {
    for _e in events.read() {}
}

// F2【r1 失败】Handle 无 clone_weak 方法（E0599，help 提示 clone）。
// fn f2_clone_weak(handle: &Handle<Image>) -> Handle<Image> {
//     handle.clone_weak()
// }

/// F2c（→PIT-B-027）：Handle 是 enum { Strong(Arc<StrongHandle>), Uuid(Uuid, PhantomData) }
/// （handle.rs:134-141）——「不持活资产」的语义位是 Uuid 变体（跨运行稳定标识，
/// drop 不释放）；运行时 load 默认得 Strong + AssetId::Index，Uuid 变体仅显式注册时用。
fn f2c_uuid_variant(handle: &Handle<Image>) -> Option<Handle<Image>> {
    // 修正轮实测：AssetId::Uuid 是 struct variant（id.rs，非 tuple variant）
    match handle.id() {
        bevy::asset::AssetId::Uuid { uuid: u } => {
            Some(Handle::Uuid(u, std::marker::PhantomData))
        }
        _ => None,
    }
}

// F3【r1 失败】SceneRoot 不存在（场景系统重构）。正解见 F3c。
// F4【r1 失败】DynamicScene 不存在。正解见 F3c。
// F12【r1 失败】DynamicScene::serialize 不存在。

/// F3c（→PIT-B-028）：0.19 场景范式 = `world.spawn_scene(bsn! { ... })`
/// （WorldSceneExt::spawn_scene，bevy_scene spawn.rs:56；官方 doctest lib.rs:65；
/// 旧 DynamicScene::from_world / SceneRoot / DynamicSceneRoot 全部移除）。
fn f3c_spawn_scene(world: &mut World) {
    use bevy::scene::prelude::*;
    // 修正轮实测：bsn! 项语法 = 裸组件名 + 行分隔（无逗号、无路径前缀）
    let _ = world.spawn_scene(bsn! {
        Ping(1.0)
    });
}

// F5【r1 失败】Time<Virtual> 无 set_scale（E0599）。
fn f5_virtual_time(time: &mut Time<Virtual>) {
    time.pause();
    time.unpause();
    let _d = time.delta();
}

/// F5c（→PIT-B-029）：缩放改名 `set_relative_speed`（virt.rs:188，
/// f64 版 :201；负值/非有限会 panic——assert :202-203）。
fn f5c_set_relative_speed(time: &mut Time<Virtual>) {
    time.set_relative_speed(2.0);
}

// F6（r1 编译通过）
fn f6_real_time(time: &Time<Real>) -> f64 {
    time.delta_secs_f64()
}

// F7（r1 编译通过）
fn f7_key_input(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::KeyW)
}

// F8【r1 失败】AccumulatedMouseMotion 不在 bevy::prelude（bevy_input prelude 仅
// Axis/ButtonInput/gamepad/keyboard 项，lib.rs:47-60）。正解 = 完整路径。
/// F8c（→PIT-B-030）：完整路径 `bevy::input::mouse::AccumulatedMouseMotion`
/// （mouse.rs:218，字段 `delta: Vec2`）。
fn f8c_mouse_motion(mouse: &bevy::input::mouse::AccumulatedMouseMotion) -> Vec2 {
    mouse.delta
}

// F9（r1 编译通过）
fn f9_load(server: &AssetServer) -> Handle<Image> {
    server.load::<Image>("icon.png")
}

// F10（r1 编译通过）
fn f10_assets_contains(assets: &Assets<Image>, handle: &Handle<Image>) -> bool {
    assets.contains(handle)
}

// F11（r1 编译通过）
fn f11_timer() -> Timer {
    Timer::new(std::time::Duration::from_secs(1), TimerMode::Repeating)
}

// F12 见 F3c 注（DynamicScene 已移除，序列化随新 Scene/Template 体系走，
// 本批不展开——记为探查边界）。

/// F13c（→PIT-B-030 附）：`load_folder(path: impl Into<AssetPath<'a>>)`
/// （server/mod.rs:1115）实测 &str 借用要求 'static（E0521）——传字面量或
/// `AssetPath::owned`；返回 `Handle<LoadedFolder>`。
fn f13c_load_folder(server: &AssetServer) -> Handle<bevy::asset::LoadedFolder> {
    server.load_folder("some/dir")
}

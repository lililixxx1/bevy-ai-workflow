//! M3 Block F 探针（资产 / 场景 / 时间 / 输入域）——「凭记忆首试」载体。
//!
//! 仓库外临时 crate。时序纪律（docs/m3-plan.md §3.1）：未查证 bevy 0.19.1
//! 源码前按训练语料直觉书写（主体 0.15/0.16 形态）；失败原文先归档
//! （docs/evidence/m3-assets/batch-f/），之后才查源码修正。首试：2026-09-27。
//!
//! 先验摇摆点：AssetEvent 的 Reader/Message 形态、Handle clone_weak 存续、
//! load_folder 公开性、scene 序列化路径、AccumulatedMouseMotion 字段/方法。
//!
//! 已验证不再探（本仓证据）：MessageWriter/Reader 双缓冲语义（PIT-B-018/020）；
//! Time::delta_secs_f64（bench.rs:68，SKILL 已录）。
#![allow(dead_code)]

use bevy::prelude::*;

// F1：资产事件的读取形态（先验摇摆：EventReader 旧形 vs Message 化）
fn f1_asset_event_reader(mut events: EventReader<AssetEvent<Image>>) {
    for _e in events.read() {}
}

// F2：弱句柄（先验摇摆：clone_weak 存续与否）
fn f2_clone_weak(handle: &Handle<Image>) -> Handle<Image> {
    handle.clone_weak()
}

// F3：场景根组件（0.15 起 required-components 形态，预期无坑）
fn f3_scene_root(commands: &mut Commands, scene: Handle<DynamicScene>) {
    commands.spawn(SceneRoot(scene));
}

// F4：从世界捕获动态场景（预期无坑）
fn f4_dynamic_scene_from_world(world: &World) -> DynamicScene {
    DynamicScene::from_world(world)
}

// F5：虚拟时间的暂停/缩放/增量（预期无坑）
fn f5_virtual_time(time: &mut Time<Virtual>) {
    time.pause();
    time.unpause();
    time.set_scale(2.0);
    let _d = time.delta();
}

// F6：真实时间增量（预期无坑）
fn f6_real_time(time: &Time<Real>) -> f64 {
    time.delta_secs_f64()
}

// F7：按键输入（预期无坑）
fn f7_key_input(keys: &ButtonInput<KeyCode>) -> bool {
    keys.just_pressed(KeyCode::KeyW)
}

// F8：累计鼠标位移（先验摇摆：字段还是方法）
fn f8_mouse_motion(mouse: &AccumulatedMouseMotion) -> Vec2 {
    mouse.delta
}

// F9：泛型化加载（预期无坑）
fn f9_load(server: &AssetServer) -> Handle<Image> {
    server.load::<Image>("icon.png")
}

// F10：资产集包含性（预期无坑）
fn f10_assets_contains(assets: &Assets<Image>, handle: &Handle<Image>) -> bool {
    assets.contains(handle)
}

// F11：定时器重复模式（预期无坑）
fn f11_timer() -> Timer {
    Timer::new(std::time::Duration::from_secs(1), TimerMode::Repeating)
}

// F12：场景序列化（先验摇摆：bevy_scene 的 serialize 路径与 feature）
fn f12_scene_serialize(scene: &DynamicScene, registry: &bevy::reflect::TypeRegistry) -> String {
    scene.serialize(registry).expect("序列化")
}

// F13：目录批量加载（先验摇摆：load_folder 公开性）
fn f13_load_folder(server: &AssetServer, path: &str) {
    let _ = server.load_folder(path);
}

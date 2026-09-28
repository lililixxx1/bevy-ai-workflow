//! M3 Block F 运行时探针——三个「凭记忆预期」的行为假设。首试：2026-09-27（未查源码）。
use bevy::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Ping(f32);

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    match which.as_str() {
        "r1" => r1_virtual_pause_freezes_delta(),
        "r2" => r2_virtual_scale_doubles_delta(),
        "r3" => r3_scene_roundtrip(),
        _ => panic!("usage: probe-f <r1|r2|r3>"),
    }
}

// R1 记忆预期：Time<Virtual> pause 后 virtual delta 冻结为 0，而 Time<Real> 继续增长。
fn r1_virtual_pause_freezes_delta() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.update();
    let real0 = app.world().resource::<Time<Real>>().delta_secs_f64();
    app.world_mut().resource_mut::<Time<Virtual>>().pause();
    app.update();
    let v = app.world().resource::<Time<Virtual>>().delta_secs_f64();
    let real1 = app.world().resource::<Time<Real>>().delta_secs_f64();
    println!("R1 暂停后 virtual delta = {v:.9}s，real delta（后帧）= {real1:.9}s（首帧 {real0:.9}）");
    assert!(v == 0.0, "记忆预期：暂停后 virtual delta 恰 0");
    assert!(real1 > 0.0, "记忆预期：real 时间不受影响");
}

// R2 记忆预期：scale=2.0 时 virtual delta ≈ 2× real delta。
fn r2_virtual_scale_doubles_delta() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.update();
    app.world_mut().resource_mut::<Time<Virtual>>().set_scale(2.0);
    app.update();
    let v = app.world().resource::<Time<Virtual>>().delta_secs_f64();
    let r = app.world().resource::<Time<Real>>().delta_secs_f64();
    let ratio = v / r;
    println!("R2 scale=2：virtual={v:.9} real={r:.9} 比值={ratio:.3}");
    assert!((ratio - 2.0).abs() < 0.3, "记忆预期：比值 ≈ 2");
}

// R3 记忆预期：DynamicScene 捕获 → 重新 spawn → update() 后实体（组件）复制。
fn r3_scene_roundtrip() {
    let mut src = World::new();
    for i in 0..5 {
        src.spawn(Ping(i as f32));
    }
    let scene = DynamicScene::from_world(&src);
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<bevy::ecs::reflect::AppTypeRegistry>();
    app.register_type::<Ping>();
    let mut world = app.world_mut();
    world.spawn(DynamicSceneRoot(scene));
    app.update(); // ScenePlugin 在跑？MinimalPlugins 不含 scene spawner——预期由断言揭示
    let n = world.query::<&Ping>().iter(&world).count();
    println!("R3 场景往返后 Ping 计数 = {n}");
    assert_eq!(n, 5, "记忆预期：恰复制 5 实体");
}

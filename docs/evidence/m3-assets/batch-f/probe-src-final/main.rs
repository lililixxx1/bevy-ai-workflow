//! M3 Block F 运行时探针——行为假设执行（r1 首试形态见 r1 快照；时序注记：
//! lib 失败阻断 bin 编译，bin 首试报错未及单独归档即进入查证，如实披露）。
use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
struct Ping(f32);

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    match which.as_str() {
        "r1" => r1_virtual_pause_freezes_delta(),
        "r2" => r2_virtual_relative_speed(),
        // R3 首试（DynamicScene::from_world + DynamicSceneRoot）随 0.19 场景重构
        // 整体失效——正解见 r3c（BSN 范式）。
        "r3c" => r3c_scene_spawn_bsn(),
        _ => panic!("usage: probe-f <r1|r2|r3c>"),
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

// R2c：set_relative_speed(2.0) 后 virtual delta ≈ 2× real delta。
fn r2_virtual_relative_speed() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.update();
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .set_relative_speed(2.0);
    app.update();
    let v = app.world().resource::<Time<Virtual>>().delta_secs_f64();
    let r = app.world().resource::<Time<Real>>().delta_secs_f64();
    let ratio = v / r;
    println!("R2 relative_speed=2：virtual={v:.9} real={r:.9} 比值={ratio:.3}");
    assert!((ratio - 2.0).abs() < 0.3, "预期：比值 ≈ 2");
}

// R3c：0.19 场景新范式——spawn_scene(bsn!{...}) 即时生成（官方 doctest 同款
// 插件组合：TaskPool + Asset + ScenePlugin）。
fn r3c_scene_spawn_bsn() {
    use bevy::scene::prelude::*;
    let mut app = App::new();
    app.add_plugins((
        bevy::app::TaskPoolPlugin::default(),
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin,
    ));
    app.register_type::<Ping>();
    let world = app.world_mut();
    let n0 = world.query::<&Ping>().iter(world).count();
    let spawned = world
        .spawn_scene(bsn! {
            Ping(1.0)
        })
        .expect("场景应可解析");
    let _ = spawned.id();
    let world = app.world_mut();
    let n1 = world.query::<&Ping>().iter(world).count();
    println!("R3c BSN spawn 前后 Ping 计数 {n0} -> {n1}");
    assert_eq!(n1 - n0, 1, "BSN 单实体场景即时生成恰 1 实体");
}

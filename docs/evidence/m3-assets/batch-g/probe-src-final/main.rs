//! M3 Block G 运行时探针 main——r2 修正形态（bin 首试错误见
//! probe-r1-bin-only-check.log：PrimaryWindow 不在 prelude、single() 返回
//! Result、Color::RED/GREEN 不存在、LinearRgba 无 red() 方法）。
//! 运行时断言（每条跑真实 app 若干帧）：
//! R1 主窗口：title 字段直读（0.19 仍是 pub 字段）+ scale_factor() > 0；
//! R2 相机 looking_at 后 GlobalTransform 朝向目标（dot > 0.99）；
//! R3 相机默认 Msaa 组件 = Sample4（per-Camera 组件，非全局资源）；
//! R4 Gizmos line/circle_2d 连续 6 帧无 panic；
//! R5 Color::srgb(0.5,..) → to_linear() 分量 = sRGB 分段传递函数值
//!    0.21404114（±1e-6，非纯 2.2 伽马）。
#![allow(dead_code)]

use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "probe-g".into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(FramesLeft::new(6))
    // r3 运行时失手：Startup 内 spawn_camera 的 commands 是延迟的，同 schedule 的
// check_samples_once 查不到相机（NoEntities 二连，probe-run-r2/r3.log）——
// 这正是 PAT-B-003（Startup 链式定序 / auto_insert_apply_deferred）的坑面，
// 探针自身踩中如实记；改挂 Update 首帧查询。
.add_systems(Startup, (spawn_camera, read_window))
    .add_systems(
        Update,
        (check_samples_once, check_camera_dir, draw_gizmos, check_color_math, exit_after),
    );
    app.run();
}

#[derive(Resource)]
struct FramesLeft(u32);

impl FramesLeft {
    fn new(n: u32) -> Self {
        Self(n)
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// R1：窗口读取（r1 通过部分：title 是公有字段；scale_factor() :610）。
fn read_window(windows: Query<&Window, With<PrimaryWindow>>) {
    let w = windows.single().expect("唯一主窗口");
    let sf = w.scale_factor();
    println!("[R1] title={:?} scale_factor={}", w.title, sf);
    assert!(!w.title.is_empty(), "title 不为空");
    assert!(sf > 0.0, "scale_factor > 0");
}

// R2：相机朝向断言——looking_at(ZERO) 后 forward 应指向原点方向。
fn check_camera_dir(query: Query<&GlobalTransform, With<Camera3d>>) {
    let gt = query.single().expect("唯一相机");
    let fwd = gt.forward();
    let to_target = (Vec3::ZERO - gt.translation()).normalize();
    let dot = fwd.dot(to_target);
    println!("[R2] forward={fwd:?} dot={dot}");
    assert!(dot > 0.99, "forward 指向目标（dot={dot}）");
}

// R3（两轮运行时失手后的事实坐实，probe-run-r2/r3/r4.log 链）：
// ① r2/r3 的 NoEntities 并非「无 Msaa 组件」，而是 Startup 内 spawn_camera
//   延迟 commands 未应用（PAT-B-003 坑面）；
// ② Update 首帧实测相机带自动插入的默认组件 Msaa(Sample4)——
//   即 Msaa 是相机默认组件，缺位语义不存在于已 spawn 相机。
fn check_samples_once(query: Query<&Msaa, With<Camera3d>>) {
    let msaa = query.single().expect("唯一相机");
    let samples = msaa.samples();
    println!("[R3] camera Msaa={msaa:?} samples={samples}");
    assert_eq!(samples, 4, "默认相机自动插入 Msaa(Sample4)");
}

// R4：gizmos 绘制（r1 通过：line/circle_2d 签名未变；颜色改 css 常量 + Color::from）。
fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0), Color::from(css::RED));
    gizmos.circle_2d(Vec2::ZERO, 5.0, Color::from(css::GREEN));
}

// R5：srgb→linear 数值口径——sRGB 官方分段传递函数（0.5 → 0.21404114），
// 非纯 2.2 幂（0.21763764）。
fn check_color_math() {
    let c = Color::srgb(0.5, 0.5, 0.5).to_linear();
    let red = c.red;
    println!("[R5] linear red={red} (sRGB 传递函数期望 0.21404114；纯 2.2 幂为 0.21763764)");
    assert!(
        (red - 0.21404114).abs() < 1e-6,
        "srgb 0.5 → sRGB 分段传递函数（got {red}）"
    );
    assert!(red != 0.5_f32.powf(2.2), "不等于纯 2.2 幂");
}

fn exit_after(mut frames: ResMut<FramesLeft>) {
    frames.0 -= 1;
    if frames.0 == 0 {
        println!("[R6] frames done, exiting");
        std::process::exit(0);
    }
}

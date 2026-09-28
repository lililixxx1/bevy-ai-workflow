//! M3 Block G 运行时探针 main——r1 同为「凭记忆首试」形态。
//! 运行时假设（每条跑真实 app 若干帧后断言）：
//! R1 主窗口可读：title 字段直读、scale_factor() > 0；
//! R2 相机 looking_at 后 GlobalTransform 朝向目标（数学验证）；
//! R3 窗口 samples 默认 1；
//! R4 Gizmos 可在 Update 中绘制线段（无 panic）；
//! R5 Color::srgb(0.5,0.5,0.5) 转 linear 后分量 ≈ 0.5^(2.2)。
//! lib r1 失败会阻断本 bin 编译——按 F 批教训，r1 先 --lib 后全量分开归档。
#![allow(dead_code)]

use bevy::prelude::*;
use probe_g::*;

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
    .add_systems(Startup, read_window)
    .add_systems(Update, (check_camera_dir, draw_gizmos, check_color_math, exit_after))
    .add_systems(Startup, spawn_camera);
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

// R1：窗口读取（记忆形态：title 是公有字段直读）。
fn read_window(windows: Query<&Window, With<PrimaryWindow>>) {
    let w = windows.single();
    let sf = w.scale_factor();
    println!("[R1] title={:?} scale_factor={}", w.title, sf);
    assert!(!w.title.is_empty(), "title 不为空");
    assert!(sf > 0.0, "scale_factor > 0");
}

// R2：相机朝向断言——looking_at(ZERO) 后 forward 应指向原点方向。
fn check_camera_dir(query: Query<&GlobalTransform, With<Camera3d>>) {
    let gt = query.single();
    let fwd = gt.forward();
    let to_target = (Vec3::ZERO - gt.translation()).normalize();
    let dot = fwd.dot(to_target);
    println!("[R2] forward={fwd:?} dot={dot}");
    assert!(dot > 0.99, "forward 指向目标（dot={dot}）");
}

// R3：samples 直读（记忆形态：Window::samples 公有字段）。
fn check_samples_once(windows: Query<&Window, With<PrimaryWindow>>) {
    let w = windows.single();
    println!("[R3] samples={}", w.samples);
}

// R4：gizmos 绘制。
fn draw_gizmos(mut gizmos: Gizmos) {
    gizmos.line(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0), Color::RED);
    gizmos.circle_2d(Vec2::ZERO, 5.0, Color::GREEN);
}

// R5：srgb→linear 数值口径（记忆：Color::to_linear() 方法存在，2.2 伽马近似）。
fn check_color_math() {
    let c = Color::srgb(0.5, 0.5, 0.5).to_linear();
    let expect = 0.5_f32.powf(2.2);
    println!("[R5] linear red={} expect(2.2)={}", c.red(), expect);
    assert!((c.red() - expect).abs() < 0.01, "srgb 0.5 → linear ≈ 2.2 伽马（{}）", c.red());
}

fn exit_after(mut frames: ResMut<FramesLeft>) {
    frames.0 -= 1;
    if frames.0 == 0 {
        println!("[R6] frames done, exiting");
        std::process::exit(0);
    }
}

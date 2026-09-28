//! M3 Block G 探针（渲染 / 窗口 / UI / 数学域）——「凭记忆首试」载体。
//! r1：全部按 0.14-0.16 期训练语料的记忆形态书写，未查任何 0.19 源码。
//!
//! 【快照勘误留痕】本文件原于 r1 时点归档后，2026-09-28 被 batch2 追加版
//! 误覆盖（cp 目标笔误）；现按 r1 原文恢复（内容与 probe-r1-lib-check.log
//! 一致；该日志共 30 错，其行号与快照行号差恰为本勘误头自身新增的 5 行，
//! 即快照行号 = 日志行号 + 5）。误覆盖与恢复操作
//! 记入 docs/evidence/m3-assets/batch-g/snapshot-restore-note.txt。
#![allow(dead_code)]

use bevy::prelude::*;

// ============ 渲染域 ============

// G1【记忆 0.14-0.15】相机整包 spawn：Camera3dBundle。
pub fn g1_camera_bundle(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

// G2【记忆 0.14】网格/材质组件 = Handle<Mesh> / Handle<StandardMaterial> 直挂。
pub fn g2_handle_components(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mat = materials.add(StandardMaterial::default());
    commands.spawn((mesh, mat, Transform::default()));
}

// G3【记忆 0.15】StandardMaterial 字段：emissive 是 Color、可乘强度。
pub fn g3_material_fields(materials: ResMut<Assets<StandardMaterial>>) {
    let _handle = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        emissive: Color::srgb(0.0, 1.0, 0.0) * 2.0,
        metallic: 0.5,
        perceptual_roughness: 0.8,
        ..default()
    });
}

// G4【记忆 0.15】多重采样 = 插 Msaa 资源。
pub fn g4_msaa_resource(app: &mut App) {
    app.insert_resource(Msaa::Sample4);
}

// G5【记忆 0.15】Camera { hdr: true } 开 HDR。
pub fn g5_camera_hdr(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Transform::default(),
    ));
}

// G6【记忆 0.15】泛光组件叫 Bloom（0.14 是 BloomSettings）。
pub fn g6_bloom(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Bloom {
            intensity: 0.5,
            ..default()
        },
        Transform::default(),
    ));
}

// G7【记忆 0.15】环境光 = AmbientLight 资源，brightness 数值 lux。
pub fn g7_ambient_light(app: &mut App) {
    app.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        ..default()
    });
}

// G8【记忆 0.15】距离雾 = DistanceFog + FogFalloff::Linear。
pub fn g8_distance_fog(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        DistanceFog {
            color: Color::GRAY,
            falloff: FogFalloff::Linear {
                start: 10.0,
                end: 50.0,
            },
            ..default()
        },
        Transform::default(),
    ));
}

// G9【记忆 0.14】可见性整包：VisibilityBundle。
pub fn g9_visibility_bundle(mut commands: Commands, meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    commands.spawn((Mesh3d(mesh), VisibilityBundle::default()));
}

// G10【记忆 0.15】清屏色 = ClearColor 资源。
pub fn g10_clear_color(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb_u8(30, 30, 40)));
}

// G22【记忆 0.15】点光源阴影 = PointLight { shadows_enabled }。
pub fn g22_point_light(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default(),
    ));
}

// G23【记忆 0.15】平行光阴影 = DirectionalLight { shadows_enabled }。
pub fn g23_directional_shadows(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::default(),
    ));
}

// G25【记忆 0.15】线框 = Wireframe { color }。
pub fn g25_wireframe(mut commands: Commands) {
    commands.spawn((Wireframe {
        color: Color::RED,
    },));
}

// G26【记忆 0.15】曝光预设 = Exposure::indoor()。
pub fn g26_exposure(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Exposure::indoor(), Transform::default()));
}

// G27【记忆 0.15】天空盒 = Skybox 组件（image handle 三件）。
pub fn g27_skybox(mut commands: Commands, assets: ResMut<Assets<Image>>) {
    let image = assets.add(Image::default());
    commands.spawn((
        Camera3d::default(),
        Skybox {
            image: image.clone(),
            brightness: 1000.0,
        },
        Transform::default(),
    ));
}

// ============ 2D/Sprite ============

// G11【记忆 0.14】精灵整包：SpriteBundle。
pub fn g11_sprite_bundle(mut commands: Commands) {
    commands.spawn(SpriteBundle::default());
}

// ============ UI 域 ============

// G12【记忆 0.14】文本整包：TextBundle::from_section + TextStyle。
pub fn g12_text_bundle(mut commands: Commands) {
    commands.spawn(TextBundle::from_section(
        "hello",
        TextStyle {
            font_size: 30.0,
            color: Color::WHITE,
            ..default()
        },
    ));
}

// G13【记忆 0.14】布局整包：NodeBundle + Style。
pub fn g13_node_bundle(mut commands: Commands) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(40.0),
            ..default()
        },
        ..default()
    });
}

// G14【记忆 0.15】按钮交互态 = Interaction 组件查询。
pub fn g14_interaction(query: Query<&Interaction, With<Button>>) -> usize {
    query.iter().filter(|i| **i == Interaction::Pressed).count()
}

// G15【记忆 0.15】UI 图像组件 = UiImage。
pub fn g15_ui_image(mut commands: Commands) {
    commands.spawn((Node::default(), UiImage::default()));
}

// G24【记忆 0.15】新版文本组件形态：Text::new + TextFont + TextColor。
pub fn g24_text_components(mut commands: Commands) {
    commands.spawn((
        Text::new("hi"),
        TextFont::from_font_size(24.0),
        TextColor(Color::WHITE),
    ));
}

// ============ 窗口域 ============

// G16【记忆 0.14-0.15】Window 内 cursor 字段（Cursor { visible }）。
pub fn g16_window_cursor() -> Window {
    Window {
        title: "probe-g".into(),
        cursor: Cursor {
            visible: false,
            ..default()
        },
        ..default()
    }
}

// G17【记忆 0.15】无边框全屏 = BorderlessFullscreen(MonitorSelection::Current)。
pub fn g17_borderless() -> WindowMode {
    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
}

// ============ 相机/数学域 ============

// G18【记忆 0.15】相机开关 = Camera { is_active }。
pub fn g18_camera_active(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            is_active: false,
            ..default()
        },
        Transform::default(),
    ));
}

// G19【记忆 0.15】渲染目标 = RenderTarget::Window(WindowRef::Primary)。
pub fn g19_render_target() -> Camera {
    Camera {
        target: RenderTarget::Window(WindowRef::Primary),
        ..default()
    }
}

// G20【记忆 0.15】Dir3::new(v) 返回 Dir3（非法向量 panic 版）。
pub fn g20_dir3(v: Vec3) -> Dir3 {
    Dir3::new(v)
}

// G21【记忆 0.15】欧拉角 = Quat::from_euler(EulerRot::XYZ, ...)。
pub fn g21_euler() -> Quat {
    Quat::from_euler(EulerRot::XYZ, 0.1, 0.2, 0.3)
}

// G28【记忆 0.15】2D 相机 + Camera { order }。
pub fn g28_camera_order(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 1,
            ..default()
        },
        Transform::default(),
    ));
}

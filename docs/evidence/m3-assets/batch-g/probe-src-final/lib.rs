//! M3 Block G 探针（渲染 / 窗口 / UI / 数学域）——「凭记忆首试」载体。
//!
//! r1 轮结论（2026-09-28，REAL_EXIT=101，lib 30 错 + bin 9 错；失败原文归档
//! docs/evidence/m3-assets/batch-g/probe-r1-lib-check.log 与 probe-r1-bin-only-check.log
//! ——后者系临时置空 lib 以单独暴露 bin 首试错误，操作不落日志注记、由 bin-only
//! 日志形态可证：unused import `probe_g::*` 警告 + 9 错且无任何 lib 错误；r1 源码快照同目录）：
//! - 失败（lib）：Camera3dBundle 不存在（G1）、Handle<Mesh> 不再是 Bundle（G2）、
//!   Color 不支持 *f32 且 emissive 是 LinearRgba（G3）、Msaa 不是 Resource（G4）、
//!   Camera 无 hdr 字段（G5/G6）、Bloom 不在 prelude（G6）、AmbientLight 不是
//!   Resource（G7）、Color::GRAY 不存在（G8）、VisibilityBundle 不存在（G9）、
//!   SpriteBundle 不存在（G11）、TextBundle/TextStyle 不存在（G12）、NodeBundle/Style
//!   不存在（G13）、UiImage 不存在（G15）、Window 无 cursor 字段且 Cursor 类型
//!   不在 prelude（G16）、WindowMode 不在 prelude（G17）、Camera 无 target 字段且
//!   RenderTarget/WindowRef 不在 prelude（G19）、shadows_enabled 字段不存在
//!   （G22/G23）、Wireframe 不在 prelude（G25）、Exposure 不在 prelude 且 indoor()
//!   是常量非方法（G26）、Skybox 不在 prelude 且 image 是 Option（G27）。
//! - 失败（bin）：PrimaryWindow 不在 prelude、query.single() 返回 Result、
//!   Color::RED/GREEN 不存在、LinearRgba 无 red() 方法（字段直读）。
//! - 编译通过（记忆写对，无坑）：ClearColor 资源（G10）、Camera{is_active}（G18）、
//!   Dir3::new（G20）、Quat::from_euler/EulerRot（G21）、Interaction/Button 查询
//!   （G14）、Text::new+TextFont::from_font_size+TextColor（G24）、Camera{order}
//!   （G28）、Gizmos line/circle_2d、FogFalloff::Linear{start,end} 结构形态。
//! r2：按本地源码逐条修正（注释标注出处行号）。
#![allow(dead_code)]

use bevy::prelude::*;
use bevy::window::{CursorOptions, MonitorSelection, WindowMode};

// ============ 渲染域 ============

// G1【r2 修正】Camera3dBundle 不存在（0.15 起 bundle→required components）；
// 正解 = Camera3d::default() + Transform（与 game/src/camera.rs:57-61 同形态）。
pub fn g1c_camera_spawn(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// G2【r2 修正】Handle<Mesh>/Handle<StandardMaterial> 不是组件位；0.15 起为
// Mesh3d/MeshMaterial3d 包装组件（game/src/sim.rs:214-215 同形态）。
pub fn g2c_mesh3d_components(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mat = materials.add(StandardMaterial::default());
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::default(),
    ));
}

// G3【r2 修正】emissive 是 LinearRgba 非 Color（bevy_pbr-0.19.1/src/pbr_material.rs:92），
// 强度直接用高数值（cd/m²），不再 Color*f32（examples/3d/bloom_3d.rs:39）。
pub fn g3c_material_fields(mut materials: ResMut<Assets<StandardMaterial>>) {
    let _handle = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        emissive: LinearRgba::rgb(0.0, 1.0, 0.0) * 2.0,
        metallic: 0.5,
        perceptual_roughness: 0.8,
        ..default()
    });
}

// G4【r2 修正】Msaa 是 per-Camera 组件非全局资源（bevy_render-0.19.1/src/view/mod.rs
// 「Component for configuring the number of samples for a Camera」，enum Off/Sample2/4/8）。
pub fn g4c_msaa_component(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Msaa::Sample8, Transform::default()));
}

// G5【r2 修正】Camera 无 hdr 字段；HDR = 独立 `Hdr` 单元组件
// （bevy_camera-0.19.1/src/components.rs:89）。
pub fn g5c_hdr_component(mut commands: Commands) {
    use bevy::camera::Hdr; // 不在 prelude
    commands.spawn((Camera3d::default(), Hdr, Transform::default()));
}

// G6【r2 修正】Bloom 在 bevy::post_process::bloom（不在 prelude），自带
// #[require(Hdr)]（bevy_post_process-0.19.1/src/bloom/settings.rs:32-33）；
// 常用预设 Bloom::NATURAL（examples/3d/bloom_3d.rs:35）。
pub fn g6c_bloom(mut commands: Commands) {
    use bevy::post_process::bloom::Bloom;
    commands.spawn((
        Camera3d::default(),
        Bloom::NATURAL,
        Transform::default(),
    ));
}

// G7【r2 修正】AmbientLight 是挂在相机上的组件（#[require(Camera)]，
// bevy_light-0.19.1/src/ambient_light.rs:17-19，覆盖 GlobalAmbientLight）；
// 全局环境光资源 = GlobalAmbientLight（examples/3d/skybox.rs:91-92 用 insert_resource）。
pub fn g7c_ambient_light(mut commands: Commands, app: &mut App) {
    // 每相机覆盖：
    commands.spawn((
        Camera3d::default(),
        AmbientLight {
            brightness: 500.0,
            ..default()
        },
        Transform::default(),
    ));
    // 全局默认：
    app.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 80.0,
        ..default() // r2 修正轮实测：还有 affects_lightmapped_meshes 字段（ambient_light.rs:87）
    });
}

// G8【r2 修正】Color::GRAY 不存在——Color 常量仅 WHITE/BLACK/NONE
// （bevy_color-0.19.1/src/color.rs:503-510）；具名色在 palettes::css（Srgba），
// 经 Color::from(css::GRAY) 或 Srgba::gray(0.5) 构造。FogFalloff::Linear 形态未变。
pub fn g8c_distance_fog(mut commands: Commands) {
    use bevy::color::palettes::css;
    commands.spawn((
        Camera3d::default(),
        DistanceFog {
            color: Color::from(css::GRAY),
            falloff: FogFalloff::Linear {
                start: 10.0,
                end: 50.0,
            },
            ..default()
        },
        Transform::default(),
    ));
}

// G9【r2 修正】VisibilityBundle 不存在；Mesh3d 已 require Visibility/
// VisibilityClass（bevy_camera visibility/mod.rs:500-501），无需手挂。
pub fn g9c_visibility(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    commands.spawn((Mesh3d(mesh),));
}

// G10（r1 通过）ClearColor 仍是 Resource（bevy_camera clear_color.rs:55）。
pub fn g10_clear_color(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb_u8(30, 30, 40)));
}

// G22【r2 修正】shadows_enabled → shadow_maps_enabled
// （bevy_light point_light.rs:70）。
pub fn g22c_point_light(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default(),
    ));
}

// G23【r2 修正】同上（bevy_light directional_light.rs:93，默认 false :152）。
pub fn g23c_directional_shadows(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default(),
    ));
}

// G25【r2 修正】Wireframe 是单元组件（无 color 字段），颜色经 WireframeColor
// { color }；路径 bevy::pbr::wireframe（examples/3d/wireframe.rs:13-16；
// wireframe.rs:845-846）。
pub fn g25c_wireframe(mut commands: Commands) {
    use bevy::color::palettes::css;
    use bevy::pbr::wireframe::{Wireframe, WireframeColor};
    commands.spawn((Wireframe, WireframeColor { color: Color::from(css::RED) }));
}

// G26【r2 修正】Exposure 在 bevy::camera（camera.rs:232）；预设是关联常量
// 非方法：SUNLIGHT/OVERCAST/INDOOR/BLENDER（camera.rs:236-247）。
pub fn g26c_exposure(mut commands: Commands) {
    use bevy::camera::Exposure;
    commands.spawn((Camera3d::default(), Exposure::INDOOR, Transform::default()));
}

// G27【r2 修正】Skybox 在 bevy::light（examples/3d/skybox.rs:9）；image 字段是
// Option<Handle<Image>>（:81-84 Some(...) 形态）。
pub fn g27c_skybox(mut commands: Commands, mut assets: ResMut<Assets<Image>>) {
    use bevy::light::Skybox;
    let image = assets.add(Image::default());
    commands.spawn((
        Camera3d::default(),
        Skybox {
            image: Some(image),
            brightness: 1000.0,
            rotation: Quat::IDENTITY, // r2 修正轮实测：还有 rotation 字段（probe.rs:245）
        },
        Transform::default(),
    ));
}

// ============ 2D/Sprite ============

// G11【r2 修正】SpriteBundle 不存在；Sprite 是自带 required components 的组件
// （bevy_sprite sprite.rs:19），裸 spawn 即可。
pub fn g11c_sprite(mut commands: Commands) {
    commands.spawn(Sprite::default());
}

// ============ UI 域 ============

// G12【r2 修正】TextBundle/TextStyle 均不存在；= Text::new + TextFont + TextColor
// （G24 记忆形态即正解；examples/ui/text/text.rs:35-43）。
pub fn g12c_text(mut commands: Commands) {
    commands.spawn((
        Text::new("hello"),
        TextFont::from_font_size(30.0),
        TextColor(Color::WHITE),
    ));
}

// G13【r2 修正】NodeBundle 与 Style 均不存在——**Style 结构体整体并入 Node**
// （0.19 UI 重构：Node 直接持 width/margin/flex_wrap/border 等样式字段，
// examples/ui/styling/borders.rs:95-99），单位用 px()/percent() 函数
// （bevy_ui geometry.rs:541/:558）。
pub fn g13c_node(mut commands: Commands) {
    commands.spawn(Node {
        width: percent(100.0),
        height: px(40.0),
        ..default()
    });
}

// G14（r1 通过）Interaction/Button 查询（bevy_ui lib.rs:71 prelude 导出 Interaction）。
pub fn g14_interaction(query: Query<&Interaction, With<Button>>) -> usize {
    query.iter().filter(|i| **i == Interaction::Pressed).count()
}

// G15【r2 修正】UiImage → ImageNode（bevy_ui lib.rs:70 widget::{Button, ImageNode,...}）。
pub fn g15c_image_node(mut commands: Commands) {
    commands.spawn((Node::default(), ImageNode::default()));
}

// G24（r1 通过）Text::new + TextFont::from_font_size + TextColor。
pub fn g24_text_components(mut commands: Commands) {
    commands.spawn((
        Text::new("hi"),
        TextFont::from_font_size(24.0),
        TextColor(Color::WHITE),
    ));
}

// ============ 窗口域 ============

// G16【r2 修正】Window 无 cursor 字段；光标选项 = 独立 CursorOptions 组件
// （bevy_window window.rs:752），挂窗口实体。
pub fn g16c_cursor_options(mut commands: Commands) {
    commands.spawn((Window::default(), CursorOptions { visible: false, ..default() }));
}

// G17【r2 修正】WindowMode/MonitorSelection 不在 prelude——完整路径
// bevy::window::（game/src/main.rs:22 同款 prelude 缺口）。
pub fn g17c_borderless() -> WindowMode {
    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
}

// ============ 相机/数学域 ============

// G18（r1 通过）Camera { is_active } 仍是字段（camera.rs:392 附近 pub is_active: bool）。
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

// G19【r2 修正】Camera 无 target 字段；渲染目标 = 独立 RenderTarget 组件
// （camera.rs:890-892 #[derive(Component)]，变体 Window/Image/TextureView/None{size}，
// Image 是 ImageRenderTarget 包装非裸 Handle）。WindowRef 同文件。
pub fn g19c_render_target(mut commands: Commands) {
    // RenderTarget 在 bevy::camera；WindowRef 定义在 bevy::window（bevy_camera 仅 use）
    use bevy::camera::RenderTarget;
    use bevy::window::WindowRef;
    commands.spawn((
        Camera3d::default(),
        RenderTarget::Window(WindowRef::Primary),
        Transform::default(),
    ));
}

// G20（r1 失败，30 错之一：显式返回位暴露 E0308）Dir3::new(v) 存在。
pub fn g20c_dir3(v: Vec3) -> Option<Dir3> {
    // r1 即实测：Dir3::new 返回 Result<Dir3, InvalidDirectionError>
    // （direction.rs:563；r1 日志 :304-317 E0308 原文）；unchecked 版 :572。
    // （入库时曾误记「r1 通过」，独立审核以归档日志纠正——显式返回位
    // 正是 r1 能抓住签名变化的原因。）
    Dir3::new(v).ok()
}

// G21（r1 通过）Quat::from_euler(EulerRot::XYZ, ...) 存在。
pub fn g21_euler() -> Quat {
    Quat::from_euler(EulerRot::XYZ, 0.1, 0.2, 0.3)
}

// G28（r1 通过）Camera { order } 仍是字段。
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

// ============ 批次二（g29-g40）：凭记忆首试，未查源码 ============

// G29【r2b 修正】BorderColor 是每边一色结构体 { top, right, bottom, left }
// （ui_node.rs:2256-2260），非单 Color 元组；From<Color> → 全边同色
// （impl From :2263-2266 BorderColor::all）。
pub fn g29c_border_color(mut commands: Commands) {
    commands.spawn((
        Node::default(),
        BorderColor {
            top: Color::WHITE,
            ..default()
        },
    ));
    // 等价捷径：
    // commands.spawn((Node::default(), BorderColor::from(Color::WHITE)));
}

// G30【记忆 0.15】背景色组件 = BackgroundColor(Color)。
pub fn g30_background_color(mut commands: Commands) {
    commands.spawn((Node::default(), BackgroundColor(Color::srgb(0.2, 0.2, 0.3))));
}

// G31【记忆 0.15】2D 世界文本 = Text2d 组件 + TextFont/TextColor。
pub fn g31_text2d(mut commands: Commands) {
    commands.spawn((
        Text2d::new("world text"),
        TextFont::from_font_size(18.0),
        TextColor(Color::WHITE),
    ));
}

// G32【r2b 修正】Tonemapping 在 bevy::core_pipeline::tonemapping（mod.rs:119），
// 不在 prelude。
pub fn g32c_tonemapping(mut commands: Commands) {
    use bevy::core_pipeline::tonemapping::Tonemapping;
    commands.spawn((Camera3d::default(), Tonemapping::TonyMcMapface, Transform::default()));
}

// G33【r2b 修正】DebandDither 也在 core_pipeline::tonemapping（mod.rs:383 enum）。
pub fn g33c_deband_dither(mut commands: Commands) {
    use bevy::core_pipeline::tonemapping::DebandDither;
    commands.spawn((Camera3d::default(), DebandDither::Enabled, Transform::default()));
}

// G34【记忆 0.16】SSAO = bevy::pbr::ScreenSpaceAmbientOcclusion。
pub fn g34_ssao(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        bevy::pbr::ScreenSpaceAmbientOcclusion::default(),
        Transform::default(),
    ));
}

// G35【记忆 0.15】HSL 构造 = Color::hsl(h, s, l)。
pub fn g35_hsl() -> Color {
    Color::hsl(0.0, 1.0, 0.5)
}

// G36【记忆 0.15】2D 旋转 = Rot2::radians(angle)。
pub fn g36_rot2() -> Rot2 {
    Rot2::radians(1.5)
}

// G37【记忆 0.15】Val 枚举变体 Px/Percent/Auto。
pub fn g37_val() -> (Val, Val, Val) {
    (Val::Px(10.0), Val::Percent(50.0), Val::Auto)
}

// G38【记忆 0.15】Sprite 字段 color/custom_size。
pub fn g38_sprite_fields(mut commands: Commands) {
    commands.spawn(Sprite {
        color: Color::WHITE,
        custom_size: Some(Vec2::new(10.0, 10.0)),
        ..default()
    });
}

// G39【记忆 0.16】指针点击 observer = On<Pointer<Click>>。
pub fn g39_pointer_click() -> impl FnMut(On<Pointer<Click>>) {
    |event: On<Pointer<Click>>| {
        let _ = event.entity;
    }
}

// G40【r2b 修正】FocusPolicy 不在 prelude；路径 bevy::ui::（focus.rs:108）。
pub fn g40c_focus_policy(mut commands: Commands) {
    use bevy::ui::FocusPolicy;
    commands.spawn((Node::default(), FocusPolicy::Block));
}

// ============ 批次三（g41-g44）：凭记忆首试，未查源码 ============

// G41【r2b 修正】Text::from_sections/TextSection 均不存在——多段文本改为
// **子实体**形态：根 Text + 子 (TextSpan::new(..), TextFont, TextColor)
// （bevy_text text.rs:193 pub struct TextSpan(pub String)；
// examples/ui/text/text.rs:69-76 .with_child((TextSpan::default(), (TextFont{..})))）。
pub fn g41c_text_span(mut commands: Commands) {
    commands
        .spawn(Text::new("root"))
        .with_child((TextSpan::new(" span A"), TextFont::from_font_size(12.0), TextColor(Color::WHITE)))
        .with_child((TextSpan::new(" span B"), TextFont::from_font_size(14.0), TextColor(Color::WHITE)));
}

// G42【r2b 修正】TextAlignment 不存在——对齐 = Justify 枚举经
// TextLayout::justify(Justify::Center)（examples/ui/text/text.rs:46）。
pub fn g42c_justify() -> TextLayout {
    TextLayout::justify(Justify::Center)
}

// G43【r2b 修正】Val::Undefined 已移除；现行变体 Auto/Px/Percent/Vw/Vh/VMin/VMax
// （geometry.rs:32-60，新增视口单位 Vw/Vh/VMin/VMax）。
pub fn g43c_val() -> Val {
    Val::Vw(50.0)
}

// G44【r2b 修正】RenderLayers 移至 bevy::camera::visibility
// （render_layers.rs:20，mod.rs:43 pub use；examples/2d/pixel_grid_snap.rs:4）；
// 旧路径 bevy::render::view::RenderLayers 导入失败（E0432 两轮实证）。
pub fn g44c_render_layers(mut commands: Commands) {
    use bevy::camera::visibility::RenderLayers;
    commands.spawn((Camera3d::default(), RenderLayers::layer(1), Transform::default()));
}

# PAT-B-013-pbr-swarm-spawn

### PAT-B-013：PBR 实体群 spawn——`Assets<T>` 系统参数 + 单 Mesh/Material 句柄复用 + `spawn_batch`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（bevy_render/bevy_pbr 资产与组件形态）
- 标签：`Assets` `Mesh3d` `MeshMaterial3d` `spawn_batch` `StandardMaterial` `DirectionalLight`
- 源码出处：`game/src/sim.rs:201-250`（网格/材质/光照/spawn_batch）

**场景**：一次生成上万同网格同材质实体（+ 地面 + 平行光），句柄只 add 一次、clone 复用；迭代器须 'static。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/sim.rs:208-244` 的逐字删减——保留关键注释）：

```rust
fn spawn_swarm(
    mut commands: Commands,
    config: Res<SimConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(0.6, 0.6, 0.6));
    let body_material = materials.add(Color::srgb_u8(124, 144, 255));
    let ground_material = materials.add(Color::srgb_u8(34, 36, 48));

    commands.spawn((
        Mesh3d(meshes.add(Circle::new(AREA * 1.2))),
        MeshMaterial3d(ground_material),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    commands.spawn((
        DirectionalLight { ..default() },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_3)),
    ));

    let iter = (0..config.entity_count).map(move |index| {
        // ……draw_initial 确定性初值（PAT-B-004）……
        (
            Mesh3d(mesh.clone()),
            MeshMaterial3d(body_material.clone()),
            Transform::from_translation(translation),
            Wanderer { index, origin, phase },
            Velocity { linear },
        )
    });
    commands.spawn_batch(iter);
}
```

要点：①`Assets<Mesh>`/`Assets<StandardMaterial>` 以 `mut ResMut` 系统参数获取（`Assets::add` 是 &mut self——PIT-B-032 旁注）；②句柄 clone 便宜（0.19 Handle 是 enum Strong(Arc)/Uuid——PIT-B-027）；③`materials.add(Color::…)` 利用 `impl From<Color> for StandardMaterial`；④`spawn_batch` 要求迭代器 'static——所需值全部 move 进闭包，不借用系统参数；⑤`DirectionalLight { ..default() }` 默认无阴影（0.19 字段名 `shadow_maps_enabled` 默认 false，PIT-B-043）——大批量基线不付阴影代价。

**为什么**：同网格同材质实体共享句柄是渲染实例化合批的前提；spawn_batch 的 'static 约束逼出「先拷贝后 move」的确定性数据流。

**验证证据**：
- 代码为 T003/T007 已验证任务代码的逐字删减，删减后未单独重新编译；
- 运行验证：TS-01 断言①500 实体 query 计数一致（docs/evidence/ts-01-brp.md，启动口径 --count 500）；TS-02 改 max_speed 后 origin 逐位一致；ts-11 50000 实体 60fps 基线。

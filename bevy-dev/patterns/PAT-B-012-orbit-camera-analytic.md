# PAT-B-012-orbit-camera-analytic

### PAT-B-012：轨道相机——`Camera3d` + `looking_at` spawn 与解析轨迹更新（模拟时间的函数）

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（bevy_camera/bevy_transform 相机与朝向 API）
- 标签：`Camera3d` `looking_at` `Transform` `轨道相机` `解析函数` `BRP断言`
- 源码出处：`game/src/camera.rs:51-56`（spawn）+ `:58-70`（轨道更新）+ `:72-86`（单测）

**场景**：演示场景需要一只持续漫游的观察相机，且其位置可被 BRP 端**精确预测**（跨调用断言距离恒等），暂停模拟时相机仍漫游（真实时钟）。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/camera.rs:51-70` 的逐字删减）：

```rust
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(90.0, 45.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn orbit_camera(rig: Res<CameraRig>, time: Res<Time>, mut query: Query<&mut Transform, With<Camera3d>>) {
    let t = time.elapsed_secs();
    let angle = rig.angular_speed * t + rig.phase;
    let pos = Vec3::new(
        rig.radius * angle.cos(),
        rig.height,
        rig.radius * angle.sin(),
    );
    for mut transform in &mut query {
        transform.translation = pos;
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
```

要点：①spawn 不用 bundle——`Camera3d` 标记组件 + `Transform`（PIT-B-031 正例面）；②位置是**解析函数** pos(t)（`Time::elapsed_secs` 真实时钟），无积分误差，BRP 端可逐位复算；③朝向每帧 `look_at` 重设（up 轴 Vec3::Y）；④查询面 `Query<&mut Transform, With<Camera3d>>` 以标记组件过滤。

**为什么**：观察者（相机）与被观察者（模拟实体）解耦时钟——模拟暂停时相机继续漫游，BRP 验证暂停语义仍可用相机视角取证。

**验证证据**：
- 代码为 T003（M1 首役）已验证任务代码的逐字删减，删减后未单独重新编译；
- 单测 `orbit_distance_is_constant`（camera.rs:72-86，断言 XZ 平面距离 `(pos - (0,pos.y,0)).length() == radius` 恒等）在 `cargo test -p game` 通过；
- 运行验证：TS-08 BRP 断言相机 Transform 到原点距离 == rig.radius（容差 0.1，两连调 90→60→90 状态机，docs/evidence/ts-08-brp.md）；探针 batch-g R2 复证 `looking_at` 后 forward·to_target = 1.0（probe-run-r5.log）。

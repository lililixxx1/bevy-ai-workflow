# PAT-B-014-brp-readable-writable-resource

### PAT-B-014：BRP 可读可写 Resource 的反射形态——四 derive + `#[reflect(Resource, Serialize, Deserialize)]`

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（bevy_reflect/bevy_remote 资源反射面）
- 标签：`Resource` `Reflect` `Serialize` `Deserialize` `BRP` `get_resources` `mutate_resources`
- 源码出处：`game/src/camera.rs:15-33`（CameraRig 定义与 Default）

**场景**：把运行时参数（如相机轨道参数）暴露给 BRP 工具端：`world.get_resources` 读、`world.mutate_resources` 原地改，供外部断言与在线调参。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/camera.rs:15-33` 的逐字删减）：

```rust
#[derive(Resource, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct CameraRig {
    pub radius: f32,
    pub height: f32,
    pub angular_speed: f32,
    pub phase: f32,
}
```

配套：插件内 `app.register_type::<CameraRig>()` + `init_resource`（camera.rs:42-43）。

要点：①类型派生 `Resource, Reflect, Serialize, Deserialize` 四件套——Serialize/Deserialize 供 BRP JSON 序列化，Reflect 供注册表；②属性**必须**显式列 `#[reflect(Resource, Serialize, Deserialize)]`——derive 与 reflect 属性是两面（对齐 PIT-B-024「注册表属性面」教训）；③`PartialEq` 便于单测与回读比对；④serde 默认值兼容旧数据用 `#[serde(default = "...")]`（PAT-B-004 ④）。

**为什么**：BRP 的资源读写走 TypeRegistry 的 ReflectResource 数据 + serde JSON——两条链都在 `#[reflect(...)]` 属性里声明，缺一只在对应方法上报错。

**验证证据**：
- 代码为 T003/T013/T020 已验证任务代码的逐字删减，删减后未单独重新编译；
- 运行验证：TS-08 用 `world.get_resources` 读 CameraRig、`world.mutate_resources` 改 radius 后 BRP 断言相机距离随之变化（docs/evidence/ts-08-brp.md）；run_tests 套件 ts-08 同口径进程内复测。

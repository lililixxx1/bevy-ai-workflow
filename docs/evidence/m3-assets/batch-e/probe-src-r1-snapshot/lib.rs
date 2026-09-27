//! M3 Block E 探针（反射 / BRP 深水区）——「凭记忆首试」载体。
//!
//! 仓库外临时 crate。时序纪律（docs/m3-plan.md §3.1）：本文件在**未查证
//! bevy 0.19.1 源码**的前提下按训练语料直觉书写（语料主体为 0.15/0.16
//! 时代 API 形态）；失败原文先归档（docs/evidence/m3-assets/batch-e/），
//! 之后才查源码修正。首试：2026-09-27。
//!
//! 先验摇摆点（诚实标注）：PartialReflect/Reflect 拆分后的方法归属、
//! FromReflect 存续、clone_value→reflect_clone 迁移、ReflectComponent 的
//! 构造方式、#[reflect_value] 改名、reflect_path 返回的 trait 对象类型。
//!
//! 已验证不再探（本仓证据）：register_type::<T>() 显式注册唯一通路 +
//! #[reflect(no_auto_register)]（SKILL §3.4，TS-09）；BRP get/trigger_event
//! 反射管线（SKILL §6.3/§6.7，PAT-B-007）；list_components/list_resources
//! 注册表判定源（SKILL §6.9）。
#![allow(dead_code)]

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

#[derive(Reflect, Debug, Default)]
struct Inner {
    x: f32,
}

#[derive(Reflect, Debug, Default)]
#[reflect(Default)]
struct MyStruct {
    hp: f32,
    name: String,
    inner: Inner,
}

// P1：reflect_path 读嵌套字段（先验摇摆：返回 &dyn Reflect 还是 &dyn PartialReflect）
fn p1_reflect_path(comp: &MyStruct) {
    let p = comp.reflect_path("inner.x").expect("路径应命中");
    let _ = format!("{p:?}");
}

// P2：FromReflect 从动态值重建类型化值（先验摇摆：trait 存续与否）
fn p2_from_reflect(v: &dyn Reflect) {
    let _t: Option<MyStruct> = <MyStruct as bevy::reflect::FromReflect>::from_reflect(v);
}

// P3：反射克隆旧形态 clone_value（先验摇摆：是否已被 reflect_clone 取代）
fn p3_clone_value(v: &dyn Reflect) -> Box<dyn Reflect> {
    v.clone_value()
}

// P4：组件的反射插入（先验摇摆：ReflectComponent 构造与 insert 签名）
fn p4_reflect_component(world: &mut World, entity: Entity, comp: &dyn Reflect) {
    use bevy::ecs::reflect::ReflectComponent;
    let rc = ReflectComponent::from_world(world);
    rc.insert(&mut world.entity_mut(entity), comp);
}

// P5：ReflectSerializer 序列化（BRP 响应值同源机制；先验摇摆：模块路径与签名）
fn p5_reflect_serializer(value: &dyn Reflect, registry: &TypeRegistry) -> String {
    use bevy::reflect::serde::ReflectSerializer;
    let ser = ReflectSerializer::new(value, registry);
    serde_json::to_string(&ser).unwrap()
}

// P6：不透明反射旧属性 #[reflect_value]（先验摇摆：0.17 起是否改名 #[reflect(opaque)]）
#[derive(Reflect)]
#[reflect_value]
struct OpaqueWrap(f32);

// P7：#[reflect(Default)]（预期无坑——0.15 时代即此形态）
#[derive(Reflect, Default)]
#[reflect(Default)]
struct WithReflectDefault {
    v: u32,
}

// P8：注册表按短名查（先验摇摆：方法名 get_with_short_name 是否存在）
fn p8_short_name(registry: &TypeRegistry) -> bool {
    registry.get_with_short_name("MyStruct").is_some()
}

// P9：TypeRegistry 作为 World 资源可取（先验摇摆：资源类型是 TypeRegistry 还是 Arc 包装）
fn p9_registry_resource(world: &World) {
    let _r: &TypeRegistry = world.resource::<TypeRegistry>();
}

// P10：as_any_mut 下转（预期无坑——古老稳定面）
fn p10_downcast(v: &mut dyn Reflect) {
    if let Some(t) = v.as_any_mut().downcast_mut::<MyStruct>() {
        t.hp = 1.0;
    }
}

// P11：apply 就地覆写（先验摇摆：apply 在 Reflect 还是 PartialReflect 上）
fn p11_apply(a: &mut dyn Reflect, b: &dyn Reflect) {
    a.apply(b);
}

// P12：reflect_clone 新形态（先验摇摆：存在于 dyn Reflect 上与否）
fn p12_reflect_clone(v: &dyn Reflect) {
    let _ = v.reflect_clone();
}

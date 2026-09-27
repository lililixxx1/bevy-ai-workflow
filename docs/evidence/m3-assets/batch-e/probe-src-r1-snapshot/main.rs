//! M3 Block E 运行时探针——四个「凭记忆预期」的行为假设；
//! 断言失败 = 假设被证伪（panic 输出即复现证据）。首试：2026-09-27（未查源码）。
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

#[derive(Reflect, Debug, Default)]
struct Inner {
    x: f32,
}

#[derive(Reflect, Debug, Default)]
struct MyStruct {
    hp: f32,
    name: String,
    inner: Inner,
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    match which.as_str() {
        "r1" => r1_registry_in_bare_world(),
        "r2" => r2_serializer_shape(),
        "r3" => r3_apply_deep_copy(),
        "r4" => r4_short_name_lookup(),
        _ => panic!("usage: probe-e <r1|r2|r3|r4>"),
    }
}

// R1 记忆预期：裸 World 上 register_type 即建立 TypeRegistry 资源并可再取。
fn r1_registry_in_bare_world() {
    let mut world = World::new();
    world.register_type::<MyStruct>();
    let n = world.resource::<TypeRegistry>().len();
    println!("R1 裸 World 注册后 registry len = {n}");
    assert!(n >= 1, "记忆预期：register_type 建立 registry 并收录");
}

// R2 记忆预期：ReflectSerializer 输出的 JSON 含字段名（BRP 响应值同源）。
fn r2_serializer_shape() {
    let mut app = App::new();
    app.register_type::<MyStruct>();
    let registry = app.world().resource::<TypeRegistry>().clone();
    let v = MyStruct { hp: 3.5, name: "abc".into(), inner: Inner { x: 0.25 } };
    use bevy::reflect::serde::ReflectSerializer;
    let ser = ReflectSerializer::new(&v, &registry);
    let s = serde_json::to_string(&ser).unwrap();
    println!("R2 序列化输出：{s}");
    assert!(s.contains("hp") && s.contains("name"), "记忆预期：字段名出现");
}

// R3 记忆预期：apply 是深拷贝语义（嵌套字段一并覆写）。
fn r3_apply_deep_copy() {
    let mut dest = MyStruct { hp: 0.0, name: String::new(), inner: Inner { x: 0.0 } };
    let src = MyStruct { hp: 9.0, name: "src".into(), inner: Inner { x: 1.5 } };
    let a: &mut dyn Reflect = &mut dest;
    let b: &dyn Reflect = &src;
    a.apply(b);
    println!("R3 apply 后 dest = {dest:?}");
    assert_eq!((dest.hp, dest.inner.x), (9.0, 1.5), "记忆预期：嵌套字段一并覆写");
}

// R4 记忆预期：App 侧 register_type 后短名可查。
fn r4_short_name_lookup() {
    let mut app = App::new();
    app.register_type::<MyStruct>();
    let reg = app.world().resource::<TypeRegistry>();
    let hit = reg.get_with_short_name("MyStruct").is_some();
    println!("R4 短名查 MyStruct 命中 = {hit}");
    assert!(hit, "记忆预期：注册后短名可查");
}

//! M3 Block E 运行时探针——四个「凭记忆预期」的行为假设；
//! 断言失败 = 假设被证伪（panic 输出即复现证据）。首试：2026-09-27（未查源码）。
use bevy::prelude::*;
use bevy::ecs::reflect::AppTypeRegistry;
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
        // r1/r2/r4 首试形态已注释留痕（r1b 失败归档），正解见 r1c/r2c/r4c。
        "r3" => r3_apply_deep_copy(),
        // ==== r2 轮运行时（2026-09-27，查证源码后追加）====
        "r1c" => r1c_registry_via_app_type_registry(),
        "r2c" => r2c_serializer_shape(),
        "r4c" => r4c_short_type_path_lookup(),
        _ => panic!("usage: probe-e <r1|r2|r3|r4|r1c|r2c|r4c>"),
    }
}

// R1【r1b 失败】E0599×2：裸 World 无 register_type；TypeRegistry 无 len 且非 Resource。
// 正解见 R1c。
// fn r1_registry_in_bare_world() {
//     let mut world = World::new();
//     world.register_type::<MyStruct>();
//     let n = world.resource::<TypeRegistry>().len();
//     println!("R1 裸 World 注册后 registry len = {n}");
//     assert!(n >= 1, "记忆预期：register_type 建立 registry 并收录");
// }

// R1c：正解——注册表资源是 AppTypeRegistry（RwLock 守卫访问）；
// 裸 World 手动建资源后 write().register::<T>()（TypeRegistry::register :201）。
fn r1c_registry_via_app_type_registry() {
    let mut world = World::new();
    world
        .get_resource_or_insert_with::<AppTypeRegistry>(Default::default)
        .write()
        .register::<MyStruct>();
    let n = world.resource::<AppTypeRegistry>().read().iter().count();
    // 修正（v2）：register::<T> 递归注册类型依赖闭包（type_registry.rs:201-207
    // T::register_type_dependencies；doc :166），故 n = 依赖闭包大小（实测 20：
    // MyStruct→Inner→f32/String→std primitives），不是 1。首版断言 ==1 被证伪，
    // 失败轮归档 probe-run-r1c.log。
    let hit = world
        .resource::<AppTypeRegistry>()
        .read()
        .contains(std::any::TypeId::of::<MyStruct>());
    println!("R1c 注册后 registry 收录 {n} 型（含依赖闭包），MyStruct 在册 = {hit}");
    assert!(hit, "目标型在册");
    assert!(n > 1, "依赖闭包一并注册（实测 20 型）");
}

// R2【r1b 失败】E0277：TypeRegistry 非 Resource（App 里是 AppTypeRegistry）。正解见 R2c。
// fn r2_serializer_shape() {
//     let mut app = App::new();
//     app.register_type::<MyStruct>();
//     let registry = app.world().resource::<TypeRegistry>().clone();
//     ...
// }

// R2c：正解——App::register_type 写入 AppTypeRegistry；序列化经 read() 守卫取 &TypeRegistry。
fn r2c_serializer_shape() {
    let mut app = App::new();
    app.register_type::<MyStruct>();
    let arc = app.world().resource::<AppTypeRegistry>().0.clone();
    let v = MyStruct { hp: 3.5, name: "abc".into(), inner: Inner { x: 0.25 } };
    use bevy::reflect::serde::ReflectSerializer;
    let reg = arc.read(); // 守卫须显式绑定：临时 &arc.read() 会 E0716 提前 drop
    let ser = ReflectSerializer::new(&v, &reg);
    let s = serde_json::to_string(&ser).unwrap();
    println!("R2c 序列化输出：{s}");
    assert!(s.contains("hp") && s.contains("name"), "字段名出现");
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

// R4【r1b 失败】E0599+E0277：get_with_short_name 不存在 + TypeRegistry 非 Resource。正解见 R4c。
// fn r4_short_name_lookup() { ... }

// R4c：正解——短名查 = get_with_short_type_path（type_registry.rs:467）。
fn r4c_short_type_path_lookup() {
    let mut app = App::new();
    app.register_type::<MyStruct>();
    let hit = app
        .world()
        .resource::<AppTypeRegistry>()
        .read()
        .get_with_short_type_path("MyStruct")
        .is_some();
    println!("R4c 短名查 MyStruct 命中 = {hit}");
    assert!(hit, "注册后短名可查");
}

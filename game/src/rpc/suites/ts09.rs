//! TS-09《新组件类型注册与 schema》的游戏进程内断言（两阶段判定第二阶段）。
//!
//! 与 `assets-methodology/taskset/ts-09-type-schema.md` 验收清单的对应：
//! - #1（`list_components` 含 `game::sim::Tagged`）→ [`tagged_reflect_visible`]：
//!   BRP `list_components` 的数据源是类型注册表中带 `ReflectComponent` 数据的
//!   类型（bevy_remote-0.19.1/src/builtin_methods.rs:1377-1400 同构判定），
//!   进程内直查同一注册表等价；
//! - #2（`registry.schema` 含字段 `tag`（string 类型））→
//!   [`tagged_schema_has_tag_field`]：经 `TypeInfo` 的 `StructInfo` 直查字段表
//!   与字段类型（与 BRP schema 输出同源自注册表）；
//! - #3（Tagged with Wanderer 行数 == 10）→ [`tagged_first_ten_count`]；
//! - #4（反证：注释 register_type 后 BRP 拿不到）→ 实现期反证，第一阶段已执行
//!   （证据 ts-09-brp.md「反证/恢复全过程」），第二阶段不重复（改注册代码
//!   属源码级干预，非运行时断言面）。
//!
//! 依据: `TypeRegistration::type_info() -> &'static TypeInfo`
//! （bevy_reflect-0.19.1/src/type_registry.rs:635）；`TypeInfo::Struct` 变体与
//! `StructInfo::field(name) -> Option<&NamedField>`（structs.rs:165）；
//! `NamedField::type_info() -> Option<&'static TypeInfo>`（fields.rs:58）；
//! `TypeInfo::is::<T>()`（type_info.rs:295）（核实 2026-09-27）。

use std::any::TypeId;

use bevy::ecs::reflect::{AppTypeRegistry, ReflectComponent};
use bevy::prelude::*;
use bevy::reflect::TypeInfo;

use super::{tc, TestCase};
use crate::sim::{Tagged, Wanderer};

/// TS-09 套件入口（约定只读世界）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    vec![
        tagged_reflect_visible(world),
        tagged_schema_has_tag_field(world),
        tagged_first_ten_count(world),
    ]
}

/// 清单 #1 等价物：Tagged 在类型注册表中且带 ReflectComponent 数据
/// （BRP list_components 的判定条件；`no_auto_register` 下显式 register_type
/// 是唯一注册通路——约束「不依赖自动链」的运行时证据）。
fn tagged_reflect_visible(world: &mut World) -> TestCase {
    let registry = world.resource::<AppTypeRegistry>();
    let visible = registry
        .read()
        .get(TypeId::of::<Tagged>())
        .is_some_and(|r| r.data::<ReflectComponent>().is_some());
    tc(
        "tagged_reflect_visible",
        visible,
        "Tagged 在 AppTypeRegistry 中且带 ReflectComponent 数据（BRP list_components 同源判定；no_auto_register 下显式注册唯一通路）".into(),
    )
}

/// 清单 #2 等价物：schema 含字段 `tag` 且类型为 String。
fn tagged_schema_has_tag_field(world: &mut World) -> TestCase {
    let registry = world.resource::<AppTypeRegistry>();
    let result = (|| {
        let reg = registry.read();
        let type_info = reg.get(TypeId::of::<Tagged>())?.type_info();
        let TypeInfo::Struct(s) = type_info else {
            return None;
        };
        let field = s.field("tag")?;
        let is_string = field
            .type_info()
            .is_some_and(|ti| ti.is::<String>());
        Some(is_string)
    })();
    tc(
        "tagged_schema_has_tag_field",
        result == Some(true),
        format!(
            "TypeInfo::Struct 字段表含 `tag` 且类型 String：{result:?}（与 BRP registry.schema 同源自注册表）"
        ),
    )
}

/// 清单 #3：Tagged + Wanderer 实体恰 10 个（index < 10 打标口径）。
fn tagged_first_ten_count(world: &mut World) -> TestCase {
    let mut q = world.query_filtered::<&Tagged, With<Wanderer>>();
    let count = q.iter(world).count();
    tc(
        "tagged_first_ten_count",
        count == 10,
        format!("Tagged(With Wanderer) 实体 {count} 个（清单要求 == 10）"),
    )
}

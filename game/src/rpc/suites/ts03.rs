//! TS-03《新增 BRP 可见 Resource》的游戏进程内断言（两阶段判定第二阶段）。
//!
//! 与 `assets-methodology/taskset/ts-03-new-resource.md` 验收清单的对应：
//! - #1（`list_resources` 含 `game::sim::SimMetadata`）→
//!   [`metadata_reflect_visible`]：BRP `list_resources` 的数据源是类型注册表中
//!   带 `ReflectResource` 数据的类型，进程内直查同一注册表等价；工具端
//!   `--expect-version` 附带真实 `world.list_resources` 调用双保险；
//! - #2（version/entity_count/seed 三字段）→ [`metadata_fields_consistent`]：
//!   entity_count/seed 与 SimConfig 自洽；version 字面 `"0.1.0"` 由工具端
//!   `--expect-version 0.1.0` 绑定（进程内只断言 == `env!("CARGO_PKG_VERSION")`）。
//!
//! 依据: `AppTypeRegistry` 为 Resource（bevy_ecs-0.19.1/src/reflect/mod.rs:36）；
//! `TypeRegistry::get(TypeId)`（bevy_reflect-0.19.1/src/type_registry.rs:423）；
//! `TypeRegistration::data::<T>()`（type_registry.rs:677）；`ReflectResource`
//! （bevy_ecs-0.19.1/src/reflect/resource.rs:31）（核实 2026-09-27）。

use std::any::TypeId;

use bevy::ecs::reflect::{AppTypeRegistry, ReflectResource};
use bevy::prelude::*;

use super::{tc, TestCase};
use crate::sim::{SimConfig, SimMetadata};

/// TS-03 套件入口（约定只读世界）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    vec![metadata_reflect_visible(world), metadata_fields_consistent(world)]
}

/// 清单 #1 等价物：SimMetadata 已注册进类型注册表且带 ReflectResource 数据
/// （BRP list_resources 的判定条件）。
fn metadata_reflect_visible(world: &mut World) -> TestCase {
    let registry = world.resource::<AppTypeRegistry>();
    let visible = registry
        .read()
        .get(TypeId::of::<SimMetadata>())
        .is_some_and(|r| r.data::<ReflectResource>().is_some());
    tc(
        "metadata_reflect_visible",
        visible,
        "SimMetadata 在 AppTypeRegistry 中且带 ReflectResource 数据（BRP list_resources 同源判定）".into(),
    )
}

/// 清单 #2：三字段值核对（version 对 CARGO_PKG_VERSION 自证；字面 0.1.0 工具端绑定）。
fn metadata_fields_consistent(world: &mut World) -> TestCase {
    let (Some(config), Some(meta)) = (
        world.get_resource::<SimConfig>(),
        world.get_resource::<SimMetadata>(),
    ) else {
        return tc("metadata_fields_consistent", false, "缺 SimConfig 或 SimMetadata 资源".into());
    };
    let pass = meta.version == env!("CARGO_PKG_VERSION")
        && meta.entity_count == config.entity_count
        && meta.seed == config.seed;
    tc(
        "metadata_fields_consistent",
        pass,
        format!(
            "version={:?}（== CARGO_PKG_VERSION {:?}，字面 0.1.0 由工具端 --expect-version 绑定）、entity_count={}（== config）、seed={}（== config）",
            meta.version,
            env!("CARGO_PKG_VERSION"),
            meta.entity_count,
            meta.seed
        ),
    )
}

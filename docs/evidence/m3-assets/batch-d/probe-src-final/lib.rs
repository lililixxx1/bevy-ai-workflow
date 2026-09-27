//! M3 Block D 探针（事件 / observer / State 域）——「凭记忆首试」载体。
//!
//! 仓库外临时 crate。时序纪律（docs/m3-plan.md §3.1）：本文件在**未查证
//! bevy 0.19.1 源码**的前提下按训练语料直觉书写；失败原文先归档
//! （docs/evidence/m3-assets/batch-d/probe-r1-check.log），之后才查源码修正。
//! 首试：2026-09-27。
//!
//! r1 轮结论（2026-09-27，REAL_EXIT=101）：
//! - 失败 3 函数 4 处：P4 World::write_event/read_event 不存在、P9
//!   Commands::write_event 不存在、P6 `On<Add<T>>` 泛型错配（E0107，Add 无泛型，
//!   bevy_ecs-0.19.1/src/lifecycle.rs:337 `pub struct Add`）。
//! - 编译通过（记忆写对，无坑）：P1 MessageReader::read、P2 App::add_message、
//!   P3 RemovedComponents<T> 系统参数、P5 On<E>::event()、P7 States derive +
//!   init_state + OnEnter(State) 调度、P8 State::get()/NextState::set。
//! 下方失败形态注释留痕（禁删），查证后以正解形态回归（r2 轮）。
//!
//! 已验证不再探（本仓证据）：`MessageWriter<AppExit>` + `.write()`（bench.rs:63/106，
//! T003）；`On<E>` observer + `App::add_observer` + `World::trigger` 同步（sim.rs，
//! SKILL §6.7，T015）；`run_if` 捕获闭包（批次 C）；`Query::single()/single_mut()`
//! Result 语义（PIT-B-005，本批直接按正解用）。
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Message, Clone)]
struct Damage {
    amount: f32,
}

#[derive(Component)]
struct Health(f32);

#[derive(Event, Clone)]
struct Exploded {
    power: f32,
}

// P1（r1 编译通过）
fn p1_message_reader(mut reader: MessageReader<Damage>, mut health: Query<&mut Health>) {
    for dmg in reader.read() {
        let mut h = health.single_mut().expect("唯一 Health");
        h.0 -= dmg.amount;
    }
}

// P2（r1 编译通过）
fn p2_add_message(app: &mut App) {
    app.add_message::<Damage>();
}

// P3（r1 编译通过）
fn p3_removed_components(mut removed: RemovedComponents<Health>) {
    for _e in removed.read() {}
}

// P4【r1 失败】E0599：World 无 write_event / read_event 方法（两处）。
// fn p4_world_event(world: &mut World) {
//     world.write_event(Exploded { power: 1.0 });
//     let _n = world.read_event::<Exploded>().count();
// }

// P5（r1 编译通过）
fn p5_observer_payload(ev: On<Exploded>) {
    let p = ev.event().power;
    println!("[obs] power={p}");
}

// P6【r1 失败】E0107：`Add` 无泛型参数（lifecycle.rs:337 `pub struct Add`），
// On<Add<Health>> 形态不存在。
// fn p6_on_add_observer(_: On<Add<Health>>) {}

// P7（r1 编译通过）
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
}

fn p7_state(app: &mut App, f: fn()) {
    app.init_state::<GameState>();
    app.add_systems(OnEnter(GameState::Menu), f);
}

// P8（r1 编译通过）
fn p8_state_res(state: Res<State<GameState>>, mut next: ResMut<NextState<GameState>>) {
    if *state.get() == GameState::Menu {
        next.set(GameState::Playing);
    }
}

// P9【r1 失败】E0599：Commands 无 write_event 方法。
// fn p9_commands_write_event(mut commands: Commands) {
//     commands.write_event(Exploded { power: 2.0 });
// }

// P10（r1 编译通过——app.add_observer 接受带载荷 On<E> 系统）
fn p10_register_payload_observer(app: &mut App) {
    app.add_observer(p5_observer_payload);
}

// ==== r1b 轮：追加首试（源码仍未查证，2026-09-27）====

// P11【r1b 失败】E0599：MessageMutator 无 iter_mut 方法（message_mutator.rs
// 仅 read :67 / read_with_id :72 / par_read :112）。正解见 P11c。
// fn p11_message_mutator(mut m: MessageMutator<Damage>) {
//     for d in m.iter_mut() {
//         d.amount += 1.0;
//     }
// }

// P12【r1b 失败】E0782「expected a type, found a trait」：裸 `Message` 在此被
// 解析为 trait（bevy_ecs::message::Message trait，与 derive 宏同名），
// `Message<Damage>` 非类型形态；且 Message 载荷不实现 Event、不可被
// trigger——缓冲消息没有 observer 形态（分流结论，见 P4c 注）。
// fn p12_on_message_observer(_: On<Message<Damage>>) {}

// P13【r1b 失败】E0599：App 无 register_event 方法（旧称 API 已整体移除）；
// 0.19 的注册入口是 `add_message::<T>()`（P2，r1 编译通过）。
// fn p13_register_event(app: &mut App) {
//     app.register_event::<Exploded>();
// }

// ==================== r2 正解形态（2026-09-27，查证后） ====================

/// P4c（→PIT-B-014）：Event = 纯 observer 触发——`World::trigger` / `Commands::trigger`
/// 同步执行观察者（event/mod.rs:8-10 文档）；无 write_event/read_event 缓冲 API
/// （event/mod.rs 中 write_event 仅测试代码 :583）；缓冲需求用 Message 系。
fn p4c_event_trigger(world: &mut World) {
    world.trigger(Exploded { power: 1.0 });
}

/// P9c（→PIT-B-014）：Commands 侧触发 = `Commands::trigger`（event/mod.rs:10 文档）。
fn p9c_commands_trigger(mut commands: Commands) {
    commands.trigger(Exploded { power: 2.0 });
}

/// P6c（→PIT-B-015）：生命周期组件过滤 = `On<Add, T>` 第二泛型（B: Bundle，
/// observer/system_param.rs:38 `pub struct On<'w,'t,E: Event,B: Bundle = ()>`；
/// 官方示例 examples/ecs/observers.rs:142 `On<Add, Mine>`）；`Add` 载荷 { entity }。
fn p6c_on_add_bundle_observer(_: On<Add, Health>) {}

/// P11c（→PIT-B-016）：MessageMutator 可变迭代 = `.read()`（message_mutator.rs:67，
/// 产 Mut<M> 可变项；另有 read_with_id :72 / par_read :112；iter_mut 不存在）。
fn p11c_mutator_read(mut m: MessageMutator<Damage>) {
    for d in m.read() {
        d.amount += 1.0;
    }
}

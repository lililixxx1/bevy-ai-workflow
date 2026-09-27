//! M3 Block D 探针（事件 / observer / State 域）——「凭记忆首试」载体。
//!
//! 仓库外临时 crate。时序纪律（docs/m3-plan.md §3.1）：本文件在**未查证
//! bevy 0.19.1 源码**的前提下按训练语料直觉书写；失败原文先归档
//! （docs/evidence/m3-assets/batch-d/），之后才查源码修正。首试：2026-09-27。
//!
//! 已验证不再探（本仓证据）：`MessageWriter<AppExit>` + `.write()`（bench.rs:63/106，
//! T003，官方示例 custom_loop.rs:36-38）；`On<E>` observer + `App::add_observer` +
//! `World::trigger` 同步执行（sim.rs，SKILL §6.7，T015）；`run_if` 捕获闭包（批次 C
//! 无坑确认）；`Query::single()/single_mut()` Result 语义（PIT-B-005，本批直接按正解用）。
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

// P1：MessageReader 的读取方法（先验摇摆：.read() vs .iter()——旧 EventReader 两代形态）
fn p1_message_reader(mut reader: MessageReader<Damage>, mut health: Query<&mut Health>) {
    for dmg in reader.read() {
        let mut h = health.single_mut().expect("唯一 Health");
        h.0 -= dmg.amount;
    }
}

// P2：消息注册（先验：存在 App::add_message，类比 init_resource）
fn p2_add_message(app: &mut App) {
    app.add_message::<Damage>();
}

// P3：RemovedComponents 系统参数（先验：仍是系统参数，方法 .read()）
fn p3_removed_components(mut removed: RemovedComponents<Health>) {
    for _e in removed.read() {}
}

// P4：缓冲事件（Event）的 World 直读写（先验：write_event / read_event::<T>() 存在）
fn p4_world_event(world: &mut World) {
    world.write_event(Exploded { power: 1.0 });
    let _n = world.read_event::<Exploded>().count();
}

// P5：observer 读事件载荷（先验：On<E> 有 .event() 方法）
fn p5_observer_payload(ev: On<Exploded>) {
    let p = ev.event().power;
    println!("[obs] power={p}");
}

// P6：实体生命周期 observer（先验：On<Add<T>> 形态可用）
fn p6_on_add_observer(_: On<Add<Health>>) {}

// P7：States derive + init_state + OnEnter 调度（先验：OnEnter(State::X) 是调度标签形态）
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

// P8：State 读取 + NextState 写入（先验：State::get() 返回引用、NextState::set）
fn p8_state_res(
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if *state.get() == GameState::Menu {
        next.set(GameState::Playing);
    }
}

// P9：Commands 侧写缓冲事件（先验摇摆：Commands::write_event 存在 vs 只有 World 侧）
fn p9_commands_write_event(mut commands: Commands) {
    commands.write_event(Exploded { power: 2.0 });
}

// P10：observe 组件实体形态以外——直接 app 级注册 payload observer（先验：add_observer
// 接受任意 On<E> 系统，含带载荷事件；与 P5 呼应验证注册面）
fn p10_register_payload_observer(app: &mut App) {
    app.add_observer(p5_observer_payload);
}

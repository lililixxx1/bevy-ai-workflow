//! M3 Block D 运行时探针——四个「凭记忆预期」的行为假设；
//! 断言失败 = 假设被证伪（panic 输出即复现证据）。首试：2026-09-27（未查源码）。
use bevy::prelude::*;
// StatesPlugin 不在 bevy::prelude（bevy_state prelude 未导出它）；完整路径
// bevy::state::app::StatesPlugin（bevy_internal lib.rs:93 `bevy_state as state`
// + bevy_state lib.rs:55 `pub mod app` + app.rs:330）。2026-09-27 源码核实。
use bevy::state::app::StatesPlugin;

#[derive(Message, Clone)]
struct Damage {
    amount: f32,
}

#[derive(Resource, Default)]
struct Counter(usize);

fn writer_once(mut wrote: Local<bool>, mut w: MessageWriter<Damage>) {
    if !*wrote {
        w.write(Damage { amount: 1.0 });
        *wrote = true;
    }
}

fn reader(mut c: ResMut<Counter>, mut r: MessageReader<Damage>) {
    c.0 += r.read().count();
}

fn on_enter_menu(mut c: ResMut<Counter>) {
    c.0 += 1;
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
}

fn on_enter_playing(mut c: ResMut<Counter>) {
    c.0 += 1;
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    match which.as_str() {
        "r1" => r1_unregistered_message(),
        "r2" => r2_message_lifetime(),
        "r3" => r3_initial_onenter(),
        "r4" => r4_next_state_transition(),
        // ==== r2 轮运行时（2026-09-27，查证源码后追加）====
        "r1c" => r1c_registered_message(),
        "r2b" => r2b_two_readers_staggered(),
        "r3c" => r3c_initial_onenter_with_plugin(),
        "r4c" => r4c_next_state_with_plugin(),
        _ => panic!("usage: probe-d <r1|r2|r3|r4|r1c|r2b|r3c|r4c>"),
    }
}

// R1 记忆预期（EventReader 时代印象）：消息队列未注册也能用——
// 直接给系统挂 MessageWriter 而不 add_message，预期正常运行。
fn r1_unregistered_message() {
    let mut world = World::new();
    let mut sched = Schedule::default();
    sched.add_systems(writer_only);
    sched.run(&mut world);
    println!("R1 未注册消息：未 panic（记忆预期成立）");
}

fn writer_only(mut w: MessageWriter<Damage>) {
    w.write(Damage { amount: 1.0 });
}

// R2 记忆预期（naive）：消息单帧生命周期——第 1 次运行读 1 条，第 2 次运行读 0 条。
fn r2_message_lifetime() {
    let mut world = World::new();
    world.init_resource::<Messages<Damage>>();
    world.init_resource::<Counter>();
    let mut sched = Schedule::default();
    sched.add_systems((writer_once, reader).chain());
    sched.run(&mut world);
    let first = world.resource::<Counter>().0;
    sched.run(&mut world);
    let second = world.resource::<Counter>().0 - first;
    println!("R2 消息生命周期：run1 读 {first} 条，run2 再读 {second} 条");
    assert_eq!((first, second), (1, 0), "记忆预期：单帧生命周期（1, 0）");
}

// R3 记忆预期（倾向）：初始状态初始化时 OnEnter(初始态) 触发一次。
fn r3_initial_onenter() {
    let mut app = App::new();
    app.init_resource::<Counter>();
    app.init_state::<GameState>();
    app.add_systems(OnEnter(GameState::Menu), on_enter_menu);
    app.update();
    let n = app.world().resource::<Counter>().0;
    println!("R3 初始 OnEnter(Menu) 触发次数 = {n}");
    assert_eq!(n, 1, "记忆预期：初始态进入触发一次");
}

// R4 记忆预期：NextState::set 后一次 update() 即完成转换（StateTransition 自动接线）。
fn r4_next_state_transition() {
    let mut app = App::new();
    app.init_resource::<Counter>();
    app.init_state::<GameState>();
    app.add_systems(OnEnter(GameState::Playing), on_enter_playing);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    let now = *app.world().resource::<State<GameState>>().get();
    let fired = app.world().resource::<Counter>().0;
    println!("R4 set 后一次 update：state={now:?}，OnEnter(Playing) 触发 {fired} 次");
    assert_eq!((now, fired), (GameState::Playing, 1), "记忆预期：自动转换并触发");
}

// ==================== r2 轮运行时（2026-09-27，查证源码后）====================

// R1c：R1 panic「Message not initialized」的根因是 Messages<T> 资源未注册
// （MessageWriter 的 param 校验）。正解 = 先注册：app.add_message::<T>() 或裸
// World 下 world.init_resource::<Messages<T>>()。此处验证裸 World 等价形态。
fn r1c_registered_message() {
    let mut world = World::new();
    world.init_resource::<Messages<Damage>>();
    world.init_resource::<Counter>();
    let mut sched = Schedule::default();
    sched.add_systems((writer_once, reader).chain());
    sched.run(&mut world);
    let n = world.resource::<Counter>().0;
    println!("R1c 注册后消息：单帧读到 {n} 条");
    assert_eq!(n, 1, "注册 Messages<Damage> 后 writer 可写、reader 可读");
}

// R2b：消息「双缓冲」语义复测——R2 的 (1, 0) 只证明同一读者游标已推进；
// 区分单/双缓冲要看**新读者错峰**：run1 写 1 条并由 reader_a 读走，手动
// Messages::update() 交换缓冲（模拟帧边界，messages.rs 双缓冲设计），
// run2 换一个新系统实例（新 MessageReader，游标 0）——双缓冲应仍读到 1 条。
fn r2b_two_readers_staggered() {
    let mut world = World::new();
    world.init_resource::<Messages<Damage>>();
    world.init_resource::<Counter>();
    let mut sched_a = Schedule::default();
    sched_a.add_systems((writer_once, reader).chain());
    sched_a.run(&mut world);
    let a = world.resource::<Counter>().0;
    world.resource_mut::<Messages<Damage>>().update();
    let mut sched_b = Schedule::default();
    sched_b.add_systems(reader_b); // 新系统实例 = 新 MessageReader（游标 0）
    sched_b.run(&mut world);
    let b = world.resource::<Counter>().0 - a;
    println!("R2b 双缓冲错峰：reader_a 第 1 帧读 {a} 条，reader_b 第 2 帧读 {b} 条");
    assert_eq!((a, b), (1, 1), "双缓冲：新读者隔帧仍能读上一帧消息（单缓冲应为 0）");
}

fn reader_b(mut c: ResMut<Counter>, mut r: MessageReader<Damage>) {
    c.0 += r.read().count();
}

// R3c：R3 panic「StateTransition schedule is missing」的根因 = init_state 前
// 未装 StatesPlugin（bevy_state/src/app.rs:102 检查 + panic）。装好后初始态
// 在 init_state 时写入 entered 转换（app.rs:107-112），首个 update() 触发
// OnEnter(初始态) 一次。
fn r3c_initial_onenter_with_plugin() {
    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.init_resource::<Counter>();
    app.init_state::<GameState>();
    app.add_systems(OnEnter(GameState::Menu), on_enter_menu);
    app.update();
    let n = app.world().resource::<Counter>().0;
    println!("R3c 装插件后初始 OnEnter(Menu) 触发次数 = {n}");
    assert_eq!(n, 1, "初始态在首个 update() 触发 OnEnter 一次");
}

// R4c：同 R4 但装 StatesPlugin——NextState::set 后一次 update() 完成转换并
// 触发 OnEnter(新态) 一次（StateTransition schedule 由插件接线）。
fn r4c_next_state_with_plugin() {
    let mut app = App::new();
    app.add_plugins(StatesPlugin);
    app.init_resource::<Counter>();
    app.init_state::<GameState>();
    app.add_systems(OnEnter(GameState::Playing), on_enter_playing);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    let now = *app.world().resource::<State<GameState>>().get();
    let fired = app.world().resource::<Counter>().0;
    println!("R4c set 后一次 update：state={now:?}，OnEnter(Playing) 触发 {fired} 次");
    assert_eq!((now, fired), (GameState::Playing, 1), "自动转换并触发一次");
}

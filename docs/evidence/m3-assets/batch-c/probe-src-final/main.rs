//! M3 Block C 运行时探针——「凭记忆预期」的行为假设（r1 轮，断言失败 = 假设被证伪，
//! panic 输出即错题复现证据）+ 查证后的修正预期（r2 轮）。
//! 用法：probe-c.exe <r1|r2|r3|r4|r1c|r2c|r3c|r4c>。首试时间：2026-09-27。
use bevy::prelude::*;

#[derive(Component)]
struct A(f32);

#[derive(Component)]
struct B(f32);

#[derive(Resource, Default)]
struct Seen(usize);

fn count_changed(mut seen: ResMut<Seen>, q: Query<&A, Changed<A>>) {
    seen.0 = q.iter().count();
}

fn writer_one(mut seen: ResMut<Seen>, mut q: Query<&mut A>) {
    for mut a in &mut q {
        a.0 += 1.0;
    }
    seen.0 += 1;
}

fn writer_two(mut seen: ResMut<Seen>, mut q: Query<&mut A>) {
    for mut a in &mut q {
        a.0 += 10.0;
    }
    seen.0 += 10;
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    match which.as_str() {
        "r1" => r1_iteration_order(),
        "r2" => r2_changed_first_frame(),
        "r3" => r3_conflict_ambiguity(),
        "r4" => r4_conflicting_params(),
        "r1c" => r1c_iteration_order_corrected(),
        "r2c" => r2c_changed_corrected(),
        "r3c" => r3c_chained_deterministic(),
        "r4c" => r4c_param_set(),
        _ => panic!("usage: probe-c <r1|r2|r3|r4|r1c|r2c|r3c|r4c>"),
    }
}

// ==================== r1 轮：记忆预期（已归档，勿改） ====================

// R4 记忆预期：同一 system 两个可变 Query 指向同组件 → 编译放行、运行初始化时
// panic（访问冲突）。
fn two_mut(mut a: Query<&mut A>, mut b: Query<&mut A>) {
    for mut x in &mut a {
        x.0 += 1.0;
    }
    for mut x in &mut b {
        x.0 += 2.0;
    }
}

fn r1_iteration_order() {
    let mut world = World::new();
    let e1 = world.spawn(A(1.0)).id();
    let _e2 = world.spawn(A(2.0)).id();
    let _e3 = world.spawn(A(3.0)).id();
    world.entity_mut(e1).insert(B(9.0));
    let mut q = world.query::<&A>();
    let order: Vec<f32> = q.iter(&world).map(|a| a.0).collect();
    println!("R1 实际迭代序 = {:?}", order);
    assert_eq!(order, vec![1.0, 2.0, 3.0], "记忆预期：迭代序 == spawn 序");
}

fn r2_changed_first_frame() {
    let mut world = World::new();
    world.init_resource::<Seen>();
    for i in 0..10 {
        world.spawn(A(i as f32));
    }
    let mut sched = Schedule::default();
    sched.add_systems(count_changed);
    sched.run(&mut world);
    println!("R2 首帧 Changed<A> 计数 = {}", world.resource::<Seen>().0);
    assert_eq!(
        world.resource::<Seen>().0,
        0,
        "记忆预期：首帧 Changed 计数 0"
    );
    sched.run(&mut world);
    println!(
        "R2 第二次运行 Changed<A> 计数 = {}",
        world.resource::<Seen>().0
    );
}

fn r3_conflict_ambiguity() {
    let mut world = World::new();
    world.init_resource::<Seen>();
    world.spawn(A(1.0));
    let mut sched = Schedule::default();
    sched.add_systems((writer_one, writer_two));
    sched.run(&mut world);
    let value = world.query::<&A>().iter(&world).next().unwrap().0;
    println!(
        "R3 两个冲突系统无序共存：未 panic，Seen={}，A={}",
        world.resource::<Seen>().0,
        value
    );
}

fn r4_conflicting_params() {
    let mut world = World::new();
    world.spawn(A(1.0));
    let mut sched = Schedule::default();
    sched.add_systems(two_mut);
    sched.run(&mut world);
    println!("R4 同系统双冲突 Query：未 panic（预期外）");
}

// ==================== r2 轮：查证后的修正预期 ====================

/// R1c：迭代序不保证（system/query.rs:654）；实体加组件会迁移 archetype，
/// 旧行以 swap_remove 退出（storage/table/mod.rs:226/:253）——本例实际序 [3,2,1]。
/// 修正做法：确定性处理按业务键排序/寻址，不依赖迭代序。
fn r1c_iteration_order_corrected() {
    let mut world = World::new();
    let e1 = world.spawn(A(1.0)).id();
    let _e2 = world.spawn(A(2.0)).id();
    let _e3 = world.spawn(A(3.0)).id();
    world.entity_mut(e1).insert(B(9.0));
    let mut q = world.query::<&A>();
    let raw: Vec<f32> = q.iter(&world).map(|a| a.0).collect();
    let mut sorted = raw.clone();
    sorted.sort_by(|x, y| x.partial_cmp(y).unwrap());
    println!("R1c 实际迭代序 = {:?}（不保证，已证实）→ 排序后 = {:?}", raw, sorted);
    assert_eq!(sorted, vec![1.0, 2.0, 3.0], "修正预期：按业务键排序后确定");
}

/// R2c：Changed<T> 含「Query 首次运行前」的变更（query/filter.rs:886-896 官方文档）；
/// 首帧全量命中，第二次运行起才只反映真实变更。
fn r2c_changed_corrected() {
    let mut world = World::new();
    world.init_resource::<Seen>();
    for i in 0..10 {
        world.spawn(A(i as f32));
    }
    let mut sched = Schedule::default();
    sched.add_systems(count_changed);
    sched.run(&mut world);
    let first = world.resource::<Seen>().0;
    sched.run(&mut world);
    let second = world.resource::<Seen>().0;
    println!("R2c 首帧 = {}（修正预期 10），第二次 = {}（修正预期 0）", first, second);
    assert_eq!(first, 10, "修正预期：首帧全量命中");
    assert_eq!(second, 0, "修正预期：无后续变更则为 0");
}

/// R3c：跨系统冲突默认不 panic、顺序无保证——确定性须 `.chain()` 显式定序。
fn r3c_chained_deterministic() {
    let mut world = World::new();
    world.init_resource::<Seen>();
    world.spawn(A(1.0));
    let mut sched = Schedule::default();
    sched.add_systems((writer_one, writer_two).chain());
    for _ in 0..5 {
        sched.run(&mut world);
    }
    let value = world.query::<&A>().iter(&world).next().unwrap().0;
    println!("R3c chain 定序：5 轮后 A={}（修正预期 56 = 1+5×(1+10)）", value);
    assert_eq!(value, 56.0, "修正预期：chain 下每轮确定 +1 再 +10（初始 1.0）");
}

/// R4c：同系统冲突 Query 的正解 = `ParamSet`（B0001 报错建议之一，state.rs:216）。
fn two_mut_set(mut set: ParamSet<(Query<&mut A>, Query<&mut A>)>) {
    for mut x in set.p0().iter_mut() {
        x.0 += 1.0;
    }
    for mut x in set.p1().iter_mut() {
        x.0 += 2.0;
    }
}

fn r4c_param_set() {
    let mut world = World::new();
    world.spawn(A(1.0));
    let mut sched = Schedule::default();
    sched.add_systems(two_mut_set);
    sched.run(&mut world);
    let value = world.query::<&A>().iter(&world).next().unwrap().0;
    println!("R4c ParamSet 双 Query：正常运行，A={}（1+1+2）", value);
    assert_eq!(value, 4.0, "修正预期：两个 p 顺序执行");
}

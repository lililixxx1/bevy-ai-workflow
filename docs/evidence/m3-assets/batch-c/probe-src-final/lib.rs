//! M3 Block C 探针（ECS 查询与调度域）——「凭记忆首试」载体。
//!
//! 仓库外临时 crate（不入本仓 workspace）。时序纪律（docs/m3-plan.md §3.1）：
//! 本文件在**未查证 bevy 0.19.1 源码**的前提下按训练语料直觉书写；
//! cargo check 的失败原文先归档（docs/evidence/m3-assets/batch-c/probe-r1-check.log），
//! 之后才查源码修正。首试时间：2026-09-27。
//!
//! r1 轮结论（2026-09-27，REAL_EXIT=101）：失败 6 处 / 编译通过 4 处（详见下方留痕）。
//! r2 轮（查证后正解，逐条对照源码行号）：见文件尾部「r2 正解形态」。
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Component, Default)]
struct Pos(f32);

#[derive(Component, Default)]
struct Vel(f32);

#[derive(Resource, Default)]
struct Cfg {
    on: bool,
}

fn noop() {}

// P1【r1 失败】E0609：no field `0` on `Result<&Pos, QuerySingleError>`——single() 返回 Result。
// fn p1_single(q: Query<&Pos>) -> f32 {
//     let p = q.single();
//     p.0
// }

// P2（r1 编译通过）
fn p2_iter_many(q: Query<&Pos>, ids: &[Entity]) -> usize {
    q.iter_many(ids).count()
}

// P3【r1 失败】E0599：QueryParIter is not an iterator。
// fn p3_par_iter(q: Query<&Pos>) -> usize {
//     q.par_iter().count()
// }

// P4【r1 失败】E0599：no method named `add`。
// fn p4_commands_add(mut commands: Commands) {
//     commands.add(|world: &mut World| {
//         let _ = world.entities().len();
//     });
// }

// P5【r1 失败】E0277：`&mut Vel: ReadOnlyQueryData` 不满足（for/IntoIterator 不可用）。
// fn p5_iter_combinations(mut q: Query<&mut Vel>) -> usize {
//     let mut n = 0;
//     for pair in q.iter_combinations_mut::<2>() {
//         let [a, b] = pair;
//         a.0 += b.0;
//         n += 1;
//     }
//     n
// }

// P6（r1 编译通过——编译期不拦截，运行语义见 R4：首帧 B0001 panic）
fn p6_two_mut_queries(mut a: Query<&mut Vel>, mut b: Query<&mut Vel>) {
    for mut v in &mut a {
        v.0 += 1.0;
    }
    for mut v in &mut b {
        v.0 += 2.0;
    }
}

// P7（r1 编译通过）
fn p7_res_is_changed(cfg: Res<Cfg>) -> bool {
    cfg.is_changed()
}

// P8（r1 编译通过）
fn p8_run_if_capture(app: &mut App, flag: bool) {
    app.add_systems(Update, noop.run_if(move || flag));
}

// P9：derive + configure_sets 编译通过；add_systems(MySets::A, ..) 失败
// 【r1 失败】E0277：`MySets: ScheduleLabel` is not satisfied（bevy_app-0.19.1/src/app.rs:323）。
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum MySets {
    A,
    B,
}

fn p9_configure_sets(app: &mut App) {
    app.configure_sets(Update, (MySets::A, MySets::B).chain());
    // app.add_systems(MySets::A, p6_two_mut_queries); // ← r1 失败行（留痕）
}

// P10【r1 失败】E0599：no method named `get_single`（help：有 single）。
// fn p10_get_single(q: Query<&Pos>) -> f32 {
//     q.get_single().unwrap().0
// }

// ==================== r2 正解形态（2026-09-27，查证后） ====================

/// P1c/P10c（→PIT-B-005）：`single()`/`single_mut()` 返回
/// `Result<_, QuerySingleError>`（system/query.rs:2097/:2126）；`get_single` 不存在
/// （query.rs 全文 0 处）。panic 语义由调用方 `.expect()` 自选。
fn p1c_single(q: Query<&Pos>) -> f32 {
    q.single().map(|p| p.0).unwrap_or(0.0)
}

/// P3c（→PIT-B-006）：`QueryParIter` **不实现任何 Iterator trait**（也无 rayon
/// ParallelIterator）——`.count()/.map()` 一律 E0599；并行面是固有方法
/// `for_each` / `for_each_init`（par_iter.rs:31/:55，内部按 `multi_threaded`
/// feature 与 ComputeTaskPool 线程数自动选并行/串行实现 :86-:120）。
/// feature 链存在（bevy Cargo.toml:2818 → bevy_internal:323-326）但门面
/// default=[2d,3d,ui,audio]（:2742-2747）不含 multi_threaded——关闭时
/// for_each 退化为串行 fold（:86 cfg 分支）。
fn p3c_par_iter(q: Query<&Pos>) -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let n = AtomicUsize::new(0);
    q.par_iter().for_each(|_| {
        n.fetch_add(1, Ordering::Relaxed);
    });
    n.load(Ordering::Relaxed)
}

/// P4c（→PIT-B-007）：入队命令是 `Commands::queue`（commands/mod.rs:641
/// `pub fn queue(&mut self, command: impl Command)`）；`add` 不存在。
fn p4c_commands_queue(mut commands: Commands) {
    commands.queue(|world: &mut World| {
        let _ = world.entities().len();
    });
}

/// P5c（→PIT-B-008）：可变组合迭代**不用 for**——`QueryCombinationIter` 的
/// `Iterator/IntoIterator` impl 仅对 `ReadOnlyQueryData` 成立；可变数据走
/// `fetch_next()`（官方示例 system/query.rs:795-800）。
fn p5c_combinations(mut q: Query<&mut Vel>) -> usize {
    let mut n = 0;
    let mut it = q.iter_combinations_mut::<2>();
    while let Some([mut a, b]) = it.fetch_next() {
        a.0 += b.0;
        n += 1;
    }
    n
}

/// P9c（→PIT-B-009）：系统入集 = `.in_set(MySets::A)`（schedule/config.rs:322/:493），
/// `add_systems` 首参恒为 `ScheduleLabel`（app.rs:323）；configure_sets 形态不变。
fn p9c_in_set(app: &mut App) {
    app.add_systems(Update, noop.in_set(MySets::A));
}

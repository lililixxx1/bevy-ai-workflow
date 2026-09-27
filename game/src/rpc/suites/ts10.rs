//! TS-10《事件驱动的状态变更》的游戏进程内断言（两阶段判定第二阶段）。
//!
//! 与 `assets-methodology/taskset/ts-10-trigger-event.md` 验收清单的对应：
//! - #1（`paused == false` 初始态）→ [`paused_initially_false`]；
//! - #2 + #3（trigger_event 后 `paused == true`）→ [`first_trigger_pauses`]
//!   （`World::trigger` 同步执行 observer——翻转在返回前已生效，BRP
//!   `world.trigger_event` 的 HTTP 响应返回前同理，sim.rs 模块文档已核）；
//! - #4（再触发一次 `paused == false`）→ [`second_trigger_resumes`]。
//! 偶数次触发后 paused 回初始值，世界无净副作用。
//!
//! 依据: `World::trigger` 同步运行 observer（bevy_ecs-0.19.1/src/observer/
//! mod.rs:63；game/src/sim.rs 单测 `pause_requested_event_flips_paused` 同形态
//! 已验证）（核实 2026-09-27）。BRP `world.trigger_event` 反射构造通路第一阶段
//! 已证（证据 ts-10-brp.md），本套件复测判定语义（翻转语义本身）。

use bevy::prelude::*;

use super::{tc, TestCase};
use crate::sim::{PauseRequested, SimConfig};

/// TS-10 套件入口（写通路型：偶数次触发回初始态，无净副作用）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    let initial = world
        .get_resource::<SimConfig>()
        .map(|c| c.paused);
    let mut results = vec![paused_initially_false(initial)];
    world.trigger(PauseRequested);
    results.push(trigger_flips(world, initial, false));
    world.trigger(PauseRequested);
    results.push(trigger_flips(world, initial, true));
    results
}

/// 清单 #1：初始态未暂停（若前置套件留下 paused=true 会如实失败——可观测）。
fn paused_initially_false(initial: Option<bool>) -> TestCase {
    tc(
        "paused_initially_false",
        initial == Some(false),
        format!("初始 paused={initial:?}（清单要求 false）"),
    )
}

/// 清单 #3/#4：触发后 paused == !initial（第一次）／ == initial（第二次，翻转回）。
fn trigger_flips(world: &mut World, initial: Option<bool>, second: bool) -> TestCase {
    let current = world.get_resource::<SimConfig>().map(|c| c.paused);
    let expect = if second { initial } else { initial.map(|b| !b) };
    let name = if second { "second_trigger_resumes" } else { "first_trigger_pauses" };
    let semantics = if second { "第二次触发回初始值" } else { "第一次触发取反" };
    tc(
        name,
        current == expect && current.is_some(),
        format!("触发后 paused={current:?}（期望 {expect:?}，翻转语义：{semantics}）"),
    )
}

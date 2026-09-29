//! [`game.snapshot`]：稳定 id 世界状态快照（窗口前置增强 T040 / A4，
//! `docs/pre-window-plan.md` §二；taskset TS-17）。
//!
//! 动因：T032/T034/T035/T036 反复出现「实体号变化属预期、用业务键绕行」
//! （PIT-B-010 迭代序纪律）——固化为一次性契约：以业务键为稳定 id 的
//! 只读快照方法，工具端（及 T041 MCP 薄桥）从此不在快照面接触实体号。
//! 字段集收编 T035 已验证的驱动侧重放快照（`docs/evidence/ts-16/campaign.js`
//! 的 `snap()`），不改语义、只改承载（驱动侧拼装 → 游戏进程内一次调用）。
//!
//! 字段来源（**逐字段指认，防混编**——写快照断言时按本表对）：
//!
//! | 面 | 字段 | 来源 |
//! |---|---|---|
//! | battle.units[] | x / y | [`crate::level::GridPos`]（业务键，按 `(x, y)` 升序——同格占用约束保证键唯一） |
//! | battle.units[] | team / hp | [`crate::level::Unit`]（move_range / attack 不在 T035 字段集，不取） |
//! | battle.units[] | moved / attacked | [`crate::battle::ActionFlags`] |
//! | battle | turn / winner / phase / rng_state | [`crate::battle::BattleState`] 资源（**不含** goal/seed/ai_kind——T035 字段集即此四项 + units） |
//! | sim | paused | [`crate::sim::SimConfig`] 资源 |
//! | sim.wanderers[] | index / origin / phase | [`crate::sim::Wanderer`] 组件（index 为确定性口径锚点，升序） |
//! | sim.wanderers[] | velocity | [`crate::sim::Velocity`] 组件（与 Wanderer 由 spawn_swarm 成对生成） |
//!
//! T035 驱动侧快照的 `level` / `op` 系**驱动侧记账**（第几关第几步），
//! 非世界状态，不入本方法。wanderers 取确定性初值四元组
//! （index/origin/velocity/phase——即 [`crate::sim::draw_initial`] 的完整输出面），
//! **不含** `Transform`/`SimStats` 等时间依赖数据：位置是 `elapsed_secs` 的解析
//! 函数，帧时序不同则不同，「同 seed 跨进程快照逐字节一致」的面在时间无关
//! 的初值上才成立（TS-16 重放口径的 sim 侧对应物）。
//!
//! 精度口径：`rng_state`（u64）在 JS 工具端沿 PIT-M-009 处理（字符串化或排除
//! 出严格相等；沿 TS-16「f64 渲染分辨率下一致」限定先例，不得倒退）。Rust /
//! serde_json 侧整数为精确 u64，不经 f64。
//!
//! 请求：无参数（`params` 缺省或空对象 `{}` 均可；其他形态 → INVALID_PARAMS
//! -32602，与 [`crate::rpc::coord_pair`] 族的严格拒绝面一致）。响应：
//!
//! ```text
//! {"battle": null | {"turn","winner","phase","rng_state","units":[…]},
//!  "sim":   null | {"paused","wanderers":[…]}}
//! ```
//!
//! `battle`/`sim` 为 `null` 的语义 = 对应资源缺失（未加载关卡 / 无 SimConfig），
//! 沿 [`crate::rpc::run_tests`] 报文内可选字段以 `null` 表缺失的约定
//! （与 `world.get_resources` 对已注册未插入资源报 -23501 是两条通路，不混用）。
//!
//! handler 形态（SKILL.md §6.2）：`fn(In(params): In<Option<Value>>,
//! world: &mut World) -> BrpResult`，`with_method_main` 注册、经
//! `world.run_system_with` 独占执行；只读、全程无 panic 路径。

use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::{json, Value};

use super::invalid_params;
use crate::battle::{units_sorted, BattleState};
use crate::sim::{SimConfig, Velocity, Wanderer};

/// `game.snapshot` handler 入口（只读型：无副作用，任意次连调互不污染）。
pub fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    match params {
        None => {}
        Some(ref v) if v.as_object().is_some_and(|m| m.is_empty()) => {}
        Some(_) => {
            return Err(invalid_params(
                "params 须为空对象或缺省（game.snapshot 无参数）",
            ))
        }
    }

    // battle 面：BattleState 先拷出再建 query（units_sorted 要 &mut World，
    // 与资源借用互斥）；缺失 → null（不自造默认值）。
    let battle_state = world.get_resource::<BattleState>().cloned();
    let battle = battle_state.map(|st| {
        json!({
            "turn": st.turn,
            "winner": st.winner,
            "phase": st.phase,
            "rng_state": st.rng_state,
            "units": units_sorted(world)
                .into_iter()
                .map(|v| json!({
                    "x": v.pos.x,
                    "y": v.pos.y,
                    "team": v.unit.team,
                    "hp": v.unit.hp,
                    "moved": v.flags.moved,
                    "attacked": v.flags.attacked,
                }))
                .collect::<Vec<Value>>(),
        })
    });

    // sim 面：paused（Copy，先取出）+ wanderers 确定性初值四元组，
    // 按 Wanderer::index 升序（PIT-B-010：不依赖 query 迭代序）。
    let paused = world.get_resource::<SimConfig>().map(|c| c.paused);
    let mut wanderers: Vec<(u32, &Wanderer, &Velocity)> = {
        let mut q = world.query::<(&Wanderer, &Velocity)>();
        q.iter(world).map(|(w, v)| (w.index, w, v)).collect()
    };
    wanderers.sort_by_key(|(index, _, _)| *index);
    let sim = paused.map(|paused| {
        json!({
            "paused": paused,
            "wanderers": wanderers
                .iter()
                .map(|(index, w, v)| json!({
                    "index": *index,
                    "origin": [w.origin.x, w.origin.y, w.origin.z],
                    "velocity": [v.linear.x, v.linear.y, v.linear.z],
                    "phase": w.phase,
                }))
                .collect::<Vec<Value>>(),
        })
    });

    Ok(json!({ "battle": battle, "sim": sim }))
}

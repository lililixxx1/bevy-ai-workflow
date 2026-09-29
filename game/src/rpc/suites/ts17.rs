//! TS-17《稳定 id 快照 game.snapshot》的游戏进程内断言（窗口前置增强
//! T040 / A4，taskset TS-17；`docs/pre-window-plan.md` §二）。
//!
//! 对 [`crate::rpc::snapshot::handler`] **直调**（与 BRP 执行同一函数——
//! `world.run_system_with` 载体不同、函数体相同），关卡面经
//! [`crate::rpc::launch_level::handler`] 同路建立（不走手植布阵，断言的是
//! 真实 RPC 语义链）。断言面：
//! - [`field_set_matches_t035`]：battle 面字段集 == T035 驱动侧快照集
//!   （units{x,y,team,hp,moved,attacked} + turn/winner/phase/rng_state；
//!   T035 的 level/op 系驱动侧记账，不入世界快照）——键集逐级精确相等；
//! - [`snapshot_equivalent_to_world_query`]：逐字段与**独立** `world.query` /
//!   资源读值等价（本套件自建 QueryState，不复用 handler 同源的
//!   [`crate::battle::units_sorted`]——独立路径才是交叉验证）；
//! - [`business_key_ordering`]：units 按 `(x, y)`、wanderers 按 `index`
//!   严格升序（PIT-B-010）；
//! - [`relaunch_same_seed_same_snapshot`]：同 seed 重 launch 快照串逐位一致
//!   **且实体号实际变化**（实体号排除口径的正证：id 换血、快照不动）；
//! - [`battle_null_when_unloaded`]：BattleState 缺失 → `battle == null`
//!   （不自造默认值）；
//! - [`world_restored_after_suite`]：只读快照 + 入场快照还原，无净副作用
//!   （launch 建立的战场在出场时清还原；实体号允许变化，三元组逐位）。

use bevy::prelude::*;
use serde_json::{json, Value};

use super::{tc, TestCase};
use crate::battle::{units_sorted, ActionFlags, BattleState};
use crate::level::{GridPos, LevelState, Unit};
use crate::rpc::{launch_level, snapshot};
use crate::sim::{SimConfig, Velocity, Wanderer};

/// 入场世界快照（与 ts-14 同款：单位三组件原值；资源原值或 None=缺失）。
struct Saved {
    units: Vec<(GridPos, Unit, ActionFlags)>,
    level: Option<LevelState>,
    battle: Option<BattleState>,
}

fn save_world(world: &mut World) -> Saved {
    let units = units_sorted(world)
        .into_iter()
        .map(|v| (v.pos, v.unit, v.flags))
        .collect();
    Saved {
        units,
        level: world.get_resource::<LevelState>().cloned(),
        battle: world.get_resource::<BattleState>().cloned(),
    }
}

/// 直调 snapshot handler（Err 一律转断言 false，不 panic——套件纪律）。
fn snap(world: &mut World) -> Result<Value, String> {
    snapshot::handler(In(None), world).map_err(|e| format!("handler 错误 code={} message={}", e.code, e.message))
}

/// 直调 launch_level handler。
fn launch(world: &mut World, level: u32, seed: u64) -> Result<Value, String> {
    launch_level::handler(In(Some(json!({ "level": level, "seed": seed }))), world)
        .map_err(|e| format!("launch code={} message={}", e.code, e.message))
}

/// 独立单位读值（本套件自建 QueryState + 自排序——交叉验证的独立性要求）。
fn query_units(world: &mut World) -> Vec<(i32, i32, u8, i32, bool, bool)> {
    let mut q = world.query::<(&GridPos, &Unit, &ActionFlags)>();
    let mut rows: Vec<_> = q
        .iter(world)
        .map(|(p, u, f)| (p.x, p.y, u.team, u.hp, f.moved, f.attacked))
        .collect();
    rows.sort_by_key(|r| (r.0, r.1));
    rows
}

/// 独立 wanderer 读值（(index, origin, velocity, phase) 确定性初值四元组）。
fn query_wanderers(world: &mut World) -> Vec<(u32, [f32; 3], [f32; 3], f32)> {
    let mut q = world.query::<(&Wanderer, &Velocity)>();
    let mut rows: Vec<_> = q
        .iter(world)
        .map(|(w, v)| {
            (
                w.index,
                [w.origin.x, w.origin.y, w.origin.z],
                [v.linear.x, v.linear.y, v.linear.z],
                w.phase,
            )
        })
        .collect();
    rows.sort_by_key(|r| r.0);
    rows
}

/// 对象键集（排序后比较——不依赖 serde Map 内部序）。
fn keys_of(v: &Value) -> Vec<String> {
    let mut ks = v
        .as_object()
        .map(|m| m.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    ks.sort();
    ks
}

/// TS-17 套件入口（只读快照 + 建场还原：launch 写入面在出场时清还原）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    let saved = save_world(world);
    let baseline_count = saved.units.len();
    let mut cases = Vec::new();

    // —— 未加载关卡面（BattleState 缺失 → battle null）——
    let had_battle = saved.battle.is_some();
    if had_battle {
        let _ = world.remove_resource::<BattleState>();
    }
    let cold = snap(world);
    let null_ok = matches!(&cold, Ok(v) if v["battle"].is_null() && !v["sim"].is_null());
    cases.push(tc(
        "battle_null_when_unloaded",
        null_ok,
        format!(
            "BattleState 缺失时 battle={}（null 约定，不自造默认值）、sim 非缺省（入场{}关卡，移除后复测）",
            cold.as_ref().map(|v| v["battle"].to_string()).unwrap_or_else(|e| format!("<{e}>")),
            if had_battle { "有" } else { "无" },
        ),
    ));

    // —— 建场（真实 launch 通路）+ 字段集 / 等价 / 键序 ——
    if let Err(e) = launch(world, 1, 20260928) {
        cases.push(tc("field_set_matches_t035", false, format!("launch 失败：{e}")));
        cases.push(tc("snapshot_equivalent_to_world_query", false, "未建场".into()));
        cases.push(tc("business_key_ordering", false, "未建场".into()));
        cases.push(tc("relaunch_same_seed_same_snapshot", false, "未建场".into()));
        return finish(world, cases, saved, baseline_count);
    }
    let resp = match snap(world) {
        Ok(v) => v,
        Err(e) => {
            let names = ["field_set_matches_t035", "snapshot_equivalent_to_world_query", "business_key_ordering", "relaunch_same_seed_same_snapshot"];
            for n in names {
                cases.push(tc(n, false, format!("snapshot 失败：{e}")));
            }
            return finish(world, cases, saved, baseline_count);
        }
    };

    // 字段集 == T035 驱动侧快照集（五级键集精确相等）。
    let expect_top = ["battle", "sim"];
    let expect_battle = ["phase", "rng_state", "turn", "units", "winner"];
    let expect_unit = ["attacked", "hp", "moved", "team", "x", "y"];
    let expect_sim = ["paused", "wanderers"];
    let expect_wanderer = ["index", "origin", "phase", "velocity"];
    let unit0 = resp["battle"]["units"].get(0).cloned().unwrap_or(Value::Null);
    let wanderer0 = resp["sim"]["wanderers"].get(0).cloned().unwrap_or(Value::Null);
    let sets_ok = keys_of(&resp) == expect_top
        && keys_of(&resp["battle"]) == expect_battle
        && keys_of(&unit0) == expect_unit
        && keys_of(&resp["sim"]) == expect_sim
        && keys_of(&wanderer0) == expect_wanderer;
    cases.push(tc(
        "field_set_matches_t035",
        sets_ok,
        format!(
            "五级键集精确相等（top/battle/unit/sim/wanderer；battle={} 关卡已加载）——T035 字段集收编契约，level/op 系驱动侧记账不入",
            if resp["battle"].is_null() { "null" } else { "非null" }
        ),
    ));

    // 逐字段等价：battle/sim 两面 vs 独立 query / 资源读值。
    let truth_units = query_units(world);
    let units_json = resp["battle"]["units"].as_array().cloned().unwrap_or_default();
    let mut units_eq = units_json.len() == truth_units.len();
    for (j, (x, y, team, hp, moved, attacked)) in truth_units.iter().enumerate() {
        let u = units_json.get(j).cloned().unwrap_or(Value::Null);
        units_eq &= u["x"].as_i64() == Some(*x as i64)
            && u["y"].as_i64() == Some(*y as i64)
            && u["team"].as_i64() == Some(*team as i64)
            && u["hp"].as_i64() == Some(*hp as i64)
            && u["moved"].as_bool() == Some(*moved)
            && u["attacked"].as_bool() == Some(*attacked);
    }
    let st = world.get_resource::<BattleState>().cloned();
    let battle_eq = st.as_ref().is_some_and(|st| {
        resp["battle"]["turn"].as_u64() == Some(st.turn as u64)
            && resp["battle"]["winner"].as_i64() == Some(st.winner as i64)
            && resp["battle"]["phase"].as_u64() == Some(st.phase as u64)
            && resp["battle"]["rng_state"].as_u64() == Some(st.rng_state)
    });
    let cfg_paused = world.get_resource::<SimConfig>().map(|c| c.paused);
    let sim_present_eq = cfg_paused.is_some_and(|p| resp["sim"]["paused"].as_bool() == Some(p));
    let truth_wanderers = query_wanderers(world);
    let wanderers_json = resp["sim"]["wanderers"].as_array().cloned().unwrap_or_default();
    let mut wanderers_eq = wanderers_json.len() == truth_wanderers.len();
    for (j, (index, origin, velocity, phase)) in truth_wanderers.iter().enumerate() {
        let w = wanderers_json.get(j).cloned().unwrap_or(Value::Null);
        let arr_eq = |v: &Value, a: &[f32; 3]| {
            v.as_array().is_some_and(|arr| {
                arr.len() == 3
                    && arr[0].as_f64() == Some(a[0] as f64)
                    && arr[1].as_f64() == Some(a[1] as f64)
                    && arr[2].as_f64() == Some(a[2] as f64)
            })
        };
        wanderers_eq &= w["index"].as_u64() == Some(*index as u64)
            && arr_eq(&w["origin"], origin)
            && arr_eq(&w["velocity"], velocity)
            && w["phase"].as_f64() == Some(*phase as f64);
    }
    cases.push(tc(
        "snapshot_equivalent_to_world_query",
        units_eq && battle_eq && sim_present_eq && wanderers_eq,
        format!(
            "units {}/{} 逐字段逐序相等（独立 QueryState）、battle 四字段==BattleState 资源、paused==SimConfig、wanderers {}/{} 四元组相等（f32 经 f64 拓宽严格相等——注入无损）",
            truth_units.len().min(units_json.len()),
            truth_units.len(),
            truth_wanderers.len().min(wanderers_json.len()),
            truth_wanderers.len(),
        ),
    ));

    // 键序：units (x,y) 严格升序；wanderers index 严格升序（键唯一 ⇒ 可严格）。
    let unit_keys: Vec<(i64, i64)> = units_json
        .iter()
        .map(|u| (u["x"].as_i64().unwrap_or(i64::MIN), u["y"].as_i64().unwrap_or(i64::MIN)))
        .collect();
    let wanderer_keys: Vec<u64> = wanderers_json.iter().map(|w| w["index"].as_u64().unwrap_or(u64::MAX)).collect();
    let sorted_units = unit_keys.windows(2).all(|w| w[0] < w[1]);
    let sorted_wanderers = wanderer_keys.windows(2).all(|w| w[0] < w[1]);
    cases.push(tc(
        "business_key_ordering",
        sorted_units && sorted_wanderers,
        format!(
            "units {} 项按 (x,y) 严格升序={}；wanderers {} 项按 index 严格升序={}（PIT-B-010，不依赖 query 迭代序）",
            unit_keys.len(), sorted_units, wanderer_keys.len(), sorted_wanderers
        ),
    ));

    // 同 seed 重 launch：快照串逐位一致 + 实体号换血（排除口径正证）。
    let s1 = serde_json::to_string(&resp).unwrap_or_default();
    let ids1: Vec<Entity> = units_sorted(world).into_iter().map(|v| v.entity).collect();
    let relaunch = launch(world, 1, 20260928);
    let s2snap = snap(world);
    let ids2: Vec<Entity> = units_sorted(world).into_iter().map(|v| v.entity).collect();
    let relaunch_ok = matches!((&relaunch, &s2snap), (Ok(_), Ok(v2)) if serde_json::to_string(v2).unwrap_or_default() == s1);
    let ids_changed = ids1 != ids2;
    cases.push(tc(
        "relaunch_same_seed_same_snapshot",
        relaunch_ok && ids_changed,
        format!(
            "重 launch 后快照串逐位一致={relaunch_ok}（serde_json 键序确定，{} 字符）且实体号集合已变化={ids_changed}（快照等值与实体号无关——跨进程逐字节一致的进程内前奏）",
            s1.len()
        ),
    ));

    finish(world, cases, saved, baseline_count)
}

/// 出场还原（清 launch 建立的战场 → 入场快照重生 / 资源还原或移除）+ 还原断言。
fn finish(world: &mut World, mut cases: Vec<TestCase>, saved: Saved, baseline_count: usize) -> Vec<TestCase> {
    for v in units_sorted(world) {
        let _ = world.despawn(v.entity);
    }
    for (pos, unit, flags) in &saved.units {
        world.spawn((*pos, *unit, *flags));
    }
    match &saved.level {
        Some(l) => world.insert_resource(l.clone()),
        None => {
            let _ = world.remove_resource::<LevelState>();
        }
    }
    match &saved.battle {
        Some(b) => world.insert_resource(b.clone()),
        None => {
            let _ = world.remove_resource::<BattleState>();
        }
    }
    let level_restored = saved
        .level
        .as_ref()
        .map(|l| world.get_resource::<LevelState>() == Some(l))
        .unwrap_or_else(|| world.get_resource::<LevelState>().is_none());
    let battle_restored = saved
        .battle
        .as_ref()
        .map(|b| world.get_resource::<BattleState>() == Some(b))
        .unwrap_or_else(|| world.get_resource::<BattleState>().is_none());
    let after: Vec<(GridPos, Unit, ActionFlags)> =
        units_sorted(world).into_iter().map(|v| (v.pos, v.unit, v.flags)).collect();
    let units_exact = after == saved.units;
    let restored = units_exact && after.len() == baseline_count && level_restored && battle_restored;
    cases.push(tc(
        "world_restored_after_suite",
        restored,
        format!(
            "出场单位 {} == 入场 {baseline_count} 且三元组逐位还原（快照重生，实体号允许变化）、LevelState/BattleState 还原（含缺失分支）",
            after.len()
        ),
    ));
    cases
}

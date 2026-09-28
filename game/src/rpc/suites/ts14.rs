//! TS-14《战斗核心》的游戏进程内断言（TS-15 任务：BRP 驱动（第一阶段，
//! `docs/evidence/ts-14-brp.md` 12/12）→ 进程内回归套件（本文件）——沿 M1
//! 两阶段判定先例）。
//!
//! 与 `assets-methodology/taskset/ts-14-battle-core.md` 验收清单的对应（BRP
//! 反射通路第一阶段已证，此处以**同一规则函数**（[`crate::battle`] 系列，
//! 与 BRP handler 同一 `&mut World` 通路）做进程内回归）：
//! - #2（`BattleState` 初值）→ [`battle_state_initial`]；
//! - #3/#4（移动成功与错误面）→ [`move_accept_and_reject`]；
//! - #5/#7/#8（攻击错误面在外、击杀-歼灭-终局封锁在内）→
//!   [`attack_kill_annihilate_and_lock`]；
//! - #6（end_turn 语义）→ [`end_turn_twin_deterministic`]：同种子**同世界双跑**
//!   （setup → 3× end_turn → 记录业务键视图与 BattleState → 清场重跑）逐位
//!   一致——比单跑更强的确定性回归（含 RNG 续跑面）；
//! - #10（占点型）→ [`reach_goal_wins_on_move`]；
//! - 无净副作用（`suites` 模块约定）：入场快照（全部单位三组件 +
//!   `LevelState`/`BattleState` 原值或缺失），出场清理测试单位并按快照重生/
//!   还原或移除资源——[`world_restored_after_suite`] 断言回基线（实体号允许
//!   变化，组件值逐位还原；ts-12 快照重生同款口径）。

use bevy::prelude::*;

use super::{tc, TestCase};
use crate::battle::{
    self, units_sorted, ActionFlags, BattleState, AI_RANDOM, GOAL_ANNIHILATE, GOAL_REACH,
    PHASE_OVER, PHASE_PLAYER,
};
use crate::level::{GridPos, LevelState, Unit};

/// 入场世界快照（单位三组件原值；资源原值或 None=缺失）。
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

/// 套件内测试底座：清空现役单位 + 写 9x9 关卡态与 L1 数值布阵（手植，与
/// `battle.rs` 单测同款但走资源插入——便于规则函数直跑）。
fn setup(world: &mut World, layout: &[(u8, i32, i32)], ai_kind: u8, seed: u64, goal: (u8, i32, i32)) {
    for v in units_sorted(world) {
        let _ = world.despawn(v.entity);
    }
    for (team, x, y) in layout {
        let (hp, mv, atk) = if *team == 0 { (10, 3, 2) } else { (8, 2, 2) };
        world.spawn((
            GridPos { x: *x, y: *y },
            Unit { team: *team, hp, move_range: mv, attack: atk },
            ActionFlags::default(),
        ));
    }
    world.insert_resource(LevelState { level: 1, width: 9, height: 9, units_spawned: layout.len() as u32 });
    world.insert_resource(BattleState {
        phase: PHASE_PLAYER,
        turn: 1,
        winner: -1,
        goal_kind: goal.0,
        goal_x: goal.1,
        goal_y: goal.2,
        seed,
        rng_state: seed,
        ai_kind,
    });
}

/// 业务键视图（确定性比对载体：同布局 → 同串）。
fn view(world: &mut World) -> String {
    let units: Vec<String> = units_sorted(world)
        .into_iter()
        .map(|v| format!("{:?}|{:?}|{:?}", v.unit.team, (v.pos.x, v.pos.y), v.unit.hp))
        .collect();
    let st = world.get_resource::<BattleState>().cloned();
    format!("{}||{:?}", units.join(";"), st)
}

/// TS-14 套件入口（写通路型：入场快照 + 出场还原，无净副作用）。
pub fn run(world: &mut World) -> Vec<TestCase> {
    let saved = save_world(world);
    let baseline_count = saved.units.len();

    let mut cases = Vec::new();
    cases.extend(battle_state_initial(world));
    cases.extend(move_accept_and_reject(world));
    cases.extend(attack_kill_annihilate_and_lock(world));
    cases.extend(end_turn_twin_deterministic(world));
    cases.extend(reach_goal_wins_on_move(world));

    // 还原：清测试单位 → 快照重生（三组件逐位）→ 资源还原或移除。
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
    // R1 审核 S3：不只比计数——出场 units_sorted 与入场快照按业务键序逐位
    // 比较三元组（GridPos/Unit/ActionFlags 原值；两侧同序，实体号不在比较面）。
    let after: Vec<(GridPos, Unit, ActionFlags)> =
        units_sorted(world).into_iter().map(|v| (v.pos, v.unit, v.flags)).collect();
    let units_exact = after == saved.units;
    let restored = units_exact && after.len() == baseline_count && level_restored && battle_restored;
    cases.push(tc(
        "world_restored_after_suite",
        restored,
        format!(
            "出场单位 {} == 入场 {baseline_count} 且三元组（位置/数值/标记）逐位还原（快照重生，实体号允许变化）、LevelState/BattleState 还原（含缺失分支）",
            after.len()
        ),
    ));
    cases
}

/// 清单 #2 等价：BattleState 初值逐位（歼灭目标字段面）。
fn battle_state_initial(world: &mut World) -> Vec<TestCase> {
    setup(world, &[(0, 1, 1), (1, 5, 7)], AI_RANDOM, 7, (GOAL_ANNIHILATE, -1, -1));
    let st = world.resource::<BattleState>().clone();
    vec![tc(
        "battle_state_initial",
        st.phase == PHASE_PLAYER && st.turn == 1 && st.winner == -1 && st.goal_kind == GOAL_ANNIHILATE
            && st.goal_x == -1 && st.goal_y == -1 && st.seed == 7 && st.rng_state == 7 && st.ai_kind == AI_RANDOM,
        format!("{st:?}（== launch 初值口径，TS-14 #2 进程内等价）"),
    )]
}

/// 清单 #3/#4 等价：移动成功 + 三类拒绝（范围/占用/重复）。
fn move_accept_and_reject(world: &mut World) -> Vec<TestCase> {
    setup(world, &[(0, 1, 1), (0, 2, 1), (1, 5, 7)], AI_RANDOM, 1, (GOAL_ANNIHILATE, -1, -1));
    let ok = battle::try_move(world, (1, 1), (1, 3)).is_ok();
    let rejected = [
        battle::try_move(world, (1, 3), (1, 4)).is_err(), // 已移动
        battle::try_move(world, (2, 1), (2, 8)).is_err(), // 超范围（7 > 3）
        battle::try_move(world, (2, 1), (1, 3)).is_err(), // 占用
    ]
    .iter().all(|b| *b);
    let pos_ok = units_sorted(world).iter().any(|v| v.pos.x == 1 && v.pos.y == 3 && v.unit.team == 0);
    vec![tc(
        "move_accept_and_reject",
        ok && rejected && pos_ok,
        format!("成功 1 例（(1,1)→(1,3) 生效 {pos_ok}）+ 拒绝 3 类（已移动/超范围/占用）全 Err（TS-14 #3/#4 进程内等价）"),
    )]
}

/// 清单 #7/#8 内核：击杀 → 歼灭终局 → 终局封锁（三条规则函数全拒）。
fn attack_kill_annihilate_and_lock(world: &mut World) -> Vec<TestCase> {
    setup(world, &[(0, 1, 1), (1, 2, 1)], AI_RANDOM, 1, (GOAL_ANNIHILATE, -1, -1));
    let mut kills = 0;
    let mut last_alive = true;
    for _ in 0..4 {
        // 每击后复位攻方 attacked（等价换回合的标记复位——end_turn 语义另测）。
        if let Some(v) = units_sorted(world).into_iter().find(|v| v.pos == GridPos { x: 1, y: 1 }) {
            if let Some(mut f) = world.get_mut::<ActionFlags>(v.entity) {
                f.attacked = false;
            }
        }
        match battle::try_attack(world, (1, 1), (2, 1)) {
            Ok(o) => {
                last_alive = o.target_alive;
                if !o.target_alive {
                    kills += 1;
                }
            }
            Err(e) => return vec![tc("attack_kill_annihilate_and_lock", false, format!("中途拒绝：{e}"))],
        }
    }
    let st = world.resource::<BattleState>().clone();
    let locked = [
        battle::try_move(world, (1, 1), (1, 2)).is_err(),
        battle::try_attack(world, (1, 1), (2, 1)).is_err(),
        battle::end_turn(world).is_err(),
    ].iter().all(|b| *b);
    vec![tc(
        "attack_kill_annihilate_and_lock",
        kills == 1 && !last_alive && st.winner == 0 && st.phase == PHASE_OVER && locked,
        format!(
            "hp8/atk2 四击歼灭（kills={kills}，末击 target_alive={last_alive}）、winner={} phase={}（TS-14 #7 内核）、终局封锁三拒 {locked}（#8 内核）",
            st.winner, st.phase
        ),
    )]
}

/// 清单 #6 等价 + 确定性回归：同世界同种子双跑（含 RNG 续跑面）逐位一致。
fn end_turn_twin_deterministic(world: &mut World) -> Vec<TestCase> {
    let layout = [(0, 1, 1), (0, 2, 1), (0, 3, 1), (1, 5, 7), (1, 6, 7), (1, 7, 7)];
    let mut runs: Vec<String> = Vec::new();
    let mut rng_advanced = false;
    for _ in 0..2 {
        setup(world, &layout, AI_RANDOM, 20260928, (GOAL_ANNIHILATE, -1, -1));
        for _ in 0..3 {
            if battle::end_turn(world).is_err() {
                return vec![tc("end_turn_twin_deterministic", false, "end_turn 中途拒绝".into())];
            }
        }
        let st = world.resource::<BattleState>();
        rng_advanced |= st.rng_state != st.seed;
        runs.push(view(world));
    }
    vec![tc(
        "end_turn_twin_deterministic",
        runs[0] == runs[1] && rng_advanced,
        format!(
            "同世界双跑（seed=20260928，各 3× end_turn）业务键视图+BattleState 逐位一致（{} 字符串相等）且 RNG 状态推进（TS-14 #6 + 确定性纪律进程内回归）",
            if runs[0] == runs[1] { "两串" } else { "不" }
        ),
    )]
}

/// 清单 #10 内核：占点型移动即终局（R1 审核 B1：带敌阵容 + 负控——零敌时
/// `check_victory` 的 enemies==0 分支先行成立，占点路径将无区分力）。
fn reach_goal_wins_on_move(world: &mut World) -> Vec<TestCase> {
    setup(world, &[(0, 3, 4), (1, 8, 8)], AI_RANDOM, 1, (GOAL_REACH, 4, 4));
    // 负控：移动不踏目标格 → 不触发胜负（区分力证明）。
    // 套件不 panic 纪律：规则函数 Err 亦走断言 false 而非 expect。
    let neg = battle::try_move(world, (3, 4), (2, 4));
    let neg_ok = matches!(&neg, Ok(o) if o.winner == -1 && o.phase == PHASE_PLAYER);
    if let Err(e) = neg {
        return vec![tc("reach_goal_wins_on_move", false, format!("负控移动被拒：{e}"))];
    }
    // 负控消耗了本回合移动：复位标记（等价换回合的标记复位）后做正向占点。
    let actor = units_sorted(world)
        .into_iter()
        .find(|v| v.unit.team == 0)
        .map(|v| v.entity);
    if let Some(e) = actor {
        if let Some(mut f) = world.get_mut::<ActionFlags>(e) {
            f.moved = false;
        }
    }
    let o = match battle::try_move(world, (2, 4), (4, 4)) {
        Ok(o) => o,
        Err(e) => {
            return vec![tc("reach_goal_wins_on_move", false, format!("占点合法移动被拒：{e}"))]
        }
    };
    vec![tc(
        "reach_goal_wins_on_move",
        neg_ok && o.winner == 0 && o.phase == PHASE_OVER,
        format!(
            "负控 (3,4)→(2,4) 不触发胜负（winner 仍 -1）+ 正控踏 (4,4) 即 winner={} phase={}（带敌 1 名，胜利只能来自占点分支——TS-14 #10 进程内等价）",
            o.winner, o.phase
        ),
    )]
}

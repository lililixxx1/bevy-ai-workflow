//! [`game.launch_level`]：关卡加载（M4 A 案系统席①；意向文档 §5.3 规划的
//! 游戏专属三方法之一——`docs/evidence/m2-rpc.md`「`launch_level` 未实现」
//! 自本任务落地，T032；T033 扩战斗态）。
//!
//! 请求：`{"level": 1}`；可选 `"seed": <u64>`（战斗种子，默认
//! [`crate::battle::DEFAULT_SEED`]——敌方 AI 与规则面随机的唯一随机源，
//! 重放比对应显式传值锁死基线）。语义（顺序固定）：
//! 1. **清场**——despawn 全部带 [`Unit`] 的实体（重载自洽：先清后生成，
//!    同一关卡重复 launch 得到同一布阵——断言按业务键 `(x, y, team)` 比对，
//!    不依赖实体号，PIT-B-010）；
//! 2. 按 [`LevelDef::units`] 定义顺序生成单位（`GridPos` + `Unit` +
//!    [`crate::battle::ActionFlags`]——定义顺序即生成顺序，确定性）；
//! 3. 写入/覆盖 [`LevelState`] 与 [`crate::battle::BattleState`] 资源
//!    （回合机初值：玩家阶段/回合 1/胜负未定；目标与 AI 档位取自关卡表）；
//! 4. 响应 `{level, name, width, height, units_spawned}`（T032 形态不变，
//!    种子经 `BattleState` 可查，响应不重复携带）。
//!
//! 错误面：缺 params / 缺 level / level 非整数 / 未知关卡号 / 超 u32 范围 /
//! seed 非法（非 u64）→ JSON-RPC `INVALID_PARAMS`（-32602）。handler 全程走
//! `Result` 路径不 panic——备忘录 A 案可玩性定义的「自研 `game.*` 方法错误
//! 路径不击穿进程」验收面。
//!
//! handler 形态（SKILL.md §6.2）：`fn(In(params): In<Option<Value>>,
//! world: &mut World) -> BrpResult`，`with_method_main` 注册、经
//! `world.run_system_with` 独占执行（与 `run_tests` / `screenshot` 同形态）。

use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::{json, Value};

use super::invalid_params;
use crate::battle::{
    ActionFlags, BattleState, AI_GREEDY, AI_RANDOM, DEFAULT_SEED, GOAL_ANNIHILATE, GOAL_REACH,
    PHASE_PLAYER,
};
use crate::level::{level_def, level_ids, GoalDef, GridPos, LevelState, Unit};

/// `game.launch_level` handler 入口（写通路型：语义即加载关卡，非套件——
/// 不适用「无净副作用」约定，副作用（世界布阵变化）即其功能）。
pub fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params =
        params.ok_or_else(|| invalid_params("缺 params（须为 {\"level\": 1}）"))?;
    let raw = params
        .get("level")
        .ok_or_else(|| invalid_params("缺 level 字段（须为 {\"level\": 1}）"))?;
    // u64 直取再验范围：JSON 负数/浮点/字符串都取不到 u64，统一落 -32602。
    let level_id = raw
        .as_u64()
        .ok_or_else(|| invalid_params("level 须为正整数（如 {\"level\": 1}）"))?;
    let level_id = u32::try_from(level_id)
        .map_err(|_| invalid_params(&format!("level 超出 u32 范围：{level_id}")))?;
    // 种子可选：到场则须为 u64（负数/浮点/字符串 → -32602）。
    let seed = match params.get("seed") {
        None => DEFAULT_SEED,
        Some(v) => v
            .as_u64()
            .ok_or_else(|| invalid_params("seed 须为非负整数（u64）"))?,
    };

    let def = level_def(level_id).ok_or_else(|| {
        invalid_params(&format!(
            "未知关卡 {level_id}；可用：{:?}",
            level_ids()
        ))
    })?;

    // 清场：先收集实体号再逐个 despawn（不在迭代中改动世界结构）。
    // `World::despawn` 返回 bool——false 的语义面不止并发（实体不存在/未构造
    // 均为 false，`bevy_ecs-0.19.1/src/world/mod.rs:1570-1580` doc）；本流程
    // 不可达 false 的真实理由：列表取自当次 query 且实体号唯一、当前世界无
    // despawn 钩子/关系级联、BRP 经 `world.run_system_with` 独占执行（无并发
    // 写世界）。返回 bool 签名依据 `world/mod.rs:1598`（T020 核）。
    let old: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, With<Unit>>();
        q.iter(world).collect()
    };
    for entity in old {
        let _ = world.despawn(entity);
    }

    let mut units_spawned = 0u32;
    for u in def.units {
        world.spawn((
            GridPos { x: u.x, y: u.y },
            Unit {
                team: u.team,
                hp: u.hp,
                move_range: u.move_range,
                attack: u.attack,
            },
            ActionFlags::default(),
        ));
        units_spawned += 1;
    }

    world.insert_resource(LevelState {
        level: level_id,
        width: def.width,
        height: def.height,
        units_spawned,
    });

    // 战斗态初值（目标/AI 档位从关卡表带出；RNG 状态=种子，首抽前不推进）。
    let (goal_kind, goal_x, goal_y) = match def.goal {
        GoalDef::Annihilate => (GOAL_ANNIHILATE, -1, -1),
        GoalDef::Reach { x, y } => (GOAL_REACH, x, y),
    };
    // 关卡表是编译期数据：档位值域在此守一道（防未来加表时手误）。
    debug_assert!(def.ai_kind == AI_RANDOM || def.ai_kind == AI_GREEDY);
    world.insert_resource(BattleState {
        phase: PHASE_PLAYER,
        turn: 1,
        winner: -1,
        goal_kind,
        goal_x,
        goal_y,
        seed,
        rng_state: seed,
        ai_kind: def.ai_kind,
    });

    Ok(json!({
        "level": level_id,
        "name": def.name,
        "width": def.width,
        "height": def.height,
        "units_spawned": units_spawned,
    }))
}

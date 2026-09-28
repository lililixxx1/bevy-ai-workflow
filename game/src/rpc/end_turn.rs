//! [`game.end_turn`]：结束玩家回合（M4 A 案系统席④⑥通路，T033）。
//!
//! 请求：params 可省略或为空对象（无必填字段；多余字段忽略——与
//! `launch_level` 只读所需字段的宽容口径一致）。
//!
//! 语义（一次调用内同步结算，见 [`crate::battle::end_turn`]）：复位玩家行动
//! 标记 → 敌方全队按 AI 档位行动（业务键序，`SplitMix64` 种子决策）→ 胜负
//! 判定 → 回到玩家阶段（终局则 `phase=2`）。响应
//! `{turn, enemy_actions: [...], winner, phase}`——`enemy_actions` 逐敌记录
//! 行动（`kind`: attack/move/move_attack/pass + 移动格/目标格/伤害/存活），
//! 是断言面与重放比对的操作序列载体。
//!
//! 拒绝面（-32602，不 panic）：未加载关卡 / 终局。

use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::{json, Value};

use super::invalid_params;
use crate::battle;

/// `game.end_turn` handler 入口（写通路型，同 [`crate::rpc::move_unit`]）。
pub fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    // 无必填字段：params 到场即容忍（含空对象）。
    let _ = params;
    let o = battle::end_turn(world).map_err(|m| invalid_params(&m))?;
    let actions: Vec<Value> = o
        .actions
        .iter()
        .map(|a| {
            json!({
                "enemy_x": a.enemy.0,
                "enemy_y": a.enemy.1,
                "kind": a.kind.as_str(),
                "to_x": a.to.map(|t| t.0),
                "to_y": a.to.map(|t| t.1),
                "target_x": a.target.map(|t| t.0),
                "target_y": a.target.map(|t| t.1),
                "damage": a.damage,
                "target_alive": a.target_alive,
            })
        })
        .collect();
    Ok(json!({
        "turn": o.turn,
        "enemy_actions": actions,
        "winner": o.winner,
        "phase": o.phase,
    }))
}

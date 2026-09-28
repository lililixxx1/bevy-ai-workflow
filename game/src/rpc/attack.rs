//! [`game.attack`]：玩家单位攻击（M4 A 案系统席⑤通路，T033）。
//!
//! 请求：`{"from_x": 1, "from_y": 1, "target_x": 2, "target_y": 1}`。
//!
//! 规则与拒绝面（全部 -32602，不 panic）：未加载关卡 / 终局 / 起点无单位 /
//! 敌方单位 / 本回合已攻击 / 目标格无单位 / 友方目标 / 不贴身（曼哈顿 ≠ 1）。
//! 语义见 [`crate::battle::try_attack`]（整数伤害 = 攻方 `attack`，HP ≤ 0
//! 即 despawn）；成功响应 `{from_x, from_y, target_x, target_y, damage,
//! target_hp, target_alive, winner, phase}`——歼灭型最后一击在此终局。

use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::{json, Value};

use super::{coord_pair, invalid_params};
use crate::battle;

/// `game.attack` handler 入口（写通路型，同 [`crate::rpc::move_unit`]）。
pub fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params = params.ok_or_else(|| {
        invalid_params("缺 params（须为 {\"from_x\":..,\"from_y\":..,\"target_x\":..,\"target_y\":..}）")
    })?;
    let from = coord_pair(&params, "from_x", "from_y")?;
    let target = coord_pair(&params, "target_x", "target_y")?;

    let o = battle::try_attack(world, from, target).map_err(|m| invalid_params(&m))?;
    Ok(json!({
        "from_x": o.from.0,
        "from_y": o.from.1,
        "target_x": o.target.0,
        "target_y": o.target.1,
        "damage": o.damage,
        "target_hp": o.target_hp,
        "target_alive": o.target_alive,
        "winner": o.winner,
        "phase": o.phase,
    }))
}

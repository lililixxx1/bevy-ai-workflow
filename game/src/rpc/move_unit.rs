//! [`game.move_unit`]：玩家单位移动（M4 A 案系统席④⑤通路，T033）。
//!
//! 请求：`{"from_x": 1, "from_y": 1, "to_x": 1, "to_y": 3}`（业务键 `(x, y)`
//! 寻址——不用实体号，规避上游 BRP 坏实体 panic 一类入口风险，见
//! [`crate::battle`] 模块文档）。
//!
//! 规则与拒绝面（全部 -32602，不 panic）：未加载关卡 / 终局 / 起点无单位 /
//! 敌方单位 / 本回合已移动 / 终点越界 / 终点被占用 / 超出移动范围（曼哈顿）。
//! 语义与整数口径见 [`crate::battle::try_move`]；成功响应
//! `{from_x, from_y, to_x, to_y, winner, phase}`（占点型关卡移动即可能终局）。

use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::{json, Value};

use super::{coord_pair, invalid_params};
use crate::battle;

/// `game.move_unit` handler 入口（写通路型：移动即副作用，与 `launch_level`
/// 同类；无净副作用约定只适用只读/恢复型**套件**，见 `suites` 模块文档）。
pub fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params = params.ok_or_else(|| {
        invalid_params("缺 params（须为 {\"from_x\":..,\"from_y\":..,\"to_x\":..,\"to_y\":..}）")
    })?;
    let from = coord_pair(&params, "from_x", "from_y")?;
    let to = coord_pair(&params, "to_x", "to_y")?;

    let o = battle::try_move(world, from, to).map_err(|m| invalid_params(&m))?;
    Ok(json!({
        "from_x": o.from.0,
        "from_y": o.from.1,
        "to_x": o.to.0,
        "to_y": o.to.1,
        "winner": o.winner,
        "phase": o.phase,
    }))
}

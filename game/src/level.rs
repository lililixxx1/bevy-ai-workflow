//! M4 试金石（A 案网格回合制战术小品，`docs/m4-game-selection.md` 定本 v1.0）
//! 关卡域：关卡定义与关卡实体组件（备忘录 §二 A 案系统席①③的最小面）。
//!
//! 定位：[`crate::rpc::launch_level`]（`game.launch_level`）的数据面——**关卡即
//! 数据**（内置常量表；数据文件/资产加载属后续任务，首个任务先以代码内定义
//! 落定格式与语义）。回合机/移动结算/敌方 AI/胜负判定的**规则面**在
//! [`crate::battle`]（系统席④-⑦）；本模块只承载数据：布阵（`units`）、
//! 关卡目标（`goal`）与敌方 AI 档位（`ai_kind`，0 = 种子随机 / 1 = 贪心）。
//!
//! 确定性口径（备忘录 §一）：关卡定义是纯数据、无随机——同一关卡号 → 同一
//! 布阵；本席不涉 RNG（敌方 AI 的种子决策属系统⑥，届时走 `SplitMix64` 整数
//! 取数）。迭代序纪律（PIT-B-010）：一切断言与规则逻辑按业务键 `(x, y, team)`
//! 排序/寻址，不依赖 query 迭代序与实体号。
//!
//! BRP 反射注册（SKILL.md §3.4 强制）：本插件 build 中显式 `register_type`
//! 全部三个 ECS 类型（`GridPos` / `Unit` / `LevelState`）；`LevelDef` /
//! `UnitDef` 是纯数据、不进 ECS，不注册。`team` 口径：`0 = 玩家、1 = 敌方`
//! （u8；enum 形态留待字段语义丰富后再议，避免为反射序列化提前付复杂度）。
//! 数值全整数——备忘录 §一「规则面伤害/结算取整数运算，不引入浮点累积」口径。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 网格坐标（业务键：断言与规则逻辑按 `(x, y)` 排序/寻址——PIT-B-010）。
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Component, Serialize, Deserialize)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

/// 战术单位（备忘录系统席③的最小面；回合机/AI 任务再扩字段）。
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Unit {
    /// 阵营：`0 = 玩家、1 = 敌方`。
    pub team: u8,
    /// 生命值（整数伤害口径——备忘录 §一确定性）。
    pub hp: i32,
    /// 移动范围（格/回合）。
    pub move_range: i32,
    /// 攻击力（整数）。
    pub attack: i32,
}

/// 当前关卡状态（`game.launch_level` 成功后由 handler 写入；BRP 可读——
/// 未加载关卡时资源实例**不存在**：`world.get_resources` 返回 -23501
/// （RESOURCE_ERROR；类型已注册而实例未插入即此路，
/// `bevy_remote-0.19.1/src/builtin_methods.rs:617-637`），**并非 null**——
/// 「null」口径仅是 `run_tests` 报文内可选字段的约定，两条通路语义不同。
/// 工具端读取按「缺失/报错」处理，不自造默认值（2026-09-28 审核活体探针实测）。
#[derive(Resource, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct LevelState {
    pub level: u32,
    pub width: u32,
    pub height: u32,
    pub units_spawned: u32,
}

/// 关卡定义中的单个单位（纯数据，不进 ECS）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitDef {
    pub team: u8,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub move_range: i32,
    pub attack: i32,
}

/// 关卡目标（纯数据；数值口径与 [`crate::battle`] 常量对应：
/// `Annihilate` = 0 歼灭全部敌方，`Reach` = 1 任一玩家单位到达目标格）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalDef {
    Annihilate,
    Reach { x: i32, y: i32 },
}

/// 关卡定义（纯数据，不进 ECS；`units` 定义顺序即生成顺序——确定性布阵；
/// `ai_kind` 选敌方 AI：`0` = 种子随机 / `1` = 贪心规则，见 [`crate::battle`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelDef {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub goal: GoalDef,
    pub ai_kind: u8,
    pub units: &'static [UnitDef],
}

/// 关卡 1「first-contact」：9×9，玩家 3 单位对敌方 3 单位（敌方 AI = 种子随机）。
static LEVEL_1: LevelDef = LevelDef {
    name: "first-contact",
    width: 9,
    height: 9,
    goal: GoalDef::Annihilate,
    ai_kind: 0,
    units: &[
        UnitDef { team: 0, x: 1, y: 1, hp: 10, move_range: 3, attack: 2 },
        UnitDef { team: 0, x: 2, y: 1, hp: 10, move_range: 3, attack: 2 },
        UnitDef { team: 0, x: 3, y: 1, hp: 10, move_range: 3, attack: 2 },
        UnitDef { team: 1, x: 5, y: 7, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 6, y: 7, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 7, y: 7, hp: 8, move_range: 2, attack: 2 },
    ],
};

/// 关卡 2「pincer」：9×9，玩家 4 单位对敌方 5 单位（敌方 AI = 贪心规则）。
static LEVEL_2: LevelDef = LevelDef {
    name: "pincer",
    width: 9,
    height: 9,
    goal: GoalDef::Annihilate,
    ai_kind: 1,
    units: &[
        UnitDef { team: 0, x: 0, y: 0, hp: 10, move_range: 3, attack: 3 },
        UnitDef { team: 0, x: 1, y: 0, hp: 10, move_range: 3, attack: 3 },
        UnitDef { team: 0, x: 2, y: 0, hp: 10, move_range: 3, attack: 3 },
        UnitDef { team: 0, x: 3, y: 0, hp: 10, move_range: 3, attack: 3 },
        UnitDef { team: 1, x: 4, y: 8, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 5, y: 8, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 6, y: 8, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 7, y: 8, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 8, y: 8, hp: 8, move_range: 2, attack: 2 },
    ],
};

/// 关卡 3「breakthrough」：9×9，占点型——任一玩家单位到达 `(8, 4)` 即胜
/// （敌方 AI = 贪心规则，5 单位守右翼；玩家 3 单位移速 4 突破）。
static LEVEL_3: LevelDef = LevelDef {
    name: "breakthrough",
    width: 9,
    height: 9,
    goal: GoalDef::Reach { x: 8, y: 4 },
    ai_kind: 1,
    units: &[
        UnitDef { team: 0, x: 0, y: 3, hp: 10, move_range: 4, attack: 3 },
        UnitDef { team: 0, x: 0, y: 4, hp: 10, move_range: 4, attack: 3 },
        UnitDef { team: 0, x: 0, y: 5, hp: 10, move_range: 4, attack: 3 },
        UnitDef { team: 1, x: 6, y: 2, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 6, y: 6, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 7, y: 4, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 8, y: 3, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 8, y: 5, hp: 8, move_range: 2, attack: 2 },
        UnitDef { team: 1, x: 8, y: 8, hp: 8, move_range: 2, attack: 2 },
    ],
};

/// 内置关卡表（关卡号 → 定义；编号从 1 起）。
pub fn level_def(id: u32) -> Option<&'static LevelDef> {
    match id {
        1 => Some(&LEVEL_1),
        2 => Some(&LEVEL_2),
        3 => Some(&LEVEL_3),
        _ => None,
    }
}

/// 可用关卡号清单（错误 message 用，升序）。
pub fn level_ids() -> &'static [u32] {
    &[1, 2, 3]
}

/// 关卡域插件：本席只做类型注册（实体生成由 `game.launch_level` 驱动，
/// 启动时无默认关卡——demo 的 wanderer 群仍按 `--count` 独立生成，两域互不感知）。
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GridPos>()
            .register_type::<Unit>()
            .register_type::<LevelState>();
    }
}

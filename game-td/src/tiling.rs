//! Auto-tiling 层（教程 05 Step 2）：邻位规则书与解析。
//!
//! 从 P4 的 `main.rs` **原样搬迁**（教程 05「moved verbatim; no behavior changed」）：
//! `Match` / `Pattern` / `TileRule` / `TileTypeRuleset` / `TileRules` 与
//! `pat` / `rule` / `build_rules`。唯一改动 = 可见性（跨模块面 `pub`，
//! 仅 `build_rules` 谓词内部使用的 `pat` / `rule` 保持私有，教程 05 Step 2 口径）。
//!
//! 依赖方向单向（教程 05）：`tiling` → `map`。

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::map::{MapLayout, TileType};

/// 单邻位匹配要求（教程 Part 4 Step 3）：语义**相对中心瓦片**。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Match {
    /// 邻位须与中心同类。
    Same,
    /// 邻位须与中心异类；越界同样算异类。
    Diff,
    /// 不设要求。
    Any,
}

/// 八向邻位采样偏移，序与 [`Pattern`] 的数组槽位一一对应。
///
/// 序 = [N, NE, E, SE, S, SW, W, NW]；Bevy 2D 为 Y-up，故 N = `(0, +1)`。
const NEIGHBORS: [(i32, i32); 8] = [
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
];

/// 八向邻位要求组合（教程 Part 4 Step 3 的 `NeighborPattern` 行为）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pattern([Match; 8]);

impl Pattern {
    /// 按 [`NEIGHBORS`] 的序逐位采样并判定（任一槽位不满足即整条不匹配）。
    fn matches(&self, center: TileType, pos: TilePos, map: &MapLayout) -> bool {
        NEIGHBORS
            .iter()
            .zip(self.0)
            .all(|(&(dx, dy), want)| match want {
                Match::Any => true,
                Match::Same => map.get(pos.x as i32 + dx, pos.y as i32 + dy) == Some(center),
                Match::Diff => map.get(pos.x as i32 + dx, pos.y as i32 + dy) != Some(center),
            })
    }
}

/// 单条规则：一套邻位要求 → 一个图集索引（教程 Part 4 Step 3 的 `TileRule`）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TileRule {
    pattern: Pattern,
    atlas_index: u32,
}

/// 单类瓦片的规则集（教程 Part 4 Step 3 的 `TileTypeRuleset` 行为）。
///
/// `rules` 按序**首次命中即用**；全不命中落 `fallback`——规则书不完整时仍出合法瓦片。
#[derive(Clone, Debug)]
pub struct TileTypeRuleset {
    rules: Vec<TileRule>,
    fallback: u32,
}

/// 规则书资源（教程 Part 4 Step 3）：按逻辑类型分表。
#[derive(Resource, Default, Debug)]
pub struct TileRules {
    rulesets: HashMap<TileType, TileTypeRuleset>,
}

impl TileRules {
    pub fn add(&mut self, tile_type: TileType, ruleset: TileTypeRuleset) {
        self.rulesets.insert(tile_type, ruleset);
    }

    /// 解析某格瓦片的图集索引；该类无规则集即 panic（规则书配置错误要显式炸）。
    pub fn resolve(&self, tile_type: TileType, pos: TilePos, map: &MapLayout) -> u32 {
        let ruleset = self
            .rulesets
            .get(&tile_type)
            .unwrap_or_else(|| panic!("no ruleset for {tile_type:?}"));
        ruleset
            .rules
            .iter()
            .find(|rule| rule.pattern.matches(tile_type, pos, map))
            .map_or(ruleset.fallback, |rule| rule.atlas_index)
    }
}

/// 便捷构造：`Pattern` 的八槽字面量（序 = [N, NE, E, SE, S, SW, W, NW]）。
const fn pat(values: [Match; 8]) -> Pattern {
    Pattern(values)
}

/// 便捷构造：单条规则。
const fn rule(pattern: Pattern, atlas_index: u32) -> TileRule {
    TileRule {
        pattern,
        atlas_index,
    }
}

/// 规则书构造（教程 Part 4 Step 3 行为规格 + 图集索引表）。
///
/// 索引语义（Kenney 图集，与 P2/P3 的硬编码集合逐一对应）：
/// 79/81/125/127 = 四角；80/126/102/104 = 四边；103 = 路面主体；129 = 草地。
///
/// 顺序纪律：角规则在边规则之前、边规则在 fallback 之前——首次命中即用。
pub fn build_rules() -> TileRules {
    use Match::{Any, Diff, Same};

    let mut rules = TileRules::default();

    rules.add(
        TileType::Path,
        TileTypeRuleset {
            rules: vec![
                // 内角（教程 06 Step 5）：四正位全是道路、恰有一条对角是草地。
                // 教程原文槽位序 = `pat(n, s, e, w, ne, nw, se, sw)`；本工程槽位序
                // = [N, NE, E, SE, S, SW, W, NW]（见 NEIGHBORS），逐条按名对齐后的落位：
                // 教程 82「upper-left」= 教程第 7 槽 se 为 diff -> 本序 SE 位 diff；
                // 教程 83「upper-right」= 教程第 8 槽 sw 为 diff -> 本序 SW 位 diff；
                // 教程 105「bottom-left」= 教程第 5 槽 ne 为 diff -> 本序 NE 位 diff；
                // 教程 106「bottom-right」= 教程第 6 槽 nw 为 diff -> 本序 NW 位 diff。
                // 内角须排在 fallback（103 路面主体）之前、且先于外角——首次命中即用。
                rule(pat([Same, Same, Same, Diff, Same, Same, Same, Same]), 82), // SE 缺角
                rule(pat([Same, Same, Same, Same, Same, Diff, Same, Same]), 83), // SW 缺角
                rule(pat([Same, Diff, Same, Same, Same, Same, Same, Same]), 105), // NE 缺角
                rule(pat([Same, Same, Same, Same, Same, Same, Same, Diff]), 106), // NW 缺角
                // 外角：两条垂直边是草地、两条平行边是道路。
                // 左上（北邻与西邻为草地）。
                rule(pat([Diff, Any, Same, Any, Same, Any, Diff, Diff]), 79),
                // 右上（北邻与东邻为草地）。
                rule(pat([Diff, Diff, Diff, Any, Same, Any, Same, Any]), 81),
                // 左下（南邻与西邻为草地）。
                rule(pat([Same, Any, Same, Any, Diff, Diff, Diff, Any]), 125),
                // 右下（南邻与东邻为草地）。
                rule(pat([Same, Any, Diff, Diff, Diff, Any, Same, Any]), 127),
                // 边：一条边是草地、其余三向是道路。
                rule(pat([Diff, Any, Same, Any, Same, Any, Same, Any]), 80), // 上沿
                rule(pat([Same, Any, Same, Any, Diff, Any, Same, Any]), 126), // 下沿
                rule(pat([Same, Any, Same, Any, Same, Any, Diff, Any]), 102), // 左端
                rule(pat([Same, Any, Diff, Any, Same, Any, Same, Any]), 104), // 右端
            ],
            fallback: 103,
        },
    );

    rules.add(
        TileType::Grass,
        TileTypeRuleset {
            rules: Vec::new(),
            fallback: 129,
        },
    );

    rules
}

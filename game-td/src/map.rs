//! 地图数据层（教程 05 Step 1）：逻辑网格与瓦片类型。
//!
//! 纯数据、无插件无系统（教程 05「`map.rs` 是 plain data」）：不知道渲染、auto-tiling、
//! 关卡加载的存在。依赖方向单向（教程 05「Dependency Direction」）：
//! `tiling` → `map`，`main` → `map`；本模块不 `use crate::tiling`。

use bevy::prelude::*;

/// 逻辑瓦片类型（教程 Part 4 Step 1）：玩法系统查询它，而不是图集索引。
///
/// derive 形态按 SKILL.md §3.4（BRP 查询要求 Reflect + `#[reflect(Component)]`）；
/// `Hash` 供规则书按类型建表（`HashMap<TileType, …>`）。
/// 反射注册在 `crate::brp::BrpPlugin::build`（`bevy_app-0.20.0/src/app.rs:664`）。
///
/// `#[type_path = "game_td"]`：模块化后冻结反射类型路径为 crate 根形态
/// （`game_td::TileType`）——BRP `world.query` 等方法按**完整类型路径**查注册表
/// （`bevy_remote-0.20.0` 多处经 `get_with_type_path` 解析，调用点散布
/// `builtin_methods.rs` 各 handler——如 `:1183`/`:1915` 等，非查询单点专属），
/// 不冻结则路径漂移为 `game_td::map::TileType`，证据脚本的组件名与网格指纹口径随之破裂。
/// 属性语义见 `bevy_reflect_derive-0.20.0/src/derive_data.rs:211`（`#[type_path = "..."]`）。
#[derive(Component, Reflect, Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[type_path = "game_td"]
#[reflect(Component)]
pub enum TileType {
    Grass,
    Path,
}

/// 网格地图瓦片标记（教程 Part 2 Step 4）：所有 150 格都挂。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub struct MapTile;

/// 道路瓦片标记（教程 Part 2 Step 4）：仅 `TileType::Path` 的格子加挂。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub struct PathTile;

/// 关卡逻辑网格（教程 Part 4 Step 2）：关卡设计的一维平铺真值。
///
/// 瓦片实体只是本数据的**视图**；auto-tiling 只读这里，不再看坐标。
#[derive(Resource, Debug, Clone)]
pub struct MapLayout {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<TileType>,
}

impl MapLayout {
    /// 网格取值（一维索引 = `y * width + x`）；越界返回 `None`。
    ///
    /// 越界 = 「与中心不同类」的天然来源：规则书不必为地图边界写特例
    /// （教程 Part 4 Step 2 的 `get` 语义）。坐标取 `i32` 是相对教程 `u32` 签名的
    /// 等价强化：邻位采样会算出负坐标，`u32` 版需在调用侧额外判负才能表达越界。
    pub fn get(&self, x: i32, y: i32) -> Option<TileType> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        Some(self.tiles[(y as u32 * self.width + x as u32) as usize])
    }
}

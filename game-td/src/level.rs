//! 关卡数据层（教程 06 Step 3–4）：TOML 外部关卡的加载与 `MapLayout` 派生。
//!
//! 数据驱动边界（教程 06「Data-Driven Design」）：游戏逻辑留在 Rust，关卡内容
//! 进 TOML——代码决定*怎么*运作，数据决定*有什么*。地图几何逐关变化，故外置。
//!
//! 格式（教程 06 Step 2）：
//! - `[map]`：`width` / `height`；
//! - `[paths.<id>]`：`waypoints = [[x, y], …]`——**宽 1 中心线**，加载时展开为
//!   宽 3 道路；航点存原始数值供玩法系统（P7 敌人寻路）直接读取；
//! - `waves` 可选（`#[serde(default)]`），留给后续波次系统。
//!
//! 失败即 panic（教程 06「Simple and strict」）：文件缺失 / 解析失败 / 对角线段 /
//! 航点不足 2 个都立刻炸——fail-fast 优于静默跳过。

use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::map::{MapLayout, TileType};

/// 关卡真值（教程 06 Step 3 + 10 Step 2）：与 TOML 形状同构，直接反序列化。
///
/// 作为资源常驻 ECS：后续系统（敌人 / 波次）不必从瓦片网格反推航点。
#[derive(Debug, Deserialize, Resource)]
pub struct LevelData {
    pub map: MapData,
    pub paths: HashMap<String, PathData>,
    /// 敌型表（教程 10 Step 2）：`[enemy_types.<id>]`，缺省为空表（旧关卡向后兼容）。
    #[serde(default)]
    pub enemy_types: HashMap<String, EnemyTypeDef>,
    /// 波次（教程 10 Step 2）：`[[waves]]`；缺省为空（`#[serde(default)]` = 旧关卡可解析）。
    #[serde(default)]
    pub waves: Vec<WaveDef>,
}

/// `[map]` 段。
#[derive(Debug, Deserialize)]
pub struct MapData {
    pub width: u32,
    pub height: u32,
}

/// `[paths.<id>]` 段：宽 1 中心线航点（`[x, y]` 网格坐标）。
#[derive(Debug, Deserialize, Clone)]
pub struct PathData {
    pub waypoints: Vec<[u32; 2]>,
}

/// `[enemy_types.<id>]` 段（教程 10 Step 2）：一种敌人的外观与数值。
#[derive(Debug, Deserialize, Clone)]
pub struct EnemyTypeDef {
    /// 图集索引（sprite = 245 等四个士兵变体）。
    pub sprite: usize,
    /// 世界单位 / 秒。
    pub speed: f32,
    /// 初始生命值。
    pub health: f32,
    /// 击杀赏金（教程 14 Step 1）：`#[serde(default)]` = 旧关卡不带该字段时归零
    /// （教程 14 Step 1 注：向后兼容但静默漏配危险——本工程三关已全量补 `bounty = 25`）。
    #[serde(default)]
    pub bounty: u32,
}

/// `[[waves]]` 段（教程 10 Step 2）：一组按 `spawn_interval` 落地的敌人。
#[derive(Debug, Deserialize)]
pub struct WaveDef {
    /// 相对关卡开始的秒数。
    pub start_time: f32,
    /// 走哪条路径（`[paths.<id>]` 的键）。
    pub path: String,
    /// 波内单敌间隔（秒）；缺省 1.0（教程 10 Step 2 `default_spawn_interval`）。
    #[serde(default = "default_spawn_interval")]
    pub spawn_interval: f32,
    /// 有序敌人组：先组 1 打满，再组 2，依此类推。
    pub enemies: Vec<WaveEnemyGroup>,
}

/// 缺省波内间隔（教程 10 Step 2）。
fn default_spawn_interval() -> f32 {
    1.0
}

/// 单个敌人组（教程 10 Step 2）：`{ type = "soldier", count = 3 }`。
#[derive(Debug, Deserialize)]
pub struct WaveEnemyGroup {
    /// `type` 是 Rust 关键字，serde 侧改名（教程 10 Step 2）。
    #[serde(rename = "type")]
    pub enemy_type: String,
    pub count: u32,
}

/// 读取 TOML 关卡（教程 06 Step 4 `load_level`）：缺失 / 解析失败即 panic。
pub fn load_level(path: &str) -> LevelData {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));
    toml::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse {}: {}", path, e))
}

/// 由关卡数据派生逻辑网格（教程 06 Step 4 `build_map_from_level`）。
///
/// 宽 1 中心线 → 宽 3 道路的展开规则（教程 06 Step 2/4）：
/// - 水平段：中心行 + 上下各一行；垂直段：中心列 + 左右各一列；
/// - 越界格静默跳过（航点可靠边）；重叠段幂等（重复写 `Path`）；
/// - 对角线段 panic（只支持轴对齐，教程 06「Axis-aligned only」）。
///
/// 第二段（拐角对角补齐）：两段宽 3 道路在拐点只形成 L 形，缺「外侧」那一格；
/// 逐内点补 `(x2 + mx, y2 + my)`（`mx` 取水平段方向、`my` 取垂直段方向的反号，
/// 教程 06 Step 4 原式），使拐弯成为完整 3×3 块。
pub fn build_map_from_level(level: &LevelData) -> MapLayout {
    let width = level.map.width;
    let height = level.map.height;
    let mut tiles = vec![TileType::Grass; (width * height) as usize];

    for (id, path) in &level.paths {
        if path.waypoints.len() < 2 {
            panic!("Path '{}' must have at least 2 waypoints", id);
        }
        for window in path.waypoints.windows(2) {
            let [x1, y1] = window[0];
            let [x2, y2] = window[1];

            if x1 == x2 {
                // Vertical segment: center column + one on each side.
                let y_start = y1.min(y2);
                let y_end = y1.max(y2);
                for y in y_start..=y_end {
                    for dx in -1i32..=1i32 {
                        let nx = x1 as i32 + dx;
                        if nx >= 0 && nx < width as i32 {
                            let idx = (y * width + nx as u32) as usize;
                            tiles[idx] = TileType::Path;
                        }
                    }
                }
            } else if y1 == y2 {
                // Horizontal segment: center row + one above and below.
                let x_start = x1.min(x2);
                let x_end = x1.max(x2);
                for x in x_start..=x_end {
                    for dy in -1i32..=1i32 {
                        let ny = y1 as i32 + dy;
                        if ny >= 0 && ny < height as i32 {
                            let idx = (ny as u32 * width + x) as usize;
                            tiles[idx] = TileType::Path;
                        }
                    }
                }
            } else {
                panic!(
                    "Path '{}' has a diagonal segment: ({},{}) -> ({},{}). \
                     Only axis-aligned segments are supported.",
                    id, x1, y1, x2, y2
                );
            }
        }
    }

    // Fill diagonal tiles at path corners so turns form complete 3×3 blocks.
    for (_id, path) in &level.paths {
        let wps = &path.waypoints;
        for i in 1..wps.len() - 1 {
            let [x1, y1] = wps[i - 1];
            let [x2, y2] = wps[i];
            let [x3, y3] = wps[i + 1];

            let dx1 = x2 as i32 - x1 as i32;
            let dy1 = y2 as i32 - y1 as i32;
            let dx2 = x3 as i32 - x2 as i32;
            let dy2 = y3 as i32 - y2 as i32;

            // The horizontal segment determines the missing x offset,
            // and the vertical segment determines the missing y offset.
            let mx = if dy1 == 0 { dx1.signum() } else { -dx2.signum() };
            let my = if dx1 == 0 { dy1.signum() } else { -dy2.signum() };

            let cx = x2 as i32 + mx;
            let cy = y2 as i32 + my;
            if cx >= 0 && cx < width as i32 && cy >= 0 && cy < height as i32 {
                let idx = (cy as u32 * width + cx as u32) as usize;
                tiles[idx] = TileType::Path;
            }
        }
    }

    MapLayout {
        width,
        height,
        tiles,
    }
}

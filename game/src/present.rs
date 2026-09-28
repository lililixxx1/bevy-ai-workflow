//! M4 表现层（A 案系统席⑧，T036）——**只读规则面，不作验收面**
//!（备忘录 §二 A 案口径：高亮/血条/动画等表现不进验收；本模块零规则副作用，
//! 规则态仍以 `game.*` BRP 断言为准）。
//!
//! 构成（最小面）：
//! - 棋盘格：`LevelState` 变更（`launch_level` 重载即 `insert_resource` → 资源
//!   change-detection 触发）时重建——`width×height` 交替双色地砖，占点型关卡
//!   （`BattleState.goal_kind == GOAL_REACH`）的目标格铺绿砖；同帧把
//!   [`crate::camera::CameraRig`] 收拢到棋盘视角（仅当关卡已加载；无关卡时
//!   demo 视角不动）；
//! - 单位棋子：独立表现实体（[`GridPiece`] 标记 + 指回规则实体），每帧对账
//!   （reconcile）——阵亡单位的棋子 despawn、新单位补棋子、位置随 `GridPos`
//!   摆放；棋子**初始**高度按入场 HP 定型（受伤不回缩——表现简化，非血条
//!   语义，R1 审核 S4 口径修正）。**规则实体不携带任何表现组件**（分层：
//!   规则面组件 = `GridPos`/`Unit`/`ActionFlags`，表现挂在棋子上）。
//!
//! API 依据：Mesh3d/MeshMaterial3d/材质形态与 `sim.rs` 同源
//!（`bevy-0.19.1/examples/remote/server.rs:32-44`，仓库 T003 起已验证）；
//! 资源 change-detection `Res::is_changed`（`Res` 实现 `DetectChanges`，
//! SKILL.md §6.10 域）。表现实体无反射注册需求（BRP 不消费）。

use std::collections::HashMap;

use bevy::prelude::*;

use crate::battle::{ActionFlags, BattleState, GOAL_REACH};
use crate::camera::CameraRig;
use crate::level::{GridPos, LevelState, Unit};

/// 表现实体标记（棋盘地砖与单位棋子共用：重载重建/对账清理的锚点）。
/// `unit = Some(规则实体)` 为单位棋子；`None` 为地砖。
#[derive(Component, Debug)]
pub struct GridPiece {
    pub unit: Option<Entity>,
}

/// 一次重建周期内共享的棋子材质句柄（普通资源，无反射注册需求；
/// 地砖/目标格材质只在 sync_board 内当次使用，不入资源）。
#[derive(Resource, Debug)]
struct PresentMaterials {
    team0: Handle<StandardMaterial>,
    team1: Handle<StandardMaterial>,
}

/// 表现层插件（席⑧）：三系统链式（重建 → 对账 → 摆放），均只读规则组件/资源。
pub struct PresentPlugin;

impl Plugin for PresentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (sync_board, sync_pieces, place_pieces).chain());
    }
}

/// 格坐标 → 世界坐标（棋盘居中于原点，格距 1——相机轨道即绕原点）。
fn world_pos(x: i32, y: i32, width: u32, height: u32) -> Vec3 {
    Vec3::new(
        x as f32 - (width - 1) as f32 / 2.0,
        0.0,
        y as f32 - (height - 1) as f32 / 2.0,
    )
}

/// `LevelState` 变更时重建棋盘与视角（首载与每次 `launch_level` 重载；
/// 地砖与棋子全部推倒重来——数量 ≤ 81+9，简单优先于增量）。
fn sync_board(
    level: Option<Res<LevelState>>,
    battle: Option<Res<BattleState>>,
    mut commands: Commands,
    old: Query<(Entity, &GridPiece)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rig: Option<ResMut<CameraRig>>,
) {
    let Some(level) = level else { return };
    if !level.is_changed() {
        return;
    }
    for (entity, _) in old.iter() {
        if let Ok(mut e) = commands.get_entity(entity) {
            e.despawn();
        }
    }
    let goal_mat = materials.add(Color::srgb_u8(60, 150, 80));
    let team0_mat = materials.add(Color::srgb_u8(90, 140, 255));
    let team1_mat = materials.add(Color::srgb_u8(230, 90, 90));
    commands.insert_resource(PresentMaterials {
        team0: team0_mat,
        team1: team1_mat,
    });

    // 占点目标格（goal 坐标在 BattleState；无战斗态则无高亮）。
    let goal_cell: Option<(i32, i32)> = battle
        .as_ref()
        .filter(|b| b.goal_kind == GOAL_REACH)
        .map(|b| (b.goal_x, b.goal_y));

    let tile_mesh = meshes.add(Cuboid::new(0.92, 0.08, 0.92));
    let tile_a = materials.add(Color::srgb_u8(52, 56, 70));
    let tile_b = materials.add(Color::srgb_u8(40, 43, 54));
    for y in 0..level.height as i32 {
        for x in 0..level.width as i32 {
            let mat = if goal_cell == Some((x, y)) {
                goal_mat.clone()
            } else if (x + y) % 2 == 0 {
                tile_a.clone()
            } else {
                tile_b.clone()
            };
            let pos = world_pos(x, y, level.width, level.height);
            commands.spawn((
                GridPiece { unit: None },
                Mesh3d(tile_mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_translation(pos),
            ));
        }
    }
    if let Some(rig) = rig.as_mut() {
        // 9×9 棋盘对角半径 ~6.4；收拢到 14/13 全盘入画（视角表现参数，无规则语义）。
        rig.radius = 14.0;
        rig.height = 13.0;
    }
}

/// 每帧对账：阵亡单位的棋子 despawn、无棋子的单位补棋子（高度随 HP 缩放）。
fn sync_pieces(
    units: Query<(Entity, &GridPos, &Unit), With<ActionFlags>>,
    pieces: Query<(Entity, &GridPiece)>,
    mut commands: Commands,
    assets: Option<Res<PresentMaterials>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(assets) = assets else { return };
    let mut piece_of: HashMap<Entity, Entity> = HashMap::new();
    let mut dead: Vec<Entity> = Vec::new();
    for (entity, piece) in pieces.iter() {
        match piece.unit {
            Some(unit) => {
                if units.get(unit).is_err() {
                    dead.push(entity);
                } else {
                    piece_of.insert(unit, entity);
                }
            }
            None => {} // 地砖由 sync_board 管理
        }
    }
    for entity in dead {
        if let Ok(mut e) = commands.get_entity(entity) {
            e.despawn();
        }
    }
    for (unit_entity, _pos, unit) in units.iter() {
        if piece_of.contains_key(&unit_entity) {
            continue;
        }
        let material = if unit.team == 0 {
            assets.team0.clone()
        } else {
            assets.team1.clone()
        };
        // 初始高度按入场 HP：满血 0.75、空血 0.30（仅补棋时定型，受伤不回缩——
        // 表现简化非血条语义，R1 审核 S4；10 为玩家满血口径）。
        let hp_scale = (unit.hp.clamp(0, 10) as f32) / 10.0;
        commands.spawn((
            GridPiece { unit: Some(unit_entity) },
            Mesh3d(meshes.add(Cuboid::new(0.55, 0.30 + 0.45 * hp_scale, 0.55))),
            MeshMaterial3d(material),
        ));
    }
}

/// 每帧摆放：棋子 Transform 跟随规则实体 `GridPos`（表现只读规则）。
fn place_pieces(
    units: Query<&GridPos, With<ActionFlags>>,
    mut pieces: Query<(&GridPiece, &mut Transform)>,
    level: Option<Res<LevelState>>,
) {
    let Some(level) = level else { return };
    for (piece, mut transform) in pieces.iter_mut() {
        if let Some(unit_entity) = piece.unit {
            if let Ok(pos) = units.get(unit_entity) {
                let p = world_pos(pos.x, pos.y, level.width, level.height);
                transform.translation.x = p.x;
                transform.translation.z = p.z;
                transform.translation.y = 0.3;
            }
        }
    }
}

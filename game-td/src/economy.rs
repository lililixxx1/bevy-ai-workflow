//! 经济层（教程 14）：金币资源、被动收入、击杀赏金、放置成本与 HUD。
//!
//! 0.20 锚点（漂移表 Part 14，全部摩擦级 0）：
//! - `Option<&T>` 查询（`bevy_ecs-0.20.0/src/query/fetch.rs:3318`）——预览幽灵三态着色；
//! - `Text` 整组件覆写（`bevy_ui-0.20.0/src/widget/text.rs:128`（`Text(pub String)`）/`:132`）；
//! - `PositionType::Absolute`（`bevy_ui-0.20.0/src/ui_node.rs:508` 字段 / `:1763` 枚举）；
//! - `Node` 的 `top` / `left` 边距字段（`ui_node.rs:528` / `:542`）。
//!
//! 设计（教程 14 Walkthrough）：
//! - [`Gold`] 用 `f32`：被动收入每固定步加 `3.0 × dt`，`u32` 会丢零头；
//!   显示时 `as u32` 向零截断即天然取整（教程 14「`as u32` truncation for display」）；
//! - [`Bounty`] 随敌人实体走：TOML `bounty` → `EnemyTypeDef` → `SpawnEvent` → 组件，
//!   击杀时由 `attack_enemies` 读（教程 14 Step 2/8）；
//! - [`PlacementDenied`] 是**瞬时组件**：点击但买不起时挂到预览幽灵、计时到期移除
//!   （教程 14「Transient components」）；`update_placement_preview` 以
//!   `Option<&PlacementDenied>` 判定红/绿/白三态（教程 14 Step 6）；
//! - 仿真系统（被动收入）挂 `FixedUpdate`，表现系统（HUD / 红闪计时）挂 `Update`
//!   ——收入随模拟时间缩放，不受渲染帧率影响（教程 14 Step 10 注）。

use bevy::prelude::*;

use crate::GameEntity;

/// 起始金币（教程 14 Step 3）。
pub const STARTING_GOLD: f32 = 300.0;

/// 被动收入速率（教程 14 Step 3：3 金 / 秒）。
pub const PASSIVE_INCOME_RATE: f32 = 3.0;

/// 建造价已随 P16 数据驱动化迁入 `assets/towers.toml` 的 `cost` 字段
/// （教程 16「Cleaning up」：这是最后一个被删除的硬编码塔常量）。

/// 买不起时的红闪时长（教程 14 Step 3：0.3s）。
pub const DENIED_FLASH_DURATION: f32 = 0.3;

/// 玩家余额（教程 14 Step 3）：`f32` 累积被动收入，显示处取整。
#[derive(Resource)]
pub struct Gold(pub f32);

/// HUD 文本标记（教程 14 Step 3）：`update_gold_hud` 按它定位。
///
/// BRP 面（本工程扩展层）：标记组件反射注册后 `world.query` 可直接计数，
/// 用于「HUD 恰存在一份」的侧信道断言（`Text` 内容本身不在 BRP 反射面，
/// 见 M6 选品 §五 U6 约束）。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub struct GoldHud;

/// 击杀赏金（教程 14 Step 3）：随敌人实体走，死亡时入账。
///
/// BRP 面：反射注册供 `world.query` 读值（元组结构体序列化形态 = `[25]`）。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub struct Bounty(pub u32);

/// 买不起的红闪（教程 14 Step 3，瞬时组件）：`Update` 内计时，到期移除。
#[derive(Component)]
pub struct PlacementDenied(pub Timer);

/// 生成 HUD（教程 14 Step 3 `spawn_gold_hud`）：左上角 "Gold: N"。
///
/// 挂 `GameEntity` → `cleanup_level` 自动回收；在 `OnEnter(InGame)` 链中排在
/// `load_level_data` **之后**（读它插入的 `Gold`，教程 14 Step 10 注）。
pub fn spawn_gold_hud(mut commands: Commands, gold: Res<Gold>) {
    commands.spawn((
        GoldHud,
        GameEntity,
        Text::new(format!("Gold: {}", gold.0 as u32)),
        TextFont {
            font_size: 32.0.into(),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));
}

/// 每帧刷新 HUD（教程 14 Step 3 `update_gold_hud`）：
/// 挂 `Update` 使赏金到账后立刻可见，不等下一固定步。
pub fn update_gold_hud(gold: Res<Gold>, mut query: Query<&mut Text, With<GoldHud>>) {
    let display = gold.0 as u32;
    for mut text in query.iter_mut() {
        *text = Text::new(format!("Gold: {}", display));
    }
}

/// 被动收入（教程 14 Step 3 `earn_passive_income`）：每固定步 `3.0 × Δt`。
pub fn earn_passive_income(time: Res<Time>, mut gold: ResMut<Gold>) {
    gold.0 += PASSIVE_INCOME_RATE * time.delta_secs();
}

/// 红闪到期移除（教程 14 Step 3 `tick_placement_denied`）：瞬时组件的回收端。
pub fn tick_placement_denied(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PlacementDenied)>,
) {
    for (entity, mut denied) in query.iter_mut() {
        denied.0.tick(time.delta());
        if denied.0.just_finished() {
            commands.entity(entity).remove::<PlacementDenied>();
        }
    }
}

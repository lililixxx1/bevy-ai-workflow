//! LevelSelect 屏（教程 12 Step 8）：全屏居中提示 + `1` 键进入关卡。
//!
//! 0.20 锚点（漂移表 Part 12）：
//! - `Node` 布局组件与 `AlignItems` / `JustifyContent`（`bevy_ui-0.20.0/src/ui_node.rs:485`/`:900`/`:1358`）；
//! - `Val::Percent`（`bevy_ui-0.20.0/src/geometry.rs:32`）；
//! - `Text`（`bevy_ui-0.20.0/src/widget/text.rs:128`）、`TextColor`（`bevy_text-0.20.0/src/text.rs:1405`）；
//! - `TextFont.font_size` 是 `FontSize` 枚举（`bevy_text-0.20.0/src/text.rs:688`/`:783`），
//!   `From<f32> for FontSize` = `FontSize::Px`（`:861-865`）——0.18 的裸 `f32` 写法
//!   在 0.20 需一次转换（漂移表 Part 12 摩擦级 1）。
//!
//! `ScreenUi` 只挂根节点：`cleanup_screen_ui` 逐实体 despawn，父节点递归带走子文本
//! （教程 12 Step 8 注）。`ScreenUi` 不进 `GameEntity` 面——它不属于关卡世界。

use bevy::prelude::*;

use crate::{GameState, ScreenUi};

/// 生成标题屏（教程 12 Step 8 `setup_level_select`）：全屏居中节点 + 一行提示。
pub fn setup_level_select(mut commands: Commands) {
    commands
        .spawn((
            ScreenUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                // 教程 12 Step 8 文案 "Press [1] — Level 1" 的 em dash 在默认内置字体
                // 无字形（渲染成豆腐块，见 p12-shot-menu.png 首轮留档），此处用 ASCII
                // 连字符——UI 文案不在行为规格断言面内。
                Text::new("Press [1] - Level 1"),
                TextFont {
                    font_size: 40.0.into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 菜单按键（教程 12 Step 8 `handle_level_select_input`）：`1` → InGame。
///
/// 状态门控在 main.rs 的 `.run_if(in_state(LevelSelect))`；本系统只发转移请求
/// （`NextState::set` 延迟到 `StateTransition` 调度生效，教程 12「NextState<T>」）。
pub fn handle_level_select_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        next_state.set(GameState::InGame);
    }
}

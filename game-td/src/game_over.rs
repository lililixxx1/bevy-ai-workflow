//! GameOver 屏（教程 12 Step 9）：按 `GameResult` 显示胜负文案 + 空格回菜单。
//!
//! `GameResult` 由 `check_game_state` 在 FixedUpdate 链末尾写入；命令在链的 sync
//! point 落地 → `OnExit(InGame)` 的 `cleanup_level` 刻意不删它 → `OnEnter(GameOver)`
//! 读得到（教程 12 Step 9「Critical ordering detail」）。若清理链删掉 `GameResult`，
//! 本屏 `Res<GameResult>` 会 panic——这条依赖是显式契约，不是巧合。
//!
//! 0.20 锚点同 `level_select.rs`（Node / Val / Text / TextFont / TextColor）。

use bevy::prelude::*;

use crate::{GameResult, GameState, ScreenUi};

/// 生成结算屏（教程 12 Step 9 `setup_game_over`）：胜负文案 + 返回提示。
///
/// 教程文案含 em dash；默认内置字体无该字形（渲染成豆腐块，见首轮
/// p12-shot-over.png 留档），此处用 ASCII 连字符——UI 文案不在行为规格断言面内。
pub fn setup_game_over(mut commands: Commands, result: Res<GameResult>) {
    let message = match *result {
        GameResult::Defeat => "Game Over - the base was destroyed!",
        GameResult::Victory => "Victory - all enemies defeated!",
    };

    commands
        .spawn((
            ScreenUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(message),
                TextFont {
                    font_size: 40.0.into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("Press Space to continue"),
                TextFont {
                    font_size: 24.0.into(),
                    ..default()
                },
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
            ));
        });
}

/// 结算屏按键（教程 12 Step 9 `handle_game_over_input`）：空格 → 回 LevelSelect。
pub fn handle_game_over_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::LevelSelect);
    }
}

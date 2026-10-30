//! 暂停（教程 21 `PausePlugin`）：Escape 冻结模拟 + 半透明 "PAUSED" 覆盖层。
//!
//! 教程（0.18 形态）是行为规格、非实现来源；0.20 锚点（漂移表 Part 21）：
//! - `Plugin` trait：`bevy_app-0.20.0/src/plugin.rs:57`（trait）/ `:59`（`build`）；
//! - 正交 `States`：`bevy_state-0.20.0/src/state/mod.rs:9`、
//!   `state/states.rs:64`（`init_state`，`bevy_state-0.20.0/src/app.rs:40`）；
//! - `.and()` 组合子：0.20 无 `.and`，等价面 = `and_then`（短路由，
//!   `bevy_ecs-0.20.0/src/schedule/condition.rs:135`）/ `and_eager`（两者都求值，
//!   `:191`）——教程语义「两个条件都真」= 文档明确「behaviorally identical to
//!   simply calling run_if multiple times」→ 本工程用 **`and_eager`**（漂移表
//!   Part 21 摩擦级 1 的落地形态）；
//! - `with_child`：`bevy_ecs-0.20.0/src/hierarchy.rs:355`（`EntityWorldMut`）/
//!   `:450`（`EntityCommands`）；
//! - `Node` / `BackgroundColor` / `Text` / `TextFont` / `TextColor`：同 Part 12/14/17
//!   锚点（`TextFont.font_size` 是 `FontSize`，`64.0.into()`，`bevy_text-0.20.0/src/text.rs:688`）。
//!
//! 设计（教程 21 Step 2–4）：
//! - **正交状态机**：`GameState` 管「哪个屏」，[`PauseState`] 管「模拟是否推进」；
//!   `OnExit(GameState::InGame)` 只由真正离场触发（暂停不触发——这是不用
//!   `GameState::Paused` 变体的原因，教程 21「a problem」）；
//! - [`toggle_pause`]：Escape 翻转；守卫 = 只在 `InGame`、且 `GameFinished`
//!   尚未插入（`FixedUpdate` 先于 `Update`，资源已可见而状态迁移尚未生效——
//!   教程 21「The race condition guard」）；
//! - [`spawn_pause_overlay`] / [`despawn_pause_overlay`]：`OnEnter/OnExit(PauseState::Paused)`
//!   的生成/回收生命周期；覆盖层不进 `GameEntity` / `ScreenUi` 面（纯 PauseState 驱动）；
//! - 玩法三组系统在 `main.rs` 以 `.and_eager(in_state(PauseState::Running))` 门控；
//!   音频不在门控面（教程 21：Bevy 音频独立于 ECS 调度，暂停不停音乐）。
//!
//! 验证面（本工程扩展层，非教程）：`td.pause` 读写通道（`brp.rs`）——BRP 无键盘
//! 注入，Escape 不可直达，暂停/恢复效果需确定性入口（`td.` 前缀纪律）。

use bevy::prelude::*;

use crate::enemy::GameFinished;
use crate::GameState;

/// 暂停状态机（教程 21 Step 2）：与 `GameState` 正交——`InGame + Running` 正常玩，
/// `InGame + Paused` 冻结；`Default` = `Running`（启动即运行态）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

/// 暂停覆盖层标记（教程 21：模块私有；`pub(crate)` 供 `td.pause` 计数）。
#[derive(Component)]
pub(crate) struct PauseOverlay;

/// Escape 翻转暂停（教程 21 `toggle_pause`）。
///
/// 守卫三件（教程 21 表）：
/// ① 只在 `InGame` 生效（菜单 / 结算屏不受理）；
/// ② `GameFinished` 已插入则忽略——`check_game_state` 在 `FixedUpdate` 写资源 +
///    请求 `GameOver`，状态迁移要到下一帧的 `StateTransition` 才生效；本系统在
///    `Update` 读到 `game_state` 仍是 `InGame`，资源守卫补上这个窗口；
/// ③ 无「帧结束」的额外守卫——`NextState` 覆盖写，重复按 Escape 只是后写覆盖。
pub fn toggle_pause(
    game_state: Res<State<GameState>>,
    pause_state: Res<State<PauseState>>,
    game_finished: Option<Res<GameFinished>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_pause: ResMut<NextState<PauseState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    if *game_state.get() != GameState::InGame {
        return;
    }
    if game_finished.is_some() {
        return;
    }

    match pause_state.get() {
        PauseState::Running => next_pause.set(PauseState::Paused),
        PauseState::Paused => next_pause.set(PauseState::Running),
    }
}

/// 生成覆盖层（教程 21 `spawn_pause_overlay`）：全屏半透明黑 + 居中 "PAUSED"。
fn spawn_pause_overlay(mut commands: Commands) {
    commands
        .spawn((
            PauseOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::Flex,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_child((
            Text::new("PAUSED"),
            TextFont {
                font_size: 64.0.into(),
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

/// 回收覆盖层（教程 21 `despawn_pause_overlay`）：`OnExit(Paused)` 按标记清场。
fn despawn_pause_overlay(mut commands: Commands, overlay: Query<Entity, With<PauseOverlay>>) {
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
}

/// 暂停插件（教程 21 Step 3 `PausePlugin`）：状态机初始化 + 三系统注册。
pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<PauseState>()
            .add_systems(Update, toggle_pause)
            .add_systems(OnEnter(PauseState::Paused), spawn_pause_overlay)
            .add_systems(OnExit(PauseState::Paused), despawn_pause_overlay);
    }
}

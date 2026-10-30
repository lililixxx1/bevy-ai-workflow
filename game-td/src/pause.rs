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
//!
//! P22（教程 22 `Custom Run Conditions & System Sets`）：三条玩法系统链重复的
//! `in_state(InGame).and_eager(in_state(Running))` 收敛为**具名 run 条件**
//! [`game_is_running`] + **具名系统集合** [`GameplaySet`]（`configure_sets` 一次挂
//! 条件、`in_set` 只做成员归属）。教程放在 `state.rs`；本工程无 `state.rs`
//! （`GameState` 在 `main.rs`，P12 起既定），暂停域自含模块持有 `PauseState`，
//! 条件与集合随之放此处（沿 P21 偏离注记同口径）。
//!
//! 0.20 锚点（漂移表 Part 22，四项摩擦级 0）：
//! - 自定义 run 条件：带系统参数、返回 `bool` 的普通函数经 blanket impl 满足
//!   `SystemCondition`（`bevy_ecs-0.20.0/src/schedule/condition.rs:600`；
//!   trait 定义 `:75`）；
//! - `SystemSet` 派生（`bevy_ecs-0.20.0/src/schedule/set.rs:11` 再导出派生宏）；
//! - `App::configure_sets`（`bevy_app-0.20.0/src/app.rs:402`）与
//!   `.run_if`（`bevy_ecs-0.20.0/src/schedule/config.rs:543`）；
//! - `.in_set`（`bevy_ecs-0.20.0/src/schedule/config.rs:365`）。
//!
//! 验证面扩展（本工程，非教程）：[`SetTickLog`] 三个计数器各由**一条只做计数的
//! 微系统**驱动，该系统以 `.in_set(...)` 归属对应集合——集合条件若失效（例如
//! `configure_sets` 漏挂），计数器在非运行态仍会推进，`td.conditions` 即读出差异。
//! 记账资源口径同 `PlacementLog` / `RangeGizmoLog` / `AudioLog`（只服务断言，
//! 不给玩法逻辑增加语义）。

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

/// 具名 run 条件（教程 22 Step 1 `game_is_running`）：
/// 「在关卡里且未暂停」——三条玩法链原本各自重复的同一表达式。
///
/// 形态 = 普通函数 + `Res` 系统参数 + `bool` 返回值；0.20 的 blanket impl
/// 使其直接满足 `SystemCondition`（`bevy_ecs-0.20.0/src/schedule/condition.rs:600`），
/// 无需宏或专用 trait 实现。
pub fn game_is_running(
    game_state: Res<State<GameState>>,
    pause_state: Res<State<PauseState>>,
) -> bool {
    // `Res<State<T>>` 的 `get()` 返回 `&T`，比较前解引用（教程 22 Step 1 注）。
    *game_state.get() == GameState::InGame && *pause_state.get() == PauseState::Running
}

/// 具名系统集合（教程 22 Step 2 `GameplaySet`）：给三组玩法系统链命名，
/// `configure_sets` 在每个调度上**一次**挂 [`game_is_running`]，`in_set` 只声明归属。
///
/// 两个动作彼此独立（教程 22「These two are independent」）：`in_set` 不施加条件，
/// `configure_sets` 不调度系统——必须按调度成对维护（`main.rs` 三处配对）。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    /// `FixedUpdate`：刷怪 / 移动 / 攻击 / 弹道 / 经济（既有 FixedUpdate 链）。
    Simulation,
    /// `Update`：预览 / 放置 / HUD / gizmo（既有视觉 + 放置链）。
    Interaction,
    /// `Update`：滚轮 / 数字键 / 槽位点击（既有输入链）。
    TowerDock,
}

/// 集合驱动计数（本工程验证面扩展，非教程）：三个集合各一条只做计数的微系统，
/// 以 `.in_set(...)` 归属——计数器能否推进即「集合条件是否生效」的可观察面。
#[derive(Resource, Default)]
pub struct SetTickLog {
    /// `GameplaySet::Simulation` 驱动次数。
    pub simulation: u64,
    /// `GameplaySet::Interaction` 驱动次数。
    pub interaction: u64,
    /// `GameplaySet::TowerDock` 驱动次数。
    pub tower_dock: u64,
}

/// `Simulation` 集合计数（`main.rs` 注册时 `.in_set(GameplaySet::Simulation)`）。
pub fn tick_simulation(mut log: ResMut<SetTickLog>) {
    log.simulation += 1;
}

/// `Interaction` 集合计数（`main.rs` 注册时 `.in_set(GameplaySet::Interaction)`）。
pub fn tick_interaction(mut log: ResMut<SetTickLog>) {
    log.interaction += 1;
}

/// `TowerDock` 集合计数（`main.rs` 注册时 `.in_set(GameplaySet::TowerDock)`）。
pub fn tick_tower_dock(mut log: ResMut<SetTickLog>) {
    log.tower_dock += 1;
}

/// 取消动作翻转暂停（教程 21 `toggle_pause` + 教程 24 Step 6）。
///
/// P24：`ButtonInput<KeyCode>` 直读移除——只消费 `GameAction::Cancel`
/// （Escape / 手柄 East / Start 在 `input.rs` 统一翻译）。
///
/// 守卫三件（教程 21 表）：
/// ① 只在 `InGame` 生效（菜单 / 结算屏不受理）；
/// ② `GameFinished` 已插入则忽略——`check_game_state` 在 `FixedUpdate` 写资源 +
///    请求 `GameOver`，状态迁移要到下一帧的 `StateTransition` 才生效；本系统在
///    `Update` 读到 `game_state` 仍是 `InGame`，资源守卫补上这个窗口；
/// ③ 无「帧结束」的额外守卫——`NextState` 覆盖写，重复按取消只是后写覆盖。
pub fn toggle_pause(
    game_state: Res<State<GameState>>,
    pause_state: Res<State<PauseState>>,
    game_finished: Option<Res<GameFinished>>,
    mut actions: MessageReader<crate::input::GameAction>,
    mut next_pause: ResMut<NextState<PauseState>>,
) {
    let mut cancelled = false;
    for action in actions.read() {
        if matches!(action, crate::input::GameAction::Cancel) {
            cancelled = true;
        }
    }
    if !cancelled {
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

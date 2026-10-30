//! 输入抽象层（教程 24 `Input Abstraction` + 教程 25 `Gamepad`）。
//!
//! **职责**：把物理设备（键盘 / 鼠标 / 滚轮 / 手柄）翻译成两类逻辑通道——
//! - [`GameAction`] 消息：离散动作（方向 / 选择 / 确认 / 取消 / 切换）——
//!   玩法系统只读动作，不再碰 `KeyCode` / `MouseButton` / `MouseWheel`；
//! - [`VirtualCursorPos`] 资源：光标所在格——鼠标经 `CursorMoved` 写、键盘 / 手柄
//!   经方向动作写（`tower::nudge_virtual_cursor`），玩法系统只读它。
//!
//! 教程 24「物理输入 vs 逻辑输入」：翻译**只发生一次**，下游只见逻辑层。
//! 教程 24 Step 6 的重构表（本文件即「完整边界」）：
//! `select_tower_by_key` / `navigate_level_select` / `toggle_pause` /
//! `handle_game_over_input` 改读 `MessageReader<GameAction>`；
//! `update_placement_preview` / `place_tower_on_click` / `draw_tower_ranges` 改读
//! `Res<VirtualCursorPos>`；`cycle_tower_on_scroll` 删除（逻辑并入 `read_mouse_wheel`）；
//! `hovered_placeable_tile` 删除（换成 `tower::tile_is_placeable` 谓词）。
//!
//! 0.20 锚点（漂移表 Part 24/25，全部摩擦级 0）：
//! - `CursorMoved { window, position, delta }`（`bevy_window-0.20.0/src/event.rs:192`
//!   结构 / `:196`/`:202` 字段；`WindowPlugin` 注册消息 `bevy_window-0.20.0/src/lib.rs:115`；
//!   prelude 导出 `bevy_window-0.20.0/src/lib.rs` 内 `pub mod prelude`）；
//! - `MessageReader`（`bevy_ecs-0.20.0/src/message/message_reader.rs:34`）/
//!   `MessageWriter`（`message/message_writer.rs:62`）/ `.write`（`:74`）/ `.read`（`:44`）；
//! - `Message` 派生（`bevy_ecs-0.20.0/src/message/mod.rs:100`）——enum 亦可（P20 已实证）；
//! - `ButtonInput<KeyCode>::just_pressed`（`bevy_input-0.20.0/src/button_input.rs:188`）、
//!   `KeyCode::Escape/Enter/Space` 与 `Digit1..9`（`bevy_input-0.20.0/src/keyboard.rs`）；
//! - 手柄（教程 25，P25 加入）：
//!   `bevy_gilrs` 特性属 bevy 默认特性集 `2d → default_platform → bevy_gilrs`
//!   （`bevy-0.20.0/Cargo.toml:2706`/`:2887`/`:2800`）——**无需改 Cargo.toml**；
//!   `GilrsPlugin`（`bevy_gilrs-0.20.0/src/lib.rs:84`；`DefaultPlugins` 内含
//!   `bevy_internal-0.20.0/src/default_plugins.rs:82-83`）；`Gamepad` 组件
//!   （`bevy_input-0.20.0/src/gamepad.rs:376`）、`Gamepad::digital()`
//!   （`:525`，返回 `&ButtonInput<GamepadButton>`）、`Gamepad::left_stick()`
//!   （`:418`，返回 `Vec2`，轴向 [-1,1]）、`GamepadButton` 基数方位命名
//!   （`:581`，如 `DPadUp` / `South` / `East` / `Start` / `LeftTrigger` / `RightTrigger`）。
//!
//! 运行条件（教程 24 Step 3）：鼠标两系统挂 `in_state(GameState::InGame)`——它们要读
//! `MapLayout`（菜单屏没有）；键盘 / 滚轮系统无条件运行（动作只有当前状态的消费者
//! 会响应，教程 25 Step 4 注）。手柄读取器同样无条件运行（P25）。

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::map::MapLayout;
use crate::tower::{world_to_tile, VirtualCursorPos};
use crate::GameState;

/// 逻辑动作（教程 24 Step 4 `GameAction`）：变体表达**意图**而非硬件。
///
/// - `Up/Down/Left/Right` —— 方向键 / 手柄方向键 / 左摇杆（P25）
/// - `SelectTower(usize)` / `SelectLevel(usize)` —— 数字键快捷（上下文相关）
/// - `NextTower` / `PrevTower` —— 滚轮 / 手柄扳机
/// - `Confirm` —— Enter / Space / 鼠标点击 / 手柄 South
/// - `Cancel` —— Escape / 手柄 East / Start
///
/// `#[derive(Message)]` 对 enum 成立（P20 `SoundType` 已实证；`message/mod.rs:100`）。
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameAction {
    /// 方向上（+y 格）。
    Up,
    /// 方向下（-y 格）。
    Down,
    /// 方向左（-x 格）。
    Left,
    /// 方向右（+x 格）。
    Right,
    /// 数字键直选塔型（`InGame` 上下文；下标 = dock 槽位）。
    SelectTower(usize),
    /// 数字键直选关卡（`LevelSelect` 上下文；下标 = 发现列表下标）。
    SelectLevel(usize),
    /// 下一个塔型（滚轮向下 / 手柄右扳机）。
    NextTower,
    /// 上一个塔型（滚轮向上 / 手柄左扳机）。
    PrevTower,
    /// 确认（Enter / Space / 左键点击 / 手柄 South）。
    Confirm,
    /// 取消 / 暂停（Escape / 手柄 East / Start）。
    Cancel,
}

/// dock 数字键位表（`Digit1..Digit5` → 槽位 0..4；教程 17 原表迁入此处）。
const DOCK_DIGITS: [KeyCode; 5] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
];

/// 菜单数字键位表（`Digit1..Digit9` → 发现列表下标 0..8；教程 13 原表迁入此处）。
const LEVEL_DIGITS: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

/// 输入插件（教程 24/25）：注册动作消息 + 物理 → 逻辑翻译系统。
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GameAction>()
            // 键盘 / 滚轮：无条件运行（动作由当前状态的消费者按需响应）。
            .add_systems(Update, (read_keyboard_for_actions, read_mouse_wheel))
            // 鼠标兼容层：要读 `MapLayout`，只在关卡内运行（教程 24 Step 3）。
            .add_systems(
                Update,
                (read_mouse_hover, read_mouse_click).run_if(in_state(GameState::InGame)),
            )
            // P25：手柄读取器无条件运行（教程 25 Step 4：动作只由当前状态的消费者响应）。
            .add_systems(Update, (read_gamepad, read_gamepad_stick));
    }
}

/// 键盘 → 动作（教程 24 Step 5 `read_keyboard_for_actions`）。
///
/// 数字键是**上下文相关**的（教程 24 Step 5 注）：`InGame` = 选塔型，
/// `LevelSelect` = 选关卡；`GameOver` 无数字语义。方向 / 确认 / 取消与状态无关。
pub fn read_keyboard_for_actions(
    keys: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<GameState>>,
    mut actions: MessageWriter<GameAction>,
) {
    if keys.just_pressed(KeyCode::ArrowUp) {
        actions.write(GameAction::Up);
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        actions.write(GameAction::Down);
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        actions.write(GameAction::Left);
    }
    if keys.just_pressed(KeyCode::ArrowRight) {
        actions.write(GameAction::Right);
    }
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        actions.write(GameAction::Confirm);
    }
    if keys.just_pressed(KeyCode::Escape) {
        actions.write(GameAction::Cancel);
    }

    match game_state.get() {
        GameState::InGame => {
            for (i, key) in DOCK_DIGITS.iter().enumerate() {
                if keys.just_pressed(*key) {
                    actions.write(GameAction::SelectTower(i));
                }
            }
        }
        GameState::LevelSelect => {
            for (i, key) in LEVEL_DIGITS.iter().enumerate() {
                if keys.just_pressed(*key) {
                    actions.write(GameAction::SelectLevel(i));
                }
            }
        }
        GameState::GameOver => {}
    }
}

/// 滚轮 → 动作（教程 24 Step 5 `read_mouse_wheel`）。
///
/// 行为保真：P17 `cycle_tower_on_scroll` 每帧只消费**首条** `MouseWheel`——本系统
/// 同样只取 `.next()` 一条（多事件同帧不叠加步进），方向语义与 clamp 在消费者
/// `tower::select_tower_by_key` 侧保持原样。
pub fn read_mouse_wheel(
    mut scroll: MessageReader<MouseWheel>,
    mut actions: MessageWriter<GameAction>,
) {
    let Some(ev) = scroll.read().next() else {
        return;
    };
    if ev.y < 0.0 {
        actions.write(GameAction::NextTower);
    } else if ev.y > 0.0 {
        actions.write(GameAction::PrevTower);
    }
}

/// 鼠标悬停 → 虚拟光标（教程 24 Step 2 `read_mouse_hover`）。
///
/// 只处理**本帧最后一条** `CursorMoved`（`.last()`：本帧最终位置）；
/// 屏幕坐标 → 世界坐标 → 网格坐标，越界不留值（保持上一格）。
pub fn read_mouse_hover(
    camera: Single<(&Camera, &GlobalTransform)>,
    map_layout: Res<MapLayout>,
    mut cursor: ResMut<VirtualCursorPos>,
    mut cursor_events: MessageReader<CursorMoved>,
) {
    let Some(event) = cursor_events.read().last() else {
        return;
    };
    let (cam, cam_transform) = *camera;
    let Ok(world) = cam.viewport_to_world_2d(cam_transform, event.position) else {
        return;
    };
    if let Some(tile) = world_to_tile(world, map_layout.width, map_layout.height) {
        cursor.0 = Some(tile);
    }
}

/// 鼠标左键 → `Confirm`（教程 24 Step 3 `read_mouse_click`）。
///
/// 只在光标落在**地图格内**时发动作（点在空白处不会误放塔——教程 24 注）；
/// 「该格是否可放置」由下游 `tower::place_tower_on_click` 用 `tile_is_placeable` 判定。
pub fn read_mouse_click(
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    map_layout: Res<MapLayout>,
    mut actions: MessageWriter<GameAction>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let (cam, cam_transform) = *camera;
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(world) = cam.viewport_to_world_2d(cam_transform, cursor_pos) else {
        return;
    };
    if world_to_tile(world, map_layout.width, map_layout.height).is_some() {
        actions.write(GameAction::Confirm);
    }
}

// ---------------------------------------------------------------------------
// P25：手柄输入（教程 25 `Gamepad`）
// ---------------------------------------------------------------------------

/// 摇杆死区（教程 25 Step 3 原文阈值 0.3）：小于此值视为松手 / 漂移，不产生动作。
pub const STICK_DEAD_ZONE: f32 = 0.3;

/// 摇杆按住时的重复间隔（教程 25 Step 3 原文 150ms 一格）。
pub const STICK_REPEAT_SECS: f32 = 0.15;

/// 手柄物理按键 → 动作的**纯映射表**（教程 25 Step 2 `read_gamepad` 的表体）。
///
/// 抽为纯函数是为可测：无手柄环境下映射正确性由 `td.gamepad probe_button` 调用
/// **同一函数**断言（纯函数探测口径同 `td.gizmos probe` 先例），
/// 结构面由 `bindings` 快照给出。
///
/// 映射（教程 25 Step 2）：
/// - `DPadUp/Down/Left/Right` → `Up/Down/Left/Right`
/// - `South` → `Confirm`（= 键盘 Enter/Space、鼠标左键）
/// - `East` / `Start` → `Cancel`（= 键盘 Escape）
/// - `LeftTrigger` / `RightTrigger` → `PrevTower` / `NextTower`（= 滚轮 / LB / RB）
///
/// 无状态相关映射：手柄按键少且语义明确（教程 25 Step 2「No context-dependent mapping」）。
pub fn map_gamepad_button(button: GamepadButton) -> Option<GameAction> {
    match button {
        GamepadButton::DPadUp => Some(GameAction::Up),
        GamepadButton::DPadDown => Some(GameAction::Down),
        GamepadButton::DPadLeft => Some(GameAction::Left),
        GamepadButton::DPadRight => Some(GameAction::Right),
        GamepadButton::South => Some(GameAction::Confirm),
        GamepadButton::East => Some(GameAction::Cancel),
        GamepadButton::Start => Some(GameAction::Cancel),
        GamepadButton::LeftTrigger => Some(GameAction::PrevTower),
        GamepadButton::RightTrigger => Some(GameAction::NextTower),
        _ => None,
    }
}

/// 摇杆向量 → 方向动作（教程 25 Step 3 的判别体，纯函数）：
/// 死区内 → `None`；否则**只取主导轴**（对角推杆不发两个方向）。
///
/// `left_stick()` 返回轴向 [-1,1] 的 `Vec2`（`bevy_input-0.20.0/src/gamepad.rs:418`）。
pub fn stick_to_action(stick: Vec2) -> Option<GameAction> {
    if stick.length() < STICK_DEAD_ZONE {
        return None;
    }
    Some(if stick.x.abs() > stick.y.abs() {
        if stick.x > 0.0 {
            GameAction::Right
        } else {
            GameAction::Left
        }
    } else if stick.y > 0.0 {
        GameAction::Up
    } else {
        GameAction::Down
    })
}

/// 手柄数字按键 → 动作（教程 25 Step 2 `read_gamepad`）。
///
/// 遍历所有已连接手柄（无「玩家 1」分配——教程 25 Simplifications），
/// 逐个读 `digital()` 的 `just_pressed` 集合并经 [`map_gamepad_button`] 翻译。
/// 无手柄时查询为空 → 零动作（零影响回归面）。
pub fn read_gamepad(gamepads: Query<&Gamepad>, mut actions: MessageWriter<GameAction>) {
    for gamepad in gamepads.iter() {
        for button in gamepad.digital().get_just_pressed() {
            if let Some(action) = map_gamepad_button(*button) {
                actions.write(action);
            }
        }
    }
}

/// 左摇杆 → 方向动作（教程 25 Step 3 `read_gamepad_stick`）。
///
/// 摇杆是**连续量**（每帧都有值），不能用 `just_pressed`——用「死区 + 定时重复」：
/// - 死区 0.3：小于阈值视为松手（回中），下一次推杆**立即**触发；
/// - `Timer` 150ms 重复：按住时每 150ms 走一格；
/// - 主导轴：对角推杆只取较强轴（4 方向模型，判别体 = [`stick_to_action`]）。
pub fn read_gamepad_stick(
    gamepads: Query<&Gamepad>,
    mut actions: MessageWriter<GameAction>,
    mut timer: Local<Timer>,
    mut was_idle: Local<bool>,
    time: Res<Time>,
) {
    // 首次运行初始化重复计时器（150ms 一格；`Local<Timer>` 默认时长 0）。
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(STICK_REPEAT_SECS, TimerMode::Repeating);
        *was_idle = true;
    }

    for gamepad in gamepads.iter() {
        let stick = gamepad.left_stick();
        let Some(action) = stick_to_action(stick) else {
            // 松手（或漂移死区内）：下一次推杆首帧即触发。
            *was_idle = true;
            continue;
        };
        timer.tick(time.delta());
        if !*was_idle && !timer.just_finished() {
            continue;
        }
        *was_idle = false;
        timer.reset();
        actions.write(action);
    }
}

/// 手柄按键 → 动作的绑定快照（`td.gamepad` 的 `bindings` 响应体；P25 结构面）。
///
/// 与 [`map_gamepad_button`] 同源（逐项遍历同一张表），断言脚本按此表独立复算。
pub const GAMEPAD_BINDINGS: [(GamepadButton, GameAction); 9] = [
    (GamepadButton::DPadUp, GameAction::Up),
    (GamepadButton::DPadDown, GameAction::Down),
    (GamepadButton::DPadLeft, GameAction::Left),
    (GamepadButton::DPadRight, GameAction::Right),
    (GamepadButton::South, GameAction::Confirm),
    (GamepadButton::East, GameAction::Cancel),
    (GamepadButton::Start, GameAction::Cancel),
    (GamepadButton::LeftTrigger, GameAction::PrevTower),
    (GamepadButton::RightTrigger, GameAction::NextTower),
];

/// `GamepadButton` → 稳定字符串（`td.gamepad` 快照与断言脚本口径）：
/// 直接用 `Debug` 输出（变体名即名字，`bevy_input-0.20.0/src/gamepad.rs:581` 起）。
pub fn gamepad_button_name(button: GamepadButton) -> String {
    format!("{button:?}")
}

/// 名字 → `GamepadButton`（`td.gamepad` 参数解析；只认标准 19 键，`Other` 不收）。
///
/// 表体 = `GamepadButton::all()` 的变体名（`bevy_input-0.20.0/src/gamepad.rs:612`）。
pub fn gamepad_button_from_name(name: &str) -> Option<GamepadButton> {
    GamepadButton::all()
        .into_iter()
        .find(|button| gamepad_button_name(*button) == name)
}

/// `GameAction` → 稳定字符串（`td.gamepad` / `td.input` 的响应与日志口径）。
pub fn action_name(action: &GameAction) -> String {
    match action {
        GameAction::Up => "Up".to_string(),
        GameAction::Down => "Down".to_string(),
        GameAction::Left => "Left".to_string(),
        GameAction::Right => "Right".to_string(),
        GameAction::SelectTower(i) => format!("SelectTower({i})"),
        GameAction::SelectLevel(i) => format!("SelectLevel({i})"),
        GameAction::NextTower => "NextTower".to_string(),
        GameAction::PrevTower => "PrevTower".to_string(),
        GameAction::Confirm => "Confirm".to_string(),
        GameAction::Cancel => "Cancel".to_string(),
    }
}

/// 动作名 + 可选下标 → `GameAction`（`td.input emit` 的解析；未知名 = `None`）。
pub fn action_from_spec(name: &str, index: Option<usize>) -> Option<GameAction> {
    match name {
        "Up" => Some(GameAction::Up),
        "Down" => Some(GameAction::Down),
        "Left" => Some(GameAction::Left),
        "Right" => Some(GameAction::Right),
        "Confirm" => Some(GameAction::Confirm),
        "Cancel" => Some(GameAction::Cancel),
        "NextTower" => Some(GameAction::NextTower),
        "PrevTower" => Some(GameAction::PrevTower),
        "SelectTower" => index.map(GameAction::SelectTower),
        "SelectLevel" => index.map(GameAction::SelectLevel),
        _ => None,
    }
}

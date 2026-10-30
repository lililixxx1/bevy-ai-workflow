//! LevelSelect 屏（教程 12 Step 8；P13 起动态关卡列表）。
//!
//! P13（教程 13 Step 4–6）把菜单从「硬编码 level_01」升级为**自动发现**：
//! - [`AvailableLevels`]：`assets/levels/level_01..09.toml` 中实际存在的文件路径表
//!   （`init_resource` 注册空表，`scan_available_levels` 每次进入菜单重扫——
//!   教程 13「New Bevy APIs」与 Step 4）；
//! - [`SelectedLevel`]：玩家选中的关卡路径（资源即通信通道，教程 13 Step 6 注）；
//! - `setup_level_select` 逐条列 `[N] level_name`（教程 13 Step 5），
//!   `handle_level_select_input` 把 `1`–`9` 映射到发现列表下标（教程 13 Step 6）。
//!
//! 0.20 锚点（漂移表 Part 12/13）：
//! - `Node` 布局组件与 `AlignItems` / `JustifyContent`（`bevy_ui-0.20.0/src/ui_node.rs:485`/`:900`/`:1358`）；
//! - `Val::Percent`（`bevy_ui-0.20.0/src/geometry.rs:32`）；
//! - `Text`（`bevy_ui-0.20.0/src/widget/text.rs:128`）、`TextColor`（`bevy_text-0.20.0/src/text.rs:1405`）；
//! - `TextFont.font_size` 是 `FontSize` 枚举（`bevy_text-0.20.0/src/text.rs:688`/`:783`），
//!   `From<f32> for FontSize` = `FontSize::Px`（`:861-865`）——0.18 的裸 `f32` 写法
//!   在 0.20 需一次转换（漂移表 Part 12 摩擦级 1）；
//! - `App::init_resource`（`bevy_app-0.20.0/src/app.rs:485`）、`KeyCode::Digit1..9`
//!   （`bevy_input-0.20.0/src/keyboard.rs:300` 起）——漂移表 Part 13 两项均摩擦级 0。
//!
//! `ScreenUi` 只挂根节点：`cleanup_screen_ui` 逐实体 despawn，父节点递归带走子文本
//! （教程 12 Step 8 注）。`ScreenUi` 不进 `GameEntity` 面——它不属于关卡世界。

use bevy::prelude::*;

use crate::{GameState, ScreenUi};

/// 已发现的关卡文件路径（教程 13 Step 1 `AvailableLevels`）。
///
/// 路径相对进程 CWD（= 工程根 `<tree>/game-td`），与 `load_level` 的
/// `std::fs::read_to_string` 同根（教程 06 口径）。
#[derive(Resource, Default)]
pub struct AvailableLevels(pub Vec<String>);

/// 当前选中的关卡路径（教程 13 Step 1 `SelectedLevel`）。
///
/// `Default` 为空串（`init_resource` 需要）；真实值由 `scan_available_levels`
/// 在每次进入菜单时写入（教程 13 Step 4 尾：总是重置为第一个发现项）。
#[derive(Resource, Default)]
pub struct SelectedLevel(pub String);

/// 探测范围上界（教程 13 Step 4：`1..=9`；固定区间是教程的确定性简化）。
const MAX_LEVELS: u32 = 9;

/// 关卡路径模板（教程 13 Step 4：`assets/levels/level_{:02}.toml`）。
fn level_path(index: u32) -> String {
    format!("assets/levels/level_{index:02}.toml")
}

/// 扫描可用关卡（教程 13 Step 4 `scan_available_levels`）：
/// `level_01..09.toml` 中存在的进表，选中项重置为第一个发现项。
///
/// 刷新时机 = `OnEnter(LevelSelect)`（链首，先于 `setup_level_select`）：
/// 若顺序倒置 UI 会读到空表（教程 13 Step 8 注）。
/// `std::path::Path::exists()` 只适用于原生构建（教程 13 Step 4 Pitfall；WASM 不可用，
/// 本工程平台约束 = PC，见 AGENTS.md 硬约束 5）。
pub fn scan_available_levels(
    mut levels: ResMut<AvailableLevels>,
    mut selected: ResMut<SelectedLevel>,
) {
    levels.0.clear();
    for i in 1..=MAX_LEVELS {
        let path = level_path(i);
        if std::path::Path::new(&path).exists() {
            levels.0.push(path);
        }
    }
    if let Some(first) = levels.0.first() {
        selected.0 = first.clone();
    }
}

/// 生成标题屏（教程 13 Step 5 `setup_level_select`）：全屏居中 + 动态关卡列表。
pub fn setup_level_select(mut commands: Commands, levels: Res<AvailableLevels>) {
    let mut text = String::new();
    for (i, path) in levels.0.iter().enumerate() {
        if !text.is_empty() {
            text.push('\n');
        }
        // 展示名 = 去掉目录前缀与 .toml 后缀（教程 13 Step 5）；不匹配时回落原路径。
        let name = path
            .strip_prefix("assets/levels/")
            .and_then(|s| s.strip_suffix(".toml"))
            .unwrap_or(path);
        text.push_str(&format!("[{}] {}", i + 1, name));
    }
    if text.is_empty() {
        // 无任何关卡文件：给出可见提示而非空白屏（教程未覆盖；行为兜底，不进断言面）。
        text.push_str("No levels found in assets/levels/");
    }

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
                Text::new(text),
                TextFont {
                    font_size: 40.0.into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 键位表（教程 13 Step 6 的 `match` 逐项等价形态；下标 = 发现列表下标）。
const LEVEL_KEYS: [KeyCode; 9] = [
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

/// 菜单按键（教程 13 Step 6 `handle_level_select_input`）：
/// 只对已发现关卡生效（`len().min(9)`），选中后请求 `InGame`。
///
/// 状态门控在 main.rs 的 `.run_if(in_state(LevelSelect))`；本系统只发转移请求
/// （`NextState::set` 延迟到 `StateTransition` 调度生效，教程 12「NextState<T>」）。
pub fn handle_level_select_input(
    keys: Res<ButtonInput<KeyCode>>,
    levels: Res<AvailableLevels>,
    mut selected: ResMut<SelectedLevel>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (i, key) in LEVEL_KEYS.iter().take(levels.0.len()).enumerate() {
        if keys.just_pressed(*key) {
            selected.0 = levels.0[i].clone();
            next_state.set(GameState::InGame);
            return;
        }
    }
}

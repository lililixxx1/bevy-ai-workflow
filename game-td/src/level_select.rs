//! LevelSelect 屏（教程 12 Step 8；P13 起动态关卡列表；P23 起 **按钮网格**）。
//!
//! P13（教程 13 Step 4–6）把菜单从「硬编码 level_01」升级为**自动发现**：
//! - [`AvailableLevels`]：`assets/levels/level_01..09.toml` 中实际存在的文件路径表
//!   （`init_resource` 注册空表，`scan_available_levels` 每次进入菜单重扫）；
//! - [`SelectedLevel`]：玩家选中的关卡路径（资源即通信通道，教程 13 Step 6 注）。
//!
//! P23（教程 23 `Level Select Buttons`）：单块 `Text` 列表 → **3 列按钮网格**：
//! - [`LevelButton`] 标记每个按钮代表的发现列表下标；
//! - [`FocusedLevel`] 焦点下标（键盘导航与视觉更新共享）；
//! - [`navigate_level_select`]：方向键移动焦点（2D 网格下标算式）+ 数字键快捷键
//!   + Enter/Space 确认；
//! - [`update_level_select_visuals`]：焦点优先的着色（焦点蓝 > 悬停/按下/默认）；
//! - [`handle_level_button_click`]：鼠标点击（`Changed<Interaction>` + `Pressed`）
//!   一步「移动焦点 + 确认」。
//!
//! 0.20 锚点（漂移表 Part 12/13/23）：
//! - `Node` / `AlignItems` / `JustifyContent` / `FlexWrap::Wrap` / `BackgroundColor` /
//!   `BorderColor::all` / `BorderRadius::all`（`bevy_ui-0.20.0/src/ui_node.rs:485`/`:900`/
//!   `:1358`/`:1788`/`:2576`/`:2627`/`:3005`；`BorderRadius::all(impl Into<CornerRadius>)`
//!   的 `From<Val>` 在 `geometry.rs:1445`）；
//! - `Val`（`bevy_ui-0.20.0/src/geometry.rs:32`）、`UiRect::all`（`:751`）；
//! - `Text`（`bevy_ui-0.20.0/src/widget/text.rs:128`）、`TextColor`（`bevy_text-0.20.0/src/text.rs:1405`）；
//! - `TextFont.font_size` 是 `FontSize` 枚举（`bevy_text-0.20.0/src/text.rs:688`/`:783`），
//!   `From<f32>` = `FontSize::Px`（`:861-865`）——0.18 的裸 `f32` 在 0.20 需一次转换；
//! - `Button` / `Interaction`：0.20 为**弃用别名**（`bevy_ui-0.20.0/src/widget/button.rs:30`；
//!   `focus.rs:50` 本体 `DeprecatedInteraction`、`:65` 别名；SKILL §6.14）。按 T044 立项
//!   **A' 裁决**沿 P17 先例：教程原步用法 + `#[expect(deprecated, reason = "…")]`
//!   多行属性落地，**不换新 API 重写**；
//! - `Changed<Interaction>` 过滤（`bevy_ecs-0.20.0/src/query/filter.rs:996`）；
//! - `Button` 的 `#[require(Node, FocusPolicy::Block)]` + `#[require(Interaction)]`
//!   （`widget/button.rs:24-28`）——`Interaction` 由引擎自动注入，**不手 spawn**；
//! - `App::init_resource`（`bevy_app-0.20.0/src/app.rs:485`）、`KeyCode::Digit1..9`
//!   （`bevy_input-0.20.0/src/keyboard.rs:300` 起）、`KeyCode::ArrowUp/Down/Left/Right`
//!   与 `Enter` / `Space`（同文件）。
//!
//! 验证面（本工程扩展层，非教程）：`td.level_select`（`brp.rs`）——BRP 无鼠标注入，
//! 但消费者系统是**真码路径**：`{"op":"click","index":n}` 先写该按钮的
//! `Interaction::Pressed`（= 替代 bevy_ui focus 系统的输入源），再以
//! `World::run_system_once` **运行真实的 [`handle_level_button_click`] 一次**——
//! `Changed<Interaction>` 过滤与选中逻辑全走原实现；`{"op":"confirm"}` 走
//! Enter/Space 分支共用的 [`apply_level_selection`]。
//!
//! `ScreenUi` 只挂根节点：`cleanup_screen_ui` 逐实体 despawn，父节点递归带走子文本
//! （教程 12 Step 8 注）。`ScreenUi` 不进 `GameEntity` 面——它不属于关卡世界。

use bevy::prelude::*;
// `FocusPolicy` 不在 prelude（0.20：`bevy_ui-0.20.0/src/focus.rs` 定义，
// `bevy_ui/src/lib.rs:42` 经 `pub use focus::*` 导出到 `bevy::ui`）。
use bevy::ui::FocusPolicy;

use crate::input::GameAction;
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

/// 菜单焦点下标（教程 23 Step 1 `FocusedLevel`）：键盘导航与视觉更新共享。
///
/// 教程用 `commands.insert_resource(FocusedLevel(0))` 每次进菜单重置；本工程以
/// `init_resource` 一次注册 + `scan_available_levels` 里 `focused.0 = 0` 重置——
/// 行为等价，且避免「资源只在 OnEnter 插入」时视觉系统早一帧读不到资源的窗口。
#[derive(Resource, Default)]
pub struct FocusedLevel(pub usize);

/// 按钮代表的发现列表下标（教程 23 Step 1 `LevelButton(usize)`）。
///
/// 反射注册（`#[type_path = "game_td"]` 冻结）：BRP 无鼠标注入，按钮结构面
/// （按钮计数 / 焦点 / 交互态 / 背景色）经 `td.level_select` 快照取得，类型须可查询
/// ——沿 P17 `TowerDockSlot` 同口径（SKILL §6.17）。
#[derive(Component, Reflect)]
#[type_path = "game_td"]
#[reflect(Component)]
pub struct LevelButton(pub usize);

/// 探测范围上界（教程 13 Step 4：`1..=9`；固定区间是教程的确定性简化）。
const MAX_LEVELS: u32 = 9;

/// 网格列数（教程 23 Step 1 设计：3 列）。
const COLS: usize = 3;
/// 单个按钮宽（教程 23 Step 1：200px）。
const BUTTON_W: f32 = 200.0;
/// 单个按钮高（教程 23 Step 1：60px）。
const BUTTON_H: f32 = 60.0;
/// 按钮间距（教程 23 Step 1：20px，横纵同值）。
const GAP: f32 = 20.0;
/// 默认底色（教程 23 Step 2 `COLOR_DEFAULT`）。
const COLOR_DEFAULT: Color = Color::srgba(0.15, 0.15, 0.15, 1.0);
/// 焦点底色（教程 23 Step 2 `COLOR_FOCUSED`：蓝。）
const COLOR_FOCUSED: Color = Color::srgba(0.1, 0.3, 0.5, 1.0);
/// 悬停底色（教程 23 Step 2 `COLOR_HOVERED`）。
const COLOR_HOVERED: Color = Color::srgba(0.2, 0.25, 0.35, 1.0);
/// 按下底色（教程 23 Step 2 `COLOR_PRESSED`）。
const COLOR_PRESSED: Color = Color::srgba(0.05, 0.15, 0.3, 1.0);
/// 按钮边框色（教程 23 按钮 `BorderColor`；非行为规格面，取中性灰）。
const COLOR_BORDER: Color = Color::srgba(0.35, 0.35, 0.35, 1.0);

/// 关卡路径模板（教程 13 Step 4：`assets/levels/level_{:02}.toml`）。
fn level_path(index: u32) -> String {
    format!("assets/levels/level_{index:02}.toml")
}

/// 展示名 = 去掉目录前缀与 `.toml` 后缀（教程 13 Step 5）；不匹配时回落原路径。
fn display_name(path: &str) -> &str {
    path.strip_prefix("assets/levels/")
        .and_then(|s| s.strip_suffix(".toml"))
        .unwrap_or(path)
}

/// 扫描可用关卡（教程 13 Step 4 `scan_available_levels`）：
/// `level_01..09.toml` 中存在的进表，选中项重置为第一个发现项。
///
/// 刷新时机 = `OnEnter(LevelSelect)`（链首，先于 `setup_level_select`）：
/// 若顺序倒置 UI 会读到空表（教程 13 Step 8 注）。P23 Step 3：焦点重置到左上角
/// （教程 23「Reset focus on entry」）。
/// `std::path::Path::exists()` 只适用于原生构建（教程 13 Step 4 Pitfall；WASM 不可用，
/// 本工程平台约束 = PC，见 AGENTS.md 硬约束 5）。
pub fn scan_available_levels(
    mut levels: ResMut<AvailableLevels>,
    mut selected: ResMut<SelectedLevel>,
    mut focused: ResMut<FocusedLevel>,
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
    // 教程 23 Step 3：每次进菜单焦点回到第一个按钮（= 左上角）。
    focused.0 = 0;
}

/// 生成标题屏（教程 23 Step 2 `setup_level_select`）：
/// 全屏居中容器 → `FlexWrap::Wrap` 网格容器（宽 = `COLS × BUTTON_W + (COLS-1) × GAP`
/// = 640px，恰容一行 3 个）→ 每关一个按钮实体 + `Text` 子节点。
///
/// 空表兜底（教程未覆盖）：给一行可见提示而非空白屏——不进断言面。
///
/// `#[expect(deprecated)]`：`Interaction` 在 0.20 是弃用别名（`focus.rs:50/:65`；
/// SKILL §6.14）——教程 23 原步用法，A' 裁决通道；`Button` 在 0.20 不可构造
/// （见下方 spawn 处实核注记），以 `Interaction::None` + `FocusPolicy::Block` 等价落位。
#[expect(
    deprecated,
    reason = "教程 23 原步使用 Interaction 三态；0.20 弃用别名按 T044 A' 裁决以 expect 属性落地"
)]
pub fn setup_level_select(mut commands: Commands, levels: Res<AvailableLevels>) {
    if levels.0.is_empty() {
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
            .with_child((
                Text::new("No levels found in assets/levels/"),
                TextFont {
                    font_size: 40.0.into(),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        return;
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
            // 网格容器：固定宽 + 换行 → 扁平子列表自动成 3 列。
            parent
                .spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    width: Val::Px(COLS as f32 * BUTTON_W + (COLS - 1) as f32 * GAP),
                    column_gap: Val::Px(GAP),
                    row_gap: Val::Px(GAP),
                    ..default()
                })
                .with_children(|grid| {
                    for (i, path) in levels.0.iter().enumerate() {
                        // 0.20 实核（表外新事实）：教程 23 的 `Button` 在 0.20 是
                        // `pub(crate) struct DeprecatedButton` 的**类型别名**
                        // （`bevy_ui-0.20.0/src/widget/button.rs:30`）——可作类型名，
                        // 不可作构造值（`Button` / `Button::default()` 均 E0603
                        // 「private type」）。该标记组件的全部作用是两条 `#[require]`：
                        // `Node` + `FocusPolicy::Block` + `Interaction`（`button.rs:24-28`）。
                        // 故以 `Interaction::None` + `FocusPolicy::Block` 显式等价落位；
                        // `ui_focus_system` 只认 `Option<&mut Interaction>`（`focus.rs:147-158`），
                        // 不要求 Button 组件，三态跟踪行为不变。A' 精神不变：仍用教程的
                        // `Interaction` 三态 + `#[expect(deprecated)]`，不换新 API 重写。
                        grid.spawn((
                            Interaction::None,
                            FocusPolicy::Block,
                            LevelButton(i),
                            Node {
                                width: Val::Px(BUTTON_W),
                                height: Val::Px(BUTTON_H),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(COLOR_DEFAULT),
                            BorderColor::all(COLOR_BORDER),
                        ))
                        .with_child((
                            Text::new(display_name(path)),
                            TextFont {
                                font_size: 24.0.into(),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    }
                });
        });
}

// P24：原 `LEVEL_KEYS`（`Digit1..Digit9` 键位表）迁入 `input.rs::LEVEL_DIGITS`
// ——教程 24「Physical-input code lives in one file」：菜单只消费 `GameAction`。

/// 选中第 `index` 个发现项并请求进入 `InGame`（三通道共用：Enter/Space、按钮点击、
/// `td.level_select confirm`）。
///
/// 返回被选中的路径（越界 = `None` 且不改任何状态）；`NextState` 是延迟转移
/// （`bevy_state-0.20.0/src/state/resources.rs:198`），下一帧 `StateTransition` 生效。
pub fn apply_level_selection(
    index: usize,
    levels: &AvailableLevels,
    selected: &mut SelectedLevel,
    next_state: &mut NextState<GameState>,
) -> Option<String> {
    let path = levels.0.get(index)?.clone();
    selected.0 = path.clone();
    next_state.set(GameState::InGame);
    Some(path)
}

/// 菜单按键（教程 23 Step 4 `navigate_level_select` + 教程 24 Step 6）：
/// P24 起只消费 [`GameAction`]（`ButtonInput<KeyCode>` 直读移除）——
/// 方向动作按 2D 网格算式移动焦点（`row = i / COLS`、`col = i % COLS`），
/// `SelectLevel(i)`（数字键）与 `Confirm`（Enter/Space/点击/手柄 South）直接选中。
///
/// 边界规则（教程 23 Step 4）：上 / 左不在边界内不动；下用 `.min(len-1)` 夹住
/// 末行短板；右同时检查列边界**与**总数（末行可能不满 `COLS`）。无焦点回绕
/// （教程 23 Simplifications：停在边界更有「刻意感」）。
pub fn navigate_level_select(
    mut actions: MessageReader<GameAction>,
    levels: Res<AvailableLevels>,
    mut focused: ResMut<FocusedLevel>,
    mut selected: ResMut<SelectedLevel>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let len = levels.0.len();
    if len == 0 {
        return;
    }
    // 焦点越界保护（关卡表变短时——例如文件被删——先把下标夹回有效区）。
    if focused.0 >= len {
        focused.0 = len - 1;
    }

    for action in actions.read() {
        match action {
            GameAction::Up => {
                if focused.0 / COLS > 0 {
                    focused.0 = focused.0.saturating_sub(COLS);
                }
            }
            GameAction::Down => {
                let total_rows = len.div_ceil(COLS);
                if focused.0 / COLS + 1 < total_rows {
                    focused.0 = (focused.0 + COLS).min(len - 1);
                }
            }
            GameAction::Left => {
                if focused.0 % COLS > 0 {
                    focused.0 -= 1;
                }
            }
            GameAction::Right => {
                if focused.0 % COLS + 1 < COLS && focused.0 + 1 < len {
                    focused.0 += 1;
                }
            }
            // 数字键快捷（教程 13 Step 6 行为原样保留：只对已发现关卡生效）。
            GameAction::SelectLevel(i) => {
                if *i < len {
                    focused.0 = *i;
                    apply_level_selection(*i, &levels, &mut selected, &mut next_state);
                    return;
                }
            }
            // Enter / Space 确认焦点项（教程 23 Step 4「handle confirm」）。
            GameAction::Confirm => {
                apply_level_selection(focused.0, &levels, &mut selected, &mut next_state);
                return;
            }
            _ => {}
        }
    }
}

/// 焦点 / 交互态着色（教程 23 Step 5 `update_level_select_visuals`）：
/// 焦点优先（焦点按钮恒为蓝，避免与悬停色闪烁）；其余按钮按三态取色。
///
/// `Interaction` 是 0.20 弃用别名（`focus.rs:50/:65`；SKILL §6.14）——教程原步，
/// A' 裁决通道。
#[expect(
    deprecated,
    reason = "教程 23 原步使用 Interaction 三态；0.20 弃用别名按 T044 A' 裁决以 expect 属性落地"
)]
pub fn update_level_select_visuals(
    focused: Res<FocusedLevel>,
    mut buttons: Query<(&LevelButton, &Interaction, &mut BackgroundColor)>,
) {
    for (btn, interaction, mut bg) in buttons.iter_mut() {
        bg.0 = if btn.0 == focused.0 {
            COLOR_FOCUSED
        } else {
            match *interaction {
                Interaction::Hovered => COLOR_HOVERED,
                Interaction::Pressed => COLOR_PRESSED,
                Interaction::None => COLOR_DEFAULT,
            }
        };
    }
}

/// 鼠标点击（教程 23 Step 6 `handle_level_button_click`）：`Changed<Interaction>` +
/// `Pressed` → 一步「移动焦点 + 选中 + 进关」（鼠标用户不必点两次）。
///
/// `Interaction` 弃用别名同上（A' 通道）。
#[expect(
    deprecated,
    reason = "教程 23 原步使用 Interaction::Pressed；0.20 弃用别名按 T044 A' 裁决以 expect 属性落地"
)]
pub fn handle_level_button_click(
    levels: Res<AvailableLevels>,
    mut focused: ResMut<FocusedLevel>,
    mut selected: ResMut<SelectedLevel>,
    mut next_state: ResMut<NextState<GameState>>,
    buttons: Query<(&LevelButton, &Interaction), Changed<Interaction>>,
) {
    for (btn, interaction) in buttons.iter() {
        if *interaction == Interaction::Pressed {
            focused.0 = btn.0;
            apply_level_selection(btn.0, &levels, &mut selected, &mut next_state);
            return;
        }
    }
}

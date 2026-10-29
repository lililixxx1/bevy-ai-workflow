# T044-W2 · 26 部外部教程 API 漂移底稿（教程 0.18 形态 → bevy 0.20.0 / bevy_ecs_tilemap 0.20.0）

> 口径：教程（`tutorial-src/01..26`，目标版本 Bevy 0.18）只作为**行为规格**，本档只记录 API 事实，不复制教程实现逻辑与业务代码（附录 C）。
> 每行的「0.18 形态摘要」取自该部正文（≤15 字）；「0.20 结论」与「0.20 证据锚」全部来自本地 0.20.0 源码实读（crate 内相对路径 + 行号）或树内 `bevy-dev/SKILL.md` §6 已核实条目，二者必居其一，无「印象中/应为」类结论。
> 证据锚写法：`<crate>-0.20.0/<crate 内相对路径>:<行号>`；SKILL 锚写法：`SKILL §6.x（bevy-dev/SKILL.md:<行号>）`。
> 结论词表（四选一）：同形直用 / 改名/小形态差 / 重写调用面 / 无对应需路线变更。
> 摩擦级：0=同形直用；1=改名或小形态差；2=重写调用面；3=路线变更。
> 「（无新 API，继承前部）」为派工单规定的显式行，仅用于纯重构/纯概念部（Part 05、Part 16）。

## Part 01 · Getting Started

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| App | 01/`App` | App::new() | 同形直用 | bevy_app-0.20.0/src/app.rs:139 | 0 |
| DefaultPlugins | 01/`DefaultPlugins` | DefaultPlugins | 同形直用 | bevy_internal-0.20.0/src/default_plugins.rs:5 | 0 |
| Commands | 01/`Commands` | Commands 参数 | 同形直用 | bevy_ecs-0.20.0/src/system/commands/mod.rs:268 | 0 |
| Startup 调度 | 01/`Schedule` and `Startup` | Startup 调度 | 同形直用 | bevy_app-0.20.0/src/main_schedule.rs:69 | 0 |
| add_systems | 01/Walkthrough Step 4 | Startup 注册系统 | 同形直用 | bevy_app-0.20.0/src/app.rs:321 | 0 |
| Camera2d | 01/`Camera2d` | Camera2d 组件 | 同形直用 | bevy_camera-0.20.0/src/components.rs:16 | 0 |
| Sprite | 01/`Sprite` and `Vec2` | Sprite 组件 | 同形直用 | bevy_sprite-0.20.0/src/sprite.rs:17 | 0 |
| Sprite::from_color | 01/`Sprite` and `Vec2` | from_color | 同形直用 | bevy_sprite-0.20.0/src/sprite.rs:73 | 0 |
| Color::srgb | 01/Walkthrough Step 5 | Color::srgb | 同形直用 | bevy_color-0.20.0/src/color.rs:123 | 0 |
| Vec2::new | 01/`Sprite` and `Vec2` | Vec2::new | 同形直用 | glam-0.33.12/src/f32/vec2.rs:90 | 0 |

## Part 02 · Drawing a Map

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Res | 02/`Res<T>` and `ResMut<T>` | Res 只读 | 同形直用 | bevy_ecs-0.20.0/src/change_detection/params.rs:543 | 0 |
| ResMut | 02/`Res<T>` and `ResMut<T>` | ResMut 可写 | 同形直用 | bevy_ecs-0.20.0/src/change_detection/params.rs:618 | 0 |
| AssetServer::load | 02/`AssetServer` | 资源异步 load() | 同形直用 | bevy_asset-0.20.0/src/server/mod.rs:364 | 0 |
| TextureAtlasLayout::from_grid | 02/`TextureAtlasLayout` and `TextureAtlas` | 五参 from_grid | 同形直用 | bevy_image-0.20.0/src/texture_atlas.rs:133 | 0 |
| TextureAtlas | 02/`TextureAtlasLayout` and `TextureAtlas` | 图集引用组件 | 同形直用 | bevy_image-0.20.0/src/texture_atlas.rs:214 | 0 |
| Sprite::from_atlas_image | 02/Walkthrough Step 3 | 图集精灵构造 | 同形直用 | bevy_sprite-0.20.0/src/sprite.rs:64 | 0 |
| Assets<TextureAtlasLayout>::add | 02/Walkthrough Step 2 | 资产 add 取句柄 | 同形直用 | bevy_asset-0.20.0/src/assets.rs:401 | 0 |
| Component 派生 | 02/`Component` | 组件派生 | 同形直用 | bevy_ecs-0.20.0/src/lib.rs:76 | 0 |
| Transform::from_xyz | 02/`Transform` | from_xyz 定位 | 同形直用 | bevy_transform-0.20.0/src/components/transform.rs:119 | 0 |
| UVec2::splat | 02/Walkthrough Step 2 | UVec2::splat | 同形直用 | glam-0.33.12/src/u32/uvec2.rs:93 | 0 |

## Part 03 · Tilemap Rendering

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| TilemapPlugin | 03/`TilemapPlugin` | 插件注册 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/lib.rs:62 | 0 |
| TilemapBundle | 03/`TilemapBundle` and `TileBundle` | 地图 Bundle | 同形直用 | bevy_ecs_tilemap-0.20.0/src/lib.rs:116（alias）；:120（结构）；examples/basic.rs:52 | 0 |
| TileBundle | 03/`TilemapBundle` and `TileBundle` | TileBundle 每格 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/tiles/mod.rs:114 | 0 |
| TileStorage | 03/`TileStorage` | empty/set 查表 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/tiles/storage.rs:17（结构）；:32（empty）；:65（set） | 0 |
| TilePos | 03/Walkthrough Step 4 | TilePos{x,y} | 同形直用 | bevy_ecs_tilemap-0.20.0/src/tiles/mod.rs:17 | 0 |
| TilemapId | 03/`TilemapBundle` and `TileBundle` | TilemapId 父实体 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/map.rs:55 | 0 |
| TileTextureIndex | 03/`TileBundle` 列表 | 纹理索引组件 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/tiles/mod.rs:73 | 0 |
| TilemapSize | 03/Walkthrough Step 3 | 尺寸结构 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/map.rs:72 | 0 |
| TilemapTileSize | 03/Walkthrough Step 3 | 格尺寸 f32 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/map.rs:214 | 0 |
| TilemapType::Square | 03/Walkthrough Step 3 | Square 类型 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/map.rs:464 | 0 |
| TilemapTexture::Single | 03/Walkthrough Step 5 | Single 纹理 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/map.rs:124 | 0 |
| TilemapAnchor::Center | 03/Walkthrough Step 5 | Center 锚点 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/anchor.rs:9 | 0 |
| ImagePlugin::default_nearest | 03/`ImagePlugin` | 最近邻过滤 | 同形直用 | bevy_image-0.20.0/src/image.rs:163（结构）；:209（default_nearest） | 0 |
| Commands::spawn_empty | 03/Walkthrough Step 3 | 空实体取 id | 同形直用 | bevy_ecs-0.20.0/src/system/commands/mod.rs:216 | 0 |
| tile_size→grid_size | 03/Walkthrough Step 3 | 尺寸转网格 | 同形直用 | bevy_ecs_tilemap-0.20.0/src/map.rs:247 | 0 |

## Part 04 · Auto-Tiling

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Resource 派生 | 04/Walkthrough Step 2 | Resource 派生 | 同形直用 | bevy_ecs-0.20.0/src/lib.rs:90 | 0 |
| Commands::insert_resource | 04/Walkthrough Step 4 | insert_resource | 同形直用 | bevy_ecs-0.20.0/src/system/commands/mod.rs:801 | 0 |

## Part 05 · Refactoring

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| （无新 API，继承前部） | 05/– | – | – | – | 0 |

## Part 06 · External Level Data

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Resource 派生（serde） | 06/Walkthrough Step 3 | Resource+serde | 同形直用 | bevy_ecs-0.20.0/src/lib.rs:90 | 0 |
| Commands::insert_resource | 06/Walkthrough Step 6 | insert_resource | 同形直用 | bevy_ecs-0.20.0/src/system/commands/mod.rs:801 | 0 |

## Part 07 · Enemies and Path Following

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Update 调度 | 07/`Update` and `FixedUpdate` | Update 每帧 | 同形直用 | bevy_app-0.20.0/src/main_schedule.rs:173 | 0 |
| FixedUpdate 调度 | 07/`Update` and `FixedUpdate` | FixedUpdate 定步 | 同形直用 | bevy_app-0.20.0/src/main_schedule.rs:133 | 0 |
| FixedUpdate 默认步频 | 07/`Update` and `FixedUpdate` | 称 60Hz 默认 | 同形直用 | bevy_time-0.20.0/src/fixed.rs:76（DEFAULT_TIMESTEP=15625µs=64Hz，教程 60Hz 口径需更正） | 0 |
| Query | 07/`Queries` | 组件集遍历 | 同形直用 | bevy_ecs-0.20.0/src/system/query.rs:716（iter_mut） | 0 |
| With<T> | 07/`Queries` | 含组件过滤 | 同形直用 | bevy_ecs-0.20.0/src/query/filter.rs:142 | 0 |
| Without<T> | 07/`Queries` | 排除组件过滤 | 同形直用 | bevy_ecs-0.20.0/src/query/filter.rs:253 | 0 |
| .chain() | 07/`System Chaining` | .chain() 定序 | 同形直用 | bevy_ecs-0.20.0/src/schedule/config.rs:566 | 0 |
| Time::delta_secs | 07/Walkthrough Step 5 | delta_secs() | 同形直用 | bevy_time-0.20.0/src/time.rs:283 | 0 |
| EntityCommands::remove | 07/Walkthrough Step 6 | remove::<T>() | 同形直用 | bevy_ecs-0.20.0/src/system/commands/mod.rs:1702 | 0 |
| EntityCommands::despawn | 07/Walkthrough Step 6 | despawn() | 同形直用 | bevy_ecs-0.20.0/src/system/commands/mod.rs:1882 | 0 |
| Quat::from_rotation_z | 07/Walkthrough Step 5 | from_rotation_z | 同形直用 | glam-0.33.12/src/f32/scalar/quat.rs:192 | 0 |
| Transform.translation | 07/Walkthrough Step 5 | translation 字段 | 同形直用 | bevy_transform-0.20.0/src/components/transform.rs:92 | 0 |

## Part 08 · Tower Placement

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Single | 08/`Single<>` | Single<&T> | 同形直用 | bevy_ecs-0.20.0/src/system/query.rs:2995 | 0 |
| Visibility | 08/`Visibility` | Visible/Hidden | 同形直用 | bevy_camera-0.20.0/src/visibility/mod.rs:83 | 0 |
| ButtonInput<MouseButton> | 08/`ButtonInput<MouseButton>` | just_pressed 判定 | 同形直用 | bevy_input-0.20.0/src/button_input.rs:188 | 0 |
| Window::cursor_position | 08/`Cursor-to-world conversion` | 光标位置取用 | 同形直用 | bevy_window-0.20.0/src/window.rs:620 | 0 |
| Camera::viewport_to_world_2d | 08/`Cursor-to-world conversion` | 屏幕→世界（Result） | 同形直用 | bevy_camera-0.20.0/src/camera.rs:710 | 0 |
| GlobalTransform | 08/Walkthrough Step 6 | 相机世界变换 | 同形直用 | bevy_transform-0.20.0/src/components/global_transform.rs:210 | 0 |
| App::init_resource | 08/Walkthrough Step 8 | init_resource | 同形直用 | bevy_app-0.20.0/src/app.rs:485 | 0 |
| Sprite 结构更新 | 08/Walkthrough Step 5 | 颜色更新构造 | 同形直用 | bevy_sprite-0.20.0/src/sprite.rs:17 | 0 |
| Color::srgba | 08/Walkthrough Step 5 | srgba 半透明色 | 同形直用 | bevy_color-0.20.0/src/color.rs:107 | 0 |

## Part 09 · Targeting and Damage

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Timer | 09/`Timer` and `TimerMode` | 计时与 tick/完成 | 同形直用 | bevy_time-0.20.0/src/timer.rs:61（from_seconds）；:280（tick）；:110（just_finished） | 0 |
| TimerMode | 09/`Timer` and `TimerMode` | Once/Repeating | 同形直用 | bevy_time-0.20.0/src/timer.rs:497 | 0 |
| Without<T> 消歧 | 09/`Query disjointness` | Without 消 B0001 | 同形直用 | bevy_ecs-0.20.0/src/query/filter.rs:253 | 0 |
| Parent / Children | 09/`Parent/child hierarchy` | Parent/Children | 改名/小形态差 | bevy_ecs-0.20.0/src/hierarchy.rs:107（ChildOf）；:152（Children） | 1 |
| with_children | 09/`Parent/child hierarchy` | with_children | 同形直用 | bevy_ecs-0.20.0/src/hierarchy.rs:366 | 0 |
| Commands::entity().despawn | 09/Walkthrough Step 4 | 立即排队 despawn | 同形直用 | bevy_ecs-0.20.0/src/system/commands/mod.rs:1882 | 0 |
| Query::get_mut | 09/Walkthrough Step 4 | 按实体再借 | 同形直用 | bevy_ecs-0.20.0/src/system/query.rs:1858 | 0 |

## Part 10 · Waves and Enemy Spawning

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Time::delta_secs（定步） | 10/`Time::delta_secs()` in FixedUpdate | 定步 delta 累积 | 同形直用 | bevy_time-0.20.0/src/time.rs:283；bevy_app-0.20.0/src/main_schedule.rs:133 | 0 |

## Part 11 · Win/Lose Conditions

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Local<T> | 11/`Local<T>` | Local 系统私有态 | 同形直用 | bevy_ecs-0.20.0/src/system/system_param.rs:1004 | 0 |
| Option<Res<T>> | 11/`Option<Res<T>>` | 资源可缺省 | 同形直用 | bevy_ecs-0.20.0/src/system/system_param.rs:1652；change_detection/params.rs:543 | 0 |
| 零尺寸 Resource | 11/`Marker resources` | 标记资源 GameOver | 同形直用 | bevy_ecs-0.20.0/src/lib.rs:90 | 0 |

## Part 12 · Multi-Level Architecture

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| States 派生 | 12/`States` | States 派生 + 约束 | 同形直用 | bevy_state-0.20.0/src/state/mod.rs:9（派生再导出）；state/states.rs:64（trait） | 0 |
| App::init_state | 12/Walkthrough Step 1 | init_state | 同形直用 | bevy_state-0.20.0/src/app.rs:96 | 0 |
| OnEnter | 12/`OnEnter` and `OnExit` | 进入态一次性调度 | 同形直用 | bevy_state-0.20.0/src/state/transitions.rs:19 | 0 |
| OnExit | 12/`OnEnter` and `OnExit` | 退出态一次性调度 | 同形直用 | bevy_state-0.20.0/src/state/transitions.rs:25 | 0 |
| NextState::set | 12/`NextState<T>` | 延迟状态转移 | 同形直用 | bevy_state-0.20.0/src/state/resources.rs:181（enum）；:198（set） | 0 |
| run_if(in_state(..)) | 12/`.run_if(in_state(...))` | 状态门控运行 | 同形直用 | bevy_state-0.20.0/src/condition.rs:103；bevy_ecs-0.20.0/src/schedule/config.rs:543 | 0 |
| queue_silenced(despawn()) | 12/Walkthrough Step 6 | 静默 despawn | 同形直用 | bevy_ecs-0.20.0/src/system/commands/mod.rs:620；system/commands/entity_command.rs:278；system/mod.rs:143（glob 再导出） | 0 |
| Node | 12/Walkthrough Step 8 | Node 布局组件 | 同形直用 | bevy_ui-0.20.0/src/ui_node.rs:485 | 0 |
| Val::Percent | 12/Walkthrough Step 8 | Val 百分比单位 | 同形直用 | bevy_ui-0.20.0/src/geometry.rs:32 | 0 |
| Text | 12/Walkthrough Step 8 | 文本组件 | 同形直用 | bevy_ui-0.20.0/src/widget/text.rs:128 | 0 |
| TextColor | 12/Walkthrough Step 8 | 文本颜色组件 | 同形直用 | bevy_text-0.20.0/src/text.rs:1405 | 0 |
| Color::WHITE | 12/Walkthrough Step 8 | 具名白色常量 | 同形直用 | bevy_color-0.20.0/src/color.rs:609 | 0 |
| Node 布局枚举 | 12/Walkthrough Step 8 | 对齐/方向枚举 | 同形直用 | bevy_ui-0.20.0/src/ui_node.rs:900（AlignItems）；:1358（JustifyContent）；:1516（FlexDirection） | 0 |
| TextFont.font_size | 12/Walkthrough Step 8 | font_size: f32 | 改名/小形态差 | bevy_text-0.20.0/src/text.rs:688（字段 FontSize）；:783（FontSize 枚举）；SKILL §6.14（bevy-dev/SKILL.md:291） | 1 |
| with_children | 12/Walkthrough Step 8 | with_children | 同形直用 | bevy_ecs-0.20.0/src/hierarchy.rs:366 | 0 |

## Part 13 · Multiple Levels

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| App::init_resource | 13/`init_resource::<T>()` | init_resource | 同形直用 | bevy_app-0.20.0/src/app.rs:485 | 0 |
| KeyCode::Digit1 | 13/Walkthrough Step 6 | Digit1..9 按键 | 同形直用 | bevy_input-0.20.0/src/keyboard.rs:300 | 0 |

## Part 14 · Economy

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Option<&T> 查询 | 14/`Option<&T>` in queries | 可选组件查询 | 同形直用 | bevy_ecs-0.20.0/src/query/fetch.rs:3318 | 0 |
| Text 整组件写 | 14/Walkthrough Step 3 | 整组件覆写 | 同形直用 | bevy_ui-0.20.0/src/widget/text.rs:128（Text(pub String)）；:132（new） | 0 |
| PositionType::Absolute | 14/Walkthrough Step 3 | 绝对定位 | 同形直用 | bevy_ui-0.20.0/src/ui_node.rs:508（字段）；:1763（枚举） | 0 |
| Node top/left | 14/Walkthrough Step 3 | HUD 边距字段 | 同形直用 | bevy_ui-0.20.0/src/ui_node.rs:528（left）；:542（top） | 0 |

## Part 15 · Rocket Launcher

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| GlobalTransform::translation | 15/`GlobalTransform` | 世界坐标取值 | 同形直用 | bevy_transform-0.20.0/src/components/global_transform.rs:210 | 0 |
| Entity 存组件 | 15/`Storing Entity in a component` | 组件内 Entity 指针 | 同形直用 | bevy_ecs-0.20.0/src/lib.rs:77（Entity）；:76（Component 派生） | 0 |
| Query::get() | 15/Walkthrough Step 7 | get 查存活目标 | 同形直用 | bevy_ecs-0.20.0/src/system/query.rs:1722 | 0 |

## Part 16 · Data-Driven Towers

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| （无新 API，继承前部） | 16/– | – | – | – | 0 |

## Part 17 · Tower Selection UI

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Node | 17/`Node` | Node 弹性布局 | 同形直用 | bevy_ui-0.20.0/src/ui_node.rs:485 | 0 |
| ImageNode::from_atlas_image | 17/`ImageNode` | UI 内图集精灵 | 同形直用 | bevy_ui-0.20.0/src/widget/image.rs:99 | 0 |
| Interaction | 17/`Interaction` | 悬停/按下三态 | 改名/小形态差 | bevy_ui-0.20.0/src/focus.rs:50（DeprecatedInteraction）；:65（Interaction 别名）；SKILL §6.14（bevy-dev/SKILL.md:280） | 1 |
| BorderColor 四边 | 17/`BorderColor` | 四字段边框色 | 同形直用 | bevy_ui-0.20.0/src/ui_node.rs:2603（四字段）；:2627（all） | 0 |
| MessageReader<MouseWheel> | 17/`MessageReader<T>` | 滚轮消息读取 | 同形直用 | bevy_ecs-0.20.0/src/message/message_reader.rs:34；bevy_input-0.20.0/src/mouse.rs:214 | 0 |
| MouseWheel 字段 | 17/Input: scroll wheel | x/y 滚动量 | 同形直用 | bevy_input-0.20.0/src/mouse.rs:218（x）；:220（y） | 0 |
| Res::is_changed | 17/Reacting to selection changes | 资源变更检测 | 同形直用 | bevy_ecs-0.20.0/src/change_detection/params.rs:603（change_detection_impl! 为 Res 生成 DetectChanges） | 0 |

## Part 18 · Custom Events

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Message 派生 | 18/`#[derive(Message)]` | 消息类型派生 | 同形直用 | bevy_ecs-0.20.0/src/message/mod.rs:100 | 0 |
| MessageWriter::write | 18/`MessageWriter<T>` | write 广播 | 同形直用 | bevy_ecs-0.20.0/src/message/message_writer.rs:74 | 0 |
| MessageReader::read | 18/`MessageReader<T>` | read 游标消费 | 同形直用 | bevy_ecs-0.20.0/src/message/message_reader.rs:44 | 0 |
| App::add_message | 18/`#[derive(Message)]` | 注册消息队列 | 同形直用 | bevy_app-0.20.0/src/app.rs:427 | 0 |
| .after() 定序 | 18/`System ordering with .after()` | 生产者在前后 | 同形直用 | bevy_ecs-0.20.0/src/schedule/config.rs:404 | 0 |

## Part 19 · Gizmos

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Gizmos | 19/`Gizmos` | Gizmos 系统参数 | 同形直用 | bevy_gizmos-0.20.0/src/gizmos.rs:143 | 0 |
| circle_2d | 19/`circle_2d` | 圆心/半径/色 | 同形直用 | bevy_gizmos-0.20.0/src/circles.rs:167；bevy_math-0.20.0/src/isometry.rs:195（Vec2→Isometry2d） | 0 |

## Part 20 · Audio

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| AudioPlayer | 20/`AudioPlayer` | 音频播放组件 | 同形直用 | bevy_audio-0.20.0/src/audio.rs:251 | 0 |
| PlaybackSettings 预设 | 20/`PlaybackSettings` | 循环/一次性预设 | 同形直用 | bevy_audio-0.20.0/src/audio.rs:100（LOOP）；:106（DESPAWN） | 0 |
| PlaybackSettings::with_volume | 20/`PlaybackSettings` | 音量修饰链式调用 | 同形直用 | bevy_audio-0.20.0/src/audio.rs:130 | 0 |
| Volume::Linear | 20/`PlaybackSettings` | 线性音量值 | 同形直用 | bevy_audio-0.20.0/src/volume.rs:36 | 0 |
| 音频资产加载 | 20/`Loading audio assets` | load 得音频句柄 | 同形直用 | bevy_asset-0.20.0/src/server/mod.rs:364；bevy_audio-0.20.0/src/audio.rs:251 | 0 |

## Part 21 · Plugins and Pause

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Plugin trait | 21/`Plugin` | Plugin::build | 同形直用 | bevy_app-0.20.0/src/plugin.rs:57（trait）；:59（build） | 0 |
| 正交 States | 21/`Orthogonal state machines` | 多 States 并存 | 同形直用 | bevy_state-0.20.0/src/state/mod.rs:9；state/states.rs:64 | 0 |
| .and() 组合子 | 21/`.and()` combinator | 条件布尔与 | 改名/小形态差 | bevy_ecs-0.20.0/src/schedule/condition.rs:135（and_then）；:191（and_eager） | 1 |
| with_child | 21/Walkthrough Step 3 | 单子实体附加 | 同形直用 | bevy_ecs-0.20.0/src/hierarchy.rs:355（EntityWorldMut）；:450（EntityCommands） | 0 |

## Part 22 · Custom Run Conditions

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| 自定义 run 条件 | 22/`Custom run conditions` | fn(参数)->bool | 同形直用 | bevy_ecs-0.20.0/src/schedule/condition.rs:75；schedule/config.rs:543 | 0 |
| SystemSet 派生 | 22/`SystemSet` enum | SystemSet 派生 | 同形直用 | bevy_ecs-0.20.0/src/schedule/set.rs:11 | 0 |
| configure_sets | 22/`configure_sets` and `in_set` | 集合级条件配置 | 同形直用 | bevy_app-0.20.0/src/app.rs:402 | 0 |
| .in_set() | 22/`configure_sets` and `in_set` | 系统归入集合 | 同形直用 | bevy_ecs-0.20.0/src/schedule/config.rs:365 | 0 |

## Part 23 · Level Select Buttons

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| Button | 23/`Button` and `Interaction` | Button 标记组件 | 改名/小形态差 | bevy_ui-0.20.0/src/widget/button.rs:30（弃用别名）；SKILL §6.14（bevy-dev/SKILL.md:280） | 1 |
| Interaction 三态 | 23/`Button` and `Interaction` | 三态交互 | 改名/小形态差 | bevy_ui-0.20.0/src/focus.rs:50（DeprecatedInteraction）；:65（别名） | 1 |
| Changed<Interaction> | 23/`Button` and `Interaction` | 交互变化过滤 | 同形直用 | bevy_ecs-0.20.0/src/query/filter.rs:996 | 0 |
| FlexWrap::Wrap | 23/`FlexWrap` | 换行网格布局 | 同形直用 | bevy_ui-0.20.0/src/ui_node.rs:1788 | 0 |
| BackgroundColor | 23/Walkthrough Step 5 | 背景色组件写 | 同形直用 | bevy_ui-0.20.0/src/ui_node.rs:2576 | 0 |

## Part 24 · Input Abstraction

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| CursorMoved | 24/`CursorMoved` event | 位置/增量消息 | 同形直用 | bevy_window-0.20.0/src/event.rs:192（结构）；:196/:202（字段） | 0 |
| MessageReader | 24/Walkthrough Step 2 | 逻辑输入消息读 | 同形直用 | bevy_ecs-0.20.0/src/message/message_reader.rs:34 | 0 |
| MessageWriter | 24/Walkthrough Step 3 | 逻辑输入消息写 | 同形直用 | bevy_ecs-0.20.0/src/message/message_writer.rs:62 | 0 |
| 虚拟光标资源 | 24/`Virtual cursor as a resource` | 自定义 Resource | 同形直用 | bevy_ecs-0.20.0/src/lib.rs:90 | 0 |

## Part 25 · Gamepad

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| bevy_gilrs 特性 | 25/`bevy_gilrs` feature | Cargo 特性启用 | 同形直用 | bevy-0.20.0/Cargo.toml:2800 | 0 |
| GilrsPlugin | 25/`bevy_gilrs` feature | 默认插件内含 | 同形直用 | bevy_gilrs-0.20.0/src/lib.rs:84 | 0 |
| Gamepad 查询 | 25/`Gamepad` component | Query<&Gamepad> | 同形直用 | bevy_input-0.20.0/src/gamepad.rs:376 | 0 |
| Gamepad::digital() | 25/`Analog stick vs digital buttons` | 数字按键输入 | 同形直用 | bevy_input-0.20.0/src/gamepad.rs:525 | 0 |
| Gamepad::left_stick() | 25/`Analog stick vs digital buttons` | 左摇杆 Vec2 | 同形直用 | bevy_input-0.20.0/src/gamepad.rs:418 | 0 |
| GamepadButton 变体 | 25/`GamepadButton` variants | 方位键/扳机 | 同形直用 | bevy_input-0.20.0/src/gamepad.rs:583（South）；:585（East）；:597（LeftTrigger）；:601（RightTrigger）；:607（Start）；:617（DPadUp） | 0 |

## Part 26 · Epilogue

| API | 出处（part#/节名） | 0.18 形态摘要（≤15字） | 0.20 结论 | 0.20 证据锚 | 摩擦级 |
|---|---|---|---|---|---|
| SpatialListener | 26/`Spatial audio` | 听者组件 | 同形直用 | bevy_audio-0.20.0/src/audio.rs:173 | 0 |
| SpatialAudioBundle | 26/`Spatial audio` | 空间音频 Bundle | 重写调用面 | bevy_audio-0.20.0/src/audio.rs:142（with_spatial）；SKILL §6.14（bevy-dev/SKILL.md:285） | 2 |
| Bundle 派生 | 26/`Custom bundles` | Bundle 派生 | 同形直用 | bevy_ecs-0.20.0/src/bundle/mod.rs:202（trait）；bevy_ecs_tilemap-0.20.0/src/lib.rs:120（存量用例） | 0 |
| DynamicScene | 26/`Scene files` | 动态场景序列化 | 无对应需路线变更 | bevy_scene-0.20.0/src/lib.rs:889-927（模块面）；SKILL §6.13（bevy-dev/SKILL.md:271） | 3 |
| Observer | 26/`Observers` | Observer 响应式系统 | 改名/小形态差 | bevy_ecs-0.20.0/src/observer/distributed_storage.rs:207（本体）；bevy_ecs-0.20.0/src/observer/system_param.rs:25（配套 On 定义）；SKILL §6.7（bevy-dev/SKILL.md:196） | 1 |
| Reflect | 26/`Reflection` | 反射派生与检查 | 同形直用 | bevy_reflect-0.20.0/src/reflect.rs:429；SKILL §6.12（bevy-dev/SKILL.md:254） | 0 |
| TypeRegistry | 26/`Reflection` | 运行期类型注册表 | 同形直用 | bevy_reflect-0.20.0/src/type_registry.rs:448；SKILL §6.12（bevy-dev/SKILL.md:255） | 0 |
| 嵌入资产特性 | 26/`Release build` | 嵌入资产特性 | 重写调用面 | bevy_asset-0.20.0/src/io/embedded/mod.rs:4（embedded_asset! 宏）；bevy-0.20.0/Cargo.toml 无该特性 | 2 |
| Window.icon（缺位断言） | 26/`Window icon` | 无稳定 icon 字段 | 同形直用 | bevy_window-0.20.0/src/window.rs:164（Window 结构无 icon 字段，断言维持） | 0 |

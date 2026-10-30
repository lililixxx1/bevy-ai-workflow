# T051-W1 · M6 塔防波次 5（教程 Part 22–25）· 证据总账

- 任务：T051-W1（M6 塔防波次 5，节点卡 = **26 部执行的最后一个波次**；派工单 = 教程
  `22-custom-run-conditions` … `25-gamepad` 的行为规格）
- 树：`<tree>` = `trees/m6/w5`（基线 = 主仓 `10ab08c` 的 git archive 纯净导出；无 `.git`，物理隔离）
- 日期：2026-10-31
- 结论：**四部（P22–P25）全部 PASS** + **收官段全量回归全绿**。终版构建在**同一新进程**里
  顺序复跑 **p21 / p22 / p23 / p24 / p25** 五个相位，断言合计 **471 条全绿**
  （120 + 51 + 54 + 93 + 153，均 `REAL_EXIT=0`）；三关 `GRID_DIGEST` 与 `ENTITY_DIGEST`
  **逐位零漂移**；`discover` = **41**（23 内置 + 18 `td.*`）；五道门禁绿；游戏进程收尾零残留。
- 口径：证据档自含（命令 + 原始输出 + `REAL_EXIT=` 同档，附录 G）。档内绝对路径为
  cargo/game 原始输出，未做改写；README 统一记 `<tree>` / `<main-repo>`。

---

## 一、判定行汇总（逐部一行结论 + 断言计数）

| 部 | 交付 | 断言档 | SUMMARY | EXIT | 截图 |
| --- | --- | --- | --- | --- | --- |
| **P22** custom-run-conditions | 三条玩法链重复的 `in_state(InGame).and_eager(in_state(Running))` 收敛为**具名 run 条件** `game_is_running`（`pause.rs`）+ **三具名系统集合** `GameplaySet{Simulation, Interaction, TowerDock}`（`configure_sets` 一次挂条件 / `in_set` 只做归属）；**纯重构零行为变化** + 集合成员计数 `SetTickLog`（验证面）+ `td.conditions` | `p22-assert.log`（失败轮：断言设计 bug）· `p22-assert-run2.log`（**51/51**）· `p22-regress-p21.log`（同构建复跑 **120/120** = 行为等价） | **51/51 PASS** | 0 | `p22-shot.png`（85096 B，暂停帧） |
| **P23** level-select-buttons | 单块 `Text` 菜单 → **3 列按钮网格**（`FlexWrap::Wrap` + 640px 定宽）：`LevelButton(i)` / `FocusedLevel` / `navigate_level_select`（2D 网格下标算式 + 数字键 + Enter/Space）/ `update_level_select_visuals`（焦点优先着色）/ `handle_level_button_click`（`Changed<Interaction>`+`Pressed` 一步进关）；`td.level_select`（snapshot / focus / click / confirm） | `p23-assert.log`（**54/54**）· `p23-cross-p22.log`（51/51）· `p23-cross-p21.log`（120/120） | **54/54 PASS** | 0 | `p23-shot.png`（菜单网格 + 焦点高亮） |
| **P24** input-abstraction | 新模块 `input.rs`（`InputPlugin`）：`GameAction` 消息（10 变体）+ `VirtualCursorPos` 资源；物理输入（键盘 / 鼠标 / `CursorMoved` / 滚轮）**单点翻译**；七个消费者重构（选塔 / 菜单导航 / 暂停 / 结算 / 预览 / 放置 / 射程圈）+ 新增 `nudge_virtual_cursor` / `draw_cursor_highlight`；`cycle_tower_on_scroll` 与 `hovered_placeable_tile` 删除；`td.input`（cursor / set_cursor / emit） | `p24-assert.log`（失败轮）· `p24-assert-run2.log`（**93/93**）· `p24-cross-p23.log`（54/54）· `p24-cross-p22.log`（51/51）· `p24-cross-p21.log`（失败轮，见 §八）· `p24-cross-p21b.log`（新进程 **120/120**） | **93/93 PASS** | 0 | `p24-shot.png`（预览 + 光标高亮） |
| **P25** gamepad | `input.rs` 追加手柄读取器：`map_gamepad_button`（9 绑定纯映射表）/ `read_gamepad`（`digital().get_just_pressed()`）/ `stick_to_action`（死区 0.3 + 主导轴，纯函数）/ `read_gamepad_stick`（150ms 重复计时器 + `was_idle`）；`td.gamepad`（snapshot / probe_button / probe_stick）；**收官段全量回归** | `p25-assert.log`（**153/153**，含全量回归） | **153/153 PASS** | 0 | `p25-shot.png`（PAUSED 冻结帧） |

**收官段（P25 后，同一新进程顺序执行）**：

| 相位 | 面 | 断言 | EXIT |
| --- | --- | --- | --- |
| `p25-assert-p21.log` | P13–P21 全量既有面（注册表 / 选择 UI / 事件 / gizmo / 音频 / 暂停 / 弹道 / 经济 / 三关 digest / ENTITY_DIGEST） | **120/120** | 0 |
| `p25-assert-p22.log` | P22 条件面 + 集合计数 | **51/51** | 0 |
| `p25-assert-p23.log` | P23 按钮网格 / 焦点 / 点击 / 确认 / 键盘通道 | **54/54** | 0 |
| `p25-assert-p24.log` | P24 动作注入 / 光标走格 / 放置 / 选塔 / 暂停 / 菜单导航 | **93/93** | 0 |
| `p25-assert.log` | P25 手柄结构面 + 无设备零影响 + 全量回归 | **153/153** | 0 |
| **合计** | | **471/471** | 全 0 |

## 二、证据档结构

```
docs/evidence/t051-m6-wave5/
├── README.md                     # 本档（docs/src/lib.rs 未 include 本目录，不进 doctest）
├── assert-td.mjs                 # 断言脚本（自 t050 档复制后追加 p22–p25；t047–t050 原件未动）
├── toml-level-expand.mjs         # 独立 TOML 展开器（自 t050 档原样复制，未改）
├── chains.log                    # 机器汇总（逐相位 SUMMARY/REAL_EXIT、discover 表、digest、md5）
├── p22-check.log / p22-build.log / p22-run.log        # P22 构建态 check / release build / 跑档
├── p22-assert.log                # 失败轮：50/51（断言设计 bug，见 §八）
├── p22-assert-run2.log           # 51/51（修正后）
├── p22-regress-p21.log           # 同构建复跑 p21：120/120（行为等价正面证据）
├── p22-shot.png                  # P22 在局截图
├── p23-check.log / p23-build.log / p23-run.log
├── p23-assert.log                # 54/54
├── p23-cross-p22.log / p23-cross-p21.log            # 跨部回归（51/51、120/120）
├── p23-shot.png                  # 菜单按钮网格 + 焦点高亮
├── p24-check.log                 # 补录档（见档头声明与 §八）
├── p24-build.log / p24-run.log
├── p24-assert.log                # 失败轮：91/92（断言设计 bug，见 §八）
├── p24-assert-run2.log           # 93/93
├── p24-cross-p23.log / p24-cross-p22.log            # 54/54、51/51
├── p24-cross-p21.log             # 失败轮：119/120（进程级计数器，见 §八）
├── p24-cross-p21b.log            # 新进程复跑：120/120
├── p24-cross-run.log / p24-run.log                  # 跑档（含 taskkill 终止声明）
├── p24-shot.png                  # 预览 + 光标高亮
├── p25-check.log / p25-build.log / p25-run.log      # 终版构建（build 首轮 101 = exe 被占用，见 §八）
├── p25-assert.log                # 153/153（P25 + 收官全量回归）
├── p25-assert-p21.log / p25-assert-p22.log / p25-assert-p23.log / p25-assert-p24.log
├── gate-check-workspace.log      # 门禁 1：check --workspace（REAL_EXIT=0）
├── gate-test-workspace.log       # 门禁 2：test --workspace --test-threads=3（REAL_EXIT=0）
├── gate-doc-test-t051.log        # 门禁 3：test --doc -p docs（78/0/17，REAL_EXIT=0）
├── gate-claim-lint.log           # 门禁 4：claim-lint 13/13（REAL_EXIT=0）
├── scope-check.log               # 写范围核对（树 vs 主仓 HEAD 10ab08c；--strip-trailing-cr）
└── process-check.log             # 收尾进程核查（grep_exit=1 = 零残留）
```

断言脚本用法（树根执行；游戏进程 CWD = `<tree>/game-td`）：

```
node docs/evidence/t051-m6-wave5/assert-td.mjs --phase p22 --expect-discover 38 --shot ../docs/evidence/t051-m6-wave5/p22-shot.png
node docs/evidence/t051-m6-wave5/assert-td.mjs --phase p23 --expect-discover 39 --shot ../docs/evidence/t051-m6-wave5/p23-shot.png
node docs/evidence/t051-m6-wave5/assert-td.mjs --phase p24 --expect-discover 40 --shot ../docs/evidence/t051-m6-wave5/p24-shot.png
node docs/evidence/t051-m6-wave5/assert-td.mjs --phase p25 --expect-discover 41 --shot ../docs/evidence/t051-m6-wave5/p25-shot.png
# 历史相位（复跑口径同 T050）：--phase p21 --expect-discover 41 --shot-pause …
```

游戏启动（release 构建；每部唯一差异 = 日志名；全卡不经 `cargo run`，遵守「游戏存活期间不起 cargo」）：

```
$ cd <tree>/game-td && BEVY_ASSET_ROOT=<tree>/game-td <main-repo>/target/release/game-td.exe > <log> 2>&1 &
$ for i in $(seq 1 200); do curl -s -m 1 --fail -o /dev/null -X POST http://127.0.0.1:15702 \
      -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"rpc.discover","id":1}' && break; sleep 0.1; done
$ node docs/evidence/t051-m6-wave5/assert-td.mjs --phase <p> ...
$ taskkill //F //IM game-td.exe
```

`BEVY_ASSET_ROOT` 沿用 SKILL §6.18。**历史相位 p21 需在新进程首跑**（其 `spawned === 1`
为进程级绝对计数，见 §八/U4）。

## 三、指纹链（与往期档逐位对拍）

### 3.1 三关 GRID_DIGEST（终版构建 p25 相位一次过）

算式 = 断言脚本 `gridDigest()`（逐格 `(x,y) -> (TileType, atlas 索引)` 排序后 sha256；与 T047/T048/T049/T050 同式）。

| 关卡 | digest | 在链断言（PASS） |
| --- | --- | --- |
| level_01 | `70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07` | T047 冻结基线；p22/p23/p24/p25 四处 `assertMap` 同值 |
| level_02 | `3edf8dffc55b743933c8c4b8484f50855b8e7d193495d5f4c2536f96ea8a4f6d` | T049 P13 登记；p25 同值 |
| level_03 | `840298ef16970017e1916a573a5333759bea8782a959509d365c12fc9dcd76f7` | T049 P13 登记；p25 同值 |

### 3.2 ENTITY_DIGEST（P11/P12/T048 冻结链）

口径沿 T048/T049/T050：`sha256(排序行)` = 塔基座世界坐标 `T:x,y` + 塔顶计数 `TT:n` +
在场敌人血量多重集 `E:hp`；场景 = 1 座无害塔 `[2,0]`（dy=320 > 射程 192）+ 波次时钟 ∈ [2.2,3.0)。

```
T048 冻结基线        1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23
P25（终版构建）      1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23   ← 逐位一致
```

即：系统集合重构 / 菜单按钮网格 / 输入抽象层 / 手柄接入**全未触碰**塔/敌实体面；
elapsed 落在采样窗口内（p24 相位另留 `[7,7]` 放置链的 gold 独立复算）。

### 3.3 行为面采样（P22–P25）

```
P22 条件      LevelSelect: game_is_running=false，三集合 tick 全冻结（0 -> 0）
              InGame+Running: true，三集合均推进（例 sim 18->45 / interaction 17->42）
              Paused: false，三集合冻结（43/46/43 -> 同值）；elapsed 0.71875 冻结、gold 302.15625 冻结
              恢复: true，三集合继续推进
P23 菜单      3 关 → 3 按钮（index [0,1,2]），进菜单焦点 = 0，恰 1 个 focused
              着色独立复算：焦点 (0.1,0.3,0.5,1) / 悬停 (0.2,0.25,0.35,1) / 按下 (0.05,0.15,0.3,1) / 默认 (0.15,0.15,0.15,1)
              click(index 2) → 真实消费者 → InGame，selected=level_03，MapTile=150
              confirm（焦点 1）→ InGame，selected=level_02
P24 输入      光标播种 [7,5]（路径格，placeable=false）→ Up→[7,6] → Up→[7,7]（草地，placeable=true）
              预览圈中心 (-64,160) = 独立算式 OFFSET_X+6*64 / OFFSET_Y+7*64；光标高亮格跟随
              clamp：y→0 / x→0 / y→9 / x→14（各方向越界后保持）
              Confirm 于 [5,7] → PlaceTower 一条，produced/spawned/gold_applied 各 +1，
                spawn_last.tile=[5,7]，gold_last.cost=200；gold 变化 = 被动收入 - 200（独立复算，容差 0.2）
              Confirm 于路径格 [7,5] → 不产生消息（placeability 门）
              SelectTower(2)/NextTower(clamp)/PrevTower/SelectTower(0) 全部符合；Cancel → Paused → Running
              菜单方向动作：Right→1，Down(单行)→1，Right→2，Right(clamp)→2，Left→1，Up→1；
                SelectLevel(2) → level_03；Confirm（焦点重置 0）→ level_01
P25 手柄      gamepads=0（本机无手柄）；gamepad_input_module=true（GamepadEvent 消息已注册）
              bindings = 9 条，与教程 25 Step 2 表逐项一致；stick 参数 dead_zone=0.3 / repeat=0.15
              probe_button 9 条全对；North（未绑定）→ null
              probe_stick：(0.1,0.1)/(0.2,-0.2)→null（死区）；(±0.5,0.1)→Left/Right；
                (±0.1,±0.5)→Up/Down；(0.4,-0.7)→Down（主导轴 y）；(-0.8,-0.2)→Left
              无设备零影响：空闲 800ms，光标 / 选中下标 / 暂停态全部不动
```

## 四、截图链（md5，原始输出见 `chains.log`）

| 文件 | md5 | bytes | 判定 |
| --- | --- | --- | --- |
| `p22-shot.png` | `1ea73585ae16c59a60646f323ed04774` | 85096 | P22 构建：在局（暂停覆盖层帧） |
| `p22-regress-shot-pause.png` | `11ff7ab931703f02b38bb07312b72b3b` | 87535 | P22 构建 p21 复跑：PAUSED |
| `p23-shot.png` | `7681953c9e42b0598bdb7e6f6f34e4a4` | 33789 | **菜单 3×1 按钮网格 + 焦点高亮**（静态屏 → 跨轮 md5 相同：p23/p24-cross/p25-regress 三处同值） |
| `p23-cross-shot-pause.png` | `cb954ea07143c6473827c90c5a3f3066` | 86489 | 跨部回归 PAUSED |
| `p24-shot.png` | `fbccfcfd731822790741c80f836e6ded` | 97834 | 预览 + 光标高亮（虚拟光标驱动） |
| `p24-cross-shot-pause.png` | `bb186134c4cd1d0fbcefe9dcd7f7b156` | 86576 | 跨部回归 PAUSED |
| `p25-shot.png` | `a57aee1ac128f4c511cceba5965eb747` | 86480 | 终版 PAUSED 冻结帧（与 `p24-cross-p21b-shot-pause.png` 同值 → 同场景确定性） |

敌方位置 / 交战帧不定，故在局截图 md5 逐轮不同属预期；本卡以断言计数、digest、
集合计数与指针链为判定面，截图为视觉留证。

## 五、`td.*` 方法面与 discover 数（逐部 +1 如实记录）

| 部 | 新增方法 | discover 总数 | 语义 |
| --- | --- | --- | --- |
| 基线（T050） | `td.screenshot` … `td.pause`（14 个自定义） | 37 | — |
| P22 | `td.conditions` | **38** | `snapshot`：`game_state` / `pause_state` / `game_is_running`（**直接 `run_system_once` 运行具名条件函数本身**）/ 三集合计数 `ticks`；未知 op → -32602 |
| P23 | `td.level_select` | **39** | `snapshot`（按钮下标 / 焦点 / `Interaction` / 背景色，逐按钮）；`focus`；`click`（写 `Interaction::Pressed` + `run_system_once` 真实消费者）；`confirm`（Enter/Space 共用 `apply_level_selection`）；非 LevelSelect 受理写通道 → -32602 |
| P24 | `td.input` | **40** | `cursor`（虚拟光标 + `placeable`）；`set_cursor`（越界 / 无 MapLayout → -32602）；`emit`（10 种 `GameAction`，返回动作名回显）；未知 action / op → -32602 |
| P25 | `td.gamepad` | **41** | `snapshot`（连接数 / `gamepad_input_module` / bindings 9 条 / 摇杆节拍参数）；`probe_button`（纯映射）；`probe_stick`（死区 + 主导轴）；未知 button / 缺 x,y → -32602 |

终版方法名清单（`p25-run.log` 原始行）：
`td.screenshot, td.spawn_tower, td.wave, td.game, td.state, td.level, td.economy, td.combat,
td.towers, td.tower_select, td.events, td.gizmos, td.audio, td.pause, td.conditions,
td.level_select, td.input, td.gamepad`（18 个自定义 + 23 内置 = 41）。
全部 `td.` 前缀、无 `game.` 前缀（脚本逐相位断言）；`tooling/mcp-bridge` 未动。

## 六、终档门禁（五行）

```
1) cargo check --workspace -j 3                     → REAL_EXIT=0，0 错 0 警        （gate-check-workspace.log）
2) cargo test --workspace -j 3 -- --test-threads=3  → REAL_EXIT=0；game 20/20、mcp-bridge 15/15、
                                                      game-td 0、doctest 78 passed / 0 failed / 17 ignored（= 基线）
                                                                                    （gate-test-workspace.log）
3) cargo test --doc -p docs -j 3 -- --test-threads=3 → 78/0/17，REAL_EXIT=0        （gate-doc-test-t051.log；
                                                      供 claim-lint 的 gate-doc-test* 挑选——本树先跑此档再跑 claim-lint）
4) cargo run -p claim-lint                          → 13/13 PASS，REAL_EXIT=0     （gate-claim-lint.log；
                                                      doctest-gate 取到本卡 gate-doc-test-t051.log：78/0/17）
5) 游戏进程收尾零残留                                → tasklist | grep -iE 'game-td|rustc|cargo' = 零命中
                                                      （GREP_EXIT=1）               （process-check.log）
```

所有 cargo 命令 `-j 3`、测试 `--test-threads=3` 透传、`CARGO_TARGET_DIR=<main-repo>/target`；
游戏存活期间未起任何 cargo（每部流程 = 编译 → 起 exe → 断言 → taskkill）。cargo 段全局互斥。

> **取证补录说明**：`p24-check.log` 为补录档（P24 构建态当轮 check 仅控制台执行，档头已声明；
> 内容 = 终版树复跑，与 `p25-check.log` 同源）。`p25-build.log` 首轮 `REAL_EXIT=101`
> （exe 被上一进程占用，os error 5）为失败轮，已留档后整句重跑为 0（见 §八）。

## 七、依赖 / 席位 / 文件清单

- **新增依赖：无**。根 `Cargo.toml` / `game-td/Cargo.toml` / `Cargo.lock` **零差异**。
  手柄走 bevy 默认特性链（`2d → default_platform → bevy_gilrs`，
  `bevy-0.20.0/Cargo.toml:2706`/`:2887`/`:2800`；`bevy_internal-0.20.0/src/default_plugins.rs:82-83`
  在 `#[cfg(feature = "bevy_gilrs")]` 下挂 `GilrsPlugin`）——**无需改 Cargo 任何行**。
- **系统席位（能力席表 `docs/m6-game-selection.md` §四，上限 10）**：P22–P25 全部**复用既有席**，
  **未新开第 11 席**：
  - P22 条件调度 → 既有调度基础设施（改三组既有链的门控方式，**不新增能力域**；派工单口径记 #2）；
  - P23 按钮网格 → **#7 UI/HUD**；
  - P24 输入抽象 → 收敛既有输入面（放置 → #4 塔 / 菜单 → #7 / 暂停门控 → 既有），不外开新席；
  - P25 手柄 → 同上（`input.rs` 为物理输入单点翻译层，能力归属沿用既有席）；
  - 全卡触及 **#8 `td.*` 方法面**（+4 → 18）与 **#9 断言面**（+4 相位）。
- **`add_systems` 调用点 / 系统数**（`main` + `audio` + `pause` + `input`）：
  - 基线（T050 收尾）：**19 调用点 / 46 系统**（main 12/39 + audio 4/4 + pause 3/3）；
  - P22：19 / **49**（+3 集合计数微系统，挂既有三链尾，**调用点不变**）；
  - P23：19 / **51**（菜单 Update 组 1 → 3 系统，调用点不变）；
  - P24：**21** / **56**（删 `cycle_tower_on_scroll` −1；+`nudge_virtual_cursor`、`draw_cursor_highlight`
    共 +2；`InputPlugin` +4 系统 / +2 调用点）；
  - P25：**22** / **58**（`read_gamepad` / `read_gamepad_stick` +2 系统 / +1 调用点）。
  - 实测核对：`grep -c 'add_systems('` = main 12 / audio 4 / pause 3 / input 3 → 22 调用点。
- **文件清单（树内相对路径）**：
  - 改：`game-td/src/main.rs`（P22 条件/集合接线 + P23 菜单三系统 + P24 插件与系统迁位 +
    光标资源回收）、`game-td/src/pause.rs`（`game_is_running` / `GameplaySet` / `SetTickLog` /
    三计数系统 / `toggle_pause` 改读动作）、`game-td/src/level_select.rs`（按钮网格 + 焦点导航 +
    点击消费者 + `apply_level_selection`）、`game-td/src/tower.rs`（`VirtualCursorPos` /
    `tile_is_placeable` / `nudge_virtual_cursor` / `draw_cursor_highlight` / 预览与放置与射程圈改读光标 /
    选塔改读动作 / 删 `cycle_tower_on_scroll` 与 `DOCK_KEYS`）、`game-td/src/game_over.rs`（改读动作）、
    `game-td/src/brp.rs`（+4 方法 + `LevelButton` 反射注册 + `cursor_highlight` 字段）
  - 增：`game-td/src/input.rs`（P24/P25 输入抽象层）、本证据目录 `docs/evidence/t051-m6-wave5/**`
  - 未动：`game-td/src/{map,tiling,level,enemy,economy,audio}.rs`、`game-td/assets/**`、
    `Cargo.toml` / `Cargo.lock`、`tooling/**`、`assets-methodology/**`、`bevy-dev/**`、
    `docs/` 其余文件、主仓任何路径（`scope-check.log`：仅 6 改 + 1 增，全在 `game-td/src/`）

## 八、跑档披露（失败轮 / 旧轮全留）

| 轮 | 现象 | 归因 | 处置 |
| --- | --- | --- | --- |
| P22 assert1 | 50/51（`Interaction tick >= Simulation tick` FAIL：25 < 27） | **断言设计 bug（本卡）**：`Simulation` 在 FixedUpdate（64Hz 定步），`Interaction` 在 Update（渲染帧率），二者速率不必相等 | 留档 `p22-assert.log`；改为「两个 Update 集合计数逐值同步」→ `p22-assert-run2.log` 51/51 |
| P23 | 首轮 54/54 即绿 | — | 终档 |
| P24 assert1 | 91/92（金币差 198.78 ≠ cost 200） | **断言设计 bug（本卡）**：gold 期间还叠加被动收入 3/s | 留档 `p24-assert.log`；改「gold 变化 = 被动收入 − cost」独立复算（容差 0.2）→ 93/93 |
| P24 cross-p21 | 119/120（`spawned === 1` FAIL，实为 3） | **断言面进程作用域**：`PlacementLog` 为进程级累计（`init_resource` 一次，不随关卡重置），同一进程先跑 p24（两次放置）后跑 p21，绝对计数不再是 1 | 留档 `p24-cross-p21.log`；**新进程先跑 p21** → `p24-cross-p21b.log` 120/120（U4 建议入复验清单） |
| P25 build1 | `REAL_EXIT=101`（`failed to remove ... game-td.exe`，os error 5 拒绝访问） | **纪律偏离（本卡）**：P24 跨部回归用的游戏进程仍在存活时发起了编译——违反「游戏进程存活期禁编译」；同窗口另有 2 次 `cargo check -p game-td`（check 不产出 exe，未被锁阻塞） | 立即 `taskkill` 清杀并核零残留，再整句重跑 → `p25-build.log` 为 0；断言面未受影响（失败构建在链接前被锁阻塞，无半成品产物；终版断言均在后续新进程执行） |
| 历史 p4–p20 相位 | 不强行原样复跑 | 源码态与终版不同（塔价 / 注册表序 / 集合门控），沿 T049/T050 M2 口径 | 其断言面由 p21/p22/p23/p24/p25 以 digest + 指纹 + 新断言覆盖 |

**可复跑口径**：`pN-assert*.log` 对应各部构建（`pN-build.log` 即源码态）；
终版（P25）构建的复跑档为 `p25-assert-p21/p22/p23/p24.log` + `p25-assert.log`。

## 九、漂移表对照注记（教程 0.18 → bevy 0.20；底稿 `docs/evidence/t044-td-spike/w2/api-drift.md`）

逐部按表核过；**表内结论全部成立**（Part 22 四项 / Part 23 五项 / Part 24 四项 / Part 25 六项，
摩擦级 0 或 1），**表外新事实 8 项（K1–K8）**。实现处注释逐条带 0.20 本地源码锚：

- **Part 22**（表内四项全 0 级）：`SystemCondition`（`bevy_ecs-0.20.0/src/schedule/condition.rs:75`；
  普通 `fn(Res…) -> bool` 经 blanket impl 满足条件——`:600`）、`SystemSet` 派生
  （`bevy_ecs-0.20.0/src/schedule/set.rs:11`）、`configure_sets`（`bevy_app-0.20.0/src/app.rs:402`）、
  `.run_if`（`bevy_ecs-0.20.0/src/schedule/config.rs:543`）、`.in_set`（`:365`）。
  **实核补充**：`App::configure_sets` 接受**元组**（`(SetA, SetB).run_if(cond)` 两集合同挂一条件）；
  `.chain().in_set(...)` 同形成立。
- **Part 23**（表内五项：`Button`/`Interaction` 摩擦级 1，其余 0）：`FlexWrap::Wrap`
  （`bevy_ui-0.20.0/src/ui_node.rs:1788`）、`BackgroundColor`（`:2576`）、`BorderColor::all`（`:2627`）、
  `Changed<Interaction>`（`bevy_ecs-0.20.0/src/query/filter.rs:996`）、`BorderRadius::all`
  （`ui_node.rs:3005`；`From<Val> for CornerRadius` `geometry.rs:1445`）、`UiRect::all`（`geometry.rs:751`）。
  **K1（表外新事实，阻塞级）**：`Button` 在 0.20 是 `pub(crate) struct DeprecatedButton` 的
  **类型别名**（`bevy_ui-0.20.0/src/widget/button.rs:30`）——可作类型名，**不可作构造值**
  （`Button` / `Button::default()` 均 E0603「private type」）。其等价面 = 两条 `#[require]`
  （`Node` + `FocusPolicy::Block` + `Interaction`，`button.rs:24-28`）；`ui_focus_system` 只认
  `Option<&mut Interaction>`（`focus.rs:147-158`）→ 以 `Interaction::None` + `FocusPolicy::Block` 落位。
- **Part 24**（表内四项全 0 级）：`CursorMoved{window,position,delta}`
  （`bevy_window-0.20.0/src/event.rs:192`/`:196`/`:202`；消息注册 `bevy_window-0.20.0/src/lib.rs:115`）、
  `MessageReader`（`message/message_reader.rs:34`）、`MessageWriter`（`message/message_writer.rs:62`）、
  虚拟光标 = 普通 `Resource`（`bevy_ecs-0.20.0/src/lib.rs:90`）。
  **实核补充**：`gizmos.rect_2d(impl Into<Isometry2d>, Vec2, color)`
  （`bevy_gizmos-0.20.0/src/gizmos.rs:898`）+ `From<Vec2> for Isometry2d`
  （`bevy_math-0.20.0/src/isometry.rs:195`）。
- **Part 25**（表内六项全 0 级）：`bevy_gilrs` 特性（`bevy-0.20.0/Cargo.toml:2800`；属默认链
  `2d:2706 → default_platform:2887`）、`GilrsPlugin`（`bevy_gilrs-0.20.0/src/lib.rs:84`；
  `default_plugins.rs:82-83`）、`Gamepad`（`bevy_input-0.20.0/src/gamepad.rs:376`）、
  `Gamepad::digital()`（`:525`）、`Gamepad::left_stick()`（`:418`）、`GamepadButton` 变体
  （`:581` 起，19 标准键 + `Other(u32)`，`all()` 于 `:612`）。
- **§6.17 type_path 冻结**：本卡新增反射组件仅 `LevelButton`（`#[type_path = "game_td"]` +
  `#[reflect(Component)]`，沿 `TowerDockSlot` 先例）；`VirtualCursorPos` / `SetTickLog` /
  `GameAction` 均只经受 typed query / 资源读，不进反射面；既有路径未漂移。

### 九·续 K2–K8（表外新事实，均本卡实测）

- **K2**：`World::run_system_once` 对**新实例**的 `Changed<T>` 过滤恒真——`initialize` 把
  `last_run` 置为 `world.change_tick().relative_to(Tick::MAX)`
  （`bevy_ecs-0.20.0/src/system/function_system.rs:781`）。本卡据此用「写 `Interaction::Pressed`
  + `run_system_once(handle_level_button_click)`」把**真实点击消费者**接进确定性断言通道。
- **K3**：`bevy_gilrs` 已在 bevy 默认特性图内 → 手柄**零 Cargo 改动**；无手柄时
  `Query<&Gamepad>` 为空 → 读取器零动作（零影响回归实测）。
- **K4**：`FixedUpdate` 定步 64Hz（`bevy_time-0.20.0/src/fixed.rs:76`）与 `Update` 渲染帧率
  不同 → 用「集合计数差」做断言时，**不可**假设两个调度的计数速率相等（P22 失败轮教训）。
- **K5**：`PlacementLog` / `AudioLog` 等验证面计数为**进程级累计**（`init_resource` 一次，
  `cleanup_level` 不重置）→ 历史相位里的绝对计数断言（`spawned === 1`）只在**新进程首跑**成立。
- **K6**：`Gamepad` 可实现 `Default`（`gamepad.rs:545`）且 `digital_mut()` 公开（`:530`），
  但**模拟轴无公开写入口**（`analog: Axis<GamepadInput>` 私有）→ 摇杆行为面只能走纯函数探测。
- **K7**：`Interaction` 三态由 `ui_focus_system` 在 `PreUpdate` **每帧按光标覆写**
  （`bevy_ui-0.20.0/src/focus.rs:167-177`）→ 外部写入后跨帧等待不可靠（必须当帧消费，见 K2）。
- **K8**：BRP 侧颜色快照用 `Color::to_srgba()`
  （`bevy_color-0.20.0/src/color.rs:95`；`From<Color> for Srgba` `:685`）——f32 分量逐位可对拍。

## 十、与教程实现形态的偏离（行为口径说明，逐条显式登记）

1. **`game_is_running` / `GameplaySet` 放 `pause.rs`**（教程 22 放 `state.rs`）：本工程无
   `state.rs`（`GameState` 在 `main.rs`，P12 起既定），暂停域自含模块持有 `PauseState`——
   沿 P21 同款偏离注记，行为零变化。
2. **P23 用 `Interaction::None` + `FocusPolicy::Block` 替代 `Button` 标记**：`Button` 在 0.20
   不可构造（K1，E0603）；其全部引擎作用即这两条 `#[require]`，`ui_focus_system` 只认
   `Interaction`——三态跟踪行为与教程一致。A' 裁决精神保留（仍用 `Interaction` +
   `#[expect(deprecated, reason = …)]` 多行属性，**未换 `ui_widgets` 新 API 重写**）。
3. **P23 焦点重置用 `init_resource::<FocusedLevel>()` + `scan_available_levels` 归零**
   （教程用 `commands.insert_resource`）：行为等价（每次进菜单回到 0），且避免「资源只在
   OnEnter 插入」时视觉系统早帧读不到的窗口。
4. **P23 鼠标通道的断言替代**：BRP 无鼠标注入，`td.level_select click` 写 `Interaction::Pressed`
   后 `run_system_once` 运行**真实** `handle_level_button_click`（`Changed` 过滤与选中逻辑全走
   真码，K2）；`bevy_ui` 的**屏幕命中测试**本身不在断言面（同 P17 槽位点击的既有边界）。
5. **P24 `cycle_tower_on_scroll` 删除、逻辑并入 `select_tower_by_key`**：教程 24 Step 6 表标
   `deleted`；滚轮→动作的翻译在 `read_mouse_wheel`，两端 clamp（不回绕）语义在原消费者保留。
6. **P24 `read_mouse_wheel` 保持「每帧只取首条事件」**（原 P17 行为），未按事件逐条叠加步进
   ——行为保真优先于「逐事件」的另一种合理写法。
7. **P24 `place_tower_on_click` 保留教程函数名**（虽已非「点击」）：确认动作来源 = 左键 /
   Enter / Space / 手柄 South；改名会扩大 diff 且教程 24 Step 6 表未改名。
8. **P25 摇杆判定体抽为纯函数 `stick_to_action`**（教程内联在 `read_gamepad_stick`）：
   行为同教程 Step 3（死区 0.3 / 主导轴），抽函数仅为无硬件时的可断言面（同 `td.gizmos probe` 先例）。
9. **验证面扩展（本工程，非教程）**：`SetTickLog`（三集合计数）、`RangeGizmoLog.cursor_tile`
   （光标高亮格）、`td.conditions` / `td.level_select` / `td.input` / `td.gamepad`——只服务断言，
   不给玩法逻辑增加语义（口径同 `PlacementLog` / `RangeGizmoLog` / `AudioLog`）。
10. **P22 三个集合计数微系统挂在既有三链尾**（不新增 `add_systems` 调用点）：为让
    「集合条件是否真的生效」成为可观察面（若 `configure_sets` 漏挂，LevelSelect / 暂停中计数
    仍会推进）。

## 十一、上报 / 未决（U）

- **U1（文档同步，待裁决）**：`docs/m6-game-selection.md` §五 的 `td.*` 方法面描述仍停在早期
  「初始方法 = `td.screenshot`」口径（T046 定本；T050 U1 已提出未关）。现方法面 = 18 个
  （T044 1 + T047–T051 17）。**选项**：(a) Lead 在收获批统一更新 §五（推荐，单点维护）；
  (b) 每卡自更新（本卡未做，避免越出写范围）。
- **U2（席位口径，待裁决）**：派工单把「条件调度」记作 #2「调度席」、「输入抽象与手柄」记作
  #3「输入席」，而 `docs/m6-game-selection.md` §四 的 #2 = tilemap 地图/路径、#3 = 敌人系统。
  本卡按**实质约束**执行：P22–P25 全部复用既有席，**未新开第 11 席**（席位表 10/10 维持）。
  请裁决是否需要统一「席位昵称 vs 表名」口径。
- **U3（新知识入库，待裁决）**：K1（`Button` 不可构造）/ K2（`run_system_once` + `Changed`）/
  K4（64Hz vs 渲染帧率）/ K5（进程级计数）/ K6（摇杆无公开写入口）/ K7（`Interaction` 每帧覆写）
  等是否入库 `bevy-dev/pitfalls.md`（PIT-B）或 SKILL §6——入库需同步
  `tooling/claim-lint` 的 expect 表与 `docs/recheck-registry.md` 联动，超出本卡写范围。
- **U4（复验清单建议）**：含 `p21` 相位的复跑必须**新进程首跑**（K5）；本卡已留失败轮
  `p24-cross-p21.log` + 新进程绿档 `p24-cross-p21b.log`。建议与 T050 U5（archive 树 mtime
  陷阱）并列写入补位/复验清单。
- **U5（无手柄环境）**：本机 `gamepads=0`；P25 行为面按立项「映射表 + 模拟路径」落地
  （9 条绑定 + 8 组摇杆向量 + 无设备零影响），**无实机手柄档**——环境限制如实注记，
  非收官门槛（`docs/m6-game-selection.md` §四/§八 未把外设列入门槛）。

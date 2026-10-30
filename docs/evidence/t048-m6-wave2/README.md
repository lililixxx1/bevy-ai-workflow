# T048-W1 · M6 塔防波次 2（教程 Part 8–12）· 证据总账

- 任务：T048-W1（M6 塔防波次 2，节点卡；派工单 = 教程 `08-tower-placement` … `12-multi-level-architecture` 的行为规格）
- 树：`<tree>` = `trees/m6/w2`（基线 = 主仓 `49657ec` 的 git archive 纯净导出；无 `.git`，物理隔离）
- 日期：2026-10-30（晚）
- 结论：**五部（P8–P12）全部 PASS**——六次断言运行共 **261 条断言全绿**（35+30+35+41+37+83），
  两道门禁绿（`cargo check --workspace` 0 错 0 警；`cargo test --workspace` REAL_EXIT=0，doctest 基线 78/0/17 恢复），
  地图层 `GRID_DIGEST` 自 P8 到 P12（含 P12 重开）**逐位零漂移**，塔/敌计数指纹 P11 ≡ P12，
  游戏进程收尾零残留。
- 口径：证据档自含（命令 + 原始输出 + `REAL_EXIT=`/`GAME_EXIT=` 同档，附录 G）。
  档内绝对路径为 cargo/winit/game 原始输出，未做改写；README 与结论行统一记 `<tree>` / `<main-repo>`。

---

## 一、判定行汇总（逐部一行结论 + 断言计数）

| 部 | 交付 | 断言日志 | SUMMARY | REAL_EXIT | 截图 |
| --- | --- | --- | --- | --- | --- |
| **P8** tower-placement | `tower.rs`（Tower/TowerTurret/TowerPreview + PlacedTowers/TowerAtlas）、悬停预览、点击放置、`td.spawn_tower` | `p8-assert.log` | **35/35 PASS** | 0 | `p8-shot.png`（65787 B，1600×900） |
| **P9** targeting-and-damage | `Health(100)`、`AttackRange(192)`/`Damage(34)`/`AttackTimer(0.5s)`、`attack_enemies`（快照索敌 + 旋转 + 扣血 + 就地 despawn + 枪口焰）、`despawn_timed` | `p9-assert.log` | **30/30 PASS** | 0 | `p9-shot.png`（70655 B） |
| **P10** waves-and-spawning | TOML `[enemy_types.*]`/`[[waves]]`、`SpawnEvent`/`SpawnSchedule`/`build_spawn_schedule`、`spawn_wave_enemies`、`td.wave`；`spawn_test_enemy` 退役 | `p10-assert.log` | **35/35 PASS** | 0 | `p10-shot.png`（68719 B） |
| **P11** win-lose（defeat） | `BaseLives(5)`、`process_base_reachers`、`GameOver` 标记、`check_game_state`（`Local<bool>`）、`td.game` | `p11-defeat-assert.log` | **41/41 PASS** | 0 | `p11-defeat-shot.png`（70395 B） |
| **P11** win-lose（victory） | 同上，六塔覆盖 → 10 敌全灭 | `p11-victory-assert.log` | **37/37 PASS** | 0 | `p11-victory-shot.png`（78167 B） |
| **P12** multi-level-architecture | `GameState` 状态机（LevelSelect/InGame/GameOver）、`OnEnter/OnExit` 装配与清场、`GameEntity` 批量回收、`level_select.rs`/`game_over.rs`、`GameFinished` 资源、`td.state` | `p12-assert.log` | **83/83 PASS** | 0 | `p12-shot-menu.png`（31110 B）· `p12-shot.png`（69127 B）· `p12-shot-over.png`（54397 B） |

**零 FAIL**：六次断言运行累计 261/261 PASS；上表外无其它 `*-assert` 档（run1/run2 失败轮全部保留，见 §八）。

## 二、证据档结构（每部 check / build / run / assert / shot）

```
docs/evidence/t048-m6-wave2/
├── README.md                     # 本档
├── assert-td.mjs                 # 断言脚本（自 t047 档复制后追加 p8–p12；t047 原件未动）
├── chains.log                    # 指纹/截图 md5/方法数/SUMMARY 的机器汇总（grep + md5sum 生成）
├── gate-check-workspace.log      # 终档门禁 1
├── gate-test-workspace.log       # 终档门禁 2
├── process-check.log             # 收尾进程核查（grep_exit=1 = 零残留）
├── p8-check.log  p8-build.log  p8-run.log  p8-assert.log  p8-shot.png
├── p9-check.log  p9-build.log  p9-run.log  p9-assert.log  p9-shot.png
├── p10-check.log p10-check2.log p10-build.log p10-run.log p10-assert.log p10-shot.png
├── p11-check.log p11-build.log
│   ├── p11-defeat-run.log   p11-defeat-assert.log   p11-defeat-shot.png
│   └── p11-victory-run.log  p11-victory-assert.log  p11-victory-shot.png
├── p12-check.log p12-check2.log p12-build.log p12-build2.log
│   ├── p12-run.log  p12-assert.log  p12-shot-menu.png  p12-shot.png  p12-shot-over.png
└── *-run1.* / *-run2.*            # 失败轮与首轮环境档（如实留档，见 §八）
```

断言脚本用法（树根执行；游戏进程 CWD = `<tree>/game-td`）：

```
node docs/evidence/t048-m6-wave2/assert-td.mjs --phase p8  --shot ../docs/evidence/t048-m6-wave2/p8-shot.png
node docs/evidence/t048-m6-wave2/assert-td.mjs --phase p9  --shot ../docs/evidence/t048-m6-wave2/p9-shot.png
node docs/evidence/t048-m6-wave2/assert-td.mjs --phase p10 --shot ../docs/evidence/t048-m6-wave2/p10-shot.png
node docs/evidence/t048-m6-wave2/assert-td.mjs --phase p11 --mode defeat  --shot ../docs/evidence/t048-m6-wave2/p11-defeat-shot.png
node docs/evidence/t048-m6-wave2/assert-td.mjs --phase p11 --mode victory --shot ../docs/evidence/t048-m6-wave2/p11-victory-shot.png
node docs/evidence/t048-m6-wave2/assert-td.mjs --phase p12 --shot ../docs/evidence/t048-m6-wave2/p12-shot.png \
     --shot-menu ../docs/evidence/t048-m6-wave2/p12-shot-menu.png --shot-over ../docs/evidence/t048-m6-wave2/p12-shot-over.png
```

游戏启动（每部唯一差异 = 日志名；本卡**不经 `cargo run`**，遵守「游戏存活期间不起 cargo」）：

```
$ cd <tree>/game-td && BEVY_ASSET_ROOT=<tree>/game-td <main-repo>/target/debug/game-td.exe > <log> 2>&1 &
$ for i in $(seq 1 200); do curl -s -m 1 --fail -o /dev/null -X POST http://127.0.0.1:15702 \
      -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"rpc.discover","id":1}' && break; sleep 0.05; done
$ node docs/evidence/t048-m6-wave2/assert-td.mjs --phase <p> ...
$ taskkill //F //IM game-td.exe
```

**`BEVY_ASSET_ROOT` 是必需环境量**（本卡新发现的环境事实）：直接启动 exe 时 `CARGO_MANIFEST_DIR`
不存在，bevy 资产根回退到 exe 所在目录（`target/debug/assets`）→ 图集加载失败、地图全空
（P8 run1 实证：`p8-run-run1.log` 的 `ERROR bevy_asset::server: Path not found …target\debug\assets\Tilesheet/...`，
截图 `p8-shot-run1.png` 为纯底色）。关卡 TOML 走 `std::fs` 相对 CWD（`game-td/`），不受影响——
两条路径不同根，是本卡踩到的第一个环境坑。

## 三、指纹链（两层）

### 3.1 地图层 GRID_DIGEST（P8 → P12 零漂移）

算式 = 断言脚本 `gridDigest()`：逐格 `(x,y) -> (TileType, atlas 索引)` 排序后 sha256（与 T047 档同式）。
取数：

```
P8           70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07
P9           70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07
P10          70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07
P11 defeat   70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07
P11 victory  70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07
P12 InGame   70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07
P12 重开     70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07
（T047 基线 P4/P5/P6/P7 同值）
```

七档逐位同值 = 塔放置 / 战斗 / 波次 / 胜负 / 状态机重构均未触碰地图面（瓦片逻辑类型与视觉索引逐格不变）。
断言脚本把该值冻结为 `GRID_DIGEST_BASELINE` 常量并逐部强断言（不是仅记录）。

### 3.2 塔/敌计数指纹（P11 ≡ P12）

口径（派工单「新增塔实体/敌人实体计数 digest 独立链」）：`sha256(排序行)`，行集 =
每座塔基座世界坐标 `T:x,y` + 炮塔计数 `TT:n` + 在场敌人血量 `E:hp`（血量多重集，不含位置与时间）。
采样窗 = 波次时钟 `elapsed ∈ [2.2, 3.0)`（此时恰已刷 3 只 soldier、零击杀、零漏怪）；场景 = 1 座
**无害塔** `[2,0]`（与道路中心行 5 的垂直距离 320px > 射程 192px——永不攻击，不改变波次/漏怪节奏）。

```
P11 defeat  ENTITY_DIGEST_A=1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23
P12 InGame  ENTITY_DIGEST_A=1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23
独立复算（脚本 expectedEntityDigest([2,0], spawned=3)）同值
```

两侧对拍均为「实测 == 脚本独立复算」，非复用执行侧数字。

## 四、截图链（md5，原始输出见 `chains.log`）

| 文件 | md5 | bytes | 尺寸 | 判定 |
| --- | --- | --- | --- | --- |
| `p8-shot.png` | `7f0b0da3c6bf028ae2e3b1b1dcb968a1` | 65787 | 1600×900 | 地图 + 塔（[7,3]）；无敌人（脚本起跑稍迟，单敌已到终点——P7 已证） |
| `p9-shot.png` | `1e6c1fe3b8ebc4e5d16f07f1264354d3` | 70655 | 1600×900 | 地图 + 2 塔 + 在路敌人 + 预览幽灵 |
| `p10-shot.png` | `f4cc3ed9cc8683963852971152e698cc` | 68719 | 1600×900 | 波 1 五敌同屏（soldier/runner 混编） |
| `p11-defeat-shot.png` | `828b6bb70feef0c98d1e339a2beda4ab` | 70395 | 1600×900 | 多敌在路 + 无害塔 |
| `p11-victory-shot.png` | `40ed275d913d90053804b5464b996d15` | 78167 | 1600×900 | 六塔 + 交战（左三塔已转向） |
| `p12-shot-menu.png` | `63f58234e5fc9ffb09ec99e947d4ee1c` | 31110 | 1600×900 | LevelSelect 屏（"Press [1] - Level 1"） |
| `p12-shot.png` | `60d9b2e97d84a6f3a8de988bcc0bcf4c` | 69127 | 1600×900 | InGame 屏（同 P11 defeat 场景） |
| `p12-shot-over.png` | `9197300475abe1346fa0683fc054a5f5` | 54397 | 1600×900 | GameOver 屏（Defeat 文案 + 返回提示） |
| 失败/首轮件 | `p8-shot-run1`(23296, 纯底色) · `p9-shot-run1`(23292, 纯黑) · `p9-shot-run2`(75528) · `p12-shot-menu-run1`(31167, em dash 豆腐块) · `p12-shot-over-run1`(54152) | — | — | 全部保留（§八） |

## 五、`td.*` 方法面与 discover 数（逐部 +1 如实记录）

| 部 | 新增方法 | discover 总数 | 语义 |
| --- | --- | --- | --- |
| 基线（T047） | `td.screenshot` | 24 | 两段式窗口帧捕获 |
| P8 | `td.spawn_tower` | **25** | `{"tile":[x,y]}` → `{"placed", "tile", "pos"}`；与点击系统复用同一校验/生成核心（`is_placeable` / `spawn_tower_entities`）；非法参数 -32602 |
| P9 | —（无新增） | 25 | 伤害/死亡经 `world.query`(`game_td::Enemy`+`game_td::Health`) 读 |
| P10 | `td.wave` | **26** | `{"available","elapsed","pending","spawned","total"}`，只读 |
| P11 | `td.game` | **27** | `{…,"lives","game_over",…}`，只读；资源缺失字段为 null |
| P12 | `td.state` | **28** | `get`（`state`/`result`/`finished`）与 `set`（`{"op":"set","state":…}`，写 `NextState`） |

全部 `td.` 前缀、无 `game.` 前缀（脚本逐部断言）；白名单/`tooling/mcp-bridge` 未动（非本卡面）。
各 phase 的 discover 期望在脚本内按 phase 预注册（`EXPECT_DISCOVER`），可 `--expect-discover` 覆盖
（终版构建全相位复跑口径 = 28）。

## 六、终档门禁（四行）

```
1) cargo check --workspace -j 3                      → REAL_EXIT=0，0 错 0 警        （gate-check-workspace.log）
2) cargo test --workspace -j 3 -- --test-threads=3   → REAL_EXIT=0；game 20/20、mcp-bridge 15/15、
                                                       doctest 78 passed / 0 failed / 17 ignored（= T047 基线 78/0/17 恢复）
                                                                                     （gate-test-workspace.log）
3) BRP discover 总数 + 各 phase 断言档               → 24→25→25→26→27→28；p8 35/35、p9 30/30、p10 35/35、
                                                       p11 41/41 + 37/37、p12 83/83（六 runs 261/261）
4) 游戏进程收尾零残留                                → tasklist | grep -iE 'game-td|rustc' = 零命中（grep_exit=1）
                                                                                     （process-check.log）
```

本卡未新增 doctest（无 Markdown 改动进 `docs/` 门禁面；实测 doctest 数与 T047 基线一致即证）。
`cargo test` 全程 `-j 3` 且 `--test-threads=3` 透传；所有 cargo 命令经 `CARGO_TARGET_DIR=<main-repo>/target`
共享 target，游戏存活期间未起任何 cargo（每部流程 = 编译 → 启 exe → 断言 → taskkill）。

## 七、依赖 / 席位 / 文件清单

- **新增依赖：无**。`Cargo.toml`（根与 game-td）与 `Cargo.lock` 与主仓基线**内容逐字节一致**
  （`diff --strip-trailing-cr` 无差异；导出树为 CRLF、主仓工作树为 LF，仅行尾不同）。
  P8–P12 全部用既有面实现（`serde`/`serde_json`/`toml`/`bevy`/`bevy_ecs_tilemap` 均为 P7 前已在锁内）。
- **系统席位（`add_systems` 计数）**：现值 **11 处调用 / 21 个系统**（P7 基线 = 2 处 / 5 个系统；
  增量 **+9 处 / +16 个系统**）。分布：Startup 1、OnEnter(LevelSelect) 2、OnExit(LevelSelect) 1、
  Update(LevelSelect) 1、OnEnter(InGame) 4、OnExit(InGame) 1、FixedUpdate(InGame) 5、
  Update(InGame) 3、OnEnter(GameOver) 1、OnExit(GameOver) 1、Update(GameOver) 1。
  M6 选品「系统席位 ≤10」按能力席表（`docs/m6-game-selection.md` §四）计：本卡触及 #3 敌人系统、
  #4 塔系统、#8 `td.*` 方法面、#9 断言面，**未新开席位，未触顶**。
- **文件清单（树内相对路径）**：
  - 改：`game-td/src/main.rs`、`game-td/src/brp.rs`、`game-td/src/enemy.rs`、`game-td/src/level.rs`、
    `game-td/assets/levels/level_01.toml`
  - 增：`game-td/src/tower.rs`（P8）、`game-td/src/level_select.rs`（P12）、`game-td/src/game_over.rs`（P12）
  - 未动：`game-td/src/map.rs`、`game-td/src/tiling.rs`、`Cargo.toml`/`Cargo.lock`、`tooling/**`、
    `assets-methodology/**`、`bevy-dev/**`、主仓任何路径
  - 证据：`docs/evidence/t048-m6-wave2/**`（本目录；t047 档案未动）

## 八、跑档披露（原始档被重跑覆盖一律如实留痕）

| 轮 | 现象 | 归因 | 处置 |
| --- | --- | --- | --- |
| P8 run1 | 断言 35/35 但截图纯底色（23296 B） | 直接启 exe 缺 `BEVY_ASSET_ROOT` → 资产根落到 `target/debug`，图集 404（run log 有 `Path not found` 原始行） | 留档 `p8-*-run1.*`；run2 设 `BEVY_ASSET_ROOT` 后 35/35 + 65787 B 截图 |
| P9 run1 | 29/30 PASS + 1 FAIL（炮塔旋转阈值 0.3，实测 0.294）+ 截图纯黑（23292 B） | ① 断言阈值与几何实际不符（该窗内 \|z\| 上界 ~0.41 但采样命中 0.294）；② 起跑 ~0.8s 内捕获 swapchain 为纯黑 | 留档；改为「朝向几何对拍」（\|实测 z − 解析 z\| < 0.05）+ 轮询中途受理截图 |
| P9 run2 | 3 FAIL：`minHealth=Infinity / died=true / lastEnemyX=null` | 脚本在史上加了 1s 等待再截图 → 敌人（4 塔合计 DPS 过高）在首采样前已被打死 | 留档；改 2 塔 + 轮询立即开始 + 第 6 个样本处受理截图 |
| P9 run3 | 30/30 PASS，`aimHits=8`、`minHealth=32`、`lastEnemyX=-37` | — | **终档** `p9-*` |
| P10 check1 | `E0531`（`_In(_params)` 非法模式）+ unused import | 编码笔误 | 留档 `p10-check.log`；`p10-check2.log` 绿 |
| P12 run1 | 81 断言中 2 FAIL：`finished=true` 期望（实际 false）、GameOver 后读 `lives/spawned`（资源已清） + 菜单/结算文案 em dash 渲染成豆腐块 | ① `GameFinished` 按教程 12 Step 4/6 在 `cleanup_level` 移除，进入 GameOver 时不可见——**断言设计错误**；② 默认内置字体无 em dash 字形 | 留档 `p12-*-run1.*`；改：GameFinished 以「已被 cleanup 移除」为负向证据、漏怪进度改在 InGame 内采样（elapsed≥5.0：spawned==5/pending==5/lives∈3..=4）、UI 文案改 ASCII 连字符 → run2 83/83 |
| P12 run2 | 83/83 PASS | — | **终档** `p12-*`（对应 check2/build2 的源码态） |

P11 两模式各一次运行即绿（defeat 41/41 @ elapsed 6.80 判负；victory 37/37 @ elapsed 19.64 判胜），无覆盖。

**可复跑口径（重要）**：每部 `p8–p11` 断言档对应**该部构建**（各 `pN-check.log`/`pN-build.log` 即其源码态；
P8/P9/P10/P11 的 check/build 均在对应部代码上跑）。`p12-*` 对应终版构建（`p12-check2.log`/`p12-build2.log`）。
后部对前部是**语义替换**而非纯增量：P10 起 `spawn_test_enemy` 退役（单测试敌不存在）、P12 起 `GameOver`
标记换成 `GameFinished`+状态机——故 p8–p11 phase **不能对终版构建原样复跑**，复跑须回到当部构建态；
这是「逐部过门禁再进下一部」的执行形态，不是缺档。

## 九、漂移表对照注记（教程 0.18 → bevy 0.20；底稿 `docs/evidence/t044-td-spike/w2/api-drift.md`）

逐部按表核过、**未发现表外新漂移**；实现处注释逐条带 0.20 本地源码锚。摩擦级 ≥1 的项：

- **Part 08**（全 0 级）：`Single`、`Visibility::{Visible,Hidden}`、`Window::cursor_position`、
  `Camera::viewport_to_world_2d`（Result 形态）、`ButtonInput<MouseButton>::just_pressed`。
- **Part 09**：`Parent` → `ChildOf`（摩擦 1，**未直接命名**该类型，只经 `with_children`，表内已注）；
  其余 `Timer`/`TimerMode`/`Without` 消歧/`Query::get_mut` 全 0。
- **Part 10**：`Time::delta_secs()` 在 `FixedUpdate` = 固定步长（0 级，锚 `bevy_time-0.20.0/src/fixed.rs:76/243-261`；
  教程「60Hz」为近似表述，1/64s 行为等价）；`VecDeque` 为 std，无表项。
- **Part 11**：`Local<T>`、`Option<Res<T>>`、零尺寸 Resource 均 0 级。
- **Part 12**：`States`/`init_state`/`OnEnter`/`OnExit`/`NextState::set`/`in_state`（0 级）；
  `queue_silenced(despawn())` 0 级，0.20 实锚 = `system/commands/mod.rs:1987`（`EntityCommands::queue_silenced`）
  + `system/commands/entity_command.rs:278`（`despawn()` 命令）+ `system/mod.rs:143`（glob 再导出）；
  **`TextFont.font_size` 摩擦 1**——0.20 为 `FontSize` 枚举（`bevy_text-0.20.0/src/text.rs:688/783`），
  `From<f32>` = `FontSize::Px`（`:861-865`），实现用 `40.0.into()`；`Node`/`Text`/`TextColor`/`Val` 0 级。
- **§6.17 type_path 冻结（P12 专项）**：BRP 面组件（`TileType`/`MapTile`/`PathTile`/`Enemy`/`Health`/
  `Tower`/`TowerTurret`/`TowerPreview`）全部保留 `#[type_path = "game_td"]` + `#[reflect(Component)]`，
  P12 新增模块（`level_select`/`game_over`）不承载 BRP 面组件；断言侧组件名全程 `game_td::*`
  未变、`world.query` 全部命中（P12 前后同脚本同口径）。本卡**未移动**既有 BRP 面组件所在模块
  （= 无需新冻结动作；见 U7）。
- **§6.14 Interaction/Button**：P8/P11 未用 UI 按钮（放置走鼠标世界坐标 + BRP 通道），P12 两个屏
  只用 `Node`/`Text`/`TextFont`/`TextColor`/`Val`，**不触碰已弃用的 `Interaction`/`Button`**。

## 十、行为规格映射（教程步骤 → 本卡实现）

- P8：Step 1 组件/资源（`Tower`/`TowerTurret`/`TowerPreview`、`PlacedTowers`、`TowerAtlas`）；
  Step 2 图集预载；Step 3 `world_to_tile`（round 非 floor）；Step 4 `hovered_placeable_tile`；
  Step 5 常驻幽灵 + `Visibility` 开关；Step 6 悬停更新；Step 7 点击放置（基座 + 炮塔双实体）；
  Step 8 接线（`Update` 输入 / `FixedUpdate` 玩法）。
- P9：Step 1 `Health(100)`；Step 2 `AttackRange/Damage/AttackTimer`（192/34/0.5s，`TimerMode::Repeating`）；
  Step 3 `MuzzleFlash`/`DespawnTimer`（295 / 0.15s）；Step 4 快照索敌 + 旋转（−π/2）+ 冷却扣血 +
  就地 despawn + 旗口焰子实体；Step 5 `despawn_timed`；Step 6 生成端接线；Step 7 `FixedUpdate` 链定序。
- P10：Step 1 TOML 敌型/波次（4 敌型 + 2 波 = 10 事件，0–4s / 12.0–16.8s）；Step 2 反序列化结构
  （`#[serde(default)]` 向后兼容、`#[serde(rename="type")]`）；Step 3 `SpawnEvent`/`SpawnSchedule`；
  Step 4 扁平化 + `total_cmp` 排序（跨波交错）；Step 5 `while let` 弹队列；Step 6 接线（刷怪在移动前）。
- P11：Step 1 `BaseLives(5)`；Step 2 `process_base_reachers`（扣命 + despawn，替代旧回收）；
  Step 3 `GameOver` 标记；Step 4 `check_game_state`（败 = lives≤0；胜 = 队列空 ∧ 无敌人；`Local<bool>` 单次）；
  Step 5 `Option<Res<GameOver>>` 早退；Step 6 链尾判定。
- P12：Step 1 `GameState` 三态；Step 2 `GameEntity`/`ScreenUi`；Step 3 全量打标（瓦片/敌人/塔/预览/
  枪口焰父子）；Step 4 `GameFinished` 资源替代 `Local<bool>` + `NextState`；Step 5 `GameResult`；
  Step 6 `cleanup_level`（`queue_silenced`）+ `cleanup_screen_ui`；Step 7 装载移入 `OnEnter(InGame)`、
  相机 Startup；Step 8/9 两屏 UI + 键控；Step 10 全量 run_if 门控 + OnEnter/OnExit 调度。

## 十一、上报项（U 编号，待主会话裁决）

- **U1 · `td.state` 的写能力**：P12 需确定性状态迁移，BRP 无键盘注入面，故 `td.state` 带
  `{"op":"set"}`（写 `NextState`）。选项：(a) 接受现状（本卡实现，discover 28，断言链可复跑）；
  (b) 若要求 `td.*` 只读，则需另辟 driver（直写 World 或系统化注入输入），代价 = P12 断言链不可经
  BRP 复跑、需新增 tooling（本卡禁改 tooling/）。**未决时不阻塞本卡交付**。
- **U2 · 教程外记账字段**：`SpawnSchedule.spawned: u32` 为 `td.wave`/断言对拍而加（教程无此字段），
  是否接受为实现注记。
- **U3 · 计数指纹口径定义权**：`T:x,y` + `TT:n` + 敌人血量多重集、采样窗 `elapsed∈[2.2,3.0)`、
  场景 = 1 座无害塔 `[2,0]`。此口径为执行侧设计，若要进 m6-all 判定面，请确认口径。
- **U4 · P12 未覆盖 victory→GameOver 转移**（只测 defeat 转移；victory 分支逻辑已由 P11 证明）。
  覆盖需再加 ~20s 运行 + 6 塔场景；是否补跑请裁。
- **U5 · UI 文案 ASCII 化**：教程文案含 em dash（默认内置字体无字形 → 豆腐块），已改 ASCII 连字符。
- **U6 · P12 屏无文本断言**：`Node`/`Text` 未在 BRP 反射面注册（bevy_ui 未 `register_type`），
  UI 屏以截图为准；若需文本断言须扩反射面（新增注册面）。
- **U7 · P12 未做「模块移动」**：教程 Part 12 本就只加 `level_select`/`game_over` 模块；BRP 面组件
  未换模块，§6.17 冻结以既有注解维持（断言组件名全绿即证）。若要求 Part 13 式重组（`state.rs`/
  `gameplay.rs`），请另立卡。
- **U8 · `[2,0]` 无害塔**：P11/P12 指纹场景输入（非教程内容，永不攻击，不改变胜负节奏）。
- **U9 · P9/P12 紧时序依赖**：敌人 4.67s 生命周期，断言须在 BRP 就绪后立即执行（本卡以 bash 内联
  poll→assert 完成）。若后续卡要求「起跑即刻断言」的通用口径，建议派工单层面固化。

## 十二、未决点（O 编号）

- **O1（沿用派工单 O1 采样窗约束）· 落实方式**：波次断言未改教程默认 spawn 节奏（0/1.0s，12.0/1.2s），而是用
  「事件表独立复算 + elapsed 驱动三检查点」在窗内采样（A: t≥2.3、B: t≥4.2、C: t≥12.3，各窗 ≥0.6s）。
  若 m6-all 需要固定时刻断言，需改 TOML 或加分批，属新裁决面。
- **O2 · `td.game.game_over` 在 P12 恒 false**：`GameFinished` 随 `cleanup_level` 移除（教程语义），
  进入 GameOver 态即不可见；m6-all 若要断言「本局已判定」应查 `td.state.result`。
- **O3 · 历史 phase 复跑口径**：脚本对 p4–p7 保留 t047 语义（discover 期望 24）；终版构建全相位
  复跑需 `--expect-discover 28`（脚本头已注）。
- **O4 · `GameFinished` 负向证据**：P12 以「进入 GameOver 时 `finished==false`」证明 `cleanup_level`
  已执行；该断言依赖教程语义（每回合重建门闩）。若后续引入 state-scoped 资源（P13+），需复核。
- **O5 · 断言脚本驱动未工具化**：launch→poll→assert→kill 目前是 bash 内联（每部一条命令留痕）；
  若要进 m6-all 终局判定面，建议固化为仓内 driver（本卡禁改 tooling/，未做）。

## 十三、环境事实（供后续波次复用）

1. **资产根**：直接启 exe 必须设 `BEVY_ASSET_ROOT=<tree>/game-td`；否则 bevy 资产根回退到 exe 目录，
   图集 404（P8 run1 原始档在案）。关卡 TOML 仍按 CWD 相对路径读取。
2. **截图首帧盲区**：进程起跑 ~0.8s 内的 `td.screenshot` 捕获读到纯黑 swapchain（P9 run1 实证，
   23292 B）；建议起跑 ≥1.2s 再受理捕获，或像 P9 那样在轮询中段受理。
3. **敌人生命窗**：P10 起敌人由波次刷出（波 1 事件 0–4s；单敌 4.67s 到终点），波 2 自 12.0s；
   任何针对单敌的交战断言窗口 = 起跑后 ~1–4s。
4. **显示器/窗口**：与 T047 同机同参（1280×720 逻辑 × DPI 1.25 = 1600×900 捕获；Vulkan/RTX 3050）。
5. **本机日志编码**：cargo/bevy 日志带 ANSI 颜色序列（原样留档）。

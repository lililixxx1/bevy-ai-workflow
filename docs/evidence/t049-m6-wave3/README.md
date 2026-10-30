# T049-W1 · M6 塔防波次 3（教程 Part 13–17）· 证据总账

- 任务：T049-W1（M6 塔防波次 3，节点卡；派工单 = 教程 `13-multiple-levels` … `17-tower-selection-ui` 的行为规格）
- 树：`<tree>` = `trees/m6/w3`（基线 = 主仓 `a218526` 的 git archive 纯净导出；无 `.git`，物理隔离）
- 日期：2026-10-30（晚）
- 结论：**五部（P13–P17）全部 PASS**——终轮五份断言共 **439 条断言全绿**（87+75+77+96+104），
  两道门禁绿（`cargo check --workspace` 0 错 0 警；`cargo test --workspace` REAL_EXIT=0，
  doctest 78/0/17 = 基线恢复），**分关卡指纹注册表**（level_01/02/03）自 P13 登记后
  P14→P17 全程强断言零漂移，塔/敌计数指纹 `ENTITY_DIGEST` 与 T048 档逐位一致
  （`1d40c7b7…890b23`），P16 数据驱动重构后 P13–P15 关键断言面全绿回归，游戏进程收尾零残留。
- 口径：证据档自含（命令 + 原始输出 + `REAL_EXIT=`/`EXIT=` 同档，附录 G）。
  档内绝对路径为 cargo/game 原始输出，未做改写；README 统一记 `<tree>` / `<main-repo>`。

---

## 一、判定行汇总（逐部一行结论 + 断言计数）

| 部 | 交付 | 断言日志 | SUMMARY | EXIT | 截图 |
| --- | --- | --- | --- | --- | --- |
| **P13** multiple-levels | `level_02/03.toml`、`AvailableLevels`/`SelectedLevel`、`scan_available_levels`、动态菜单（列表 + `1..9` 键）、`td.level`；`setup_tower_atlas` 移 Startup | `p13-assert.log` | **87/87 PASS** | 0 | `p13-shot.png`（85992 B，1600×900）· `p13-shot-menu.png`（42131 B） |
| **P14** economy | `Gold(300)`、被动收入 3/s、`Bounty(25)` 链路（TOML→组件）、建造价 100、余额门、`PlacementDenied` 红闪、HUD；`td.economy` | `p14-assert.log` | **75/75 PASS** | 0 | `p14-shot.png`（83982 B） |
| **P15** rocket-launcher | 火箭发射器（三槽弹药 / 2s 补弹 / 追踪弹 / 60px 溅射 / 爆炸特效）、`TOWER_COST`→150；`td.combat` | `p15-assert.log` | **77/77 PASS** | 0 | `p15-shot.png`（76216 B） |
| **P16** data-driven-towers | `assets/towers.toml` + `TowerRegistry`（raw/runtime 两段 + projectile 键解析）、组件合并（`TowerAttacker`/`InstantShooter`/`AmmoState`/`TowerTypeId`）、`TOWER_COST` 删除；**P13–P15 回归段**；`td.towers` | `p16-assert.log` | **96/96 PASS** | 0 | `p16-shot.png`（76111 B） |
| **P17** tower-selection-ui | dock（槽位 / 预览 / 名称 / 造价 / 金边）、滚轮 / 数字键 / 槽位点击（`Interaction` + `#[expect(deprecated)]`）、注册表向量化、`big_rocket` 纯 TOML 第三塔、预览随选切换；`td.tower_select` | `p17-assert.log`（= run4）· `p17-assert-final.log`（终版构建复跑） | **104/104 PASS** ×2 | 0 | `p17-shot.png`（85866 B）· `p17-shot-dock.png`（85651 B，dock 三槽位） |

**零 FAIL（终轮）**：439/439 PASS。失败/失败轮全部保留（见 §八）。

## 二、证据档结构（每部 check / build / run / assert / shot）

```
docs/evidence/t049-m6-wave3/
├── README.md                     # 本档（不含 doctest 面：docs/src/lib.rs 未 include 本目录）
├── assert-td.mjs                 # 断言脚本（自 t048 档复制后追加 p13–p17；t048 原件未动）
├── toml-level-expand.mjs         # 独立 TOML 展开器（PathTile/Grass 计数 + 波次事件数，双盲对拍）
├── chains.log                    # 指纹/截图 md5/方法数/SUMMARY 的机器汇总（grep + md5sum）
├── baseline-check.log            # 基线（未改源码）check
├── gate-check-workspace.log      # 终档门禁 1（终版源码）
├── gate-test-workspace.log       # 终档门禁 2（终版源码）
├── gate-build-game-td.log        # 终版二进制构建（供 p17 终版复跑）
├── process-check.log             # 收尾进程核查（grep_exit=1 = 零残留）
├── p13-check.log  p13-build.log  p13-run-run1.log  p13-assert-run1.log  p13-assert.log  p13-shot*.png
├── p14-check.log  p14-check2.log p14-build.log  p14-run.log  p14-assert.log  p14-shot.png
├── p15-check.log  p15-build.log  p15-run.log  p15-assert.log  p15-shot.png
├── p16-check.log  p16-build.log  p16-run.log  p16-assert-run1.log  p16-assert.log  p16-shot.png
└── p17-check.log  p17-check2.log p17-build.log p17-run.log p17-run-final.log
    p17-assert-run1.log（脚本 ReferenceError）p17-assert-run2.log（98/99）
    p17-assert-run3.log（102/103）p17-assert.log（run4 104/104）p17-assert-final.log（终版构建 104/104）
    p17-shot.png  p17-shot-dock.png
```

断言脚本用法（树根执行；游戏进程 CWD = `<tree>/game-td`）：

```
node docs/evidence/t049-m6-wave3/assert-td.mjs --phase p13 --shot ../docs/evidence/t049-m6-wave3/p13-shot.png \
     --shot-menu ../docs/evidence/t049-m6-wave3/p13-shot-menu.png
node docs/evidence/t049-m6-wave3/assert-td.mjs --phase p14 --shot ../docs/evidence/t049-m6-wave3/p14-shot.png
node docs/evidence/t049-m6-wave3/assert-td.mjs --phase p15 --shot ../docs/evidence/t049-m6-wave3/p15-shot.png
node docs/evidence/t049-m6-wave3/assert-td.mjs --phase p16 --shot ../docs/evidence/t049-m6-wave3/p16-shot.png
node docs/evidence/t049-m6-wave3/assert-td.mjs --phase p17 --shot ../docs/evidence/t049-m6-wave3/p17-shot.png \
     --shot-dock ../docs/evidence/t049-m6-wave3/p17-shot-dock.png
```

游戏启动（每部唯一差异 = 日志名；全卡不经 `cargo run`，遵守「游戏存活期间不起 cargo」）：

```
$ cd <tree>/game-td && BEVY_ASSET_ROOT=<tree>/game-td <main-repo>/target/debug/game-td.exe > <log> 2>&1 &
$ for i in $(seq 1 200); do curl -s -m 1 --fail -o /dev/null -X POST http://127.0.0.1:15702 \
      -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"rpc.discover","id":1}' && break; sleep 0.05; done
$ node docs/evidence/t049-m6-wave3/assert-td.mjs --phase <p> ...
$ taskkill //F //IM game-td.exe
```

`BEVY_ASSET_ROOT` 沿用 §6.18（本卡未复踩；P13 起 `towers.toml`/关卡 TOML 同走
`std::fs` 相对 CWD，资产根只影响图集）。

## 三、指纹链（三层）

### 3.1 分关卡 GRID_DIGEST 注册表（P13 登记，P14→P17 全程在链）

算式 = 断言脚本 `gridDigest()`：逐格 `(x,y) -> (TileType, atlas 索引)` 排序后 sha256
（与 T047/T048 档同式；`all` 查询面未请求 `TileType` 时该字段为 undefined，与 T047 基线同口径
——摘要实际区分度来自 `(x,y)+atlas 索引`，逐格 TileType 另有独立对拍，见 §3.2）。

| 关卡 | digest | 首次登记 | 在链断言（PASS） |
| --- | --- | --- | --- |
| level_01 | `70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07` | T047 冻结基线 | P13 / P14 / P15 / P16[A] / P17-clean |
| level_02 | `3edf8dffc55b743933c8c4b8484f50855b8e7d193495d5f4c2536f96ea8a4f6d` | P13 run1（`--register` 实测登记） | P13 / P14 / P15 / P16 / P17 |
| level_03 | `840298ef16970017e1916a573a5333759bea8782a959509d365c12fc9dcd76f7` | 同上 | P13 / P14 / P15 / P16 / P17 |

`level_01` 零漂移**贯穿 P13 → P17**（含 P13 跨关卡切换后回切复验、P17 重进一局），
五部全部逐位同值 = 多关卡装载 / 经济 / 弹道 / 数据驱动重构 / UI 均未触碰地图面。

### 3.2 逐格几何独立对拍（level_02/03）

`toml-level-expand.mjs` 按教程 06 Step 2/4 独立重写宽 3 展开 + 拐点补齐，从 TOML 真解析：

```
level_01 EXPECTED_PATH=45  EXPECTED_GRASS=105 TOTAL=150 EVENTS=10
level_02 EXPECTED_PATH=60  EXPECTED_GRASS=90  TOTAL=150 EVENTS=12
level_03 EXPECTED_PATH=72  EXPECTED_GRASS=78  TOTAL=150 EVENTS=15
```

断言逐项对拍：`PathTile` 计数、`TileType` 直方图、**逐格 `(x,y)` 的 TileType 与独立网格一致**、
`TilePos` 覆盖 15×10 恰一次、`td.game.total` == 独立事件数。

### 3.3 塔/敌实体计数指纹（P16 重构回归）

口径沿用 T048：`sha256(排序行)`，行集 = 塔基座世界坐标 `T:x,y` + 塔顶计数 `TT:n` + 在场敌人
血量多重集 `E:hp`；场景 = 1 座无害塔 `[2,0]`（dy=320 > 射程 192）+ 波次时钟 ∈ [2.2,3.0)。

```
P11/P12（T048 档）  1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23
P16  [A]            1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23   ← 逐位一致
```

即：数据驱动重构（组件合并 / 注册表化 / 价格来源替换）后**塔实体指纹链延续**，未改口径。

### 3.4 经济 / 战斗采样值（P14–P17，O1 采样窗保留）

```
P14 passive  dt=1.031s delta=3.094   （3.0/s ±20% 窗内）
P14 kill     dt=0.141s dGold=25.42 → net=25.00（k=1）
P15 combat   maxProjectiles=1 minAmmo=0 maxFired=5 refill=true killBeforeLeak=true
             exactDamageHealths=50,10（100−50 / 60−50 = 精确 50 伤害）
P16 [B]      projectile=true minAmmo=0 maxFired=5 refill=true exactDamageHealths=50,10
P17 big      fired=1 capacity=1 kill=true（1 槽 / 4s 补弹来自 TOML）
```

## 四、截图链（md5，原始输出见 `chains.log`）

| 文件 | md5 | bytes | 判定 |
| --- | --- | --- | --- |
| `p13-shot-menu.png` | `8ed891cd161b3b435be59fe241657097` | 42131 | 菜单动态列表 `[1] level_01 / [2] level_02 / [3] level_03` |
| `p13-shot.png` | `7bcd671c5f49820c1ec677b11ae7787b` | 85992 | level_02 在局（L 形道路） |
| `p14-shot.png` | `9bef9b3a16d436d3471f0a9eba411bc0` | 83982 | HUD「Gold: N」+ 三塔交战 |
| `p15-shot.png` | `e55cbf2c1044df2dff14c2c8299f15a9` | 76216 | 火箭发射器 + 弹体飞行帧 |
| `p16-shot.png` | `f07dc76764eb6a39119e8a7a38376b23` | 76111 | 数据驱动构建在局 |
| `p17-shot.png` | `39ff6741294257258db0a02e38f95655` | 85866 | 选 big_rocket 放置后（dock 在场） |
| `p17-shot-dock.png` | `c65a70a6cd5e58211c7d6efec4af9a0c` | 85651 | dock 三槽位（名称/造价/金边）近景 |

## 五、`td.*` 方法面与 discover 数（逐部 +1 如实记录）

| 部 | 新增方法 | discover 总数 | 语义 |
| --- | --- | --- | --- |
| 基线（T048） | `td.screenshot` … `td.state`（6 个自定义） | 28 | — |
| P13 | `td.level` | **29** | `list`（发现列表 + 选中项）/ `select`（index 或 path）；非法 -32602 |
| P14 | `td.economy` | **30** | `{"available","gold","starting_gold","passive_rate","tower_cost"}`，只读 |
| P15 | `td.combat` | **31** | 发射器 / 弹体 / 爆点计数 + 弹药聚合 + `fired` + 火箭数值（P16 起读注册表） |
| P16 | `td.towers` | **32** | 注册表只读快照（P16 键序稳定输出；P17 向量序 + `index`） |
| P17 | `td.tower_select` | **33** | `get` / `set`（index 或 key）——U6 侧信道；非法 -32602 |

全部 `td.` 前缀、无 `game.` 前缀（脚本逐部断言）；`tooling/mcp-bridge` 未动。

## 六、终档门禁（四行）

```
1) cargo check --workspace -j 3                      → REAL_EXIT=0，0 错 0 警        （gate-check-workspace.log）
2) cargo test --workspace -j 3 -- --test-threads=3   → REAL_EXIT=0；game 20/20、mcp-bridge 15/15、
                                                       doctest 78 passed / 0 failed / 17 ignored（= 基线）
                                                                                     （gate-test-workspace.log）
3) BRP discover 总数 + 各 phase 断言档               → 28→29→30→31→32→33；p13 87/87、p14 75/75、
                                                       p15 77/77、p16 96/96、p17 104/104（终轮 439/439）
                                                                                     （p1[3-7]-assert*.log）
4) 游戏进程收尾零残留                                → tasklist | grep -iE 'game-td|rustc|cargo' = 零命中
                                                       （grep_exit=1）                （process-check.log）
```

本卡未新增 doctest（`docs/src/lib.rs` 未 include 本证据目录；实测 78/0/17 即证）。
所有 cargo 命令 `-j 3`、测试 `--test-threads=3` 透传、`CARGO_TARGET_DIR=<main-repo>/target`；
游戏存活期间未起任何 cargo（每部流程 = 编译 → 起 exe → 断言 → taskkill）。
cargo 段全局互斥：本卡全程单条 cargo，无并发。

> **取证补录说明**：执行时退出码取自控制台（本工具对每条命令回报 `REAL_EXIT`/`EXIT`），
> 部分 cargo/run 日志正文只含原始 stdout/stderr。为满足「退出码同档」要求，已在对应档尾
> **显式补录**一行 `# [取证补录 …] REAL_EXIT=…`（失败轮如实补 101；p14-check 与 p17-check
> 为真实编译失败轮）。game 进程档补录 `GAME_EXIT=1 = taskkill 终止`，非自身退出。

## 七、依赖 / 席位 / 文件清单

- **新增依赖：无**。`Cargo.toml`（根与 game-td）与 `Cargo.lock` 未改；P13–P17 全部用既有面
  （`serde`/`serde_json`/`toml`/`bevy`/`bevy_ecs_tilemap` 均在 P13 前已在锁内）。
- **系统席位（`add_systems` 计数）**：现值 **12 处调用 / 37 个系统**（T048 = 11 处 / 21 个系统；
  增量 **+1 处 / +16 个系统**）。分布：Startup 3、OnEnter(LevelSelect) 2、OnExit(LevelSelect) 1、
  Update(LevelSelect) 1、OnEnter(InGame) 6、OnExit(InGame) 1、FixedUpdate(InGame) 10、
  Update(InGame) 9（输入组 3 + 视觉组 6）、OnEnter(GameOver) 1、OnExit(GameOver) 1、Update(GameOver) 1。
  M6 选品「系统席位 ≤10」按能力席表（`docs/m6-game-selection.md` §四）计：本卡触及 **#2 地图/关卡、
  #3 敌人（赏金链路）、#4 塔、#5 弹道与伤害、#6 经济与波次、#7 UI/HUD（P17 激活）、#8 td.* 方法面、
  #9 断言面**；**未新开席位，未触顶**。
- **文件清单（树内相对路径）**：
  - 改：`game-td/src/main.rs`、`game-td/src/brp.rs`、`game-td/src/level.rs`、`game-td/src/enemy.rs`、
    `game-td/src/level_select.rs`（P13 重写为动态列表）、`game-td/src/tower.rs`（P15/16/17 大改）、
    `game-td/assets/levels/level_01.toml`（四敌型各加 `bounty = 25`；CRLF 行尾保持）
  - 增：`game-td/src/economy.rs`（P14）、`game-td/assets/levels/level_02.toml`、`level_03.toml`（P13）、
    `game-td/assets/towers.toml`（P16，P17 追加 `big_rocket`）、本证据目录 `docs/evidence/t049-m6-wave3/**`
  - 未动：`game-td/src/map.rs`、`game-td/src/tiling.rs`、`game-td/src/game_over.rs`、
    `Cargo.toml`/`Cargo.lock`、`tooling/**`、`assets-methodology/**`、`bevy-dev/**`、主仓任何路径

## 八、跑档披露（原始档被重跑覆盖一律如实留痕）

| 轮 | 现象 | 归因 | 处置 |
| --- | --- | --- | --- |
| P13 run1 | 81/85（4 FAIL：`per-cell TileType` 取不到 + 发现列表比较口径） | ① `MapTile` 查询未请求 `TileType` 列；② 脚本把关卡「读取路径（game-td/ 前缀）」误当「发现路径」比较 | 留档 `p13-assert-run1.log`；改为 TileType+POS 联查逐格对拍、`discovered` 字段分离 → run2 87/87 |
| P13 登记轮 | `--register` 只记录 digest 不断言 | 分关卡注册表按派工单「digest 常量进断言脚本」两段式登记 | 注册后第 2 轮起全程强断言 |
| P14 | 首轮 75/75 即绿（passive 3.094/1.031s，kill net 25.00） | — | 终档 |
| P15 | 首轮 77/77 即绿（fired 5 / minAmmo 0 / refill / 伤害 50,10） | — | 终档 |
| P16 check1 | 无（一次过） | — | — |
| P16 run1 | 95/96（1 FAIL：`attack_cooldown==0.3` 浮点） | f32 `0.3` 落 JSON = `0.30000001192092896` | 留档 `p16-assert-run1.log`；改 `<1e-6` 容差 → run2 96/96 |
| P17 run1 | 脚本崩溃 `ReferenceError: rapid is not defined` | 断言脚本变量名笔误 | 留档 `p17-assert-run1.log`；修脚本 |
| P17 run2 | 98/99（rapid 34 伤害未观测） | 同局先放 big_rocket（150 一击必杀）吞掉 rapid 两段伤害窗口 | 留档；把 rapid 移到独立新局验证（非行为差异） |
| P17 run3 | 102/103（big_rocket 拒绝失败） | 新局余额 300−100=200 恰好买得起 200 塔 | 留档；先再放一座 rapid（−100）压低余额再验拒绝 |
| P17 run4 | 104/104 PASS | — | **终档** `p17-assert.log` |
| P17 final | 104/104 PASS（终版构建，换 doc 注释后重编复跑） | — | **终档** `p17-assert-final.log` |

**可复跑口径（重要）**：p13/p14/p15 断言档对应**该部构建**（各 `pN-check/build.log` 即源码态）；
第 1 局/第 2 局的经济价与选择序在 P16/P17 后已变（价格改读注册表、注册表向量化且装载序
= HashMap 迭代序，教程 17 明确「arbitrary but stable within session」），故 p14/p15 phase
**不能对终版构建原样复跑**；p16 phase 内已含 P13–P15 关键断言回归段（地图三关指纹 / 经济
常量与门 / 塔指纹 / 弹道链），p17 终版复跑 104/104。这是「逐部过门禁再进下一部」的执行形态，
不是缺档。

## 九、漂移表对照注记（教程 0.18 → bevy 0.20；底稿 `docs/evidence/t044-td-spike/w2/api-drift.md`）

逐部按表核过；**表内无新漂移**，**表外新事实 1 项（见 U1）**。实现处注释逐条带 0.20 本地源码锚：

- **Part 13**（表内两项全 0 级）：`init_resource`（`bevy_app-0.20.0/src/app.rs:485`）、
  `KeyCode::Digit1..9`（`bevy_input-0.20.0/src/keyboard.rs:300` 起）。
- **Part 14**（四项全 0 级）：`Option<&T>` 查询（`bevy_ecs-0.20.0/src/query/fetch.rs:3318`）、
  `Text` 整组件覆写（`bevy_ui-0.20.0/src/widget/text.rs:128`/`:132`）、
  `PositionType::Absolute`（`ui_node.rs:508`/`:1763`）、`Node top/left`（`ui_node.rs:528`/`:542`）。
- **Part 15**（三项全 0 级）：`GlobalTransform::translation`（`bevy_transform-0.20.0/src/components/global_transform.rs:210`）、
  组件内存 `Entity`（`bevy_ecs-0.20.0/src/lib.rs:76-77`）、`Query::get`（`system/query.rs:1722`）。
- **Part 16（派工单指定「漂移实证点」——逐条核过）**：表内行 = 「（无新 API，继承前部）」。
  实核结论 **成立**：本重构只用到 `serde::Deserialize`（P13 前已在依赖）、`std::fs::read_to_string`
  与 `toml::from_str`（`level.rs` 既有同款用法，`toml` 依赖 P6 起在锁内）——**未引入任何 bevy 新 API**，
  故无表外漂移；组件合并 / 注册表化均为纯 Rust 结构改动。
- **Part 17**（表内七项）：`Node` 0、`ImageNode::from_atlas_image(image, TextureAtlas)`
  （`bevy_ui-0.20.0/src/widget/image.rs:99`）0、`BorderColor` 四字段 / `all`
  （`ui_node.rs:2603`/`:2627`）0、`MessageReader<MouseWheel>` 0
  （`message_reader.rs:34`；`mouse.rs:214` 字段 `:218`/`:220`）、`Res::is_changed` 0
  （`change_detection/params.rs:603`）；**`Interaction` 摩擦 1**——0.20 为 `DeprecatedInteraction`
  弃用别名（`focus.rs:50` 本体 / `:65` 别名；SKILL §6.14），按 **A' 裁决通道**以
  `#[expect(deprecated)]` 落两处（`setup_tower_dock` 的 `Interaction::None`、
  `handle_dock_slot_click` 的查询与 `Pressed` 比对），每处注教程原步 + 0.20 现况，未换 API 重写；
  `#[expect]` 未被「未满足期望」告警触发（check 0 警即证两处确实踩到弃用面）。
- **§6.17 type_path 冻结**：新增 BRP 面组件 `GoldHud`/`Bounty`（P14）、`TowerTypeId`/`TowerDockSlot`（P17）
  全部 `#[type_path = "game_td"]` + `#[reflect(Component)]` + 注册；既有面（`TileType`/`MapTile`/
  `PathTile`/`Enemy`/`Health`/`Tower`/`TowerTurret`/`TowerPreview`）路径未漂移。
- **§6.14 Interaction/Button**：全程只 P17 用到 `Interaction`（弃用通道，见上），未用 `Button`。
- **U6（Node/Text 不在 BRP 反射面）**：P13 菜单文案、P14 HUD 文案、P17 dock 视觉态均由截图留证；
  可查询侧信道 = `GoldHud` 计数、`TowerDockSlot` 计数、`td.tower_select` 选中下标、
  `TowerTypeId` 放置塔型 —— 全部实际断言命中。

## 十、与教程实现形态的偏离（行为零变化，显式登记）

1. **保留两个零尺寸验证面标记**（教程 16 删除它们）：`Tower`（基座计数）与 `TowerTurret`
   （**塔顶实体**通用标记，P15 起火箭发射器炮管也挂）。理由 = 派工单要求「塔实体指纹链延续」
   + BRP 断言面按 `game_td::Tower` / `game_td::TowerTurret` 计数；玩法判别仍严格按教程
   （`InstantShooter` / `AmmoState`）。**未改指纹口径**（见 §3.3 逐位一致）。→ 见 U2。
2. **模块拆分不照搬**：教程 13 把 `state.rs`/`gameplay.rs` 拆出；本侧保留既有模块边界
   （`main.rs` 薄接线 + 领域模块），仅新增 `economy.rs`。教程为行为规格非实现来源（附录 C）。
3. **`TowerDefinition.key` 显式保留**：教程 17 向量化后不存键串；本侧保留供 `td.towers`
   快照与按键选择定位（断言面需要稳定标识）。
4. **预览换肤按 z 判别**（教程 17 用「收集 + 断言恰 2 个 + 按 z 排序」）；本侧直接
   `z < 2.05 → base`，等价且不引入排序 panic 面。
5. **dock 水平居中**：教程未给完整 Node 参数；本侧 = 全宽 `Node` + `justify_content: Center`
   + 绝对定位 `bottom`（视觉等价，截图留证）。
6. **`SelectedTowerType` 键→下标**：P16 教程硬编码 `"rocket"`；本侧 P16 同样硬编码键，
   P17 起点改下标 0（教程 17 向量化），装载序来自 HashMap 迭代（教程明确无序契约，
   断言按 key 反查 index）。

## 十一、上报 / 未决（U/O）

- **U1（表外新事实，需裁决是否入档）**：`MouseWheel` **不在** `bevy::prelude`（0.20 实测：
  `use bevy::prelude::*` 下 `MessageReader<MouseWheel>` 报 E0425，`p17-check.log` 原文在档）；
  需显式 `use bevy::input::mouse::MouseWheel`。API 本身按漂移表 Part 17 为 0 级（同形直用），
  仅导入路径属表外事实。**选项**：(a) 接受本侧显式导入，补 SKILL §6.13「prelude 缺口」一条
  （附 `bevy_input-0.20.0/src/lib.rs` 实况锚）；(b) 仅记漂移表 Part 17 注记不动 SKILL。代价均极低。
- **U2（实现形态 vs 指纹口径）**：教程 16 删除 `Tower`/`TowerTurret` 标记，本侧为「指纹链延续」
  保留为验证面标记（§十.1）。**选项**：(a) 接受（推荐——行为零变化、指纹逐位一致）；
  (b) 若要求严格照教程删除，则须改指纹口径（塔基座坐标集合 + 攻击组件计数）并重登记基线，
  代价 = 跨 P11–P17 指纹链断开 + T048 档对照失效。
- **U3（验证面局限）**：溅射「多目标」无自然场景——三关敌人沿路间距（96–256px）均 > 60/100px
  半径，故只断言「单目标也走溅射逐敌结算且伤害精确 = 50/150」+ 半径/伤害数据对表。
  若要硬证多目标：需新增聚怪关卡或 `td.*` 刷怪通道（超本卡范围）。**选项**：(a) 接受局限（推荐）；
  (b) 下一波次加聚怪场景。
- **U4（输入注入面）**：滚轮 / 数字键 / 槽位点击 / 悬停三态着色无 BRP 注入通道，未直接驱动；
  已按 U6 预置口径以「选中下标资源 + 放置塔型 + TowerTypeId」链 + 截图覆盖。
  **选项**：(a) 接受（U6 既定）；(b) 后续加 `td.input` 注入通道（会让 discover 再 +1）。
- **O1**：分关卡 digest 由 P13 run1 实测登记（`--register`），非独立复算；独立复算覆盖
  计数 / 逐格 TileType / 波次事件数（§3.2）。登记轮与断言轮均留档。
- **O2**：`level_01.toml` 原为 CRLF（archive 导出态），插入 `bounty` 行后已统一回 CRLF
  （58/58 行带 CR）；level_02/03 为 LF。内容语义仅新增 `bounty = 25`。
- **O3**：P16/P17 断言脚本按 key 反查注册表 index，**不假设** HashMap 装载序；
  `td.towers` P16 输出按键排序、P17 输出向量序 + `index` 字段（响应契约显式注记）。

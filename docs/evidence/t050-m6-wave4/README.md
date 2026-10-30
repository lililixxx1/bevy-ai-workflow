# T050-W1 · M6 塔防波次 4（教程 Part 18–21）· 证据总账

- 任务：T050-W1（M6 塔防波次 4，节点卡；派工单 = 教程 `18-custom-events` … `21-plugins-and-pause` 的行为规格）
- 树：`<tree>` = `trees/m6/w4`（基线 = 主仓 `41ec1a2` 的 git archive 纯净导出；无 `.git`，物理隔离）
- 日期：2026-10-30（夜）–2026-10-31（凌晨）
- 结论：**四部（P18–P21）全部 PASS**——终轮签收断言 **364 条全绿**
  （p18 86/86 · p19 86/86 · p20 72/72 · p21 120/120，均 REAL_EXIT=0），
  终版构建跨部回归复跑同样全绿；三道门禁绿（`cargo check --workspace` 0 错 0 警 /
  workspace test REAL_EXIT=0〔game 20/20、mcp-bridge 15/15、doctest 78/0/17 = 基线〕/
  claim-lint 13/13）；**三关 GRID_DIGEST 与 ENTITY_DIGEST 零漂移**（`1d40c7b7…890b23`
  与 T048 冻结基线逐位一致）；游戏进程收尾零残留。
- 口径：证据档自含（命令 + 原始输出 + `REAL_EXIT=` 同档，附录 G）。
  档内绝对路径为 cargo/game 原始输出，未做改写；README 统一记 `<tree>` / `<main-repo>`。

---

## 一、判定行汇总（逐部一行结论 + 断言计数）

| 部 | 交付 | 断言档 | SUMMARY | EXIT | 截图 |
| --- | --- | --- | --- | --- | --- |
| **P18** custom-events | `PlaceTower` 消息（`#[derive(Message)]`）+ 生产者 `place_tower_on_click`（只校验）+ 两消费者 `spawn_tower_from_event` / `deduct_gold_on_placement`，`.after()` 定序同帧闭环；`PlacementLog` 记账；`td.events`（快照 + `emit_place` 确定性发布） | `p18-assert.log`（P18 构建）· `p18-assert-final.log`（终版复跑） | **86/86 PASS** ×2 | 0 | `p18-shot.png`（82701 B）· `p18-shot-final.png`（96460 B） |
| **P19** gizmos | `PlacedTowers` → `HashMap<[u32;2], Entity>`（生成函数返回塔顶实体、占格记账移入生成消费者）；`draw_tower_ranges`（每帧 `circle_2d`：预览金圈 0.4 / 悬停白圈 0.5）；`RangeGizmoLog` 绘制参数记录；`td.gizmos`（snapshot + probe） | `p19-assert.log`（终版复跑） | **86/86 PASS** | 0 | `p19-shot.png`（83256 B） |
| **P20** audio | `audio.rs`：`AudioAssets` / `BackgroundMusic` / `PlaySound` 消息 / 四系统（装载 / 起播 / 停播 / 消费）；激光与爆炸音效挂 `MessageWriter`，火箭推进器循环音挂**弹体实体**（`AudioPlayer + PlaybackSettings::LOOP`）；`AudioLog` 计数；4 个合成 ogg 资产 + CREDITS 注记；`td.audio` | `p20-assert.log`（P20 构建）· `p20-assert-final.log`（终版复跑） | **72/72 PASS** ×2 | 0 | `p20-shot.png`（87394 B） |
| **P21** plugins-and-pause | `AudioPlugin`（六处注册收拢，纯重构）；`pause.rs`：`PauseState`（Running/Paused 正交状态机）+ `PausePlugin`（Escape 翻转 / 覆盖层生成回收）；三组玩法系统 `.and_eager(in_state(PauseState::Running))`；`td.pause`（get/set/toggle + 守卫）；**收官全量回归**（注册表 / 三关 digest / ENTITY_DIGEST / 经济 / 弹道 / 选择 UI / 事件 / gizmo / 音频 / 暂停冻结-恢复） | `p21-assert.log` | **120/120 PASS** | 0 | `p21-shot-pause.png`（86622 B，PAUSED 覆盖层） |

**零 FAIL（终轮）**：364/364 PASS（86+86+72+120）。失败/失败轮全部保留（见 §八）。

## 二、证据档结构

```
docs/evidence/t050-m6-wave4/
├── README.md                  # 本档（不含 doctest 面：docs/src/lib.rs 未 include 本目录）
├── assert-td.mjs              # 断言脚本（自 t049 档复制后追加 p18–p21；t049 原件未动）
├── toml-level-expand.mjs      # 独立 TOML 展开器（自 t049 档原样复制，未改）
├── chains.log                 # 机器汇总（SUMMARY / discover / digest / 截图 md5 原始输出）
├── p18-check.log              # P18 构建态 check（REAL_EXIT=0）
├── p18-build.log              # P18 release build（REAL_EXIT=0）
├── p18-run.log                # P18 构建态游戏进程日志（GAME_EXIT=1 = taskkill）
├── p18-assert.log             # P18 构建态断言：86/86
├── p18-shot.png               # P18 截图（消息放置出的塔在局）
├── p18-run-final.log / p18-assert-final.log / p18-shot-final.png   # 终版构建复跑：86/86
├── p19-check.log              # P19 构建态 check（REAL_EXIT=0）
├── p19-build.log              # P19 release build（REAL_EXIT=0）
├── p19-run.log                # P19 构建态跑档
├── p19-assert-run1.log        # P19 构建态断言（旧脚本）：85/85
├── p19-run-final.log          # P20 构建上复跑（跨构建）
├── p19-assert-run2.log        # 失败轮：84/88（脚本假设注册表装载序 = bug，见 §八）
├── p19-run3.log / p19-assert.log / p19-shot.png   # 终版构建终轮：86/86
├── p20-check.log（失败轮 101）· p20-check2.log（REAL_EXIT=0）
├── p20-build.log / p20-run.log / p20-assert.log / p20-shot.png     # P20 构建态：72/72
├── p20-run-final.log / p20-assert-final.log                        # 终版构建复跑：72/72
├── p21-check.log（失败轮 101）· p21-check2.log（REAL_EXIT=0）
├── p21-build.log              # 终版 release build（REAL_EXIT=0）
├── p21-run.log / p21-assert-run1.log   # 失败轮：脚本语法错（await in arrow）
├── p21-run2.log / p21-assert.log / p21-shot-pause.png   # 终轮：120/120
├── gate-check-workspace.log   # 门禁 1：check --workspace（REAL_EXIT=0，0 错 0 警）
├── gate-test-workspace.log    # 门禁 2：test --workspace --test-threads=3（REAL_EXIT=0）
├── gate-doc-test-t050.log     # 门禁 3：test --doc -p docs（78/0/17，REAL_EXIT=0）
├── gate-claim-lint.log（失败轮：旧证据档 mtime 1）· gate-claim-lint2.log（13/13 PASS）
├── scope-check.log            # 写范围核对（树 vs 主仓 HEAD 41ec1a2；--strip-trailing-cr）
└── process-check.log          # 收尾进程核查（grep_exit=1 = 零残留）
```

断言脚本用法（树根执行；游戏进程 CWD = `<tree>/game-td`）：

```
node docs/evidence/t050-m6-wave4/assert-td.mjs --phase p18 --shot ../docs/evidence/t050-m6-wave4/p18-shot.png
node docs/evidence/t050-m6-wave4/assert-td.mjs --phase p19 --expect-discover 37 --shot ../docs/evidence/t050-m6-wave4/p19-shot.png
node docs/evidence/t050-m6-wave4/assert-td.mjs --phase p20 --expect-discover 37 --shot ../docs/evidence/t050-m6-wave4/p20-shot.png
node docs/evidence/t050-m6-wave4/assert-td.mjs --phase p21 --shot-pause ../docs/evidence/t050-m6-wave4/p21-shot-pause.png
```

游戏启动（release 构建；每部唯一差异 = 日志名；全卡不经 `cargo run`，遵守「游戏存活期间不起 cargo」）：

```
$ cd <tree>/game-td && BEVY_ASSET_ROOT=<tree>/game-td <main-repo>/target/release/game-td.exe > <log> 2>&1 &
$ for i in $(seq 1 300); do curl -s -m 1 --fail -o /dev/null -X POST http://127.0.0.1:15702 \
      -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"rpc.discover","id":1}' && break; sleep 0.05; done
$ node docs/evidence/t050-m6-wave4/assert-td.mjs --phase <p> ...
$ taskkill //F //IM game-td.exe
```

`BEVY_ASSET_ROOT` 沿用 SKILL §6.18；P20 起音频资产同样走资产根（`assets/audio/*.ogg`）。

## 三、指纹链（与往期档逐位对拍）

### 3.1 三关 GRID_DIGEST（P21 终版构建一次过）

算式 = 断言脚本 `gridDigest()`（逐格 `(x,y) -> (TileType, atlas 索引)` 排序后 sha256；与 T047/T048/T049 同式）。

| 关卡 | digest | 在链断言（PASS） |
| --- | --- | --- |
| level_01 | `70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07` | T047 冻结基线；p18 / p19 / p20 / **p21** 全部同值 |
| level_02 | `3edf8dffc55b743933c8c4b8484f50855b8e7d193495d5f4c2536f96ea8a4f6d` | T049 P13 登记；p18–p21 同值 |
| level_03 | `840298ef16970017e1916a573a5333759bea8782a959509d365c12fc9dcd76f7` | T049 P13 登记；p18–p21 同值 |

### 3.2 ENTITY_DIGEST（P11/P12/T048 冻结链）

口径沿 T048/T049：`sha256(排序行)` = 塔基座世界坐标 `T:x,y` + 塔顶计数 `TT:n` + 在场敌人血量多重集 `E:hp`；
场景 = 1 座无害塔 `[2,0]`（dy=320 > 射程 192）+ 波次时钟 ∈ [2.2,3.0)。

```
T048 冻结基线        1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23
P21（终版构建）      1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23   ← 逐位一致
```

即：自定义消息重构 / `PlacedTowers` HashMap 化 / gizmo / 音频 / 插件化 / 暂停门控**全未触碰**
塔/敌实体面；elapsed=2.234375（窗口 [2.2,3.0) 内）。

### 3.3 行为面采样（P18–P21）

```
P18 events   injected=1 spawned=1 gold_applied=1 produced=0；双消费者载荷逐字段一致
             （tile [2,0] / tower_id 0 / cost 100 / world_pos [-320,-288]）
P18 世界坐标 emit.world_pos=[-320,-288] == 独立算式（OFFSET_X+2*64, OFFSET_Y+0）
P19 gizmo    big_rocket [7,7] → center [0,160] radius 256（TowerAttacker）；rapid [5,3] → center
             [-128,-96] radius 192；空/路径/越界格 → ring null。预览圈分支两态均有覆盖：
             run2（p19-assert-run2.log）真实光标在地图内 → `preview_visible=true`，
             两枚同心金圈 (1,0.84,0,0.4) radius 256（当时选中 big_rocket）全部 PASS；
             终轮（p19-assert.log）光标不在窗口内 → `preview_visible=false`、rings=[]，
             走「不可见」分支（INFO + probe 补位）——两态断言均成立。
P20 audio    music Loop/0.3（music_sink=true：本机有音频设备）；projectile_audio≥1；
             rocket_explosion 触发；laser_fire 触发；last_volume 0.8/1.0；离场 music 回收
P21 pause    暂停 500ms：位置/血量/金币/elapsed 逐位冻结（gold 203.984375 → 同值；
             elapsed 1.328125 → 同值）；覆盖层恰 1；恢复后 elapsed 1.328125→1.96875、敌人继续移动；
             菜单外 set/toggle → -32602
P21 combat   projectile=true minAmmo=0 maxFired=5 refill=true killBeforeLeak=true
             exactDamageHealths=50,10（与 T049 P15/P16 同值同形）
```

## 四、截图链（md5，原始输出见 `chains.log`）

| 文件 | md5 | bytes | 判定 |
| --- | --- | --- | --- |
| `p18-shot.png` | `9ba6c7ebdd56d92cf4c51e4fa8db2e90` | 82701 | P18 构建：消息链放置的塔在局 |
| `p18-shot-final.png` | `173766e2285ee67f2cdb58bae4e9c26c` | 96460 | 终版构建复跑 |
| `p19-shot.png` | `2ed65a7cecd562c379502132552051fa` | 83256 | 两塔在局（[7,7] big_rocket + [5,3] rapid） |
| `p20-shot.png` | `783a84ed5d96adad258c8d64e388f6f6` | 87394 | 火箭交战帧（弹体/爆炸） |
| `p21-shot-pause.png` | `b43c8c9a177339e7ee64837e30f2b2e4` | 86622 | **PAUSED 覆盖层**（半透明黑 + 居中大字） |

截图 md5 逐轮不同属预期（敌人位置/交战帧不定）；本卡以断言计数与 digest 为判定面，
截图为视觉留证（P19 射程圈在真实光标下的绘制由 snapshot 参数断言覆盖，见 §3.3）。

## 五、`td.*` 方法面与 discover 数（逐部 +1 如实记录）

| 部 | 新增方法 | discover 总数 | 语义 |
| --- | --- | --- | --- |
| 基线（T049） | `td.screenshot` … `td.tower_select`（10 个自定义） | 33 | — |
| P18 | `td.events` | **34** | `snapshot`（produced/injected/spawned/gold_applied + 双消费者载荷快照 + placed_tiles/gold）/ `emit_place`（与生产者同一校验写 `PlaceTower` 消息）；非法 → -32602，不可放置 → `queued:false` |
| P19 | `td.gizmos` | **35** | `snapshot`（上一帧 `draw_tower_ranges` 绘制参数）/ `probe`（单格悬停圈参数，复用同一纯函数 `hover_ring_params`）；非法 → -32602 |
| P20 | `td.audio` | **36** | 音乐实体与 PlaybackSettings（Loop/0.3）/ AudioSink 存在性 / sfx 实体与计数 / projectile_audio / last_volume |
| P21 | `td.pause` | **37** | `get`（state/overlay/game_state）/ `set`（Running/Paused）/ `toggle`；非 InGame 或 GameFinished 后 → -32602 |

全部 `td.` 前缀、无 `game.` 前缀（脚本逐部断言）；`tooling/mcp-bridge` 未动。

## 六、终档门禁（五行）

```
1) cargo check --workspace -j 3                     → REAL_EXIT=0，0 错 0 警        （gate-check-workspace.log）
2) cargo test --workspace -j 3 -- --test-threads=3  → REAL_EXIT=0；game 20/20、mcp-bridge 15/15、
                                                      doctest 78 passed / 0 failed / 17 ignored（= 基线）
                                                                                    （gate-test-workspace.log）
3) cargo test --doc -p docs -j 3 -- --test-threads=3 → 78/0/17，REAL_EXIT=0        （gate-doc-test-t050.log；
                                                      供 claim-lint 的 gate-doc-test* 回退式挑选用）
4) cargo run -p claim-lint                          → 13/13 PASS，REAL_EXIT=0     （gate-claim-lint2.log）
5) 游戏进程收尾零残留                                → tasklist | grep -iE 'game-td|rustc|cargo' = 零命中
                                                      （grep_exit=1）               （process-check.log）
```

所有 cargo 命令 `-j 3`、测试 `--test-threads=3` 透传、`CARGO_TARGET_DIR=<main-repo>/target`；
游戏存活期间未起任何 cargo（每部流程 = 编译 → 起 exe → 断言 → taskkill）。cargo 段全局互斥。

> **取证补录说明**：执行时退出码取自控制台（本工具对每条命令回报 `REAL_EXIT`/`EXIT`），
> 部分 cargo/run 日志正文只含原始 stdout/stderr。为满足「退出码同档」要求，已在对应档尾
> **显式补录**一行 `# [取证补录 2026-10-30] REAL_EXIT=…`（失败轮如实补 101）。game 进程档
> 补录 `GAME_EXIT=1 = taskkill 终止`，非自身退出。

## 七、依赖 / 席位 / 文件清单

- **新增依赖：无**。根 `Cargo.toml` / `game-td/Cargo.toml` / `Cargo.lock` 未改（零差异）；
  ogg/vorbis 解码走 bevy 默认特性 `audio = ["bevy_audio", "vorbis"]`（`bevy-0.20.0/Cargo.toml:2765-2768`）。
- **新增资产**：`game-td/assets/audio/{laser_fire,rocket_explosion,rocket_launch,background_music}.ogg`
  （本工程 ffmpeg 合成占位音，见 `game-td/CREDITS.md` 新节；无第三方权利）。
- **系统席位（`add_systems`）**：基线 **12 调用点 / 36 系统**（T049 收尾态；T049 README 记
  「12/37」为分项求和笔误——按其自列分布求和 = 36，本卡以实数为准并如实记此差异）→
  终态 **main.rs 12/39 + audio.rs（AudioPlugin）4/4 + pause.rs（PausePlugin）3/3 = 19 调用点 / 46 系统**。
  逐部增量：P18 +2（两消费者，入既有 Update 组）/ P19 +1（draw_tower_ranges）/ P20 +4（音频四系统，
  三处入既有元组、一处入既有 Update 组）/ P21 零净增（音频四系统迁入插件 + 暂停三系统入插件）。
  **main.rs 调用点数全程 12 未变**；总调用点 19 超 14 属插件分文件口径（`Plugin::build` 的
  `add_systems` 计点），已在汇报单列。M6 选品「系统席位 ≤10」按能力席表（`docs/m6-game-selection.md` §四）计：
  本卡触及 #4 塔（消息链重构）、#5 弹道与伤害（音频钩子）、#7 UI/HUD（目标圈 / 暂停覆盖层）、
  #8 td.* 方法面（+4）、#9 断言面；**并激活 #10 预留席（音频）**——该席定义即
  「音频（非门槛，owner 裁量可裁）」，本波按派工单「行为等价照做」落地，**不超过 10 席上限，
  但席位表至此 10/10 全激活**（后续波次无预留席，需 Lead 记录）。
- **文件清单（树内相对路径）**：
  - 改：`game-td/src/main.rs`（P18 接线 / P19 gizmo / P20 音频 / P21 插件化 + 暂停门控）、
    `game-td/src/tower.rs`（`PlaceTower`/`PlacementLog`/生产者与生成消费者/`PlacedTowers` HashMap/
    gizmo/音效钩子）、`game-td/src/economy.rs`（经济消费者）、`game-td/src/brp.rs`（+4 方法）、
    `game-td/CREDITS.md`（音频资产节）
  - 增：`game-td/src/audio.rs`（P20 + P21 `AudioPlugin`）、`game-td/src/pause.rs`（P21）、
    `game-td/assets/audio/*.ogg`（4 个）、本证据目录 `docs/evidence/t050-m6-wave4/**`
  - 未动：`game-td/src/{map,tiling,level,level_select,enemy,game_over}.rs`、
    `game-td/assets/{towers.toml,levels/*}`、`Cargo.toml`/`Cargo.lock`、`tooling/**`、
    `assets-methodology/**`、`bevy-dev/**`、`docs/` 其余文件、主仓任何路径

## 八、跑档披露（失败轮 / 旧轮全留）

| 轮 | 现象 | 归因 | 处置 |
| --- | --- | --- | --- |
| P18 | 首轮 86/86 即绿（消息链、载荷、副作用、校验门） | — | 终档；终版复跑 86/86 |
| P19 run1 | 85/85（P19 构建；旧脚本版） | — | 留档 `p19-assert-run1.log` |
| P19 run2 | 84/88（big_rocket 余额不足） | **断言脚本 bug**：假设注册表下标 0 = rapid；实际装载序 = HashMap 迭代序，**跨进程不同**（本进程 index 0 = rocket → 首次放置扣 150），默认选中塔不同导致 300 预算下 big_rocket（200）不可负担。同轮「预览圈可见分支」4 条断言全 PASS（真实光标在地图内，两枚金圈参数正确） | 留档 `p19-assert-run2.log`；脚本改「按 key 反查 + 先放 big_rocket 再放 rapid」→ 终轮 86/86（§九 K3） |
| P20 check1 | REAL_EXIT=101（2×E0502：`AudioLog` 借用跨查询 + 1 warning unused import `GameState`） | 先拷计数再建查询；P20 阶段 GameState 尚未使用（P21 插件化才需要） | 留档 `p20-check.log`；`p20-check2.log` 绿 |
| P20 run | 首轮 72/72 即绿（含 audio sink=true） | — | 终档；终版复跑 72/72 |
| P21 check1 | REAL_EXIT=101（E0408：or-pattern 绑定不一致） | `PauseOp::Set(target) \| PauseOp::Toggle` 绑定面不齐 | 留档 `p21-check.log`；改无绑定模式 + `#[derive(Copy)]` → `p21-check2.log` 绿 |
| P21 run1 | 脚本崩溃（SyntaxError: Unexpected reserved word） | 断言脚本 bug：`await` 出现在非 async 箭头函数内 | 留档 `p21-assert-run1.log`；提取 `selEco` → 终轮 120/120 |
| claim-lint run1 | 12/13（doctest-gate 取到 `ts-17/gate-doc-test-t040.log` 的 77/0/17） | 树为 git archive 导出，**全文件 mtime 相同**（1793372447），claim-lint 的 mtime 降序回退序与主仓不同 → 取到旧档 | 留档 `gate-claim-lint.log`；补跑 `cargo test --doc -p docs` 产 `gate-doc-test-t050.log`（最新候选，78/0/17）→ `gate-claim-lint2.log` 13/13（§十 U5） |

**可复跑口径**：p18/p20 的 `pN-assert.log` 对应**该部构建**（各自 `pN-build.log` 即源码态）；
终版构建复跑档为 `p18-assert-final.log` / `p20-assert-final.log` / `p19-assert.log`（终版）/
`p21-assert.log`。往期 p4–p17 phase 的源码态与终版不同（p16/p17 经济价与注册表序已变），
沿 T049 M2 口径**不强行原样复跑**，其断言面由 p21 收官段以 digest + 指纹 + 新断言覆盖
（注册表 3 塔 / 三关 digest / ENTITY_DIGEST / 经济常量 / 弹道链 / 选择 UI / dock+HUD）。

## 九、漂移表对照注记（教程 0.18 → bevy 0.20；底稿 `docs/evidence/t044-td-spike/w2/api-drift.md`）

逐部按表核过；**表内结论全部成立**，**表外新事实 3 项（K1–K3，均已在汇报单列）**。
实现处注释逐条带 0.20 本地源码锚：

- **Part 18**（表内五项全 0 级）：`Message` trait（`bevy_ecs-0.20.0/src/message/mod.rs:100`；
  派生宏经 prelude 导出，`use bevy::prelude::*` 下 `#[derive(Message)]` 直接可用）、
  `MessageWriter::write`（`message/message_writer.rs:74`）、`MessageReader::read`
  （`message/message_reader.rs:44`）、`App::add_message`（`bevy_app-0.20.0/src/app.rs:427`）、
  `.after()`（`bevy_ecs-0.20.0/src/schedule/config.rs:404`）。**实核补充**：`World::write_message`
  （`world/mod.rs:3041`）返回 `Option<MessageId>`——未 `add_message` 时返回 `None` 并打
  `log::error`（`world/mod.rs:3059-3065`），本侧 handler 对该 `None` 显式 -32602 而非静默。
- **Part 19**（表内两项全 0 级）：`Gizmos`（`bevy_gizmos-0.20.0/src/gizmos.rs:143`；prelude
  `bevy_internal-0.20.0/src/prelude.rs:99-100`；`GizmoPlugin` 随 DefaultPlugins，
  `bevy_internal-0.20.0/src/default_plugins.rs:87`；`2d` 默认特性含 `bevy_gizmos_render`，
  `bevy-0.20.0/Cargo.toml:2706-2724`）；`circle_2d(isometry: impl Into<Isometry2d>, r, color)`
  （`bevy_gizmos-0.20.0/src/circles.rs:167`）+ `From<Vec2> for Isometry2d`
  （`bevy_math-0.20.0/src/isometry.rs:195`）——教程 `circle_2d(Vec2, …)` 同形直用成立。
- **Part 20**（表内五项全 0 级）：`AudioPlayer`（`bevy_audio-0.20.0/src/audio.rs:251`；
  **`#[require(PlaybackSettings)]` 自动补组件**）、`PlaybackSettings::LOOP`（`:100`）/
  `::DESPAWN`（`:106`）/ `with_volume`（`:130`）、`Volume::Linear`（`volume.rs:36`）、
  `AssetServer::load`（`bevy_asset-0.20.0/src/server/mod.rs:364`）。**实核补充**：`AudioSink`
  存在性即「Bevy 已建播放器」——本机 `music_sink=true`（有音频设备），无设备时该值为 false
  属环境事实（`td.audio` 已结构化暴露，断言不作硬门槛，沿立项「音频非收官门槛」）。
- **Part 21**（表内四项）：`Plugin` trait（`bevy_app-0.20.0/src/plugin.rs:57`/`:59`）0、
  正交 `States`（`bevy_state-0.20.0/src/state/mod.rs:9`/`state/states.rs:64`）0、
  `with_child`（`bevy_ecs-0.20.0/src/hierarchy.rs:355`/`:450`）0；
  **`.and()` 摩擦 1 实核成立**——0.20 **无** `.and()`，等价面 = `and_then`
  （短路由，`bevy_ecs-0.20.0/src/schedule/condition.rs:135`）/ `and_eager`
  （两者都求值，`:191`，文档明示「behaviorally identical to simply calling run_if multiple
  times」）；本侧三组玩法系统用 **`and_eager`**（教程语义「两条件都真」），未走弃用通道。
- **§6.17 type_path 冻结**：本卡未新增 BRP 反射组件（`PauseOverlay` 为 `pub(crate)`、只经
  `td.pause` 的 typed query 计数，不进反射面；音频/消息/日志资源均 handler typed query）；
  既有面（`TileType`/`MapTile`/`PathTile`/`Enemy`/`Health`/`Tower`/`TowerTurret`/`TowerPreview`/
  `GoldHud`/`Bounty`/`TowerTypeId`/`TowerDockSlot`）路径未漂移。
- **K1（表外新事实）**：`Volume` / `PlaybackMode` **不在** `bevy::prelude`（0.20 实测：prelude
  只出 `AudioPlayer/AudioSink/AudioSinkPlayback/AudioSource/Decodable/GlobalVolume/Pitch/
  PlaybackSettings/SpatialAudioSink/SpatialListener`，`bevy_audio-0.20.0/src/lib.rs:36-44`）；
  需 `use bevy::audio::{Volume, PlaybackMode}`（同 U1 `MouseWheel` 形态的第二例）。
- **K2（表外新事实）**：`#[derive(Message)]` 可用于 **enum**（`SoundType`）与 struct；消息
  实体在场性由 `Messages<T>` 资源维护，**跨 2 帧自动清理**——暂停期间写消息而消费者不跑
  （P21 门控）时，消息会在恢复前被清掉。本卡未在暂停中写入放置/音效消息（BRP 通道要求
  InGame 且 pause 测试只读取），但这是暂停 + 消息组合的潜在陷阱，记此备查。
- **K3（表外新事实，影响断言设计）**：`TowerRegistry` 向量装载序 = TOML 键经 `HashMap`
  迭代的序，**跨进程不同**（本卡实测两次启动 index 0 分别为 rapid / rocket；会话内稳定）。
  教程 17 已声明「arbitrary but stable within session」，但跨进程不同对本卡是实证首例——
  断言脚本**必须按 key 反查 index**，不得假设 index 0（P19 run2 失败即此教训）。

## 十、与教程实现形态的偏离（行为零变化，显式登记）

1. **`PlacedTowers` 的 `td.spawn_tower` 直通道仍是同步路径**：教程 19 后点击路径走消息链，
   而 BRP `td.spawn_tower` 为确定性即时放置（先例 P8 起、T048/T049 六批断言依赖其「调用返回
   即实体在场」语义）。两条通道共用 `is_placeable` + `spawn_tower_at`，但**不走消息链**——
   消息链本身由 `td.events emit_place` 独立验证（§3.3）。→ 见 U2。
2. **`PauseState` 放在 `pause.rs`**（教程放 `state.rs`）：本工程无 `state.rs`（`GameState` 在
   `main.rs`，P12 起既定），暂停域自含一个新模块更贴合既有模块边界；教程为行为规格非实现来源。
3. **暂停覆盖层不进 `GameEntity` / `ScreenUi` 面**：纯 `PauseState` 驱动（教程 21 同构）。
   由此 `cleanup_level` 不清它——本侧以 `td.pause` 守卫（仅 InGame 受理 + `GameFinished` 后
   拒绝）保证「暂停中无法离场」，覆盖层不会孤儿化（人工按 Escape 的自然路径同样被
   `toggle_pause` 的 InGame 守卫挡住）。
4. **音频资产为合成占位音**：教程仓未随附音频文件（recon 树只有 Markdown），本侧以 ffmpeg
   合成 4 个 ogg（正弦/噪声音），文件名与行为（Loop/Despawn/弹体循环）对齐教程 Step；
   波形内容不在行为规格面内（`game-td/CREDITS.md` 有许可注记）。
5. **`td.audio` 的 sink 字段为环境注记而非硬门槛**：立项定案音频非 M6 收官门槛；本机有设备
   （sink=true）如实记录，断言面走「计数 + 实体 + PlaybackSettings」结构面。
6. **`RangeGizmoLog` / `PlacementLog` / `AudioLog` 三个验证面记账资源**：教程无对应物，同
   `SpawnSchedule::spawned` / `AmmoState::fired` 既有口径（只服务 `td.*` 快照与断言，
   不给玩法逻辑增加语义）。P19 snapshot 中预览圈出现**两枚同心同参金圈**属教程原样
   （`preview.iter()` 两个幽灵实体各画一圈），非缺陷。

## 十一、上报 / 未决（U/O）

- **U1（文档同步，待裁决）**：`docs/m6-game-selection.md` §五 的 `td.*` 方法面描述仍停在
  「初始方法 = `td.screenshot`；候选按实现需要定」时期（T046 定本）——现方法面已 14 个
  （T044 1 + T047–T050 13）。**选项**：(a) 由 Lead 在本波收获批统一更新 §五 清单
  （推荐，单点维护）；(b) 每卡自更新（本卡未做，避免越出派工单写范围）。
- **U2（`td.spawn_tower` 与消息链双路径）**：见 §十.1。**选项**：(a) 维持双路径
  （推荐——直通道语义被六批断言依赖，消息链另有 `td.events` 全链验证）；
  (b) 让 `td.spawn_tower` 也走消息（须改断言时序并重登记受影响档，代价高）。
- **U3（暂停与音频）**：教程 21 明示音频不随暂停停（Bevy 音频独立于 ECS 调度）；本侧照做，
  未做听觉验证（结构面已覆盖）。**选项**：(a) 接受（立项定案）；(b) 后续窗口评估暂停时降音量。
- **U4（暂停覆盖层无躲藏态截图对照）**：`PAUSED` 文案渲染由截图留证（默认字体有拉丁字形，
  无需 em dash 类规避）；未做 OCR。**选项**：(a) 接受（与 M5/M6 既往 UI 留证口径一致）。
- **U5（claim-lint 在 archive 树上的 mtime 陷阱）**：见 §八末行。主仓（有 .git / 各档真实
  mtime）不受影响；archive 导出树里 gate-doc-test 候选并列时回退序任意，须靠「最新产出的
  门禁日志」压过旧档。本卡以 `gate-doc-test-t050.log` 解决并留失败轮对照。建议 Lead 在
  补位 / 复验清单里记一条（复验含 claim-lint 的树：先跑 `cargo test --doc -p docs` 出档）。
- **O1**：P18 `produced` 计数 = 0（点击生产者未被驱动）；点击路径本身无鼠标注入面，
  其「校验 → 写消息」逻辑与 `td.events emit_place` 共享 `is_placeable` 谓词，但
  `place_tower_on_click` 的鼠标分支（红闪拒绝 / `just_pressed`）未在断言面——沿 U6 输
  入注入面局限（T049 U4 同款），不新增通道。
- **O2**：`p21-shot.png` 未产出（p21 phase 只消费 `--shot-pause`）；在局视觉由 p18/p19/p20
  截图覆盖，暂停态由图 `p21-shot-pause.png` 留证。
- **O3**：P19 `snapshot` 的预览圈依赖真实桌面光标位置（本机光标在窗口内 → 可见）；
  断言对两种情形都成立（可见时强断言半径/颜色/同心，不可见时 INFO + probe 补位）。
- **O4**：T049 README §七 记系统席位「12 调用点 / 37 系统」——按其自列分布求和实为 36，
  本卡以实数 36 为基线并在 §七 留痕（不影响任何判定）。

# T052-W1 · 件二：26 部行为等价对照 · 终局逐部核对表

- 任务：T052-W1（M6 塔防 m6-all 终局判定卡，收官节点卡）
- 树：`<tree>` = `trees/m6/w6`（基线 = `<main-repo>` `d6d7609` 的 git archive 纯净导出；无 `.git`，物理隔离）
- 日期：2026-10-31
- 对照底稿：`docs/evidence/t044-td-spike/w2/api-drift.md`（26 部 **148 行**）
- 对照口径（`docs/m6-game-selection.md` §三，定案① / ②）：**行为等价、实现独立**——教程（`trees/m6-recon/tutorial-src/`，目标版本 Bevy 0.18）只作**行为规格**，不作实现来源；本仓实现按 0.20 本地源码锚独立重写。
- 本档性质：**终局核对**，不改判底稿分类——「漂移表分类」列逐部从 `api-drift.md` **机器统计**（脚本按 `^## Part NN` 与表格行扫描，禁手数），与底稿原文一字不改对齐；本卡只补「落地波次 · 证据档」与「终局结论」。

## 一、分类总账（机器统计，与底稿/宣告值三方对齐）

| 口径 | 值 | 来源 |
| --- | --- | --- |
| 底稿行数 | **148** | `api-drift.md` 全表 `Part 01..26` 表格数据行计数 |
| 同形直用 | **136** | 结论列分类计数 |
| 改名/小形态差 | **7** | 同上（P09 / P12 / P17 / P21 / P23×2 / P26） |
| 重写调用面 | **2** | 同上（P26 `SpatialAudioBundle` / 嵌入资产特性） |
| 无对应需路线变更 | **1** | 同上（P26 `DynamicScene`） |
| 纯重构·无新 API | **2** | 同上（P05 / P16 的「（无新 API，继承前部）」显式行） |
| 校验 | 136+7+2+1+2 = **148** | 恒等式成立 |

**与宣告值对齐**：`docs/m6-game-selection.md` §三 记载「148 行：136 同形/7 形态/2 路线/1 语义」——四类合计 146 行，余 2 行为底稿声明的「纯重构无新 API」显式行；本档在正文中把这 2 行单列，避免 148 与 146 的口径歧义（**不是改判**，是补全记账分母）。

## 二、终局逐部核对表（26 部 × 每部一行）

证据档路径均相对 `docs/evidence/`；「落地波次」= 该部作为行为规格被执行的任务卡（台账 T 编号）。

| 部 | 部名（底稿原文） | 漂移表分类（行数 / 明细，机器统计） | 落地波次 · 证据档 | 终局结论 |
| --- | --- | --- | --- | --- |
| P01 | Getting Started | 10 行 · 同形 10 | T044 → `t044-td-spike/`（verdict ①；game-td 宿主工程） | 行为等价 PASS（0 漂移；工程骨架仍为终版形态） |
| P02 | Drawing a Map | 10 行 · 同形 10 | T044 → `t044-td-spike/`（P1 红方块 → P2 图集网格） | 行为等价 PASS（P2↔P3 像素等价 99.96% 实证） |
| P03 | Tilemap Rendering | 15 行 · 同形 15 | T044 → `t044-td-spike/`（P3 单 draw call tilemap） | 行为等价 PASS（bevy_ecs_tilemap 0.20.0 逐字段同形） |
| P04 | Auto-Tiling | 2 行 · 同形 2 | T047 → `t047-m6-wave1/p4-assert.log`（+ `p4-assert2.log` 复跑） | 行为等价 PASS（逐格索引规格 = P3 基线；GRID_DIGEST 冻结） |
| P05 | Refactoring | 1 行 · **（无新 API，继承前部）** | T047 → `t047-m6-wave1/p5-assert.log` | 纯重构零行为变化 PASS（无新 API 行；行为由 p4 面覆盖） |
| P06 | External Level Data | 2 行 · 同形 2 | T047 → `t047-m6-wave1/p6-assert.log` + `p6b-assert.log`（toml 级增删实验） | 行为等价 PASS（外部关卡 TOML 装载） |
| P07 | Enemies and Path Following | 12 行 · 同形 12 | T047 → `t047-m6-wave1/p7-assert.log`（+ `p7-despawn.log`） | 行为等价 PASS（路径跟随/终点回收；despawn 链） |
| P08 | Tower Placement | 9 行 · 同形 9 | T048 → `t048-m6-wave2/p8-assert.log`（run1 = 空图失败轮，BEVY_ASSET_ROOT 知识发现） | 行为等价 PASS（`td.spawn_tower` 与点击同源校验） |
| P09 | Targeting and Damage | 7 行 · 同形 6 / **改名 1**（`Parent`/`Children`） | T048 → `t048-m6-wave2/p9-assert.log`（run1 断言设计假红 → run2 几何对拍） | 行为等价 PASS（改名项按 0.20 关系组件落地，行为不变） |
| P10 | Waves and Enemy Spawning | 1 行 · 同形 1 | T048 → `t048-m6-wave2/p10-assert.log` | 行为等价 PASS（事件表独立复算 + O1 三检查点） |
| P11 | Win/Lose Conditions | 3 行 · 同形 3 | T048 → `t048-m6-wave2/p11-{defeat,victory}-assert.log` | 行为等价 PASS（defeat 6.80s / victory 19.64s 双模式） |
| P12 | Multi-Level Architecture | 15 行 · 同形 14 / **改名 1**（`TextFont.font_size`） | T048 → `t048-m6-wave2/p12-assert.log`（run1 2 FAIL = em-dash 豆腐块） | 行为等价 PASS（三态机 + cleanup 契约；`td.state` 写通道） |
| P13 | Multiple Levels | 2 行 · 同形 2 | T049 → `t049-m6-wave3/p13-assert.log`（run1 失败轮留档） | 行为等价 PASS（level_02/03 落地；分关卡 GRID_DIGEST 登记） |
| P14 | Economy | 4 行 · 同形 4 | T049 → `t049-m6-wave3/p14-assert.log` | 行为等价 PASS（300 / 3.0s / bounty 25 / 第 4 塔拒） |
| P15 | Rocket Launcher | 3 行 · 同形 3 | T049 → `t049-m6-wave3/p15-assert.log` | 行为等价 PASS（三槽弹药 / 2s 补弹 / 溅射 60px） |
| P16 | Data-Driven Towers | 1 行 · **（无新 API，继承前部）** | T049 → `t049-m6-wave3/p16-assert.log`（+ run1/run2） | 纯数据驱动零新 API PASS（towers.toml 装载；键解析两段） |
| P17 | Tower Selection UI | 7 行 · 同形 6 / **改名 1**（`Interaction`） | T049 → `t049-m6-wave3/p17-assert-final.log`（run1-4 失败轮全留档） | 行为等价 PASS（A' 通道：`#[expect(deprecated)]` + 真实 `Interaction`） |
| P18 | Custom Events | 5 行 · 同形 5 | T050 → `t050-m6-wave4/p18-assert-final.log`（+ `p18-assert.log`） | 行为等价 PASS（生产者 + 双消费者同帧闭环） |
| P19 | Gizmos | 2 行 · 同形 2 | T050 → `t050-m6-wave4/p19-assert*.log`（run1 知识发现 K3 装载序） | 行为等价 PASS（hover/preview 圈；`td.gizmos probe` 纯函数共用） |
| P20 | Audio | 5 行 · 同形 5 | T050 → `t050-m6-wave4/p20-assert-final.log` | 行为等价 PASS（Loop 0.3 / 激光 1.0 / 爆炸 0.8；音频非门槛） |
| P21 | Plugins and Pause | 4 行 · 同形 3 / **改名 1**（`.and()` 组合子） | T050 → `t050-m6-wave4/p21-assert.log`（+ run1） | 行为等价 PASS（条件用 `.and_eager` 等价；暂停冻结逐位） |
| P22 | Custom Run Conditions | 4 行 · 同形 4 | T051 → `t051-m6-wave5/p22-assert-run2.log`（run1 = K4 断言设计假红） | 行为等价 PASS（具名条件 + 三集合；纯重构零行为变化） |
| P23 | Level Select Buttons | 5 行 · **改名 2**（`Button` / `Interaction` 三态）/ 同形 3 | T051 → `t051-m6-wave5/p23-assert.log`（+ 跨部回归） | 行为等价 PASS（A' 通道；`Button` 在 0.20 为弃用别名 → 等价落位） |
| P24 | Input Abstraction | 4 行 · 同形 4 | T051 → `t051-m6-wave5/p24-assert-run2.log`（run1 + cross-p21 失败轮） | 行为等价 PASS（`GameAction` 单点翻译；七消费者重构） |
| P25 | Gamepad | 6 行 · 同形 6 | T051 → `t051-m6-wave5/p25-assert.log`（153/153 含全量回归） | 行为等价 PASS（9 绑定 + 死区/主导轴纯函数；无实机手柄如实注记） |
| P26 | Epilogue | 9 行 · 同形 5 / **重写 2**（`SpatialAudioBundle`、嵌入资产特性）/ **无对应 1**（`DynamicScene`）/ **改名 1**（`Observer`） | **裁弃行**（L1-2 裁决；`t044-td-spike/verdict.md` 执行注记 + `m6-game-selection.md` §三） | **不判 FAIL**（载体记为「25 部执行 + P26 裁弃」）。注记见 §三 |

## 三、P26 裁弃行注记（不算 FAIL，逐项交代）

- **裁决依据**：`t044-td-spike/verdict.md`「执行注记（W2 批 L1 裁决随行）：P26 DynamicScene 概念在 0.20 丢弃（L1-2）」；`m6-game-selection.md` §三「P26 DynamicScene 裁弃（0.20 概念已丢，L1-2 裁决）」与 §五「25 部执行 + P26 裁弃」。
- **为什么整部裁弃而非逐项执行**：P26 是 epilogue（概念收束部，无新玩法），其 9 行中摩擦集中在 4 项（Σ摩擦 8/14 全表最高档 L）；其中 `DynamicScene`（3 级）在 0.20 已无对应概念（底稿锚 `bevy_scene-0.20.0/src/lib.rs:889-927` + `SKILL §6.13`），构成本工程路线外的能力，无法以「行为等价」方式对照。
- **裁弃后代码面上真实存在的 P26 概念**（如实注记，非新增实现，均为既有面的顺带证明）：
  - `Observer`（底稿 1 级）：`game-td/src/brp.rs:348` 的 `.observe(on_captured)`（截图两段式契约）；
  - `Reflect` / `TypeRegistry`（底稿 0 级）：`BrpPlugin::build` 的 `register_type` 注册面（BRP 查询/证据链的基座）；
  - 其余（`SpatialListener` / `SpatialAudioBundle` / `Bundle` 派生 / `嵌入资产特性` / `Window.icon` 缺位断言）：**未实现**，属 P26 裁弃范围；P20 音频按「非空间 `AudioPlayer` + 音量/循环」落地（音频非门槛，定案② 附带）。

## 四、跨波指纹链（终局核对的行为等价硬证据）

| 层 | 冻结值 | 本轮（T052 m6-all）复测 | 判定 |
| --- | --- | --- | --- |
| 三关 GRID_DIGEST | level_01 `70282f80…37a07`（T047）/ level_02 `3edf8dff…`、level_03 `840298ef…`（T049 登记） | `m6-all-run4.log` 三关各自 `GRID_DIGEST == registry` PASS | 逐位零漂移 |
| ENTITY_DIGEST（level_01 无害塔场景） | `1d40c7b7…890b23`（T048 冻结） | `m6-all-run4.log` `ENTITY_DIGEST == T048 frozen baseline` PASS | 逐位零漂移 |
| 每关事件表 | 脚本内独立 TOML 解析（`parseLevelEvents`） | 每关 ≥3 个 elapsed 检查点 `spawned == 期望` PASS | 波次推进等价 |

## 五、残余风险与已知边界（如实记）

1. **P23 屏幕命中测试不在断言面**（`td.level_select click` 写 `Interaction::Pressed` 后 `run_system_once` 运行真实消费者；`bevy_ui` 的指针命中本身无 BRP 注入面）——沿 P17 既有边界，非本卡新增缺口。
2. **P25 无实机手柄**（本机 `gamepads=0`）：行为面 = 映射表 + 纯函数探测 + 无设备零影响，沿 T051 U5 注记。
3. **P20 音频为合成占位音**（教程未随附音频资产）：播放模式/音量/实体数等价，资源内容不等价——音频非收官门槛（定案②）。
4. **P26 裁弃**：见 §三；不构成 FAIL，但「26 部全量执行」的口径应始终表述为「25 部执行 + P26 裁弃」。

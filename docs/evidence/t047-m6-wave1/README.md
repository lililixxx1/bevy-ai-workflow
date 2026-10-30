# T047-W1R · M6 塔防 Part 4–7 补位续做 · 证据总账

- 任务：T047-W1R（续做单；原实例 T047-W1 死于 2026-10-30 10:45 宿主整机冻结硬重启）
- 树：`trees/m6/w1`（基线 commit `d035230` 导出；无 `.git` = 物理隔离）
- 交付：`game-td/` Part 4 收尾（截图断言 20/21 → 21/21）+ Part 5 / 6 / 6b / 7 实现与断言全绿
- 日期：2026-10-30（晚间续做批）
- 结论：**五 phase 断言全绿（21/21、21/21、21/21、12/12、25/25），双门禁绿（check 0 错 0 警 / workspace test REAL_EXIT=0）**

> 证据档自含纪律（附录 G）：每份日志自带命令全文 + 原始输出 + `REAL_EXIT=`；
> 机器绝对路径前缀已按先例统一替换（`<tree>` / `<main-repo>`），命中行号与内容原样留痕。

---

## 一、判定行汇总（逐 phase，明细见各 `*-assert*.log`）

| Phase | 断言日志 | SUMMARY | REAL_EXIT | 说明 |
| --- | --- | --- | --- | --- |
| P4（收尾复跑） | `p4-assert2.log` | **21/21 PASS** | 0 | 上午 20/21 唯一 FAIL = 截图 1×1；晚间复跑恢复 1600×900 → 环境性假红（§四） |
| P5（模块重构） | `p5-assert.log` | **21/21 PASS** | 0 | 计数逐项同 P4；网格指纹 == P4；跨运行截图 md5 逐位一致 |
| P6（默认 TOML 关卡） | `p6-assert.log` | **21/21 PASS** | 0 | 计数 / 指纹 / 截图 md5 全同 P5（外部化零漂移） |
| P6b（真变体 TOML） | `p6b-assert.log` | **12/12 PASS** | 0 | PathTile 实测 57 == `toml-path-count.mjs` 独立计数 57 |
| P7（敌人寻路） | `p7-assert.log` | **25/25 PASS** | 0 | Enemy==1；0.4s 位移 78.00（期望 76.8，+1.56%，容差 ±15%）；dx>0、dy=0；截图与 P6 字节差异 |
| 门禁 · 静态检查 | `gate-check-workspace.log` | 0 错 0 警 | 0 | `cargo check --workspace -j 3` |
| 门禁 · 全量测试 | `gate-test-workspace.log` | game 20/20 + mcp-bridge 15/15 + doctest 78/0/17 | 0 | `cargo test --workspace -j 3 -- --test-threads=3`（基线恢复） |
| P5 编译检查 | `p5-check.log` | Finished | 0 | 重构后 0 警 |
| P6 编译检查 / 构建 | `p6-check.log` / `p6-build.log` | Finished | 0 | `toml = "1"` 入成员依赖（锁内 1.1.8，仅加 game-td→toml 边） |
| P7 编译检查 / 构建 | `p7-check.log` / `p7-build.log` | Finished | 0 | 0 警 |

acceptance ①–⑬ 对应关系（与 `assert-td.mjs` phase 断言一一对应，脚本判定行为准）：

- ①②③④ → `p4-assert2.log`（PNG 魔数/IHDR==log 尺寸、MapTile 150 / PathTile 45 / Grass 105、PathTile⊂MapTile、auto-tiling 生效且索引集合 == P3 规格、逐格索引 == expectedIndex、网格指纹产出、截图 1600×900）；
- ⑤⑥ → `p5-assert.log` + `chains.log`（计数逐项同 P4；GRID_DIGEST 同档、跨运行截图 md5 一致）；
- ⑦ → `p6b-expected.log`（独立计数）+ `p6b-assert.log`（measured == expected）；
- ⑧ → `p6-assert.log`（默认 TOML 计数 + 指纹全同 P5）；
- ⑨⑩⑪ → `p7-assert.log`（Enemy==1；位移/方向；截图字节差异，md5 见 `chains.log`）；
- ⑫ → `gate-test-workspace.log`；⑬ → 各 phase 日志首 4 行（discover 24 = 23 内置 + `td.screenshot`，无 `game.*`）。

---

## 二、网格指纹链（P4 → P5 → P6 → P7 逐字并列，全同）

```
INFO | GRID_DIGEST=70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07   # p4-assert2.log
INFO | GRID_DIGEST=70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07   # p5-assert.log
INFO | GRID_DIGEST=70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07   # p6-assert.log
INFO | GRID_DIGEST=70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07   # p7-assert.log
```

指纹算式（脚本内置，与派工单口径一致）：逐格 `(x,y) -> (TileType, atlas 索引)` 排序后 sha256
（`assert-td.mjs` `gridDigest()`）。四阶段全同 = P5 重构 / P6 数据外置 / P7 加敌人均未改动地图面
（每格逻辑类型与图集索引逐位不变）。

## 三、截图 md5 链（原始输出见 `chains.log`）

| 文件 | md5 | bytes | 尺寸 | 判定 |
| --- | --- | --- | --- | --- |
| `p4-shot.png` | `4a1fb170d03ed6a5917be21506665cbb` | 72 | **1×1** | 上午故障件（留存作对照，§四） |
| `p4-shot2.png` | `fb6798b75998071bbd94ee8d67973448` | 64454 | 1600×900 | 晚间 P4 复跑（21/21 的那张） |
| `p5-shot.png` | `fb6798b75998071bbd94ee8d67973448` | 64454 | 1600×900 | 与 P4 **逐位一致** |
| `p6-shot.png` | `fb6798b75998071bbd94ee8d67973448` | 64454 | 1600×900 | 与 P5 **逐位一致**（TOML 零漂移的最强证据） |
| `p7-shot.png` | `3a1ee02eadf195ccd913ad881eccc2b6` | 65287 | 1600×900 | 与 P6 **字节差异**（多出敌人 sprite；截于敌人存活期内，见 §六） |

跨运行确定性沿 T044 P3 先例（双击截图逐位一致）：P4→P5→P6 三次独立进程起跑、同场景同相机，
输出 md5 与字节数全同。

## 四、1×1 截图问题：排查与结论

**结论：环境性假红，非代码回归。** 排查阶梯按派工单执行 step 0–1；step 1 即恢复，未进入 step 2。

事实与取证：

1. **阶梯 step 0（Lead 已核，本档引用）**：截图 handler（`game-td/src/brp.rs`）与窗口设置
   与 T044 成功版逐字相同；T044 的 1600×900 截图在档。回归不在 brp.rs。
2. **阶梯 step 1（本单实测）**：晚间空闲窗口复跑同一二进制 + `--phase p4`，
   截图恢复 **1600×900 / 64454 B / md5 fb6798b7…**，断言 21/21、`REAL_EXIT=0`
   （`p4-assert2.log`、`p4-run2.log`）。
3. **winit 显示器行对照（`winit-display-compare.log`）**：上午失败跑（`diag-run.log`，10:38）
   与今晚通过跑（`p5-run.log`）的 `Display information` 行**逐字相同**：

   ```
   Display information:  Window physical resolution: 1280x720  Window logical resolution: 1280x720  Monitor name:   Scale factor: 0  Refresh rate (Hz): 0.000
   ```

   → 「Monitor name 空 / Scale factor 0 / Refresh 0」是本机 winit 显示器枚举的**常量噪声**，
   与 1×1 无因果；它不是健康判据，也不再作为排查线索。真实分辨率语义由捕获结果决定
   （1280×720 逻辑 × DPI 1.25 = 1600×900，与 T044 同机先例一致）。
4. **时间线旁证**：上午失败跑（10:36 起跑、10:37:05 截得 1×1）与诊断跑（10:38:47）均处于
   整机高载走向冻结的窗口内；事件日志冻结时刻 10:44:13 / 重启 10:45:35（见 §九事故注记）。
   晚间接续复跑同一二进制即恢复 → 判定为**高载/临冻结状态下的环境性伪影**（屏幕/交换链尺寸
   在退化状态下被报为 1×1），非产品代码缺陷、非脚本期望值问题。
5. **期望值纪律遵守**：未改脚本期望值（1600×900 = 1280×720 逻辑 × 1.25 DPI，T044 同机先例）；
   未出现「实测 1280×720 + scale=1.0」情形，故未触发上报条件。

## 五、P6b：变体关卡与恢复

- **默认关卡**（`game-td/assets/levels/level_01.toml`，交付态）：
  `[map] 15×10` + `[paths.main_road] waypoints = [[0,5],[14,5]]` —— 宽 1 中心线经加载期
  宽 3 展开 = 行 {4,5,6} 横向道路 45 格，与 P5 `build_demo_map()` 逐格同形（教程 06 格式；
  零漂移是派工单对默认关卡的硬要求）。
- **真变体**（`levels/variant-level_01.toml.txt`，= 教程 06 Step 2 示例同形阶梯路
  `[[2,9],[2,5],[12,5],[12,1]]`，含两个拐点）：临时覆盖 `level_01.toml` 后**免重编译**重启
  （教程 06「editing the TOML changes the map without recompiling」），跑 `--phase p6b`。
- **期望值双盲**：`toml-path-count.mjs` 独立解析 TOML + 独立实现宽 3 展开与拐角对角补齐，
  实跑得 `EXPECTED_PATH=57 EXPECTED_GRASS=93`（`p6b-expected.log`，含逐行形状图）；
  游戏侧实测 `Grass:93 Path:57`，`measured == expected`（`p6b-assert.log` 12/12）。
  默认关卡上同一脚本自校验得 45（与 P6 实测 45 相同）= 脚本自身可信性先证。
- **内角规则生效旁证**（非门禁读数）：变体下 PathTile 索引集合
  `79,80,81,83,102,103,104,105,125,126,127`（`p6b-indices.log`），其中 83/105 各 1 格 =
  教程 06 Step 5 内角规则族被真实命中；逐格索引网格见 `p6b-indices2.log`。
- **恢复**：`cp levels/default-level_01.toml.txt → assets/levels/level_01.toml`，
  恢复后 md5 与快照逐位一致（`p6b-restore.log`：双方均 `a8199db634bada29f75edc432455c4aa`）。
  之后全部 P7 运行与门禁均基于恢复后的默认关卡。

## 六、P7：时序与读数（判据可达性）

- 敌人路径 = 默认关卡 2 航点，全程 14 格 × 64px = 896px，`MoveSpeed(192)` →
  **4.67s 后到终点并被回收**（教程 07 Step 6 语义）。故 p7 断言必须在起跑后 ~4s 内执行：
  本次窗口创建 11:19:50.03Z，首采样 t0 在 ~11:19:53.1Z（起跑约 3.3s），第二采样 +0.4s，
  截图 11:19:53.95Z（起跑约 3.9s，敌人在场）——均在存活窗口内。
- `t0=(194,32) t1=(272,32)`：spawn 点世界坐标 = `(-448, 32)`（`origin_y + 5*64`），
  位移 78.00px / 0.4s ≈ 195px/s（期望 192，+1.56%）；方向 +x、dy=0.000。
- 终末回收旁证（非门禁）：`p7-despawn.log` —— 起跑 ~40s 后 `Enemy count = 0`，
  即「到终点移除 PathFollower → cleanup despawn」链路按教程 07 Step 6 生效。

## 七、门禁与运行纪律

- 全部 cargo 命令：`-j 3`；测试 `-- --test-threads=3`；`CARGO_TARGET_DIR` = 共享 target；
  **全局同一时刻仅一条 cargo**；游戏进程存活期间未跑任何编译（时间线可从各 log 时间戳复核）。
- `cargo check --workspace -j 3`：`REAL_EXIT=0`，0 错 0 警（`gate-check-workspace.log`）。
- `cargo test --workspace -j 3 -- --test-threads=3`：`REAL_EXIT=0`；
  game 20/20、mcp-bridge 15/15、doctest **78 passed / 0 failed / 17 ignored**（基线恢复）；
  其余成员 0 测试（与基线同）（`gate-test-workspace.log`）。
  game 测试产物为共享 target 曾被 `clean -p game` 后的预期增量重建，非冷编风暴（附录 A 安全阀定量边界）。
- 游戏起跑一律 `cargo run -p game-td -j 3`（进程 CWD = `game-td/`）；每次用毕
  `taskkill /F /IM game-td.exe` 清杀，逐次被杀 PID：21052（P4 复跑）/ 20400（P5）/
  23712（P6）/ 15140（P6b）/ 18312（P7）。清杀的可核旁证 = 每轮次跑均重新成功绑定
  15702（同端口二次绑定失败即前实例未死）；收尾终态见 `process-check.log`
  （`grep_exit=1` = 零命中 = 无 game-td / rustc 残留）。

## 八、代码交付清单（相对基线 `d035230`，仅 game-td/** + 根 Cargo.lock）

| 文件 | 变更 | 内容 |
| --- | --- | --- |
| `game-td/src/main.rs` | 改 | P4 auto-tiling 接线（原实例产出）+ P6 改读 TOML + P7 Startup 三系统 chain 与 `FixedUpdate` 挂载、`spawn_test_enemy`（教程 07 Step 3） |
| `game-td/src/map.rs` | 新 | 教程 05 Step 1：`TileType` / `MapTile` / `PathTile` / `MapLayout`（`get` 越界语义） |
| `game-td/src/tiling.rs` | 新 | 教程 05 Step 2：`Match` / `Pattern` / `TileRule` / `TileTypeRuleset` / `TileRules` / `build_rules`；P6 增教程 06 Step 5 内角规则族 82/83/105/106（槽位序换算见代码注释） |
| `game-td/src/level.rs` | 新 | 教程 06 Step 3–4：`LevelData` / `MapData` / `PathData` / `WaveData`、`load_level`、`build_map_from_level`（宽 3 展开 + 拐角对角补齐 + 对角段 panic） |
| `game-td/src/enemy.rs` | 新 | 教程 07 Step 2/4/5/6：`Enemy` / `PathFollower` / `MoveSpeed`、`tile_to_world`、`move_enemies`、`advance_waypoint`、`cleanup_finished_enemies` |
| `game-td/src/brp.rs` | 改 | 组件注册路径随模块化（`crate::map::*`）+ P7 注册 `Enemy`；`td.screenshot` 本体零改动 |
| `game-td/Cargo.toml` | 改 | 增 `toml = "1"`（派工单已裁决放行；锁内 1.1.8） |
| `game-td/assets/levels/level_01.toml` | 新 | 默认关卡（零漂移直线路，见 §五） |
| `Cargo.lock` | 改 | 仅 `game-td` 依赖边 +1 行（`toml`）；无新增包（1.1.8 此前已在锁内） |

**反射类型路径冻结**：模块化会把反射路径改为 `game_td::map::MapTile` 等，而 BRP `world.query`
按**完整类型路径**查注册表（`bevy_remote-0.20.0/src/builtin_methods.rs:1183` `get_with_type_path`）。
故对 `TileType` / `MapTile` / `PathTile` / `Enemy` 加 `#[type_path = "game_td"]`
（`bevy_reflect_derive-0.20.0/src/derive_data.rs:211`），BRP 组件名与证据脚本口径（含网格指纹算式）
在模块化后逐字不变——P4/P5/P6 指纹与截图逐位一致即其行为验证。

## 九、补位留痕与事故注记

- **补位事实**：原实例 T047-W1 于 **2026-10-30 10:45** 随宿主整机冻结硬重启死亡（事故属 Lead 侧，
  与本卡执行无涉）；树 `trees/m6/w1` 进度保留（P4 本体 + 断言脚本 + P4 日志）。本单为**续做单**：
  按收割判定「有进度 → 进树续做至绿」，未重做 P4 本体，未改动其代码语义（仅 P5 起按教程拆分）。
- **事故时间戳（如实记录）**：10:44:13 资源耗尽/冻结时刻、10:45:35 异常重启（见 Lead 侧事件日志
  与 T045 提交说明）；上午失败件 `p4-shot.png`（1×1, 72B）与 `diag-run.log` 原样留存。
- **本单起跑状态**：晚间空闲（CommitFree_G=81.4 / Limit 95.8，无 cargo / game-td 进程），
  按附录 A 预检后开工。

## 十、偏离与披露（自查项，均不影响验收语义）

1. **P5「module/plugin 结构」措辞**：教程 05 明确把 Plugin 列为「deliberately avoided」，
   其行为规格 = `map.rs` / `tiling.rs` / 薄 `main.rs` 的**模块拆分**（无插件）。本单按教程实现
   模块拆分；未引入 `MapPlugin`/`TilingPlugin`（教程之外的架构）。判定：教程为行为规格 → 以教程为准，
   派工单措辞按此解释。
2. **`assert-td.mjs` p6b 参数化 bug 修复**（派工单允许「修 bug」）：首跑 p6b 时 `assertMap()` 内
   `TileType Grass==105` / `TileType Path==45` 为 P4/P5/P6 专用硬编码，未随 `--expect-path`
   参数化，导致变体跑出 10/12（两 FAIL 即此两行）。最小修复：两行期望值改用
   `MAP_TILE - measured` / `measured`；`measured` 缺省仍 = `PATH_TILE` = 45，故
   **p4/p5/p6 判定值与修复前逐字相同**（对应日志无需重跑即口径一致）。修复后 p6b 12/12。
   首次（修复前）p6b 输出为 10/12、`REAL_EXIT=1`；该首跑未单独留档（已被重跑覆盖，此为主观失误，
   如实披露），本档以修复后 `p6b-assert.log` 为准，并在其命令行注明「重跑（脚本 p6b 参数化 bug 已修）」。
3. **异步时序**：P4/P6b/P7 的复跑时间戳之间隔较大（含 `cargo build` 14.7s 链接），日志时间戳
   可逐条复核，无删档。

## 十一、SOURCE-DEVIATION-NOTE（教程 = 行为规格；形态差异登记）

```
SOURCE-DEVIATION-NOTE 1: game-td/src/tiling.rs:Match/Pattern 源=教程 04 `pat(n,s,e,w,ne,nw,se,sw)` 八槽具名结构 本侧=[Match;8] 数组 + NEIGHBORS 槽位序 [N,NE,E,SE,S,SW,W,NW]（P4 既有形态） 理由=匹配语义与图集索引逐格等价（P4 断言逐格验证；教程 06 内角四规则已按槽位名逐一换算） 裁决=T047-W1
SOURCE-DEVIATION-NOTE 2: game-td/assets/levels/level_01.toml 源=教程 06 Step 2 示例阶梯路 [[2,9],[2,5],[12,5],[12,1]] 本侧=默认关卡直线路 [[0,5],[14,5]]（格式同构） 理由=派工单要求默认 TOML = P5 行为零变化；教程原形改作 P6b 真变体实测（含内角规则命中） 裁决=T047-W1
SOURCE-DEVIATION-NOTE 3: game-td/src/map.rs:TileType 等 源=教程 05 仅 `#[derive(Component, …)]` 本侧=追加 `Reflect` + `#[type_path = "game_td"]` + `#[reflect(Component)]` 理由=工作流三要求组件可被 BRP 查询；type_path 冻结反射路径使模块化不改 BRP 组件名 裁决=T047-W1
SOURCE-DEVIATION-NOTE 4: game-td/src/enemy.rs:move_enemies 源=教程 07 正文「FixedUpdate … 60 Hz by default」 本侧=按引擎实测 64 Hz（`Time<Fixed>` DEFAULT_TIMESTEP 15625µs） 理由=引擎默认值事实；行为规格（每 tick 走 speed×固定步长）不变 裁决=T047-W1
```

## 十二、证据索引（本目录）

| 文件 | 内容 |
| --- | --- |
| `README.md` | 本总账 |
| `assert-td.mjs` | 全波断言脚本（p4/p5/p6/p6b/p7 五 phase；判定行由脚本计算） |
| `toml-path-count.mjs` | P6b 独立 TOML 展开计数（含逐行形状图；期望值实跑推导） |
| `levels/default-level_01.toml.txt` | 默认关卡快照（恢复基准，md5 a8199db6…） |
| `levels/variant-level_01.toml.txt` | P6b 真变体关卡（教程原形阶梯路） |
| `diag-run.log` | 上午诊断跑（含 winit 显示器行；不属证据档，留作对照） |
| `p4-run.log` / `p4-build.log` / `p4-check.log` / `p4-assert.log` / `p4-shot.png` | 上午 P4 原始批（断言 20/21，截图 1×1 故障件） |
| `p4-run2.log` / `p4-assert2.log` / `p4-shot2.png` | 晚间 P4 收尾复跑（21/21，1600×900） |
| `p5-check.log` / `p5-run.log` / `p5-assert.log` / `p5-shot.png` | P5 重构（21/21；含 winit 显示器行） |
| `p6-check.log` / `p6-build.log` / `p6-run.log` / `p6-assert.log` / `p6-shot.png` | P6 默认 TOML（21/21） |
| `p6b-expected.log` / `p6b-swap.log` / `p6b-run.log` / `p6b-assert.log` / `p6b-indices.log` / `p6b-indices2.log` / `p6b-restore.log` | P6b 变体（独立计数、换入/恢复留痕、12/12、内角旁证） |
| `p7-check.log` / `p7-build.log` / `p7-run.log` / `p7-assert.log` / `p7-shot.png` / `p7-despawn.log` | P7 敌人寻路（25/25、终末回收旁证） |
| `gate-check-workspace.log` / `gate-test-workspace.log` | 双门禁 |
| `chains.log` | 截图 md5 链 + 网格指纹链（原始输出） |
| `winit-display-compare.log` | 上午失败跑 vs 晚间通过跑的 winit 显示器行对照 |
| `process-check.log` | 收尾终态进程核查（零 game-td / rustc 残留） |

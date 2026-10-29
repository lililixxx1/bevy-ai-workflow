# T044-W1 证据总账 · M6 塔防 spike Part 1–3（`game-td`）

- 基线 commit：`424a079`（主仓 HEAD；隔离树由它导出，无 `.git` 属正常物理隔离）。
- 目标：bevy 0.20 上按外部教程 Part 1→2→3 逐部实现**行为** parity，每部留运行时 BRP 证据与挂钟耗时，产出 `bevy_ecs_tilemap` 0.20 兼容性结论。
- 教程角色：行为规格、**非实现来源**（教程代码为 bevy 0.18.1 形态）；所有 API 按 bevy 0.20.0 / bevy_ecs_tilemap 0.20.0 本地源码查证后重写，锚点见 §4。
- 环境：Windows 10 19044 / NVIDIA RTX 3050（Vulkan，driver 537.58）/ rustc 1.98.1 / Node v24.12.0；全部 cargo 命令携带 `CARGO_TARGET_DIR=<main-repo>/target` 与 `-j 3`（附录 A）。

## 1. 结论

判定行（脚本计算）：**`tilemap-0.20: PASS`**（判据：V1 编译过 + P3 阶段 V6 计数/索引/截图全过）。

| 判定 | 结果 | 判据 |
| --- | --- | --- |
| **`tilemap-0.20`** | **PASS** | V1 编译过（`bevy_ecs_tilemap` 0.20.0 随 `game-td` 编译，exit 0、0 warning）+ P3 阶段 V6 组件计数/索引/截图全过（`v6-assert-p3.log` 17/17、终态复跑 `v6b-assert-p3-final.log` 17/17） |
| P1 行为 parity | PASS | `v4-assert-p1.log` 15/15 |
| P2 行为 parity | PASS（索引值断言口径见 §6.1） | `v5-assert-p2.log` 16/16 |
| P3 行为 parity | PASS | `v6-assert-p3.log` 17/17（+ 终态复跑 17/17） |
| BRP 方法面 | **24 = 23 内置 + 1 自定义** | `v3-discover.log`（脚本计算 `METHODS_TOTAL=24` / `METHODS_BUILTIN=23` / `METHODS_CUSTOM=1`）；三阶段原始响应 `discover-p1/p2/p3.json` 均 24 |
| 最终提交态 | **P3 形态**（`TileBundle` + `TilemapBundle` + `TileStorage`） | `game-td/src/main.rs` |

判定行由脚本计算生成：断言档 `SUMMARY: n/n PASS` 与 `v3-discover.log` 的计数行均来自 `assert-td.mjs` / `self-check.mjs`，无手填。

## 2. 验收逐条（V1–V8）

命令均在树根单独整句执行；`<env>` = `CARGO_TARGET_DIR=<main-repo>/target`。

| # | 命令 | 期望 | 实测 | 证据 |
| --- | --- | --- | --- | --- |
| V1 | `cargo check -p game-td -j 3` | exit 0 且无 `warning:` | **exit 0，0 warning**（P1 代码全量 check 50.29s；终态代码 `p3-check.log` 1.14s；终态复跑 `v1-check-final.log` 0.83s） | `v1-check-p1.log` / `p3-check.log` / `v1-check-final.log` |
| V2 | `cargo check --workspace -j 3` | exit 0 且无 `warning:` | **exit 0，0 warning**（4s；终态复跑 0.99s；既有面 game/docs/tooling/* 全通过） | `v2-check-workspace.log` / `v2-check-final.log` |
| V3 | `cargo run -p game-td -j 3`（后台）+ BRP `rpc.discover` | 方法总数 **24** = 23 内置 + 1 自定义 | **24**（`METHODS_BUILTIN=23` / `METHODS_CUSTOM=1` / `HAS_TD_SCREENSHOT=true`；自定义唯一项 `td.screenshot`，无 `game.*` 前缀） | `v3-discover.log`（终态，含命令 + 原始响应 + `CURL_EXIT=0` + 脚本计数）；`discover-p1/p2/p3.json` |
| V4 | P1：BRP 调 `td.screenshot` | PNG 落盘非空 + 记录像素尺寸；进程存活 | **PASS**：`p1-shot.png` 23348 字节，PNG 魔数 ✓，IHDR **1600x900**（= 逻辑 1280×720 × DPI 1.25）；受理时进程存活（`p1-run.log` BRP 监听 + 建窗行） | `v4-assert-p1.log`（15/15）、`p1-shot.png`、`p1-run.log` |
| V5 | P2：BRP query 自有组件计数 | `MapTile`==150 且 `PathTile`==45 + 截图 | **PASS**：150 / 45（150=15×10；45=3×15，路行 {4,5,6}）；草 150−45=105；150 格坐标与居中算式逐格一致；截图 `p2-shot.png` 63592 字节 1600x900 | `v5-assert-p2.log`（16/16）、`p2-shot.png` |
| V6 | P3：同 V5 断言 | 150 / 45 + 截图 | **PASS**：150 / 45；`TilePos` 覆盖 15×10 恰一次；**150 格 atlas 索引逐格与规格一致（bad=0）**；路行 y∈{4,5,6}；无逐格 `Sprite`；截图 `p3-shot.png`/`p3-final-shot.png` 64454 字节 1600x900 | `v6-assert-p3.log`（17/17）、`v6b-assert-p3-final.log`（17/17）、`p3-shot.png`、`p3-final-shot.png` |
| V7 | 收尾 `taskkill` 后 `tasklist` | 无 `game-td.exe` 残留 | **PASS**：两次收尾 `TASKKILL_EXIT=0`；残留检查空输出、`GREP_EXIT=1`、`RESIDUE_COUNT=0` | `v7-taskkill.log`、`v7-tasklist.log`、`v7-tasklist-full.log`（原始全量快照） |
| V8 | 证据档自检 | `SELF_CHECK_PASS` 行 | 见 §8（`v8-selfcheck.log`，由 `self-check.mjs` 计算生成） | `v8-selfcheck.log` |

关键算式（派工单口径，脚本内独立复算）：
- `MapTile` 150 = 15 列 × 10 行；`PathTile` 45 = 3 路行 × 15 列（`path_mid = 10/2 = 5`，路行 {4,5,6}）。
- 路行 atlas 索引集 {79,80,81,102,103,104,125,126,127} 全 ≠ 129；其余 105 格 = 129（草）。
- 居中坐标：`offset = -(n × 64)/2 + 32` → x ∈ {-448..448, 步长 64}、y ∈ {-288..288}；路行 y ∈ {-32, 32, 96}。
- 物理像素 1600×900 = 1280×720 × DPI 1.25（SKILL §6.8：截图为物理分辨率）。

时点说明（如实）：P3 运行时证据（`p3-run.log` / `p3-shot.png` / `p3-final-run.log` / `p3-final-shot.png` / `v6*`）采集后，终态源码只发生一处**行内注释锚点勘误**（`TilemapPlugin` 依据行 `lib.rs:64`→`:62`，见 `game-td/src/main.rs`），**语义零差异**；终态 `v1-check-final.log` / `v2-check-final.log` 在该勘误后复跑（均 exit 0、0 warning）。

## 3. 交付物与证据索引

树内落盘（全部在本隔离树）：

| 路径 | 内容 |
| --- | --- |
| `Cargo.toml` | 根 workspace：`members` 增 `"game-td"`；`[workspace.dependencies]` 增 `bevy_ecs_tilemap = "0.20"`（两处增行，其余零改动） |
| `Cargo.lock` | 依赖解析副作用：仅新增 `bevy_ecs_tilemap 0.20.0`（checksum `fe378ed0…`）与 `game-td 0.1.0` 两个 package 条目；既有包的 name/version 逐行 diff 零变化（7089→7109 行） |
| `game-td/Cargo.toml` | edition 2021；`bevy = { workspace = true, features = ["bevy_remote"] }`；`bevy_ecs_tilemap = { workspace = true }`；`serde`/`serde_json`（对齐 `game/Cargo.toml` 形态） |
| `game-td/src/main.rs` | 窗口契约 + P3 形态地图（`MapTile`/`PathTile` + `TilemapPlugin` + `TileBundle`/`TilemapBundle`/`TileStorage`） |
| `game-td/src/brp.rs` | `127.0.0.1:15702` 显式回环绑定 + 唯一自定义方法 `td.screenshot`（受理 + 状态查询）+ `MapTile`/`PathTile` 反射注册 |
| `game-td/assets/Tilesheet/towerDefense_tilesheet.png` | 91302 字节，sha256 `3681f42ae963aa4eb16063170642fe2957d38b94838aba2aa5064112863d1ec3`（与侦察源一致） |
| `game-td/assets/Tilesheet/License.txt` | Kenney CC0 许可原文 |

证据档（本目录）：

| 文件 | 用途 |
| --- | --- |
| `README.md` | 本总账（索引 + 结论 + 判定行 + 自检） |
| `timing.md` | 各部挂钟起止 + cargo 编译等待单列 |
| `assert-td.mjs` | 断言脚本（`--phase p1/p2/p3 --shot <path>`；判定行脚本生成） |
| `self-check.mjs` | 证据档自检脚本（V8） |
| `v1-check-p1.log` / `v1-check-final.log` | V1（P1 代码 / 终态复跑） |
| `v2-check-workspace.log` / `v2-check-final.log` | V2（P1 时点 / 终态复跑） |
| `p1-build.log` / `p2-build.log` / `p3-build.log` | 各部 debug 构建（均 0 警） |
| `p1-run.log` / `p2-run.log` / `p3-run.log` / `p3-final-run.log` | 各部运行日志（含 BRP 监听/建窗/截图落盘行） |
| `discover-p1.json` / `discover-p2.json` / `discover-p3.json` / `v3-discover.log` | V3 方法清单（逐阶段原始响应 + 终态含命令/退出码/脚本计数） |
| `v4-assert-p1.log` / `v5-assert-p2.log` / `v6-assert-p3.log` / `v6b-assert-p3-final.log` | 各阶段断言输出（均 `REAL_EXIT=0`） |
| `p1-shot.png` / `p2-shot.png` / `p3-shot.png` / `p3-final-shot.png` | 各阶段截图（1600×900） |
| `p2-check.log` / `p3-check.log` | 各部实现后的 `cargo check -p game-td`（0 警） |
| `v7-taskkill.log` / `v7-tasklist.log` / `v7-tasklist-full.log` | V7 进程收尾与残留检查 |
| `claim-lint-supplementary.log` / `claim-lint-supplementary-mtimes.log` | 补充门禁（非验收项）：`claim-lint` 在隔离树内的状态与假红归因（见 §6.5） |
| `v8-selfcheck.log` / `v8-selfcheck-run1.log` | V8 自检输出（run1 = 首跑，见 §8 说明） |

## 4. API 查证台账（bevy 0.20.0 / bevy_ecs_tilemap 0.20.0 本地源码）

`<reg>` = `~/.cargo/registry/src/` 下 registry 目录；`<tm>` = 侦察目录解包的 `bevy_ecs_tilemap-0.20.0/src/`（只读）。代码内锚点与本表同源。

| API | 锚点 | 用途 |
| --- | --- | --- |
| `Camera2d` | `bevy_camera-0.20.0/src/components.rs:16` | P1 相机 |
| `Sprite::from_color` / `from_atlas_image` | `bevy_sprite-0.20.0/src/sprite.rs:73` / `:64` | P1 红方块 / P2 瓦片 |
| `Color::srgb` | `bevy_color-0.20.0/src/color.rs:123` | P1 纯红 |
| `TextureAtlasLayout::from_grid` | `bevy_image-0.20.0/src/texture_atlas.rs:133-139` | P2 23×13 切图 |
| `TextureAtlas { layout, index: usize }` | `bevy_image-0.20.0/src/texture_atlas.rs:214-218` | P2 索引 |
| `ImagePlugin::default_nearest` | `bevy_image-0.20.0/src/image.rs:209` | 像素美术取样（P2/P3） |
| `WindowMode::Windowed` | `bevy_window-0.20.0/src/window.rs:1338` | 窗口契约 |
| `RemotePlugin` / `with_method_main` | `bevy_remote-0.20.0/src/lib.rs:572` / `:591` | BRP 接线 |
| `RemoteHttpPlugin` / `with_address` / 默认端口 | `bevy_remote-0.20.0/src/http.rs:113` / `:167` / `:52`（`DEFAULT_PORT=15702`）、`DEFAULT_ADDR` `:60` | 显式回环绑定 |
| `BrpResult` / `BrpError` / `error_codes` | `bevy_remote-0.20.0/src/lib.rs:1428` / `:1304` / `:1387` 起 | handler 签名与 -32602 |
| `Screenshot` / `ScreenshotCaptured` / `save_to_disk` | `bevy_render-0.20.0/src/view/window/screenshot.rs:84` / `:53` / `:145` | `td.screenshot` 管线 |
| `clear_screenshots` 定义 / First 注册 | 同文件 `:255` / `:473-479`（**注：主仓 `game/src/rpc/screenshot.rs` 模块注释原带 0.19 锚 `:197`/`:416-418`——G2 前勘误归因，主仓已随 T044 收尾批重锚；见 §6.3**） | 捕获实体清理 |
| `register_type` | `bevy_app-0.20.0/src/app.rs:664` | 组件反射注册（SKILL §3.4） |
| `TilemapPlugin` | `<tm>/lib.rs:62`（Plugin impl `:64`） | P3 渲染插件 |
| `TilemapBundle` 别名 / `MaterialTilemapBundle` | `<tm>/lib.rs:115` / `:120`（默认材质注入 `render/mod.rs:124-130`） | P3 父实体 |
| `TileBundle` 字段 | `<tm>/tiles/mod.rs:114-124` | P3 逐格 bundle |
| `TilePos` / `TileTextureIndex` | `<tm>/tiles/mod.rs:17` / `:73` | 网格坐标 / 图集索引 |
| `TilemapId` / `TileStorage::empty`/`set` | `<tm>/map.rs:55` / `tiles/storage.rs:32` / `:65` | 父子关联 / 存储 |
| `TilemapSize` / `TilemapTileSize` / `TilemapGridSize` | `<tm>/map.rs:72` / `:214` / `:281` | 尺寸 |
| `TilemapTexture::Single` / `TilemapType::Square` | `<tm>/map.rs:125` / `:464-467` | 纹理 / 方形网格 |
| `TilemapAnchor::Center` | `<tm>/anchor.rs:13`（居中算式 `render/prepare.rs:68`） | 居中于原点 |
| 非 atlas 路径的图集数组层序 | `<tm>/render/texture_array_cache.rs:275-315`（行主序切片）、`:171`/`:220`（层数 = 切图数） | 索引语义与 P2 一致 |
| tilemap 采样器取 `ImagePlugin` 默认值 | `<tm>/render/mod.rs:153-156` | nearest 生效 |

## 5. 行为规格对照（教程 Part 1–3 → 实现 → 证据）

| 阶段 | 行为规格要点 | 实现位置 | 证据 |
| --- | --- | --- | --- |
| P1 | `App` + `DefaultPlugins` + `Startup setup`；`Camera2d`×1；红色方块 Sprite×1（srgb(1,0,0)、100×100、世界原点） | `game-td/src/main.rs` `setup`（P1 版本见 `v4-assert-p1.log` 时点源码；终态已删） | 15/15：相机 1、Sprite 1、颜色/尺寸/坐标 |
| P2 | 移除红方块；tilesheet + `TextureAtlasLayout(64×64, 23×13)`；15×10 居中网格；三行道路（行 4/5/6，索引 125/126/127、102/103/104、79/80/81）；其余 129；`MapTile` 全挂、`PathTile` 仅索引≠129 | 同文件 P2 版本（`setup` 内 match + 逐格 spawn） | 16/16：150/45、105 草、坐标逐格、路行 y |
| P3 | `TilemapPlugin` + `ImagePlugin::default_nearest`；父实体 `TilemapBundle` + 逐格 `TileBundle` + `TileStorage`；标记保持、视觉不变 | `game-td/src/main.rs` 终态 `setup` | 17/17：150/45、`TilePos` 15×10、索引逐格、路行 {4,5,6}、无逐格 Sprite、截图 |

## 6. 已知限制与发现（如实登记）

### 6.1 P2 的 atlas 索引值无法经 BRP 读出（限制 + 替代覆盖）

- 现象：对 P2 瓦片 `world.get_components`（strict）查 `bevy_sprite::sprite::Sprite` 报
  `-23402 type bevy_platform::sync::Arc<bevy_asset::handle::StrongHandle> did not register the ReflectSerialize…`（stack: `Sprite` → `Handle<Image>` → `Arc<StrongHandle>`）；`world.query` 则静默省略该组件（`p2-run.log` 有对应 WARN 行）。
- 原因：P1 红方块的 `Handle<Image>` 是默认（弱）句柄，可序列化为 `{"Uuid":…}`；P2 载入真实图片后是强句柄，`StrongHandle` 未注册 `ReflectSerialize`。
- 处置：P2 断言改为网格几何（150 格坐标逐格）+ 标记语义（150/45/105、路行 y）；**索引值逐格断言由 P3 的 `TileTextureIndex`（可序列化）覆盖**——两阶段使用同一 match 逻辑，P3 的 `bad=0` 即索引规格成立的证据。不为此改动游戏反射注册面。
- 若 Lead 要求 P2 也做索引级断言：需新增可反射镜像资源或用 `registry.schema` 之外的通道（属范围外决策，未做）。

### 6.2 `bevy_ecs_tilemap` 0.20 默认 feature 的图集数组层数前提

- 0.20.0 默认 features = `["render"]`（`<tm>/../Cargo.toml` `[features]`；`atlas` 非默认）。非 atlas 路径把整张 tilesheet 按 `tile_size` 切成 **299 层**（23×13）纹理数组（`texture_array_cache.rs:171`）。
- wgpu 的 `Limits::default()` 上限是 256 层，但 bevy 默认 `WgpuSettingsPriority::Functionality` 会改用 **adapter limits**（`bevy_render-0.20.0/src/settings.rs:89`、`src/renderer/mod.rs:332`）；本机 RTX 3050 / Vulkan 实测支持并成功渲染（`p3-run.log` 无验证错误，截图 1600×900）。
- **兼容性提示**：若目标设备 `max_texture_array_layers < 299`，默认路径会在建纹理时失败；届时需在成员层加 `features = ["atlas"]`（走普通 2D 图集 + UV 计算，`render/shaders/tilemap_vertex.wesl:31-45 ATLAS 分支`）——属依赖 feature 变更，**未自行启用**，留待 Lead 裁决。
- 另：`ArrayTextureLoader` 预载是可选的（抽取期 `render/mod.rs:279-290` + `render/extract.rs:350-360` 自动登记），本实现未用预载，首帧即可渲染。

### 6.3 screenshot.rs 行号漂移（已上报项；G2 前归因勘误）

- 上报原文指向「SKILL §6.8 记 `:197`/`:416-418`」——**G2 前勘误**：主仓 SKILL §6.8 现行锚（`ScreenshotCaptured :53`、`save_to_disk :145`）已是 0.20 正锚，不带该两项；实际携带 0.19 锚 `clear_screenshots :197`、First 注册 `:416-418` 的是主仓 `game/src/rpc/screenshot.rs` 模块注释（T042 迁移漏扫面）。漂移事实本身不变：0.20.0 本地源码实为 `:255` / `:473-479`。
- 处置：worker 只记录未改 SKILL（正确）；主仓侧已随 T044 收尾批重锚（连同 game/src 其余 0.19 注锚清扫）。

### 6.4 `cargo run` 强制收尾的退出码

- 各阶段 `cargo run` 由 `taskkill /F /IM game-td.exe` 收尾，日志末行 `REAL_EXIT=1` 是**强制终止**后的 cargo 报码（`error: process didn't exit successfully … exit code: 1`），非应用失败；应用侧 BRP 全程可用（discover/断言均成功）。

### 6.5 补充门禁 `claim-lint`：隔离树内假红（非基线缺陷、非本卡改动）

- `cargo run -p claim-lint -j 3` → 12 PASS / 1 FAIL / 0 SKIP，`REAL_EXIT=1`；FAIL 项 = `doctest-gate`（选中 `docs/evidence/ts-17/gate-doc-test-t040.log` 的 77/0/17，期望 78/0/17）。档：`claim-lint-supplementary.log`。
- 归因（已核）：主仓内 `gate-doc-test*.log` 的 mtime 有序，最新为 T043 执行批终档 `gate-doc-test-t043-exec.log`（78/0/17，2026-10-29 22:25）；**隔离树导出把全部 mtime 压平为同一时刻 22:38:43**（见 `claim-lint-supplementary-mtimes.log` 的主仓/树对照），`claim-lint` 的「按 mtime 降序回退式挑选」因此落到旧档 → 假红。
- 结论：① 该 FAIL **不是**基线 424a079 的缺陷，也**不是**本卡改动所致（本卡未触碰 claim-lint / doctest / 期望表 / 任何 gate-doc-test 档）；② 隔离树内 `claim-lint` 的 doctest-gate 断言不可信，**G1/G2 请在收获后的主仓跑该门禁**（主仓 mtime 真实有序）。同档另证：`game-methods` 断言 **PASS**（`game.*` 计数仍为 8——`td.screenshot` 未污染该计数面）、`ledger` 断言 PASS。
- 建议（Lead 裁决）：把「隔离树 mtime 压平 → claim-lint doctest 按 mtime 选档失效」登记进附录 B.1 误报清单，或给隔离树导出加 `--preserve-timestamps`。本卡不自行改附录。

## 7. 预注册预期 vs 实测（耗时）

派工单预注册：P1 ≈40min / P2 ≈90min / P3 ≈90min / 验证与证据整理 ≈80min（合计 ≈300min）。

| 部 | 预期 | 实测墙钟 | 其中编译等待 | 净实现+验证 |
| --- | --- | --- | --- | --- |
| P1 | ≈40min | 10m21s | 4m03s | 6m18s |
| P2 | ≈90min | 3m53s | 22.8s | 3m30s |
| P3 | ≈90min | 3m36s | 17.2s | 3m19s |
| 验证与证据整理 | ≈80min | ≈16min | 1.82s | ≈16min |

- **偏离：全部显著低于预期**（合计 ≈49min vs 预期 ≈300min）。归因（如实）：① 教程 Part 1–3 行为面很小（共 3 个 setup + 1 个 match）；② 共享 target 已含 0.20 依赖的 check 元数据，只有首次 debug codegen 3m10s；③ API 查证以本地源码 grep + 行号锚点为主，未走 docs.rs 网络往返。
- 逐部明细与编译等待分列见 `timing.md`。

## 8. 自检清单（V8）与路径清理声明

自检由 `self-check.mjs` 计算生成（`v8-selfcheck.log`），检查项：

1. 必备文件齐备（§3 索引全表）。
2. 四个断言档 `SUMMARY` 自洽（n/n PASS，n>0）。
3. 每个 `.log` 含退出码/证据行（`REAL_EXIT` / `CURL_EXIT` / `TASKKILL_EXIT` / `TASKLIST_EXIT` / `RESIDUE_COUNT`）。
4. 四张截图 PNG 魔数 + IHDR 正尺寸。
5. 三份 discover 原始响应均 24 方法且含 `td.screenshot`。
6. 全部 log/json/md/mjs 零机器绝对路径。
7. 素材 sha256 前 16 位 `3681f42ae963aa4e` + 91302 字节。
8. README 含判定行与 V1–V8 索引。

**路径清理声明**：证据日志落盘后做过一次前缀清理——把树路径与主仓路径前缀替换为 `<tree>` / `<main-repo>`（共 12 个文件命中，只改前缀；行号、行内容其余部分与输出字节次序未动）。对 `v7-tasklist.log` 做过一次**格式整理**（两次 taskkill 输出逐字保留；仅去掉两处因 harness 计数命令产生的空 `RESIDUE_COUNT=` 行与一处乱码注释行）；原始全量快照另存 `v7-tasklist-full.log`。

### 自检运行结果（`v8-selfcheck.log` 逐字摘录）

```
PASS | 必备文件齐备 | count=32
PASS | v4-assert-p1.log SUMMARY 全 PASS | 15/15
PASS | v5-assert-p2.log SUMMARY 全 PASS | 16/16
PASS | v6-assert-p3.log SUMMARY 全 PASS | 17/17
PASS | v6b-assert-p3-final.log SUMMARY 全 PASS | 17/17
PASS | 每个 .log 含退出码/证据行 | logs=23
PASS | p1-shot.png PNG 魔数/尺寸 | magic=89504e470d0a1a0a IHDR=1600x900 bytes=23348
PASS | p2-shot.png PNG 魔数/尺寸 | magic=89504e470d0a1a0a IHDR=1600x900 bytes=63592
PASS | p3-shot.png PNG 魔数/尺寸 | magic=89504e470d0a1a0a IHDR=1600x900 bytes=64454
PASS | p3-final-shot.png PNG 魔数/尺寸 | magic=89504e470d0a1a0a IHDR=1600x900 bytes=64454
PASS | discover-p1.json methods==24 + td.screenshot | len=24 custom=1
PASS | discover-p2.json methods==24 + td.screenshot | len=24 custom=1
PASS | discover-p3.json methods==24 + td.screenshot | len=24 custom=1
PASS | 零机器绝对路径 | scanned=32
PASS | tilesheet sha256/字节数 | sha256=3681f42ae963aa4e... bytes=91302
PASS | README 含 tilemap 判定行
PASS | README 含验收 V1-V8 索引
SUMMARY: 17/17 PASS
SELF_CHECK_PASS
REAL_EXIT=0
```

说明：首跑（`v8-selfcheck-run1.log`，16/17）有 1 项 FAIL——检查器扫到**自身输出档**（`v8-selfcheck.log` 生成中、尚无退出码行），属检查器自引用缺陷而非证据缺陷；修正是把自检自身输出档排除出「每 .log 含退出码」扫描（改动只在 `self-check.mjs`）。首跑档如实保留。

**SELF_CHECK_PASS**

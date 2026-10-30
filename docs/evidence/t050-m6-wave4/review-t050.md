# G2 · 完整轮审核档（T050 M6 波次 4 P18-21 节点卡）

日期 2026-10-31 凌晨；审核者 plan-code-reviewer（**完整轮**——节点卡，含独立复算）；对象 = 树 `trees/m6/w4`（基线 41ec1a2）批次 T050 全量产出。本档为审核者报告原文归档（结论/阻塞/建议/复算原文如实转录）。

## 合并裁决

**裁决：通过（✅ 可合并 / 可收获）**

T050 波次 4（P18–21）四部行为等价成立、断言面为真实行为谓词、指纹链零漂移、席位与纪律账实相符。G1 六项核证与 U1–U6 裁决全部同意，无阻塞项，可转收获批（`m6-game-selection.md` §五 td.* 面更新 + SKILL v0.21 §6.13 增补 K1 + 台账注记 10/10 已满）。

## 审核范围

- 基线：主仓 `C:\Users\Administrator\Desktop\ccc\bevy-ai-workflow` HEAD=`41ec1a2`，`git status --porcelain` 零输出（亲核，主仓未动）。
- 对象：`C:\Users\Administrator\Desktop\ccc\trees\m6\w4`（非 git 树，无 `.git`；以主仓 `41ec1a2` 的 `git archive | tar -x` 纯净导出为比对基线，`--strip-trailing-cr` 归一）。
- 教程规格：`C:\Users\Administrator\Desktop\ccc\trees\m6-recon\tutorial-src\{18-custom-events,19-gizmos,20-audio,21-plugins-and-pause}.md`（行为/数据面，不含实现）；漂移底稿 `trees/m6/w4/docs/evidence/t044-td-spike/w2/`。
- 全量 diff 结果（亲跑）：改 5 文件 `game-td/src/{main,tower,economy,brp}.rs` + `game-td/CREDITS.md`；增 `game-td/src/{audio,pause}.rs` + `game-td/assets/audio/*.ogg`×4；`docs/` 仅增 `docs/evidence/t050-m6-wave4/`（45 件）；`Cargo.toml` 根/`game-td` 与 `Cargo.lock` 三文件逐字节 SAME。恰为 worker 声明清单，无越权写面。

覆盖率：改动面 12 项 — 已审 12 / 跳过 0 / 覆盖率 100%。

## 七维度

**1. 范围 — PASS**
`diff -r --strip-trailing-cr` 主仓基线↔树仅上述差异；`scope-check.log:3-10` 与亲跑一致；`docs` 差异仅新证据目录（亲跑 `Only in ... t050-m6-wave4`）。G1 称"44 件"实为 45 件（`ls|wc -l`=45），系 G1 笔误，不影响写面结论（见 Minor）。

**2. 行为等价（P18–21 逐部对教程 Step）— PASS，偏离 6 条全部采信**

- P18（`18-custom-events.md:19-25,37-47,124-183,194-266`）：`PlaceTower{tile,world_pos,tower_id,cost}`（`trees/m6/w4/game-td/src/tower.rs:272-286`）、生产者只校验（`tower.rs:703-760`，不再持有 `ResMut<Gold/PlacedTowers>`/atlas/生成逻辑，买不起红闪保留与教程 `186` 一致）、两消费者各 `read()+assert!(iter.next().is_none())`（`tower.rs:762-790`、`economy.rs:115-136`）、`.after(place_tower_on_click)` 同帧闭环（`main.rs:266-269`）、`.add_message::<PlaceTower>()`（`main.rs:178`）。`td.events emit_place` 复用同一 `is_placeable`+余额门（`brp.rs:1046-1118`），`World::write_message` 未注册显式 -32602（`brp.rs:1119-1124`，教程未覆盖的 0.20 真实语义，属加固）。
- P19（`19-gizmos.md:74-146,168-235`）：`PlacedTowers(HashMap<[u32;2],Entity>)`（`tower.rs:161-169`）、`contains_key`（`tower.rs:399`）、两生成函数返 turret 实体（`tower.rs:444,582`）、`spawn_tower_at`/`spawn_tower_entities` 共用出口（`tower.rs:575-602`）、`deduct_gold_on_placement` 收缩为只扣金（`economy.rs:115-136`，与教程 `131-146` 一致）、`draw_tower_ranges` 双分支（悬停白圈 0.5/预览金圈 0.4，`tower.rs:1244-1277`，常量 `PREVIEW_RING_RGBA`/`HOVER_RING_RGBA` 与教程 `213-217` 一致）、纯函数 `hover_ring_params`/`preview_ring_params`（`tower.rs:1215-1242`）供绘制与 `td.gizmos probe` 共用。
- P20（`20-audio.md:80-187,211-277`）：`AudioAssets/BackgroundMusic/SoundType/PlaySound`（`audio.rs:37-62`）、四系统（`audio.rs:86-136`）、接线 Startup装载/OnEnter起播/OnExit停播/Update消费（`audio.rs:147-159`）、`LOOP+0.3`（`audio.rs:99,MUSIC_VOLUME`）、`DESPAWN+volume`（`audio.rs:125-128`）、弹体挂 `AudioPlayer+LOOP/Linear(2.0)`（`tower.rs:1006-1010`）、激光 `volume 1.0`（`tower.rs:845-849`）、爆炸 `volume 0.8`（`tower.rs:1071-1075`）。
- P21（`21-plugins-and-pause.md:86-119,127-268`）：`AudioPlugin` 六触点收拢（`audio.rs:147-159`：message+记账资源+Startup+OnEnter+OnExit+Update，与教程 `91-104` 逐一对应，纯重构）、`PauseState{Running,Paused}` 默认 Running（`pause.rs:39-44`）、`toggle_pause` 三守卫 Escape/InGame/`GameFinished`（`pause.rs:58-79`，与教程 `158-198` 表一致）、覆盖层生成/回收（`pause.rs:82-112`）、三组玩法系统 `.and_eager(in_state(PauseState::Running))`（`main.rs:244,256,266`；0.20 无 `.and`，`and_eager` 为文档明示等价，`pause.rs:7-11` 锚点在档）、音频不进暂停门控（`audio.rs:154-157` 无门控 + `audio.rs:143-144` 注释，与教程 `273-278` 一致）。
- 偏离 6 条（README §十）：①直通道语义保留随 U4；②`PauseState` 落 `pause.rs`（工程无 `state.rs`）；③覆盖层不进 `GameEntity/ScreenUi`（`td.pause` InGame+`GameFinished` 双守卫防孤儿，`brp.rs:1255-1290`）；④合成占位音随 U5；⑤sink 环境注记随立项音频非门槛；⑥三验证面记账资源+同心双金圈为教程 `preview.iter()` 原样。全部采信，无行为变化。

**3. 断言面 — PASS（行为谓词，无恒真；指纹真值）**

- P18 86/86（`p18-assert-final.log`）：载荷逐字段对拍 `spawn_last==gold_last==emit`（tile[2,0]/tower_id 0/cost 100/world_pos[-320,-288]）、`world_pos` 独立算式对拍 `[-320,-288]`、金币扣费 `cost±2` 含被动收入、非法/不可放置 `-32602`/`queued:false` 三 rejected+计数冻结。`produced=0` 系点击生产者无鼠标注入，`td.events` 为发布路径（O6 口径延续），非缺陷。
- P19 86/86（`p19-assert.log`；run2 84/88 失败轮如实留档，根因为注册表装载序跨进程不同，脚本已改按 key 反查 — K3 落实）：hover 半径 192/kind hover、空/路径/越界格 ring null、预览圈可见/不可见双分支分别由 run2 与终轮覆盖。
- P20 72/72 ×2（`p20-assert-final.log`）：音乐 `Loop/0.3000000119`（容差 1e-5）、`music_sink=true` 环境如实记录、推进器 `projectile_audio≥1`、爆炸/激光触发、`last_volume 0.8/1.0` 载荷穿越、`sfx_spawned==l+r {s:5,l:1,r:4}` 账平、离场音乐回收 `music=0`。
- P21 120/120（`p21-assert.log`）：暂停 500ms 位置/血量/金币/elapsed 逐位冻结（`gold 203.984375→同值`、`elapsed 1.328125→同值`）、覆盖层恰 1、`GameState` 不动（正交）、恢复 `elapsed 1.328125→1.96875`+敌人续动、菜单外 set/toggle `-32602`。收官段含注册表/三关 digest/ENTITY_DIGEST/经济/弹道/选择 UI 全回归。
- 指纹链：`chains.log` 三关 GRID_DIGEST（level_01 `70282f80…737a07`=T047 基线；level_02 `3edf8dff…a4f6d`；level_03 `840298ef…d76f7`）+ ENTITY_DIGEST `1d40c7b7…890b23`=T048 基线逐位一致（`p21-assert.log:72-73` 独立复算行同值）。

**4. U1–U6 裁决（对照 `g1-lead.log` §二）— 全部同意**

- U1 能力席 #10 激活→10/10（a 认可）：立项条款为上限 ≤10（`docs/m6-game-selection.md:25,40`），P20 属 26 部内行为等价，音频非门槛指验收面非"不做该部"。后续波次新开第 11 席即触发 owner 裁决，须台账注记 — 同意。
- U2 调用点 19>14（a 接受）：T048 G2 既定口径席位≠调用点；`audio.rs` 4 + `pause.rs` 3 为教程 21"一域一 plugin"行为面，`main.rs` 接线点恒 12（亲数 `main.rs:191,197,201,202,208,222,224,249,258,277,278,279` 共 12）— 同意。
- U3 `m6-game-selection.md` §五 td.* 面过期（a 收获批 Lead 统一更新）：决策文档写面归 Lead，worker 未越权 — 同意。
- U4 双路径维持（a）：直通道"返回即实体在场"为 T048 以来六批断言既定语义，教程消息化对象是点击链非 BRP 面；核心 `is_placeable`+`spawn_tower_at` 共用 — 同意。
- U5 合成占位音（a 接受）：recon 树仅 md 无音频文件，文件名/播放模式对齐 Step，CREDITS 注记在档 — 同意。
- U6 claim-lint mtime 回退（a 流程口径）：archive 树全文件 mtime 并列致回退取旧档（失败轮 `gate-claim-lint.log` 12/13 留痕），补产 `gate-doc-test-t050.log` 后 13/13；主仓有 `.git`/真实 mtime 不受影响 — 同意（复验命令见下）。

**5. 席位账 — PASS（如实）**

- 能力席 10/10：`m6-game-selection.md:50-63` §四对照 — 本卡触及 #4 塔（消息链重构）/#5 弹道（音频钩子）/#7 UI（目标圈/暂停覆盖层）/#8 td.* 面（+4）/#9 断言面，并激活 #10 预留席（音频）。≤10 上限未超，但已满席 — 后续波次无预留席，Lead 须记录。
- 调用点 19 = main.rs 12 + audio.rs 4 + pause.rs 3（亲数 `grep add_systems` 结果一致）；系统 46 = 39+4+3。T049"12/37"实为求和笔误（实 36），本卡以实数为基线并留痕（README §七/§十一 O4）— 采信，历史档不改。
- 方法面 14 自定义（10 基线 + events/gizmos/audio/pause），`discover.methods==37`（23 内置 + 14 自定义，`chains.log:14` 方法清单亲核），`td.` 前缀零 `game.*`（`p20-assert-final.log:16`）。

**6. 独立复算 — PASS（亲跑亲算，至少一组）**

- 复算 A · 暂停冻结算术（`awk` 亲算，取数 `p21-assert.log:114-115,125` + 常量 `economy.rs:26,29` + 接线 `main.rs:224-244`）：
  - 命令：`awk "BEGIN {print 203.984375-203.984375; print 1.96875-1.328125; print 203.984375-200.984375}"`
  - 输出：`0 / 0.640625 / 3`
  - 结论：冻结差为零（逐位冻结成立）；恢复推进 0.640625s＞0，与 `sleep(600)`+轮询/状态迁移开销同级（恢复成立）；暂停前金币累计 +3.0 = 1.0s 模拟时×3.0/s（`PASSIVE_INCOME_RATE=3.0`，`earn_passive_income` 挂 `FixedUpdate` 链受暂停门控），与 `waitGameUntilElapsed(1.2)`+采样轮询自洽；事件快照 `gold=200.984375` = 300−100+0.328125s×3，亦自洽。
- 复算 B · `td.audio` 结构面（`p20-assert-final.log:23-24,32-39` 原文）：`music_mode=Loop`、`vol=0.30000001192092896`（|v−0.3|＜1e-5）、`music_sink=true`、`projectile_audio≥1`、`last_volume 0.8/1.0`、`sfx 5=1+4`。与教程 Step 播放模式对表成立。
- 复算 C · 纪律计数（亲跑）：`#[expect` 属性恰 2 处（`tower.rs:1379,1528` 多行形态，G1 方法论注记成立）；BRP `with_method_main` 14 处（`brp.rs:161-174`）；绑定仅 `Ipv4Addr::LOCALHOST`（`brp.rs:176-177`），零 `0.0.0.0`；版本 `bevy="0.20"` 未动。

**7. 纪律面 — PASS**

BRP 仅 `127.0.0.1:15702`（`brp.rs:176-177`）；`td.` 前缀零 `game.*`；版本字面量未动（Cargo 三文件 SAME）；`#[expect(deprecated)]` 恰 2 处 P17 基线；CREDITS 音频节在档；往期证据未动（`docs` diff 仅新目录）；门禁档：check 0 错 0 警、workspace test REAL_EXIT=0（game 20/20、mcp-bridge 15/15、doctest 78/0/17）、claim-lint 13/13（`gate-claim-lint2.log:31-32`）、进程零残留（`process-check.log` grep_exit=1）。

## Critical / Important / Minor

- Critical：无。
- Important：无。
- Minor：
  1. `g1-lead.log` §二.2 写证据目录"44 件"，实物 45 件（`ls|wc -l`=45，README §二记 45 正确）— G1 笔误，不影响结论，收获批无需改 G1 档（留此注记即可）。
  2. T049 README"12/37"笔误本卡已留痕（§七/§十一 O4）— 历史档不改，本档为准，无行动项。

## 遗漏 / 疑问

无。`produced=0`、预览圈光标依赖、暂停期消息 2 帧清理（K2 未复现不入库）均已在 README §十一 O1–O3 / §九 K2 显式登记，口径可接受。

## 复验命令清单（树根执行，`CARGO_TARGET_DIR` 指主仓 target，游戏存活期间不起 cargo）

```
git -C C:/Users/Administrator/Desktop/ccc/bevy-ai-workflow rev-parse HEAD  # 期望 41ec1a2
git -C C:/Users/Administrator/Desktop/ccc/bevy-ai-workflow status --porcelain  # 期望空
cargo check --workspace -j 3  # REAL_EXIT=0，0错0警
cargo test --workspace -j 3 -- --test-threads=3  # game 20/20、mcp 15/15、doctest 78/0/17
cargo test --doc -p docs -j 3 -- --test-threads=3  # 先产 gate-doc-test 档，再跑 claim-lint（archive 树 mtime 口径）
cargo run -p claim-lint  # 13/13
node docs/evidence/t050-m6-wave4/assert-td.mjs --phase p21 --shot-pause ../docs/evidence/t050-m6-wave4/p21-shot-pause.png  # 120/120（需先起终版 exe，见 README §二流程）
```

收尾：`tasklist | findstr /I "game-td rustc cargo"` 应零命中。

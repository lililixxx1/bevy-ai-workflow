# G2 · 完整轮审核档（T051 M6 波次 5 P22-25 节点卡——26 部执行末波）

日期 2026-10-31 凌晨；审核者 plan-code-reviewer（**完整轮**——节点卡，含独立复算）；对象 = 树 `trees/m6/w5`（基线 10ab08c，G1 档亲核 HEAD=10ab08c）批次 T051 全量产出。本档为审核者报告原文归档（结论/阻塞/建议/复算原文如实转录；报告中「HEAD 10ab68c」系笔误，实为 10ab08c——G1 档 §一.1 为准）。

## 合并裁决

**✅ 通过** —— 末波四部行为等价成立、收官段 471/471 链自洽、席位零新增、纪律瑕疵（偏离 1）经核未污染证据。无阻塞项，不返工。

## 审核范围

- 基线：主仓 HEAD `10ab08c`（报告原文一处作 10ab68c 系笔误），`git status --porcelain` 零输出（主仓零触碰亲核）。
- 对象：工作树 `C:\Users\Administrator\Desktop\ccc\trees\m6\w5`（非 git 目录，`git archive` 纯净导出，无 `.git`）。
- 范围核验：`diff -rq --strip-trailing-cr` 主仓↔树（排除 `.git/.zcode/target/team-prompt` 后）恰 = **改 6**（`game-td/src/main.rs, pause.rs, level_select.rs, tower.rs, game_over.rs, brp.rs`）+ **增 1**（`game-td/src/input.rs`，405 行）+ **增证据目录** `docs/evidence/t051-m6-wave5/`（56 件）。与 worker 声明逐项一致；往期证据档零改动；`Cargo.toml / Cargo.lock / game-td/Cargo.toml` 三处 `--strip-trailing-cr` 下 SAME。
- 教程规格：`trees/m6-recon/tutorial-src/` 22/23/24/25 四部已全文通读；漂移底稿 `docs/evidence/t044-td-spike/w2/` 为对照口径（未重读全文，采信 G1 引用的锚点行号并抽核关键两处源码）。
- 覆盖率：改动面 7 文件已审 7 / 证据核心档（README、chains、g1-lead、assert-td.mjs 关键段、各 SUMMARY/REAL_EXIT 行）已审 / 覆盖率 100%（实现面）。

## 优点

- `game-td/src/pause.rs:72-131` —— `game_is_running` 签名与教程 22 Step 1 逐字同形（双 `Res<State>` + 解引用比较），`configure_sets` 元组形态与教程 22 Step 3 一致，三链 `.run_if`→`.in_set` 迁移在 `main.rs:254-313` 逐链可对上，无残留旧条件。
- `game-td/src/input.rs:257-291,347-357` —— 映射表与摇杆判别抽为纯函数 + `GAMEPAD_BINDINGS` 常量表与 `map_gamepad_button` 同源，`td.gamepad probe_*` 直接调同一函数，可测性设计到位。
- 失败轮三档（`p22-assert.log` 50/51、`p24-assert.log` 91/92、`p24-cross-p21.log` 119/120）全部留档并给出根因，K4/K5 教训由失败转成功对照（`p22-assert-run2.log` 51/51、`p24-cross-p21b.log` 120/120），fail-loud 而非 fail-silent。
- `game-td/src/brp.rs:1553-1830` —— `td.conditions` 直接 `run_system_once` 运行条件函数本身（不复述判断逻辑，避免双实现漂移）；`click` 通道写 `Pressed` + 运行真实消费者，断言检查的是下游效应（焦点/状态/地图），K2 用法正确。

## Critical（必须修复，阻塞合并）

无。七维度核验中未发现 ≥80 置信度的正确性/安全问题。

## Important（强烈建议修复）

无代码级 Important。本卡无返工需求；待办均为收获批文档动作（见"遗漏/疑问"与闸门五字段）。

## Minor（可选）

1. `game-td/src/input.rs:328-341` —— `read_gamepad_stick` 死区分支用 `continue`，而教程 25 Step 3 原文是 `return`。单/零手柄下行为一致；多手柄同时推杆时树实现按手柄逐个 `timer.tick`（教程只 tick 一次），重复率在 ≥2 摇杆同活跃时略快。单人单手柄游戏 + 无实机环境，属 benign 改进，**不改也行**；若改请以注释记偏离。
2. 取证小瑕疵：`p25-build.log` 仅保留绿档重跑，首轮 `REAL_EXIT=101`（os error 5）无原始行留档，仅 README §八 + G1 档文字记录。污染结论不受影响（见下），建议收获批复验清单加一条"失败轮另名留档、禁止覆盖"（与 U4 并列）。
3. `game-td/src/input.rs:176-184` —— `read_mouse_wheel` 取首条事件（P17 行为保真，教程未定多事件语义），注释已说明，此处仅确认采信。

## 七维度结论

1. **范围 PASS** —— 亲 `diff` 锁定恰 6 改 + 1 增 + 证据目录；Cargo 三处 SAME；bevy 版本字面量 `0.20` 未动。
2. **行为等价 PASS** —— P22 纯重构（条件真值 8 采样点由脚本独立复算式覆盖）；P23 导航算式/双通道/着色规则与教程逐项对上；**K1 等价落位成立**（亲核源码：`bevy_ui-0.20.0/src/widget/button.rs` 内 `pub(crate) struct DeprecatedButton` + `pub type Button = DeprecatedButton`，`#[require(Node, FocusPolicy::Block)]` + `#[require(InteractionTemp)]`；`focus.rs` 的 `NodeQuery.interaction: Option<&mut DeprecatedInteraction>` 证实 focus 系统只认 `Interaction`；树内无任何 `With<Button>`/`Query<…Button>` 查询残留——`Button` 组件本身不被任何逻辑依赖）；P24 七消费者重构 + clamp 不回绕 + 首事件滚轮语义保留，树内玩法系统已无原始输入直读（`input.rs` 外仅注释提及）；P25 绑定 9/9 与教程 Step 2 表一致，死区/主导轴/150ms 与 Step 3 一致（除 Minor-1 的 `continue` 改进）；偏离 2–8 全部采信。
3. **断言面 PASS** —— 无恒真谓词（P22 条件交叉复算、tick 冻结/推进/锁步三式、P23 颜色独立复算、P24/P25 独立复算均非自证）；**K2 未造成假绿**（click 断言检查焦点/状态/地图三重下游效应）；P24 经济 `gold 308.0625→109.421875, cost=200, passive=1.2188` 自洽（独立复算残差 0.14 < 0.2，见下）；**收官段 471 = 120+51+54+93+153 加和吻合**，且 K5 风险是 fail-loud 的（错序会在 `p24-cross-p21.log` 式 119/120 显形），`p25-assert-p21.log:101` 的 `spawned:1` PASS 证实收官进程内 p21 相位确为首跑。
4. **U1–U5 / 偏离对照：同意 G1 全部裁决** —— U1（G1 已更正：§五实况 14 个在档，待办为 14→18 收获批刷新）；U2（责任归属 Lead 派工单措辞，worker 按实质约束执行正确）；U3（K1/K2 入 SKILL v0.22、PIT-B 暂不新增——另注：56 件中未见 E0603 原始失败行，仅终版绿档，故 PIT-B 暂缓是稳妥的；K1 事实本身我已亲核源码成立）；U4（K5 新进程首跑入收获批清单）；U5（`gamepads=0` 注记 + 映射表/探测/零影响口径，立项非门槛）；**偏离 1 同意"如实记台账、不返工、再犯升级"**（未污染论证见下）。
5. **席位账 PASS（亲数）** —— `add_systems` 调用点 main 12 + audio 4 + pause 3 + input 3 = **22**；系统数 main 45（3+2+1+3+6+1+11+3+12+1+1+1）+ audio 4 + pause 3 + input 6 = **58**，与自报 22/58（含 P22→P25 增量链 46→49→51→56→58）逐项吻合。实质归类：P22 = 既有调度基础设施，P23 = #7 UI/HUD，P24/P25 = 输入翻译层复用既有席 + #8/#9 方法/断言面。**未新开第 11 席，10/10 维持**。
6. **独立复算 PASS（6 组亲跑亲算，详见下节）** —— 摇杆 8 向量 8/8、预览中心、P24 经济、discover 41 清单、expect 七站、add_systems 22。
7. **纪律面 PASS** —— BRP 仅 `127.0.0.1:15702`（`brp.rs:223` 显式回环，无 `0.0.0.0`）；`td.` 前缀零 `game.` 方法字面量（`grep game\.` 仅命中 `game_td/gamepad/game_is_running/GameplaySet/game_state/GameState/game_over` 合法标识）；门禁终档 REAL_EXIT 全 0（27 处；3 处 `=1` 均为留档失败轮，各有绿档对照）；`process-check.log` GREP_EXIT=1 零残留；往期证据零改动。**偏离 1 未污染证据**：(a) 2 次 check 不产出 exe；(b) 失败 build 卡在链接前 exe 替换（os error 5），无半成品产物，旧进程继续服务旧码；(c) 终版全部断言在后续新进程执行且 GRID/ENTITY digest 零漂移；(d) 已即时 taskkill + 零残留核验。唯一缺口是 101 原始行被重跑覆盖（Minor-2）。

## 独立复算原文（亲跑亲算）

```
命令：python -c（摇杆判别复刻 input.rs:276-291：length<0.3→None，否则主导轴）
(0.1,0.1)->None  (0.2,-0.2)->None  (0.5,0.1)->Right  (-0.5,0.1)->Left
(0.1,0.5)->Up  (0.1,-0.5)->Down  (0.4,-0.7)->Down  (-0.8,-0.2)->Left
——与 p25-assert.log:39-46 的 8 组 probe_stick 逐项一致，8/8 OK

命令：python -c（tile_to_world 复刻 enemy.rs:208-215；地图 15x10，TILE=64）
OX=-448.0  OY=-288.0；[6,7] -> [-64.0, 160.0]
——与 p24-assert-run2.log:35 的预览圈 center [-64,160] 逐位一致 OK

P24 经济（p24-assert-run2.log:53）：
gold 308.0625 -> 109.421875, cost=200, passive=1.2188
残差 (109.421875-308.0625)+200-1.2188 = 0.1406 < 0.2 → OK

discover（p25-run.log:4）：td.* 18 个亲数
screenshot, spawn_tower, wave, game, state, level, economy, combat, towers,
tower_select, events, gizmos, audio, pause, conditions, level_select, input, gamepad
= 18；18 + 23 内置 = 41，与 assert 首行 methods==41 一致 OK

expect 七站（grep '^#\[expect($' 跨文件亲数）：
level_select.rs:157/:337/:362 + brp.rs:1665/:1684 + tower.rs:1453/:1610 = 7
——全多行形态带 reason 串，5 新站 + 2 基线 OK

add_systems（grep -c）：main 12 + audio 4 + pause 3 + input 3 = 22 OK
```

## 遗漏 / 疑问（无需作者澄清，均已有归宿）

- U1 方法面 14→18 刷新、U4 复验清单（K5 新进程首跑 + 建议追加"失败轮另名留档"）——归收获批 Lead 执行，本卡不做（避免越出写范围，同意 G1(a) 口径）。
- U3 的 SKILL v0.22（K1→§6.14、K2 收录）——归收获批/SKILL 线，本卡 README §九在档已足够。
- E0603 原始失败行未在 56 件中留档——K1 事实已由源码亲核确立，不影响结论，仅支持"暂不入 PIT-B"。

## 复验命令清单（Lead 收获批/归档用，树根执行）

```
diff -rq --strip-trailing-cr <main-repo> <tree>   # 期望：仅 6 改 + input.rs + t051 证据目录
grep -c "add_systems(" game-td/src/main.rs game-td/src/audio.rs game-td/src/pause.rs game-td/src/input.rs  # 12/4/3/3
grep -rn -A3 "^#\[expect($" game-td/src/ | grep -c "reason"  # 7 站
grep -h "REAL_EXIT" docs/evidence/t051-m6-wave5/*.log | sort | uniq -c  # 终档全 0，仅 3 失败轮 =1
grep -n "SUMMARY" docs/evidence/t051-m6-wave5/p25-assert*.log docs/evidence/t051-m6-wave5/p2*-assert*.log  # 471 加和
```

## 闸门五字段

- `[结论]` 通过（✅ 可合并；G1 通过之 G2 完整轮确认，无返工项）
- `[已核对]` ①范围（6改+1增+证据目录，Cargo SAME×3）②行为等价（P22/P23/P24/P25 逐链 + K1 源码锚 + 偏离8条）③断言面（无恒真、K2 正确使用、P24 经济复算 OK、471 加和与 p21 首跑自洽）④U1–U5 同意 G1 全裁决⑤席位（22 调用点/58 系统，未新开席）⑥独立复算 6 组⑦纪律面（回环绑定/前缀/门禁/零残留）+ 偏离1未污染论证——退出码：终档 REAL_EXIT 全 0，3 处 =1 均为留档失败轮且各有绿档对照
- `[阻塞项]` 无
- `[最小修复指令]` 代码零修改；收获批做：①`docs/m6-game-selection.md` §五 14→18 刷新 ②复验清单追加 K5（p21 新进程首跑）+ 失败轮另名留档 ③SKILL v0.22（K1→§6.14、K2 收录）
- `[复验命令]` 见上节清单

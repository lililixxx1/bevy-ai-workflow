# G2 · 完整轮审核档（T052 M6 m6-all 收官卡——四件套终局判定）

日期 2026-10-31；审核者 = plan-code-reviewer **代位实例**（general-purpose 桥接执行同一审核协议）。**代位注记（Lead 落档）**：原 G2 派发两次死于 reviewer 代理模型路由 DNS 宕机（`gpt.lioi.pics` ENOTFOUND——dnspod/8.8.8.8/1.1.1.1 三解析器全灭，new-provider 宿主级故障），按「新上下文 + 只读 + 同套七维度协议 + ≥2 组独立复算」代位执行；模型身份与常例（muse-spark-1.3-contributor）不同，独立性以过程隔离保底，如实注记不追溯。以下为报告原文逐字归档。

---

# review-t052 · G2 完整轮审核报告（T052 M6 m6-all 收官卡）

审核者：plan-code-reviewer 代位实例（general-purpose 代位执行同一审核协议；代位事实由 Lead 落档头）。只读审核，未修改任何被审文件；全部结论基于亲核证据。

---

## [结论]

**通过**（0 Critical / 0 Important / 3 Minor）。四件套 4/4 成立，M6 收官判定**维持**。

## [已核对]（七维度逐行）

**D1 范围 — PASS**。亲跑 `diff -rq --strip-trailing-cr`（排除 .git/.zcode/target/team-prompt/.zcodeignore）主仓↔树：差异恰 3 处 = `docs/evidence/t052-m6-all/`（新增，53 件）+ `game/src/rpc/suites/ts06.rs` + `game-td/src/game_over.rs`；Cargo 三文件未出现在 diff = SAME。主仓 `git status --porcelain` 空、HEAD = `d6d7609`、工作树净——零触碰属实。与 `scope-check-t052b.log` 逐行一致。

**D2 U1 修复正确性 — PASS**。① diff 恰 2 处代码变更：签名 `Option<Res<GameResult>>`（game_over.rs:28）+ `let Some(result) = result else { info!(...) ; return }`（:29-32）；**Some 正路逐字等价**——`match *result` 以下 spawn 块与基线一字未动。② `Option<Res<T>>` 语义对 0.20 源码实读确认：`bevy_ecs-0.20.0/src/system/system_param.rs:1652`（`impl SystemParam for Option<T>`，get_param 委托后 `.ok()` → 资源缺席时系统仍运行拿到 None）；`Res<T>::get_param`（同文件 :706-709）缺席时 `Err("Resource does not exist")`——与 p0probe-run1.log:6 panic 文本逐字吻合（红档证伪链完整）。③ 「唯一读者」论断亲证：枚举全部 OnEnter/OnExit 系统（main.rs:218-321），其余各系统读的均为 Startup/init_resource 常驻资源（AvailableLevels/SelectedLevel/TowerRegistry/TowerAtlas/SelectedTowerType/AssetServer）或同链前序插入的关卡资源（`load_level_data`→链内 auto sync point）；`cleanup_level`/`cleanup_screen_ui` 仅 Commands+Query。`grep -n "Res<" game-td/src/*.rs` 全量 66 处过目，BRP 直连 `world.resource/resource_mut` 23 处逐一核对：必达态仅常驻资源，PlacedTowers/Gold 的直连（brp.rs:444/:480/:497/:1110）均处 `get_resource::<MapLayout>()` 早退门之后，而 MapLayout⟺Gold⟺PlacedTowers 同生共死（main.rs:339-346 同插 / :457-468 同删）——无可击穿路径，G1 论断成立。

**D3 件一判定面质量 — PASS**。`m6-all.mjs`（1400 行）通读：① 总判定由 `failed===0` 机器生成（:1397-1399），`check()` 写 `process.exitCode`（:43），无手填结论；② 期望值全部独立：脚本内双 TOML 解析器互证（:242-244 不符即 throw）、指纹常量注明 T047/T048/T049 冻结出处（:121-126）、未发现恒真谓词或读游戏侧值自证；③ SKIP=2 如实——唯一 skip() 调用点是溅射签名（:1023），l1/l3 两处（run4.log:101/:346），且火箭弹药/弹道面仍真断言（l1 fired=7/projectiles=1、l3 fired=7/projectiles=2，run4.log combat PASS 行）——可选项弃置非关键断言静默跳过；④ 负控价值保持：run1 红档（470/1/2 + p0probe 4/1/5）原样留档，p0probe 断言翻转为「存活+落地+result 保持 null+skip 标记」——若回退修复即翻红（进程死→aliveAfter=false→FAIL），红→绿对照链完整；⑤ 一致采样 ≤3 次全档披露且**不放宽阈值**（仍 1.0 金，:715），l3 cp1 首采 diff=25.14（恰 1 bounty）→ 重采 0.0938 的实况在档。

**D4 件二分类核对 — PASS**。对 `api-drift.md` 底稿亲数（awk 表格数据行）：**148 行**，26 部逐部行数与 w2-final-check §二表**全部一致**（P01=10…P26=9，Σ=148）；结论列机器分类复算 = **同形 136 / 改名 7 / 重写 2 / 无对应 1 / 显式无新 API 行 2**，恒等式 136+7+2+1+2=148 成立，与 §一总账逐项相等。抽查 ≥5 部对底稿原文：P26 九行实为 5 同形+重写 2（SpatialAudioBundle/嵌入资产）+无对应 1（DynamicScene）+改名 1（Observer）——与终局表逐字一致；P23 改名恰 2 行（Button/Interaction 三态）；P05/P16 各恰 1 行「（无新 API，继承前部）」显式行；P09 改名 1（Parent/Children）。无改判底稿迹象；「146+2」口径歧义由分母补全解释合理。

**D5 件四纪律核对 — PASS**。台账原文亲读：行 70-77 = T044/T046/T045/T047/T048/T049/T050/T051（T046 先于 T045 的行序说明属实，非断档）；返工列 8/8 全对（T044=2/T045=0/T046=1/T047=2/**T048=4**/T049=2/T050=5/T051=7——T048 行含 `\|实测−解析\|` 转义竖线，用转义感知解析复核，discipline-check §解析口径自述的坑真实且其处理正确）。`ls taskset/ts-*.md | wc -l` = **17** 亲数；台账总任务行 grep = **54**。claim-lint 同值（17/54）双源一致。

**D6 独立复算（3 组，全过）**——命令与数值原样：
① **经济体恒等式 5 点抽算**（从 m6-all-run4.log 原始数值亲算，公式 gold ≡ 300+3.0·t+25·kills−Σcost）：
- l1 cp1（log:64）：300+3.0×2.359375+25×0−300 = **7.078125**，实测 7.21875，残差 **+0.140625** ≤1.0（隐含 Σcost=300=3×rapid ✓）
- l1 cp2（log:74）：300+30.65625+125−450 = **5.65625**，实测 5.75，残差 **+0.09375**（Σcost=450=3×rapid+rocket ✓）
- l1 cp3（log:80）：300+52.6875+225−550 = **27.6875**，实测 27.78125，残差 **+0.09375**（Σcost=550 ✓）
- l2 cp3（log:205）：300+43.96875+250−500 = **93.96875**，实测 94.0625，残差 **+0.09375**（l2 无火箭，Σcost=500=5×rapid ✓）
- l3 cp2（log:321）：300+24.890625+125−300 = **149.890625**，实测 149.984375，残差 **+0.09375**（Σcost=300 ✓）
残差全部 = 3.0/64（FixedUpdate 被动 tick 0.046875）× 2~3 拍采样偏斜，量级与机制自洽；各点 kills=spawned−alive−leaked 内部一致。恒等式**成立**。
② **148 行分类复算**（见 D4）：136/7/2/1/2 全对。
③ **ts06 注锚实读**：`bevy_ecs-0.20.0/src/world/mod.rs:934` = `pub fn get_entity<F: WorldEntityFetch>(`、`:937` = `) -> Result<F::Ref<'_>, EntityNotSpawnedError> {` 逐字属实；旧锚 `:951` 实落在 `get_entity_mut` 文档块内（该 doc 块 :943 起）——0.19→0.20 漂移残留认定正确，收紧注记属实；diff 仅注释零代码。

**D7 纪律面 — PASS**。五闸 b 档亲读：check REAL_EXIT=0 且 warning 计数 **0**（grep -ci=0）；workspace test REAL_EXIT=0（game 20 + mcp-bridge 15 + doc 78-0-17/209.73s）；doc-test b = 78/0/17 REAL_EXIT=0；claim-lint b = 13/13 且「选中 b 档」行亲读在档（`gate-claim-lint-t052b.log` doctest-gate 节明文 `docs\evidence\t052-m6-all\gate-doc-test-t052b.log：78 passed / 0 failed / 17 ignored`）；process-check-t052b GREP_EXIT=1 零残留。a 档四闸另名未覆盖。**BRP 仅回环**：游戏日志 bind = `127.0.0.1:15702`，无 0.0.0.0。**td. 前缀零 game.***：run4 discover==41 PASS + 18 方法名枚举 + no game.* collision PASS。**cargo 互斥（本卡最敏感项，T051 前科）**：全时间线重建——首轮 build 03:05:12 → 首个游戏进程 03:06:58；a 档四闸 03:24:48–03:33:01 恰在 run2 游戏终了（03:23:18）与 run3 游戏启动（03:34:46）之间；**返工轮 build-run2 终于 03:50:02，run4 游戏进程 03:50:13 启动（间隔 11 秒）**，前一游戏进程（run3）03:37:13 已终了；b 档四闸 03:54:48 起在 p0probe-run2 游戏终了（03:54:04）之后——**无任何编译与游戏进程存活期重叠**，前科未重犯（置信度高：game log 首/末时间戳 + 文件 mtime 交叉）。**席位**：add_systems 调用点亲数 = main 12 + audio 4 + pause 3 + input 3 = **22**（其他源文件 0），与 T051 终态一致，零新席。**截图覆盖瑕疵**：README §7.3/:196 + §八/:210 + discipline-check §三/:66 三处披露；run3 的 5 条 md5 在 m6-all-run3.log 亲数在档，且 victory 屏 md5 `68ee729b…` run2/run3/run4 三轮同值（跨轮确定性独立佐证）——披露足量。

## [阻塞项]

无。

## [最小修复指令]

无阻塞修复。3 条 Minor（全部可选，收获批顺手即可，不阻塞收官）：

- **Minor-1（防御纵深）** `game-td/src/brp.rs:480`：`world.resource_mut::<Gold>()` 依赖「MapLayout 存在 ⟹ Gold 存在」的生命周期不变式（同插 main.rs:339-346 / 同删 :457-468）而非本地守卫；当前无可达击穿路径，但未来若解耦 Gold 生命周期即复萌 panic 面。建议镜像 :446 的 `get_resource` 门或注记该不变式。
- **Minor-2（常量耦合注记）** `docs/evidence/t052-m6-all/m6-all.mjs:709`：恒等式 bounty 硬编码 `25`；TOML bounty==25 由 defeat 相位（:1100-1102）独立断言兜底，若关卡 TOML 改 bounty 判定面会**响亮失败**而非假绿——失败方向安全，仅建议在常量旁注一行来源耦合说明。
- **Minor-3（收获批提醒，非本卡缺陷）**：台账 T052 行未登记（本卡「禁改」纪律使然，discipline-check §三已声明由主会话收获批登记）；收获时 LEDGER 54→55 须同步 claim-lint expect 表（T038 起规则），连同 G1 U4 的 m6-game-selection §五/§三 落字一并执行。

## [复验命令]

```
# 范围（D1）
diff -rq --strip-trailing-cr -x .git -x .zcode -x target -x team-prompt -x .zcodeignore <主仓> <树>
git -C <主仓> status --porcelain   # 期望空
# U1 语义源（D2）
sed -n '1652,1673p' ~/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/bevy_ecs-0.20.0/src/system/system_param.rs
# 148 行分类复算（D4/D6②）：按 '^## Part' 分段 + 表格数据行第 4 列（结论）计数，期望 136/7/2/1/2=148
# 经济抽算（D6①）：grep "economy ledger" docs/evidence/t052-m6-all/m6-all-run4.log，按 gold≡300+3.0t+25kills−Σcost 亲算
# ts06 锚（D6③）
sed -n '934p;937p;943p;951p' ~/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/bevy_ecs-0.20.0/src/world/mod.rs
# 门禁 b 档（D7）
grep -E "REAL_EXIT|汇总|b 档|warning" docs/evidence/t052-m6-all/gate-*-t052b.log process-check-t052b.log
# cargo 互斥（D7）：date -r 各 build/gate 档 + 各 *.game.log 首末时间戳交叉，期望零重叠
```

---

**置信度声明**：D1/D4/D5/D6 三组复算/门禁五项/席位计数 = 确证（亲跑亲数）；cargo 互斥 = 高置信（文件 mtime = 完成时刻 + game log 时间戳区间交叉，无反证）；「唯一读者」= 高置信（状态机 8 个 enter/exit 系统 + BRP 23 处直连资源访问全量过目，未发现第二可达击穿路径）。

**报告完**（供 Lead 逐字归档为 review-t052.md；除树/仓/registry 绝对路径外无环境私有信息）。

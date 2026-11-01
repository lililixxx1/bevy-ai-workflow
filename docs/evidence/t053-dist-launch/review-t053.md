# G2 · 完整轮审核档（T053 PIT-B-052 知识卡——分发面双击直启坑入库）

日期 2026-10-31；审核者 = plan-code-reviewer **代位实例**（general-purpose 桥接执行同一审核协议）。**代位注记（Lead 落档）**：原 reviewer 路由宿主 DNS 仍宕（`gpt.lioi.pics` NXDOMAIN——dnspod 与 8.8.8.8 双验，T052 同源故障延续），沿 T052 代位先例按「新上下文 + 只读 + 同套七维度协议 + 独立复算」派发；模型身份与常例不同，独立性以过程隔离保底，如实注记不追溯。以下为报告原文逐字归档（HTML 转义还原）。

---

# review-t053 · G2 完整轮审核报告（T053 PIT-B-052 知识卡）

审核者：plan-code-reviewer 代位实例（原路由宿主 DNS 宕机，由 Lead 按同一七维度协议桥接派发；只读审核，未修改任何仓库文件）

## [结论]

**通过**（0 Critical / 0 Important / 3 Minor）

变更面恰为申报清单、游戏代码零触碰；schema 逐字段合规、两条入库门禁口径成立；证据档逐行对档全吻合；计数联动独立复算四项全对；净新性成立。3 条 Minor 均为注记精度/台账补列级，收获批一句话可了，不阻塞入库。

## [已核对]（D1–D7 逐维度）

**D1 范围：PASS**
- `git status --porcelain` 亲跑输出恰为 4 修改 + 1 新目录：`M AGENTS.md`、`M assets-methodology/task-ledger.md`、`M bevy-dev/pitfalls.md`、`M tooling/claim-lint/src/main.rs`、`?? docs/evidence/t053-dist-launch/`（内 3 文件：README.md 3212B / direct-launch-no-assets.log 2812B / relaunch-with-assets.log 2587B）。
- **game-td/src 与 game/src 零出现**（游戏代码零改动属实）；HEAD 亲验 = `3ccb5300ff51e0ad3e1ba85739c1a37cb8b4a533`。
- `git diff` 四文件全部亲读：pitfalls.md +34/-1（自述头 51→52 + 末尾 052 条目）、台账 +1 行（T053）、claim-lint PIT_B 51→52 / LEDGER 55→56（注释链同步追加「T053 PIT-B-052 知识卡行 55→56」）、AGENTS.md +1 bullet。

**D2 schema 合规：PASS**
- 对照 `assets-methodology/pitfalls-schema.md` 条目字段表逐项：id `PIT-B-052` ✓ / 日期 2026-10-31 ✓ / 适用版本「bevy 0.20.0（…触发点 = game-td/src/tower.rs:340-341）」✓（Cargo.lock:446 `version = "0.20.0"` 亲验）/ 分型「错题（运行时行为…非编译面）」✓ / 通用性分级 bevy-specific + 注记 ✓ / 现象（text 围栏失败原文）✓ / 根因（双链分裂一段话）✓ / 修复 ✓ / 验证证据 ✓ / 标签 9 个 ✓——十字段无一缺。
- 门禁两条成立性评审：①「失败在标注版本上复现」——标注版本 bevy 0.20.0 / HEAD 3ccb530 双验，失败原文（panic 栈全文 + 5 条 Path not found）在 `direct-launch-no-assets.log` 原始在档，成立无夸大（exit 101 以叙述行标注，与先例 PIT-B-051「（进程退出码 101…）」同型口径）；②「修复过编译与运行验证」——零代码方案的编译面 = 被验二进制即 3ccb530 构建本身（该构建在仓门禁中已过编译），README §三明示这一等价逻辑而非含糊宣称，运行验证档在案，口径成立、无夸大。
- `text` 围栏用法：符合 pitfalls.md 头部 2026-09-28「非代码载体反例」增补口径（运行时行为观察类坑、无编译面可断言、text 围栏收录原文并锚定证据文件）；与先例 PIT-B-051 逐段同构（分型行注明 + 现象段括号注记证据档来源 + 无「最小复现」编译段——先例同款偏离且头部口径已覆盖）。字段顺序与段落结构与 051 完全一致。

**D3 事实核证（逐行对档）：PASS（含 Minor-1）**
- ① direct-launch-no-assets.log：`thread 'Compute Task Pool (2)' (23540) panicked at game-td\src\tower.rs:341:33: Failed to read assets/towers.toml: 系统找不到指定的路径。 (os error 3)`（第 9-10 行，grep -c panicked = 1）；bevy_asset `Path not found` **5 条**（第 12-16 行，grep 亲计）全为 `target\release\assets\...` 路径——≥2 要求满足。
- ② relaunch-with-assets.log：第 8 行 `tower registry loaded: 3 towers (["rocket", "rapid", "big_rocket"])`；grep -ciE "panic|ERROR" = **0**；`spawn schedule built: 10 events (2 waves, 4 enemy types)` 与 `No windows are open, exiting → Closing window 67v0` 在档（第 9/14-15 行）。
- ③ game-td/src/tower.rs 亲读：`:70` `pub const TOWERS_FILE: &str = "assets/towers.toml";`（注释明言「相对进程 CWD」）；`:338-344` `load_tower_registry` 内 `std::fs::read_to_string(TOWERS_FILE).unwrap_or_else(|e| panic!("Failed to read {TOWERS_FILE}: {e}"))`——`panic!` 宏恰在 **341 行**，列 33 与日志 panic 位置逐字吻合。
- ④ AGENTS.md T053 bullet / 台账 T053 行所述（exit 101、3 塔载入、spawn 10 events、exit 0、零代码、101MB 注记）与两证据档及源码逐项对得上。
- ⑤ 带外补录口径：BRP `td.game` 应答与 exit 0 **确不在两档中**且 README §二第 20 行明确标注为「带外补录（[取证补录 2026-10-31]）」并给出双重佐证（后台任务 exit code 0 + tasklist 零残留）——如实。「同一二进制、仅 assets 差异」关键核对：两档 SystemInfo 逐字段相同（同 OS 同 kernel 同 CPU 同核数同内存）、AdapterInfo 逐字段相同（同卡同 vendor/device 同驱动同 backend）、BRP 127.0.0.1:15702 + 18 个 td.* 方法清单逐字一致、窗口 ID 同为 `game-td (67v0)`、时间戳紧邻顺承（direct 00:34:43 崩溃 → relaunch 00:35:37 重启，间隔约 53s）、差异面恰为 assets 存在性（5 条 not found → 0 条）——叙事自洽成立。
- **Minor-1**：README §二注记「原始探针档 relaunch-with-assets.log 止于 registry loaded 行」与归档档终态 15 行（含 spawn×3 / Game Over / Victory / Closing window）存在字面张力——该叙事可自洽解读（取证时刻快照 + 游戏进程 stdout 重定向持续追加 +「关闭发生在探针 shell 之外」），且 §二 bullet 3/4 引用的档内行本身真实，方向是「从宽标注补录」而非漏标；但读者对照终态档会困惑，宜补一句「spawn/退出行为后续行由存活进程 stdout 持续追加」澄清。

**D4 计数联动（独立复算）：PASS（含 Minor-2）**
- ① `grep -c "^### PIT-B-" bevy-dev/pitfalls.md` = **53**；模板行 `### PIT-B-00X` 经 `grep -n "^### PIT-B-00X"` 亲验**恰 1 处**（:16）；53 − 1 = **52** ✓。
- ② 自述头（:10）「当前：52 条」且追加「2026-10-31 T053 分发面双击直启坑 052 证据 docs/evidence/t053-dist-launch/」✓。
- ③ claim-lint expect 表亲读：`PIT_B: usize = 52`（main.rs:31）、`LEDGER: usize = 56`（main.rs:38），LEDGER 注释链末尾同步「T053 PIT-B-052 知识卡行 55→56」✓。
- ④ 台账 `grep -c "^| T0"` = **56** ✓。
- ⑤ T053 行「一次通过 = 是 / 返工 = 0 / 无返工——复现/修复一次过（复现档 exit 101 + 修复档 exit 0 双留痕）」——与证据档实况（各一轮即中，无失败轮留档）一致 ✓。
- **Minor-2**：台账 T053 行「产出」列漏列 `AGENTS.md`（本轮实改含 AGENTS.md bullet；先例 T050/T051/T052 行产出列均列 AGENTS.md）——一词补列即了，无事实错误。

**D5 诚实性：PASS（含 Minor-3 建议）**
- ① 代码面备选双处（条目修复段 2 + README §三）均标注「**未实施不宣称验证**……若实施须另立卡过编译 + 运行验证」——如实 ✓。
- ② 「烘焙值为何未按链序命中未深究，以实测为准」注记恰当性：亲读 SKILL §6.18（:324-329）确认张力真实存在——三级链 ② `CARGO_MANIFEST_DIR` 烘焙值在无环境变量工况按链序应命中，而 T053 实测（且 T048 debug 直启同型）落 ③ exe 目录回退。条目**未掩盖**：明写「未深究」、以实测路径证据锚定事实、不虚构根因解释——这是对已知未解疑点的诚实处理而非搪塞。**Minor-3（建议）**：该疑点（§6.18 ② 分支的适用条件）经 T048/T053 两轮实测均未命中，值得后续挂 recheck-registry 或在 §6.18 加互引；非本卡义务。
- ③ 通用性分级 bevy-specific + 「『文件读取不得依赖 cwd』这半条换引擎仍成立，若他引擎复发应以方法论形态另记」——半条教训的跨引擎性被明示而非混入，处理诚实 ✓。
- ④ 「exe 101MB = 无 profile 调优常态（strip/LTO 可至 25-45MB 量级，仅对话留痕未落仓）」在 AGENTS bullet 与台账行双处框定「未落仓」——未验证量级被如实标注为对话留痕，不入知识资产，分寸恰当 ✓。

**D6 门禁预检（静态面）：PASS**
- 追加段（pitfalls.md:2763 起）围栏亲验：**仅 1 对 ```text、0 个 ```rust 围栏**——doctest 面零新增，计数应维持 78/0/17 基线。claim-lint 四闸按协议分工由主会话执行，本审核仅作静态判定。

**D7 知识价值与净新性：PASS**
- 既有 PIT-B 标题全量 grep（asset/cwd/fs/路径/分发/双击/目录关键词）无同题条目（命中仅 003/026/027/041 等不相关条目与 052 自身）。
- 对照 SKILL §6.18：其覆盖面 = bevy_asset 侧三级链（BRP 验证场景 cargo 外直启、缺省症状为**静默空图非报错**、修复 = 设 `BEVY_ASSET_ROOT`）；PIT-B-052 补的是 **std::fs 裸读侧**（不经 bevy_asset 链、cwd 相对、失败即 panic）+ **双链分裂面**（两套解析链互不相通、开发期双绿掩盖分发面）+ **分发工况**（双击直启、症状 = panic 101 崩溃、修复 = assets 随附发行包纪律）——工况/根因链/症状/修复方向四维皆异，且条目「同源先证」段明确引用 §6.18 为 bevy_asset 侧先证而非重复收录。净新成立。
- 教训一句话（标题）含完整检索面：`std::fs` / `cwd` / `bevy_asset` / `双击直启` / `panic 101`，另有 9 个标签——可检索、可复用 ✓。

## [阻塞项]

无。

## [最小修复指令]

三条 Minor 均为收获批一句话级，不阻塞入库：

1. **[M1] README §二补录注记澄清**：`docs/evidence/t053-dist-launch/README.md` 第 20 行「原始探针档 relaunch-with-assets.log 止于 registry loaded 行」后补一句（建议）：「后续 spawn/退出行由存活进程 stdout 重定向持续追加（取证时刻快照 ≠ 档终态）」，消除与档 15 行终态的字面张力。
2. **[M2] 台账产出列补列**：`assets-methodology/task-ledger.md` T053 行产出列补 `AGENTS.md`（对齐 T050–T052 先例口径）。
3. **[M3] 建议级，可并入下批**：烘焙值分支（SKILL §6.18 三级链 ②）在 T048/T053 两轮实测中均未命中的疑点，宜在 recheck-registry 或 §6.18 挂一条互引/待重核注记。

## [复验命令]

```
git status --porcelain && git rev-parse HEAD
git diff --stat
git diff -- bevy-dev/pitfalls.md
git diff -- AGENTS.md assets-methodology/task-ledger.md tooling/claim-lint/src/main.rs
ls -la docs/evidence/t053-dist-launch/
grep -c "^### PIT-B-" bevy-dev/pitfalls.md            # -> 53
grep -n "^### PIT-B-00X" bevy-dev/pitfalls.md          # -> 恰 1 处 (:16 模板行)
grep -c "^| T0" assets-methodology/task-ledger.md     # -> 56
grep -n "当前：52 条" bevy-dev/pitfalls.md              # -> :10
grep -n "PIT_B\|LEDGER" tooling/claim-lint/src/main.rs # -> PIT_B=52 (:31), LEDGER=56 (:38)
grep -c "panicked" docs/evidence/t053-dist-launch/direct-launch-no-assets.log      # -> 1
grep -c "Path not found" docs/evidence/t053-dist-launch/direct-launch-no-assets.log # -> 5
grep -ciE "panic|ERROR" docs/evidence/t053-dist-launch/relaunch-with-assets.log    # -> 0
grep -n "tower registry loaded" docs/evidence/t053-dist-launch/relaunch-with-assets.log # -> :8
sed -n '2763,2797p' bevy-dev/pitfalls.md | grep -n '```'   # -> 仅 1 对 text 围栏
awk 'NR>=2763' bevy-dev/pitfalls.md | grep -c '```rust'    # -> 0
grep -n '^name = "bevy"' -A1 Cargo.lock                     # -> 0.20.0
sed -n '324,339p' bevy-dev/SKILL.md                         # §6.18 全文对照
sed -n '2731,2762p' bevy-dev/pitfalls.md                    # 先例 PIT-B-051 格式对照
grep -n "^### PIT-B-" bevy-dev/pitfalls.md | grep -iE "asset|cwd|fs|路径|分发|双击|目录"  # 净新性
```

另亲读：`game-td/src/tower.rs` :60-79（TOWERS_FILE 常量）与 :332-351（load_tower_registry panic 结构）、`assets-methodology/pitfalls-schema.md` 全文、`docs/evidence/t053-dist-launch/` 三文件全文。

---

**Lead 裁决（2026-10-31，随档落字）**：M1/M2 收获批采纳（README §二 补录口径澄清 + 台账产出列补列）；M3（烘焙值分支两轮未命中疑点挂 recheck-registry/§6.18 互引）**挂账下批**——registry 为 doctest 自校受管面，联动须连同计数断言一并改，不值当在本卡顺手扩面。四闸 + 提交随后由主会话执行。

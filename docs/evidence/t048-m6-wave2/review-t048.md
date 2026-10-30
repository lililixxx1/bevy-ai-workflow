# G2 · 完整轮审核档（T048 M6 波次 2 P8-12 节点卡）

日期 2026-10-30（深夜）；审核者 plan-code-reviewer（**完整轮**——节点卡，含独立复算）；对象 = 树 `<tree>` 批次 T048 全量产出。本档为审核者报告原文归档（结论/阻塞/建议/复算原文如实转录）。

## 合并裁决

**✅ 通过** —— 范围零越界、行为等价成立、六断言档终档全绿且指纹链独立复算命中、门禁双绿、U1–U9 与三偏离维持 G1 裁决、无新增阻塞项。

## 审核范围

写面 9 文件已审 9 / 跳过 0 / 覆盖率 100%；另抽查 `map.rs`/`tiling.rs`/双 `Cargo.toml`/t047 档案（零触碰核证）。规则文件 review-protocol default + ts-js（断言脚本为 JS）。

## 优点

- `world_to_tile` 用 `round` 并注释 `floor` 反例（`tower.rs:115-128`），与教程 08 Step 3 逐字对应；`hovered_placeable_tile` 三段 `and_then` 链（`tower.rs:145-151`）同形；点击与 `td.spawn_tower` 共用 `is_placeable` + `spawn_tower_entities`（`brp.rs:301-346`），双通道无漂移。
- `attack_enemies` 快照→逐塔→旋转→`just_finished`→就地 despawn 单链（`tower.rs:309-406`）；`Without<Enemy>`/`Without<TowerTurret>` 显式互斥防 B0001；枪口焰父子同挂 `GameEntity` + `cleanup_level` 用 `queue_silenced(despawn())`（`main.rs:326-329`）消解递归 despawn 失败命令——教程 12 Step 6 的坑被显式处理。
- **U2 关键核证**：`spawned` 全仓仅三处——`enemy.rs:154` 自增、`brp.rs:359-360,383-384` 只读上报，**零行为分支读取**，「只读记账」裁决代码证实。
- 失败轮（p9-run1/run2、p12-run1）全部留档且 README §八 逐轮归因（如 `minHealth=Infinity` 系首采样前已打死），非藏拙。

## 阻塞 / Important

无（≥80 置信度的正确性/安全问题为零）。

## Minor

- M1：主仓工作区有未跟踪 `team-prompt/`（本地纪律目录，不入库预期态），方向为「主仓多、树少」，不影响基线比对。
- M2：证据 `gate-*.log`/`p*-build.log` 含本机绝对路径，沿 T047 环境痕迹口径。

## 维度 PASS/FAIL 一览

| # | 维度 | 结论 |
|---|---|---|
| 1 | 范围 | ✅（diff -qr 恰为声明写面；Cargo 双文件/map/tiling/t047 档案零输出） |
| 2 | 行为等价 P8-P12 | ✅（常量/语义逐部对教程原文：P8 round/HashSet 占用/草地校验；P9 192/34/0.5 + 单系统链；P10 逐波累时 + `total_cmp` 排序 + `while let` 弹到期；P11 BaseLives(5)/漏怪扣命/胜负双条件；P12 三态机/OnEnter 四段链/`run_if(InGame)` 门控/GameResult 保留 + GameFinished 回收） |
| 3 | 断言面真实性 | ✅（全类型路径、digest 口径、md5 链、discover 预注册相位；无恒真断言；U3 指纹口径三要素可复算） |
| 4 | U1-U9 与三偏离 | ✅（逐项复核代码/脚本，维持 G1 九裁决；U1 set 面 `brp.rs:464-502`；U7 type_path 全员 `game_td`） |
| 5 | 席位账 | ✅（`add_systems` 11 处是**调用点**计数；能力席位触及既有 #3/#4/#8/#9 未新开席、未触顶——选品 §四 口径） |
| 6 | 独立复算 + 门禁 | ✅（见下；check REAL_EXIT=0） |
| 7 | 纪律面 | ✅（绑定 `Ipv4Addr::LOCALHOST`〔`brp.rs:106`〕；`td.` 全前缀 10 档 PASS；版本字面量随 Cargo 零 diff 未动） |

P12 六塔覆盖窗验算：塔心 x=-320..320、单塔水平覆盖 ±143.1（√(192²−128²)）、并集 [-463,463] ⊇ 路径 [-448,448] ✓。

## 独立复算（原文）

1. **ENTITY_DIGEST 重算**（[2,0] 无害塔 + spawned=3）：`recomputed: 1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23` == expect → **MATCH**。
2. **波次表 + 检查点 + 时序**：events `[(0.0,100),(1.0,100),(2.0,100),(3.0,60),(4.0,60),(12.0,100),(13.2,100),(14.4,80),(15.6,80),(16.8,300)]` total 10；elapsed 2.5→3 / 4.2→5 / 5.0→5 / 12.3→6；soldier 896/192 = 4.67s、末位 runner 漏怪 4.0+896/320 = 6.8（defeat 档称 6.80s ✓）；victory 16.8s + 2~4s ≈ 19.64s 自洽。
3. **六断言档对账**：`35/35 · 30/30 · 35/35 · 41/41 · 37/37 · 83/83`；GRID_DIGEST `70282f80…37a07` 12 档同值（含 7 处零漂移 PASS 行）。

## 建议

- S1：m6-all 派工单沿用「能力席位」口径（`add_systems` 调用点 ≠ 席位），避免「≤10」误读。
- S2：U9/O1 家族（BRP-ready 立即执行、采样窗、长路径延迟）与 U4（victory 走完整状态机重测）在 m6-all 派工单确认落字，本波无欠账。

## 复验命令

`CARGO_TARGET_DIR=<shared> cargo check --workspace -j 3`；`grep -h SUMMARY docs/evidence/t048-m6-wave2/*-assert.log`

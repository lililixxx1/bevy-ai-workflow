# G2 · 完整轮审核档（T049 M6 波次 3 P13-17 节点卡）

日期 2026-10-30（深夜·三）；审核者 plan-code-reviewer（**完整轮**——节点卡，含独立复算）；对象 = 树 `<tree>` 批次 T049 全量产出。本档为审核者报告原文归档（结论/阻塞/建议/复算原文如实转录）。

## 合并裁决

**通过（可合并/可收获）**——T049 波次 3 五部行为等价成立，六断言档全绿、指纹链延续、门禁复跑绿，无阻塞项。

## 审核范围

11 个差异项已审 11 / 跳过 0 / 覆盖率 100%。改 7（main.rs:137-256 接线、brp.rs:92-106/:136-145/:664-909 五方法、tower.rs:244-298/:358-510/:547-1247 注册表与弹道链、enemy.rs:51-63/:92-115/:159-190 赏金链、level.rs:50-63 bounty 字段、level_select.rs:28-146 自动发现、level_01.toml:16-31 四敌型 bounty）；增 4（economy.rs:24-111、level_02/03.toml、towers.toml 全 59 行、证据目录全档）。map/tiling/game_over/Cargo 双文件零触碰独立核证（G1 正确）。

## 优点

- **P16 漂移实证点做实**：tower.rs:249-298 只用 serde/toml/std::fs + HashMap，无 bevy 新 API，教程 16「无新 API」成立；brp.rs:722-765 combat 快照 P16 起读注册表 rocket 条目，数据驱动闭环。
- **U2 零尺寸标记保留诚实且无行为分支**：tower.rs:57-70 Tower/TowerTurret 仅 spawn 挂载（:368/:381/:428/:442）与 BRP register_type（brp.rs:122-123），玩法判别全走 InstantShooter（tower.rs:666-672）/AmmoState（:778-786）；P16 实体指纹 `1d40c7b7…890b23` 与 T048 逐位一致即行为证据。
- **A' 通道干净**：全树 `#[expect(deprecated)]` 恰 2 处（tower.rs:1085-1088 setup_tower_dock、:1234-1237 handle_dock_slot_click），注教程原步与 0.20 现况（focus.rs:50/:65），check 0 警即证两处确踩弃用面。
- 取证纪律：p14/p17 真实编译失败轮（E0308/E0425+E0560+E0609 在档）如实留痕补录 101，非粉饰。

## Critical / Important

无（U1-U4 接受 G1 裁决：U1 E0425 在 p17-check.log:2 实证、显式导入 tower.rs:42 正确；U3 溅射单目标精确伤害 50/150 + 半径对表已断言、聚怪列 m6-all 可选项合理；U4 `td.tower_select` 为既定侧信道）。

## Minor

- M1：main.rs:209-210 注释「场上无 TowerTurret+Damage 实体」为 P15 前形态残留（P16 起判别器 = InstantShooter/AmmoState，Damage 已合并）——**收获批已改**（措辞对齐「场上无 InstantShooter 实体时 no-op；火箭系走弹道链不挂它」，纯注释零行为）。
- M2：README p13-p15 不可对终版复跑口径易误读——**收获批由 Lead 在台账 T049 行加澄清句**（回归由 p16 回归段 + p17 终版复跑覆盖）。

## 各维度 PASS/FAIL

1. 范围 ✅（11 项 = 声明写面，level_01 增量仅 4×bounty 行）。
2. 行为等价 ✅（P13 发现/选中/Startup 图集；P14 Gold300/3s/bounty/门禁/HUD；P15 三槽/2s/600/50/60；P16 raw→runtime、projectile 键解析、`def.damage.is_some()` 分派；P17 dock/滚轮/数字键/槽位点击/金边/换肤 z 判别等价、big_rocket 纯 TOML）。
3. 断言面 ✅（分关卡注册表三 digest 强断言零漂移；展开器双盲对拍；TowerTypeId 端到端 big→1/rapid→0；经济 passive 3.094/1.031s、首杀 net 25.00、三塔扣 300、第 4 塔拒；无恒真断言——`check(` 258 处均为行为谓词）。
4. U1-U4 与偏离 ✅（对照 g1-lead.log 逐项核；§十 6 条偏离行为零变化）。
5. 席位账 ✅（独立数得 main.rs 12 处 add_systems；系统 36 de-chained〔worker 口径 37 含链式展开差一，量级一致〕；触及能力席 #2-#9 未新开席、未触顶）。
6. 独立复算 + 纪律 ✅（BRP 仅 127.0.0.1 brp.rs:147；td. 全前缀无 game.；版本字面量未动；Cargo 零差异）。

## 独立复算（原文）

1. **独立展开器双盲**（toml-level-expand.mjs 三关实跑，EXIT=0）：level_01 `EXPECTED_PATH=45 EXPECTED_GRASS=105 TOTAL=150 EVENTS=10`；level_02 `60/90/150 EVENTS=12`；level_03 `72/78/150 EVENTS=15`——与 README §3.2 及断言档逐项 MATCH。
2. **towers.toml 对教程 15/17 常量表**：rapid cost=100/range 192/cooldown 0.5/damage 34.0；rocket cost=150/三槽 2.0s；projectile rocket damage 50/speed 600/splash 60；big_rocket cost 200/range 256/cooldown 1.5/单槽 4.0s、damage 150/speed 400/splash 100——逐项 MATCH；p15/p17 断言档同值。
3. **经济算术**：300−3×100=0（3 塔后 gold 4.55±2 含被动，p14-assert.log:26）；bounty 链 TOML 25→EnemyTypeDef→SpawnEvent→组件→attack/explode 双入账点均有 `health>0` 守卫；第 4 塔拒 `placed:false`（p14-assert.log:28）。
4. **门禁复跑**：`cargo check --workspace -j 3` → Finished + REAL_EXIT=0。

## 复验命令

`CARGO_TARGET_DIR=<shared> cargo check --workspace -j 3`；`grep -h SUMMARY docs/evidence/t049-m6-wave3/*-assert*.log`

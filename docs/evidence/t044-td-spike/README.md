# T044 M6 塔防 spike 证据总账（2026-10-29~10-30）

四条判定与结论：`verdict.md`（tilemap 0.20 兼容 ✓ / 卡模板纪律 ✓ / 最小 BRP 面 ✓ /
26 部外推 17~27h 中值 ≈22h → 立项批可开）。

## 目录

- `w1/` —— 实现卡（worker-1，树 t044/w1）：game-td P1→P3 + BRP 面。README 索引 V1–V8
  验收 + timing（P1 10m21s / P2 3m53s / P3 3m36s）+ 36 件原始档；G1 复验（`g1-w1.log`
  含像素级视觉复核与手工 world.query 抽查、`g1-w1-rulings.md` 六裁决〔R1 含 G2 前归因
  勘误〕）+ G2 完整轮（`g2-w1-report.md`：0 必改 + 5 建议 S1–S5 全落实）。
- `w2/` —— 漂移底稿卡（worker-2，树 t044/w2）：`api-drift.md` 26 部 148 行漂移表
  （136 同形 / 7 形态差 / 2 路线变更 / 1 语义变）+ `friction-summary.md` S23/M2/L1 分层
  外推输入 + `selfcheck.md`/`count.*` 计数自检 + G1/G2 双档。
- 收尾批门禁档（主仓根，本目录）：
  - `gate-check-workspace.log` —— check --workspace 0 警 REAL_EXIT=0；
  - `gate-test-workspace.log` —— 单测面全绿（game 20 + mcp-bridge 15）；尾部 doctest 段
    46/32/17 系 **12 线程并行编译 E0463 类环境假红**（PAT-M-011/PAT-B-021 签名：24×
    E0463 + rlib 缺失），按纪律由 `--test-threads=3` 单独门禁复跑排除（下一档）；
  - `gate-doc-test-t044.log` —— **doctest 门禁 78/0/17 REAL_EXIT=0**（197.61s，
    `-p docs --test-threads=3` 单套件口径——注：doctest 档必须单 crate 落档，全 workspace
    跑法末行是无 doctest 套件的 0/0/0，claim-lint 回退式按末行解析会假红，本批首跑实证）；
  - `claim-lint-t044-run1/run2.log` —— **13/13 双跑**（LEDGER 46→47 随台账 T044 行联动；
    run1 首跑红一次 = 上条口径问题，规范落档后双绿）；
  - `gate-banner-run.log` —— cli.rs 横幅勘误的运行时核验（`bevy 0.20 (workspace locked)`
    实证 + BRP 127.0.0.1:15702 回环监听 + 8 game.* 方法清单同档；taskkill 清杀零残留）。

## 收尾批另涉主仓文件

`game-td/`（工程与资产）、根 `Cargo.toml`/`Cargo.lock`（恰两行 / +2 包）、
`bevy-dev/SKILL.md` v0.17（§6.16 + §6.14 修正）、`docs/recheck-registry.md`（§6 表
15→16 节联动 + 自校断言）、`docs/src/lib.rs`（87→88 条）、`tooling/claim-lint/src/main.rs`
（LEDGER 47 + repo_root 运行时 cwd 优先〔共享 target 烘焙路径污染根治〕）、
`game/src/` 六文件 0.19 注锚清扫 8 处（余 25 处 → T045）、`team-prompt/PROJECT-APPENDIX.md`
（A 安全阀定量边界 + D 树 mtime 假红 / D 烘焙路径污染）、AGENTS.md 状态节。

## 清理注记

树 `trees/t044/w1`、`trees/t044/w2` 随本批关账删除（删树三条件满足：台账已回写 /
无 superseded 待核 / 收获提交入主仓）；`trees/m6-recon/`（教程 27 件 + tilesheet +
tilemap 0.20 源码）**保留至 M6 收官**。

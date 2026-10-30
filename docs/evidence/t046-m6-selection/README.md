# T046 M6 立项批证据目录

- 任务：M6 塔防立项批（选品三件套固化）——台账 T046 行；纯文档批零代码零围栏变更（T043 立项批同型）。
- 决策链：owner 方向拍板 2026-10-28（四定案）→ T044 spike 四判定全过（`../t044-td-spike/verdict.md`，26 部外推 17~27h 中值 ≈22h 为立项决策输入）→ owner 2026-10-30 获准全量立项 + grill 四题定案（扩展层 = 验证面扩展 / 收官四件套 / 体量周 ≤3+席位 ≤10+taskset ≤10 / T045 随首波次并行）——均为对话留痕，固化于 `docs/m6-game-selection.md` §七。
- 产出：`docs/m6-game-selection.md` 定本 v1.0 + 意向文档 v0.3.5（§4 M6 行 / §6 结构图 game-td/ + 版本锁 0.19→0.20 勘误〔T042 漏同步补正〕/ §8 M6 选品段 / 变更记录）+ AGENTS（M6 立项 bullet + 结构面）+ `docs/src/lib.rs` m6_game_selection 模块（纯决策文档零 doctest 增量，沿 m4/m5 先例）+ 台账 T046 行 + claim-lint LEDGER 47→48。

## 门禁留档

| 档 | 结果 |
|---|---|
| `gate-check-workspace.log` | cargo check --workspace 0 错 0 警，REAL_EXIT=0 |
| `gate-doc-test-t046.log` | doctest 78/0/17（计数与 T044 基线一致——备忘录零 API 示例零增量），204.75s，`-p docs --test-threads=3`，REAL_EXIT=0 |
| `claim-lint-t046-run1.log` / `run2.log` | 13/13 双跑 REAL_EXIT=0（LEDGER 47→48 联动生效） |

## 审核轮

- plan-code-reviewer 轻审轮（纯文档批先例，T043 立项批同型）：**有条件通过（1 必改）→ 已整改**——B1 = 台账 T044 历史行被追加编辑误触（行内重复片段，claim-lint 行首计数不告警的盲区，审核侧 diff 抓出）；整改 = 从 HEAD 整体恢复该行（三核验：T044 标记回 1/与 HEAD 逐字节一致/总数仍 48），S1/S2 随批落实（审核报告落档 `review-t046.md`；备忘录 CREDITS 落点明确为 `game-td/CREDITS.md` 首波次落地）。报告：`review-t046.md`。
- 整改后门禁复跑：claim-lint 13/13 双跑（`claim-lint-t046-postreview-run1/run2.log`）+ doctest 新鲜终档 78/0/17（`gate-doc-test-t046-final.log`，253.45s，S2 改述零围栏变更、沿 T028/T043 先例本可不重跑、仍以新鲜跑收口）。

## 执行面归后续批次

- M6 26 部执行·首波次（T047 起，T045 清扫卡随行并行）——W2 漂移表为对照底稿、`trees/m6-recon/` 教程源续用至 M6 收官。

# T043 立项批证据（B2 机器可读待重核清单 · 立项）

- **批次定位**：T043 立项批（纯文档、零代码、零围栏变更）——任务卡落地 `docs/pre-window-plan.md` §二 T043；范围按立项时点真值并修正 B2 原文两处旧口径（留痕）：**51 条 PIT-B + 21 条 PAT-B**（原文 20，T028 PAT-B-021 入库后为 21）+ **SKILL §6.1–6.15**（原文 –6.14，T041 新增 §6.15 后扩入）。清单本体的编写与逐条目行号级重核 = 后续执行批，不与本批捆绑。
- **受管面联动**：claim-lint `LEDGER 44→45`（台账 T043 立项批行）；其余计数不变（PIT_B 51 / PAT_B 21 / TASKSET 17 / GAME_METHODS 8 / SUITES 13 / BRP 31）。
- **门禁结论**：
  - `gate-check-t043-init.log` — `cargo check -p docs` 0 错 0 警，REAL_EXIT=0（立项时点）。
  - `claim-lint-t043-init-run1/run2.log` — 13/13 双跑绿（LEDGER 45=45；doctest-gate 断言经 S1 回退式挑选解析 T041 终档 77/0/17 REAL_EXIT=0）。
  - `gate-doc-test-t043-init.log` — 本批收尾新鲜终档（审核轮 N5：零围栏变更不触发 doctest 重跑义务，沿 T028 先例；仍新鲜落档自含）。
  - `claim-lint-t043-final.log` — 回填后终跑（§六/台账行/B2 指针编辑后复跑，确认文档编辑不触计数面）。
- **审核轮**：plan-code-reviewer 轻审轮（2026-10-29，纯文档批 = 核口径真值 / 卡可行性 / 表述一致性）——**通过（0 必改 + 3 建议 + 6 备注）**；记录见 `docs/pre-window-plan.md` §六 与台账 T043 行。建议处置：S1（断言形态先例精化：doc-conventions 自含断言 + pitfalls compile_fail 围栏）与 S2（计数同步口径 = 字面量镜像 + 随批联动、不引 docs→claim-lint crate 依赖）已随批落进卡面〔精化留痕〕；S3（门禁日志内机器路径）沿 T039/T042 环境痕迹口径保留，可选剥离归执行批。
- **执行批移交面**（本批不改、审核轮备注确认）：PAT-B-021 过程型条目的验证方式分型归类；门禁日志机器路径可选剥离；清单本体 doctest 自校实现（解析表体断言计数 == claim-lint expect 真值）。

## 执行批（2026-10-29 同日；清单本体交付）

- **交付**：`docs/recheck-registry.md`（87 条目三表 = 51 PIT-B + 21 PAT-B + SKILL §6.1–6.15 共 15，四字段机器可读）+ `docs/src/lib.rs` recheck_registry 接线 + claim-lint `DOCTEST (77,0,17)→(78,0,17)`、`LEDGER 45→46` 随批联动。
- **执行世系**：worker-1（PIT-B 片段，树 t043x/w1）∥ worker-2（PAT-B + §6 片段，树 t043x/w2）→ Lead G1 双批复验 → Lead 组装（片段逐字并入，G2 diff 复核 IDENTICAL）。
- **证据清单**：
  - `worker-selfchecks/w1-selfcheck.log` / `w2-selfcheck.log`——worker 侧自查（W2 含 C5b 修正留痕：CORRECTION 行 + sort -V 复跑 + SUMMARY checks=26 EQUAL=26）。
  - `g1-w1.log` / `g1-w2.log`——Lead G1 独立重跑（计数/分布/抽查/裁决记录）。
  - `gate-doc-test-t043-exec.log`——全量 doctest 终档 **78/0/17 REAL_EXIT=0**（192.04s；自校 fence 首跑路径 `../` 笔误一次即修，过滤单跑绿后全量）。
  - `claim-lint-t043-exec-run1.log`——13/13（doctest-gate 断言选中本批档 78/0/17）。
  - `claim-lint-t043-exec-negative-control.log`——DOCTEST expect 负控：临时 79 → FAIL REAL_EXIT=1 → 复原 78 → 13/13 REAL_EXIT=0。
  - `gate-check-t043-exec.log` / `claim-lint-t043-exec-final.log`——收尾终跑（回填后）。
- **G2 完整轮（plan-code-reviewer）**：**通过（0 必改 + 0 建议 + 1 Minor）**——独立复算全绿（计数/分布、片段 diff IDENTICAL、三项门禁复跑、6 条事实抽核、可公开态零命中）；记录见 `docs/pre-window-plan.md` §六 与台账 T043 执行批行。
- **登记项（附录 C 保真纪律：源缺陷只登记不修复）**：PAT-B-006 源文件（`bevy-dev/patterns/PAT-B-006-timed-self-exit.md`）头注 `bench.rs:58-107` 与做法行 `:58-108` 自不一致——核对表随头注取 `:58-107`（G2 Minor 提示，逐条重核执行时如动源文件可顺手统一）。
- **移交后续**：逐条目行号级重核执行归窗口空闲档/下一版本窗口（使用说明口径在 `docs/recheck-registry.md` 文末）。

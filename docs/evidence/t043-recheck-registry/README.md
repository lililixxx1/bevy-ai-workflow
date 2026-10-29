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

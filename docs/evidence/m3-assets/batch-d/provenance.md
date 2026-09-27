# batch-d 证据出处页（provenance）

本页说明本目录各日志的产生时序与性质，供独立核验（M3 Block D，2026-09-27；审核建议 1 落实）。

## 时序（本块内）

1. **probe-src-r1-snapshot/**——「凭记忆首试」源码快照（r1 轮，未查证 bevy 源码前书写）。
2. **probe-r1-check.log**（REAL_EXIT=101，4 错）——r1 首次 check，失败原文归档。
3. **probe-run-r1..r4.log**——r1 运行时四假设执行（bin 部分编译通过后跑）：R1 panic「Message not initialized」、R2 输出 (1, 0)、R3/R4 panic「StateTransition schedule is missing」。
4. **probe-r1b-check.log**（REAL_EXIT=101，3 错）——r1b 追加轮：三个「先验摇摆」探针（P11/P12/P13）首试失败归档。此时序在查证源码**之前**（r1/r1b 两轮失败原文均先归档、后查本地 `<reg>` 源码，符合 docs/m3-plan.md §3.1 诚实性红线）。
5. 查证源码 → r2 修正：lib.rs 注释留痕 P4/P6/P9/P11/P12/P13 失败形态，追加 r2 正解（p4c/p9c/p6c/p11c）。
6. **probe-r2-check.log**（REAL_EXIT=0）——r2 正解编译面。注：本日志曾两度重跑——①P12 注释勘误（初版「Message 是 derive 宏」与 E0782 原文「found a trait」不符，修正注释后重跑以保持证据—源码一致）；②R3c 注释勘误（app.rs:108-113 → :107-112，同因）。两次均为注释级改动，编译结果不变。
7. **probe-run-r1c/r2b/r3c/r4c.log**（全 REAL_EXIT=0）——r2 运行时正解四探针。
8. **probe-import-prelude-check.log**（REAL_EXIT=101，E0425 ×2）——**复现实验**，非首次失败 transcript：该失败真实发生于步骤 5 之后（r2 编写 main.rs import 时凭 prelude 惯例漏写显式路径，会话内即时构建输出）；因未单独归档，为满足「失败须归档」纪律以复现实验重跑留痕（临时注释正确 import → cargo check → 恢复）。日志内行号 165/179 与「注释替换而非删行」的方法自洽（删行会整体前移）。详见 PIT-B-019 验证证据段与 m3-plan §七 D 行的双处披露。
9. **probe-src-final/**——最终态源码快照（含全部留痕注释与 r2 正解）。
10. **gate-check-workspace.log / gate-doc-test.log**——主仓两道门禁（本块收尾；审核后条目修订再跑，以最终提交态为准）。

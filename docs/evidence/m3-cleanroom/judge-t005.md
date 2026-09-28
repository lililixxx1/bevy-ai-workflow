# T005 净室重跑·裁判判定（M3 Block H）

- 裁判：主会话（与执行者不同上下文）；日期 2026-09-28。
- 判定依据：`docs/evidence/m3-cleanroom/judge-protocol.md`（预注册于 ad4953e）+ `cleanroom/req-t005.md` 验收清单。
- 复核方式：执行者自报 `cleanroom/t005/report.md` 逐项对照归档日志，并对运行日志做**独立全量复算**（banner 计数、帧单调性、补丁边界、时延）。

## 一、验收清单裁判复核（全 PASS）

| 项 | 自报 | 裁判复核（归档证据 + 独立复算） | 裁定 |
|---|---|---|---|
| a 冒烟 crate 构建/运行 | PASS | `logs/r02-cargo-check-1.log` 末两行 `Finished \`dev\` profile ... in 1m 56s` + `REAL_EXIT=0`，logs/ 内无更早失败 check（r02 即判定首轮）；`logs/run/hotpatch-run.log` `Build completed successfully in 272.36s, launching app!` + banner `pid=10004` | PASS |
| b 两次热补丁生效 + 帧连续性 | PASS | 运行日志 `:230` `Hot-patching: src\main.rs took 1191ms`、`:903` `took 1186ms`（两次，无 rebuild/relaunch）。裁判独立全量解析（去 ANSI）：**1595 心跳行，严格单调 True，frame 30 → 33770**；banner `app started` 计数 **1**（裁判 grep 独立复核）；补丁 1 边界 6540(BEFORE)→6570(AFTER)，补丁 2 边界 26730→26740（+10 节奏即时切换）；BEFORE 218 行 / AFTER 1377 行 | PASS |
| c 生效时延量级 | ~1.40s / 1.36s | 编辑时间戳文件 `PATCH1_EDIT_DONE_UTC=2026-09-28T08:04:32.174` / `PATCH2_EDIT_DONE_UTC=...T08:07:29.670` vs 运行日志行为变更首行 `08:04:33.572521` / `08:07:31.029918` → 1.399s / 1.360s，与 dx 自报 1191/1186ms 互证（口径不同：前者含文件监视延迟） | PASS |
| d 收尾零残留 | PASS | `logs/residue-check-final.log` `grep_rc=1 (1 = no residue)`；TaskStop 连带进程树 + taskkill RC=128（无可杀） | PASS |

## 二、计数裁定

- 试验态 3 笔（补丁 1 编辑脚本被 assert 护栏在**写盘前**中止[文件零变更，dx 无感知]、轮询脚本 shell warning、一次被 ANSI 码干扰的只读 grep）：均不触及判定轮（首次 check / 首次运行），不计返工——执行者如实单列，裁判复核日志属实。
- **裁判返工计数：0（一次通过）。** 首次 `cargo check` REAL_EXIT=0 零警告 + 首次运行即全达验收 a–d。
- 小误差如实记（不影响结论）：report §二「全部 1422 行心跳…30 → 33770」系 parse-transition 中途快照（末行 frame=32040@08:08:17）与终局帧号的拼接表述；终局全量为 1595 行（终行 frame=33770@08:08:32.787），单调性经裁判全量复算成立，表述瑕疵由本判定补正。

## 三、独立性局限（重要——量化解释必载）

1. **注入资产含原任务结论**：`bevy-dev/SKILL.md` §6.5 与台账 T005 行本身即原任务结论回写（技术路线 + 两条编译教训），重跑独立性限于流程层面。执行者已主动披露，并将 14 项 API 事实全部按本地 registry 源码带行号重新核实（report §三）。该注入为预注册协议的设计（知识资产 = 注入面），但含义是：**T005 的 0 返工部分归因于资产直接含原任务教训——这正是「资产积累降低重做率」命题的预期效应，不能再称为无信息泄漏的独立复证**。
2. registry 漂移：bevy_ecs 要求 subsecond `"0.7.0-rc.0"`（caret），本地现解析 **0.7.10**（原任务时点 0.7.0-rc.0）。semver 兼容、冒烟行为无差异，复现环境非逐位同版。
3. 运行节奏自定差异：本次 `run_loop(8ms)` 固定心跳（原为默认无界循环），帧率数字与原证据不可直接对比（验收无此项）。

## 四、对照原任务（量化输入，Block H 报告汇总用）

原 T005（台账）：返工 1（LogPlugin 不在 prelude / Local 解引用——编译教训两条）。重跑：**0**。单任务下降 1 → 重跑返工 ≤1 达标。

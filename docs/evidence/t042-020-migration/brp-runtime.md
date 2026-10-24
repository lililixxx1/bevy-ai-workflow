# T042 BRP 运行时回归证据（0.19.1 → 0.20.0 升级窗口，2026-10-24）

- 目的：意向文档 §4 升级窗口完成判定第一件「任务测试集回归通过率恢复到升级前基线」的运行时实证（工作流三，AGENTS.md 硬约束 3）。
- 口径：release 构建（build-release.log REAL_EXIT=0、0 警告）、`--count 1000 --seed 20260926`、BRP `127.0.0.1:15702`、curl 驱动；进程启动/收杀留痕 game-run.log + taskkill 输出。

## 1. 通路与计数（rpc.discover）

- 命令：`curl -s -m 5 http://127.0.0.1:15702 -d '{"jsonrpc":"2.0","method":"rpc.discover","id":1}'` → CURL_EXIT=0。
- 判定：`result.info.version == "0.20.0"`；方法总数 **31**（= 23 内置 + 8 `game.*`，与 0.19 基线逐数一致——SKILL §6.2 口径，claim-lint BRP_TOTAL 断言维持）。
- `game.*` 全集：attack / end_turn / launch_level / move_unit / run_tests / screenshot / screenshot_log / snapshot。
- 原始档：`discover.json`。

## 2. 任务测试集回归（game.run_tests，逐套件）

- 命令形态：`curl -s -m 300 ... -d '{"jsonrpc":"2.0","method":"game.run_tests","params":{"suite":"ts-XX"},"id":3}'`；省 params 的错误透传原文（-32602 + 可用套件清单）见 run-tests.json。
- 结果（判定 = 响应 `failed==0 && passed==total`，逐行重算）：

| 套件 | 结果 | 断言 |
|---|---|---|
| ts-01 | PASS | 5/5 |
| ts-02 | PASS | 2/2 |
| ts-03 | PASS | 2/2 |
| ts-05 | PASS | 4/4 |
| ts-06 | PASS | 2/2 |
| ts-07 | PASS | 3/3 |
| ts-08 | PASS | 3/3 |
| ts-09 | PASS | 3/3 |
| ts-10 | PASS | 3/3 |
| ts-11 | PASS | 2/2 |
| ts-12 | PASS | 4/4 |
| ts-14 | PASS | 6/6 |
| ts-17 | PASS | 6/6 |

- **汇总：13/13 套件全绿，45/45 断言全过**（0.19 基线同口径恢复）。
- 原始档：`run-tests-all.jsonl`（13 行逐套件原始响应）。
- 过程留痕（如实）：首轮汇总脚本误按 `result.pass` 取值（实际键为 `passed/failed`）报 13 FAIL 一次——工具侧判定器键名未对齐断言面（PAT-M-010 同型教训），按正确键重判即全绿；未动游戏代码。

## 3. 截图管线（game.screenshot，§6.8 面）

- 响应：受理 `queued:true` + 异步完成提示（与 0.19 契约一致）；+4s 后文件在盘。
- 判定：PNG 魔数 `\x89PNG\r\n\x1a\n` ✓；IHDR **1600x900**（= 逻辑 1280x720 × DPI 1.25 物理分辨率，与 0.19 基线同口径）；大小 1,341,717 字节。
- 原始档：`shot0.png`。

## 4. 稳定 id 快照契约（game.snapshot，TS-17 面）

- 两次调用（间隔 1s）：`sim.wanderers` **1000** 个、全体键集一致 `{index, origin, velocity, phase}`、`index` 自 0 起严格递增（业务键序口径不变）；`sim.paused == false`；`battle == null`（未加载关分支）。
- 原始档：`snapshot-1.json` / `snapshot-2.json`（跨进程逐字节一致口径属 ts-17 套件域，进程内断言已含于 §2）。

## 5. 进程收杀

- `taskkill /F /IM game.exe` → 「成功: 已终止进程 PID 3544」；复检 tasklist 无残留。

## 6. 门禁总账（T042 批）

| 门禁 | 结果 | 档 |
|---|---|---|
| cargo check --workspace -j 3 | 0 错 0 警 REAL_EXIT=0（1m29s） | check-run1.log |
| cargo test --workspace（单测面） | game 单测 20/20 | test-run1.log（中途环境截杀，单测段已完整） |
| cargo test --doc -p docs | 首跑 33 败（12 线程内存耗尽环境性假红 31 + 真反转 2）→ 修 PIT-B-015 + `--test-threads=3` 复跑 **77/0/17 REAL_EXIT=0**（217.70s） | gate-doc-test-run1/run2.log |
| cargo build --release -p game | 0 警 REAL_EXIT=0 | build-release.log |
| BRP 运行时（本文件） | 套件 13/13、discover 31、截图/快照契约 ✓ | 本目录 |

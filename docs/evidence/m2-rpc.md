# M2 自研 RPC 证据：`game.run_tests` / `game.screenshot` / `game.screenshot_log`

- 日期：2026-09-26
- 对象：`game/src/rpc/`（自定义方法，经 `RemotePlugin::with_method_main` 注册）+
  `tooling/task-runner`（游戏外自动判定驱动）
- 目的：意向文档 §4 M2 验收③（`tooling/` 至少 `run_tests` + `screenshot` 可用，并以
  ≥ 1 个任务的验收清单跑通一次全自动判定）与验收②的全链留证（spawn → 改组件 →
  截图 → 读回断言——自研 `screenshot` 落地后，全链不再依赖 OS 级窗口截屏）。
- 原始文件：本目录 `m2-rpc/`（curl 响应 JSON、task-runner transcript、PNG 两张、进程与门禁日志）。
  **本文所有响应摘录与 raw 逐字一致**（内联节选处单独标注；PIT-M-004 纪律）。
- 本文档断言组 A–D 为首轮验证；断言组 E 为独立审核（plan-code-reviewer，有条件通过）
  后的复验轮（必改/建议项已落实，见台账 T019 返工②）。

## 启动口径

`./target/release/game.exe --count 500 --seed 20260926`（后台，日志 `/tmp/m2-game.log`；
release 构建、1280x720 逻辑分辨率窗口化、AutoVsync、种子 20260926——与 TS-01 相同基线）。
BRP 端点 `http://127.0.0.1:15702`。

## 断言组 A：curl 直调 `game.run_tests`（协议面）—— 4/4 PASS

命令（两次调用间隔 1s）：

```bash
curl -s --max-time 10 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"game.run_tests","id":101,"params":{"suite":"ts-01"}}'
# 1s 后同形请求 id=102（raw：m2-rpc/run1-call1.json / run1-call2.json）
```

- **A1 首调 5/5 通过**：`passed=5, failed=0, total=5`，逐条 PASS
  （wanderer_count_matches_config / config_metadata_consistent / sim_ticking /
  wanderers_have_velocity / initial_state_matches_seed_redraw——后者即「500 个实体初值与
  seed=20260926 + max_speed=3 重抽序列逐位一致」）。`snapshot.tick=1080`。
- **A2 tick 递增**：第二次调用 `snapshot.tick=1145 > 1080`（清单断言 3 后半，跨调用采样）。
- **A3 缺 params 错误分支**（raw：run1-nosuite.json，原文）：

```json
{"jsonrpc":"2.0","id":103,"error":{"code":-32602,"message":"缺 params（须为 {\"suite\": \"...\"}）；可用套件：ts-01"}}
```

- **A4 未知套件错误分支**（raw：run1-bogus.json，原文）：

```json
{"jsonrpc":"2.0","id":104,"error":{"code":-32602,"message":"未知套件 `bogus`；可用：ts-01"}}
```

- 附证：`rpc.discover` 方法清单收录全部三个自定义方法（raw：discover.json 中
  `game.run_tests` / `game.screenshot` / `game.screenshot_log` 均在列——自定义方法与
  内置方法同域可见）。

## 断言组 B：task-runner run 模式（M2 验收③「全自动判定」）—— PASS

```bash
./target/release/task-runner.exe run --suite ts-01 --gap-secs 1 \
  > docs/evidence/m2-rpc/taskrunner-run1.txt 2>&1
```

结果（raw：taskrunner-run1.txt）：两次调用均 5/5 PASS，完整性核对（自报 passed 与逐条
计数一致）通过，跨调用断言 `snapshot.tick2(2911) > tick1(2850)` PASS，**退出码 0**
（REAL_EXIT=0 行在文件尾）。TS-01 验收清单 4 条断言在无人工参与下全部自动判定——
两阶段判定的第二阶段（脚本）首次闭环。

## 断言组 C：task-runner chain 模式（M2 验收②全链）—— 6/6 PASS

```bash
./target/release/task-runner.exe chain --out-dir docs/evidence/m2-rpc/chain \
  > docs/evidence/m2-rpc/taskrunner-chain.txt 2>&1
```

逐链（raw：taskrunner-chain.txt，请求/响应逐字留痕）：

1. `world.spawn_entity`（Tagged + Velocity）→ `{"entity":4294966371}`；
2. `world.mutate_components`（`path:"tag"` → `"brp-chain-mutated"`）→ `result:null` 无 error；
3. `game.screenshot`（显式 path）→ 受理 `id=0`；
4. `world.get_components` 读回 →
   `{"components":{"game::sim::Tagged":{"tag":"brp-chain-mutated"}},"errors":{}}`（改写已生效）；
5. `game.screenshot_log` 轮询：首次 `status="pending"`（`frame_requested=3484`）→ 第二次
   `"captured"`（`frame_captured=3487`，`width=1600, height=900`）——**受理后 3 帧完成**；
   随后文件核验：`m2-chain-4294966371.png` 1082508 字节，PNG 魔数 + IHDR 尺寸 1600x900
   与日志回填一致；
6. `world.despawn_entity` 清理 → `result:null`。

**退出码 0**。全程经 BRP（无 OS 级截屏、无人工点击）。

## 断言组 D：`game.screenshot` 默认路径分支 —— PASS

无 `path` 参数受理（raw：defaultpath-accept.json / defaultpath-log.json）：返回
`{"id":1,"path":"screenshot-1.png","queued":true}`（节选，`note` 字段略；序号接续递增，不与已用 id 冲突）；
2s 后日志 `"captured"`（frame 4231→4234）。文件归档为
`m2-rpc/screenshot-default-path.png`（1086854 字节，`od` 核验首 8 字节
`89 50 4e 47 0d 0a 1a 0a`，IHDR 1600x900）。

## 断言组 E：审核后复验轮（必改/建议项落实 + 负例与类型错误分支）—— PASS

独立审核指出：TS-01 断言 1/2 的脚本侧判定是「配置自洽」口径（`count == entity_count`），
不绑定清单字面值 500/20260926——任意 `--count` 启动也会全绿。落实为 task-runner 新增
`--expect-count` / `--expect-seed` 字面值绑定（经 `world.get_resources` 读 `SimConfig`
核对），并补负例与类型错误分支：

```bash
./target/release/task-runner.exe run --suite ts-01 --gap-secs 1 \
  --expect-count 500 --expect-seed 20260926   # → REAL_EXIT=0（raw：taskrunner-run2.txt）
./target/release/task-runner.exe run --suite ts-01 --expect-seed 1 \
                                              # → REAL_EXIT=1（raw：taskrunner-run2-negative.txt）
```

- **正例**：字面值 PASS（`entity_count=500`、`seed=20260926`）+ 套件 5/5×2 + tick
  289→351 递增，REAL_EXIT=0；
- **负例（判别力证明）**：`--expect-seed 1` → 字面值 false → 结论失败 REAL_EXIT=1
  （套件与 tick 仍全绿，失败点精准落在字面值上）；
- **path 类型错误分支**：`{"path":123}` → `-32602`「path 须为字符串（收到 123）」
  （raw：run2-pathtype.json；原文件 UTF-8，终端显示乱码不影响内容）；
- 进程收尾 `taskkill //F //IM game.exe`（PID 8884），tasklist 复核无残留。

## 口径备注

- **物理 vs 逻辑分辨率**：窗口逻辑 1280x720，截图为**物理** 1600x900（本机 DPI 缩放
  1.25）——升级窗口/跨机器复跑时尺寸断言以物理分辨率为准。
- **`captured` 的承诺边界**：`screenshot_log` 的 `captured` 只承诺 `ScreenshotCaptured`
  事件已触发（官方 `save_to_disk` 的写盘 observer 已执行）；文件存在性/魔数核验归
  工具端（task-runner 已内置）——最强证据在文件本身。
- 进程收尾：`taskkill //F //IM game.exe`（Git Bash 下 `/F` 会被 MSYS 路径转换吃掉，
  需双斜杠转义或 `MSYS_NO_PATHCONV=1`——先例 T007 / `docs/evidence/ts-02-brp.md:96`，
  正式错题条目 PIT-M-005），退出无残留。

## 归档日志（m2-rpc/ 下）

- `run1-game.log` / `run2-game.log`：两轮游戏进程完整日志（`[CONFIG]` 启动口径、
  `[BRP] custom methods` 行、`[RPC] screenshot #N queued/captured` 时间线——默认命名
  与 +3 帧完成的非 BRP 旁证）；
- 门禁与构建：`cargo-check-r1-failed.log`（首轮 4 编译错 REAL_EXIT=101，台账 T019
  返工①的原始记录）、`cargo-check-r2.log`（修复后 REAL_EXIT=0）、
  `cargo-check-final.log`（审核修复后 REAL_EXIT=0）、`build-release.log`（REAL_EXIT=0）、
  `doc-test.log`（2 passed，REAL_EXIT=0）、`gate-check-r3.log`（首轮收尾门禁 REAL_EXIT=0）。

## M2 验收映射说明

- 验收③「`tooling/` 至少 `run_tests` + `screenshot` 可用」按 SKILL.md §3.6 边界拆为：
  **方法本体 `game/src/rpc/`**（BRP 自定义方法，编译进游戏进程——`RemotePlugin` 扩展
  通道的唯一形态）+ **`tooling/task-runner`**（游戏外驱动/判定，不链接 bevy 进游戏进程）。
- 验收②「全程无人工点击」按意向文档可核查定义执行；chain 依赖游戏进程先启动
  （可核查定义不含启动环节）。`launch_level` 未实现（③ 不要求）。
- M1 遗留的「其余 11 个任务清单转脚本断言 + 重跑校准」不在本次范围（③ 仅要求 ≥1 个），
  见 `docs/m1-acceptance.md` 两阶段判定衔接节。

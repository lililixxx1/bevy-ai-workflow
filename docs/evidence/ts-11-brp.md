# TS-11 断言证据：阶梯性能回归验证（BRP 原文 + 日志留痕）

- 任务：`assets-methodology/taskset/ts-11-perf-ladder.md`（4 条断言，含基线对照；零改码测量型）
- 判定者/日期：执行 agent，2026-09-26
- 结论：**4/4 断言首次采集运行全部通过；一次通过，返工 0；不改码**
- 被测对象：`game/` demo 原样（本任务零改码，`git status` 干净态下执行）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 50000
  --seed 20260926 --bench-secs 10`；实际执行 `cargo build --release -p game`（REAL_EXIT=0，
  `ts-11/cargo-build-release.exit`）后直接运行同一产物
  `./target/release/game.exe --count 50000 --seed 20260926 --bench-secs 10`（后台，日志重定向
  `ts-11/run1-game.log`）——与 `cargo run --release` 构建并运行的是同一二进制同一参数，口径等价
  （TS-01..10 同款注记）。横幅（`run1-game.log:1`）：
  `[CONFIG] game demo | count=50000 seed=20260926 max_speed=3 bench_secs=10 vsync=on(AutoVsync) | bevy 0.19 (workspace locked)`
  （`--max-speed` 未传 → 默认 3.0，与基线口径一致，`docs/fps-baseline.md` 末注）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`）
- 进程收尾：`--bench-secs 10` 到点自退出（`AppExit::Success`），`run1-game.exit` **REAL_EXIT=0**；
  `tasklist | grep game.exe` 无残留（无需 taskkill）。全程**单次运行**（无重跑、无挑结果）

## 测量口径与环境影响

- 口径五要素核对（`docs/fps-baseline.md`）：release 构建 ✓；1280×720 窗口化 AutoVsync（横幅
  `vsync=on(AutoVsync)`）✓；种子 20260926 ✓；warmup 2.0s（`[BENCH]` 行 `warmup_secs=2.0`）✓；
  N=50000 ✓。差异一处（清单明示，非偏差）：本任务 `--bench-secs 10`（测量窗 ≈8.0s）vs 基线采集
  `--bench-secs 12`（测量窗 ≈10.0s）——窗口长度差异，vsync 锁帧负载下不影响 avg_fps。
- 同机判定：`run1-game.log` SystemInfo/AdapterInfo 与基线完全一致——
  `cpu: "12th Gen Intel(R) Core(TM) i5-12490F"`、`NVIDIA GeForce RTX 3050`（Vulkan，驱动
  537.58，DiscreteGpu）、`kernel: "19044"`——即 `docs/fps-baseline.md` 采集机本身，
  断言 4 适用「同机」分支（阈值 = 基线 × 0.8）。
- 采集期间无其他重负载任务：门禁（check/build）在启动前已完成（时间戳在前），
  12.1s 运行期间仅 curl/轻量 node 轮询。

## 门禁（公共前置，零改码仍执行）

- `cargo check --workspace`：REAL_EXIT=0（增量 0.57s，`ts-11/cargo-check.exit`）
- `cargo build --release -p game`：REAL_EXIT=0（增量 0.52s，`ts-11/cargo-build-release.exit`；
  源码自 T015 提交后未动，二进制与源码一致）

## 采样执行过程（如实记录，含工具级偏差）

- 采样协议（测量侧设计，非断言）：bash 循环以 curl 每 0.25s 轮询 `world.get_resources`
  （`check-ready.mjs` 判定采样点：avg_fps>0 且 fps_1s>0 且 tick>=150，即测量窗已开且避开
  warmup 边界）；命中后保存该响应为 a1（断言 1/2 证据），1s 后再采 a2（清单外的补充复核）。
- **工具级偏差（非返工，如实记录）**：后台启动游戏与轮询循环两次工具调用之间存在调度间隙，
  轮询首访时游戏已在 game-t≈9.9s（临近 10s 自退出）——**首个轮询（i=1）即命中采样条件**，
  a1 即该响应（tick=584 已远超 150），落在 `[BENCH]` 打印前 ≈0.19s、进程退出前，恰为清单
  「进程退出前采样」的严格执行；整个运行仅此一次 BRP 读（无采样点挑选余地，也无挑选行为）。
- 清单外的 a2 补充采样在 1.44s 后发出时进程已退出，**响应为空**（`a2-simstats.raw.json`
  0 字节，curl 连接失败如实留痕）——a2 非清单断言（清单断言 2「请求要点：**同上**」，即与
  断言 1 同一 world.get_resources 请求），断言 2 按 a1 响应判定，不构成断言失败；
  tick 逐帧推进的旁证由日志 `[STATS]` 行承担（见断言 2）。
- 轮询留痕：`ts-11/a0-polls.tsv`（1 行：i=1, offset=78ms, READY, 解析值）、
  `a0-polls-raw.txt`（a1 响应原文）；时间戳 `ts-11/a-timing.txt`（a1 epoch=1790434491980，
  对应日志时钟 14:54:51.980Z，`[BENCH]` 行 14:54:52.167Z）。

## 断言 1：`avg_fps > 0` 且 `fps_1s > 0`（采集链路活）—— PASS

命令原文（轮询循环首个请求，i=1 → id=1101）：

```bash
curl -s --max-time 1 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":1101,"params":{"resource":"game::sim::SimStats"}}'
```

响应原文（全量，`ts-11/a1-simstats.raw.json`）：

```json
{"jsonrpc":"2.0","id":1101,"result":{"value":{"avg_fps":60.00309884925726,"elapsed_secs":9.873733390122652,"fps_1s":60.009967557234276,"frame_count":584,"tick":584}}}
```

判定：`avg_fps=60.003 > 0` 且 `fps_1s=60.010 > 0`（node 严格比较，`eval.mjs` 输出
`a1_avg_fps_pos=true` / `a1_fps_1s_pos=true`）——进程退出前 BRP 可读到非零帧率，采集链路活。
**通过**。

## 断言 2：`tick > 0`（50000 实体下模拟仍每帧推进）—— PASS

请求要点「同上」（清单原文）——与断言 1 同一 `world.get_resources` 响应：
`tick=584 > 0`（`eval.mjs` 输出 `a2_tick_pos=true`，来源标注 `a1(清单断言2请求=同上)`）。
**通过**。

「每帧推进」旁证（日志侧，非断言本体）：`run1-game.log` `[STATS]` 行 tick 单调推进
52 → 113 → 174 → 235 → 295 → 356 → 417 → 477 → 538（t=1.0s→9.1s，每秒 +60±1，
恰为 60fps 逐帧 +1 的速率），且 `frame_count=584 == tick=584`（BRP 读数）——50000 实体下
模拟与渲染同频推进，无掉帧卡停。

## 断言 3：`[BENCH]` 汇总行（n / avg_fps 量级 / warmup）—— PASS

`run1-game.log` 汇总行原文：

```text
[BENCH] n=50000 seed=20260926 bench_secs=10.0 warmup_secs=2.0 measure_secs=8.016 measure_frames=481 avg_fps=60.0
```

逐项判定（`eval.mjs`）：

- `n=50000` ✓（与启动参数一致）
- `warmup_secs=2.0` ✓（基线口径常量）
- `avg_fps=60.0` 与 #1 读值 60.00310 一致性：|60.0 − 60.00310| / 60.00310 =
  **0.0052%**，远小于 ±10%（`a3_within_10pct=true`）——**双源一致**（[BENCH] 汇总行与 BRP
  读到的 `SimStats.avg_fps` 为同一 Resource 字段的两个观察点，实测互证）

**通过**。

## 断言 4：基线对照（同机，avg_fps ≥ 基线 × 0.8）—— PASS

- 基线值：`docs/fps-baseline.md` vsync（AutoVsync）组 N=50000 → **60.0**（本次启动同为
  AutoVsync，同阶梯同组对照）
- 阈值：60.0 × 0.8 = **48.0**
- 实测：[BENCH] avg_fps = **60.0** ≥ 48.0 ✓（BRP 源 60.003 同判）；
  比值 = 60.0 / 60.0 = **1.000**（BRP 源 1.00005）
- 同机依据：见「测量口径与环境影响」节（CPU/GPU/驱动/内核版本与基线逐字一致）

**通过**（无回归；vsync 锁帧下与基线同样满帧于 60Hz）。

## 汇总与口径说明

- 结论：**4/4 断言首次执行全部通过；一次通过，返工 0；零改码**（`eval.mjs` 全
  checks=true，输出存 `ts-11/eval-out.txt` 口径即本文件各节）。
- 运行次数：**1 次**（run1 即判定运行，无任何重跑）。

## 与清单口径的偏差

- **断言未放宽**；清单 4 条全部按原文评估。
- 工具级偏差 1（非返工，如实记录）：轮询循环因工具调度间隙启动晚，首个轮询落点
  game-t≈9.87s（tick=584 即满足采样条件），a1 为整个运行唯一一次 BRP 读；清单外的 a2
  补充采样落在进程退出后、响应为空（0 字节文件保留）——断言 2 按清单「同上」由 a1 判定，
  「每帧推进」以 `[STATS]` tick 序列旁证。
- 口径注记（清单明示，非偏差）：`--bench-secs 10`（测量窗 8.016s）vs 基线采集窗 10.033s，
  窗口长度差异对 vsync 锁帧的 avg_fps 无影响。
- 基线组选择注记：`docs/fps-baseline.md` 建议升级窗口回归以 no-vsync 组为对照（区分度）；
  本清单启动方式明定 AutoVsync（无 `--no-vsync`），故按同口径对照 vsync 组 N=50000=60.0——
  清单自洽，按清单执行；该建议原样保留供升级窗口参考。
- 无新踩坑入库（零改码任务；工具调度间隙属本测量环境执行细节，已在上方如实记录，
  未构成失败）。环境侧注记：本机 GameViewer 虚拟显示器环境与基线采集时相同
  （`docs/fps-baseline.md` 环境备注），绝对值可比。

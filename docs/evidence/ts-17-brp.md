# TS-17《稳定 id 快照 game.snapshot》BRP 证据（窗口前置增强 T040 / A4）

- 执行：2026-09-29；判定 = taskset `ts-17-snapshot.md` 验收清单 6/6 PASS。
- 环境：Windows（本仓标准口径）；`cargo build --release -p game`（build-release.log，REAL_EXIT=0）；驱动 `snapshot-probe.js`（node，直连 `http://127.0.0.1:15702`）；进程 A/B 冷启动各一次（`--count 100 --seed 20260926`，后台任务托管 + 显式 taskkill + 残留核对，PAT-M-005）。
- 摘录逐字取自本目录 raw 日志（PAT-M-006）；跨进程比对走**原始响应体**（不经 JS 数值面，PIT-M-009 修复口径①的强化形态）。

## 逐条判定

### #1 rpc.discover 在列（30→31）

run-a-probe.log 原文：

```text
PASS: discover 含 game.snapshot（总 31 = 23 内置 + 8 game.*：game.attack,game.end_turn,game.launch_level,game.move_unit,game.run_tests,game.screenshot,game.screenshot_log,game.snapshot）
```

claim-lint `game-methods` 7→8、`brp-total` 30→31 联动（见 claim-lint-t040-run1.log）。

### #2 未加载关卡面 + params 负控

run-a-probe.log 原文：

```text
PASS: 未加载关卡 battle=null、sim.paused=false（wanderers=100）
PASS: params 负控 -32602（params 须为空对象或缺省（game.snapshot 无参数））
```

### #3 与 world.query / world.get_resources 逐字段等价

run-b-probe.log 原文（launch(1, seed=20260928) 后）：

```text
PASS: launch(1,20260928) 后逐字段等价：units 6 行、battle 四字段（turn=1/winner=-1/phase=0/rng_state=20260928）、paused=false、wanderers 100 行四元组（< 2^53 值域内严格相等，PIT-M-009）
```

等价路径独立于被测 handler：BRP 反射通路（world.query 三组件 / world.get_resources 两资源）另采真值，快照响应逐字段逐序比对（A/B 双进程各证一次）。

### #4 进程内套件 ts-17

run-b-probe.log 原文：

```text
PASS: 套件 ts-17：6/6 PASS
```

6 断言 = battle_null_when_unloaded / field_set_matches_t035（五级键集精确相等）/ snapshot_equivalent_to_world_query（独立 QueryState 交叉验证）/ business_key_ordering / relaunch_same_seed_same_snapshot（快照串逐位一致**且实体号集合已变化**——排除口径正证）/ world_restored_after_suite。

### #5 同 seed 跨进程两次启动快照逐字节一致

快照调用固定 JSON-RPC id=9001，原始响应体直落文件；snapshot-xproc-diff.log 原文：

```text
DIFF_EXIT=0
479388c3908c8a70939ea5f4dff6cdf6 *docs/evidence/ts-17/ts-17-snapshot-a.raw
479388c3908c8a70939ea5f4dff6cdf6 *docs/evidence/ts-17/ts-17-snapshot-b.raw
```

双侧各 16756 字节、md5 同值——实体号不在快照面（本断言无需「排除」：快照里根本没有）。

### #6 回归 + 日志面

run-b-probe.log 原文（13 套件 = 既有 12 + ts-17）：

```text
PASS: 套件 ts-01：5/5 PASS
PASS: 套件 ts-02：2/2 PASS
PASS: 套件 ts-03：2/2 PASS
PASS: 套件 ts-05：4/4 PASS
PASS: 套件 ts-06：2/2 PASS
PASS: 套件 ts-07：3/3 PASS
PASS: 套件 ts-08：3/3 PASS
PASS: 套件 ts-09：3/3 PASS
PASS: 套件 ts-10：3/3 PASS
PASS: 套件 ts-11：2/2 PASS
PASS: 套件 ts-12：4/4 PASS
PASS: 套件 ts-14：6/6 PASS
PASS: 套件 ts-17：6/6 PASS
```

panic-check.log 原文：双档计数行 `run-b-game.log:0` / `run-a-game.log:0` + `PANIC_GREP_EXIT=1`（grep 退出 1 = 零命中，0 panic）；run-a/b-residue.log：`RESIDUE=clean` 双侧。

## 过程偏差（如实记）

1. **首轮 cargo check 编译错误 ×1**：ts17.rs `world.resource::<T>().cloned()`（`cloned` 是 Option/Iterator 方法，裸引用须 `.clone()`）——即改即过（/tmp 侧无存档，返工计入台账 T040）。
2. **首轮驱动顺序假红（PIT-M-010 入库）**：套件排在 launch 之后 → ts-08 相位 A「radius == 90 默认」假红（实测 14 = 表现层棋盘视角收拢，present.rs）。复现档 run-b-probe-attempt1-pitm010.log（ts-08 failed=1，其余 12 套件全过——精确圈定进程史耦合而非代码回归）；修序（套件先跑）后 13/13 全绿。两层根因与修复口径见 `assets-methodology/pitfalls.md` PIT-M-010；ts-08 套件文档同步补进程史前提注。原子性正证 atomicity-check.log（自含 REAL_EXIT=0）：先跑 ts-17（内含 launch+还原）隔 1.5s 再跑 ts-08——相位 A 仍见 radius==90（|Δ|=0.000e0），套件内中间态对帧边界表现层不可见，「无净副作用」在表现层可观测量上同样成立。
3. **审核轮（独立代行审核，有条件通过 2 必改，全落实）**：B1 `claim-lint-t040-final` 档补跑归档（见 claim-lint-t040-final.log，13/13）；B2 证据保真——panic-check.log / build-release.log 重跑自含化（计数行/编译输出 + REAL_EXIT 入档）、brp.md 引文对齐档内实序；S1 atomicity-check.log 补命令前言与时序说明；S2 `"params": null` → -32602 实测注记入 pre-window-plan T041 卡（桥侧须省略 params 或显式 `{}`）；S3 sim.rs `Wanderer`/`Velocity` 初值不变量钉注（快照确定性基线的显式警示）。审核同时独立复现：第三进程冷启动快照原始体 md5 与 A/B 同值（16756 字节）、check/test/claim-lint 全绿复跑、S3 锚定恰 8 行、SKILL 行号三处命中。

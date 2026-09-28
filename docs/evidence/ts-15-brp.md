# TS-15 证据：战斗规则进程内回归套件

- 日期：2026-09-28。启动：`./target/release/game.exe --count 0 --seed 20260926`（release 构建 `gate-build-release.log` REAL_EXIT=0）。
- 驱动：`drive-ts15.js`（全量请求/响应落 `ts-15-transcript.jsonl`）。
- 判定：**5/5 PASS（运行时 4/4 退出码 0 + 日志面 #5：0 panic、无进程残留）**。

## 逐条判定

| # | 判定 | 关键证据（摘自 transcript） |
|---|---|---|
| 1 | PASS | `run_tests{"suite":"ts-14"}` → `total=6, passed=6, failed=0`，六断言全 P：`battle_state_initial / move_accept_and_reject / attack_kill_annihilate_and_lock / end_turn_twin_deterministic / reach_goal_wins_on_move / world_restored_after_suite` |
| 2 | PASS | 同进程二跑仍 `failed=0`（连跑自洽） |
| 3 | PASS | `launch(1,seed=7)` → 套件 `failed=0` → 套件后布阵 6 行业务键与加载后逐位一致、`BattleState` 回 `turn=1/winner=-1/rng_state=7`（快照恢复含 RNG 状态面——套件内战斗消耗被恢复覆盖） |
| 4 | PASS | `{"suite":"ts-99"}` → `-32602`，message 可用清单含 `ts-14`（`可用：ts-01, ts-02, ts-03, ts-05, …, ts-14`） |
| 5 | PASS | `run1-game.log` 无 `panicked`；taskkill 收尾无 game.exe 残留 |

## 过程偏差（如实记）

- **P1（门禁返工，计台账 T034 返工①②）**：`cargo check` 首轮 E0382 ×2（`saved.level.map(...)` 按值消费后又在 `is_none()` 借用）；修复改 `as_ref().map(...)` 后二轮 E0308 ×2（`Option<&LevelState> == Some(*l)` 形态错——比较对象是 `Option<&T>` 应 `Some(l)`）；三轮起 check 0 警告 + `cargo test -p game` 20/20。

## 门禁

- `cargo check --workspace`：r1/r2 REAL_EXIT=101（P4 两轮）→ r3 REAL_EXIT=0（0 警告）。
- `cargo test -p game`：r2 REAL_EXIT=0（20 passed / 0 failed）。
- `cargo build --release -p game`：REAL_EXIT=0。
- doctest 门禁 docs/ 无涉——T034 收尾未改门禁内文档，终版双门禁归 TS-16 回归任务统一留档。

## 文件清单

`drive-ts15.js`、`drive-ts15-out.txt`、`ts-15-transcript.jsonl`、`run1-game.log`、`run1-kill.log`、`gate-check-r1/r2/r3.log`、`gate-test-r1/r2.log`、`gate-build-release.log`。

# TS-16 证据：战役闭环 + 重放一致 + 核心回归

- 日期：2026-09-28。战役启动：`./target/release/game.exe --count 0 --seed 20260926` 冷启动 ×2（进程 A/B）；回归按各任务清单口径（`run-all-m1-regression.sh`）。
- 判定：**6/6 PASS**（零驱动侧偏差——战役、回归、复断言一次全过）。

## 逐条判定

| # | 判定 | 关键证据 |
|---|---|---|
| 1 | PASS | 战役闭环（campaign.js，seed=20260928，单进程）：**L1 歼灭 8 回合（3 存活）→ L2 歼灭 6 回合（4 存活 3）→ L3 占点 2 回合（3 存活、敌 6 存活）**；73 条指令 0 非预期 error；关卡间 launch 清场自洽。摘要原文：`{"seed":20260928,"levels":[{"level":1,...,"winner":0,"turns":8,...},{"level":2,...,"winner":0,"turns":6,...},{"level":3,"mode":"reach","winner":0,"turns":2,"playersLeft":3,"enemiesLeft":6,"ops":3}],"ops":73,"errors":0}` |
| 2 | PASS | 跨进程重放逐位一致：进程 A/B 各 78 步快照（业务键布阵 + HP + moved/attacked + winner/phase + **rng_state**，实体号排除——PIT-B-010 口径）`diff -q` 无输出（`ts-16-replay-a.jsonl` ≡ `ts-16-replay-b.jsonl`）——备忘录「同种子 + 同操作序列 → 状态序列逐位一致」由测试驱动承担的直接兑现 |
| 3 | PASS | M1 十二任务回归（run-all 复本，ROOT 重定向 `m1-regression/`）：**12× RUNNER_EXIT=0 + kill_clean**；ts-11 bench 双源 `BENCH_JUDGE_EXIT=0`（brp_avg=60.0039）；脚本聚合退出码 0 |
| 4 | PASS | 回归第 13 任务 ts-14（战斗套件，--count 0）：RUNNER_EXIT=0 kill_clean=0（套件两连调全过——T034 语义在回归序列中复证） |
| 5 | PASS | TS-13 复断言（新进程重采 raw + 存档 `assert-ts13.js` 原文复本）：**6/6 ALL PASS 退出码 0**（关卡加载面无回归；实体号例 4294966867→4294966853 属预期，断言按业务键） |
| 6 | PASS | 战役双进程 + 回归 13 任务 + 复断言进程 game.log 全部 0 panic；全部收尾无 game.exe 残留；`docs/evidence/m1-phase2/` 零改动（git status 干净——ROOT 重定向防覆盖，T021 stray 事故教训落地） |

## 过程偏差

- 无（本轮零偏差：战役首跑全胜未触发关卡数值调整；回归/复断言一次全绿）。

## 门禁

- 本任务零改码（驱动/脚本/文档新增不触 `cargo check` 面）；T035 收尾终版双门禁留档 `gate-check-final.log`（REAL_EXIT=0）与 `gate-doc-test-final.log`（77/0/17 REAL_EXIT=0）。

## 文件清单

`campaign.js`（战役驱动器）、`campaign-a/b-out.txt`（双跑摘要）、`ts-16-replay-a/b.jsonl`（78 步快照 ×2）、`run-a/b-game.log` + `run-a/b-kill.log`、`run-all-m1-regression.sh`（m1-phase2 复本：ROOT 重定向 + ts-14 追加，两处差异头注）+ `m1-regression/`（13 任务全量产物 + summary.txt）、`recheck-ts13.sh` + `ts-13-recheck/`（11 raw + 存档脚本复本 + assert-out.txt）+ `ts13-recheck-game.log`/`ts13-recheck-kill.log`、`run-all-stdout.log`、门禁 ×2。

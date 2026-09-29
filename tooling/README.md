# tooling/

自研件目录（意向文档 §6）：游戏专属 RPC 的工具侧（run_tests / screenshot / launch_level 等）。

- 本目录下的每个子目录（含 Cargo.toml）会被根 workspace 的 glob 成员 `tooling/*` 自动纳入；
  `brp-logs/`（无 Cargo.toml 的证据目录）已被根 workspace `exclude`。
- 架构边界（SKILL.md §3.6）：本目录是**游戏外进程**（不链接 bevy 进游戏进程）；
  方法本体在 `game/src/rpc/`（BRP 自定义方法），本目录提供驱动与判定。

## task-runner/

任务测试集的自动判定驱动（M2 验收③；M1 两阶段判定第二阶段的脚本判定执行器；
`brp-logs/` 是 T004 的通路证据、`hotpatch-smoke/` 是 §5.3 冒烟证据，均非代码）。

```bash
# 游戏运行中（BRP 127.0.0.1:15702）：
task-runner run --suite ts-01 [--gap-secs 1]   # run_tests 两连调 + tick 递增，退出码即判定
             #   [--expect-count N] [--expect-seed N] [--expect-max-speed F] [--expect-version S]
             #   字面值绑定经 world.get_resources；--expect-version 附带 list_resources 可见性实证
task-runner pause [--gap-secs 1]               # TS-04 暂停/恢复全时序（mutate 注入经 BRP，
                                               #   tick 冻结容差 2 帧 + Transform 全量逐位比对）
task-runner bench-judge --log X --brp-avg A --baseline-fps B --expect-count N
                                               # TS-11 的 [BENCH] 行解析 + 双源一致 + 基线阈值
task-runner chain [--out-dir brp-chain]         # M2 验收②全链：spawn→改组件→截图→读回断言→PNG 核验
```

stdout 即证据日志（请求/响应逐字留痕，重定向保存）。依赖面刻意最小：仅 serde_json
（已在 Cargo.lock 内）+ std TCP 极简 HTTP（见 `src/brp.rs`）。

- 首次运行验证（run/chain）：`docs/evidence/m2-rpc.md`（T019）。
- 全 12 任务脚本化重跑（run/pause/bench-judge + 字面值绑定）：`docs/evidence/m1-phase2.md`
  （T020，驱动脚本 `docs/evidence/m1-phase2/run-all.sh`——必须以后台任务运行，见
  PIT-M-007；套件侧约定「无净副作用」见 `game/src/rpc/suites/mod.rs` 模块文档）。

## claim-lint/

计数与宣称的机器门禁（窗口前置增强 T038 / A2，`docs/pre-window-plan.md`；动因 =
台账实测「计数/口径」系审核返工第一大来源 + 外部教训 SpawnForge 380 注册 vs 2 proven）。
两类断言：**真值断言** 8 条（PIT-B / PIT-M / PAT-B / taskset / 台账行 / `game.*` 方法 /
进程内套件 / BRP 总数——BRP 总数的 23 内置基数为常量输入，判别力等价于 `game.*` 方法条）
+ **运行时真值** 1 条（doctest 门禁计数，解析 mtime 最新 `gate-doc-test*.log`；日志缺失
时该条 SKIP 且退出码仍 0——「门禁跑过」是前提约定，负控/复核时此条不得为 SKIP）+
**一致性断言**（自述头「当前：N 条」×3、意向文档阈值 ×1）。**以上为首批受管面（T038）**，
扩展随任务按联动规则增补。期望值硬编码于 `src/main.rs` 的 `expect` 模块——**联动规则：
实况变化（新增条目/方法/套件/台账行）必须同步期望值**，否则收尾即红。行内
`<!-- claim-lint:ignore 理由 -->` 豁免历史性数字（豁免清单在审核面内）。零第三方依赖。

```bash
cargo run -p claim-lint   # 退出码 0=无 FAIL（doctest 日志缺失时该条 SKIP 亦 0，见上）
```

- 首跑/负控/复原/终跑四档证据：`docs/evidence/pre-window/`（claim-lint-run1.log /
  claim-lint-negative-control.log（自述头路负控）/ claim-lint-run2.log /
  claim-lint-run-final.log——终跑为台账 T038 行写入后 `expect::LEDGER` 37→38 联动口径；
  四份均自含 REAL_EXIT）。

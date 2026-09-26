# tooling/

自研件目录（意向文档 §6）：游戏专属 RPC 的工具侧（run_tests / screenshot / launch_level 等）。

- 本目录下的每个子目录（含 Cargo.toml）会被根 workspace 的 glob 成员 `tooling/*` 自动纳入；
  `brp-logs/`（无 Cargo.toml 的证据目录）已被根 workspace `exclude`。
- 架构边界（SKILL.md §3.6）：本目录是**游戏外进程**（不链接 bevy 进游戏进程）；
  方法本体在 `game/src/rpc/`（BRP 自定义方法），本目录提供驱动与判定。

## task-runner/

任务测试集的自动判定驱动（M2 验收③；`brp-logs/` 是 T004 的通路证据、
`hotpatch-smoke/` 是 §5.3 冒烟证据，均非代码）。

```bash
# 游戏运行中（BRP 127.0.0.1:15702）：
task-runner run --suite ts-01 [--gap-secs 1]   # run_tests 两连调 + tick 递增，退出码即判定
task-runner chain [--out-dir brp-chain]         # M2 验收②全链：spawn→改组件→截图→读回断言→PNG 核验
```

stdout 即证据日志（请求/响应逐字留痕，重定向保存）。依赖面刻意最小：仅 serde_json
（已在 Cargo.lock 内）+ std TCP 极简 HTTP（见 `src/brp.rs`）。
首次运行验证的证据：`docs/evidence/m2-rpc.md`。

# T041 MCP 薄桥 — 证据总账

任务：`tooling/mcp-bridge`（stdio MCP → BRP HTTP 薄转发）+ 仓根 `.mcp.json` 注册。终判：**driver 全链路 C1–C8 共 78 断言 run2 78/78 REAL_EXIT=0**（`mcp-drive-run2.log`）；check 0 警 / 单测 20/20（含桥 15）/ claim-lint 13/13（LEDGER 43→44）。台账判定行见 `assets-methodology/task-ledger.md` T041 实现行。

## 执行世系（补位先例表启用：两实例连续死亡 → Lead 接管）

- **worker-1（主体实现，2026-10-28）**：crate 全量（lib.rs 906 行 + main.rs + Cargo.toml）、15 单测、`.mcp.json`、`spec-quotes.md` 规范双源核实、`mcp-drive.mjs` driver 首版。产物经 Lead 逐行尽职调查合格采纳（含派工单 C4 矛盾的正确处置：非空探针桥内 -32602 + 直调对照，上报留痕）。
- **worker-1 之死（环境事故，非执行侧责任）**：2026-10-28 晚 commit 耗尽——6 rustc×7.3G + 3 link×7G 超 49.8G 上限（远程观测 20:07/21:46 两波，详见 `ops-incident-remediation.md`）。Lead 冷编授权低估 link 阶段峰值系授权侧责任；W1 按授权 `-j 3` 执行无责。
- **worker-2（承接收尾，未竟）**：doctest 门禁两跑均因会话/provider 中断截断（`gate-doc-test-attempt1-syntax.log` 为 --test-threads 参数位次错的重跑留痕、`gate-doc-test.log` 为未竟截断档——**两档均标废**，终判以 Lead 终树新鲜档为准）；单测/预检档完整（`gate-test-mcp-bridge.log` / `preflight-*.log`）。
- **Lead 接管（2026-10-28~29）**：driver `-j 3→-j 1` 补丁、release 冷编 `-j 1`（43m12s，峰值单 rustc 2.7G）、driver 两轮跑批、S2 注勘误、完成态文档批、G2 送审。

## 文件清单

| 类 | 文件 | 说明 |
|---|---|---|
| 验收终判 | `mcp-drive-run2.log` / `transcript.jsonl` / `judge.json` | run2 78/78 REAL_EXIT=0；transcript 每行 JSON、零机器绝对路径（C8 双断言） |
| 验收红档（返工留痕） | `mcp-drive-run1-3fails.log` / `mcp-drive-run1.log` / `transcript-run1.jsonl` / `c4-snapshot-run1.json` | run1 75/78：ts-11 前置等待缺失假红 ×2 + S2 注前提失实假红 ×1（详见下节） |
| driver 本体 | `mcp-drive.mjs` | C1–C8 全链路（含 run1→run2 两处修正的内嵌注释）；桥 spawn 用 `.mcp.json` 逐字命令自证 |
| 分面证据 | `c1-initialize.json` ~ `c7-negative-control.json` / `c3-run-tests.jsonl` / `mcp-shot.png` | 握手/白名单逐一比对/13 套件/三态与透传/只读四法/截图 1600x900/白名单拒绝+拒绝后游戏存活 |
| 规范核实 | `spec-quotes.md` | MCP 2025-06-18 规范原文摘录 + T004 transcript 双源对照表；Tool.name 点号合法性裁决 |
| 编译与门禁 | `build-release.log` / `gate-check.log` / `gate-test.log` / `gate-test-mcp-bridge.log` / `preflight-*.log` / `gate-check-final.log` / `claim-lint-t041-impl-run1.log` | release 冷编 -j 1（机器路径按可公开态替换 `<TREE>/`）；check/test 冷档（W1 期 -j 3）；桥单测档；预检档；G2 前终档双绿。**gate-test.log 内 doctest 片段 32 败系 `-j 3` 并发负载抖动（E0463，PIT-M-011 签名）作废**，单测段 20/20 有效；doctest 终判以 `gate-doc-test-t041-impl.log` + claim-lint doctest-gate 回退选择为准（G2 审核侧独立复跑 77/0/17 亦证）。可公开态注：C8 零路径断言仅约束 transcript/协议原文面；进程/门禁日志中的工具链绝对路径（共享 target 等）沿 T039 口径属环境痕迹、定性可公开 |
| 事故与运维 | `ops-incident-remediation.md` | commit 耗尽事故时间线与责任面、页文件两盘布局（C 16G NVMe + E 48G HDD 固定，重启生效）、-j 1 裁决、峰值观察器读数（含初版「观察器输出丢失」误判的勘误注） |
| 标废档 | `gate-doc-test.log` / `gate-doc-test-attempt1-syntax.log` | W2 未竟/参数位次错两档——终判以 `gate-doc-test-t041-impl.log`（终树新鲜档）为准 |
| 游戏进程 | `game-run.log` / `game-readiness.log` / `game-kill.log` / `mcp-bridge-stderr.log` | run2 批 spawn/就绪轮询/taskkill 清杀（无残留、15702 释放——C8 断言）；桥 stderr 空 = stdout 协议零污染佐证 |

## run1 三假红与修正（S2 注勘误）

1. **ts-11 前置等待缺失（×2 断言）**：套件头注要求「启动 ≥3s 后首次调用」（avg_fps 在 2s warmup 后才开始累计）；run1 的 C1→C3 链路过快，ts-11 在游戏年龄 1.35s 被调，avg_fps=0.000。修：driver 补 4s 前置等待（注释留痕）。fps_1s=55.7 正常佐证链路本身无恙。
2. **T040 审核轮 S2 注前提失实（×1 断言）**：S2 注称「`game.snapshot` 对 `"params": null` 报 -32602（serde 落 `Some(Null)`）」——实况为 bevy_remote 反序列化 `params.flatten()`（`lib.rs:1150`，**0.19.1 与 0.20.0 同款**）把线上 null 归一为缺省 `None`，与 params 键缺失不可区分，直调实测 200 成功。「serde 落 `Some(Null)`」恰是 flatten 阻断的步骤，原注系推断失实（T040 ts-17 实测档只测过缺省与「非空对象 → -32602」两档）。处置：driver C4 断言改期待实况、勘误同步 `docs/pre-window-plan.md` §二与 SKILL §6.15；**桥侧「省略 params」操作结论不受影响**，非空对象 -32602 拒绝面维持。

## 遗留项

- ZCode 工作区注册（`.zcode/config.json` 本地手动、按仓 gitignore 不入库）**新会话实载验证**未做——桥与 driver 全链路已绿，注册面留待下次会话实载确认。
- 页文件扩容（C 16G + E 48G 固定）**重启后生效**（机器例行 ~08:00 重启）；生效后 commit 上限 49.8→≈96G。
- 可选 `.cargo/config.toml` 全局 jobs 上限未做（-j 1/3 纪律现以派工附录条款承载）——是否落配置待 owner 拍板。

# T036 证据：M4 表现层（A 案系统席⑧）——非验收面佐证

- 日期：2026-09-28。启动：`./target/release/game.exe --count 0 --seed 20260926`。
- 定位声明：备忘录 §二 A 案口径——**表现层不作验收面**；本目录仅为「席⑧已落地且不伤规则面」的佐证。无 taskset 条目（M4 验收③「核心系统入测试集」不含表现席——设计如此，非缺漏）。
- 实现：`game/src/present.rs`（`PresentPlugin`：`sync_board`（`LevelState` change-detection 重建棋盘/目标格高亮/收拢相机）+ `sync_pieces`（阵亡 despawn/新单位补棋子/初始高度按入场 HP 定型——受伤不回缩，R1 审核 S4 口径修正）+ `place_pieces`（每帧随 `GridPos` 摆放）三系统链；规则实体零表现组件——分层只在表现侧挂靠）。

## 佐证结果

| 项 | 结果 | 证据 |
|---|---|---|
| 棋盘渲染 | **画面确认**（独立视觉核验四问全 YES：9×9 深浅棋盘 / 蓝 3 红 3 立方棋子 / 非全黑 / 斜俯视相机） | `board-l1.png`（1600×900，PNG 魔数核验 1359905 字节；`p3-shot-log.json` status=captured frame 544/541） |
| 规则面非回归 | `run_tests{"suite":"ts-14"}` 6/6 failed=0（表现系统与规则共存下战斗全绿） | `p4-suite.json` |
| 关卡加载链路 | `launch(1,seed=20260928)` 响应正常（present 依赖的 change-detection 触发源） | `p1-launch.json` |
| 进程面 | 0 panic、无 game.exe 残留 | `run1-game.log` / `run1-kill.log` |

## 过程偏差（如实记）

- **P1（门禁返工，计台账 T036 返工①②）**：`cargo check` 首轮 E0308 ×2——`Commands::get_entity` 在 bevy 0.19 返回 `Result<EntityCommands, InvalidEntityError>`（旧记忆的 `Option` 形态）；`if let Some` 改 `if let Ok`。二轮 3 警告（两个不需 mut 的 Query 参数 + `PresentMaterials.goal` 字段仅构造未读——目标格材质实为 `sync_board` 当次使用，资源结构收窄为棋子双色）；三轮 0 警告 + 单测 20/20 + release 全绿。
- **P2（证据目录整理）**：首名 `ts-17-present` 有任务编号暗示之嫌（本任务**无** taskset 条目，设计如此）——整理为 `present-layer/`（期间 PNG 被读取句柄占用短暂阻断 rename，cp+rm 完成）。

## 门禁

`gate-check-r3.log` REAL_EXIT=0（0 警告）、`gate-test-r1.log` 20/20、`gate-build-release.log` REAL_EXIT=0；doctest 面无涉（docs/ 未动）。T036 后终版双门禁归 T037 收口。

## 过程偏差（终审补记）

> **勘误（终审 B1，2026-09-28）**：T033 起本批门禁日志未自含 `REAL_EXIT` 行（bash 侧回显未落盘，T031/T032 口径回退）——成文所记退出码以当次 bash 回显为准；自 T037 收口起恢复自含，`docs/evidence/m4-acceptance/gate-check-final2.log` / `gate-doc-test-final2.log`（77/0/17）为自含复跑档，终审核立独立复跑同绿。

## 文件清单

> 勘误（R1 审核 S5）：`p2-shot-req.json` / `p3-shot-log.json` 内记录的截图路径为改名前行文（`docs/evidence/ts-17-present/board-l1.png`）——raw 不改写，文件实体位于本目录 `board-l1.png`。

`board-l1.png`、`p1..p4.json`（launch/截图请求/截图日志/套件）、`run1-game.log`、`run1-kill.log`、`gate-check-r1/r2/r3.log`、`gate-test-r1.log`、`gate-build-release.log`。

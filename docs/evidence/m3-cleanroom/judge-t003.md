# T003 净室重跑·裁判判定（M3 Block H）

- 裁判：主会话（与执行者不同上下文）；日期 2026-09-28。
- 判定依据：`docs/evidence/m3-cleanroom/judge-protocol.md`（预注册于 ad4953e）+ `cleanroom/req-t003.md` 验收清单。
- 复核方式：执行者自报 `cleanroom/t003/report.md` 逐项对照**归档日志**抽检（失败原文以归档日志为准，不以自报为准）。

## 一、验收清单裁判复核（全 PASS）

| 项 | 自报 | 裁判复核（归档证据引文） | 裁定 |
|---|---|---|---|
| a 首次 `cargo check` | 通过 | `logs/check-attempt1.log` 末两行 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2m 08s` + `REAL_EXIT=0`；logs/ 内无任何更早的失败 check 轮（attempt1 即判定首轮） | PASS |
| b release 构建 | 成功 | `logs/build-release-attempt1.log` `Finished \`release\` profile [optimized] target(s) in 8m 19s` REAL_EXIT=0 | PASS |
| c BRP 验证 | 9/9 PASS | `logs/brp1000/assert-motion-result.txt` 逐条 PASS ×9 + `REAL_EXIT=0`；解析式运动数学断言 6 条（A1–A6，≥5 满足）+ 计数/配置回读/tick 3 条；三样留档齐备（`tools/assert-motion.js` 断言原文 + `logs/brp1000/*.json` curl 取值 + 判定文件）。裁判独立复算抽检：`discover.json` 方法数 23；`count-q.json` result 数组两档精确 1000 / 50000 行；`config.json` 回读 `entity_count/seed` 与 CLI 字面一致（1000/20260926、50000/20260926）；`brpbench/stats-50000-vsync2.json` `fps_1s=59.986499010308805` vs `[BENCH] avg_fps=60.1` 偏差 0.19% | PASS |
| d 基线五要素 | 齐 | `baseline.md` §一 表五要素各有取值与证据指针（release / 1280×720 Windowed / vsync 双组 / i5-12490F+RTX 3050+537.58 / seed 20260926）；§二 [BENCH] 原文 5 行逐字在录（20s 复测轮带注）；四阶梯 60.1 / 82.7 / 60.1 / 139.8 | PASS |

**安全红线**：`src/brp.rs:19` `RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST)` —— 仅回环监听；文件头注释引 `http.rs:52/:60` 出处（DEFAULT_ADDR/PORT）。合规。
**残留核对**：`logs/residue-check-final.log` REAL_EXIT=1（tasklist 无 demo.exe 匹配）。无残留。

## 二、失败轮与计数裁定

- **#17**（`logs/brpbench/stats-50000-vsync.json` 0 字节 + `.exit` REAL_EXIT=7）：归档确认存在、未删改。需求书验收 d 仅要求「五要素齐备」——#16 轮后四阶梯 + 五要素已齐，BRP 双源一致性属清单外佐证深度（原始 T003 判定形态亦无此项），故**不计验收返工**。
- **#2**（死字段清理后复查）：首判已过后的卫生性修改，非返工。
- 常驻轮强杀 REAL_EXIT=1：预期收尾行为，非失败。

**裁判返工计数：0（验收口径）；备选口径「验收断言返工 0 + 基线补测 1」如实并列记录。** 与执行者自报一致。

## 三、局限与如实标注（沿用预注册协议）

1. 裁判无法穷证执行者的实际读取集；禁区遵行以自报 + 报告引用面核查（report API 出处仅本地 registry 源码与注入资产条目，未见 `game/`、`tooling/`、`docs/` 引用）。
2. 同一 AI 模型重跑，训练数据泄漏风险不可排除 → 结论限定为「流程净室」口径（协议 §局限）。
3. 断言自拟为需求书已声明的已知偏差（原任务断言集由其自建 `run_tests` 提供；重跑为脚本自判的第二阶段形态）。

## 四、对照原任务（量化输入，Block H 报告汇总用）

原 T003（台账）：返工 1（spawn_batch `'static` / PresentMode 不在 prelude / f32 不满足 Eq——三类编译错）。重跑：**0**。单任务下降 1 → 重跑返工 ≤1 达标。

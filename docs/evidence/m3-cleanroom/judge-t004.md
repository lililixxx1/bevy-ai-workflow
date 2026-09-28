# T004 净室重跑·裁判判定（M3 Block H）

- 裁判：主会话（与执行者不同上下文）；日期 2026-09-28。
- 判定依据：`docs/evidence/m3-cleanroom/judge-protocol.md`（预注册于 ad4953e）+ `cleanroom/req-t004.md` 验收清单。
- 复核方式：执行者自报 `cleanroom/t004/report.md` 逐项对照归档日志（193 文件）+ 裁判独立抽检/复算；**净新发现（mutate 崩溃语义）由裁判对本地 registry 源码行亲自复核**。

## 一、验收清单裁判复核（全 PASS）

| 项 | 自报 | 裁判复核（归档证据 + 独立复算） | 裁定 |
|---|---|---|---|
| a rpc.discover 原文+计数 | 23 方法 | 裁判独立解析 `logs/discover-poll-1.json`：`len(result.methods) == 23` ✓ | PASS |
| b ≥8 方法成功存证 | 16 个 | 四必测（query/spawn_entity/mutate_components/registry.schema）证据文件在档；抽检 `04-world-query-ok.json` result 数组 **1000 行**、`43-final-query-wanderer-count.json` **0 行**（终态净零副作用自洽） | PASS |
| c ≥3 组错误探针报错原文 | 6 组 7 码 | 裁判全量普查 `logs/*.json` error 码：**-32600/-32601/-32602×4/-23401×3/-23402×3/-23404/-23501×3**，7 种码 16 个探针全在档 | PASS |
| d 收尾杀净无残留 | 通过 | `logs/residue-check-final.log` `TASKKILL_EXIT=0` + `AFTER_EXIT=1`（tasklist 无匹配） | PASS |

安全红线：被验 demo 显式 `with_address(LOCALHOST)`（t003/src/brp.rs:19，T003 判定已核）；全程 curl 仅打 127.0.0.1:15702。合规。

## 二、净新发现的裁判源码级核实（重大——注入资产无记载）

**`world.mutate_components` 对不存在实体无优雅错误分支，panic 直接杀死整个游戏进程。**

裁判独立核实（三重）：

1. **请求原文**（`logs/15-probe-mutate-bad-entity.req`）：`{"jsonrpc":"2.0","method":"world.mutate_components","id":15,"params":{"entity":123456,"component":"demo::sim::Velocity","path":"linear","value":[1.0,0.0,0.0]}}`
2. **崩溃原文**（`logs/demo-run.log:120-128`）：`panicked at ...bevy_remote-0.19.1\src\builtin_methods.rs:1194:28: Entity not yet spawned: The entity with ID 4294843839v0 is not spawned` → `Encountered a panic in system bevy_remote::process_remote_requests` → 进程退出；客户端侧 `15-*.exit` REAL_EXIT=52（空响应，0 字节 json）+ `901-*.exit` REAL_EXIT=7（死后连接拒绝）。（进程退出码 101 为推断口径：Rust panic 默认值，归档未直接落进程退出码，以死后拒连互证——审核备注 N1。）
3. **源码行亲自复核**（裁判 sed 本地 registry；对照行号经审核轮双重复核修正）：`builtin_methods.rs:1194` = `.reflect_mut(world.entity_mut(entity))` —— 裸 `entity_mut`（panic 路径）；对照 insert/remove/despawn/reparent 均走 `get_entity_mut` → 温和 `-23401`（insert `:1129` / remove `:1304` / despawn `:1341` / reparent 挂父 `:1359` / 解除 `:1370`，函数定义 `:1811`），11/19/25 三个对照探针实测 -23401 佐证（reparent 源码同型，其探针为自父 -23404，未做坏实体探针）。mutate 是唯一裸 panic 路径。**勘误注（2026-09-28 审核轮）**：执行者报告 §三 对照行号 :1128/:1301-1303/:1342/:1356-1357 系载体差异（正确值如上）；净室执行者报告原文保真不改写，以本判定为准。

附加净新观察（裁判抽查在档）：remove_components 移除实体上不存在的已注册组件 → 幂等 `result:null`（13-*，非 -23403）；缺 `jsonrpc` 的 wire 错误码为 -32600；registry.schema 的 `with_crates` 过滤对无 crate 名类型不生效（38-* 反证 + 源码 :1681-1687）；schedule.graph 未知 label / trigger_event 未知事件 = -23501。

**入库处置**：该发现按知识资产纪律入库 PIT-B-051（bevy-dev/pitfalls.md），证据摘录落 `docs/evidence/m3-cleanroom/t004-crash-evidence.txt`（含请求原文、panic 原文、源码行引文、对照探针）。原任务 9 方法存证是否曾触发此语义不可考（净室禁读原证据），如实标注。

## 三、计数裁定

- 16 个方法的成功调用**全部首试通过**（无一例请求形态失败重试）；判定证据首轮采齐 → 首判全链跑通有存证。
- **R15 进程崩溃是故意错误探针的观察结果（验收项 c）**，其代价（重启 demo 一次，R00c）为探针语义的自然后果与继续执行的后勤动作，非任何判定内容的重做；崩溃/空响应/死亡复核三重留档未删改。
- **裁判返工计数：0（一次通过）。** 与执行者自报一致。备选口径如实并列：若计「计划外进程重启」则 +1 次（非验收断言失败）。

## 四、独立性局限（重要——量化解释必载）

本重跑为**知情重跑**：注入资产中台账 T004 行、PIT-B-004（由原任务失败直接转化）、SKILL §6.1/§6.2/§6.4（端点/23 方法/请求体形态）直接含原任务结论——原任务的返工 2 恰因 params 形态与统计对齐，而 params 形态教训已在注入资产中。故 **T004 的 0 返工是「资产携带原任务教训阻止其复发」的直接演示——这正是 M3④ 命题的预期效应，但不能称为无信息泄漏的独立复证**。mutate 崩溃等 6 项净新观察不受先验污染（不在任何注入材料中，本次首次触发并归档）。

## 五、对照原任务（量化输入，Block H 报告汇总用）

原 T004（台账）：返工 2（BRP params 形态 → PIT-B-004；统计口径对齐）。重跑：**0**。单任务下降 2 → 重跑返工 ≤1 达标。

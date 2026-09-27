# patterns/（Bevy 特定层模式库）

- 收录：通用性分级为 `bevy-specific` 的模式条目（正例）——**本项目编译通过且运行验证过的真实代码**（意向文档 §6；入库门禁见 [assets-methodology/pitfalls-schema.md](../../assets-methodology/pitfalls-schema.md)）。
- 条目形式（SKILL.md v0 定案）：**一条一文件**，命名 `PAT-B-XXX-<slug>.md`（XXX 三位自增，从 001 起；slug 用小写短横线英文），从 [_TEMPLATE.md](./_TEMPLATE.md) 复制起手。
- 入库步骤：
  1. 确认门禁已过：代码过 `cargo check` + 运行验证（BRP 闭环，SKILL.md §4.2）；
  2. 复制 `_TEMPLATE.md` → 改名 `PAT-B-XXX-slug.md` → 填完全部必填字段（模板内注释一并删除）；
  3. 在下方索引表加一行；
  4. 反例形态的教训不进本目录——走 `bevy-dev/pitfalls.md`。
- 当前：8 条（2026-09-26 起；2026-09-27 M3 批次一新增 002–005；批次二新增 006–008）。

## 索引

| id | 标题 | 适用版本 | 文件 | 验证证据（命令与结果） |
|---|---|---|---|---|
| PAT-B-001 | BRP 常驻最小集成（双插件+回环绑定+显式反射注册） | bevy 0.19.1 | [PAT-B-001-brp-resident-plugin.md](./PAT-B-001-brp-resident-plugin.md) | check REAL_EXIT=0；BRP 11/11 断言 PASS（2026-09-26） |
| PAT-B-002 | spawn_batch 批量生成——owned 值拷出 + move 闭包满足 'static | bevy 0.19.1 | [PAT-B-002-spawn-batch-owned-iterator.md](./PAT-B-002-spawn-batch-owned-iterator.md) | T003 check REAL_EXIT=0；BRP 计数三档实测（1000：brp-assert #3；500：ts-01；50000：ts-11 runner+--expect-count）（2026-09-26） |
| PAT-B-003 | Startup 链式定序——auto_insert_apply_deferred 自动 flush | bevy 0.19.1 | [PAT-B-003-startup-chain-auto-deferred.md](./PAT-B-003-startup-chain-auto-deferred.md) | T014 check REAL_EXIT=0；TS-09 恰 10 行 tag 断言 PASS + 套件复测（2026-09-26） |
| PAT-B-004 | 确定性模拟——自研 PRNG + 单一抽取落点 + (seed,index) 派生 | bevy 0.19.1 | [PAT-B-004-deterministic-prng-single-draw-point.md](./PAT-B-004-deterministic-prng-single-draw-point.md) | cargo test -p game 单测 4 项；TS-01 跨进程 500 向量逐位一致 mismatch=0（2026-09-26） |
| PAT-B-005 | 解析式运动——位置=时间解析函数，BRP 可精确复算 | bevy 0.19.1 | [PAT-B-005-analytic-motion-brp-verifiable.md](./PAT-B-005-analytic-motion-brp-verifiable.md) | 单测确定性 PASS；TS-07 独立复算 \|Δ\|≤0.1、套件化后逐位一致 0e0（2026-09-26/27） |
| PAT-B-006 | 定时自退出采集——BenchState 幂等闸门 + MessageWriter\<AppExit\> 条件写出 | bevy 0.19.1 | [PAT-B-006-timed-self-exit.md](./PAT-B-006-timed-self-exit.md) | TS-11 `--bench-secs` 自退出 REAL_EXIT=0 + `[BENCH]` 汇总行；常驻模式 [STATS]/BRP 可读（2026-09-26） |
| PAT-B-007 | 反射 Event + observer 资源翻转——BRP trigger_event 到生效零往返 | bevy 0.19.1 | [PAT-B-007-reflect-event-observer-flip.md](./PAT-B-007-reflect-event-observer-flip.md) | T015 发/收/读三步闭环；TS-04 pause 五步时序（2026-09-26/27） |
| PAT-B-008 | run_if 资源门控 + .chain() 定序——暂停语义一行调度表达 | bevy 0.19.1 | [PAT-B-008-runif-gate-chain.md](./PAT-B-008-runif-gate-chain.md) | TS-04 暂停后 tick 停滞/恢复递增 + BRP 旁路不受门控（2026-09-27） |

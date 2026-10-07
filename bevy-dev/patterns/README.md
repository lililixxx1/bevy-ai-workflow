# patterns/（Bevy 特定层模式库）

- 收录：通用性分级为 `bevy-specific` 的模式条目（正例）——**本项目编译通过且运行验证过的真实代码**（意向文档 §6；入库门禁见 [assets-methodology/pitfalls-schema.md](../../assets-methodology/pitfalls-schema.md)）。
- 条目形式（SKILL.md v0 定案）：**一条一文件**，命名 `PAT-B-XXX-<slug>.md`（XXX 三位自增，从 001 起；slug 用小写短横线英文），从 [_TEMPLATE.md](./_TEMPLATE.md) 复制起手。
- 入库步骤：
  1. 确认门禁已过：代码过 `cargo check` + 运行验证（BRP 闭环，SKILL.md §4.2）；
  2. 复制 `_TEMPLATE.md` → 改名 `PAT-B-XXX-slug.md` → 填完全部必填字段（模板内注释一并删除）；
  3. 在下方索引表加一行；
  4. 反例形态的教训不进本目录——走 `bevy-dev/pitfalls.md`。
- 当前：21 条（2026-09-26 起；2026-09-27 M3 批次一新增 002–005；批次二新增 006–008；批次三新增 009–010；2026-09-28 批次五新增 011–020；2026-10-07 跨仓回流批新增 021——过程型分型，万阵试金石仓留痕，门禁见 pitfalls-schema.md 过程型条款）。

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
| PAT-B-009 | BRP 自定义方法组装——with_method_main 链 + 错误码复用受理侧纪律 | bevy 0.19.1 | [PAT-B-009-brp-custom-method-assembly.md](./PAT-B-009-brp-custom-method-assembly.md) | M2 三方法 BRP 实测全通 + rpc.discover 收录（2026-09-26） |
| PAT-B-010 | 异步截图受理-轮询契约——spawn+observe 双 observer + 日志资源两段式 | bevy 0.19.1 | [PAT-B-010-async-screenshot-accept-poll.md](./PAT-B-010-async-screenshot-accept-poll.md) | M2 验收②全链 PNG 魔数+IHDR 断言、捕获 +3 帧口径（2026-09-26） |
| PAT-B-011 | 窗口口径配置——DefaultPlugins.set(WindowPlugin) + CLI 直通 PresentMode | bevy 0.19.1 | [PAT-B-011-window-config-baseline.md](./PAT-B-011-window-config-baseline.md) | 12 任务运行实证 + ts-11 vsync 双口径（2026-09-28） |
| PAT-B-012 | 轨道相机——Camera3d+looking_at spawn 与解析轨迹更新 | bevy 0.19.1 | [PAT-B-012-orbit-camera-analytic.md](./PAT-B-012-orbit-camera-analytic.md) | 单测恒等 + TS-08 BRP 距离断言 + 探针 dot=1（2026-09-28） |
| PAT-B-013 | PBR 实体群 spawn——Assets 系统参数 + 句柄复用 + spawn_batch | bevy 0.19.1 | [PAT-B-013-pbr-swarm-spawn.md](./PAT-B-013-pbr-swarm-spawn.md) | TS-01/02/11 计数与逐位一致断言（2026-09-28） |
| PAT-B-014 | BRP 可读写 Resource 反射形态——四 derive + reflect 属性双面 | bevy 0.19.1 | [PAT-B-014-brp-readable-writable-resource.md](./PAT-B-014-brp-readable-writable-resource.md) | TS-08 get/mutate_resources 断言（2026-09-28） |
| PAT-B-015 | 插件组装分层——配置物化 + 引擎/领域/观察/通路/工具五层 | bevy 0.19.1 | [PAT-B-015-plugin-assembly-layers.md](./PAT-B-015-plugin-assembly-layers.md) | 12 任务 + ts-11 bench 形态自退出（2026-09-28） |
| PAT-B-016 | 可观测统计 Resource——tick/frame_count 分层心跳面 | bevy 0.19.1 | [PAT-B-016-observable-stats-resource.md](./PAT-B-016-observable-stats-resource.md) | TS-12 tick 递增 + ts-11 双源一致 0.49%（2026-09-28） |
| PAT-B-017 | 进程内测试套件——SuiteFn 注册表 + 无净副作用约定 | bevy 0.19.1 | [PAT-B-017-in-process-test-suites.md](./PAT-B-017-in-process-test-suites.md) | M2 验收③退出码绑定 + m1-phase2 十套件重跑（2026-09-28） |
| PAT-B-018 | BRP 安全绑定——显式 with_address(LOCALHOST) 不依赖默认 | bevy 0.19.1 | [PAT-B-018-brp-loopback-bind.md](./PAT-B-018-brp-loopback-bind.md) | 全部 BRP 证据经 127.0.0.1:15702（2026-09-28） |
| PAT-B-019 | 零依赖 CLI——手写 parse + 退出码协议 + 启动横幅自含口径 | bevy 0.19.1 | [PAT-B-019-std-only-cli-args.md](./PAT-B-019-std-only-cli-args.md) | 12 任务横幅自含口径 + 退出码 2 负例消费（2026-09-28） |
| PAT-B-020 | 任务集驱动脚本——绝对路径锚定+BRP 业务探针轮询+聚合退出码 | bevy 0.19.1 | [PAT-B-020-taskset-driver-script.md](./PAT-B-020-taskset-driver-script.md) | m1-phase2 全量重跑 12 任务退出码全 0 残留 clean（2026-09-28） |
| PAT-B-021 | 量测窗口空闲独占 + 冷编分型授权（负载敏感门禁/构建预检/降并行放行） | bevy 0.19.1（万阵仓同锁版本实测） | [PAT-B-021-measurement-window-cold-build-typing.md](./PAT-B-021-measurement-window-cold-build-typing.md) | 万阵 T010/T019/T022/T024 实测留痕（2026-10-07 跨仓回流；证据指针见条目） |

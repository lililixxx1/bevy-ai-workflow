# PAT-B-008-runif-gate-chain

### PAT-B-008：`run_if` 资源门控 + `.chain()` 定序——暂停语义的一行调度表达

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（Bevy 调度条件 API 形态）
- 标签：`run_if` `chain` `暂停` `调度` `Res条件` `状态门控`
- 源码出处：`game/src/sim.rs:170-176`

**场景**：一组 Update 系统需要共享同一开关（暂停时整组不跑、恢复后继续），且组内有固定顺序（先统计后运动）——不侵入系统函数体（不加 `if paused` 分支），在**调度声明处**一行表达门控 + 定序。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/sim.rs:170-176` 的逐字复制——未删减，系 `SimPlugin::build` 链中段片段，不独立编译，未单独重新编译）：

```rust
// 逐字抄自 game/src/sim.rs:170-176（build 链中段，前接 .register_type 等链式调用）：
.add_systems(Startup, (init_metadata, spawn_swarm, tag_first_ten).chain())
.add_systems(
    Update,
    (update_stats, move_swarm)
        .chain()
        .run_if(|config: Res<SimConfig>| !config.paused),
);
```

要点：①`.run_if()` 挂在**元组整体**上——整组系统同门同帧开关，避免逐系统挂条件产生部分启停的中间态；②闭包直接取 `Res<SimConfig>` 作条件（Bevy 系统参数注入），与 PAT-B-007 的 observer 翻转配套：BRP `trigger_event` 同步翻转 `config.paused`，下一帧起整组停走；③`.chain()` 保组内序（统计读到的 tick 与运动写入的帧对齐，跨系统确定性见 PIT-B-012——无序共存不报错但顺序无契约）；④注意 `run_if` 用**值语义闭包**捕获资源参数，不借用外部状态（批次 C 探针无坑确认：捕获闭包与 `Res` 注入两种形态均编译通过）。

**为什么**：暂停/倍速/调试开关类「横切一组系统」的需求，放在调度声明处比散在函数体内更可审计（一处看全哪些系统受控），也让「暂停时什么在跑什么停了」可被外部脚本用 tick/位置双指标验证。

**验证证据**：
- 代码为 T002/T015 已验证任务代码的逐字复制（未删减，原样可编译），**删减后未单独重新编译**；
- 运行验证：TS-04 pause 子命令五步时序（暂停后 tick 停滞、位置冻结；恢复后 tick 递增——`docs/evidence/m1-phase2/ts-04/`，2026-09-27）；暂停期间 BRP 仍可读资源/截图（M2 RPC 全链不受门控影响）。

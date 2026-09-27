# PAT-B-002-spawn-batch-owned-iterator

### PAT-B-002：spawn_batch 批量生成——owned 值拷出 + move 闭包满足 `'static` 约束

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific
- 标签：`spawn_batch` `Commands` `'static` `move 闭包` `批量生成` `确定性`
- 源码出处：`game/src/sim.rs:227-244`（spawn_swarm，T003 首役落地）

**场景**：Startup/单次系统里按配置批量生成 N 个实体（N 可达数万），闭包需要 rng、配置字段与 mesh/material 句柄。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/sim.rs:227-244` 的逐字连续子段，仅删减，未改写）：

```rust
let seed = config.seed;
let max_speed = config.max_speed;
let mut rng = SplitMix64::new(seed);
let iter = (0..config.entity_count).map(move |index| {
    let (origin, linear, phase) = draw_initial(&mut rng, seed, index, max_speed);
    let translation = wanderer_translation(&Wanderer { index, origin, phase }, &Velocity { linear }, 0.0);
    (
        Mesh3d(mesh.clone()),
        MeshMaterial3d(body_material.clone()),
        Transform::from_translation(translation),
        Wanderer { index, origin, phase },
        Velocity { linear },
    )
});
commands.spawn_batch(iter);
```

**为什么**：`Commands::spawn_batch` 要求迭代器 `I: IntoIterator + Send + Sync + 'static`（`bevy_ecs-0.19.1/src/system/commands/mod.rs:587-593`）——命令队列延迟到帧末执行，闭包必须自带全部数据；`Res`/`ResMut` 系统参数非 `'static` 不能 move，只拷贝所需标量字段（`let seed = config.seed;`），句柄 `clone()` 是 Arc 内部指针代价可忽略。反例面（E0373/E0521）见 [pitfalls.md](../pitfalls.md) PIT-B-001。

**验证证据**：
- 代码为 T003 已验证任务代码的逐字删减，**删减后未单独重新编译**（模板注释口径）；原代码在 T003 中过 `cargo check --workspace --all-targets` REAL_EXIT=0（2026-09-26）；
- 运行验证（三档指针）：1000 档——BRP 计数 == 1000（`docs/evidence/brp-assert-result.txt` #3）；500 档——TS-01 断言① query 计数 500 == entity_count（`docs/evidence/ts-01-brp.md` 与 `docs/evidence/m1-phase2/ts-01/runner.txt`）；50000 档——`--expect-count 50000` 绑定（`docs/evidence/m1-phase2/run-all.sh:132` 与 `ts-11/runner.txt`）。

# PAT-B-003-startup-chain-auto-deferred

### PAT-B-003：Startup 系统链式定序——spawn 与下游消费者之间依赖 auto_insert_apply_deferred 自动 flush

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific
- 标签：`Startup` `chain` `ApplyDeferred` `auto_insert_apply_deferred` `系统定序`
- 源码出处：`game/src/sim.rs:170`（`(init_metadata, spawn_swarm, tag_first_ten).chain()`）与 `:260-271`（tag_first_ten）

**场景**：多个 Startup 系统有数据依赖（A 用 Commands spawn 实体，B 查询这些实体继续加工），需要确定性次序且不想手工插 `ApplyDeferred`。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/sim.rs:170` 与 `:260-271` 的逐字删减——删除 `tagged` 计数语句与 `info!` 行，未改写）：

```rust
// 注册侧（sim.rs:170）：
app.add_systems(Startup, (init_metadata, spawn_swarm, tag_first_ten).chain());

// 消费者（sim.rs:260-268，删减计数语句）：
fn tag_first_ten(mut commands: Commands, query: Query<(Entity, &Wanderer)>) {
    for (entity, wanderer) in &query {
        if wanderer.index < 10 {
            commands.entity(entity).insert(Tagged {
                tag: format!("wanderer-{}", wanderer.index),
            });
        }
    }
}
```

**为什么**：`.chain()` 声明排序依赖后，构建通道 `auto_insert_apply_deferred` 默认开启（`bevy_ecs-0.19.1/src/schedule/schedule.rs:1629`；语义 `auto_insert_apply_deferred.rs:13-17`）——对「上游含 Deferred 参数（Commands 即是）且存在排序依赖」的边自动插入 ApplyDeferred，spawn 的实体在下游系统运行前已入 World，无需手工 `apply_deferred`。显式 ordering 本身是 SKILL §3.3 纪律（禁止依赖注册顺序碰巧对）。

**验证证据**：
- 代码为 T014 已验证任务代码的逐字删减（删除计数语句），**删减后未单独重新编译**；原代码过 `cargo check --workspace` REAL_EXIT=0（增量 1.37s，2026-09-26）；
- 运行验证：TS-09 断言③ query `Tagged` + `With<Wanderer>` 恰 10 行 tag=wanderer-0..9（`docs/evidence/ts-09-brp.md`）；ts-09 套件进程内复测同值（T020，`docs/evidence/m1-phase2.md`）。

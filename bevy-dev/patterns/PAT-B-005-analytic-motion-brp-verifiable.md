# PAT-B-005-analytic-motion-brp-verifiable

### PAT-B-005：解析式运动——位置是时间的解析函数，BRP 端可精确复算断言

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（为 BRP 状态断言设计的模拟形态）
- 标签：`解析式` `确定性` `BRP 断言` `状态注入` `elapsed_secs` `公式单点`
- 源码出处：`game/src/sim.rs:319-335`（wanderer_translation + move_swarm）+ 模块注释 `:3-15`（确定性口径）

**场景**：模拟体位置需要被外部（BRP/测试）**任意时刻精确断言**，而非只能抽样近似比对；工作流三「状态注入 + 数学复算优先」取向（意向文档 §3/§5.3）的模拟侧配套。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/sim.rs:318-335` 的逐字删减——删去 `wanderer_translation` 的 doc 注释行，未改写）：

```rust
// sim.rs:319-322（公式单点定义，测试与 BRP 断言共用）：
pub fn wanderer_translation(w: &Wanderer, v: &Velocity, t: f32) -> Vec3 {
    w.origin + v.linear * t
        + Vec3::Y * (BOUNCE_AMP * (std::f32::consts::TAU * BOUNCE_HZ * t + w.phase).sin())
}

// sim.rs:325-335（统计先行写 t，运动后行读 t，.chain() 定序）：
fn update_stats(mut stats: ResMut<SimStats>, time: Res<Time>) {
    stats.tick += 1;
    stats.elapsed_secs += f64::from(time.delta_secs());
}

fn move_swarm(stats: Res<SimStats>, query: Query<(&Wanderer, &Velocity, &mut Transform)>) {
    let t = stats.elapsed_secs as f32;
    for (w, v, mut transform) in query {
        transform.translation = wanderer_translation(w, v, t);
    }
}
```

要点：①位置 = `origin + linear·t + 扰动项(t)` 的**解析函数**——同 seed 同 t 必逐位相同，与帧率无关；②唯一的时钟源是 `SimStats.elapsed_secs`（仅未暂停时累计，BRP 可读），运动系统与统计系统 `.chain()` 定序（统计先行写 t，运动后行读 t）；③公式 `pub` 单点定义，BRP 断言脚本与单测复算同一份公式而非抄 demo 输出（PIT-M-002 纪律）。

**为什么**：帧驱动积分（`pos += vel * dt`）的位置依赖帧历史，外部只能近似断言；解析式让断言方「读 t → 复算 → 与 Transform 逐位比对」成为精确闭环，还天然兼容暂停（t 冻结 ⇒ 位置冻结，TS-04 口径）。

**验证证据**：
- 代码为 T003 已验证任务代码的逐字删减（删 doc 注释行），**删减后未单独重新编译**；
- `cargo test -p game` 单测 `analytic_motion_is_deterministic_per_seed_and_time`（T003）；
- 运行验证：TS-07 独立复算（node f64）与 BRP 读回 Transform 三分量 |Δ|≤0.1、x/z 精确为 0（T012，`docs/evidence/ts-07-brp.md`）；T020 套件化后全量 Transform 与解析式复算**逐位一致最大偏差 0e0**（ts-07 套件，`docs/evidence/m1-phase2.md`）。

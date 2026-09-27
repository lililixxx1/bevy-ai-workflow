# PAT-B-004-deterministic-prng-single-draw-point

### PAT-B-004：确定性模拟——自研 PRNG Resource + 单一抽取落点 + (seed, index) 派生

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（Bevy 模拟确定性口径的落地形态；PRNG 算法本身引擎无关）
- 标签：`确定性` `SplitMix64` `seed` `可复现` `单测锁定` `Resource`
- 源码出处：`game/src/rng.rs`（SplitMix64 实现）+ `game/src/sim.rs:293-316`（draw_initial 单一落点）+ `:33-35`（serde 兼容默认）

**场景**：模拟需要跨进程/跨运行逐位可复现（同 seed + 同 count ⇒ 同初值、同位置），供 BRP 断言、任务测试集回归与帧率基线共用口径。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/sim.rs:291-316` 的逐字删减——保留 doc 注释原文，未改写）：

```rust
// 逐字抄自 game/src/sim.rs:293-316（doc 注释为原文，仅删去首行标题句）：
/// 抽取顺序（**禁改**，`Wanderer.origin` 跨配置逐位一致的前提）：
/// `next_range_f32(AREA)`×2（origin.x/z）→ `next_range_f32(1.0)`×2（dir.x/y）
/// → `next_f32()`×1（单位速度）。`max_speed` 仅缩放最后一步；phase 不占抽取，
/// 由 `phase_hash(seed, index)` 独立派生。
pub(crate) fn draw_initial(
    rng: &mut SplitMix64,
    seed: u64,
    index: u32,
    max_speed: f32,
) -> (Vec3, Vec3, f32) {
    let origin = Vec3::new(
        rng.next_range_f32(AREA),
        0.8,
        rng.next_range_f32(AREA),
    );
    let dir = Vec2::new(rng.next_range_f32(1.0), rng.next_range_f32(1.0));
    let speed = rng.next_f32() * max_speed;
    let linear = Vec3::new(dir.x, 0.0, dir.y).normalize_or_zero() * speed;
    let phase = phase_hash(seed, index);
    (origin, linear, phase)
}
```

要点：①不引 `rand` 依赖，SplitMix64 纯数学映射（rng.rs，参考向量由**独立实现**计算——见 [pitfalls.md](../../assets-methodology/pitfalls.md) PIT-M-002 反例面）；②抽取顺序只在一处定义（`pub(crate)` 供 run_tests 套件复用，禁止第二份实现）；③每实体不依赖 PRNG 流位置的量用 `(seed, index)` 哈希派生（phase_hash）；④配置字段变更走 serde 兼容默认（`#[serde(default = "...")]` 保持旧数据可读）。

**为什么**：任何第二份抽取实现或顺序改动都会破坏「同 seed 同 count ⇒ 逐位一致」——这是任务测试集当回归集（升级窗口）与 BRP 精确断言的前提。

**验证证据**：
- 代码为 T003/T007 已验证任务代码的逐字删减（Vec3::new 保持原多行形态），**删减后未单独重新编译**；
- `cargo test -p game` 单测 4 项锁定（同 seed 同序列 / 抽取顺序与 cap 无关 / 参考向量 / 解析运动确定性，T003/T007）；
- 运行验证：TS-01 断言④同 seed 跨进程 500 个 linear 向量逐位一致（compared=500/mismatch=0，`docs/evidence/ts-01-brp.md`）；TS-02 断言③改 max_speed 后 origin 逐位一致（T007）。

# PAT-B-016-observable-stats-resource

### PAT-B-016：可观测统计 Resource——每帧刷新的字段就地为 BRP/套件提供只读断言面

- 日期：2026-09-28
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（bevy 反射资源 + BRP 观测面组合）
- 标签：`Resource` `SimStats` `tick` `fps` `观测` `BRP` `轮询`
- 源码出处：`game/src/sim.rs:61-75`（SimStats 定义）+ bench 刷新侧 `game/src/bench.rs`

**场景**：外部工具（BRP / 进程内套件）需要不侵入系统的「心跳」证据：帧是否推进、帧率多少、模拟时间多长——一个 Resource 每帧由系统刷新，全程只读消费。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/sim.rs:61-75` 的逐字删减——保留 doc 注释原文）：

```rust
#[derive(Resource, Reflect, Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct SimStats {
    /// 模拟 tick 数（每未暂停帧 +1；BRP 断言「每帧真实更新」的直接证据）。
    pub tick: u64,
    /// 渲染帧计数（含暂停帧，由 bench 模块维护）。
    pub frame_count: u64,
    /// 累计模拟时间（秒，仅未暂停时累计；解析式运动的 t）。
    pub elapsed_secs: f64,
    /// 最近 1 秒窗口帧率（bench 模块每秒刷新）。
    pub fps_1s: f64,
    /// 稳态平均帧率（warmup 丢弃后；bench 模块刷新）。
    pub avg_fps: f64,
}
```

要点：①字段语义分层：`tick`（模拟推进）/ `frame_count`（渲染帧）分离——暂停时 tick 停、frame_count 走，两种「活着」的证据；②`elapsed_secs` 是解析式运动的 t，与实体位置可互相复算（PAT-B-005）；③同一 derive 四件套 + reflect 属性（同 PAT-B-014）；④fps 字段由 bench 模块刷新——观测字段允许「字段归属资源、刷新归属专项系统」。

**为什么**：跨进程断言最廉价的面是「可序列化只读资源」——比事件/日志轮询稳，比直查组件广（一次调用多字段互证）。

**验证证据**：
- 代码为 T003/T016 已验证任务代码的逐字删减，删减后未单独重新编译；
- 运行验证：TS-12 tick 三点采样严格递增断言（docs/evidence/ts-12-brp.md）；ts-11 avg_fps BRP 读值与 [BENCH] 行双源一致 0.49%（docs/evidence/ts-11-brp.md）；run_tests 多套件以 SimStats 判定帧推进。

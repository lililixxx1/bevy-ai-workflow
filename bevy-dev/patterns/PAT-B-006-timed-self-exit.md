# PAT-B-006-timed-self-exit

### PAT-B-006：定时自退出采集——BenchState 幂等闸门 + `MessageWriter<AppExit>` 条件写出

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（AppExit 的 Message 化 API 形态；「warmup + 定时退出」采集口径引擎无关）
- 标签：`AppExit` `MessageWriter` `采集` `自退出` `幂等` `基线`
- 源码出处：`game/src/bench.rs:58-107`（bench_sample 主体）

**场景**：无人值守的定时采集——跑满 `--bench-secs S` 秒后打印汇总行并让进程以 `AppExit::Success`（退出码 0）自行结束，供任务测试集/基线脚本直接判退出码；同一系统又要支持 `S=0` 常驻模式（每秒打印 `[STATS]`，BRP 可随时读）。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/bench.rs:58-108` 的逐字删减 + 两处行内注解为本条目添加，其余原文）：

```rust
// 抄自 game/src/bench.rs:58-108（删去 warmup 统计中段；两条行内注释为本条目添加）：
fn bench_sample(
    time: Res<Time>,
    config: Res<SimConfigImport>,
    mut state: ResMut<BenchState>,
    mut stats: ResMut<SimStatsImport>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.finished {
        return; // 幂等闸门：写出 AppExit 后本系统空转，直到进程退出
    }
    // ...（每秒窗口打印 [STATS]、warmup 后累计 measure_frames/measure_secs）
    if config.bench_secs > 0.0 && state.wall_secs >= config.bench_secs as f64 {
        state.finished = true; // 先置闸门再退出——防同帧重复写出
        info!(
            "[BENCH] n={} seed={} bench_secs={:.1} warmup_secs={:.1} measure_secs={:.3} measure_frames={} avg_fps={:.1}",
            /* ...字段实参略（原文 :98-104）... */
        );
        exit.write(AppExit::Success);
    }
}
```

要点：①0.19 退出事件走 Message 系——系统参数 `mut exit: MessageWriter<AppExit>` + `exit.write(AppExit::Success)`（旧语料 `AppExit::send()`/`EventWriter` 均不存在；官方示例 `bevy-0.19.1/examples/app/custom_loop.rs:36-38`，核实 2026-09-26）；②`finished` 闸门**先置位后写出**，保证写出后到进程真正退出前的余帧不再触发；③App 侧由 `bevy_app` 的 runner 消费 AppExit 消息退出（无需手写监听）。

**为什么**：采集/回归脚本只认退出码与汇总行——「到时自退 + 退出码 0」把帧率采集变成无副作用的黑盒命令；幂等闸门避免退出窗口期重复写消息导致日志双行。

**验证证据**：
- 代码为 T003/T005 已验证任务代码的逐字删减，**删减后未单独重新编译**；
- 运行验证：`--bench-secs` 模式进程自退出 REAL_EXIT=0 且输出 `[BENCH]` 汇总行（TS-11：`[BENCH] n=50000 ... avg_fps=60.0`，`docs/evidence/ts-11-brp.md`，2026-09-26）；
- 常驻模式（S=0）每秒 `[STATS]` 且 BRP 可读 SimStats（TS-01 断言①、TS-11 断言①，`docs/evidence/`）。

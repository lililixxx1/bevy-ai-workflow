# t003 帧率基线数据与口径说明

> 净室重跑 T003（M1 第一役·裁剪版）采集。判定口径五要素逐项列于下；任何复测须同口径才可比。

## 一、采集口径（五要素）

| # | 要素 | 取值 | 证据 |
|---|---|---|---|
| 1 | release 构建 | `cargo build --release`（优化构建，非 bench profile） | `logs/build-release-attempt1.log` REAL_EXIT=0 |
| 2 | 固定分辨率 / 窗口模式 | 1280×720，`WindowMode::Windowed`（窗口化） | `src/main.rs`（PAT-B-011 形态） |
| 3 | vsync 设置 | `PresentMode::AutoVsync`（vsync 组）/ `PresentMode::AutoNoVsync`（no-vsync 对照组），CLI `--no-vsync` 切换，同一二进制 | 同上 |
| 4 | CPU / GPU / 驱动 | CPU：12th Gen Intel Core i5-12490F（12 逻辑核）；GPU：NVIDIA GeForce RTX 3050；驱动：NVIDIA 537.58（wmi DriverVersion 31.0.15.3758，游戏内 AdapterInfo driver_info "537.58"，Vulkan）；OS：Windows 10 19044；RAM 31.8 GiB | `logs/env-hardware.txt`、`logs/run-1000.log` AdapterInfo 行 |
| 5 | 固定随机种子 | `--seed 20260926`（确定性 PRNG SplitMix64，抽取单点 `sim::draw_initial`） | `src/rng.rs` / `src/sim.rs`、各 `[CLI]` 横幅行 |

其他固定项：`--max-speed 2.0`（默认）；warmup 2.0s 丢弃后取均值（`avg_fps = measure_frames / measure_secs`）；场景 = 1 相机 + 1 地面 + 1 平行光 + N 个 Cuboid 实体（解析式运动）。

## 二、基线数据（[BENCH] 汇总行原文，逐字摘自 logs/bench-*.log）

| 阶梯 | vsync | no-vsync | 原始日志 |
|---|---|---|---|
| N=1000 | avg_fps=60.1（measure_secs=8.001, frames=481） | avg_fps=82.7（8.003s, 662 帧） | `logs/bench-1000-vsync.log` / `logs/bench-1000-novsync.log` |
| N=50000 | avg_fps=60.1（8.000s, 481 帧） | avg_fps=139.8（7.998s, 1118 帧） | `logs/bench-50000-vsync.log` / `logs/bench-50000-novsync.log` |

原文（时间戳为 INFO 日志自带）：

```text
2026-09-28T07:30:04.466921Z  INFO demo::bench: [BENCH] n=1000 seed=20260926 bench_secs=10.0 warmup_secs=2.0 measure_secs=8.001 measure_frames=481 avg_fps=60.1
2026-09-28T07:30:24.105549Z  INFO demo::bench: [BENCH] n=1000 seed=20260926 bench_secs=10.0 warmup_secs=2.0 measure_secs=8.003 measure_frames=662 avg_fps=82.7
2026-09-28T07:30:44.686304Z  INFO demo::bench: [BENCH] n=50000 seed=20260926 bench_secs=10.0 warmup_secs=2.0 measure_secs=8.000 measure_frames=481 avg_fps=60.1
2026-09-28T07:31:48.537602Z  INFO demo::bench: [BENCH] n=50000 seed=20260926 bench_secs=20.0 warmup_secs=2.0 measure_secs=17.999 measure_frames=1081 avg_fps=60.1
2026-09-28T07:32:16.656444Z  INFO demo::bench: [BENCH] n=50000 seed=20260926 bench_secs=10.0 warmup_secs=2.0 measure_secs=7.998 measure_frames=1118 avg_fps=139.8
```

（第四行是 50000 vsync 的 20s 复测轮，用于 BRP 双源读取；三轮 vsync@50000 结果一致 60.1。）

## 三、双源一致性（BRP 读值 vs [BENCH] 行）

50000 vsync 20s 轮运行至 ~18.5s 时经 BRP `world.get_resources` 读 `demo::sim::SimStats`：
`fps_1s=59.986499010308805`（raw：`logs/brpbench/stats-50000-vsync2.json`）
vs 该轮 `[BENCH] avg_fps=60.1`：相对偏差 |60.1−59.986|/59.986 = 0.19%，双源一致（阈值 ±10%）。

## 四、口径说明与可比性边界

- vsync 组被显示器刷新率钳制（本机 60Hz → 恒 60.1），仅证明采集链路活着；回归哨兵应使用 no-vsync 组（82.7 / 139.8 阶梯才有区分度）——与原仓库回归口径注意事项第 3 条同判。
- 1000 档 no-vsync（82.7）低于 50000 档 no-vsync（139.8）非笔误：本机两轮实测如此（可能与窗口合成器/后台负载抖动有关），如实记录；后续复测如需稳定对照，建议加大 bench_secs 并多轮取中位数（本轮未做，属采集深度边界）。
- 跨机器不可比：CPU/GPU/驱动任一变更即失可比性（要素 4 必须同值）。

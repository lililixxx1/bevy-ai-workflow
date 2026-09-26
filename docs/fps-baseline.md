# 帧率基线（M1 第一役 demo）

- 适用 Bevy 版本：**0.19**（Cargo.lock 解析 0.19.1）
- 采集日期：2026-09-26
- 数据角色：升级窗口回归对照（意向文档 §4「升级窗口完成判定」、§10 第 5 条）与 taskset TS-11 的基线源
- 证据：`docs/evidence/bench-n*.log`（vsync 组与 no-vsync 组，各 5 阶梯）

## 口径（五要素，缺一不可）

| 要素 | 取值 |
|---|---|
| 1. 构建 | `cargo build --release -p game`（默认 release profile，无 LTO 定制） |
| 2. 窗口 | 1280×720 物理分辨率、`WindowMode::Windowed` 窗口化；present mode 两组：**AutoVsync**（默认口径，即 vsync on）与 **AutoNoVsync**（`--no-vsync`，取阶梯区分度） |
| 3. 机器 | CPU：12th Gen Intel Core i5-12490F（6C/12T）；GPU：NVIDIA GeForce RTX 3050 4GB（Vulkan 后端，驱动 537.58，DiscreteGpu）；OS：Windows 10 IoT 企业版 LTSC 10.0.19044 |
| 4. 随机种子 | `--seed 20260926`（口径常量，`game/src/cli.rs` 默认值） |
| 5. 负载定义 | 相机轨道漫游（半径 90m、高度 45m、0.1 rad/s）+ N 个 Cuboid(0.6³) 实体，解析式运动 `p(t)=origin+v·t+Y·0.8·sin(2π·0.7·t+phase)`（确定性初值见 `game/src/sim.rs` 模块注释）；平行光无阴影；地面 Circle(144m) |

采集参数：每阶梯 `--bench-secs 12`，warmup 丢弃 2.0s，测量窗 ≈10.0s（`[BENCH]` 汇总行为准）；单进程单阶梯串行执行，无其他重负载任务。

## 数据

### vsync 组（AutoVsync，默认口径）

| N | measure_frames | measure_secs | 平均 FPS |
|---|---|---|---|
| 1000 | 601 | 10.016 | 60.0 |
| 5000 | 601 | 10.016 | 60.0 |
| 10000 | 601 | 10.016 | 60.0 |
| 25000 | 601 | 10.016 | 60.0 |
| 50000 | 602 | 10.033 | 60.0 |

vsync 组全部锁在显示器刷新率 60Hz（Fifo 队列）——**50000 实体也未跌破 vsync 上限**，该口径下阶梯无区分度，仅证明「该负载在本机满帧运行」。

### no-vsync 组（AutoNoVsync，`--no-vsync`）

| N | measure_frames | measure_secs | 平均 FPS |
|---|---|---|---|
| 1000 | 5348 | 10.001 | **534.7** |
| 5000 | 4418 | 10.004 | **441.6** |
| 10000 | 3501 | 10.002 | **350.0** |
| 25000 | 1986 | 10.002 | **198.6** |
| 50000 | 1139 | 10.005 | **113.8** |

no-vsync 组阶梯区分度清晰（1000→50000 实体，帧率 534.7→113.8，约 4.7 倍衰减）——**升级窗口回归与 TS-11 判定建议以本组为对照**。

## 环境备注（影响可比性，如实记录）

- 本机存在 GameViewer Virtual Display Adapter（远程串流虚拟显示器）；游戏窗口渲染于该显示环境，BRP 断言（`docs/brp-smoke.md`）不受影响，但绝对帧率数值换显示环境后不可比。
- 跨机器绝对值不可比；同机重测须保持口径五要素一致。
- 实体位置为解析式（含线性漂移项），长时间运行后实体会离开初始 ±120m 区域（12s 采集窗内位移 ≤36m，对负载无影响：实体始终在视锥内由相机环绕观察）。

## 复测方式

```text
cargo build --release -p game
./target/release/game.exe --count <N> --seed 20260926 --bench-secs 12 [--no-vsync]
# 结果取日志中 [BENCH] 汇总行；对照上表同组同阶梯
```

注：TS-02（2026-09-26）起速度上限经 `--max-speed F` 配置化（默认 3.0，`game::sim::DEFAULT_MAX_SPEED`）；上表基线数据均在默认值下采集，复测对照时不传该参即口径一致。

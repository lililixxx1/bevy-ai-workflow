# T003 净室重跑报告（M1 第一役·裁剪版）

- 执行者：净室重跑独立执行者（无 prior 会话上下文；开局材料 = `cleanroom/INJECTED-agents-discipline.md` + `cleanroom/req-t003.md`，知识资产 = 仓库 `bevy-dev/` 与 `assets-methodology/`）
- 日期：2026-09-28
- 工作区：`C:\Users\Administrator\Desktop\ccc\cleanroom\t003\`（独立 crate，不入任何 workspace）
- 判定口径（需求书）：**首次 `cargo check` 通过 + demo 首次运行满足验收清单 = 一次通过**

## 一、逐轮时间线（全部输出 + 真实退出码在 `logs/`）

| # | 轮次 | 内容 | 结果 | 退出码 | 证据 |
|---|---|---|---|---|---|
| 0 | 环境核对 | rustc 1.98.1（≥ MSRV 1.95.0）、node v24.12.0、registry bevy-0.19.1 就位 | 通过 | — | 会话记录 |
| 0b | 硬件采集 | CPU/GPU/驱动/OS（基线要素 4） | 成功 | 0 | `logs/env-hardware.txt` |
| 1 | **check attempt1（判定轮）** | 首次 `cargo check`（全量，2m08s） | **通过**（1 个 dead_code warning：`BenchState.last_report` 从未读取） | **0** | `logs/check-attempt1.log` |
| 2 | check attempt2 | 删除死字段后复查（增量 1.05s，零警告）——卫生性修改，非返工（首判已过） | 通过 | 0 | `logs/check-attempt2.log` |
| 3 | build --release | 8m19s | 成功 | 0 | `logs/build-release-attempt1.log` |
| 4 | 运行 1000 档（常驻） | `demo.exe --count 1000 --seed 20260926`，后台任务托管（PAT-M-005），PID 22616 | 存活，[STATS] 60fps 推进 | 1（末轮 taskkill 强杀所致，预期） | `logs/run-1000.log` |
| 5 | BRP 冒烟 | `rpc.discover` → 23 方法 | 成功 | 0 | `logs/brp1000/discover.json/.exit` |
| 6 | 类型路径实测 | `world.list_resources` / `world.list_components`（311 组件）确认 `demo::sim::*`、`demo::camera::*`、`bevy_transform::components::transform::Transform` 全路径（PIT-B-003 纪律：不凭记忆写内置类型路径） | 成功 | 0 | `logs/brp1000/list-*.json/.exit` |
| 7 | 计数/配置查询 | `world.query` Wanderer（strict）→ 1000 行；`get_resources` SimConfig 回读 1000/20260926 | 成功 | 0 | `logs/brp1000/count-q.json`、`config.json`、`rig.json` |
| 8 | 括号采样 ×2 | `tools/brp-sample.sh` s1/s2（间隔 1.5s；8 个 curl 各 66-83ms） | 成功 | 0×8 | `logs/brp1000/s1-*`、`s2-*` |
| 9 | **断言判定（首判）** | `node tools/assert-motion.js logs/brp1000 1000 20260926` | **9/9 PASS** | **0** | `logs/brp1000/assert-motion-result.txt` |
| 10 | 杀 1000 档 | `taskkill //F //IM demo.exe`（PID 22616）+ 残留核对 | 无残留 | 0/1(grep 无匹配) | `logs/kill-1000.log`、`residue-check-1000.log` |
| 11 | 运行 50000 档（常驻） | PID 5720，[STATS] 60fps | 存活 | 1（强杀，预期） | `logs/run-50000.log` |
| 12 | 50000 档 BRP | `rpc.discover` + 计数查询（8.3MB 响应）→ **50000 行** + 配置回读 + tick 1795→1861 递增 | 成功 | 0×5 | `logs/brp50000/*` |
| 13 | 杀 50000 档 | PID 5720 + 残留核对 | 无残留 | 0/1 | `logs/kill-50000.log`、`residue-check-50000.log` |
| 14 | 基线轮 A | 1000 vsync，`--bench-secs 10` 自退出 | avg_fps=60.1 | 0 | `logs/bench-1000-vsync.log` |
| 15 | 基线轮 B | 1000 no-vsync | avg_fps=82.7 | 0 | `logs/bench-1000-novsync.log` |
| 16 | 基线轮 C | 50000 vsync | avg_fps=60.1 | 0 | `logs/bench-50000-vsync.log` |
| 17 | C 轮 BRP 双源读取（失败轮，留痕不删） | sleep 9 后 curl → 连接拒绝：**进程已先退场**（50000 档启动至首帧 ~12s，10s bench 已结束） | 失败 | **7** | `logs/brpbench/stats-50000-vsync.json/.exit`（空文件） |
| 18 | 基线轮 C2（复测） | 50000 vsync `--bench-secs 20`，14s 处 BRP 读 SimStats：fps_1s=59.986 vs [BENCH] 60.1，偏差 0.19% | 成功 | 0 | `logs/bench-50000-vsync-brp.log`、`logs/brpbench/stats-50000-vsync2.json` |
| 19 | 基线轮 D | 50000 no-vsync | avg_fps=139.8 | 0 | `logs/bench-50000-novsync.log` |
| 20 | 终残留核对 | `tasklist \| grep demo.exe` | 无残留 | 1(无匹配) | `logs/residue-check-final.log` |
| 21 | 版本核对 | `Cargo.lock`: bevy 0.19.1；manifest `bevy = "0.19"` | 一致 | — | `Cargo.lock` |

## 二、验收清单逐项判定

### a) 首次 `cargo check` 通过 —— **PASS**

- 判定轮 = 时间线 #1：REAL_EXIT=0（`logs/check-attempt1.log` 末行）。全量 2m08s，8 个依赖 warning 无（仅本 crate 1 个 dead_code）。
- attempt2（#2）为死字段清理后的复查，零警告 REAL_EXIT=0。失败轮数为 0（无任何失败 check）。

### b) release 构建成功 —— **PASS**

- `logs/build-release-attempt1.log`：`Finished release profile [optimized] ... in 8m 19s`，REAL_EXIT=0。

### c) demo 运行后经 BRP 验证 —— **PASS**

**实体计数与 CLI 参数一致（两档各验一次）：**

- 1000 档：`world.query {"data":{"components":["demo::sim::Wanderer"]},"strict":true}` → result 数组 **1000 行**；`get_resources SimConfig` → `entity_count=1000, seed=20260926`（`logs/brp1000/count-q.json`、`config.json`）。
- 50000 档：同查询 → **50000 行**（8.3MB 响应）；config 回读 `entity_count=50000`；tick 1795→1861 严格递增（`logs/brp50000/count-q.json`、`config.json`、`stats*.json`）。

**解析式运动数学断言（自拟 6 条 ≥ 5，独立 f64 复算，9/9 PASS）：** 断言原文、curl 取值、判定结果三样留档 = `tools/assert-motion.js`（原文）+ `logs/brp1000/*.json`（curl 取值）+ `logs/brp1000/assert-motion-result.txt`（判定）。逐条结果：

| id | 断言 | 实测 | 判定 |
|---|---|---|---|
| A1 | 运动参数时不变：origin/phase/linear 两采样（间隔 ~1.9s）逐位一致 | origin1==origin2==[89.601806640625, 0.800000011920929, 80.34127044677734]，phase 4.691539764404297 双采样同值 | PASS |
| A2 | 相机轨道半径恒等式 `hypot(x,z) == rig.radius`（t 无关） | 90.000005 / 90.000001 vs 90 | PASS |
| A3 | 相机高度恒等式 `y == rig.height` | 45, 45 vs 45 | PASS |
| A4 | 实体解析位置（index=0）`pos(t)=origin+linear·t+ŷ·AMP·sin(TAU·HZ·t+phase)` | 实测 [170.277, 0.719, 154.666] vs 复算(t1=84.8698) [170.181, 0.532, 154.578]，\|Δ\|=0.2272 ≤ 1.3304 | PASS |
| A5 | 相机解析位置 `(r·cos(ωt+φ), h, r·sin(ωt+φ))` | 实测 [-39.798, 45.000, -80.722] vs 复算 [-40.671, 45.000, -80.286]，\|Δ\|=0.9758 ≤ 1.5252 | PASS |
| A6 | 相机角位移 `Δatan2(z,x) == ω·Δt` | 0.100841 rad vs 0.101256 rad（ω=0.05, Δt=2.0251s） | PASS |
| C1 | 计数 == CLI | 1000/1000 | PASS |
| C2 | 配置回读 == 字面值 | count/seed 双档一致 | PASS |
| C3 | tick 严格递增 | 5076→5198；50000 档 1795→1861 | PASS |

### d) 帧率基线数据齐五要素 —— **PASS**

见 `baseline.md`：release 构建（#3 证据）/ 1280×720 Windowed / vsync 双组（AutoVsync 与 AutoNoVsync）/ CPU i5-12490F + RTX 3050 + 驱动 537.58（wmi 31.0.15.3758）/ 种子 20260926；四阶梯数据（60.1 / 82.7 / 60.1 / 139.8）+ BRP 双源一致性 0.19%。

## 三、自拟断言原文与容差说明

断言代码原文 = `tools/assert-motion.js`（判定输出的断言描述逐字来自该脚本）。核心断言原文（JS）：

```js
// A1（容差 0——确定性证据）
sameArr(w1.origin, w2.origin) && w1.phase === w2.phase && sameArr(v1.linear, v2.linear)
// A2（容差 0.01，t 无关恒等式，f32 舍入余量）
Math.abs(Math.hypot(x, z) - rig.radius) <= 0.01   // 双采样
// A3（容差 0.01）
Math.abs(y - rig.height) <= 0.01                  // 双采样
// A4（容差 = v_max·括号 + 0.1）
norm3(p1 - predictEntity(w1, v1, t1)) <= (config.max_speed + AMP*TAU*HZ) * (t_b - t_a) + 0.1
// A5（容差 = v_cam·括号 + 0.1）
norm3(cam1 - predictCam(t1)) <= (rig.radius * rig.angular_speed) * (t_b - t_a) + 0.1
// A6（容差 = ω·(br1+br2)/2 + 0.02）
Math.abs((angleOf(cam2) - angleOf(cam1)) - rig.angular_speed * (tm2 - tm1)) <= rig.angular_speed*(br1+br2)/2 + 0.02
```

**容差论证（时滞括号法）**：BRP 的 HTTP 请求在帧间对世界做原子读；系统链 `(update_stats → move_swarm → orbit_camera).chain/.after` 保证 Transform 写入帧的 t 与同帧 `SimStats.elapsed_secs` 一致。采样序 = stats-a(t_a) → 实体 query → 相机 query → stats-b(t_b)，故 Transform 所用 t_w ∈ [t_a, t_b] 必然成立。以 t_a 复算时位置误差上界 = 速度上限 × (t_b − t_a)（实测括号 ~0.317s，四个 curl 各 66-83ms）；再 +0.1 覆盖 f32（引擎）vs f64（断言侧）舍入（同 seed 下 PRNG 输出同位、公式同式，实际舍入差 ≪ 0.1，见 A2/A3 实测偏差 5e-6 量级）。A1/C1/C2/C3 容差 0 或严格字面相等（PAT-M-003 字面值绑定取向）。

## 四、基线口径五要素对照

见 `baseline.md` §一（逐要素取值与证据指针）与 §二（[BENCH] 原文逐字摘自 raw 日志）。要点：vsync 组被 60Hz 钳制仅证链路活着，回归哨兵用 no-vsync 组（82.7/139.8 阶梯）；1000 档 no-vsync 低于 50000 档系本机实测如此，如实记录（`baseline.md` §四）。

## 五、与原任务/知识资产的差异标注（如实）

1. **断言自拟**（需求书已声明的已知偏差）：原任务的断言集由其自建测试集（`run_tests`）提供；本次由 `tools/assert-motion.js` 自拟（两阶段判定的第二阶段形态，第一阶段人工对照不适用——重跑即脚本自判）。
2. **相机时钟取 `SimStats.elapsed_secs`（模拟时钟）**而非 PAT-B-012 的 `Time::elapsed_secs()`（真实时钟）：裁剪版无暂停功能，单一外部可读时钟使相机与实体共用同一 t，「读 t → 复算 → 比对」对两者统一成立（设计决定，非疏漏）。
3. **`--no-vsync` 旗标带哑值**（`--no-vsync 1`）：统一「旗标 值」双参解析循环，与 PAT-B-019 的布尔旗标形态略异。
4. **未运行 `cargo test`**（资源纪律，不跑重负载测试）：验证路径为 BRP 运行时断言（工作流三），编译门禁为 cargo check；crate 内未内置测试模块。
5. **运动参数取值自拟**（AREA=120 / max_speed=2.0 / AMP=1.2 / HZ=0.25 / 相机 r=90 h=45 ω=0.05）：需求书未钉死数值，取值以便断言容差论证为先。
6. 失败轮 2 个，均留痕未删：#17（BRP 读已退场进程，curl REAL_EXIT=7）与常驻轮强杀 REAL_EXIT=1（预期行为非失败，如实标注）。

## 六、返工次数（自认定）

**0 次。** 首判（#1 首次 check REAL_EXIT=0；#9 首次断言 9/9 PASS）一次通过；#2 死字段清理属卫生性修改（首判已过，不改变判定），#17→#18 是基线采集的补测（验收清单 d 的数据采集深度问题，非验收断言失败——d 项四轮数据齐备且 #17 轮本身不在验收清单内）。若将 #17 计为采集返工，则口径为「验收断言返工 0 次、基线补测 1 次」。

## 七、API 查证清单（禁凭记忆纪律的执行记录，本地 registry 源码核实 2026-09-28）

| API/事实 | 出处 | 去向 |
|---|---|---|
| `RemoteHttpPlugin::with_address(impl Into<IpAddr>)`、DEFAULT_ADDR=127.0.0.1、端口 15702 | `bevy_remote-0.19.1/src/http.rs:167/:60/:52` | brp.rs |
| `bevy::remote` 模块路径 | `bevy_internal-0.19.1/src/lib.rs:78` | brp.rs |
| `RemotePlugin`（lib.rs:572）/ `add_default_methods` 23 方法 | SKILL §6.2 + rpc.discover 实测 23 | 冒烟 |
| `Time::delta_secs() -> f32`、`elapsed_secs() -> f32` | `bevy_time-0.19.1/src/time.rs:283/:306` | sim.rs/bench.rs |
| `AppExit::Success`（Message 系写出） | `bevy_app-0.19.1/src/app.rs:1560-1567` + PAT-B-006 | bench.rs |
| `world.query` params 形态（data.components/option、filter、strict） | `bevy_remote-0.19.1/src/builtin_methods.rs:152-165/:842-848` | 断言请求 |
| `get_resources` 单数 `resource` 字段、单键 map 响应 | builtin_methods.rs:618-644 + 实测 | 断言 |
| 类型全路径（Transform 在 bevy_transform；311 组件清单） | `world.list_components` 实测 | 断言 |
| spawn_batch `'static`（owned move 闭包）、`Assets::add` &mut、Mesh3d/MeshMaterial3d、Camera3d+Transform 裸组件、Window 字段/PresentMode/WindowMode 不在 prelude | PAT-B-002/011/012/013 + PIT-B-001/002/031（引用速查条目） | sim/camera/main |

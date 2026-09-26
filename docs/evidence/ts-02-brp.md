# TS-02 断言证据：速度上限配置化（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-02-speed-cap.md`（3 条断言，逐条执行）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo，本次**改码任务**（`MAX_SPEED` 常量 → `SimConfig.max_speed` 字段 + CLI `--max-speed`）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 1000 --seed 20260926 --max-speed 9.0`；
  实际执行 `cargo build --release -p game`（REAL_EXIT=0）后直接运行同一产物
  `./target/release/game.exe --count 1000 --seed 20260926 --max-speed 9.0`（后台）——与 `cargo run --release`
  构建并运行的是同一二进制同一参数，口径等价。
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`；向量 `[x,y,z]` 数组形态）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），两进程均首次轮询即就绪；
  进程 1 横幅 `[CONFIG] game demo | count=1000 seed=20260926 max_speed=9 bench_secs=0 vsync=on(AutoVsync)`（`run1-game.log`）
- 原始响应文件：本目录 `ts-02/` 下 `a1-*.json` / `a2-*.json` / `a3-*.json` / `run{1,2}-game.log` / `compare-origins.mjs`（判定脚本）与各 cargo 日志

## 门禁（一次通过口径的关键测量点）

- **首次 `cargo check --workspace`（改码完成后）：REAL_EXIT=0**（`ts-02/cargo-check-first.exit`：21:11:44 起 2s，
  增量缓存；覆盖全部功能性改动 sim.rs/cli.rs/main.rs/fps-baseline.md）。
- 补充门禁（如实记录时序）：①首次 check 后有一处**纯空行**编辑落在后台测试运行期间，为消除
  「产物与源码不一致」的不确定性，以最终源码重跑 check + `cargo test -p game`（9/9）+
  `cargo build --release`，全 REAL_EXIT=0——**断言所用二进制即此产物**；②断言完成后补加 serde 兼容单测
  （test-only + dev-dependency serde_json，不进 release 构建），再跑 check/test/build 全 REAL_EXIT=0
  （10/10，`cargo-check-post-test.log` / `cargo-test-game-final2.log`）；`cargo test --doc -p docs` 2/2。
- 单测新增两条（`game/src/sim.rs`）：`max_speed_only_scales_speed_draw_order_untouched`
  （0.5/3.0/9.0/25.0 四档 ×128 实体：origin/phase 逐位一致、模长按比例缩放）与
  `simconfig_deserializes_legacy_json_with_default_max_speed`（旧 JSON 无 max_speed 字段 → 读入 3.0）。

## 断言 1：`world.get_resources` SimConfig.max_speed == 9.0 —— PASS

命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":201,"params":{"resource":"game::sim::SimConfig"}}'
```

响应原文（全量，`ts-02/a1-get-simconfig-run1.raw.json`）：

```json
{"jsonrpc":"2.0","id":201,"result":{"value":{"bench_secs":0.0,"entity_count":1000,"max_speed":9.0,"paused":false,"seed":20260926}}}
```

判定：`max_speed == 9.0`（且 entity_count/seed/paused 与启动参数一致）。**通过**。

## 断言 2：`world.query` Velocity 模长 ∈ (0, 9.0]，且存在 > 3.0 —— PASS

命令原文：

```bash
curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":202,"params":{"data":{"components":["game::sim::Velocity"]}}}'
```

响应摘录（全量 1000 行见 `ts-02/a2-query-velocity-run1.raw.json`；`result` 数组，向量 `[x,y,z]`）：

```json
row0: {"components":{"game::sim::Velocity":{"linear":[5.455456256866455,0,1.9965593814849854]}},"entity":...}
```

判定（node 脚本 `ts-02/compare-origins.mjs --speeds a2-query-velocity-run1.raw.json 9.0` 输出原文）：

```json
{"sampled":1000,"cap":9,"all_in_open_interval":true,"violations":0,"count_over_default_cap":675,"min":0.0024054050342493336,"max":8.998622528461944}
```

采样 1000（≥100）个实体：全部模长 ∈ (0, 9.0]（min>0、max≤9、违例 0）；其中 **675 个 > 3.0**（≥1）。**通过**。

## 断言 3：`--max-speed 9.0` 与默认 `3.0` 同 seed 同 count，`Wanderer.origin` 逐位一致 —— PASS

进程 1（9.0）侧命令与响应文件：`id:203` query Wanderer → `ts-02/a3-query-wanderer-run1.raw.json`（1000 行）。
杀净进程 1 后以**同 seed 同 count、不传 `--max-speed`** 二次启动（进程 2，对照测量，清单本身要求的第二口径）：
横幅 `max_speed=3`、`world.get_resources`（`id:301`，`ts-02/a3-get-simconfig-run2.raw.json`）：

```json
{"jsonrpc":"2.0","id":301,"result":{"value":{"bench_secs":0.0,"entity_count":1000,"max_speed":3.0,"paused":false,"seed":20260926}}}
```

（默认值 3.0 回读证实。）进程 2 侧 query Wanderer（`id:302`）→ `ts-02/a3-query-wanderer-run2.raw.json`。

判定（按 `Wanderer.index` 对齐后逐位比较，脚本输出原文）：

```json
{"compared":1000,"mismatches":0,"firstMismatch":null}
```

1000/1000 `origin` 逐位一致（serde_json 为 f32 最短往返表示，JSON 文本相等 ⇔ 位模式相等）；两响应的
`result` 数组整体 JSON 文本亦相等（`cmp` 仅 `id` 字段不同）。index 0 的
`origin=[59.20360565185547,0.800000011920929,40.68254470825195]` 与 **TS-01 存档值逐位相同**
（`ts-01-brp.md` 断言 1 首行）——重构不仅保住跨 max_speed 一致，也未改变改动前的老口径初值。**通过**。

## 汇总与口径说明

- 结论：**3/3 断言首次执行全部通过，一次通过，返工 0**（进程 2 是断言 3 清单要求的对照口径运行，非重跑）。
- 测量过程如实记录的两点工具级偏差（均非游戏代码返工）：①证据脚本 `compare-origins.mjs` 首版取数
  写成裸数组（实际响应为完整 JSON-RPC 包），`rows.map is not a function` 后修脚本取 `.result`——判定脚本
  修正，断言本身首次数据即合格；②Git Bash 下 `taskkill /F /IM` 被路径转义吞参，改 `MSYS_NO_PATHCONV=1`
  前缀后成功（TS-01 未记录此环境细节，本次留痕）。
- 游戏收尾：两进程均 `taskkill /F` 终止并 `tasklist` 确认无 `game.exe` 残留；run 日志 REAL_EXIT=1 为
  强杀预期退出码（非游戏故障）。
- 与清单口径的偏差：**无断言放宽**。一处实现口径说明——任务约束「新字段需带 `#[serde(default)]` 兼容旧序列化」
  实现为 `#[serde(default = "default_max_speed")]`（serde default 属性的带路径形态，缺字段默认 **3.0** 而非
  裸 default 的 0.0），依据是同句约束「默认 3.0，保持现口径不变」；单测锁定该行为（旧 JSON → 3.0）。
  启动形态以 `cargo build --release` + 直跑 exe 等价执行清单的 `cargo run --release`（见文首注记）。

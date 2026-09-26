# TS-12 断言证据：批量实体操作一致性（BRP 原文留痕）

- 任务：`assets-methodology/taskset/ts-12-batch-ops.md`（4 条断言，逐条执行）
- 判定者/日期：执行 agent，2026-09-26
- 被测对象：`game/` demo，本次**零改码核对型任务**（任务需求即「不改动代码」）
- 启动命令（与清单一致，等价形态）：清单写 `cargo run --release -p game -- --count 100 --seed 20260926`；
  实际执行 `cargo build --release -p game`（REAL_EXIT=0）后直接运行同一产物
  `./target/release/game.exe --count 100 --seed 20260926`（后台，PID 23924）——与
  `cargo run --release` 构建并运行的是同一二进制同一参数，口径等价（TS-01~11 同款注记）。横幅：
  `[CONFIG] game demo | count=100 seed=20260926 max_speed=3 bench_secs=0 vsync=on(AutoVsync)`（`ts-12/run1-game.log`）
- BRP 端点：`http://127.0.0.1:15702`（JSON-RPC 2.0 over HTTP POST，均含 `jsonrpc` 与 `id`；向量 `[x,y,z]` 数组形态）
- 就绪探测：轮询 `rpc.discover`（非清单断言，仅通路确认），1 次即就绪（`ts-12/ready-discover.json`，
  23 方法清单与 SKILL.md §6.2 一致）
- 原始文件：本目录 `ts-12/` 下各 `a*.raw.json` / `loop*.raw.json`（30 个）/ `tick-*.raw.json`（3 个）/
  `loop-summary.txt` / `tick-timing.txt` / `run1-game.log` / `run1-game.exit` /
  `cargo-check.*` / `cargo-build-release.*`
- 断言用响应字段均以 node 严格 `===` / `error === undefined` 判定（计数、entity 比较无浮点歧义）
- 全程**单一游戏进程无重启**（PID 23924），10 次成对操作连续执行，循环总墙钟 ≈2.65s（`tick-timing.txt`）

## 门禁（公共前置）

- `cargo check --workspace`：**REAL_EXIT=0**（`ts-12/cargo-check.exit`；增量 0.52s；零改码任务仍按公共前置执行）
- `cargo build --release -p game`：REAL_EXIT=0（`ts-12/cargo-build-release.exit`；断言所用即此产物，源码零变更）

## 请求形态查证（写断言前，非事后）

- `world.query` 参数结构 `BrpQueryParams { data: BrpQuery, filter: BrpQueryFilter, strict }`、
  `BrpQuery.components: Vec<String>`（`bevy_remote-0.19.1/src/builtin_methods.rs:383-387`）、
  `BrpQueryFilter.without: Vec<String>`（`:409-418`）——与 TS-01/05 已核实形态一致。
- `world.despawn_entity` 参数 `BrpDespawnEntityParams { entity: Entity }`（`:186-191`），成功返回
  `Ok(Value::Null)` 即 `result: null`（`:1334-1344`）——TS-06 已核实。
- `world.spawn_entity` 参数 `BrpSpawnEntityParams { components: HashMap<String, Value> }`（`:172-177`），
  响应 `result.entity` 为 u64 位串数字，Vec3 以 `[x,y,z]` 数组传入（glam serde）——TS-05 已核实。
- `world.get_resources` 参数 `{ resource }`，单资源响应形态 `result.value.<字段>`——本次落笔前直查
  `ts-11/a1-simstats.raw.json` 存档确认（吸取 T012「误按 result[类型名] 写表达式」的工具级偏差教训，
  本次一次写对）。
- 清单写法与源码全部一致，**未放宽**。

## 断言 1：基线 `w0 == 100`、`v0 == 0`（`world.query` ×2）—— PASS

命令原文（其一，Wanderer 全量计数）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":1201,"params":{"data":{"components":["game::sim::Wanderer"]}}}'
```

命令原文（其二，Velocity + `without` Wanderer 筛选计数）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":1202,"params":{"data":{"components":["game::sim::Velocity"]},"filter":{"without":["game::sim::Wanderer"]}}}'
```

响应原文摘录（其一 `result` 数组首行，全量 100 行见 `ts-12/a1-query-wanderer-baseline.raw.json`）：

```json
{"components":{"game::sim::Wanderer":{"index":0,"origin":[59.20360565185547,0.800000011920929,40.68254470825195],"phase":0.6825083494186401}},"entity":4294966882}
```

响应原文（其二全量，`ts-12/a2-query-velocity-without-wanderer-baseline.raw.json`）：

```json
{"jsonrpc":"2.0","id":1202,"result":[]}
```

判定：两个响应均无 `error`；行数 **100** 与 **0**——**w0 = 100、v0 = 0**（100 个 Wanderer 各带
Velocity 被 `without` 全部滤除，初始无「有 Velocity 无 Wanderer」实体）。**通过**。

## 断言 2：循环 10 次 query → despawn → spawn，每次操作均无 error —— PASS

循环脚本核心（单进程连续执行，id 按迭代递增；`E` 为当次 query 首行实体，数字原样回填）：

```bash
for i in $(seq 1 10); do
  curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"world.query","id":'$((1220+i))',"params":{"data":{"components":["game::sim::Wanderer"]}}}' -o loop${i}-query.raw.json
  E=$(node -e "console.log(require('./loop${i}-query.raw.json').result[0].entity)")
  curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"world.despawn_entity","id":'$((1240+i))',"params":{"entity":'${E}'}}' -o loop${i}-despawn.raw.json
  curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"world.spawn_entity","id":'$((1260+i))',"params":{"components":{"game::sim::Velocity":{"linear":[1.0,0.0,2.0]}}}}' -o loop${i}-spawn.raw.json
done
```

迭代 1 的三个响应原文（`ts-12/loop1-*.raw.json`；其余 9 组同形态，逐文件核验）：

```json
{"components":{"game::sim::Wanderer":{"index":0,"origin":[59.20360565185547,0.800000011920929,40.68254470825195],"phase":0.6825083494186401}},"entity":4294966882}
{"jsonrpc":"2.0","id":1241,"result":null}
{"jsonrpc":"2.0","id":1261,"result":{"entity":4294966775}}
```

（依次为 query 首行 / despawn 全量 `result:null` / spawn 全量返回新实体号。）

10 次迭代明细（despawn 目标与 spawn 产物，`ts-12/loop-summary.txt` 与各 raw 核验一致）：

| iter | despawn 目标（query 首行实体） | spawn 新实体 |
|---|---|---|
| 1 | 4294966882 | 4294966775 |
| 2 | 4294966873 | 4294966774 |
| 3 | 4294966874 | 4294966773 |
| 4 | 4294966875 | 4294966772 |
| 5 | 4294966876 | 4294966771 |
| 6 | 4294966877 | 4294966770 |
| 7 | 4294966878 | 4294966769 |
| 8 | 4294966879 | 4294966768 |
| 9 | 4294966880 | 4294966767 |
| 10 | 4294966881 | 4294966766 |

判定：**10 × 3 = 30 个响应逐文件以 `error === undefined && result !== undefined` 核验，全部无
error**（despawn 均 `result:null`、spawn 均返回有效实体号、query 均返回非空行）。**通过**。

## 断言 3：终态 `w1 == 90`、`v1 == 10`（净变化精确一致）—— PASS

命令原文：同断言 1（id 1281 / 1282，`ts-12/a3-query-wanderer-final.raw.json` /
`ts-12/a4-query-velocity-without-wanderer-final.raw.json`）。

响应原文摘录（a3 首行 / a4 首行）：

```json
{"components":{"game::sim::Wanderer":{"index":99,"origin":[-51.02488708496094,0.800000011920929,30.711864471435547],"phase":3.308243989944458}},"entity":4294966783}
{"components":{"game::sim::Velocity":{"linear":[1,0,2]}},"entity":4294966775}
```

判定：两响应均无 `error`；行数 **90** 与 **10**——**w1 = 90、v1 = 10**（w：100→90 净 −10；
v：0→10 净 +10，与 10 次成对操作精确一致）。自洽侧证（非清单要求，node 全量扫描）：
① `v1` 的 10 个实体号集合与断言 2 的 10 个 spawn 产物**逐一相等**；② 10 个 despawn 目标实体
均不出现在 `w1` 行内；③ `w1` 的 90 个 `Wanderer.index` 恰为 10..99（被销毁的 10 个恰为
index 0..9——TS-09 起这 10 个实体带 `Tagged` 组件构成独立 archetype，本次查询迭代序其首，
清单只要求「取一个 Wanderer」，该取法不影响判定）。**通过**。

## 断言 4：操作期间 `tick` 持续增长（批量操作不阻塞模拟）—— PASS

命令原文（三点采样：循环前 / 第 5 次迭代后 / 循环后，id 1210 / 1211 / 1212）：

```bash
curl -s --max-time 30 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.get_resources","id":1210,"params":{"resource":"game::sim::SimStats"}}'
```

响应原文（`ts-12/tick-{t0,mid,t1}.raw.json`，各取 `result.value` 字段组）：

```json
{"tick":5754,"frame_count":5754,"elapsed_secs":95.95915523311123}   /* t0：循环前 */
{"tick":5834,"frame_count":5834,"elapsed_secs":97.29235523799434}   /* mid：第 5 次迭代后 */
{"tick":5913,"frame_count":5913,"elapsed_secs":98.60896793799475}   /* t1：循环后 */
```

判定：**5754 < 5834 < 5913 严格递增**（两段增量 +80 / +79 帧，对应墙钟 1.339s / 1.316s
（`tick-timing.txt` epoch 1790434983.99 → 1790434985.33 → 1790434986.65），速率 ≈60 tick/s 与
vsync 锁帧一致；`elapsed_secs` 同步推进）——批量 BRP 写操作期间模拟持续运行未被阻塞。**通过**。

## 汇总与口径说明

- 结论：**4/4 断言首次执行全部通过；一次通过，返工 0；零改码**（`cargo check` 仍按公共前置执行且
  REAL_EXIT=0）。
- 与清单口径的偏差：**无**。断言未放宽；启动命令、请求形态（`entity` 数字原样回填、Vec3 `[x,y,z]`
  数组、`filter.without`）、判定值（100/0 → 90/10、tick 严格递增）均按清单原文执行。
- 游戏收尾：`taskkill /F /IM game.exe`（PID 23924）终止，`tasklist | grep game.exe` 确认无残留；
  `run1-game.exit` REAL_EXIT=1 为强杀预期退出码（非游戏故障，TS-01~11 同款注记）。
- 无代码/行为/接口变更，无文档同步义务触发；零踩坑（四个方法的请求形态均沿用本仓库已核实结论 +
  落笔前直查 `ts-11` 存档确认 `get_resources` 响应形态，一次写对）。

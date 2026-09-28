# TS-13《关卡加载 game.launch_level》BRP 验证实录（第一阶段脚本判定）

- 日期：2026-09-28。执行：agent（台账 T032）。
- 构建/启动：`cargo build --release -p game` REAL_EXIT=0；`./target/release/game.exe --count 0 --seed 20260926`（后台常驻，PIT-M-007 形态；`--count 0` 自本任务放开——原 CLI 校验 `1..=200000` 拒 0，M4 关卡域需要空 demo 场，见「过程偏差」P1）。
- BRP 端点：`http://127.0.0.1:15702`（显式回环绑定，run1-game.log `[BRP]` 行）。进程收尾：`MSYS_NO_PATHCONV=1 taskkill /F /PID <pid>` 成功，tasklist 无残留（run1-game-kill.log；后台任务退出码 1 = 强杀预期，沿 T006+ 口径）。
- 判定：`node assert-ts13.js` 逐条断言 **6/6 全 PASS，退出码 0**（assert-ts13-out.txt）。业务键口径 `(team,x,y)` 排序后逐位比对，不依赖迭代序与实体号（PIT-B-010）。

## 断言与原文摘录

启动方式与清单见 `assets-methodology/taskset/ts-13-launch-level.md`。请求均带 `jsonrpc` 字段（T002 教训）。

### 断言 1：启动基线 Unit==0 —— PASS

```bash
curl -s -m 10 http://127.0.0.1:15702 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1301,"method":"world.query","params":{"data":{"components":["game::level::Unit"]}}}'
```

响应 `result` 为空数组 `[]`（a1-query-baseline.raw.json）——启动无默认关卡。

### 断言 2：launch(1) 响应 —— PASS

`game.launch_level {"level":1}` → `result`（serde_json 键序为字母序）：

```json
{"height":9,"level":1,"name":"first-contact","units_spawned":6,"width":9}
```

与期望逐字段判等（键序无关）。

### 断言 3：关卡 1 布阵 6 行逐位 —— PASS

`world.query` `["game::level::GridPos","game::level::Unit"]` → 6 行；按 `(team,x,y)` 排序后 `hp/move_range/attack` 逐位等于 `LEVEL_1` 定义（a3-query-l1.raw.json 全量，比对脚本内嵌定义同源常量表）。

### 断言 4：重复 launch(1) 幂等 —— PASS

连续两次 `{"level":1}`（id 1304/1305）均 `units_spawned:6`；再 query 仍 6 行、业务键集合与断言 3 逐位一致；实体号 `…865..870` → `…852..857`（清场后重生，实体号变化属预期，断言不依赖实体号）。

### 断言 5：launch(2) 清场重载 6→9 —— PASS

`{"level":2}` → `units_spawned:9`；query 9 行业务键集合逐位等于 `LEVEL_2` 定义（旧 6 单位被清）。

### 断言 6：错误路径 -32602 + 无副作用 + 进程存活 —— PASS

`{"level":99}` → `{"error":{"code":-32602,"message":"未知关卡 99；可用：[1, 2]"}}`；随后 `world.get_resources {"resource":"game::level::LevelState"}` → `{"value":{"height":9,"level":2,"units_spawned":9,"width":9}}`（仍关卡 2 态，错误调用无副作用）；再 query Unit 仍 9 行——**进程存活，自研方法错误路径未击穿进程**（备忘录 A 案可玩性定义验收面）。

### 佐证：rpc.discover 收录

`rpc.discover` 共 **27** 方法（0.19.1 内置 23 + 自研 4），含 `{"name":"game.launch_level","params":[]}`（a7-discover.raw.json；discover 不描述自定义方法参数，沿 T014 对 registry.schema 的同款观察）。

## 过程偏差（如实记录，均非游戏侧缺陷）

- **P1（代码变更，非偏差）**：首次启动 `--count 0` 被 CLI 拒（`run1-game.log` 首启形态：`[CLI] --count 须为 1..=200000 的整数，收到 0`，退出码 2）——顺势放开下界至 0（`game/src/cli.rs`，M4 合法配置），重建后二启成功。该轮按台账口径计 T032 一次返工（首次启动未达任务预期）。
- **P2（驱动侧笔误）**：断言 6 的 `get_resources` 首发误写复数形态 `{"resources":[...]}` → `-32602 missing field \`resource\``（原文留档 `a6b-wrong-params-driver-error.raw.json`）——违反「请求形态先查存档再发」纪律（ts-03-brp.md 既有正确形态），修正为单数 `{"resource":"..."}` 后一次成功。属驱动侧返工，不改判游戏侧结论。
- **P3（脚本笔误）**：断言脚本首版 #2 用 `JSON.stringify` 直比，被 serde_json 字母序键序绊倒（FAIL 为假阴性），改键集合 + 逐字段判等；#6 的 `result.value` 形态随 P2 修正后落定。两处均在首版 `assert-ts13-out.txt` 留 FAIL 痕迹后修正复跑。
- **P4（工具级）**：`taskkill //F` 与 `MSYS_NO_PATHCONV=1` 并用会以字面 `//F` 传入报「无效参数」（run1-game-kill.log 两轮原文）——PIT-M-005 的 `//F` 转义与该环境变量**不叠加使用**：设了 `MSYS_NO_PATHCONV=1` 就用单斜杠 `/F /PID`。成功形态已留档。

## 证据文件

`docs/evidence/ts-13/`：

- `run1-game.log`：二启形态（`[BRP]` 两行注册日志，含 `game.launch_level`）。**勘误注（2026-09-28 审核轮）**：本 md 初版曾称其「含首启拒绝前的 CLI 行」——不实，首启失败日志已被二启同路径覆盖，首启拒绝原文仅存 P1 引文（其文本可经提交前版本 `game/src/cli.rs` 校验口径复核）；未来失败输出独立命名归档，沿本注纪律。
- `.raw.json` ×13：a1..a6c 断言输入 11 份 + a7 discover 佐证 1 份 + a6b-wrong-params 驱动错误形态 1 份（初版清单「11 raw + 1 错误形态」计数不确，本行修正）。
- `assert-ts13.js` + `assert-ts13-out.txt`：判定 6/6 PASS 退出码 0。首版两处 FAIL 输出已被复跑覆盖、无独立归档件（P3 如实记；经审核独立复跑 6/6 复现）。
- `run1-game-kill.log`：两轮 `//F` 失败原文 + 成功轮。
- 门禁日志：`cargo-check.log` / `cargo-test-game.log`（11 passed）/ `gate-doc-test-final.log`（77/0/17）为审核前轮；审核修复后复跑 `gate-check-final2.log` / `cargo-test-game-final2.log`（11 passed）/ `gate-doc-test-final2.log`（77/0/17）。release 构建无独立日志（由 release 二进制运行与 run1-game.log 间接佐证，如实记）。

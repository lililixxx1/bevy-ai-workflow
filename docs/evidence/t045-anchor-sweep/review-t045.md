# G2 · T045 轻量轮审核报告（plan-code-reviewer，2026-10-30 晚）

> 审核对象：树 `trees/t045/w1` T045 批（game/src 0.19→0.20 注锚清扫 14 文件 + 证据档四件）。
> 审核员独立 diff（对主仓基线 d035230）、逐条核对门禁断言、registry 源码锚点抽查复跑。
> 以下为审核员原始报告（五字段），Lead 收获时归档。

## 报告原文

**[结论] 通过**

**[已核对]**

1. diff 审阅（14 文件，树 vs 主仓基线 d035230）：`diff --strip-trailing-cr` 后内容差异仅 33 对（33 `<` / 33 `>`），逐行核完——其中 31 对为 `0.19.1→0.20.0` 版本注锚交换，2 对为纯行号漂移（`world_mut.rs:181→182`、`world/mod.rs:2287→2330`）；`grep '^[<>]'` 非 `//`/`///`/`//!` 行 = 0，即**零代码语义改动**。初看整文件差异系行尾符所致（抽查 `game/src/sim.rs` 主仓 LF vs 树 CRLF），`--strip-trailing-cr` 消除后复核通过。
2. `grep "0\.19\.1" game/src` 零残留：独立复跑 `grep -rnF "0.19.1" <树>/game/src | wc -l` = 0。交叉核对 README §1.2：14 文件 `grep -rc "0.20.0"` 分文件计数逐文件一致；余量 7 行归属派工单外 3 文件（`bench.rs:3`/`launch_level.rs:1`/`screenshot.rs:3`），41 行 / 42 处口径自洽。
3. check.log：首行命令含 `-j 3`，`REAL_EXIT=0`，`error`/`warning` 行 = 0，含 `Checking game v0.1.0` + `Finished ... 3.08s`。通过。
4. test.log：首行命令含 `-j 3 -- --test-threads=3`，`REAL_EXIT=0`（193 行档）；10 个 `test result` 段机器求和 `113/0/17`（awk 独立计算一致），即 game 20/20、mcp_bridge lib 15/15、Doc-tests docs 78 passed / 17 ignored；`error` 形态行 = 0。档-缓存时序与 README §2 取证注记一字相符。通过。
5. README 结构：§1–§5 五节齐（清扫统计 / 门禁 / 零登记 / 两段死亡留痕 / 可公开态）；§4 两次实例死亡事实（10:45 整机冻结 / 11:09 停工）如实。通过。
6. 可公开态：字面量扫描 `C:/`、`C:\`、`D:/`、`D:\` 四档全 0；唯一 `:\` 命中是 README:78 文档化正则示例内转义符，非机器路径。通过。
7. g1-lead.log：U1（范围外 `cargo clean -p game`）接受·已披露偏离；U2（test.log 先于 clean）裁 A 采信；与 README §2 披露行完全对应。通过。
8. 抽查复跑（本轮零 cargo 调用，只读 grep/sed）：registry 逐点核对——bevy_remote `:327/:331/:333` 与 `BrpTriggerEventParams` 一致；`bevy_ecs-0.20.0/src/reflect/mod.rs:36` = `pub struct AppTypeRegistry`；`type_registry.rs:448` = `pub fn get(&self, type_id: TypeId)`；http.rs `:167` = `with_address`、`:52` = `DEFAULT_PORT=15702`、`:60` = `DEFAULT_ADDR=127.0.0.1`；`lib.rs:591` = `with_method_main`；两处纯行号漂移实证正确。抽样 5+2 全命中。

**[阻塞项] 无**

**[最小修复指令] 无**（附带观察，非修复项：① 14 改动文件行尾 CRLF vs 主仓 LF——收获时已由 Lead `sed` 归一 LF，diff 复核 33+/33− 纯注释面；② `ts06.rs:16-17` 的 `world/mod.rs:951` 注锚（既有写法、非本卡 31 处改动面）实际落在 `get_entity_mut` 文档区而非 `get_entity` 签名行（934-937），语义描述仍真，登记遗留后续随手收紧，不阻塞。）

**[复验命令]**

- `grep -rnF "0.19.1" <树>/game/src | wc -l`（期望 0；审核轮已执行 = 0）
- `diff --strip-trailing-cr <主仓>/game/src/sim.rs <树>/game/src/sim.rs`（期望仅 `///` 注锚行差异；已执行通过）

## Lead 处置注记

- 观察①（CRLF）：收获拷贝时统一 `sed -i 's/\r$//'` 归一，git diff 终态 14 文件 33+/33−、语义守卫（非注释改动行）为空。
- 观察②（ts06 锚偏松）：登记遗留，随下次触碰 game/src 的卡片顺手收紧（≤2 行改动）。

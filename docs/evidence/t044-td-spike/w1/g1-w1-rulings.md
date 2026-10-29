# T044-W1 批 L1 裁决（Lead；2026-10-29）

worker 上报 5 项 + Lead 自查 1 项，裁决如下（编号 W1-R1..R6；L1 权限内，无 L2 上报项）。

## W1-R1：screenshot.rs 行号漂移（G2 前归因勘误：出处非 SKILL §6.8）

**事实（修正后）**：worker 上报「SKILL §6.8 记 `clear_screenshots` `:197`、First 注册 `:416-418`」——Lead G1 复核发现**归因有误**：主仓 SKILL §6.8 现行锚（`:53`/`:145`）已是 0.20 正锚，不带该两项；实际携带 0.19 锚的是主仓 `game/src/rpc/screenshot.rs:21` 模块注释（T042 迁移漏扫面）。**漂移本身真实**（0.20 实况 `:255` / `:473-479`，Lead 独立复核命中）。顺藤清扫出同类残留：game/src 内 0.19 注锚共约 33 处。
**裁决**：worker 不改 SKILL（正确）。Lead 收尾批**已执行**：① `game/src/rpc/screenshot.rs` 全块重锚 + 同类清扫 8 处（bench.rs×2 / sim.rs observer / launch_level.rs despawn / main.rs window / cli.rs 运行时横幅 0.19→0.20 失实勘误，全部对 0.20 源码逐锚核证）；② 余量 ~25 处立 T045 清扫卡；③ SKILL §6.8 无需改动。G2 对本裁决无异议并附 S1/S2 两个注释级锚点偏差（map.rs:125→:126、居中算式 prepare.rs:68→anchor.rs:43+prepare.rs:161、材质注入 :124-130→:122+:127-130），随批已修（树内 main.rs + 主仓 SKILL §6.16 同步）。

## W1-R2：claim-lint 在隔离树内假红（tree mtime 平坦化）

**事实**：`git archive` 导出树所有文件 mtime 相同 → claim-lint S1 回退式按 mtime 降序
挑 doctest 日志，选中最旧 77/0/17（expect=78/0/17）→ doctest-gate 假红（12/13 PASS）。
**裁决**：mtime 依赖的门禁（claim-lint 全量、S1 回退路径）**只在主仓跑**，入
PROJECT-APPENDIX「隔离树注意事项」。worker 的 supplementary mtime 日志保留为现象证据
（属主列 Administrator 的公开前例问题见 R6）。树内 12/13 不计返工（门禁环境性假红，
非产物缺陷——主仓复跑全绿为证）。

## W1-R3：tilemap 默认纹理数组路径接受（设备可移植性挂账）

**事实**：非 atlas 默认路径走 299 层纹理数组（23×13），需请求 adapter limits；
本机 RTX 3050/Vulkan 实测可用；受限设备有 `atlas` feature 备选（未接线）。
**裁决**：**接受默认路径**。硬约束 5 = 平台仅 PC（Windows），本仓工作流面向本机
与同类独显 PC；P3 实测 + 17/17 断言即本机证据。「受限核显设备可移植性」登记为
立项批考虑项（若 owner 要求核显可达，届时加 atlas 接线卡，非 spike 范围）。

## W1-R4：P2 图集索引不可经 BRP 查询（断言降级接受）

**事实**：P2 阶段瓦片索引存于 Sprite 的 `TextureAtlas`（`Arc<StrongHandle>` 字段
无 ReflectSerialize）→ world.query 返回 -32602。P2 断言降级为几何+标记（150/45/
坐标覆盖），索引级由 P3 tilemap 组件（`TileTextureIndex` 可反射）逐格钉死。
**裁决**：**接受降级**（覆盖等效：P3 的 bad=0 逐格断言 + P2/P3 像素级等价双向印证
索引正确）。「BRP 无法序列化 strong-handle 字段」登记 **PIT-B 候选**（方法论级：
反射序列化面与非反射句柄字段的查询盲区）入回灌清单，随 T043 recheck-registry
同类面下次统一处理。

## W1-R5：安全阀 51 行 Compiling 未触发中止（裁决：worker 判断正确）

**事实**：P2 增量构建 3m10s 内出现 51 行 `Compiling bevy_*`——附录 A 安全阀针对
**冷编风暴**（全量重编，内存/磁盘风险），此为 game-td 新增依赖的**代码生成增量**
（bevy_ecs_tilemap + 其 feature 面首编），自我完备、必然终止；worker 继续完成正确。
**裁决**：**接受**。附录 A 阀值条款补定量边界：短时自完备增量（如新增依赖首编、
无 `error: linking`、无磁盘击穿）不属冷编风暴；判据 = 趋势（持续新增 Compiling 且
不见 Finished）而非行数绝对值。收尾批 Lead 改附录。

## W1-R6（Lead 自查）：可公开态三命中定性

`rsproxy-sparse` 别名（cargo 标准输出）、`Administrator 197121` 属主列
（m1-phase2 公开日志同前例）、self-check.mjs 构造式拆分（PAT-M-010 合规）。
**裁决**：全部接受，无需脱敏（内置账户名与 registry 别名无识别力，主仓公开面已有
同类前例；机器绝对路径连续字面为零）。

## 汇总

| # | 事项 | 裁决 | 后续动作归 |
|---|---|---|---|
| R1 | SKILL §6.8 锚点漂移 | 收尾批重锚 | Lead（closeout） |
| R2 | 树内 claim-lint 假红 | 主仓跑 + 附录登记 | Lead（closeout） |
| R3 | 纹理数组默认路径 | 接受；可移植性挂立项批 | 立项批考虑项 |
| R4 | P2 索引查询盲区 | 断言降级接受；PIT-B 候选 | 回灌清单 |
| R5 | 安全阀边界 | 接受；补定量边界 | Lead（closeout 改附录 A） |
| R6 | 可公开态三命中 | 接受 | 无 |

无 L2 上报项。G1 通过，转 G2 完整轮（节点卡：新工程 + 运行时证据）。

## 附：G2 完整轮结果（2026-10-30 收到）

**通过——0 必改 + 0 阻塞 + 5 建议（全部随收尾批落实）**：S1 居中算式锚（prepare.rs:68 → anchor.rs:43 本体 + prepare.rs:161 使用点）、S2 map.rs:125→126 + 材质注入 :122+:127-130、S3 README 终态复跑耗时 0.56s/1.79s→0.83s/0.99s、S4 树根 Cargo.toml CRLF 收获时归一 LF、S5 supplementary 日志 mojibake 加注记。16 项独立重算全部与宣称一致（含 md5 三证确定性渲染、Cargo.lock name+version 联合零漂移、教程行为等价实现独立抽核）；G1 六裁决零异议。审核覆盖与未覆盖面均诚实声明在案（报告全文见主仓证据档 g2-w1-report.md，收获时随批入档）。

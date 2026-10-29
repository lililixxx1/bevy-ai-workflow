# T044-W1 批 G2 完整轮审核报告（plan-code-reviewer，2026-10-30 收到；Lead 落档）

## 判定：通过（0 必改 / 0 阻塞 / 5 建议）

隔离树产物（`game-td` 新工程 + 运行时证据）经独立重算全部自洽：代码面 API 锚点逐条命中
0.20 本地源码、BRP 显式回环绑定、清单增量恰为 +2 包且 bevy 无漂移、证据 PASS 数/SUMMARY/
discover/PNG/REAL_EXIT 全部重算一致、G1 六项裁决无失当。

## 建议项（非阻塞，收获前处理——全部已随收尾批落实）

- **S1** `main.rs` 居中算式锚失准：`render/prepare.rs:68` 实为查询参数区；居中公式本体
  `src/anchor.rs:43`（`as_offset`）、使用点 `render/prepare.rs:161`。→ 已修（树 main.rs +
  主仓 SKILL §6.16 同步）。
- **S2** 同文件两处行号偏差：`TilemapTexture::Single` `map.rs:125`→实 `:126`；默认材质注入
  `render/mod.rs:124-130`→实 `:122`（add_plugins）+ `:127-130`（insert）。→ 已修（同 S1 双面）。
- **S3** README §2 终态复跑耗时 0.56s/1.79s 与日志实测（0.83s/0.99s，timing.md 正确）不符，
  疑旧数残留。→ 已修（README 改 0.83s/0.99s）。
- **S4** 树根 `Cargo.toml` 为 CRLF（基线 LF），内容增量确两行但整文件 diff 噪音。→ 收获时
  归一 LF（已执行）。
- **S5** `claim-lint-supplementary*.log` 中文说明行为 mojibake（编码差），机器判定行
  （PASS/FAIL/REAL_EXIT）完整可解析不影响结论。→ 注记于此，原档保留（乱码属环境编码痕迹，
  与 T039「环境痕迹保留」口径一致）。

## 独立重算表（16 项，全部与宣称一致）

| # | 重算项 | 结果 |
|---|---|---|
| R1 | v4/v5/v6/v6b PASS 行数（grep 独立计数） | 15/16/17/17 = 各档 SUMMARY |
| R2 | discover（node 解析 4 份 JSON） | 均 24 方法、0.20.0、含 td.screenshot、`game.*` 0 命中 |
| R3 | PNG 魔数/IHDR/字节（node 独立解析） | 5 张魔数 ✓、IHDR 1600×900；p1=23348/p2=63592/p3=p3-final=g1=64454 |
| R4 | 确定性渲染（md5sum） | p3/p3-final/g1 同哈希 `fb6798b7…`（逐字节相同，第三次佐证） |
| R5 | Cargo.lock 包集合（base 对比） | 628→630，仅 +bevy_ecs_tilemap/+game-td，零删除 |
| R6 | Cargo.lock 版本漂移（name+version 联合） | 既有包零变化；全文 diff 仅 +22 行；bevy 0.20.0 维持；tilemap checksum fe378ed0… |
| R7 | 根 manifest 增量 | 语义恰两行（members + dependencies） |
| R8 | REAL_EXIT 同档（全 log 扫） | 判定档均 0；run 系 =1 且均带 taskkill 强杀报码行（口径在档） |
| R9 | timing 时间戳自洽 | 各阶段 run/shot mtime 均落声明窗口内，无秒级硬伤 |
| R10 | 可公开态（更宽模式独立扫） | 零新命中；仅 3 处已知形态（R6 裁决内） |
| R11 | 资产合规（sha256+字节） | tilesheet 与侦察源逐字节同、Kenney CC0 原文 |
| R12 | 编译耗时（Finished 行） | 与 timing.md 一致（README 终态两数见 S3） |
| R13 | Compiling 行数 | 53 = 51 bevy_* + 2 tilemap/game-td（口径调和一致） |
| R14 | API 锚点抽查（registry+recon 逐行 grep） | 全部命中；仅 S1/S2 注释级偏差 |
| R15 | `game.` 字面碰撞 | 树源零命中（与 claim-lint 树内 game-methods PASS 互证） |
| R16 | g1 手工查询 | 45 行 x=0..14 全覆盖 y={4,5,6} |

## 对 G1 六裁决：零异议

R1 无异议（附 S1/S2 同类锚点并入同批）；R2–R6 均复核成立、处置恰当（详见各条）。

## 覆盖声明（审核方原文摘录）

已覆盖：game-td 两源文件全文、双 manifest、Cargo.lock 全文 diff、w1 证据目录全部 40 文件、
API 锚点对照 registry 与 recon tilemap 源码、教程 Part 1–3 对照（行为等价、实现独立）。
未覆盖（诚实声明）：未运行 cargo/游戏（按派工单——运行时归 Lead G1 重跑，已留档）；G1 §5
像素级视觉判定未独立复算（核到字节/md5/IHDR 层）；教程未逐行比对；主仓既有代码面不在本批
范围。未审项均不影响「通过」结论。

（全文完——本档为审核方回复的忠实落档，独立重算表数值原样；格式按本仓证据档规范轻度排版。）

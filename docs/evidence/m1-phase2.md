# M1 两阶段判定第二阶段：12 任务验收清单的脚本化重跑校准

- 日期：2026-09-27
- 依据：`docs/m1-acceptance.md` §六（两阶段判定的衔接）——「`tooling/` 的 `run_tests`
  自研件就绪后，应将全部 12 个任务的验收清单转为脚本断言并重跑校准本批数据」；
  意向文档 §10 第 5 条
- 载体：`game.run_tests` 套件（game/src/rpc/suites/，11 个）+ task-runner
  （`run` 两连调 / `pause` / `bench-judge` 三判定模式 + `--expect-*` 字面值绑定）
- 驱动：`docs/evidence/m1-phase2/run-all.sh`（串行逐任务冷启动，进程口径=各清单
  启动方式原文）；原始证据 `docs/evidence/m1-phase2/ts-XX/`（runner transcript +
  游戏日志 + BRP 就绪探测 + taskkill/残留核对 + 退出码），汇总 `summary.txt`
- 台账：T020

## 一、判定架构（三层）

| 层 | 载体 | 覆盖面 |
|---|---|---|
| 进程内套件 | `game.run_tests {"suite":"ts-XX"}`（11 个：01/02/03/05/06/07/08/09/10/11/12） | 只读断言 + 写通路判定语义（ECS 直写：`World::spawn`/`despawn`/`get_mut`/`resource_mut`/`trigger`，与 BRP handler 同一 `&mut World` 通路） |
| 工具侧驱动 | task-runner `run`（两连调比对 `snapshot.tick`）、`pause`（TS-04 全时序）、`bench-judge`（TS-11 日志行） | 跨帧采样断言（tick 递增、暂停冻结/恢复、相机随动生效窗口、[BENCH] 行） |
| 字面值绑定 | `run --expect-count/--expect-seed/--expect-max-speed/--expect-version` | 清单字面值（500/20260926/9.0/0.1.0 等）经 `world.get_resources` 绑定；`--expect-version` 附带 `world.list_resources` 可见性实证 |

**TS-04 无套件**：其清单全部为跨帧时序断言（冻结需跨秒采样），由 `pause` 子命令
全程经 BRP 驱动（mutate_resources 注入暂停/恢复）。

**套件约定（无净副作用）**：套件返回时世界须与调用前等价——只读型（01/02/03/09/11）
天然满足；写通路型（05/06/07/10/12）断言完成即自恢复（探针 despawn / 初值还原 /
事件偶数次触发回初始态）；ts-08 为两连调状态机（首次调用改 `radius=60`，第二次
断言随动并还原 `90`，断言名跨相位稳定）。恢复断言本身入套件
（`world_restored_after_suite`）。

**BRP 反射通路不重复覆盖**：spawn_entity/mutate_components/trigger_event 的 JSON
形态与错误面（-23401 等）属第一阶段已验证面（ts-XX-brp.md），第二阶段以 ECS
直写复测**判定语义**；两者数据面等价（同一 `&mut World`）。

## 二、结果总表

| 任务 | 套件断言（单次调用） | 工具侧 | 字面值 | 判定退出码 | 结论 |
|---|---|---|---|---|---|
| ts-01 | 5 | tick 递增 | count/seed | 0 | 通过 |
| ts-02 | 2 | tick 递增 | count/seed/max-speed | 0 | 通过 |
| ts-03 | 2 | tick 递增 | count/seed/version(+list_resources) | 0 | 通过 |
| ts-04 | —（无套件） | pause 五步时序 | —（清单无字面值断言） | 0 | 通过 |
| ts-05 | 4 | tick 递增 | count/seed | 0（round2） | 通过 |
| ts-06 | 2 | tick 递增 | count/seed | 0 | 通过 |
| ts-07 | 3 | tick 递增 | count/seed | 0 | 通过 |
| ts-08 | 3（两相位） | tick 递增 | count/seed | 0 | 通过 |
| ts-09 | 3 | tick 递增 | count/seed | 0 | 通过 |
| ts-10 | 3 | tick 递增 | count/seed | 0（第二轮） | 通过（首跑见 §四） |
| ts-11 | 2 | tick 递增 + bench-judge 四查 | count/seed | 0 + 0 | 通过 |
| ts-12 | 4 | tick 递增 | count/seed | 0（round2） | 通过 |

- 断言去重计数：**78 条**（套件 33 + 工具侧 45），全部 PASS；粒度与第一阶段
  47 条不同（映射见 §三），不做逐条强制对齐——校准口径为「清单每行语义均有
  脚本化载体且通过」。计数基准：套件断言按断言名去重（各在两次调用中重复执行），
  工具侧按 [PASS] 行（tick/字面值/pause 步骤/bench-judge 四查）。
- **round2 说明**：ts-05 / ts-12 的证据为审计修复后的 round2（见 §四之 2）——
  ts-05 增补基线自洽断言（4 断言）、ts-12 补 Tagged 组件还原；首轮（全绿）退出码
  记录于 `summary.txt`，其 raw 在 round2 归档整理中被覆盖未单独保留，权威证据以
  round2（与最终源码同版本）为准。ts-10 首轮失败 raw 已完整保留（`runner-round1.txt`）。
- 每任务两次 `run_tests` 调用（间隔 1s）均须全过（ts-08 两相位各一次）。
- 进程收尾：每任务 `taskkill //F //IM game.exe` + `tasklist` 残留核对（全部
  `kill_clean=0` / ts-11 自退出无残留）。

## 三、逐任务判定详情（关键实测值）

### ts-01（--count 500 --seed 20260926）

5 断言两连调全过：计数 500==500；config↔metadata 一致且未暂停；tick=235>0
（跨调用递增见工具侧）；Wanderer+Velocity 500/500；**500 实体初值与 seed+max_speed
重抽序列逐位一致（不符 0）**。字面值 500/20260926 绑定 PASS。

### ts-02（--count 1000 --max-speed 9.0）

- `velocity_within_configured_cap`：1000 实体模长全部 ∈ (0, 9]（越界 0，最大实测
  8.9986）；**>默认上限 3.0 的 675 个 ≥ 1**（新上限生效；全量扫描强于清单「采样
  ≥100」）；
- `origin_invariant_to_max_speed`：1000 实体 origin/phase 与默认上限重抽序列逐位
  一致（不符 0）——清单 #3 的跨进程对照口径以进程内重抽等价替换（单一落点
  `sim::draw_initial`）；
- 字面值：count 1000 / seed 20260926 / **max_speed 9.0（|Δ|≤1e-6）**。

### ts-03（--count 250 --seed 7）

- `metadata_reflect_visible`：SimMetadata 在 AppTypeRegistry 且带 ReflectResource
  数据（BRP list_resources 同源判定）；
- `metadata_fields_consistent`：version=="0.1.0"==CARGO_PKG_VERSION、
  entity_count 250==config、seed 7==config；
- 工具端 BRP 实证：`world.list_resources` 含 `game::sim::SimMetadata`，
  get_resources version=="0.1.0"（字面绑定）。

### ts-04（--count 1000，pause 子命令，五步全过）

1. 运行中：paused=false，tick 237→298（+61）；
2. BRP 注入：`world.mutate_resources` paused=true 响应无 error，复读 paused=true；
3. 冻结：tick 299→299（Δ0 ≤ 容差 2）；
4. Transform 逐位不变：两次全量 query（各 1000 行）按 entity 排序后**逐位一致**
   （强于清单「任一实体」）；
5. 恢复：paused=false，tick 301→362（+61）。

### ts-05（--count 100，round2＝审计修复后最终证据）

`wanderer_baseline_matches_config`：Wanderer 计数 100 == entity_count（清单 #1
进程内自洽，与 ts-12 对称；字面 100 工具端绑定——审计 P2-8 补齐）；spawn 实体
524v0 后**同帧回读** linear==[1.0,0.0,2.0] 逐位；Velocity-only 计数 1 == 基线
0+1；探针**无条件** despawn 后计数回 0（`world_restored_after_suite`，失败路径
不留脏——审计 P2-1 修复）。tick 411→473 递增。
（BRP spawn_entity/get_components 通路第一阶段已证，此处 ECS 直写复测判定语义。）

### ts-06（--count 100）

探针（真实 Wanderer 语义，index=u32::MAX 隔离）spawn 后 101==100+1；despawn 返回
true、计数回 100、`get_entity` Err；**重复 despawn 返回 false**（BRP 侧即 -23401
ENTITY_NOT_FOUND，第一阶段已证）+ 终态计数 100==基线（世界无残留）。

### ts-07（--count 100）

- index==0 定位（entity 419v0，origin/linear_v0/phase 记录）；
- 改写 linear→[0,0,0] 受理（同帧回读逐位）+ 还原 v0 受理——即写即还原，无净副作用；
- **`transforms_match_analytic_formula`：100 个实体 Transform 与
  `wanderer_translation(w, v, elapsed_secs)` 复算逐位一致，最大偏差 0e0**
  ——强于第一阶段跨帧采样容差 0.1 的口径：进程内 `update_stats`（写 elapsed）与
  `move_swarm`（用 elapsed 写 Transform）同帧同条件 chain，BRP handler 帧间读到的
  elapsed 正是写 Transform 所用 t，同函数同输入 f32 逐位相等。改写的跨帧生效
  版本属第一阶段测量口径（ts-07-brp.md，err≈0.06≤0.1）。

### ts-08（--count 1000，两连调状态机）

- 相位 A：Camera3d+Transform 恰 1；XZ 距离 90.000000000 ≈ radius 90（|Δ|=0.000e0
  ≤0.1）；radius==90 && height==45（CameraRig::default 字面）→ 改 radius=60；
- 相位 B（第二次调用）：距离 60.000000000 ≈ 60（mutate 随动经 1s 间隔帧生效）；
  受理 radius==60、height==45 → 还原 radius=90（世界回默认）。

### ts-09（--count 100）

Tagged 在 AppTypeRegistry 且带 ReflectComponent（BRP list_components 同源判定，
`no_auto_register` 下显式注册唯一通路）；TypeInfo::Struct 字段表含 `tag` 且类型
String（与 registry.schema 同源）；Tagged(With Wanderer) 恰 10。清单 #4 反证为
实现期干预，第一阶段已执行，不重复。

### ts-10（--count 100）

初始 paused=false → 触发后 **true**（`first_trigger_pauses`）→ 再触发 **false**
（`second_trigger_resumes`，翻转语义 + 套件净零）。工具侧 tick 递增：round2
（通过轮）404→466；round1（失败轮，参数错位但世界语义正确）243→305。首跑失败
记录见 §四。

### ts-11（--count 50000 --bench-secs 10）

- 套件（运行中，warmup 2s 后）：avg_fps=59.695>0 且 fps_1s>0；tick>0 且实体
  50000==启动值；
- `bench-judge`（进程自退出后）：`[BENCH] n=50000 seed=20260926 bench_secs=10.0
  warmup_secs=2.0 measure_secs=8.016 measure_frames=476 avg_fps=59.4`——n 字面
  PASS、warmup 口径 PASS、**双源一致 |59.4−59.695|/59.695=0.49% ≤10%** PASS、
  基线对照 59.4 ≥ 60.0×0.8=48 PASS（vsync 组口径，同机）。

### ts-12（--count 100，round2＝审计修复后最终证据）

基线 w0=100==entity_count、v0=0（含 w0≥10 循环配额前置，`saturating_sub` 防下溢
——审计 P2-2）；10 轮成对操作（spawn 探针 + despawn 剩余最小 index 的真实
Wanderer——确定性取序，强于第一阶段依赖 archetype 迭代序）全部成功；终态
**w1=90==w0−10、v1=10==v0+10**（净变化精确一致）；恢复后 w=100、v=0、
**Tagged=10==t0(10)**（探针清理 + 全组件快照重生含 Tagged——最小 index 恰为
`index<10` 打标实体，五组件版会丢 Tagged 致组件集不等价，审计 P1-3 修复并实测
验证；index/初值/Tagged 逐位还原，实体号允许变化——回归口径禁断言绝对 entity
id）。tick 427→489 递增（工具侧，操作不阻塞模拟）。

## 四、返工记录（如实）

1. **ts-10 首跑 FAIL**（round1 transcript 已存档 `runner-round1.txt`）：套件内两次
   `trigger` 的 `second` 布尔实参交叉错位（首触发传 true、次触发传 false），断言名
   与期望错配——**世界翻转语义自始正确**（false→true→false、净零，round1 详情可证），
   纯套件参数笔误。修复（对调实参）+ cargo check + release 重建后重跑：3/3×2 调用
   全过，REAL_EXIT=0（round2）。
2. 驱动脚本首轮全灭（PIT-M-006，CWD 锚定层级差一）与前台进程托管限制（PIT-M-007）
   见 §五。
3. **审核轮（plan-code-reviewer，有条件通过）必改落实**：
   - P1-1 断言计数分解失准（初写 35+42，实为套件 33 去重 + 工具侧 45 = 78）——
     本文档与台账 T020 同步改正；
   - P1-2 ts-10 段落误引失败轮（round1）tick 值——改引 round2 404→466 并标注轮次；
   - P1-3 ts-12 恢复丢 `Tagged` 组件（组件集维度违反无净副作用契约，同进程连跑
     ts-09 会假性失败）——快照补 `Option<Tagged>` 重生还原 + 恢复断言加
     Tagged 计数核对，round2 实测 Tagged=10==t0；
   - P2 落实：P2-1 ts-05 恢复改无条件 despawn（失败路径不留脏）；P2-2 ts-12
     `saturating_sub` + w0≥10 前置；P2-3 run-all.sh 锚定注释层级改正；P2-4 SKILL
     §6.9 引用补 list_resources handler（`builtin_methods.rs:1414-1424`）；P2-5
     「全量比对强于任一实体」归属改正（属 TS-04；ts-12 的更强点是 10/10 逐轮核验
     + 确定性取序）；P2-6 run-all.sh 加聚合退出码（任一任务失败整体非 0）；P2-7
     新增 `.gitattributes`（`*.sh text eol=lf`，防 autocrlf 破坏重跑入口）；P2-8
     ts-05 补基线自洽断言（与 ts-12 对称）。
   - 修复涉及 game 源码（ts-05/ts-12 套件）→ check/build 全绿后以最终二进制
     round2 复跑两任务（退出码均 0、无残留、raw 已归档）；其余 10 任务证据由
     「仅 ts-05/ts-12 套件行为变更、其余套件代码路径未动」的等价性说明承接。

## 五、驱动口径（run-all.sh 运行纪律）

- **后台任务托管**：驱动必须以长生命周期后台任务运行——前台工具调用会随调用清理
  进程组，其中 `&` 起的游戏子进程数秒内即被回收（PIT-M-007，实测两形态对照）；
- **绝对路径锚定**：脚本以自身真实路径上溯三级定位仓库根（`SCRIPT_DIR=$(cd --
  "$(dirname -- "$0")" && pwd)` + `../../..`），不依赖调用方 CWD；首版按 `$0`
  相对形态推断且只上溯两级，全部产物落错目录（`docs/docs/...` 杂散树已清理，
  最小复现 `pit-m006-repro.txt`，PIT-M-006）；
- **前置残留防护**：每任务启动前 `tasklist` 检查，发现残留先清理（防端口占用导致
  就绪探测假成功）；
- **游戏存活快速失败**：启动 +2s 存活检查，早夭即判失败并带 game.log 末行；
- **收尾纪律**：`taskkill //F //IM game.exe`（MSYS 双斜杠，PIT-M-005）+
  `tasklist` 残留核对，逐任务留档；
- **聚合退出码**（审核 P2-6 后加入）：任一任务判定非 0 / 收尾不净 → 脚本整体
  exit 非 0（「退出码即判定」口径对驱动本身同样成立）；换行防护见 `.gitattributes`
  （`*.sh text eol=lf`，审核 P2-7）。

## 六、门禁

- `cargo check --workspace` REAL_EXIT=0（含 10 个新套件与 task-runner 扩展；
  `gate-cargo-check-workspace.log`）；
- `cargo test -p game` 11/11 REAL_EXIT=0（`gate-cargo-test-game.log`）；
- `cargo build --release` REAL_EXIT=0（`gate-build-release.log`；ts-10 修复后重建
  `gate-check-ts10-fix.log` / `gate-build-ts10-fix.log` 均 0）；
- `cargo test --doc -p docs`（本证据文档注册后，见下）。

## 七、M1 校准结论

- 第一阶段 12/12（断言 47/47）的全部清单语义经第二阶段脚本化重跑**全部可复现且
  通过**（78 条脚本断言＝套件 33 去重 + 工具侧 45，粒度映射见 §一/§三；口径升级：
  ts-07 逐位复算强于容差 0.1、ts-04 全量逐位比对强于「任一实体」、ts-12 十轮
  逐轮核验 + 确定性取序强于「取一个 Wanderer」）；
- 脚本化差异如实记录：清单 #1 类字面值（500/1000/100/250/50000、seed、9.0/3.0、
  0.1.0）由 `--expect-*` 绑定；TS-02 #3 跨进程对照、TS-01 #4 跨进程一致为测量
  口径，进程内以重抽等价替换（各任务判定记录注明）；TS-09 #4 反证为实现期干预
  不重复；
- `docs/m1-acceptance.md` §六所书「届时本节数字将被脚本化复测结果取代或确认」
  ——**确认**：第一阶段测量数字维持有效，第二阶段脚本断言为可重复执行的等价
  载体（重跑入口 `run-all.sh`，约 2 分钟全程）。

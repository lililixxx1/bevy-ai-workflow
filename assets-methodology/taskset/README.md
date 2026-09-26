# taskset/（标准化任务测试集）

- 用途：M1 建立 ≥ 10 个标准化任务；每任务含**需求描述 + BRP 可断言的验收清单**（意向文档 §4 M1）。
- 双重身份：①「一次通过率」的测量基准（M1 验收）；②版本升级后的回归集，量化迁移成本（升级窗口完成判定）。
- 两阶段判定：M1 期间人工对照验收清单判定并留存运行证据；BRP 与 `run_tests` 就绪后转为脚本断言并重跑校准旧数据（§10 第 5 条）。
- 编号口径：本目录条目用 `TS-xx`（TaskSet），与**台账**的 `Txxx`（任务实例记录）是两个编号空间——taskset 是任务定义，台账是每次执行的记录（执行 taskset 任务时台账「任务」列标注 TS 条目号）。
- 公共前置（所有任务默认遵守，不再逐条重复）：
  - 遵守 [`bevy-dev/SKILL.md`](../../bevy-dev/SKILL.md)（版本锁、查证流程、架构约定）与 [`AGENTS.md`](../../AGENTS.md)；
  - demo 基线代码在 `game/`（`--count`/`--seed`/`--bench-secs` CLI、BRP 常驻 127.0.0.1:15702、模拟确定性口径见 `game/src/sim.rs` 模块注释）；
  - BRP 请求均为 JSON-RPC 2.0 over HTTP POST `http://127.0.0.1:15702`（必须含 `"jsonrpc":"2.0"` 与 `"id"`；组件/资源引用用 Rust 全路径，如 `game::sim::SimStats`）；
  - 完成判定 = 验收清单逐条断言通过 + `cargo check --workspace` REAL_EXIT=0（运行时行为另按 SKILL.md §4.2 走 BRP 闭环）；
  - 证据留存：验收断言的命令与响应摘录（或日志路径）记入台账「证据」列。

## 条目模板

```markdown
# TS-xx：<任务名>

- 难度：<低/中/高>
- 前置：<依赖的 TS 条目 / 无>
- 类型：<功能/验证/重构…>

## 需求描述

<做什么；边界与约束；禁止事项>

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- <args>`（后台），BRP 端点 `http://127.0.0.1:15702`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.query` | params.data.components=["game::sim::Wanderer"] | … |

## 判定记录

- 判定者/日期/结论：（两阶段：人工对照 / run_tests 脚本）
- 证据：（命令与响应摘录 / 日志路径）
```

## 索引

| id | 任务 | 难度 | 验证面 | 文件 |
|---|---|---|---|---|
| TS-01 | 实体数量参数化与计数核对 | 低 | world.query / world.get_resources | [ts-01-entity-count.md](./ts-01-entity-count.md) |
| TS-02 | 速度上限配置化 | 中 | world.query 组件值采样 / world.get_resources | [ts-02-speed-cap.md](./ts-02-speed-cap.md) |
| TS-03 | 新增 BRP 可见 Resource | 低 | world.get_resources / world.list_resources | [ts-03-new-resource.md](./ts-03-new-resource.md) |
| TS-04 | 远程暂停/恢复模拟 | 中 | world.mutate_resources / world.get_resources | [ts-04-pause-resume.md](./ts-04-pause-resume.md) |
| TS-05 | BRP 生成实体并回读 | 低 | world.spawn_entity / world.get_components | [ts-05-spawn-entity.md](./ts-05-spawn-entity.md) |
| TS-06 | BRP 销毁实体 | 低 | world.despawn_entity / world.query | [ts-06-despawn-entity.md](./ts-06-despawn-entity.md) |
| TS-07 | 远程改写实体运动参数 | 高 | world.mutate_components / world.get_components | [ts-07-mutate-velocity.md](./ts-07-mutate-velocity.md) |
| TS-08 | 相机轨道参数远程调整 | 中 | world.mutate_resources / world.query(Transform) | [ts-08-camera-rig.md](./ts-08-camera-rig.md) |
| TS-09 | 新组件类型注册与 schema | 中 | world.list_components / registry.schema | [ts-09-type-schema.md](./ts-09-type-schema.md) |
| TS-10 | 事件驱动的状态变更 | 高 | world.trigger_event / world.get_resources | [ts-10-trigger-event.md](./ts-10-trigger-event.md) |
| TS-11 | 阶梯性能回归验证 | 中 | world.get_resources(SimStats) + 基线对照 | [ts-11-perf-ladder.md](./ts-11-perf-ladder.md) |
| TS-12 | 批量实体操作一致性 | 中 | world.insert_components / world.query 计数 | [ts-12-batch-ops.md](./ts-12-batch-ops.md) |

> JSON 数值形态注记（2026-09-26 实测，详见 `docs/brp-smoke.md`）：**向量（`Vec3`/`Vec2` 等）在 BRP 请求与响应中均为 `[x,y,z]` 数组**（glam serde 形态）；传 `{"x":..,"y":..}` 对象会报 `expected a sequence of 3 f32 values`。引用 bevy 内置类型时全路径以 `world.list_components` 实测为准（如 0.19 的 Camera 是 `bevy_camera::camera::Camera`，bevy_camera crate 而非 bevy_render）。任务书写断言时以 `world.query` 实测响应为准，不凭记忆猜形态。

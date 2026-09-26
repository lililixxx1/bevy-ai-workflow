# TS-08：相机轨道参数远程调整

- 难度：中
- 前置：TS-04
- 类型：功能（核对型：world.mutate_resources 驱动渲染视角，状态断言不依赖截图）

## 需求描述

不改动代码。经 BRP 修改 `CameraRig.radius`，核对相机 Transform 的轨道半径随之变化（渲染结果无需截图即被断言）。

## 验收清单（BRP 断言）

启动方式：`cargo run --release -p game -- --count 1000 --seed 20260926`。

| # | BRP 方法 | 请求要点 | 断言 |
|---|---|---|---|
| 1 | `world.query` | `data.components=["bevy_camera::camera::Camera","bevy_transform::components::transform::Transform"]`（**Camera 全路径在 0.19 为 `bevy_camera::camera::Camera`**——bevy_camera crate，非 bevy_render；2026-09-26 经 `world.list_components` 实测） | 恰 1 行；translation（`[x,y,z]` 数组形态）到 XZ 原点距离 ≈ 90.0（CameraRig::default().radius），容差 0.1 |
| 2 | `world.get_resources` | `resource="game::camera::CameraRig"` | `radius == 90.0`、`height == 45.0` |
| 3 | `world.mutate_resources` | `resource="game::camera::CameraRig"`, `path="radius"`, `value=60.0` | 响应无 error |
| 4 | `world.query` | 同 #1 | 距离 ≈ 60.0（容差 0.1；下一帧起生效，失败则等待 0.2s 重查） |

## 判定记录

- 判定者/日期/结论：执行 agent，2026-09-26 / **通过（两阶段判定之第一阶段：人工对照）**——零改码，4/4 断言首次执行全部通过（恰 1 行 Camera+Transform 且 XZ 距离 89.999999842972，|Δ|≈1.6e-7；radius===90.0、height===45.0；mutate radius→60.0 无 error 且 result:null；重查距离 59.999999758730，|Δ|≈2.4e-7，首查即过未触发 0.2s 重查条款）；`cargo check --workspace` REAL_EXIT=0
- 证据：`docs/evidence/ts-08-brp.md`（curl 命令原文 + 响应原文摘录 + 逐条判定）；原始响应/运行日志/门禁日志 `docs/evidence/ts-08/`

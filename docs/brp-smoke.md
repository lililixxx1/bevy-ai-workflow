# BRP 冒烟与断言实录（M1 第一役）

- 适用 Bevy 版本：**0.19**（bevy_remote 0.19.1）
- 日期：2026-09-26
- 角色：工作流三（AI 操控 Bevy，意向文档 §5.3）分步走第 1 步「curl/脚本直调内置方法验证通路」的完成凭证；taskset 各任务 BRP 断言的形态基准
- 证据：`docs/evidence/brp-smoke-run.log`（游戏运行日志）、`docs/evidence/brp-assert.mjs`（断言脚本）、`docs/evidence/brp-assert-result.txt`（断言结果，11/11 PASS）

## 通路确认

- 游戏侧：`game/src/brp.rs` 挂 `RemotePlugin::default()` + `RemoteHttpPlugin::default().with_address(Ipv4Addr::LOCALHOST)`，显式绑定回环；主 app 端口 15702（默认值，`bevy_remote-0.19.1/src/http.rs:52`）。
- `rpc.discover` 返回 `{"info":{"title":"Bevy Remote Protocol","version":"0.19.1"}, ...}`，方法集含 `world.query` / `world.spawn_entity` / `world.get_resources` / `world.mutate_resources` / `registry.schema` 等全部 23 个内置方法。

## 断言链（11 项全部 PASS，2026-09-26）

启动：`./target/release/game.exe --count 1000 --seed 20260926`（release、后台）。

| # | 断言 | BRP 方法 | 结果 |
|---|---|---|---|
| 1 | `SimConfig` 读回 count=1000/seed=20260926/paused=false | `world.get_resources` | PASS |
| 2 | `SimStats.tick` 随时间增长（每帧真实更新） | `world.get_resources` ×2 | PASS（360→427） |
| 3 | Wanderer 实体计数 == 1000 | `world.query` | PASS |
| 4 | 实体 Transform.translation 随帧变化 | `world.get_components` ×2 | PASS |
| 5 | `paused=true` 后 tick 冻结（帧边界容差 ≤2） | `world.mutate_resources` + `get_resources` ×2 | PASS（467→468，差 1 为 mutate 生效点竞态） |
| 6 | `paused=false` 后 tick 恢复增长 | 同上 | PASS（468→535） |
| 7 | spawn 实体注入 Velocity 后回读一致 | `world.spawn_entity` + `get_components` | PASS（`[1,0,2]`） |
| 8 | despawn 后 Wanderer 计数恢复 1000 | `world.despawn_entity` + `query` | PASS |
| 9 | 相机轨道半径 == 90.0 | `world.query`（Camera+Transform） | PASS（dist=90.00） |
| 10 | 自定义类型可被列出 | `world.list_components` | PASS（含 `game::sim::Wanderer`/`Velocity`） |
| 11 | 解析式运动数学断言 | `world.query` + `get_resources` | PASS（t=9.13s，‖p−f(t)‖≈0.059 < 0.1） |

断言 #11 的意义：位置是 `SimStats.elapsed_secs` 的解析函数，BRP 端**不依赖时序即可精确复算任意实体位置**——「状态注入 > 截图」验证取向（意向文档 §3）的最小完整样例。

## 实测形态结论（写断言必读）

1. **向量是数组不是对象**：`Vec3`（及 `Vec2`/`Quat`）在请求与响应中均为 `[x,y,z]` 序列（glam serde 形态）。反例实测：`world.spawn_entity` 传 `{"linear":{"x":1.0,"y":0.0,"z":2.0}}` 被拒——`{"code":-23402,"message":"game::sim::Velocity is invalid: invalid type: map, expected a sequence of 3 f32 values"}`；改传 `[1.0,0.0,2.0]` 成功。
2. **Camera 全路径在 0.19 是 `bevy_camera::camera::Camera`**（bevy_camera crate），不是 bevy_render——凭 0.16 前的印象写 `bevy_render::camera::Camera` 会查到空结果。引用内置类型先 `world.list_components` 实测。
3. **远程 mutate 有帧边界竞态**：`mutate_resources` 的生效点在请求抵达后的下一次帧边界；「立即冻结」类断言要容忍 O(1) tick 边界差（实测恰好差 1 tick）。
4. `world.query` 响应行含组件值（`BrpQueryRow{entity, components}`），不只是实体号——可直接做值断言；`Transform` 路径 `bevy_transform::components::transform::Transform`。

## 复现方式

```text
./target/release/game.exe --count 1000 --seed 20260926   # 后台
node docs/evidence/brp-assert.mjs                        # 11 项断言
# 用完杀干净游戏进程（taskkill /F /IM game.exe）
```

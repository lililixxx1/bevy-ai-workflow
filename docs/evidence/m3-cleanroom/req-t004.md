# 净室重跑需求书 T004（BRP 通路验证·裁剪版）

来源：意向文档 §10 第 4 条 + §5.3 分三步走的第 1、2 步（任务时点原文）。判定口径与原任务相同：首判全链跑通有存证 = 一次通过；返工 = 首判未过后的重做次数。

## 原始需求原文

> 4. BRP 通路验证（可与第 5 项并行）：curl 直调内置方法确认通路 → 接入 bevy_brp_mcp 实测 0.19 兼容性（M2 首战）。

> §5.3 分三步走第 1 步：人工用 curl/脚本直调 BRP 内置方法（`world.query` / `world.spawn_entity` / `world.mutate_components` / `registry.schema`），验证通路；
> §5.3 分三步走第 2 步：接入 bevy_brp_mcp，验证其对 0.19 的兼容性（M2 首个任务；不兼容则修补/fork）。

## 重跑范围（裁剪与差异标注）

1. **[保留]** curl 直调内置方法验证通路：被验对象 = 净室重跑 T003 的 demo 产物（`cleanroom/t003/`，由你启动，进程须由长生命周期后台任务托管——短命前台调用内的子进程活不过几秒）。
   - 方法覆盖：`world.query` / `world.spawn_entity` / `world.mutate_components` / `registry.schema` 四方法必测；其余内置方法以 `rpc.discover` 实测结果为准，**总覆盖 ≥ 8 个方法**；
   - 每个方法 ≥ 1 次成功调用 + ≥ 1 次故意错误探针（观察并留档错误语义与错误码）；
   - 每次调用的请求 JSON 与响应原文逐条留档 logs/。
2. **[裁剪]** bevy_brp_mcp 接入部分：重跑环境无该 MCP server 配置，不可执行，裁剪并如实标注（原任务此半为三轮 MCP 会话存证，其协议级兼容结论不因重跑改变，不在本重跑判定面内）。

## 验收清单

- a) `rpc.discover` 拿到方法清单（原文留档，方法计数）；
- b) ≥ 8 个内置方法成功调用存证；
- c) 错误探针的报错原文留档（≥ 3 组）；
- d) 游戏进程收尾：杀净并核对无残留。

## 纪律

- 遵守 INJECTED-agents-discipline.md 全部硬约束；BRP 仅 127.0.0.1。
- 可读知识资产（仅限）：仓库 `bevy-dev/` 与 `assets-methodology/` 全部；`cleanroom/t003/` 的产物与 README（被验对象）；本目录其他 req/INJECTED 文件。
- **禁读**：仓库 `game/`、`tooling/`、`docs/`、`.git/` 与 `bevy-probe-*`、`.zcode/`。
- 全程留痕：所有 curl 请求/响应、进程管理命令与真实退出码写 logs/；最终报告如实列出每一轮尝试。

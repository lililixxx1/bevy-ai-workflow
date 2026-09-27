# batch-c 勘误与过程注记（2026-09-27，独立审核后落档）

归档文件为原始证据，一律不回改；本页为勘误与补注的权威落点。

1. **`probe-src-final/src/lib.rs:110` 行号失准**（P3c 注释中 `par_iter.rs:31/:55`）：`for_each`/`for_each_init` 实际位于 `par_iter.rs:42/:77`（`Cargo.toml :2757` 同理失准——`default_platform` 起点 `:2762`、`multi_threaded` 条目 `:2768`）。资产条目（PIT-B-006、SKILL §6.10）已按正确行号修正。
2. **multi_threaded 事实勘误**：探查初判「本仓默认单线程执行器」有误——门面 default=[2d,3d,ui,audio] 经 2d/3d/ui→default_platform 链传递启用 multi_threaded（`bevy-0.19.1/Cargo.toml:2586-2592/:2605-2611/:2884-2890/:2762-2774`），本仓 game/docs 实际为 MultiThreadedExecutor（`cargo tree -e features` 实证，独立审核纠正）。PIT-B-006「feature 关闭时 for_each 退化串行」为一般性事实，在本仓默认链下不生效。
3. **attempt 轮源码留存缺失**：本批仅留 attempt 轮日志（probe-r2-check-attempt1.log / probe-run-r3c-attempt1.log），未留当轮源码快照（与最终快照的差异可由日志行号自洽佐证）。自批次 D 起，attempt 轮源码一并归档（已入 m3-plan §3.3 批次规格）。
4. **r3c-attempt1 性质**：断言算术失误（预期值 55，忘初始值 1.0；实际 56）——探针侧测试错误，非 bevy 行为异常；失败轮输出恰好实证 chain 定序的确定性（56 = 1+5×11 逐位吻合）。

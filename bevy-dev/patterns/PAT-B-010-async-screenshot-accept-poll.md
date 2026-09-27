# PAT-B-010-async-screenshot-accept-poll

### PAT-B-010：异步截图的受理-轮询契约——spawn+observe 双 observer + 日志资源两段式

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（Screenshot 管线 + BRP 异步方法契约设计）
- 标签：`Screenshot` `ScreenshotCaptured` `observer` `异步` `轮询` `两段式` `PNG魔数`
- 源码出处：`game/src/rpc/screenshot.rs:114-122`（受理）+ `:146-158`（回填）

**场景**：渲染侧捕获天然异步（受理帧 ≠ 完成帧），BRP 方法要把异步动作暴露成可判定的契约——受理即返回 id，完成状态可轮询，最终证据（文件本身）由工具端核验。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/rpc/screenshot.rs` 的逐字删减）：

```rust
// 抄自 game/src/rpc/screenshot.rs:114-118（受理：spawn 携带 Screenshot 组件的
// 实体并挂两个 observer——官方 save_to_disk 落盘 + 本模块 on_captured 回填日志；
// :120-122 落账三行删去）：
let entity = world
    .spawn(Screenshot::primary_window())
    .observe(save_to_disk(std::path::PathBuf::from(path.clone())))
    .observe(on_captured)
    .id();

// 抄自 game/src/rpc/screenshot.rs:146-158（完成：observer 回填，截选回填主体）：
fn on_captured(
    event: On<ScreenshotCaptured>,
    mut log: ResMut<ScreenshotLog>,
    stats: Option<Res<crate::sim::SimStats>>,
) {
    let entity = event.entity;
    let size = event.image.texture_descriptor.size;
    // ...按 entity 反查日志条目，翻转 status=captured 并补 frame/尺寸...
}
```

要点：①`Screenshot(pub RenderTarget)` 组件实体 + `ScreenshotCaptured` EntityEvent 触发**同一实体**上的 observer（`bevy_render-0.19.1/src/view/window/screenshot.rs:68-101`，核实 2026-09-26）；②两个 observer 相对次序无契约——故状态只记「事件已到」，**不承诺文件已写**，文件核验（存在性 + PNG 魔数）留给工具端做最强证据；③`ScreenshotLog` 资源两段式（pending → captured），`game.screenshot_log` 轮询至 `status == "captured"`；④受理时顺手记 `SimStats.frame_count`（完成时再记一次）——受理/完成帧差即「捕获 +N 帧」的量化口径。

**为什么**：把「不可同步完成的动作」契约化：调用方拿 id 轮询、不盲等；完成信号（事件）与副作用（落盘）解耦；帧号双记使异步延迟可测量可断言。

**验证证据**：
- 代码为 M2（T019）已验证任务代码的逐字删减，**删减后未单独重新编译**；
- 运行验证：M2 验收②全链——spawn→screenshot→读回断言→PNG 魔数+IHDR 与日志互证（捕获受理后 +3 帧）→despawn，`docs/evidence/m2-rpc.md` + `m2-rpc/`（PNG ×2 + transcript ×4，2026-09-26）。

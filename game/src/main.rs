//! 游戏工程（试金石）——最小骨架。
//!
//! 骨架阶段目标（意向文档 §10 第 1 条）：仅验证 workspace 与 bevy 0.19 依赖
//! 解析、编译链路可用；不引入复杂特性。首个 demo（相机漫游 + N 动态实体）
//! 由后续任务实现（§10 第 5 条）。

fn main() {
    // 骨架阶段刻意不引用任何 bevy 符号（含曾考虑的 bevy::VERSION——经查本地源码
    // bevy-0.19.1 并无该常量，容器 crate 仅 `pub use bevy_internal::*`）。
    // bevy 依赖树的真实编译验证由 `cargo check --workspace` + Cargo.lock 承担；
    // 首个 bevy API 使用出现在 demo 任务（§10 第 5 条）。
    println!("game skeleton | bevy 0.19 (locked by workspace)");
}

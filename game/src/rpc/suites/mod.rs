//! 任务套件注册表：套件名 → 游戏进程内断言集（`game.run_tests` 的载荷）。
//!
//! 命名与 `assets-methodology/taskset/ts-XX-*.md` 一一对应（ts-01 ↔ TS-01）。
//! 新增套件：新文件实现 `fn(&mut World) -> Vec<TestCase>`（约定**只读**世界，
//! 不 spawn/despawn/mutate——断言方法不得有副作用），然后在 [`all`] 注册。
//! 断言设计口径见 `rpc/run_tests.rs` 模块文档（跨帧采样归工具端）。

pub mod ts01;

use bevy::prelude::*;

/// 单条断言结果（`game.run_tests` 响应 `results` 数组元素）。
#[derive(serde::Serialize, Clone, Debug)]
pub struct TestCase {
    /// 断言名（蛇形，跨版本稳定标识；工具端按名引用）。
    pub name: String,
    /// 是否通过（false 不是协议错误，由工具端汇总裁决）。
    pub pass: bool,
    /// 人类可读细节（含关键实测数值，供留证与人工复核）。
    pub detail: String,
}

/// 套件函数形态（约定只读世界；取 `&mut World` 仅为 `World::query` 便利）。
pub type SuiteFn = fn(&mut World) -> Vec<TestCase>;

/// 全部套件 `(名称, 入口)`。
pub fn all() -> Vec<(&'static str, SuiteFn)> {
    vec![("ts-01", ts01::run)]
}

/// 按名查找套件。
pub fn lookup(name: &str) -> Option<(&'static str, SuiteFn)> {
    all().into_iter().find(|(n, _)| *n == name)
}

/// 可用套件名清单（错误 message 用）。
pub fn names() -> Vec<String> {
    all().into_iter().map(|(n, _)| n.to_string()).collect()
}

/// 断言结果构造小工具（套件内统一三参形态）。
pub(crate) fn tc(name: &str, pass: bool, detail: String) -> TestCase {
    TestCase {
        name: name.to_string(),
        pass,
        detail,
    }
}

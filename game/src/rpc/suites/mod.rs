//! 任务套件注册表：套件名 → 游戏进程内断言集（`game.run_tests` 的载荷）。
//!
//! 命名与 `assets-methodology/taskset/ts-XX-*.md` 一一对应（ts-01 ↔ TS-01）。
//! **TS-04 无套件**：其清单全部为跨帧时序断言（暂停冻结/恢复需跨秒采样），
//! 由 task-runner `pause` 子命令全程经 BRP 驱动判定。
//! **ts-14 是 TS-14 的第二阶段套件**（TS-15 任务落档：BRP 第一阶段
//! `docs/evidence/ts-14-brp.md` → 进程内回归，沿 M1 两阶段判定先例）。
//!
//! 套件约定（两阶段判定第二阶段，M1 校准口径）：
//! - **无净副作用**：套件返回时世界须与调用前状态等价（计数/资源值/初值还原；
//!   实体号允许变化——回归口径禁止断言绝对 entity id）。只读型套件
//!   （01/02/03/09/11/17）天然满足；写通路型套件（05/06/07/10/12）以 ECS 直写
//!   （`World::spawn`/`despawn`/`get_mut`/`resource_mut`/`trigger`——与 BRP
//!   handler 同一 `&mut World` 通路）复测判定语义，断言完成即自恢复；08 为
//!   两连调状态机（首次改 radius=60，第二次断言随动并还原 90）；17 只读快照
//!   断言（T040），其 launch 建场面在出场时清还原。
//! - BRP 反射序列化通路（spawn_entity/mutate_components/trigger_event 的
//!   JSON 形态与错误面）属第一阶段已验证面，套件不重复覆盖。
//! - 跨帧采样断言（tick 递增、相机随动生效窗口）归工具端（task-runner
//!   `run` 两连调比对 `snapshot.tick`）。
//! - 新增套件：新文件实现 `fn(&mut World) -> Vec<TestCase>`，在 [`all`] 注册。
//! 断言设计口径见 `rpc/run_tests.rs` 模块文档。

pub mod ts01;
pub mod ts02;
pub mod ts03;
pub mod ts05;
pub mod ts06;
pub mod ts07;
pub mod ts08;
pub mod ts09;
pub mod ts10;
pub mod ts11;
pub mod ts12;
pub mod ts14;
pub mod ts17;

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

/// 套件函数形态（约定无净副作用：只读或写后自恢复，见模块文档）。
pub type SuiteFn = fn(&mut World) -> Vec<TestCase>;

/// 全部套件 `(名称, 入口)`。
pub fn all() -> Vec<(&'static str, SuiteFn)> {
    vec![
        ("ts-01", ts01::run),
        ("ts-02", ts02::run),
        ("ts-03", ts03::run),
        ("ts-05", ts05::run),
        ("ts-06", ts06::run),
        ("ts-07", ts07::run),
        ("ts-08", ts08::run),
        ("ts-09", ts09::run),
        ("ts-10", ts10::run),
        ("ts-11", ts11::run),
        ("ts-12", ts12::run),
        ("ts-14", ts14::run),
        ("ts-17", ts17::run),
    ]
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

//! 工作流一（AI 写文档）的 doctest 门禁。
//!
//! `docs/` 下的 Markdown 经 `include_str!` 薄封装纳入本 crate 的 rustdoc，
//! 由 `cargo test --doc -p docs` 编译并运行文档内示例代码——编译器当裁判，
//! 过滤 AI 幻觉（意向文档 §5.1）。长运行/需窗口的示例标 `no_run` 并注明
//! 理由；禁止无理由 `ignore`。
//!
//! 新增文档的接入方式：在 `docs/` 放 Markdown 文件，然后仿照下方条目
//! 追加一个 `#[doc = include_str!(...)]` 的公开空模块。

/// 《文档写法规范》——`docs/` 全部文档的写法基准（seed 文档）。
#[doc = include_str!("../doc-conventions.md")]
pub mod doc_conventions {}

/// 《帧率基线》（M1 第一役）——口径五要素 + 阶梯数据 + 复测方式。
#[doc = include_str!("../fps-baseline.md")]
pub mod fps_baseline {}

/// 《BRP 冒烟与断言实录》（M1 第一役）——通路确认 + 断言链 + 实测形态结论。
#[doc = include_str!("../brp-smoke.md")]
pub mod brp_smoke {}

/// 《M1 验收结论》（一次通过率测量）——协议、结果、独立审计与样本量限制。
#[doc = include_str!("../m1-acceptance.md")]
pub mod m1_acceptance {}

/// 《M2 自研 RPC 证据》——game.run_tests / game.screenshot 的协议面、全自动判定与全链留证。
#[doc = include_str!("../evidence/m2-rpc.md")]
pub mod m2_rpc {}

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

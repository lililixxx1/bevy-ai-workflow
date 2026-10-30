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

/// 《M3 验收结论》（两层知识资产成型与增值证明）——①-④ 分列证据、局限声明、分块审核汇总。
#[doc = include_str!("../m3-acceptance.md")]
pub mod m3_acceptance {}

/// 《M2 自研 RPC 证据》——game.run_tests / game.screenshot 的协议面、全自动判定与全链留证。
#[doc = include_str!("../evidence/m2-rpc.md")]
pub mod m2_rpc {}

/// 《M1 两阶段判定第二阶段》——12 任务验收清单的脚本化重跑校准（套件架构、78 断言结果、返工与驱动口径）。
#[doc = include_str!("../evidence/m1-phase2.md")]
pub mod m1_phase2 {}

/// 《M3 执行计划》——验收映射与块划分、② 资产生产方法学、④ 取样与量化口径预注册。
#[doc = include_str!("../m3-plan.md")]
pub mod m3_plan {}

/// 《M4 选品备忘录》——候选案对照（A 战术小品 / B 推箱子 / C 卡组 roguelike）与三固化项草案，待 owner 拍板转正（M4 验收①）。
#[doc = include_str!("../m4-game-selection.md")]
pub mod m4_game_selection {}

/// 《M4 验收文档》——①选品备忘录 / ②可玩交付（战役闭环、无 P0、回归全绿、重放一致）/ ③台账与核心系统任务化，席位对照与体量封顶核对。
#[doc = include_str!("../m4-acceptance.md")]
pub mod m4_acceptance {}

/// 《窗口前置增强方案（定案版）》——grill 七项定案、A 级三项任务卡（claim-lint / game.snapshot / MCP 薄桥）、B/C/D 级处置与时序红线。
#[doc = include_str!("../pre-window-plan.md")]
pub mod pre_window_plan {}

/// 《M5 选品备忘录》——万阵收缩形态（grill 八题定案）：三固化项、系统席位表、game.* 验证面与薄 BRP host 补课，待 owner 批复转正（意向文档 §4 M5 行验收①）。
#[doc = include_str!("../m5-game-selection.md")]
pub mod m5_game_selection {}

/// 《M6 选品备忘录》——塔防教程试金石（立项获准 + grill 四题定案 2026-10-30）：验证面扩展层、四件套收官判定、三重封顶与 td.* 方法面（纯决策文档不含 API 示例——沿 m4/m5 先例，零 doctest 增量）。
#[doc = include_str!("../m6-game-selection.md")]
pub mod m6_game_selection {}

/// 《Bevy 特定层错题本》——反例经 compile_fail 围栏由 doctest 机器断言（M3 起纳入门禁，围栏约定见其文件头）。
#[doc = include_str!("../../bevy-dev/pitfalls.md")]
pub mod bevy_dev_pitfalls {}

/// 《窗口回归重核清单（B2）》——T043 执行批 + T044 扩节：PIT-B 51 + PAT-B 21 + SKILL §6.1–6.16 共 88 条机器可读核对表，
/// 内嵌 doctest 自校（表体计数 == claim-lint expect 镜像 + 分型词表校验）。
#[doc = include_str!("../recheck-registry.md")]
pub mod recheck_registry {}

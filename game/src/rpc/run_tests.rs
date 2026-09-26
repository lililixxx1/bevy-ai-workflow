//! [`game.run_tests`]：任务测试集验收清单的游戏进程内自动判定（M2 验收③）。
//!
//! 设计口径（意向文档 §10 第 3 条「状态注入优先、截图作辅助证据」）：本方法
//! 只读世界状态做断言，不产生任何副作用；清单中需要跨调用采样的断言（如
//! TS-01 断言 3 的「间隔 ≥1s 后 tick2 > tick1」）由工具端驱动两次调用并比对
//! 响应中的 `snapshot` 完成——进程内单次调用无法跨帧采样。
//!
//! 请求：`{"suite": "ts-01"}`。缺 params / 缺 suite / 未知套件 → JSON-RPC 错误
//! `INVALID_PARAMS`（-32602，message 附可用套件清单）；**断言失败不是协议错误**，
//! 正常返回逐条 `pass=false`，由工具端按退出码判定。

use bevy::prelude::*;
use bevy::remote::BrpResult;
use serde_json::{json, Value};

use super::{invalid_params, suites};

/// handler 入口（`with_method_main` 注册，独占 `&mut World`；本方法约定只读）。
pub fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    let params = params.ok_or_else(|| {
        invalid_params(&format!("缺 params（须为 {{\"suite\": \"...\"}}）；可用套件：{}", suites::names().join(", ")))
    })?;
    let suite = params
        .get("suite")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            invalid_params(&format!("缺 suite 字段（须为 {{\"suite\": \"...\"}}）；可用套件：{}", suites::names().join(", ")))
        })?;

    let Some((_, run)) = suites::lookup(suite) else {
        return Err(invalid_params(&format!(
            "未知套件 `{suite}`；可用：{}",
            suites::names().join(", ")
        )));
    };

    let results = run(world);
    let passed = results.iter().filter(|r| r.pass).count();
    let failed = results.len() - passed;

    // 世界快照：工具端跨调用断言的数据面（SimStats 缺失时为 null，不自造默认值）。
    let stats = world.get_resource::<crate::sim::SimStats>();
    Ok(json!({
        "suite": suite,
        "total": results.len(),
        "passed": passed,
        "failed": failed,
        "results": results,
        "snapshot": {
            "tick": stats.map(|s| s.tick),
            "frame_count": stats.map(|s| s.frame_count),
            "elapsed_secs": stats.map(|s| s.elapsed_secs),
        },
    }))
}

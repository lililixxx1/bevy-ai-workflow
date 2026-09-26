//! task-runner：任务测试集的自动判定驱动（自研件，M2 验收③的工具侧载体）。
//!
//! 游戏外进程（不链接 bevy，SKILL.md §3.6），经 BRP over HTTP（默认
//! 127.0.0.1:15702）驱动运行中的游戏：
//!
//! - `run --suite ts-01 [--gap-secs 1]`：调用 `game.run_tests` 两次（间隔
//!   gap-secs），逐条断言打印 + 跨调用 `snapshot.tick` 递增判定（TS-01 清单
//!   断言 3 的第二次采样）；全部通过退出码 0，否则 1。
//! - `chain [--out-dir brp-chain]`：M2 验收②全链——spawn 实体 → 改组件 →
//!   截图 → 读回断言 → PNG 文件核验（存在 / 魔数 / IHDR 尺寸与日志一致）→
//!   despawn 清理，全程 BRP，请求/响应逐条留痕 stdout（重定向即证据日志）。
//!
//! 依赖面刻意最小（Cargo.toml 注释）：HTTP 为 std TCP 极简实现，见 [`brp`]。

mod brp;

use brp::BrpClient;
use serde_json::{json, Value};

const USAGE: &str = "用法：
  task-runner run  --suite ts-01 [--url http://127.0.0.1:15702] [--gap-secs 1]
                   [--expect-count 500] [--expect-seed 20260926]
  task-runner chain [--url http://127.0.0.1:15702] [--out-dir brp-chain]

--expect-count/--expect-seed：清单字面值绑定（套件内断言 1/2 为配置自洽口径——
count==entity_count、config↔metadata 一致；任意 --count 启动也会全绿。带本参数时
工具端另经 world.get_resources 核对 SimConfig 字面值，绑定清单原文的 ==500/==20260926）。";

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match args.first().map(String::as_str) {
        Some("run") => cmd_run(&args[1..]),
        Some("chain") => cmd_chain(&args[1..]),
        _ => Err(USAGE.to_string()),
    };
    match outcome {
        Ok(()) => {
            println!("== 结论：全部通过 ==");
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("== 结论：失败 ==\n{e}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// 从剩余参数取 `--key value`（flag 存在但缺值 → 报错，不静默取默认）。
fn opt_value(args: &[String], key: &str) -> Result<Option<String>, String> {
    match args.iter().position(|a| a == key) {
        None => Ok(None),
        Some(i) => args
            .get(i + 1)
            .cloned()
            .map(Some)
            .ok_or_else(|| format!("{key} 缺值（--key value 形态）")),
    }
}

fn client_from(args: &[String]) -> Result<BrpClient, String> {
    let url = opt_value(args, "--url")?
        .unwrap_or_else(|| "http://127.0.0.1:15702".into());
    BrpClient::new(&url)
}

/// `run` 模式：run_tests 两连调（间隔 gap）+ 跨调用 tick 递增
/// （+ 可选的字面值绑定：--expect-count / --expect-seed）。
fn cmd_run(args: &[String]) -> Result<(), String> {
    let suite = opt_value(args, "--suite")?
        .ok_or_else(|| format!("run 模式需要 --suite <名称>\n{USAGE}"))?;
    let gap: u64 = opt_value(args, "--gap-secs")?
        .map(|s| s.parse().map_err(|e| format!("--gap-secs 非数字：{e}")))
        .transpose()?
        .unwrap_or(1);
    let expect_count: Option<u64> = opt_value(args, "--expect-count")?
        .map(|s| s.parse().map_err(|e| format!("--expect-count 非数字：{e}")))
        .transpose()?;
    let expect_seed: Option<u64> = opt_value(args, "--expect-seed")?
        .map(|s| s.parse().map_err(|e| format!("--expect-seed 非数字：{e}")))
        .transpose()?;
    let mut client = client_from(args)?;

    println!("== [1/4] game.run_tests（第 1 次调用，套件 {suite}）==");
    let (pass1, total1, tick1) = call_and_report(&mut client, &suite)?;

    println!("== [2/4] 字面值绑定（SimConfig 核对：--expect-count/--expect-seed）==");
    let literal_ok = literal_checks(&mut client, expect_count, expect_seed)?;

    println!("== [3/4] 间隔 {gap}s（tick 第二采样窗口）==");
    std::thread::sleep(std::time::Duration::from_secs(gap));

    println!("== [4/4] game.run_tests（第 2 次调用）==");
    let (pass2, total2, tick2) = call_and_report(&mut client, &suite)?;

    println!(
        "== 跨调用断言：snapshot.tick2({tick2:?}) > tick1({tick1:?})（TS-01 清单断言 3 后半）=="
    );
    let tick_ok = matches!((tick1, tick2), (Some(a), Some(b)) if b > a);
    println!("{}", if tick_ok { "[PASS]" } else { "[FAIL]" });

    let all_ok = tick_ok && literal_ok && pass1 == total1 && pass2 == total2 && total1 > 0;
    if !all_ok {
        return Err(format!(
            "套件 {suite} 未全通过：第 1 次 {pass1}/{total1}，第 2 次 {pass2}/{total2}，tick 递增 {tick_ok}，字面值 {literal_ok}"
        ));
    }
    Ok(())
}

/// 清单断言 1/2 的字面值绑定：经 `world.get_resources` 读 SimConfig，
/// 核对 entity_count / seed 与期望字面值（不提供期望则本环节跳过，literal_ok=true）。
fn literal_checks(
    client: &mut BrpClient,
    expect_count: Option<u64>,
    expect_seed: Option<u64>,
) -> Result<bool, String> {
    if expect_count.is_none() && expect_seed.is_none() {
        println!("-- 未提供 --expect-count/--expect-seed，跳过（套件内为配置自洽口径）");
        return Ok(true);
    }
    let cfg = client.call(
        "world.get_resources",
        json!({ "resource": "game::sim::SimConfig" }),
    )?;
    let actual_count = cfg.pointer("/value/entity_count").and_then(Value::as_u64);
    let actual_seed = cfg.pointer("/value/seed").and_then(Value::as_u64);
    let count_ok = match expect_count {
        Some(e) if actual_count == Some(e) => {
            println!("[PASS] entity_count={:?} == 期望 {e}", actual_count);
            true
        }
        Some(e) => {
            println!("[FAIL] entity_count={actual_count:?} ≠ 期望 {e}");
            false
        }
        None => true,
    };
    let seed_ok = match expect_seed {
        Some(e) if actual_seed == Some(e) => {
            println!("[PASS] seed={:?} == 期望 {e}", actual_seed);
            true
        }
        Some(e) => {
            println!("[FAIL] seed={actual_seed:?} ≠ 期望 {e}");
            false
        }
        None => true,
    };
    Ok(count_ok && seed_ok)
}

/// 一次 run_tests 调用：打印逐条断言，返回 (通过数, 总数, snapshot.tick)。
fn call_and_report(client: &mut BrpClient, suite: &str) -> Result<(usize, usize, Option<u64>), String> {
    let result = client.call("game.run_tests", json!({ "suite": suite }))?;
    let results = result
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("run_tests 响应缺 results 数组：{result}"))?;
    for case in results {
        let name = case.get("name").and_then(Value::as_str).unwrap_or("<无名>");
        let pass = case.get("pass").and_then(Value::as_bool).unwrap_or(false);
        let detail = case.get("detail").and_then(Value::as_str).unwrap_or("");
        println!(
            "{} {name} — {detail}",
            if pass { "[PASS]" } else { "[FAIL]" }
        );
    }
    let counted_pass = results
        .iter()
        .filter(|c| c.get("pass").and_then(Value::as_bool).unwrap_or(false))
        .count();
    let reported_pass = result.get("passed").and_then(Value::as_u64).unwrap_or(0) as usize;
    if counted_pass != reported_pass {
        return Err(format!(
            "完整性核对失败：响应自报 passed={reported_pass} 与逐条计数 {counted_pass} 不符"
        ));
    }
    let tick = result
        .pointer("/snapshot/tick")
        .and_then(Value::as_u64);
    println!("-- 汇总：{counted_pass}/{} 通过，snapshot.tick={tick:?} --", results.len());
    Ok((counted_pass, results.len(), tick))
}

/// `chain` 模式：M2 验收②全链（spawn → mutate → screenshot → 读回断言 → 文件核验）。
fn cmd_chain(args: &[String]) -> Result<(), String> {
    let out_dir = opt_value(args, "--out-dir")?.unwrap_or_else(|| "brp-chain".into());
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("建目录 {out_dir} 失败：{e}"))?;
    let mut client = client_from(args)?;

    // ① spawn：新实体（Tagged + Velocity，均已注册可反射序列化）。
    println!("== [1/6] world.spawn_entity（Tagged + Velocity）==");
    let spawned = client.call(
        "world.spawn_entity",
        json!({
            "components": {
                "game::sim::Tagged": { "tag": "brp-chain-marker" },
                "game::sim::Velocity": { "linear": [0.5, 0.0, 0.5] }
            }
        }),
    )?;
    let entity = spawned
        .get("entity")
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("spawn 响应缺 entity：{spawned}"))?;
    println!("[PASS] 实体号 {entity}");

    // ② mutate：改写 tag 字段（字段级 path 改写）。
    println!("== [2/6] world.mutate_components（tag → brp-chain-mutated）==");
    client.call(
        "world.mutate_components",
        json!({
            "entity": entity,
            "component": "game::sim::Tagged",
            "path": "tag",
            "value": "brp-chain-mutated"
        }),
    )?;
    println!("[PASS] 响应无 error（改写已受理）");

    // ③ screenshot：受理异步捕获，输出路径显式指定。
    println!("== [3/6] game.screenshot（主窗口帧）==");
    let png_path = format!("{out_dir}/m2-chain-{entity}.png");
    let shot = client.call("game.screenshot", json!({ "path": png_path }))?;
    let shot_id = shot.get("id").and_then(Value::as_u64);
    println!("[PASS] 受理 id={shot_id:?} path={png_path}");

    // ④ 读回断言：tag 已是新值。
    println!("== [4/6] world.get_components 读回断言（tag == brp-chain-mutated）==");
    let read = client.call(
        "world.get_components",
        json!({ "entity": entity, "components": ["game::sim::Tagged"] }),
    )?;
    let tag = read
        .pointer("/components/game::sim::Tagged/tag")
        .or_else(|| read.pointer("/game::sim::Tagged/tag"))
        .and_then(Value::as_str)
        .ok_or_else(|| format!("读回响应中找不到 Tagged.tag：{read}"))?;
    if tag != "brp-chain-mutated" {
        return Err(format!("读回断言失败：tag={tag:?} ≠ \"brp-chain-mutated\""));
    }
    println!("[PASS] tag={tag:?}");

    // ⑤ 轮询 screenshot_log 至 captured（超时 15s），再核验文件本身。
    println!("== [5/6] game.screenshot_log 轮询 + PNG 文件核验 ==");
    let entry = poll_captured(&mut client, shot_id, 15)?;
    let bytes = std::fs::read(&png_path)
        .map_err(|e| format!("截图文件读取失败（{png_path}）：{e}（captured 只承诺事件已到，文件核验属工具端）"))?;
    const PNG_MAGIC: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() < 24 || bytes[..8] != PNG_MAGIC {
        return Err(format!("PNG 魔数核验失败（{} 字节）：{png_path}", bytes.len()));
    }
    // IHDR 尺寸（魔数 8 + 长度 4 + "IHDR" 4 之后）：与日志回填的宽高互证。
    let ihdr_w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let ihdr_h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    let (log_w, log_h) = (
        entry.get("width").and_then(Value::as_u64),
        entry.get("height").and_then(Value::as_u64),
    );
    if Some(ihdr_w as u64) != log_w || Some(ihdr_h as u64) != log_h {
        return Err(format!(
            "IHDR 尺寸 ({ihdr_w}x{ihdr_h}) 与 screenshot_log 回填 ({log_w:?}x{log_h:?}) 不一致"
        ));
    }
    println!(
        "[PASS] {png_path}（{} 字节，PNG {ihdr_w}x{ihdr_h}，与日志一致；status={}）",
        bytes.len(),
        entry.get("status").and_then(Value::as_str).unwrap_or("?")
    );

    // ⑥ 清理：despawn 本次 spawn 的实体。
    println!("== [6/6] world.despawn_entity（清理链路实体）==");
    client.call("world.despawn_entity", json!({ "entity": entity }))?;
    println!("[PASS] 实体 {entity} 已清理");

    Ok(())
}

/// 轮询 `game.screenshot_log` 直到目标 id 的条目 `status == "captured"`。
/// 返回该条目（含 width/height/frame_captured）。超时则 Err。
/// 受理响应缺 id 视为协议异常直接报错（不降级为「任一 captured 即命中」的假阳性路径）。
fn poll_captured(client: &mut BrpClient, shot_id: Option<u64>, timeout_secs: u64) -> Result<Value, String> {
    let shot_id = shot_id.ok_or("screenshot 受理响应缺 id，无法定位轮询目标")?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        let log = client.call("game.screenshot_log", json!({}))?;
        let entries = log
            .get("entries")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("screenshot_log 响应缺 entries：{log}"))?;
        if let Some(entry) = entries
            .iter()
            .find(|e| e.get("id").and_then(Value::as_u64) == Some(shot_id))
        {
            if entry.get("status").and_then(Value::as_str) == Some("captured") {
                return Ok(entry.clone());
            }
        }
        if std::time::Instant::now() >= deadline {
            return Err(format!(
                "截图捕获超时（{timeout_secs}s，id={shot_id}，pending 未清零）"
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

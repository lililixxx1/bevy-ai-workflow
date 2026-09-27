//! task-runner：任务测试集的自动判定驱动（自研件，M2 验收③的工具侧载体；
//! M1 两阶段判定第二阶段的脚本判定执行器）。
//!
//! 游戏外进程（不链接 bevy，SKILL.md §3.6），经 BRP over HTTP（默认
//! 127.0.0.1:15702）驱动运行中的游戏：
//!
//! - `run --suite ts-XX [--gap-secs 1] [--expect-* ...]`：调用 `game.run_tests`
//!   两次（间隔 gap-secs），逐条断言打印 + 跨调用 `snapshot.tick` 递增判定
//!   （各清单跨帧采样断言的工具侧半）；全部通过退出码 0，否则 1。
//!   `--expect-*` 为清单字面值绑定（套件内多为配置自洽口径——任意 --count
//!   启动也全绿；带参数时工具端另经 `world.get_resources` 核对启动字面值）：
//!   count / seed / max-speed（SimConfig）+ version（SimMetadata，附带
//!   `world.list_resources` 可见性断言——TS-03 清单 #1 的 BRP 侧实证）。
//! - `pause [--gap-secs 1]`：TS-04 清单全五断言的时序驱动（跨帧冻结/恢复无法
//!   进程内单调用判定）——运行中 tick 递增 → mutate paused=true（BRP 注入）→
//!   tick 冻结（容差 2 帧）→ Transform 逐位不变 → 恢复后 tick 再增长。
//! - `bench-judge --log X --brp-avg A --baseline-fps B --expect-count N`：
//!   TS-11 清单 #3/#4——解析游戏日志 `[BENCH]` 汇总行，断言 n 字面、warmup
//!   口径、avg_fps 与 BRP 读值（`--brp-avg`，取自 run --suite ts-11 的 detail）
//!   双源 ±10% 一致、≥ 基线 × 0.8。
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
                   [--expect-max-speed 9.0] [--expect-version 0.1.0]
  task-runner pause [--url http://127.0.0.1:15702] [--gap-secs 1]
  task-runner bench-judge --log <游戏日志路径> --brp-avg <BRP 读得 avg_fps>
                         --baseline-fps 60.0 --expect-count 50000 [--warmup 2.0]
  task-runner chain [--url http://127.0.0.1:15702] [--out-dir brp-chain]

run：game.run_tests 两连调 + 跨调用 snapshot.tick 递增 + 可选字面值绑定；
pause：TS-04 暂停/恢复全时序（mutate 注入经 BRP，tick 冻结/Transform 逐位不变）；
bench-judge：TS-11 的 [BENCH] 日志行解析 + 双源一致 + 基线阈值（进程退出后执行）。";

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match args.first().map(String::as_str) {
        Some("run") => cmd_run(&args[1..]),
        Some("pause") => cmd_pause(&args[1..]),
        Some("bench-judge") => cmd_bench_judge(&args[1..]),
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

/// 数值型可选参数（flag 存在但值非数字 → 报错）。
fn opt_parse<T: std::str::FromStr>(args: &[String], key: &str) -> Result<Option<T>, String> {
    opt_value(args, key)?
        .map(|s| s.parse::<T>().map_err(|_| format!("{key} 非数字：{s}")))
        .transpose()
}

fn client_from(args: &[String]) -> Result<BrpClient, String> {
    let url = opt_value(args, "--url")?
        .unwrap_or_else(|| "http://127.0.0.1:15702".into());
    BrpClient::new(&url)
}

/// `run` 模式：run_tests 两连调（间隔 gap）+ 跨调用 tick 递增
/// （+ 可选的字面值绑定：--expect-count / --expect-seed / --expect-max-speed /
/// --expect-version）。
fn cmd_run(args: &[String]) -> Result<(), String> {
    let suite = opt_value(args, "--suite")?
        .ok_or_else(|| format!("run 模式需要 --suite <名称>\n{USAGE}"))?;
    let gap: u64 = opt_parse(args, "--gap-secs")?.unwrap_or(1);
    let expect_count: Option<u64> = opt_parse(args, "--expect-count")?;
    let expect_seed: Option<u64> = opt_parse(args, "--expect-seed")?;
    let expect_max_speed: Option<f64> = opt_parse(args, "--expect-max-speed")?;
    let expect_version: Option<String> = opt_value(args, "--expect-version")?;
    let mut client = client_from(args)?;

    println!("== [1/4] game.run_tests（第 1 次调用，套件 {suite}）==");
    let (pass1, total1, tick1) = call_and_report(&mut client, &suite)?;

    println!("== [2/4] 字面值绑定（SimConfig/SimMetadata 核对：--expect-*）==");
    let literal_ok = literal_checks(
        &mut client,
        expect_count,
        expect_seed,
        expect_max_speed,
        expect_version.as_deref(),
    )?;

    println!("== [3/4] 间隔 {gap}s（tick 第二采样窗口）==");
    std::thread::sleep(std::time::Duration::from_secs(gap));

    println!("== [4/4] game.run_tests（第 2 次调用）==");
    let (pass2, total2, tick2) = call_and_report(&mut client, &suite)?;

    println!(
        "== 跨调用断言：snapshot.tick2({tick2:?}) > tick1({tick1:?})（清单跨帧采样断言的工具侧半：套件执行间隔不阻塞模拟）=="
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

/// 清单字面值绑定：经 `world.get_resources` 读 SimConfig/SimMetadata 核对期望
/// 字面值；`--expect-version` 附带 `world.list_resources` 含 SimMetadata 的可见
/// 性断言（TS-03 清单 #1 的 BRP 侧实证）。全部不提供则本环节跳过。
#[allow(clippy::too_many_arguments)]
fn literal_checks(
    client: &mut BrpClient,
    expect_count: Option<u64>,
    expect_seed: Option<u64>,
    expect_max_speed: Option<f64>,
    expect_version: Option<&str>,
) -> Result<bool, String> {
    if expect_count.is_none()
        && expect_seed.is_none()
        && expect_max_speed.is_none()
        && expect_version.is_none()
    {
        println!("-- 未提供 --expect-*，跳过（套件内为配置自洽口径）");
        return Ok(true);
    }
    let cfg = client.call(
        "world.get_resources",
        json!({ "resource": "game::sim::SimConfig" }),
    )?;
    let mut all_ok = true;

    let check_u64 = |label: &str, actual: Option<u64>, expect: Option<u64>, ok: &mut bool| {
        if let Some(e) = expect {
            let pass = actual == Some(e);
            println!("{} {label}={actual:?}（期望 {e}）", if pass { "[PASS]" } else { "[FAIL]" });
            *ok &= pass;
        }
    };
    check_u64(
        "entity_count",
        cfg.pointer("/value/entity_count").and_then(Value::as_u64),
        expect_count,
        &mut all_ok,
    );
    check_u64(
        "seed",
        cfg.pointer("/value/seed").and_then(Value::as_u64),
        expect_seed,
        &mut all_ok,
    );
    if let Some(e) = expect_max_speed {
        let actual = cfg.pointer("/value/max_speed").and_then(Value::as_f64);
        let pass = actual.is_some_and(|a| (a - e).abs() <= 1e-6);
        println!(
            "{} max_speed={actual:?}（期望 {e}，|Δ|≤1e-6）",
            if pass { "[PASS]" } else { "[FAIL]" }
        );
        all_ok &= pass;
    }
    if let Some(e) = expect_version {
        let meta = client.call(
            "world.get_resources",
            json!({ "resource": "game::sim::SimMetadata" }),
        )?;
        let actual = meta.pointer("/value/version").and_then(Value::as_str);
        let pass = actual == Some(e);
        println!(
            "{} SimMetadata.version={actual:?}（期望 {e:?}）",
            if pass { "[PASS]" } else { "[FAIL]" }
        );
        all_ok &= pass;
        // TS-03 #1：BRP list_resources 可见性实证（get 成功已证可读，再证可列）。
        let listed = client.call("world.list_resources", json!({}))?;
        let contains = listed
            .as_array()
            .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some("game::sim::SimMetadata")));
        println!(
            "{} world.list_resources 含 game::sim::SimMetadata",
            if contains { "[PASS]" } else { "[FAIL]" }
        );
        all_ok &= contains;
    }
    Ok(all_ok)
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

// ---- TS-04：暂停/恢复全时序（跨帧断言的工具侧驱动）----

/// `pause` 模式：TS-04 清单五断言——1 运行中递增 / 2 BRP 注入暂停 / 3 冻结
/// （容差 2 帧，见 docs/brp-smoke.md 帧边界竞态口径）/ 4 Transform 逐位不变 /
/// 5 恢复后再增长。全部经 BRP，单进程内完成。
fn cmd_pause(args: &[String]) -> Result<(), String> {
    let gap: u64 = opt_parse(args, "--gap-secs")?.unwrap_or(1);
    let mut client = client_from(args)?;
    let mut all_ok = true;

    println!("== [1/5] 运行中：paused==false 且 tick 递增 ==");
    let paused0 = get_config_paused(&mut client)?;
    let tick1 = get_stats_tick(&mut client)?;
    std::thread::sleep(std::time::Duration::from_secs(gap));
    let tick2 = get_stats_tick(&mut client)?;
    let running_ok = paused0 == Some(false) && matches!((tick1, tick2), (Some(a), Some(b)) if b > a);
    println!(
        "{} paused={paused0:?}，tick {tick1:?} → {tick2:?}",
        if running_ok { "[PASS]" } else { "[FAIL]" }
    );
    all_ok &= running_ok;

    println!("== [2/5] BRP 注入暂停（world.mutate_resources paused=true）==");
    client.call(
        "world.mutate_resources",
        json!({ "resource": "game::sim::SimConfig", "path": "paused", "value": true }),
    )?;
    let paused1 = get_config_paused(&mut client)?;
    let inject_ok = paused1 == Some(true);
    println!(
        "{} mutate 响应无 error，复读 paused={paused1:?}",
        if inject_ok { "[PASS]" } else { "[FAIL]" }
    );
    all_ok &= inject_ok;

    println!("== [3/5] 冻结：tick4 - tick3 <= 2（间隔 {gap}s，容差 2 帧）==");
    let tick3 = get_stats_tick(&mut client)?;
    std::thread::sleep(std::time::Duration::from_secs(gap));
    let tick4 = get_stats_tick(&mut client)?;
    let frozen_ok = matches!((tick3, tick4), (Some(a), Some(b)) if b.saturating_sub(a) <= 2);
    println!(
        "{} tick {tick3:?} → {tick4:?}",
        if frozen_ok { "[PASS]" } else { "[FAIL]" }
    );
    all_ok &= frozen_ok;

    println!("== [4/5] 暂停期间 Transform 逐位不变（两次全量采样，间隔 {gap}s）==");
    let sample1 = query_translations(&mut client)?;
    std::thread::sleep(std::time::Duration::from_secs(gap));
    let sample2 = query_translations(&mut client)?;
    let same_len = sample1.len() == sample2.len() && !sample1.is_empty();
    let bitwise_same = sample1 == sample2;
    println!(
        "{} 行数 {} / {}，逐位一致 {bitwise_same}（全量比对，强于清单「任一实体」）",
        if same_len && bitwise_same { "[PASS]" } else { "[FAIL]" },
        sample1.len(),
        sample2.len()
    );
    all_ok &= same_len && bitwise_same;

    println!("== [5/5] 恢复（paused=false）后 tick 恢复增长 ==");
    client.call(
        "world.mutate_resources",
        json!({ "resource": "game::sim::SimConfig", "path": "paused", "value": false }),
    )?;
    let paused2 = get_config_paused(&mut client)?;
    let tick5 = get_stats_tick(&mut client)?;
    std::thread::sleep(std::time::Duration::from_secs(gap));
    let tick6 = get_stats_tick(&mut client)?;
    let resumed_ok = paused2 == Some(false) && matches!((tick5, tick6), (Some(a), Some(b)) if b > a);
    println!(
        "{} paused={paused2:?}，tick {tick5:?} → {tick6:?}",
        if resumed_ok { "[PASS]" } else { "[FAIL]" }
    );
    all_ok &= resumed_ok;

    if !all_ok {
        return Err("TS-04 时序断言未全通过（见上方 [FAIL] 行）".into());
    }
    Ok(())
}

/// 读 SimConfig.paused。
fn get_config_paused(client: &mut BrpClient) -> Result<Option<bool>, String> {
    let cfg = client.call(
        "world.get_resources",
        json!({ "resource": "game::sim::SimConfig" }),
    )?;
    Ok(cfg.pointer("/value/paused").and_then(Value::as_bool))
}

/// 读 SimStats.tick。
fn get_stats_tick(client: &mut BrpClient) -> Result<Option<u64>, String> {
    let stats = client.call(
        "world.get_resources",
        json!({ "resource": "game::sim::SimStats" }),
    )?;
    Ok(stats.pointer("/value/tick").and_then(Value::as_u64))
}

/// 全量 Wanderer + Transform 采样：按 entity 排序的 (entity, translation) 列表
/// （两次采样逐位比对用；暂停期无结构变更，行序亦稳定，排序后比对与行序无关）。
fn query_translations(client: &mut BrpClient) -> Result<Vec<(u64, Vec<f64>)>, String> {
    let result = client.call(
        "world.query",
        json!({ "data": { "components": ["game::sim::Wanderer", "bevy_transform::components::transform::Transform"] } }),
    )?;
    let rows = result
        .as_array()
        .ok_or_else(|| format!("world.query 响应非数组：{result}"))?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let entity = row.get("entity").and_then(Value::as_u64);
        let translation = row
            .pointer("/components/bevy_transform::components::transform::Transform/translation")
            .and_then(Value::as_array)
            .map(|arr| arr.iter().filter_map(Value::as_f64).collect::<Vec<f64>>());
        let (Some(entity), Some(translation)) = (entity, translation) else {
            return Err(format!("query 行缺 entity/translation：{row}"));
        };
        out.push((entity, translation));
    }
    out.sort_by_key(|(entity, _)| *entity);
    Ok(out)
}

// ---- TS-11 #3/#4：[BENCH] 日志行判定（进程退出后执行）----

/// `bench-judge` 模式：解析 `[BENCH]` 汇总行并断言 n 字面 / warmup 口径 /
/// 双源一致（log avg_fps vs `--brp-avg` 的 BRP 读值，±10%）/ 基线阈值
/// （log avg_fps ≥ `--baseline-fps` × 0.8）。
fn cmd_bench_judge(args: &[String]) -> Result<(), String> {
    let log_path = opt_value(args, "--log")?
        .ok_or_else(|| format!("bench-judge 需要 --log <游戏日志路径>\n{USAGE}"))?;
    let brp_avg: f64 = opt_parse(args, "--brp-avg")?
        .ok_or_else(|| format!("bench-judge 需要 --brp-avg <BRP 读得 avg_fps>\n{USAGE}"))?;
    let baseline: f64 = opt_parse(args, "--baseline-fps")?
        .ok_or_else(|| format!("bench-judge 需要 --baseline-fps <基线 fps>\n{USAGE}"))?;
    let expect_count: u64 = opt_parse(args, "--expect-count")?
        .ok_or_else(|| format!("bench-judge 需要 --expect-count <阶梯 n>\n{USAGE}"))?;
    let warmup_expect: f64 = opt_parse(args, "--warmup")?.unwrap_or(2.0);

    let log = std::fs::read_to_string(&log_path)
        .map_err(|e| format!("读日志 {log_path} 失败：{e}"))?;
    let bench_line = log
        .lines()
        .rev()
        .find(|l| l.contains("[BENCH]"))
        .ok_or_else(|| format!("日志中未找到 [BENCH] 汇总行：{log_path}"))?;
    println!("== [BENCH] 行原文：{bench_line} ==");

    // k=v 解析（bench.rs 的 info! 按空格分隔输出）。
    let fields: std::collections::HashMap<&str, &str> = bench_line
        .split_whitespace()
        .filter_map(|tok| tok.split_once('='))
        .collect();
    let get_num = |key: &str| fields.get(key).and_then(|v| v.parse::<f64>().ok());

    let mut all_ok = true;
    let n = fields.get("n").and_then(|v| v.parse::<u64>().ok());
    let n_ok = n == Some(expect_count);
    println!(
        "{} n={n:?} == 期望 {expect_count}",
        if n_ok { "[PASS]" } else { "[FAIL]" }
    );
    all_ok &= n_ok;

    let warmup = get_num("warmup_secs");
    let warmup_ok = warmup.is_some_and(|w| (w - warmup_expect).abs() <= 1e-9);
    println!(
        "{} warmup_secs={warmup:?} == {warmup_expect}（基线口径）",
        if warmup_ok { "[PASS]" } else { "[FAIL]" }
    );
    all_ok &= warmup_ok;

    let log_avg = get_num("avg_fps");
    let dual_ok = log_avg.is_some_and(|a| (a - brp_avg).abs() / brp_avg <= 0.10);
    println!(
        "{} 双源一致：[BENCH] avg_fps={log_avg:?} vs BRP 读值 {brp_avg}（|Δ|/brp ≤ 10%）",
        if dual_ok { "[PASS]" } else { "[FAIL]" }
    );
    all_ok &= dual_ok;

    let regression_ok = log_avg.is_some_and(|a| a >= baseline * 0.8);
    println!(
        "{} 基线对照：avg_fps={log_avg:?} ≥ {baseline} × 0.8 = {}",
        if regression_ok { "[PASS]" } else { "[FAIL]" },
        baseline * 0.8
    );
    all_ok &= regression_ok;

    if !all_ok {
        return Err("TS-11 bench-judge 断言未全通过（见上方 [FAIL] 行）".into());
    }
    Ok(())
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

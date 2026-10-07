//! claim-lint：计数与宣称的机器门禁（窗口前置增强 T038 / A2）。
//!
//! 两类断言：
//! 1. **真值断言**：关键计数（错题/模式/taskset/台账/方法/套件/doctest）由本工具
//!    从仓库实况再计算，与下方 `EXPECT_*` 期望值逐一比对——期望值即「宣称」，
//!    再计算即「实证」，两者不一致即红。**联动规则**：任何任务新增条目/方法/套件/
//!    台账行时，必须同步更新对应期望值（这正是宣称被钉住的方式：改了实况不改
//!    宣称 → 红；改了宣称没改实况 → 红）。
//! 2. **一致性断言**：受管文件的自述头（「当前：N 条」）与意向文档的阈值型断言
//!    （「≥ N 条」）必须与真值一致；行内 `<!-- claim-lint:ignore 理由 -->` 豁免
//!    历史性数字，豁免行本身留在审核面内。
//!
//! doctest 门禁计数为运行时真值：候选 `docs/evidence/**/gate-doc-test*.log`
//! 按 mtime 降序**回退式挑选**（T040 S1：须能同时解析出 `test result:` 计数与
//! `REAL_EXIT=` 行才入选，防 mtime 重排选中历史残缺日志致假红；比选中日志
//! 更新的不可解析日志在报告明细中披露，不静默吞掉）。无候选日志时 SKIP
//! （不算过，汇总单独计数，要求门禁跑过才算全绿口径——本条 SKIP 不使退出码
//! 非零，但会在输出显式标注，负控/复核时人工核对此条不得为 SKIP）；有候选
//! 但零份可解析时 FAIL（非自含门禁日志，T037 教训）。
//!
//! 退出码：0 = 无 FAIL；1 = 存在 FAIL。留痕沿门禁惯例由外层追加
//! `echo "REAL_EXIT=$?" >> log`；程序自身末行亦打印 `REAL_EXIT=`（与退出码同值）——
//! 日志中两行并存时以最后一行（外层行）为准。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// 期望值表（= 受管宣称）。联动规则：实况变化时同步改这里。
mod expect {
    pub const PIT_B: usize = 51;
    /// T040 PIT-M-010 入库后 9→10；T041 收官 PIT-M-011 入库（门禁负载敏感假红）10→11。
    pub const PIT_M: usize = 11;
    pub const PAT_B: usize = 21;
    /// T040 TS-17 任务书入库后 16→17（ts-17-snapshot.md）。
    pub const TASKSET: usize = 17;
    /// T040 行写入后 39→40；T041 冻结标注行写入后 40→41（联动规则：实况 +1 行，期望同步）。
    pub const LEDGER: usize = 42;
    /// T040 `game.snapshot` 落地后 7→8（联动规则：每新增 `game.*` 方法同步）。
    pub const GAME_METHODS: usize = 8;
    /// 0.19.1 内置 BRP 方法数（docs/brp-smoke.md 实测口径）。
    pub const BRP_BUILTIN: usize = 23;
    /// T040 ts-17 套件注册后 12→13（rpc/suites/mod.rs all() 注册表）。
    pub const IN_PROC_SUITES: usize = 13;
    /// doctest 门禁计数（passed, failed, ignored）。
    pub const DOCTEST: (usize, usize, usize) = (77, 0, 17);
}

#[derive(Debug)]
enum Status {
    Pass(String),
    Fail(String),
    Skip(String),
}

struct Report {
    id: &'static str,
    desc: String,
    status: Status,
}

fn main() -> ExitCode {
    let root = repo_root();
    let mut reports: Vec<Report> = Vec::new();

    // ---- 真值计算 ----
    let pit_b = count_lines("bevy-dev/pitfalls.md", &root, |l| {
        l.starts_with("### PIT-B-0") && !l.contains("PIT-B-00X") // 模板行扣除
    });
    let pit_m = count_lines("assets-methodology/pitfalls.md", &root, |l| {
        l.starts_with("### PIT-M-") // 方法论层无模板行，不扣
    });
    let pat_b = count_files("bevy-dev/patterns", &root, |n| {
        n.starts_with("PAT-B-") && n.ends_with(".md") && n != "_TEMPLATE.md"
    });
    let taskset = count_files("assets-methodology/taskset", &root, |n| {
        n.starts_with("ts-") && n.ends_with(".md")
    });
    let ledger = count_lines("assets-methodology/task-ledger.md", &root, |l| {
        l.starts_with("| T0")
    });
    let methods = count_lines("game/src/rpc/mod.rs", &root, |l| {
        // S3（T040）：锚 `pub const` + `"game.` 字面量——不按常量名后缀过滤
        // （T002 同型教训：无 `_METHOD` 后缀的方法名常量会被漏计且整体仍绿）。
        l.contains("pub const ") && l.contains("\"game.")
    });
    let suites = count_lines("game/src/rpc/suites/mod.rs", &root, |l| {
        l.trim_start().starts_with("(\"ts-")
    });
    let brp_total = expect::BRP_BUILTIN + methods;

    let checks: Vec<(&'static str, String, usize, usize)> = vec![
        ("pit-b", "PIT-B 条数（bevy-dev/pitfalls.md 标题计数，模板行扣除）".into(), pit_b, expect::PIT_B),
        ("pit-m", "PIT-M 条数（assets-methodology/pitfalls.md 标题计数）".into(), pit_m, expect::PIT_M),
        ("pat-b", "PAT-B 条数（bevy-dev/patterns/PAT-B-*.md 文件计数）".into(), pat_b, expect::PAT_B),
        ("taskset", "taskset 条目数（assets-methodology/taskset/ts-*.md）".into(), taskset, expect::TASKSET),
        ("ledger", "台账任务行数（task-ledger.md 行首 | T0xx）".into(), ledger, expect::LEDGER),
        ("game-methods", "game.* 自定义方法数（rpc/mod.rs pub const + \"game. 字面量锚定，S3/T040）".into(), methods, expect::GAME_METHODS),
        ("suites", "进程内套件数（rpc/suites/mod.rs all() 注册表）".into(), suites, expect::IN_PROC_SUITES),
        ("brp-total", "BRP 方法总数（23 内置 + game.* 推算）".into(), brp_total, expect::BRP_BUILTIN + expect::GAME_METHODS),
    ];
    for (id, desc, actual, expected) in checks {
        let status = if actual == expected {
            Status::Pass(format!("实际 {actual} = 期望 {expected}"))
        } else {
            Status::Fail(format!("实际 {actual} ≠ 期望 {expected}（联动规则：实况变化须同步 expect 表）"))
        };
        reports.push(Report { id, desc, status });
    }

    // ---- 一致性断言：自述头「当前：N 条」----（豁免：行内 claim-lint:ignore）
    for (id, path, truth, label) in [
        ("self-pit-b", "bevy-dev/pitfalls.md", pit_b, "PIT-B"),
        ("self-pit-m", "assets-methodology/pitfalls.md", pit_m, "PIT-M"),
        ("self-pat-b", "bevy-dev/patterns/README.md", pat_b, "PAT-B"),
    ] {
        reports.push(check_self_report(&root, path, truth, label, id));
    }

    // ---- 一致性断言：意向文档阈值（M3 验收面「错题本 ≥ 50 条、模式库 ≥ 20 条」）----
    reports.push(check_intent_thresholds(&root, pit_b, pat_b));

    // ---- 运行时真值：doctest 门禁日志 ----
    reports.push(check_doctest_log(&root));

    // ---- 输出 ----
    let mut fail = 0usize;
    let mut skip = 0usize;
    for r in &reports {
        let (tag, detail) = match &r.status {
            Status::Pass(d) => ("PASS", d),
            Status::Fail(d) => ("FAIL", d),
            Status::Skip(d) => ("SKIP", d),
        };
        if matches!(r.status, Status::Fail(_)) {
            fail += 1;
        }
        if matches!(r.status, Status::Skip(_)) {
            skip += 1;
        }
        println!("[{tag}] {:<14} {}\n         {}", r.id, r.desc, detail);
    }
    println!(
        "\nclaim-lint 汇总：{} 条断言，PASS {} / FAIL {} / SKIP {}",
        reports.len(),
        reports.len() - fail - skip,
        fail,
        skip
    );
    if fail > 0 {
        println!("REAL_EXIT=1（存在 FAIL——宣称与实况不一致，先修复再收尾）");
        ExitCode::from(1)
    } else {
        println!("REAL_EXIT=0");
        ExitCode::SUCCESS
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("仓库根定位失败（claim-lint 须在仓库内编译运行）")
}

fn read_rel(root: &Path, rel: &str) -> String {
    fs::read_to_string(root.join(rel))
        .unwrap_or_else(|e| panic!("读取失败 {rel}: {e}"))
}

fn count_lines(rel: &str, root: &Path, pred: impl Fn(&str) -> bool) -> usize {
    read_rel(root, rel).lines().filter(|l| pred(l)).count()
}

fn count_files(rel_dir: &str, root: &Path, pred: impl Fn(&str) -> bool) -> usize {
    fs::read_dir(root.join(rel_dir))
        .unwrap_or_else(|e| panic!("读目录失败 {rel_dir}: {e}"))
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            pred(&name)
        })
        .count()
}

/// 行内豁免标记（含理由）：`<!-- claim-lint:ignore ... -->`。
fn exempt(line: &str) -> bool {
    line.contains("<!-- claim-lint:ignore")
}

/// 从「当前：N 条」形态提取 N；同文件多处出现时全部校验。
fn check_self_report(root: &Path, rel: &str, truth: usize, label: &str, id: &'static str) -> Report {
    let mut mismatches = Vec::new();
    let mut found = 0usize;
    for (no, line) in read_rel(root, rel).lines().enumerate() {
        if exempt(line) || !line.contains("当前：") {
            continue;
        }
        if let Some(n) = extract_first_number_after(line, "当前：") {
            found += 1;
            if n != truth {
                mismatches.push(format!("第 {} 行自述 {} 条 ≠ 真值 {}", no + 1, n, truth));
            }
        }
    }
    let status = if mismatches.is_empty() {
        if found == 0 {
            Status::Fail(format!("{rel} 未找到「当前：N 条」自述头——锚点丢失须先修锚点"))
        } else {
            Status::Pass(format!("{rel} 自述头 {found} 处 = 真值 {truth}"))
        }
    } else {
        Status::Fail(format!("{rel}：{}", mismatches.join("；")))
    };
    Report { id, desc: format!("{label} 自述头一致性（{rel}）"), status }
}

/// 意向文档 §4 M3 行的阈值断言：错题本 ≥ N、模式库 ≥ M（PIT-B 真值须 ≥ N；PAT-B ≥ M）。
fn check_intent_thresholds(root: &Path, pit_b: usize, pat_b: usize) -> Report {
    let text = read_rel(root, "Bevy-AI开发意向文档.md");
    let line = text
        .lines()
        .find(|l| l.contains("错题本 ≥") && l.contains("模式库 ≥"))
        .map(|l| l.to_string());
    let Some(line) = line else {
        return Report {
            id: "intent-threshold",
            desc: "意向文档 M3 阈值断言（错题本 ≥ 50 / 模式库 ≥ 20）".into(),
            status: Status::Fail("意向文档中未找到「错题本 ≥ … 条、模式库 ≥ … 条」锚点".into()),
        };
    };
    let want_pit = extract_first_number_after(&line, "错题本 ≥").unwrap_or(usize::MAX);
    let want_pat = extract_first_number_after(&line, "模式库 ≥").unwrap_or(usize::MAX);
    let ok = pit_b >= want_pit && pat_b >= want_pat;
    Report {
        id: "intent-threshold",
        desc: "意向文档 M3 阈值断言（错题本 ≥ 50 / 模式库 ≥ 20）".into(),
        status: if ok {
            Status::Pass(format!("PIT-B {pit_b} ≥ {want_pit}，PAT-B {pat_b} ≥ {want_pat}"))
        } else {
            Status::Fail(format!("PIT-B {pit_b} 或 PAT-B {pat_b} 跌破阈值（{want_pit}/{want_pat}）"))
        },
    }
}

/// doctest 门禁计数（S1/T040 回退式挑选）：候选日志按 mtime 降序逐一尝试
/// 解析（末处 `test result:` 计数 + `REAL_EXIT=` 行均在且可读），第一份可解析
/// 者入选；被跳过的更新日志（mtime 更大但不可解析）在明细中披露。
fn check_doctest_log(root: &Path) -> Report {
    let mut candidates = gate_log_candidates(root);
    if candidates.is_empty() {
        return Report {
            id: "doctest-gate",
            desc: "doctest 门禁计数（gate-doc-test*.log 回退式挑选）".into(),
            status: Status::Skip("docs/evidence 下未找到 gate-doc-test*.log——门禁跑过后此条不得为 SKIP".into()),
        };
    }
    let mut skipped: Vec<String> = Vec::new();
    for (path, text) in &mut candidates {
        let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string();
        let Some(((p, f, i), exit)) = parse_gate_summary(text) else {
            skipped.push(rel);
            continue;
        };
        let want = expect::DOCTEST;
        let ok = (p, f, i) == want && exit == "REAL_EXIT=0";
        let skip_note = if skipped.is_empty() {
            String::new()
        } else {
            format!("；跳过 mtime 更新的不可解析日志 {} 份：{}", skipped.len(), skipped.join(", "))
        };
        return Report {
            id: "doctest-gate",
            desc: "doctest 门禁计数（gate-doc-test*.log 回退式挑选）".into(),
            status: if ok {
                Status::Pass(format!("{rel}：{p} passed / {f} failed / {i} ignored，REAL_EXIT=0{skip_note}"))
            } else {
                Status::Fail(format!("{rel}：{p}/{f}/{i} + REAL_EXIT={exit} ≠ 期望 {want:?} + REAL_EXIT=0{skip_note}"))
            },
        };
    }
    Report {
        id: "doctest-gate",
        desc: "doctest 门禁计数（gate-doc-test*.log 回退式挑选）".into(),
        status: Status::Fail(format!(
            "候选 {} 份全部不可解析（缺 test result/REAL_EXIT 或计数不可读——非自含门禁日志，T037 教训）：{}",
            skipped.len(),
            skipped.join(", ")
        )),
    }
}

/// 从日志文本提取（计数三元组, REAL_EXIT 值）；任一缺失或不可解析 → `None`。
fn parse_gate_summary(text: &str) -> Option<((usize, usize, usize), String)> {
    let res = text.lines().filter(|l| l.starts_with("test result:")).next_back()?;
    let exit = text
        .lines()
        .filter(|l| l.starts_with("REAL_EXIT="))
        .next_back()?
        .trim()
        .to_string();
    let (p, f, i) = parse_test_result(res)?;
    Some(((p, f, i), exit))
}

/// 收集 `docs/evidence/**/gate-doc-test*.log`（mtime 降序——回退序即此序）。
fn gate_log_candidates(root: &Path) -> Vec<(PathBuf, String)> {
    let mut found: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    let mut stack = vec![root.join("docs/evidence")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("gate-doc-test"))
                .unwrap_or(false)
            {
                if let Ok(m) = fs::metadata(&p).and_then(|m| m.modified()) {
                    found.push((m, p));
                }
            }
        }
    }
    found.sort_by(|a, b| b.0.cmp(&a.0)); // mtime 降序
    found
        .into_iter()
        .map(|(_, p)| {
            let text = fs::read_to_string(&p).unwrap_or_default();
            (p, text)
        })
        .collect()
}

/// 解析 `test result: ok. 77 passed; 0 failed; 17 ignored; ...` 形态。
fn parse_test_result(line: &str) -> Option<(usize, usize, usize)> {
    let p = extract_first_number_after(line, "result")?;
    // result 后紧跟的第一个数是 passed；其后依次 failed / ignored。
    let rest_after_p = &line[line.find(&p.to_string())? + p.to_string().len()..];
    let f = rest_after_p
        .split("failed")
        .next()?
        .rsplit(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse().ok())?;
    let i = line
        .split("ignored")
        .next()?
        .rsplit(|c: char| !c.is_ascii_digit())
        .find(|s| !s.is_empty())
        .and_then(|s| s.parse().ok())?;
    Some((p, f, i))
}

fn extract_first_number_after(line: &str, marker: &str) -> Option<usize> {
    let idx = line.find(marker)? + marker.len();
    let digits: String = line[idx..]
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

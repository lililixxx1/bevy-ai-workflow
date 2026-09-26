//! 命令行参数解析（无第三方依赖，`std::env::args` 手写）。
//!
//! 参数口径（基线采集与 BRP 断言均以本文件为准）：
//! - `--count N`      动态实体数，默认 1000
//! - `--seed S`       随机种子（u64），默认 20260926（口径常量）
//! - `--max-speed F`  速度模长上限（f32，米/秒），默认 3.0
//!                    （`game::sim::DEFAULT_MAX_SPEED`，TS-02 配置化；只缩放速度
//!                    抽取，不改变 PRNG 抽取顺序）
//! - `--bench-secs S` 采集时长（秒，f32）；0 = 常驻运行（默认）
//! - `--no-vsync`     开关（无值）：窗口 present mode 用 AutoNoVsync（默认 AutoVsync）

/// 解析失败的退出码（与常见 CLI 约定一致：2 = 用法错误）。
pub const EXIT_USAGE: i32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct CliArgs {
    pub count: u32,
    pub seed: u64,
    pub max_speed: f32,
    pub bench_secs: f32,
    pub no_vsync: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            count: 1000,
            seed: 20260926,
            max_speed: crate::sim::DEFAULT_MAX_SPEED,
            bench_secs: 0.0,
            no_vsync: false,
        }
    }
}

impl CliArgs {
    /// 从进程参数解析；非法输入打印 usage 并以 `EXIT_USAGE` 退出。
    pub fn parse() -> Self {
        let mut args = Self::default();
        let mut it = std::env::args().skip(1);
        while let Some(flag) = it.next() {
            if flag == "--no-vsync" {
                args.no_vsync = true;
                continue;
            }
            let value = it.next().unwrap_or_else(|| {
                eprintln!("[CLI] 缺少 {flag} 的取值");
                std::process::exit(EXIT_USAGE);
            });
            match flag.as_str() {
                "--count" => match value.parse::<u32>() {
                    Ok(v) if v > 0 && v <= 200_000 => args.count = v,
                    _ => {
                        eprintln!("[CLI] --count 须为 1..=200000 的整数，收到 {value}");
                        std::process::exit(EXIT_USAGE);
                    }
                },
                "--seed" => match value.parse::<u64>() {
                    Ok(v) => args.seed = v,
                    Err(_) => {
                        eprintln!("[CLI] --seed 须为 u64，收到 {value}");
                        std::process::exit(EXIT_USAGE);
                    }
                },
                "--max-speed" => match value.parse::<f32>() {
                    Ok(v) if v.is_finite() && v > 0.0 => args.max_speed = v,
                    _ => {
                        eprintln!("[CLI] --max-speed 须为 > 0 的有限数字，收到 {value}");
                        std::process::exit(EXIT_USAGE);
                    }
                },
                "--bench-secs" => match value.parse::<f32>() {
                    Ok(v) if v >= 0.0 => args.bench_secs = v,
                    _ => {
                        eprintln!("[CLI] --bench-secs 须为 >= 0 的数字，收到 {value}");
                        std::process::exit(EXIT_USAGE);
                    }
                },
                other => {
                    eprintln!("[CLI] 未知参数 {other}");
                    eprint!("{}", Self::usage());
                    std::process::exit(EXIT_USAGE);
                }
            }
        }
        args
    }

    pub fn usage() -> String {
        "用法: game [--count N] [--seed S] [--max-speed F] [--bench-secs S] [--no-vsync]\n".to_string()
    }

    /// 启动横幅（一行，便于日志归档解析）。
    pub fn banner(&self) -> String {
        format!(
            "[CONFIG] game demo | count={} seed={} max_speed={} bench_secs={} vsync={} | bevy 0.19 (workspace locked)",
            self.count,
            self.seed,
            self.max_speed,
            self.bench_secs,
            if self.no_vsync { "off(AutoNoVsync)" } else { "on(AutoVsync)" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_baseline_caliber() {
        // 基线口径常量：种子 20260926、默认实体 1000、默认常驻。
        let d = CliArgs::default();
        assert_eq!(d.seed, 20260926);
        assert_eq!(d.count, 1000);
        assert_eq!(d.bench_secs, 0.0);
        // TS-02：速度上限配置化后默认仍为 3.0（原 MAX_SPEED 口径不变）。
        assert_eq!(d.max_speed, crate::sim::DEFAULT_MAX_SPEED);
        assert_eq!(d.max_speed, 3.0);
    }
}

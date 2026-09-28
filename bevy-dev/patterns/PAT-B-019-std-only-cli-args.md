# PAT-B-019-std-only-cli-args

### PAT-B-019：零依赖 CLI 解析——手写 `CliArgs::parse()` 与领域配置的直通绑定

- 日期：2026-09-28
- 适用版本：bevy 0.19.1（解析本身纯 std；录入理由：与 game 组装/基线口径强耦合）
- 分型：模式
- 通用性分级：bevy-specific（录入理由：与 game 组装/基线口径强耦合；解析本身引擎无关）
- 标签：`CLI` `std` `CliArgs` `零依赖` `seed` `vsync` `退出码`
- 源码出处：`game/src/cli.rs:13-103`（EXIT_USAGE 常量/CliArgs/parse/usage/banner）+ `game/src/main.rs:25-34`（args→SimConfig 直通）

**场景**：demo 只需 5 个标量参数（count/seed/max_speed/bench_secs/no_vsync），不值得引入 clap；且参数错误要用**非零退出码**区分（脚本化判定绑定）。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/cli.rs:13-38` 的逐字删减）：

```rust
pub const EXIT_USAGE: i32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct CliArgs {
    pub count: u32,
    pub seed: u64,
    pub max_speed: f32,
    pub bench_secs: f32,
    pub no_vsync: bool,
}

impl CliArgs {
    pub fn parse() -> Self {
        // ……逐参 std::env::args 解析；缺参/坏值 → eprintln + exit(EXIT_USAGE)
    }

    pub fn banner(&self) -> String {
        // 启动横幅：全参数回显——日志自含运行口径（基线五要素取证面）
    }
}
```

要点：①`EXIT_USAGE = 2` 常量与 shell 判定约定共享（退出码即协议）；②`banner()` 把全部参数打进 stdout 首行——每份游戏日志自含当次运行口径，证据链不用回猜命令行；③字段类型直接对齐 `SimConfig` 字段（main 里逐字段拷贝，无中间转换层）；④`PartialEq` 支持单测（cli.rs 内含解析单测）。

**为什么**：依赖面最小化（游戏运行时依赖零新增）与脚本化判定（退出码/横幅可 grep）是本仓两条硬需求——手写 5 参解析比引入 clap 更符合。

**验证证据**：
- 代码为 T003/T007 已验证任务代码的逐字删减，删减后未单独重新编译；
- 运行验证：12 任务全部以 CLI 参数启动（m1-phase2 各 runner.txt 首行横幅自含口径）；负例退出码 2 由 task-runner 与 shell 脚本判定消费。

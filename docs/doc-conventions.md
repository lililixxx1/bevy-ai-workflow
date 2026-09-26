# 文档写法规范（seed）

- 适用范围：`docs/` 下全部文档。
- 适用 Bevy 版本：**0.19**（版本升级时对照官方 migration guide 批量修订后重过 doctest）。
- 本文档自身即按此规范编写，经 `cargo test --doc -p docs` 门禁验证。

## 规则

1. 每篇文档标注适用的 Bevy 版本号。
2. 示例代码默认真实可测——doctest 会编译并运行。
3. 需要窗口 / GPU / 长运行的示例标 `no_run` 并注明理由。
4. 禁止无理由的 `ignore`（语义不符时以 `ignore` + 理由替代 `compile_fail`）。

## 示例：真实可测

纯逻辑断言，doctest 直接运行：

```rust
let locked_version = "0.19";
assert_eq!(locked_version, "0.19");
```

## 示例：no_run

引用 bevy 符号、只验证编译不运行。下例为官方 crate 文档内嵌的 hello world
（本地源码 bevy-0.19.1/src/lib.rs 第 17-30 行）。
`no_run` 理由：`.run()` 进入应用事件循环，doctest 环境不应启动长运行循环：

```rust,no_run
use bevy::prelude::*;

fn main() {
    App::new()
        .add_systems(Update, hello_world_system)
        .run();
}

fn hello_world_system() {
    println!("hello world");
}
```

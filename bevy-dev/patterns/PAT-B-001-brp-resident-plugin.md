# PAT-B-001-brp-resident-plugin

### PAT-B-001：BRP 常驻最小集成——双插件 + 显式回环绑定 + 显式反射注册

- 日期：2026-09-26
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific
- 标签：`BRP` `bevy_remote` `RemotePlugin` `127.0.0.1` `register_type` `Reflect`
- 源码出处：`game/src/brp.rs`、`game/src/sim.rs`（注册侧）、`game/Cargo.toml`（feature）

**场景**：任何要让 AI/外部进程操控运行中游戏的 Bevy 工程（工作流三的通路底座，意向文档 §5.3）。

**做法**（已过 cargo check + 运行验证）：

```rust
// game/Cargo.toml：
// [dependencies]
// bevy = { workspace = true, features = ["bevy_remote"] }
// serde = { version = "1", features = ["derive"] }

use bevy::prelude::*;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::RemotePlugin;

pub struct BrpPlugin;

impl Plugin for BrpPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RemotePlugin::default());
        // 安全约束：显式绑定回环，即使默认值已是回环（防默认值变化、防误配）。
        app.add_plugins(
            RemoteHttpPlugin::default().with_address(std::net::Ipv4Addr::LOCALHOST),
        );
        // 监听 127.0.0.1:15702（默认端口，bevy_remote-0.19.1/src/http.rs:52）。
    }
}

// 需被 BRP 操控的组件/资源（derive 形态对齐官方示例 remote/server.rs:69-70,89-91）：
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Wanderer { /* 字段 */ }

// 在 owning 插件的 build 中显式注册（不依赖 reflect_auto_register 默认链）：
// app.register_type::<Wanderer>();
```

**为什么**：①`with_address(impl Into<IpAddr>)` 是公开 builder（`bevy_remote-0.19.1/src/http.rs:167-170`），`Ipv4Addr::LOCALHOST` 经 std 的 `Into<IpAddr>` 转换；②BRP 一切组件/资源操作经 `AppTypeRegistry` 解析，未注册类型报 `Unknown component type`（`builtin_methods.rs:608-613,686`）——显式 `register_type` 无条件成立且自文档化（SKILL.md §3.4）；③BRP 无鉴权，绑定回环是安全底线（AGENTS.md 硬约束 4）。

**验证证据**：
- `cargo check --workspace --all-targets` → REAL_EXIT=0（2026-09-26）；
- 运行验证：`rpc.discover` 返回 0.19.1 全方法集；11 项 BRP 断言全过（query 计数、get/mutate_resources、spawn/despawn、相机轨道半径、解析式运动数学断言）——`docs/evidence/brp-assert-result.txt`（11/11 PASS）。

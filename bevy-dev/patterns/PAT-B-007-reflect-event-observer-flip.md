# PAT-B-007-reflect-event-observer-flip

### PAT-B-007：反射 Event + observer 资源翻转——BRP `trigger_event` 到状态生效零往返

- 日期：2026-09-27
- 适用版本：bevy 0.19.1
- 分型：模式
- 通用性分级：bevy-specific（BRP 反射事件管线与 observer 同步语义）
- 标签：`Event` `observer` `Reflect` `trigger_event` `BRP` `同步` `幂等翻转`
- 源码出处：`game/src/sim.rs:149-151`（PauseRequested 定义）+ `:168-169`（注册）+ `:283-289`（observer）

**场景**：外部操控方（BRP / 任务脚本）需要翻转引擎内开关（暂停/恢复）——不引入专门 RPC 方法，用**可反射 Event + observer** 承载：一次 `bevy/trigger_event` 调用即完成「触发 → observer 同步执行 → 资源翻转」，HTTP 响应返回前已生效。

**做法**（已过 cargo check + 运行验证；代码为 `game/src/sim.rs` 的逐字删减——doc 依据段未随块迁移，出处行号在下方要点内标注）：

```rust
// 抄自 game/src/sim.rs:149-151 + :283-289（两段间注册代码 :168-169 见下）：
#[derive(Event, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[reflect(Event, Serialize, Deserialize)]
pub struct PauseRequested;

// 注册（sim.rs:168-169）：register_type::<PauseRequested>() + add_observer(on_pause_requested)

fn on_pause_requested(_: On<PauseRequested>, mut config: ResMut<SimConfig>) {
    config.paused = !config.paused;
    info!(
        "[SIM] PauseRequested observed -> paused={}",
        config.paused
    );
}
```

要点：①Event 定义要同时 `#[derive(Event, Reflect, ...)]` + `#[reflect(Event, ...)]` 才能走 BRP `trigger_event` 反射构造（handler 逐层校验：未注册类型报 `Unknown event type`、缺 `ReflectEvent` 报 `is not reflectable`，`bevy_remote-0.19.1/src/builtin_methods.rs:327-333/:1481-1516`，核实 2026-09-26）；②observer 形态 `(_: On<E>, mut res: ResMut<R>)`——`On<E>` 与普通系统参数并用的官方形态（`examples/ecs/observers.rs:142`）；③`World::trigger` **同步**执行 observer（`bevy_ecs-0.19.1/src/observer/mod.rs:63`），翻转在 BRP HTTP 响应返回前已落资源——外部下一次 `get_resources` 必然读到新值，无需等帧；④幂等翻转（`!paused`）使同一事件可重复触发做开关循环。

**为什么**：把「引擎内副作用」收敛为一条声明式事件管线——BRP 侧零自定义方法、零手写 RPC，操控语义（谁在何时翻转了什么）以日志与事件类型留痕；同步性保证外部脚本时序确定性（发 → 收 → 读 三步闭环）。

**验证证据**：
- 代码为 T015 已验证任务代码的逐字删减，**删减后未单独重新编译**；
- 运行验证：T015 BRP `trigger_event` → `get_resources` 读回翻转后的 `paused`（发/收/读三步，`docs/evidence/` 任务记录 2026-09-26）；
- 套件回归：TS-04 pause 子命令五步时序（pause → tick 停滞 → resume → tick 恢复 → 状态核对，M1 第二阶段 `docs/evidence/m1-phase2/ts-04/`，2026-09-27）。

//! M4 战斗核心（`docs/m4-game-selection.md` §二 A 案系统席②④⑤⑥⑦）：
//! 网格与占用 / 回合机（阵营轮转）/ 移动与战斗结算（整数伤害）/ 敌方 AI
//! （两级：种子随机 / 贪心规则）/ 胜负判定（歼灭 + 占点两型）。
//!
//! 架构口径（备忘录 §一硬标准 1 的落地）：核心循环 = 「玩家动作 → 离散结算 →
//! 新状态」，全部**同步**发生在 BRP handler 的独占 `&mut World` 执行内
//! （[`crate::rpc`] 家族同通路，SKILL.md §3.3 认可的 `&mut World` 场景）——
//! 调度器不参与规则面，帧边界竞态与调度序不确定性天然出局。敌方回合在
//! `game.end_turn` 内一次算完（`PHASE_ENEMY` 仅存在于该调用执行期间，BRP
//! 两个请求之间不可观测）。表现层（席⑧）只读本模块组件，不进规则面。
//!
//! 确定性纪律（备忘录 §一两条落地）：
//! - **迭代序**：一切多单位遍历先按业务键 `(x, y)` 排序快照再处理（同格唯一，
//!   占用即约束），不依赖 query 迭代序（PIT-B-010：迭代序不保证且实测非插入序）；
//! - **RNG**：规则面随机只走 [`crate::rng::SplitMix64`] 整数形态 `next_u64`
//!   取模映射（备忘录口径「整数取数」；取模微偏差不影响「同种子 + 同操作
//!   序列 → 同状态序列」的可复现性要求），进度存 [`BattleState::rng_state`]
//!   跨请求续跑；浮点形态仅表现层可用。
//!
//! 数值口径：伤害/HP/距离全整数。伤害 = 攻方 `attack` 固定值（无防御/命中
//! 变量——小品面刻意收窄）；移动距离与攻击距离（贴身 1 格）均为曼哈顿距离；
//! 移动只校验**终点**在范围内且空闲（不做寻路——9×9 小品的显式简化，穿越
//! 中途占用格允许，备忘录席⑤注记）。
//!
//! 错误语义面：规则拒绝一律 `Err(String)` → handler 转 JSON-RPC -32602，
//! 全程不 panic（备忘录可玩性定义「自研 `game.*` 错误路径不击穿进程」）。
//! 命令寻址用业务键 `(x, y)` 而非实体号——上游 BRP `world.mutate_components`
//! 坏实体裸 panic（PIT-B-051）一类的入口风险由此天然规避。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::level::{GridPos, LevelState, Unit};
use crate::rng::SplitMix64;

/// 回合阶段：玩家行动（BRP 可观测的稳态）。
pub const PHASE_PLAYER: u8 = 0;
/// 回合阶段：敌方行动。仅存在于 `game.end_turn` 执行期间（同步结算），
/// 两个 BRP 请求之间不可观测——保留常量供文档与响应解释。
pub const PHASE_ENEMY: u8 = 1;
/// 回合阶段：终局（winner 已定）。
pub const PHASE_OVER: u8 = 2;

/// 关卡目标：歼灭全部敌方。
pub const GOAL_ANNIHILATE: u8 = 0;
/// 关卡目标：任一玩家单位到达目标格（`goal_x`/`goal_y`）。
pub const GOAL_REACH: u8 = 1;

/// 敌方 AI 档位：种子随机（见 [`run_enemy_turn`]）。
pub const AI_RANDOM: u8 = 0;
/// 敌方 AI 档位：贪心规则（击杀优先 → 最低血量 → 业务键）。
pub const AI_GREEDY: u8 = 1;

/// 未加载关卡时 `game.launch_level` 的默认战斗种子（备忘录「同种子同行为」
/// 的缺省口径；驱动侧应显式传 seed 以锁死重放基线）。
pub const DEFAULT_SEED: u64 = 20260928;

/// 单位本回合行动标记。仅玩家回合语义面使用（敌方回合在 `end_turn` 内
/// 原子结算，不消费标记）；`game.end_turn` 返回玩家阶段前全量复位。
#[derive(Component, Reflect, Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[reflect(Component, Serialize, Deserialize)]
pub struct ActionFlags {
    pub moved: bool,
    pub attacked: bool,
}

/// 战斗全局状态（`game.launch_level` 写入，三条战斗 RPC 读写；BRP 经
/// `world.get_resources` 可读——同 [`crate::level::LevelState`] 的 -23501
/// 口径：未加载关卡时资源实例不存在，报错而非 null）。
/// 数值口径全整数/标量（u8 枚举值配常量，避免 enum 反射序列化面）。
#[derive(Resource, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct BattleState {
    /// [`PHASE_PLAYER`] / [`PHASE_ENEMY`]（瞬态）/ [`PHASE_OVER`]。
    pub phase: u8,
    /// 回合数（玩家回合计，从 1 起；`end_turn` 递增）。
    pub turn: u32,
    /// 胜方：`-1` 未定 / `0` 玩家 / `1` 敌方。
    pub winner: i8,
    /// [`GOAL_ANNIHILATE`] / [`GOAL_REACH`]。
    pub goal_kind: u8,
    /// 占点目标格（歼灭目标时无意义，恒 `-1`）。
    pub goal_x: i32,
    pub goal_y: i32,
    /// 本局种子（敌方 AI 唯一随机源）。
    pub seed: u64,
    /// RNG 当前状态（[`SplitMix64::state`]；跨请求续跑，重放比对的数据面之一）。
    pub rng_state: u64,
    /// [`AI_RANDOM`] / [`AI_GREEDY`]（关卡表数据）。
    pub ai_kind: u8,
}

/// 单位业务视图（快照；一切规则逻辑按 `(x, y)` 排序使用——PIT-B-010）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitView {
    pub entity: Entity,
    pub pos: GridPos,
    pub unit: Unit,
    pub flags: ActionFlags,
}

/// 全量单位按业务键 `(x, y)` 升序（同格被占用约束保证键唯一）。
pub fn units_sorted(world: &mut World) -> Vec<UnitView> {
    let mut q = world.query::<(Entity, &GridPos, &Unit, &ActionFlags)>();
    let mut v: Vec<UnitView> = q
        .iter(world)
        .map(|(entity, pos, unit, flags)| UnitView {
            entity,
            pos: *pos,
            unit: *unit,
            flags: *flags,
        })
        .collect();
    v.sort_by_key(|u| (u.pos.x, u.pos.y));
    v
}

/// 曼哈顿距离（整数；移动与攻击距离口径）。
pub fn manhattan(a: GridPos, b: GridPos) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

/// `(x, y)` 位置上的单位（占用即唯一）。
pub fn unit_at(world: &mut World, x: i32, y: i32) -> Option<UnitView> {
    units_sorted(world)
        .into_iter()
        .find(|u| u.pos.x == x && u.pos.y == y)
}

/// 未加载关卡时的统一拒绝语（三条战斗 RPC 共用前置）。
fn require_battle(world: &mut World) -> Result<(BattleState, u32, u32), String> {
    let Some(st) = world.get_resource::<BattleState>() else {
        return Err("未加载关卡（先 game.launch_level）".into());
    };
    let lvl = world
        .get_resource::<LevelState>()
        .ok_or("未加载关卡（缺 LevelState）")?;
    Ok((st.clone(), lvl.width, lvl.height))
}

/// 胜负判定（幂等；每次可能改变存亡/占位的状态变更后调用）：
/// 玩家全灭 → 敌方胜；敌方全灭 → 玩家胜；占点目标且玩家单位在目标格 → 玩家胜。
pub fn check_victory(world: &mut World) {
    let Some(st) = world.get_resource::<BattleState>().cloned() else {
        return;
    };
    if st.winner != -1 {
        return;
    }
    let units = units_sorted(world);
    let players = units.iter().filter(|u| u.unit.team == 0).count();
    let enemies = units.iter().filter(|u| u.unit.team == 1).count();
    let reach_hit = st.goal_kind == GOAL_REACH
        && units
            .iter()
            .any(|u| u.unit.team == 0 && u.pos.x == st.goal_x && u.pos.y == st.goal_y);
    if players == 0 || enemies == 0 || reach_hit {
        let mut st = world.resource_mut::<BattleState>();
        st.winner = if players == 0 { 1 } else { 0 };
        st.phase = PHASE_OVER;
    }
}

/// [`crate::rpc::move_unit`] 的移动结算结果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveOutcome {
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub winner: i8,
    pub phase: u8,
}

/// 玩家单位移动（席⑤）。规则与拒绝语见模块文档；成功副作用 =
/// `GridPos` 改写 + `ActionFlags::moved = true` + 胜负判定（占点型）。
pub fn try_move(world: &mut World, from: (i32, i32), to: (i32, i32)) -> Result<MoveOutcome, String> {
    let (st, width, height) = require_battle(world)?;
    if st.winner != -1 {
        return Err(format!("终局（winner={}），不可再操作；重开请 game.launch_level", st.winner));
    }
    let actor = unit_at(world, from.0, from.1).ok_or_else(|| {
        format!("起点 ({},{}) 无单位", from.0, from.1)
    })?;
    if actor.unit.team != 0 {
        return Err(format!("起点 ({},{}) 是敌方单位，不可由玩家操作", from.0, from.1));
    }
    if actor.flags.moved {
        return Err(format!("单位 ({},{}) 本回合已移动", from.0, from.1));
    }
    if to.0 < 0 || to.1 < 0 || to.0 >= width as i32 || to.1 >= height as i32 {
        return Err(format!(
            "终点 ({},{}) 越界（战场 {}x{}）",
            to.0, to.1, width, height
        ));
    }
    if unit_at(world, to.0, to.1).is_some() {
        return Err(format!("终点 ({},{}) 已被占用", to.0, to.1));
    }
    let dist = manhattan(actor.pos, GridPos { x: to.0, y: to.1 });
    if dist > actor.unit.move_range {
        return Err(format!(
            "超出移动范围：曼哈顿距离 {} > {} 格",
            dist, actor.unit.move_range
        ));
    }
    if let Some(mut pos) = world.get_mut::<GridPos>(actor.entity) {
        pos.x = to.0;
        pos.y = to.1;
    }
    if let Some(mut flags) = world.get_mut::<ActionFlags>(actor.entity) {
        flags.moved = true;
    }
    check_victory(world);
    let st = world.resource::<BattleState>();
    Ok(MoveOutcome {
        from,
        to,
        winner: st.winner,
        phase: st.phase,
    })
}

/// [`crate::rpc::attack`] 的攻击结算结果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttackOutcome {
    pub from: (i32, i32),
    pub target: (i32, i32),
    pub damage: i32,
    /// 结算后目标 HP（死亡为 0）。
    pub target_hp: i32,
    pub target_alive: bool,
    pub winner: i8,
    pub phase: u8,
}

/// 玩家单位攻击（席⑤）：贴身（曼哈顿距离 1）整数伤害 = 攻方 `attack`；
/// 目标 HP ≤ 0 即 despawn；随后胜负判定（歼灭型在此翻转）。
pub fn try_attack(
    world: &mut World,
    from: (i32, i32),
    target: (i32, i32),
) -> Result<AttackOutcome, String> {
    let (st, _, _) = require_battle(world)?;
    if st.winner != -1 {
        return Err(format!("终局（winner={}），不可再操作；重开请 game.launch_level", st.winner));
    }
    let actor =
        unit_at(world, from.0, from.1).ok_or_else(|| format!("起点 ({},{}) 无单位", from.0, from.1))?;
    if actor.unit.team != 0 {
        return Err(format!("起点 ({},{}) 是敌方单位，不可由玩家操作", from.0, from.1));
    }
    if actor.flags.attacked {
        return Err(format!("单位 ({},{}) 本回合已攻击", from.0, from.1));
    }
    let victim = unit_at(world, target.0, target.1)
        .ok_or_else(|| format!("目标 ({},{}) 无单位", target.0, target.1))?;
    if victim.unit.team == actor.unit.team {
        return Err(format!("目标 ({},{}) 是友方单位", target.0, target.1));
    }
    let dist = manhattan(actor.pos, victim.pos);
    if dist != 1 {
        return Err(format!("攻击须贴身（曼哈顿距离 1），当前 {dist} 格"));
    }
    let damage = actor.unit.attack;
    let hp_after = victim.unit.hp - damage;
    let alive = hp_after > 0;
    if let Some(mut u) = world.get_mut::<Unit>(victim.entity) {
        u.hp = hp_after;
    }
    if let Some(mut flags) = world.get_mut::<ActionFlags>(actor.entity) {
        flags.attacked = true;
    }
    if !alive {
        let _ = world.despawn(victim.entity);
    }
    check_victory(world);
    let st = world.resource::<BattleState>();
    Ok(AttackOutcome {
        from,
        target,
        damage,
        target_hp: hp_after.max(0),
        target_alive: alive,
        winner: st.winner,
        phase: st.phase,
    })
}

/// 敌方回合中的单个敌方行动（`game.end_turn` 响应的数据面，重放比对的
/// 操作序列载体之一）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnemyAction {
    /// 行动前位置。
    pub enemy: (i32, i32),
    pub kind: EnemyActionKind,
    /// 移动格（未移动为 `None`）。
    pub to: Option<(i32, i32)>,
    /// 攻击目标格（未攻击为 `None`）。
    pub target: Option<(i32, i32)>,
    pub damage: i32,
    pub target_alive: bool,
}

/// 行动分类（`as_str` 进 JSON）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyActionKind {
    Attack,
    Move,
    MoveAttack,
    Pass,
}

impl EnemyActionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EnemyActionKind::Attack => "attack",
            EnemyActionKind::Move => "move",
            EnemyActionKind::MoveAttack => "move_attack",
            EnemyActionKind::Pass => "pass",
        }
    }
}

/// [`crate::rpc::end_turn`] 的结算结果。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndTurnOutcome {
    /// 新回合号（原值 +1）。
    pub turn: u32,
    pub actions: Vec<EnemyAction>,
    pub winner: i8,
    pub phase: u8,
}

/// 结束玩家回合（席④阵营轮转）：复位玩家行动标记 → 敌方全队依 AI 档位
/// 行动（席⑥，按业务键序）→ 胜负判定 → 回到玩家阶段（终局则 [`PHASE_OVER`]）。
pub fn end_turn(world: &mut World) -> Result<EndTurnOutcome, String> {
    let (st, _, _) = require_battle(world)?;
    if st.winner != -1 {
        return Err(format!("终局（winner={}），不可再操作；重开请 game.launch_level", st.winner));
    }
    // 复位玩家标记（敌方标记不消费，见 ActionFlags 文档）。
    for v in units_sorted(world) {
        if v.unit.team == 0 {
            if let Some(mut flags) = world.get_mut::<ActionFlags>(v.entity) {
                flags.moved = false;
                flags.attacked = false;
            }
        }
    }
    // 瞬态敌方阶段：同步结算期间在场（如未来加观察者/日志可感知），调用
    // 返回前必离开——未分胜负回玩家阶段，分胜负由 check_victory 落 PHASE_OVER。
    world.resource_mut::<BattleState>().phase = PHASE_ENEMY;
    let actions = run_enemy_turn(world);
    check_victory(world);
    let turn = world.resource::<BattleState>().turn + 1;
    {
        let mut st = world.resource_mut::<BattleState>();
        st.turn = turn;
        if st.winner == -1 {
            st.phase = PHASE_PLAYER;
        }
    }
    let st = world.resource::<BattleState>();
    Ok(EndTurnOutcome {
        turn,
        actions,
        winner: st.winner,
        phase: st.phase,
    })
}

/// 敌方全队行动（席⑥）。档位：
/// - [`AI_RANDOM`]（种子随机）：未贴身时在可达空闲格（含原地）均匀抽取
///   （`next_u64 % 选项数`）；贴身时在相邻玩家单位中均匀抽取目标。
/// - [`AI_GREEDY`]（贪心规则）：未贴身时选「能贴身 > 离最近玩家最近 > 业务键
///   (x, y) 最小」的格子（含原地）；攻击目标「本击可击杀 > HP 最低 > 业务键最小」。
/// 共同约束：贴身时不移动（先攻后动不值得——小品面定案）；一次移动 + 一次攻击。
fn run_enemy_turn(world: &mut World) -> Vec<EnemyAction> {
    let mut actions = Vec::new();
    let (ai_kind, rng_state) = {
        let st = world.resource::<BattleState>();
        (st.ai_kind, st.rng_state)
    };
    let mut rng = SplitMix64::from_state(rng_state);
    // 敌方快照按业务键序（PIT-B-010）；敌方单位不会在本函数内死亡（敌不攻敌），
    // 快照实体全程有效。玩家侧状态逐敌重查（玩家单位会阵亡）。
    let enemies: Vec<(Entity, GridPos, Unit)> = units_sorted(world)
        .into_iter()
        .filter(|v| v.unit.team == 1)
        .map(|v| (v.entity, v.pos, v.unit))
        .collect();
    for (entity, pos, unit) in enemies {
        if !units_sorted(world).iter().any(|v| v.unit.team == 0) {
            break; // 玩家全灭：后续敌人不再行动（胜负由 end_turn 收口判定）
        }
        let mut action = EnemyAction {
            enemy: (pos.x, pos.y),
            kind: EnemyActionKind::Pass,
            to: None,
            target: None,
            damage: 0,
            target_alive: true,
        };
        let mut cur = pos;
        // 贴身即攻，不移动（两档共同定案，见模块文档）。
        if let Some(victim) = pick_attack_target(world, cur, ai_kind, unit.attack, &mut rng) {
            let (damage, alive) = apply_enemy_attack(world, unit.attack, &victim);
            action.kind = EnemyActionKind::Attack;
            action.target = Some((victim.pos.x, victim.pos.y));
            action.damage = damage;
            action.target_alive = alive;
        } else {
            // 未贴身：择格移动（None = 原地/无处可去）。
            let dest = match ai_kind {
                AI_RANDOM => random_dest(world, cur, unit.move_range, &mut rng),
                _ => greedy_dest(world, cur, unit.move_range),
            };
            if let Some(d) = dest {
                if d != cur {
                    if let Some(mut p) = world.get_mut::<GridPos>(entity) {
                        p.x = d.x;
                        p.y = d.y;
                    }
                    cur = d;
                    action.to = Some((d.x, d.y));
                    action.kind = EnemyActionKind::Move;
                }
                // 移动后贴身则攻击。
                if let Some(victim) =
                    pick_attack_target(world, cur, ai_kind, unit.attack, &mut rng)
                {
                    let (damage, alive) = apply_enemy_attack(world, unit.attack, &victim);
                    action.kind = EnemyActionKind::MoveAttack;
                    action.target = Some((victim.pos.x, victim.pos.y));
                    action.damage = damage;
                    action.target_alive = alive;
                }
            }
        }
        actions.push(action);
    }
    world.resource_mut::<BattleState>().rng_state = rng.state();
    actions
}

/// 相邻玩家单位中选攻击目标（无相邻返回 `None`；候选集按业务键序）。
fn pick_attack_target(
    world: &mut World,
    from: GridPos,
    ai_kind: u8,
    attack: i32,
    rng: &mut SplitMix64,
) -> Option<UnitView> {
    let adjacent: Vec<UnitView> = units_sorted(world)
        .into_iter()
        .filter(|v| v.unit.team == 0 && manhattan(from, v.pos) == 1)
        .collect();
    if adjacent.is_empty() {
        return None;
    }
    match ai_kind {
        AI_RANDOM => {
            let idx = (rng.next_u64() % adjacent.len() as u64) as usize;
            adjacent.get(idx).copied()
        }
        _ => adjacent
            .iter()
            .min_by_key(|v| {
                (
                    if v.unit.hp <= attack { 0 } else { 1 },
                    v.unit.hp,
                    v.pos.x,
                    v.pos.y,
                )
            })
            .copied(),
    }
}

/// 空闲可达格（在场内、曼哈顿距离 ≤ `range`、未被占用；**y 主序**——枚举
/// 序即随机档 RNG 的取模映射序），**不含原地**（原地作为移动选项由调用方
/// 另行并入）。未加载关卡返回空（调用链已被 `require_battle` 守卫，此路
/// 防御性兜底，不自造默认场界）。
///
/// **枚举序是载荷性的**（R1 审核 N2）：`random_dest` 的 `next_u64 % pool`
/// 直接在此序上取索引——顺序即重放基线的一部分，「顺手」改成 (x, y)
/// 业务键序会静默改写全部随机档重放基线（跨进程一致性不受影响，因两侧
/// 同序；受影响的是与既有基线的可比性）。
fn free_cells_within(world: &mut World, from: GridPos, range: i32) -> Vec<GridPos> {
    let Some(lvl) = world.get_resource::<LevelState>() else {
        return Vec::new();
    };
    let (width, height) = (lvl.width as i32, lvl.height as i32);
    let mut cells = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let c = GridPos { x, y };
            if manhattan(from, c) <= range && c != from && unit_at(world, x, y).is_none() {
                cells.push(c);
            }
        }
    }
    cells
}

/// 随机档目的地：可达空闲格 + 原地，均匀抽取（返回 `None` = 原地）。
fn random_dest(
    world: &mut World,
    from: GridPos,
    range: i32,
    rng: &mut SplitMix64,
) -> Option<GridPos> {
    let cells = free_cells_within(world, from, range);
    if cells.is_empty() {
        return None; // 无处可去 = 原地
    }
    let pool = cells.len() as u64 + 1; // +1 = 原地
    let idx = rng.next_u64() % pool;
    if idx == cells.len() as u64 {
        None
    } else {
        cells.get(idx as usize).copied()
    }
}

/// 贪心档目的地：可达空闲格 ∪ {原地}，按
/// `(不贴身任何玩家, 到最近玩家距离, x, y)` 字典序取最小。
fn greedy_dest(world: &mut World, from: GridPos, range: i32) -> Option<GridPos> {
    let players: Vec<GridPos> = units_sorted(world)
        .into_iter()
        .filter(|v| v.unit.team == 0)
        .map(|v| v.pos)
        .collect();
    if players.is_empty() {
        return None;
    }
    let score = |c: GridPos| {
        let dmin = players
            .iter()
            .map(|p| manhattan(c, *p))
            .min()
            .unwrap_or(i32::MAX);
        let adjacent = players.iter().any(|p| manhattan(c, *p) == 1);
        (if adjacent { 0 } else { 1 }, dmin, c.x, c.y)
    };
    let mut candidates = free_cells_within(world, from, range);
    candidates.push(from); // 原地参与比较
    candidates.into_iter().min_by_key(|c| score(*c))
}

/// 敌方攻击结算（整数伤害 = `attack`；目标阵亡即 despawn）。返回 `(伤害, 目标存活)`。
fn apply_enemy_attack(world: &mut World, attack: i32, victim: &UnitView) -> (i32, bool) {
    let hp_after = victim.unit.hp - attack;
    let alive = hp_after > 0;
    if let Some(mut u) = world.get_mut::<Unit>(victim.entity) {
        u.hp = hp_after;
    }
    if !alive {
        let _ = world.despawn(victim.entity);
    }
    (attack, alive)
}

/// 战斗域插件：类型注册（规则面无调度系统——离散结算全在 BRP 独占执行内，
/// 表现层只读组件；架构口径见模块文档）。
pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BattleState>()
            .register_type::<ActionFlags>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试底座：手植布阵（不经 launch handler——规则函数单测与 RPC 壳解耦）。
    /// 元组 = (team, x, y)；玩家 hp10/mv3/atk2、敌方 hp8/mv2/atk2（沿 L1 数值）。
    fn setup(units: &[(u8, i32, i32)], ai_kind: u8, seed: u64) -> World {
        let mut world = World::new();
        for (team, x, y) in units {
            let (hp, mv, atk) = if *team == 0 { (10, 3, 2) } else { (8, 2, 2) };
            world.spawn((
                GridPos { x: *x, y: *y },
                Unit { team: *team, hp, move_range: mv, attack: atk },
                ActionFlags::default(),
            ));
        }
        world.insert_resource(LevelState {
            level: 1,
            width: 9,
            height: 9,
            units_spawned: units.len() as u32,
        });
        world.insert_resource(BattleState {
            phase: PHASE_PLAYER,
            turn: 1,
            winner: -1,
            goal_kind: GOAL_ANNIHILATE,
            goal_x: -1,
            goal_y: -1,
            seed,
            rng_state: seed,
            ai_kind,
        });
        world
    }

    fn view(world: &mut World) -> Vec<(u8, i32, i32, i32)> {
        units_sorted(world)
            .into_iter()
            .map(|v| (v.unit.team, v.pos.x, v.pos.y, v.unit.hp))
            .collect()
    }

    #[test]
    fn move_rules_and_flags() {
        let mut w = setup(&[(0, 1, 1), (0, 2, 1), (1, 5, 7)], AI_RANDOM, 1);
        // 正常移动：范围 3、终点空闲。
        let o = try_move(&mut w, (1, 1), (1, 3)).expect("合法移动");
        assert_eq!(o.to, (1, 3));
        assert_eq!(unit_at(&mut w, 1, 3).unwrap().unit.team, 0);
        // 已移动 → 拒绝。
        assert!(try_move(&mut w, (1, 3), (1, 1)).is_err());
        // 超出范围（曼哈顿 4 > 3）→ 拒绝。
        assert!(try_move(&mut w, (2, 1), (2, 5)).is_err());
        // 终点被占用 → 拒绝。
        assert!(try_move(&mut w, (2, 1), (1, 3)).is_err());
        // 越界 → 拒绝。
        assert!(try_move(&mut w, (2, 1), (9, 1)).is_err());
        assert!(try_move(&mut w, (2, 1), (2, -1)).is_err());
        // 敌方单位不可由玩家操作。
        assert!(try_move(&mut w, (5, 7), (5, 6)).is_err());
        // 空起点。
        assert!(try_move(&mut w, (0, 0), (0, 1)).is_err());
    }

    #[test]
    fn attack_kill_and_annihilate_victory() {
        let mut w = setup(&[(0, 1, 1), (1, 2, 1)], AI_RANDOM, 1);
        // 不相邻（距离 1？(1,1)-(2,1) 恰相邻）：先验友方/距离错误分支。
        assert!(try_attack(&mut w, (1, 1), (1, 1)).is_err()); // 友方（自身）
        // 连续攻击至阵亡：hp 8 - 2*4 = 0 → despawn + 玩家胜（歼灭）。
        let mut last = None;
        for i in 0..4 {
            last = try_attack(&mut w, (1, 1), (2, 1)).ok();
            if i < 3 {
                // 每回合只能攻击一次：第二击须先换回合——此处单测直接复位标记。
                let e = unit_at(&mut w, 1, 1).unwrap().entity;
                if let Some(mut f) = w.get_mut::<ActionFlags>(e) {
                    f.attacked = false;
                }
            }
        }
        let last = last.expect("末次攻击成功");
        assert!(!last.target_alive);
        assert_eq!(last.target_hp, 0);
        assert_eq!(unit_at(&mut w, 2, 1), None);
        let st = w.resource::<BattleState>();
        assert_eq!(st.winner, 0);
        assert_eq!(st.phase, PHASE_OVER);
        // 终局后任何战斗指令拒绝（不 panic）。
        assert!(try_move(&mut w, (1, 1), (1, 2)).is_err());
        assert!(try_attack(&mut w, (1, 1), (1, 2)).is_err());
        assert!(end_turn(&mut w).is_err());
    }

    #[test]
    fn attack_distance_once_per_turn_and_empty_target() {
        let mut w = setup(&[(0, 1, 1), (1, 3, 1)], AI_RANDOM, 1);
        // 距离 2 → 拒绝（须贴身 1 格）。
        assert!(try_attack(&mut w, (1, 1), (3, 1)).is_err());
        // 目标格无单位 → 拒绝。
        assert!(try_attack(&mut w, (1, 1), (2, 1)).is_err());
        // 贴身成功一次后，同回合第二击 → 拒绝。
        let o = try_attack(&mut w, (1, 1), (2, 1)).ok();
        assert!(o.is_none()); // (2,1) 无单位——占位断言，真正的一击在下面的布局
        let mut w2 = setup(&[(0, 1, 1), (1, 2, 1)], AI_RANDOM, 1);
        let o2 = try_attack(&mut w2, (1, 1), (2, 1)).expect("贴身一击");
        assert_eq!(o2.damage, 2);
        assert_eq!(o2.target_hp, 6);
        assert!(o2.target_alive);
        assert!(try_attack(&mut w2, (1, 1), (2, 1)).is_err());
    }

    #[test]
    fn end_turn_resets_flags_advances_turn_and_rng() {
        let mut w = setup(&[(0, 1, 1), (1, 5, 7), (1, 6, 7)], AI_RANDOM, 42);
        try_move(&mut w, (1, 1), (1, 2)).expect("先耗掉本回合移动");
        let o = end_turn(&mut w).expect("end_turn");
        assert_eq!(o.turn, 2);
        let st = w.resource::<BattleState>();
        assert_eq!(st.phase, PHASE_PLAYER);
        assert_eq!(st.winner, -1);
        assert_ne!(st.rng_state, st.seed, "随机档至少一次抽取后 RNG 状态须推进");
        // 玩家标记复位：可再次移动。
        assert!(try_move(&mut w, (1, 2), (1, 3)).is_ok());
    }

    /// 确定性主证（备忘录 §一）：同业务布阵 + 同种子，即使**生成顺序不同**，
    /// 同一操作序列（连打 end_turn）后状态逐位一致——迭代序纪律的直接检验
    /// （PIT-B-010：若任何分支依赖 query 迭代序，本测试必红）。
    #[test]
    fn end_turn_deterministic_across_spawn_order() {
        let layout = [(0, 1, 1), (0, 2, 1), (0, 3, 1), (1, 5, 7), (1, 6, 7), (1, 7, 7)];
        let mut a = setup(&layout, AI_RANDOM, 20260928);
        let mut reversed: Vec<(u8, i32, i32)> = layout.iter().rev().copied().collect();
        let mut b = setup(&mut reversed, AI_RANDOM, 20260928);
        for _ in 0..5 {
            end_turn(&mut a).expect("a end_turn");
            end_turn(&mut b).expect("b end_turn");
        }
        assert_eq!(view(&mut a), view(&mut b));
        let sa = a.resource::<BattleState>().clone();
        let sb = b.resource::<BattleState>().clone();
        assert_eq!(sa, sb);
    }

    #[test]
    fn greedy_ai_closes_distance_and_prefers_killable() {
        // 距离收敛：敌 (5,5) mv2 → 玩家 (1,1)：贪心移动后到最近玩家距离必减。
        let mut w = setup(&[(0, 1, 1), (1, 5, 5)], AI_GREEDY, 1);
        end_turn(&mut w).expect("end_turn");
        let d0 = (5 - 1) + (5 - 1);
        let d1 = units_sorted(&mut w)
            .into_iter()
            .find(|v| v.unit.team == 1)
            .map(|v| manhattan(v.pos, GridPos { x: 1, y: 1 }))
            .unwrap();
        assert!(d1 < d0, "贪心敌须靠近（{d0} → {d1}）");
        // 击杀优先：两玩家贴身（hp 10 与 hp 8——后者可被一击致死 atk=8…测试敌 atk=2
        // 不可击杀，故改验「最低 HP 优先」次级键：贴身双目标，打 hp 较低者）。
        let mut w2 = World::new();
        w2.spawn((
            GridPos { x: 3, y: 3 },
            Unit { team: 0, hp: 10, move_range: 3, attack: 2 },
            ActionFlags::default(),
        ));
        w2.spawn((
            GridPos { x: 5, y: 3 },
            Unit { team: 0, hp: 9, move_range: 3, attack: 2 },
            ActionFlags::default(),
        ));
        w2.spawn((
            GridPos { x: 4, y: 3 },
            Unit { team: 1, hp: 8, move_range: 2, attack: 2 },
            ActionFlags::default(),
        ));
        w2.insert_resource(LevelState { level: 1, width: 9, height: 9, units_spawned: 3 });
        w2.insert_resource(BattleState {
            phase: PHASE_PLAYER, turn: 1, winner: -1, goal_kind: GOAL_ANNIHILATE,
            goal_x: -1, goal_y: -1, seed: 1, rng_state: 1, ai_kind: AI_GREEDY,
        });
        let o = end_turn(&mut w2).expect("end_turn");
        // 贴身即攻：kind=attack，且目标必是 hp9 的 (5,3)（同为不可击杀时取 HP 低者）。
        assert_eq!(o.actions.len(), 1);
        assert_eq!(o.actions[0].kind, EnemyActionKind::Attack);
        assert_eq!(o.actions[0].target, Some((5, 3)));
        assert_eq!(o.actions[0].damage, 2);
        assert_eq!(unit_at(&mut w2, 5, 3).unwrap().unit.hp, 7);
    }

    #[test]
    fn reach_goal_wins_on_move() {
        // R1 审核 B1：必须带敌——零敌时 check_victory 的 enemies==0 分支先行
        // 成立，占点路径无区分力（远角敌不参与本测试的判定路径）。
        let mut w = setup(&[(0, 3, 4), (1, 8, 8)], AI_RANDOM, 1);
        w.resource_mut::<BattleState>().goal_kind = GOAL_REACH;
        w.resource_mut::<BattleState>().goal_x = 4;
        w.resource_mut::<BattleState>().goal_y = 4;
        let o = try_move(&mut w, (3, 4), (4, 4)).expect("占点合法移动");
        assert_eq!(o.winner, 0);
        assert_eq!(o.phase, PHASE_OVER);
        // 负控：同阵容移动**不**踏目标格 → 不触发胜负（区分力证明）。
        let mut w2 = setup(&[(0, 3, 4), (1, 8, 8)], AI_RANDOM, 1);
        w2.resource_mut::<BattleState>().goal_kind = GOAL_REACH;
        w2.resource_mut::<BattleState>().goal_x = 4;
        w2.resource_mut::<BattleState>().goal_y = 4;
        let n = try_move(&mut w2, (3, 4), (2, 4)).expect("合法移动（不踏目标）");
        assert_eq!(n.winner, -1);
        assert_eq!(n.phase, PHASE_PLAYER);
    }

    #[test]
    fn player_annihilation_loses() {
        // 单玩家被双敌贴身围杀到 hp<=0（敌 atk2×2 回合 ×…直接构造低血）。
        let mut w = setup(&[(0, 4, 4), (1, 3, 4), (1, 5, 4)], AI_GREEDY, 1);
        // 玩家 hp 调到 2：一击阵亡。
        let e = unit_at(&mut w, 4, 4).unwrap().entity;
        if let Some(mut u) = w.get_mut::<Unit>(e) {
            u.hp = 2;
        }
        let o = end_turn(&mut w).expect("end_turn");
        assert_eq!(o.winner, 1);
        assert_eq!(unit_at(&mut w, 4, 4), None);
    }
}

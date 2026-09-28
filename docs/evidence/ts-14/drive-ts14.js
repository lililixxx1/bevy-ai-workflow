// TS-14 驱动 + 断言脚本（战斗核心：回合机/移动/攻击/敌方 AI/胜负）。
// 对 assets-methodology/taskset/ts-14-battle-core.md 验收清单逐条判定；
// 全量请求/响应原文落 ts-14-transcript.jsonl，对局状态快照落 ts-14-replay.jsonl。
// 驱动器口径：贪心玩家（趋近/贴身攻击最低血，击杀优先；业务键 (x,y) 决策定序），
// 只经 BRP 读写——不依赖进程内任何旁路。
const fs = require('fs');
const path = require('path');

const BRP = 'http://127.0.0.1:15702';
const DIR = __dirname;
const transcript = fs.createWriteStream(path.join(DIR, 'ts-14-transcript.jsonl'));
const replay = fs.createWriteStream(path.join(DIR, 'ts-14-replay.jsonl'));

let nextId = 1;
async function rpc(method, params) {
  const req = { jsonrpc: '2.0', id: nextId++, method, params };
  const res = await fetch(BRP, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(req),
  });
  const body = await res.json();
  transcript.write(JSON.stringify({ req, res: body }) + '\n');
  return body;
}

// ---- 读态助手（全部经 BRP） ----
async function qUnits() {
  const r = await rpc('world.query', {
    data: {
      components: ['game::level::GridPos', 'game::level::Unit', 'game::battle::ActionFlags'],
    },
  });
  if (r.error) throw new Error('world.query 失败: ' + JSON.stringify(r.error));
  return r.result
    .map((row) => ({
      entity: row.entity,
      x: row.components['game::level::GridPos'].x,
      y: row.components['game::level::GridPos'].y,
      ...row.components['game::level::Unit'],
      ...row.components['game::battle::ActionFlags'],
    }))
    .sort((a, b) => (a.x - b.x) || (a.y - b.y)); // 业务键 (x,y)
}
async function getBattle() {
  const r = await rpc('world.get_resources', { resource: 'game::battle::BattleState' });
  if (r.error) return { err: r.error };
  return { value: r.result.value };
}
async function snapshot(tag) {
  const [units, b] = [await qUnits(), await getBattle()];
  if (b.err) throw new Error('BattleState 读取失败');
  replay.write(JSON.stringify({ tag, battle: b.value, units: units.map(({ x, y, team, hp, moved, attacked }) => ({ x, y, team, hp, moved, attacked })) }) + '\n');
  return { units, battle: b.value };
}

// ---- 断言输出 ----
let fail = 0;
function check(id, name, ok, detail) {
  if (!ok) fail++;
  console.log(`#${id} ${name}: ${ok ? 'PASS' : 'FAIL'}${detail ? ' — ' + detail : ''}`);
}
const e32602 = (r) => r.error && r.error.code === -32602;

// ---- 贪心玩家决策（与规则面同口径：业务键序、击杀优先、最低血、(x,y) 平键） ----
const manh = (a, b) => Math.abs(a.x - b.x) + Math.abs(a.y - b.y);
const lexLess = (a, b) => {
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return a[i] < b[i];
  }
  return false;
};
const pickTarget = (adjacent, atk) =>
  [...adjacent].sort((a, b) => {
    const ka = [(a.hp <= atk ? 0 : 1), a.hp, a.x, a.y];
    const kb = [(b.hp <= atk ? 0 : 1), b.hp, b.x, b.y];
    for (let i = 0; i < 4; i++) if (ka[i] !== kb[i]) return ka[i] - kb[i];
    return 0;
  })[0];

async function oneUnitAct(u, enemies, width, height, mode, goal) {
  // mode 'annihilate'：趋近最近敌；'reach'：奔目标格（能缩短距离就走，走不动才打）。
  const occ = new Set();
  const all = await qUnits();
  for (const v of all) occ.add(`${v.x},${v.y}`);
  const targets = mode === 'reach' ? [goal] : enemies;
  const adjacent = enemies.filter((e) => manh(u, e) === 1);
  const bestDest = () => {
    let best = null, bestScore = null;
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        if (occ.has(`${x},${y}`)) continue;
        const c = { x, y };
        if (manh(u, c) > u.move_range || manh(u, c) === 0) continue;
        const dmin = Math.min(...targets.map((t) => manh(c, t)));
        const adjacentAny = targets.some((t) => manh(c, t) === 1);
        const score = [(mode === 'reach' ? 0 : adjacentAny ? 0 : 1), dmin, c.x, c.y];
        // 字典序取最小（与规则面贪心同口径）。
        if (bestScore == null || lexLess(score, bestScore)) { best = c; bestScore = score; }
      }
    }
    return best; // null = 无可达空闲格
  };
  if (mode === 'reach') {
    // 奔目标：先走（能缩短到目标的距离就走），走不动/走完仍贴身才打。
    const before = Math.min(...targets.map((t) => manh(u, t)));
    const dest = bestDest();
    if (dest) {
      const after = manh(dest, goal);
      if (after < before) {
        const r = await rpc('game.move_unit', { from_x: u.x, from_y: u.y, to_x: dest.x, to_y: dest.y });
        if (r.error) throw new Error('reach 移动失败: ' + JSON.stringify(r.error));
        return r;
      }
    }
    if (adjacent.length) {
      const t = pickTarget(adjacent, u.attack);
      const r = await rpc('game.attack', { from_x: u.x, from_y: u.y, target_x: t.x, target_y: t.y });
      if (r.error) throw new Error('reach 攻击失败: ' + JSON.stringify(r.error));
      return r;
    }
    return null;
  }
  // annihilate：贴身即攻（省移动），否则趋近后视情攻击。
  if (adjacent.length) {
    const t = pickTarget(adjacent, u.attack);
    return attackAt(u, t);
  }
  const dest = bestDest();
  if (dest) {
    const r = await rpc('game.move_unit', { from_x: u.x, from_y: u.y, to_x: dest.x, to_y: dest.y });
    if (r.error) throw new Error('移动失败: ' + JSON.stringify(r.error));
    const after = (await qUnits()).find((v) => v.team === 0 && v.x === dest.x && v.y === dest.y);
    if (after) {
      const adj2 = (await qUnits()).filter((e) => e.team === 1 && manh(after, e) === 1);
      if (adj2.length) return attackAt(after, pickTarget(adj2, after.attack));
    }
    return r;
  }
  return null;
}
async function attackAt(u, t) {
  const r = await rpc('game.attack', { from_x: u.x, from_y: u.y, target_x: t.x, target_y: t.y });
  if (r.error) throw new Error('攻击失败: ' + JSON.stringify(r.error));
  return r;
}

async function playAnnihilate(maxTurns, tag) {
  let st = await snapshot(tag + '-start');
  for (let turn = 1; turn <= maxTurns && st.battle.winner === -1; turn++) {
    const players = (await qUnits()).filter((u) => u.team === 0);
    for (const p of players) {
      st = await snapshot(`${tag}-t${turn}`);
      if (st.battle.winner !== -1) break;
      const fresh = (await qUnits()).find((v) => v.team === 0 && v.x === p.x && v.y === p.y && !v.moved);
      if (!fresh || fresh.moved) continue;
      const enemies = (await qUnits()).filter((v) => v.team === 1);
      if (!enemies.length) break;
      await oneUnitAct(fresh, enemies, 9, 9, 'annihilate');
    }
    st = await snapshot(`${tag}-t${turn}-end`);
    if (st.battle.winner !== -1) break;
    const r = await rpc('game.end_turn', {});
    if (r.error) throw new Error('end_turn 失败: ' + JSON.stringify(r.error));
    st = await snapshot(`${tag}-t${turn}-post`);
  }
  return st.battle;
}

// ---- 主流程（对照清单 #1-#12） ----
(async () => {
  // #1 未加载关卡：BattleState -> -23501
  const b0 = await getBattle();
  check(1, '未加载 BattleState=-23501', b0.err && b0.err.code === -23501, `code=${b0.err && b0.err.code}`);

  // #2 launch(1, seed=7) + BattleState 初值
  const launch = await rpc('game.launch_level', { level: 1, seed: 7 });
  const expKeys = ['level', 'name', 'width', 'height', 'units_spawned'];
  const ok2 = launch.result && expKeys.every((k) => k in launch.result) && Object.keys(launch.result).length === 5 &&
    launch.result.units_spawned === 6;
  const b2 = await getBattle();
  const bs2 = b2.value;
  const ok2b = bs2 && bs2.phase === 0 && bs2.turn === 1 && bs2.winner === -1 && bs2.goal_kind === 0 &&
    bs2.goal_x === -1 && bs2.goal_y === -1 && bs2.seed === 7 && bs2.rng_state === 7 && bs2.ai_kind === 0;
  check(2, 'launch(1,seed=7) 响应形态 + BattleState 初值', ok2 && ok2b, JSON.stringify(launch.result) + ' / ' + JSON.stringify(bs2));

  // #3 移动成功 + moved 标记
  const m3 = await rpc('game.move_unit', { from_x: 1, from_y: 1, to_x: 1, to_y: 3 });
  const units3 = await qUnits();
  const u13 = units3.find((u) => u.x === 1 && u.y === 3);
  check(3, '移动 (1,1)→(1,3) + moved 标记', m3.result && m3.result.winner === -1 && m3.result.phase === 0 &&
    u13 && u13.team === 0 && u13.moved === true && !u13.attacked, JSON.stringify(m3.result));

  const baseline = JSON.stringify(units3);
  const bBefore = JSON.stringify(bs2);

  // #4 移动错误面 ×4 + 无副作用
  const probes4 = [
    { from_x: 1, from_y: 3, to_x: 1, to_y: 4 },   // 已移动（same unit moved=true… 距离1 但已 moved）
    { from_x: 2, from_y: 1, to_x: 2, to_y: 8 },   // 超范围（7 > 3）
    { from_x: 2, from_y: 1, to_x: 1, to_y: 3 },   // 占用
    { from_x: 2, from_y: 1, to_x: 9, to_y: 1 },   // 越界
  ];
  let ok4 = true;
  for (const p of probes4) {
    const r = await rpc('game.move_unit', p);
    if (!e32602(r)) { ok4 = false; console.log('  probe fail:', JSON.stringify(p), '->', JSON.stringify(r)); }
  }
  const units4 = await qUnits();
  const b4 = await getBattle();
  check(4, '移动错误面 ×4 = -32602 + 无副作用', ok4 && JSON.stringify(units4) === baseline && JSON.stringify(b4.value) === bBefore);

  // #5 攻击错误面 ×3 + 无副作用
  const probes5 = [
    { from_x: 1, from_y: 3, target_x: 5, target_y: 7 }, // 距离 >1
    { from_x: 1, from_y: 3, target_x: 4, target_y: 4 }, // 无单位
    { from_x: 1, from_y: 3, target_x: 2, target_y: 1 }, // 友方
  ];
  let ok5 = true;
  for (const p of probes5) {
    const r = await rpc('game.attack', p);
    if (!e32602(r)) { ok5 = false; console.log('  probe fail:', JSON.stringify(p), '->', JSON.stringify(r)); }
  }
  const units5 = await qUnits();
  const b5 = await getBattle();
  check(5, '攻击错误面 ×3 = -32602 + 无副作用', ok5 && JSON.stringify(units5) === baseline && JSON.stringify(b5.value) === bBefore);

  // #6 end_turn：turn=2 / 敌方全员行动记录 / 玩家标记复位 / RNG 推进
  const enemiesBefore = units5.filter((u) => u.team === 1).length;
  const et = await rpc('game.end_turn', {});
  const kindsOk = Array.isArray(et.result.enemy_actions) && et.result.enemy_actions.length === enemiesBefore &&
    et.result.enemy_actions.every((a) => ['attack', 'move', 'move_attack', 'pass'].includes(a.kind));
  const b6 = await getBattle();
  const m6 = await rpc('game.move_unit', { from_x: 1, from_y: 3, to_x: 1, to_y: 4 });
  const ok6 = et.result && et.result.turn === 2 && et.result.phase === 0 && et.result.winner === -1 &&
    kindsOk && b6.value.rng_state !== 7 && !m6.error && m6.result.winner === -1;
  check(6, 'end_turn（turn=2/敌方全员记录/标记复位/RNG 推进）', ok6,
    `turn=${et.result && et.result.turn} actions=${et.result && et.result.enemy_actions && et.result.enemy_actions.length}/${enemiesBefore} rng=${b6.value && b6.value.rng_state}`);

  // #7 完整对局（贪心驱动，上限 30 回合）→ 玩家胜（歼灭）
  const battle7 = await playAnnihilate(30, 'l1');
  const units7 = await qUnits();
  const enemyLeft = units7.filter((u) => u.team === 1).length;
  check(7, '关卡 1 完整对局 winner=0（歼灭）', battle7.winner === 0 && battle7.phase === 2 && enemyLeft === 0,
    `turn=${battle7.turn} winner=${battle7.winner} enemyLeft=${enemyLeft}`);

  // #8 终局封锁 ×3 + 进程存活
  const probes8 = [
    () => rpc('game.move_unit', { from_x: 1, from_y: 4, to_x: 1, to_y: 5 }),
    () => rpc('game.attack', { from_x: 1, from_y: 4, target_x: 5, target_y: 7 }),
    () => rpc('game.end_turn', {}),
  ];
  let ok8 = true;
  for (const p of probes8) {
    const r = await p();
    if (!e32602(r)) { ok8 = false; console.log('  probe fail:', JSON.stringify(r)); }
  }
  const alive8 = await rpc('rpc.discover', {});
  check(8, '终局封锁 ×3 = -32602 + 进程存活', ok8 && alive8.result && Array.isArray(alive8.result.methods));

  // #9 relaunch 复位
  const rl = await rpc('game.launch_level', { level: 1, seed: 7 });
  const b9 = await getBattle();
  const units9 = await qUnits();
  check(9, 'relaunch(1,seed=7) BattleState 复位 + 6 行布阵', rl.result.units_spawned === 6 &&
    b9.value.phase === 0 && b9.value.turn === 1 && b9.value.winner === -1 && b9.value.rng_state === 7 &&
    units9.length === 6, JSON.stringify(b9.value));

  // #10 占点型关卡 3：直奔 (8,4) 即胜
  const l3 = await rpc('game.launch_level', { level: 3, seed: 7 });
  const b10 = await getBattle();
  const goalOk = b10.value.goal_kind === 1 && b10.value.goal_x === 8 && b10.value.goal_y === 4 && b10.value.ai_kind === 1;
  let midWinner = null, finalWinner = null, moves10 = 0;
  for (let turn = 0; turn < 20 && (await getBattle()).value.winner === -1; turn++) {
    const runner0 = (await qUnits()).find((u) => u.team === 0 && u.x === 0 && u.y === 4) ||
      (await qUnits()).filter((u) => u.team === 0).sort((a, b) => manh(a, { x: 8, y: 4 }) - manh(b, { x: 8, y: 4 }))[0];
    if (!runner0) break;
    const r = await oneUnitAct(runner0, (await qUnits()).filter((v) => v.team === 1), 9, 9, 'reach', { x: 8, y: 4 });
    moves10++;
    if (r && r.result) { if (r.result.winner === -1) midWinner = -1; if (r.result.winner === 0) finalWinner = 0; }
    if ((await getBattle()).value.winner !== -1) break;
    const et10 = await rpc('game.end_turn', {});
    if (et10.error) throw new Error('L3 end_turn 失败: ' + JSON.stringify(et10.error));
  }
  const b10b = await getBattle();
  check(10, '关卡 3 占点：中途 winner=-1 → 踏 (8,4) winner=0', goalOk && l3.result.units_spawned === 9 &&
    midWinner === -1 && finalWinner === 0 && b10b.value.winner === 0 && b10b.value.phase === 2,
    `goal=${JSON.stringify([b10.value.goal_kind, b10.value.goal_x, b10.value.goal_y])} ai=${b10.value.ai_kind} moves=${moves10} winner=${b10b.value.winner}`);

  // #11 rpc.discover 30 方法（27+3）
  const disc = await rpc('rpc.discover', {});
  const names = disc.result.methods.map((m) => m.name);
  check(11, 'rpc.discover 30 方法含三新方法', names.length === 30 &&
    ['game.move_unit', 'game.attack', 'game.end_turn'].every((n) => names.includes(n)), `count=${names.length}`);

  // #12 由外层脚本核游戏日志（此处只收尾）
  transcript.end(); replay.end();
  console.log(fail === 0 ? 'ALL PASS (12 items pending #12 log check)' : `FAILED (${fail})`);
  process.exit(fail === 0 ? 0 : 1);
})().catch((e) => { console.error('DRIVER ERROR', e); transcript.end(); replay.end(); process.exit(2); });

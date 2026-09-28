// TS-16 战役驱动器：3 关连打（核心循环闭环）+ 逐步状态快照。
// 单次进程调用跑完整战役（launch→胜→launch→胜→launch→胜）；
// 快照（业务键排序布阵 + BattleState，实体号排除——PIT-B-010 口径）逐操作落
// ts-16-replay-<tag>.jsonl；跨进程双跑后逐位 diff = 备忘录「同种子 + 同操作
// 序列 → 状态序列逐位一致」验收面。
// 用法：node campaign.js <tag> [seed]
const fs = require('fs');
const path = require('path');
const BRP = 'http://127.0.0.1:15702';
const TAG = process.argv[2] || 'a';
const SEED = Number(process.argv[3] || 20260928);
const replay = fs.createWriteStream(path.join(__dirname, `ts-16-replay-${TAG}.jsonl`));
const summary = { seed: SEED, levels: [], ops: 0, errors: 0 };

let nextId = 1;
async function rpc(method, params) {
  const req = { jsonrpc: '2.0', id: nextId++, method, params };
  const res = await fetch(BRP, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(req) });
  const body = await res.json();
  if (body.error) summary.errors++;
  return body;
}
async function qUnits() {
  const r = await rpc('world.query', { data: { components: ['game::level::GridPos', 'game::level::Unit', 'game::battle::ActionFlags'] } });
  if (r.error) throw new Error('query: ' + JSON.stringify(r.error));
  return r.result.map((row) => ({
    x: row.components['game::level::GridPos'].x,
    y: row.components['game::level::GridPos'].y,
    ...row.components['game::level::Unit'],
    ...row.components['game::battle::ActionFlags'],
  })).sort((a, b) => (a.x - b.x) || (a.y - b.y));
}
async function getBattle() {
  const r = await rpc('world.get_resources', { resource: 'game::battle::BattleState' });
  if (r.error) throw new Error('battle: ' + JSON.stringify(r.error));
  return r.result.value;
}
async function snap(level, op) {
  const [units, battle] = [await qUnits(), await getBattle()];
  replay.write(JSON.stringify({
    level, op, turn: battle.turn, winner: battle.winner, phase: battle.phase, rng: battle.rng_state,
    units: units.map(({ x, y, team, hp, moved, attacked }) => ({ x, y, team, hp, moved, attacked })),
  }) + '\n');
  return battle;
}

const manh = (a, b) => Math.abs(a.x - b.x) + Math.abs(a.y - b.y);
const lexLess = (a, b) => { for (let i = 0; i < a.length; i++) { if (a[i] !== b[i]) return a[i] < b[i]; } return false; };
const pickTarget = (adj, atk) => [...adj].sort((a, b) => {
  const ka = [(a.hp <= atk ? 0 : 1), a.hp, a.x, a.y], kb = [(b.hp <= atk ? 0 : 1), b.hp, b.x, b.y];
  for (let i = 0; i < 4; i++) if (ka[i] !== kb[i]) return ka[i] - kb[i];
  return 0;
})[0];
async function bestMove(u, targets, mode, goal) {
  const all = await qUnits();
  const occ = new Set(all.map((v) => `${v.x},${v.y}`));
  let best = null, bestScore = null;
  for (let y = 0; y < 9; y++) for (let x = 0; x < 9; x++) {
    if (occ.has(`${x},${y}`)) continue;
    const c = { x, y };
    if (manh(u, c) > u.move_range || manh(u, c) === 0) continue;
    const dmin = Math.min(...targets.map((t) => manh(c, t)));
    const adjacentAny = targets.some((t) => manh(c, t) === 1);
    const score = [(mode === 'reach' ? 0 : adjacentAny ? 0 : 1), dmin, c.x, c.y];
    if (bestScore == null || lexLess(score, bestScore)) { best = c; bestScore = score; }
  }
  return best;
}
async function playerUnitAct(u, enemies, mode, goal) {
  const adjacent = enemies.filter((e) => manh(u, e) === 1);
  if (mode === 'reach') {
    const before = manh(u, goal);
    const dest = await bestMove(u, [goal], 'reach', goal);
    if (dest && manh(dest, goal) < before) {
      summary.ops++;
      return rpc('game.move_unit', { from_x: u.x, from_y: u.y, to_x: dest.x, to_y: dest.y });
    }
    if (adjacent.length) {
      const t = pickTarget(adjacent, u.attack);
      summary.ops++;
      return rpc('game.attack', { from_x: u.x, from_y: u.y, target_x: t.x, target_y: t.y });
    }
    return null;
  }
  if (adjacent.length) {
    const t = pickTarget(adjacent, u.attack);
    summary.ops++;
    return rpc('game.attack', { from_x: u.x, from_y: u.y, target_x: t.x, target_y: t.y });
  }
  const dest = await bestMove(u, enemies, 'annihilate', null);
  if (dest) {
    summary.ops++;
    const r = await rpc('game.move_unit', { from_x: u.x, from_y: u.y, to_x: dest.x, to_y: dest.y });
    if (r.error) return r;
    const after = (await qUnits()).find((v) => v.team === 0 && v.x === dest.x && v.y === dest.y);
    if (after && !after.attacked) {
      const adj2 = (await qUnits()).filter((e) => e.team === 1 && manh(after, e) === 1);
      if (adj2.length) {
        const t = pickTarget(adj2, after.attack);
        summary.ops++;
        return rpc('game.attack', { from_x: after.x, from_y: after.y, target_x: t.x, target_y: t.y });
      }
    }
  }
  return null;
}
async function playLevel(level, maxTurns) {
  const launch = await rpc('game.launch_level', { level, seed: SEED });
  if (launch.error) throw new Error(`launch(${level}) 失败: ` + JSON.stringify(launch.error));
  const b0 = await getBattle();
  const mode = b0.goal_kind === 1 ? 'reach' : 'annihilate';
  await snap(level, 'launch');
  let b = b0, opsAtStart = summary.ops;
  for (let turn = 1; turn <= maxTurns && b.winner === -1; turn++) {
    if (mode === 'reach') {
      const units = await qUnits();
      const runner = units.filter((u) => u.team === 0).sort((a, c) => manh(a, { x: b.goal_x, y: b.goal_y }) - manh(c, { x: b.goal_x, y: b.goal_y }))[0];
      if (runner) await playerUnitAct(runner, units.filter((v) => v.team === 1), 'reach', { x: b.goal_x, y: b.goal_y });
    } else {
      const players = (await qUnits()).filter((u) => u.team === 0).map((u) => ({ ...u }));
      for (const p of players) {
        b = await snap(level, `t${turn}`);
        if (b.winner !== -1) break;
        const fresh = (await qUnits()).find((v) => v.team === 0 && v.x === p.x && v.y === p.y && !v.moved && !v.attacked);
        if (!fresh) continue;
        const enemies = (await qUnits()).filter((v) => v.team === 1);
        if (!enemies.length) break;
        await playerUnitAct(fresh, enemies, 'annihilate', null);
      }
    }
    b = await snap(level, `t${turn}-end`);
    if (b.winner !== -1) break;
    summary.ops++;
    const et = await rpc('game.end_turn', {});
    if (et.error) throw new Error(`L${level} end_turn: ` + JSON.stringify(et.error));
    b = await snap(level, `t${turn}-post`);
  }
  const units = await qUnits();
  summary.levels.push({
    level, mode, winner: b.winner, turns: b.turn,
    playersLeft: units.filter((u) => u.team === 0).length,
    enemiesLeft: units.filter((u) => u.team === 1).length,
    ops: summary.ops - opsAtStart,
  });
  return b;
}
(async () => {
  for (const lv of [1, 2, 3]) {
    const b = await playLevel(lv, 40);
    if (b.winner !== 0) {
      console.error(`LEVEL ${lv} NOT WON (winner=${b.winner}, turn=${b.turn})`);
      process.exit(3);
    }
  }
  replay.end();
  console.log(JSON.stringify(summary));
  process.exit(summary.errors === 0 ? 0 : 4);
})().catch((e) => { console.error('DRIVER ERROR', e); replay.end(); process.exit(2); });

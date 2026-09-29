// TS-17 稳定 id 快照驱动器（窗口前置增强 T040 / A4，taskset TS-17）。
// 单进程断言面 + 快照原始体采集；跨进程 A/B 一致性由外层对两份原始体 diff 判定。
// 用法：node snapshot-probe.js <tag> [--suites]   （游戏进程已由外层启动）
// 断言（exit 0 = 全过）：
//  1) rpc.discover 含 game.snapshot；总数 31 = 23 内置 + 8 game.*；
//  2) 未 launch 前（params 缺省）snapshot：battle === null 且 sim.paused 为布尔；
//     负控：params 非空对象 → -32602；
//  3) --suites：13 套件 game.run_tests 全 PASS（failed=0）——**先于本驱动的
//     launch 执行**（PIT-M-010：ts-08 相位 A 的默认 90 断言隐含「进程未加载过
//     关卡」前提，T036 表现层在关卡跨帧后把 CameraRig.radius 收拢到 14；套件
//     本身在 handler 独占执行内原子完成，ts-17 的 launch+还原不跨帧泄露）；
//  4) launch(1, seed=20260928) 后 snapshot 与 world.query / world.get_resources
//     逐字段等价（等价面在 < 2^53 值域内严格相等，PIT-M-009 口径注记）；
//  5) 快照【原始响应体】落盘 ts-17-snapshot-<tag>.raw（JSON-RPC id 固定 9001——
//     双侧同 id 同序列化，diff 即逐字节；不经 JS 数值面，TS-16 限定先例的强化）。
const fs = require('fs');
const path = require('path');

const BRP = 'http://127.0.0.1:15702';
const TAG = process.argv[2] || 'a';
const RUN_SUITES = process.argv.includes('--suites');
const SUITES = ['ts-01', 'ts-02', 'ts-03', 'ts-05', 'ts-06', 'ts-07', 'ts-08', 'ts-09', 'ts-10', 'ts-11', 'ts-12', 'ts-14', 'ts-17'];
let nextId = 1;
let failures = 0;

function fail(msg) { failures++; console.error('FAIL: ' + msg); }
function ok(msg) { console.log('PASS: ' + msg); }

async function rpc(method, params, id) {
  const res = await fetch(BRP, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id: id ?? nextId++, method, params }),
  });
  return res.json();
}
async function rpcRaw(method, params, id) {
  const res = await fetch(BRP, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ jsonrpc: '2.0', id, method, params }),
  });
  return res.text(); // 原始体：跨进程逐字节比较的载体（不经 JS 解析）
}
async function waitReady() {
  for (let i = 0; i < 120; i++) {
    try { await rpc('rpc.discover', {}); return; } catch { }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error('BRP 60s 未就绪');
}

async function qUnits() {
  const r = await rpc('world.query', { data: { components: ['game::level::GridPos', 'game::level::Unit', 'game::battle::ActionFlags'] } });
  if (r.error) throw new Error('query units: ' + JSON.stringify(r.error));
  return r.result.map((row) => ({
    x: row.components['game::level::GridPos'].x,
    y: row.components['game::level::GridPos'].y,
    ...row.components['game::level::Unit'],
    ...row.components['game::battle::ActionFlags'],
  })).sort((a, b) => (a.x - b.x) || (a.y - b.y));
}
async function qWanderers() {
  const r = await rpc('world.query', { data: { components: ['game::sim::Wanderer', 'game::sim::Velocity'] } });
  if (r.error) throw new Error('query wanderers: ' + JSON.stringify(r.error));
  return r.result.map((row) => ({
    index: row.components['game::sim::Wanderer'].index,
    origin: row.components['game::sim::Wanderer'].origin,
    phase: row.components['game::sim::Wanderer'].phase,
    velocity: row.components['game::sim::Velocity'].linear,
  })).sort((a, b) => a.index - b.index);
}
async function getResource(name) {
  const r = await rpc('world.get_resources', { resource: name });
  if (r.error) throw new Error(`get ${name}: ` + JSON.stringify(r.error));
  return r.result.value;
}

(async () => {
  await waitReady();
  console.log(`[probe:${TAG}] BRP ready`);

  // ① rpc.discover
  const disc = await rpc('rpc.discover', {});
  if (disc.error) throw new Error('discover: ' + JSON.stringify(disc.error));
  const names = disc.result.methods.map((m) => m.name);
  const gameMethods = names.filter((n) => n.startsWith('game.')).sort();
  if (!names.includes('game.snapshot')) fail('discover 不含 game.snapshot');
  else ok(`discover 含 game.snapshot（总 ${names.length} = 23 内置 + ${gameMethods.length} game.*：${gameMethods.join(',')}）`);
  if (names.length !== 31) fail(`方法总数 ${names.length} ≠ 31`);
  if (gameMethods.length !== 8) fail(`game.* 数 ${gameMethods.length} ≠ 8`);

  // ② 未加载关卡面 + params 负控
  const cold = await rpc('game.snapshot');
  if (cold.error) fail('未加载关卡 snapshot 报错: ' + JSON.stringify(cold.error));
  else if (cold.result.battle !== null) fail(`未加载关卡 battle=${JSON.stringify(cold.result.battle)} ≠ null`);
  else if (typeof cold.result.sim?.paused !== 'boolean') fail('sim.paused 非布尔');
  else ok('未加载关卡 battle=null、sim.paused=' + cold.result.sim.paused + '（wanderers=' + cold.result.sim.wanderers.length + '）');
  const badParams = await rpc('game.snapshot', { level: 1 });
  if (!(badParams.error && badParams.error.code === -32602)) fail('params 负控未报 -32602: ' + JSON.stringify(badParams));
  else ok('params 负控 -32602（' + badParams.error.message + '）');

  // ③ 套件复跑（13 套件；先于 launch——PIT-M-010 顺序前提，见文件头注 3）
  if (RUN_SUITES) {
    for (const s of SUITES) {
      const r = await rpc('game.run_tests', { suite: s });
      if (r.error) { fail(`套件 ${s} 协议错误: ` + JSON.stringify(r.error)); continue; }
      if (r.result.failed !== 0) fail(`套件 ${s} failed=${r.result.failed}`);
      else ok(`套件 ${s}：${r.result.passed}/${r.result.total} PASS`);
    }
  }

  // ④ launch + 逐字段等价（BRP 反射通路独立于 snapshot handler）
  const launch = await rpc('game.launch_level', { level: 1, seed: 20260928 });
  if (launch.error) throw new Error('launch: ' + JSON.stringify(launch.error));
  const snap = await rpc('game.snapshot');
  if (snap.error) throw new Error('snapshot: ' + JSON.stringify(snap.error));
  const [units, wanderers, battle, simcfg] = [await qUnits(), await qWanderers(), await getResource('game::battle::BattleState'), await getResource('game::sim::SimConfig')];

  const b = snap.result.battle;
  const eqUnits = Array.isArray(b.units) && b.units.length === units.length && units.every((u, i) =>
    b.units[i].x === u.x && b.units[i].y === u.y && b.units[i].team === u.team &&
    b.units[i].hp === u.hp && b.units[i].moved === u.moved && b.units[i].attacked === u.attacked);
  const eqBattle = b.turn === battle.turn && b.winner === battle.winner && b.phase === battle.phase && b.rng_state === battle.rng_state;
  const eqSim = snap.result.sim.paused === simcfg.paused;
  const w = snap.result.sim.wanderers;
  const eqWanderers = Array.isArray(w) && w.length === wanderers.length && wanderers.every((q, i) =>
    w[i].index === q.index && w[i].phase === q.phase &&
    w[i].origin[0] === q.origin[0] && w[i].origin[1] === q.origin[1] && w[i].origin[2] === q.origin[2] &&
    w[i].velocity[0] === q.velocity[0] && w[i].velocity[1] === q.velocity[1] && w[i].velocity[2] === q.velocity[2]);
  if (!(eqUnits && eqBattle && eqSim && eqWanderers))
    fail(`等价断裂 units=${eqUnits} battle=${eqBattle} sim=${eqSim} wanderers=${eqWanderers}`);
  else
    ok(`launch(1,20260928) 后逐字段等价：units ${units.length} 行、battle 四字段（turn=${b.turn}/winner=${b.winner}/phase=${b.phase}/rng_state=${b.rng_state}）、paused=${snap.result.sim.paused}、wanderers ${wanderers.length} 行四元组（< 2^53 值域内严格相等，PIT-M-009）`);

  // ⑤ 快照原始体落盘（id 固定 9001——双侧同 id，diff 即逐字节）
  const raw = await rpcRaw('game.snapshot', undefined, 9001);
  const out = path.join(__dirname, `ts-17-snapshot-${TAG}.raw`);
  fs.writeFileSync(out, raw);
  console.log(`[probe:${TAG}] raw snapshot -> ${path.basename(out)}（${Buffer.byteLength(raw)} bytes）`);

  console.log(`[probe:${TAG}] failures=${failures}`);
  process.exit(failures === 0 ? 0 : 1);
})().catch((e) => { console.error('DRIVER ERROR', e); process.exit(2); });

// TS-15 驱动 + 断言：战斗规则进程内回归套件（game.run_tests "ts-14"）。
// 对 assets-methodology/taskset/ts-15-battle-suite.md 验收清单逐条判定；
// 全量请求/响应原文落 ts-15-transcript.jsonl。
const fs = require('fs');
const path = require('path');
const BRP = 'http://127.0.0.1:15702';
const DIR = __dirname;
const transcript = fs.createWriteStream(path.join(DIR, 'ts-15-transcript.jsonl'));
let nextId = 1;
async function rpc(method, params) {
  const req = { jsonrpc: '2.0', id: nextId++, method, params };
  const res = await fetch(BRP, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(req) });
  const body = await res.json();
  transcript.write(JSON.stringify({ req, res: body }) + '\n');
  return body;
}
let fail = 0;
const check = (id, name, ok, detail) => { if (!ok) fail++; console.log(`#${id} ${name}: ${ok ? 'PASS' : 'FAIL'}${detail ? ' — ' + detail : ''}`); };

const L1_KEYS = ['0,1,1', '0,2,1', '0,3,1', '1,5,7', '1,6,7', '1,7,7'];

(async () => {
  // #1 首跑：6/6 全过
  const r1 = await rpc('game.run_tests', { suite: 'ts-14' });
  const names1 = (r1.result?.results || []).map((r) => `${r.name}:${r.pass ? 'P' : 'F'}`).join(' ');
  check(1, '套件 ts-14 首跑全过', r1.result && r1.result.total === 6 && r1.result.failed === 0,
    `total=${r1.result?.total} passed=${r1.result?.passed} failed=${r1.result?.failed} [${names1}]`);

  // #2 同进程二跑：仍全过（无净副作用自洽）
  const r2 = await rpc('game.run_tests', { suite: 'ts-14' });
  check(2, '同进程二跑仍全过', r2.result && r2.result.failed === 0 && r2.result.total === 6,
    `failed=${r2.result?.failed}`);

  // #3 已加载关卡状态下跑：全过 + 世界恢复（布阵业务键 + BattleState 初值含 rng_state 未被推进）
  const launch = await rpc('game.launch_level', { level: 1, seed: 7 });
  const q0 = await rpc('world.query', { data: { components: ['game::level::GridPos', 'game::level::Unit'] } });
  const keys0 = q0.result.map((row) => `${row.components['game::level::Unit'].team},${row.components['game::level::GridPos'].x},${row.components['game::level::GridPos'].y}`).sort().join(';');
  const r3 = await rpc('game.run_tests', { suite: 'ts-14' });
  const q1 = await rpc('world.query', { data: { components: ['game::level::GridPos', 'game::level::Unit'] } });
  const keys1 = q1.result.map((row) => `${row.components['game::level::Unit'].team},${row.components['game::level::GridPos'].x},${row.components['game::level::GridPos'].y}`).sort().join(';');
  const b1 = await rpc('world.get_resources', { resource: 'game::battle::BattleState' });
  const bs = b1.result?.value;
  check(3, '加载态跑套件全过 + 快照恢复', launch.result?.units_spawned === 6 && keys0 === L1_KEYS.join(';') &&
    r3.result?.failed === 0 && keys1 === keys0 && q1.result.length === 6 &&
    bs && bs.phase === 0 && bs.turn === 1 && bs.winner === -1 && bs.rng_state === 7,
    `suite failed=${r3.result?.failed}；布阵 ${keys1 === keys0 ? '恢复一致' : '漂移!'}；BattleState=${bs ? `turn=${bs.turn}/rng=${bs.rng_state}` : '缺失'}`);

  // #4 未知套件：-32602 且可用清单含 ts-14
  const r4 = await rpc('game.run_tests', { suite: 'ts-99' });
  check(4, '未知套件 -32602 + 清单含 ts-14', r4.error?.code === -32602 && (r4.error.message + '').includes('ts-14'),
    `code=${r4.error?.code} msg=${(r4.error?.message || '').slice(0, 80)}…`);

  // #5 游戏日志面由外层脚本核验
  transcript.end();
  console.log(fail === 0 ? 'ALL PASS (4 runtime items pending #5 log check)' : `FAILED (${fail})`);
  process.exit(fail === 0 ? 0 : 1);
})().catch((e) => { console.error('DRIVER ERROR', e); transcript.end(); process.exit(2); });

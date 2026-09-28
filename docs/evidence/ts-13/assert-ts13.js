// TS-13 断言脚本（第一阶段脚本判定）：逐条对照 taskset/ts-13-launch-level.md 验收清单。
// 输入：同目录 a*.raw.json（curl 原始响应）；输出：逐条判定到 stdout（重定向至 assert-ts13-out.txt）。
const fs = require('fs');
const dir = __dirname;
const read = (f) => JSON.parse(fs.readFileSync(`${dir}/${f}`, 'utf8'));

// 关卡定义（与 game/src/level.rs 常量表同源；断言按业务键 (team,x,y) 排序后逐位比对——PIT-B-010）
const L1 = [
  { team: 0, x: 1, y: 1, hp: 10, move_range: 3, attack: 2 },
  { team: 0, x: 2, y: 1, hp: 10, move_range: 3, attack: 2 },
  { team: 0, x: 3, y: 1, hp: 10, move_range: 3, attack: 2 },
  { team: 1, x: 5, y: 7, hp: 8, move_range: 2, attack: 2 },
  { team: 1, x: 6, y: 7, hp: 8, move_range: 2, attack: 2 },
  { team: 1, x: 7, y: 7, hp: 8, move_range: 2, attack: 2 },
];
const L2 = [
  { team: 0, x: 0, y: 0, hp: 10, move_range: 3, attack: 3 },
  { team: 0, x: 1, y: 0, hp: 10, move_range: 3, attack: 3 },
  { team: 0, x: 2, y: 0, hp: 10, move_range: 3, attack: 3 },
  { team: 0, x: 3, y: 0, hp: 10, move_range: 3, attack: 3 },
  { team: 1, x: 4, y: 8, hp: 8, move_range: 2, attack: 2 },
  { team: 1, x: 5, y: 8, hp: 8, move_range: 2, attack: 2 },
  { team: 1, x: 6, y: 8, hp: 8, move_range: 2, attack: 2 },
  { team: 1, x: 7, y: 8, hp: 8, move_range: 2, attack: 2 },
  { team: 1, x: 8, y: 8, hp: 8, move_range: 2, attack: 2 },
];
const key = (r) => `${r.team},${r.x},${r.y}`;
const byKey = (r) => [r.team, r.x, r.y].join('|');
const rowsToUnits = (rows) =>
  rows
    .map((row) => {
      const u = row.components['game::level::Unit'];
      const g = row.components['game::level::GridPos'];
      return { team: u.team, x: g.x, y: g.y, hp: u.hp, move_range: u.move_range, attack: u.attack, entity: row.entity };
    })
    .sort((a, b) => byKey(a).localeCompare(byKey(b)));
const defSorted = (def) => [...def].sort((a, b) => byKey(a).localeCompare(byKey(b)));

let fail = 0;
const check = (id, name, ok, detail) => {
  if (!ok) fail++;
  console.log(`#${id} ${name}: ${ok ? 'PASS' : 'FAIL'}${detail ? ' — ' + detail : ''}`);
};

// #1 启动无默认关卡：query Unit 0 行
const a1 = read('a1-query-baseline.raw.json');
check(1, '启动基线 Unit==0', Array.isArray(a1.result) && a1.result.length === 0, `rows=${a1.result.length}`);

// #2 launch 1 响应字段（键序无关比较：serde_json 默认按字母序输出键，键集合 + 逐字段判等）
const a2 = read('a2-launch-1.raw.json');
const exp2 = { level: 1, name: 'first-contact', width: 9, height: 9, units_spawned: 6 };
const objEqUnordered = (a, b) => {
  const ka = Object.keys(a).sort(), kb = Object.keys(b).sort();
  return JSON.stringify(ka) === JSON.stringify(kb) && ka.every((k) => a[k] === b[k]);
};
check(2, 'launch(1) 响应', objEqUnordered(a2.result, exp2), JSON.stringify(a2.result));

// #3 布阵逐位（业务键排序）
const a3 = read('a3-query-l1.raw.json');
const got3 = rowsToUnits(a3.result);
const strip = (rs) => rs.map(({ entity, ...r }) => r);
check(3, '关卡1 布阵 6 行逐位', a3.result.length === 6 && JSON.stringify(strip(got3)) === JSON.stringify(defSorted(L1)),
  `rows=${a3.result.length}`);

// #4 幂等：连续两次 relaunch(1) 后布阵不变（实体号允许变化）
const a4a = read('a4a-relaunch-1.raw.json');
const a4b = read('a4b-relaunch-1.raw.json');
const a4c = read('a4c-query-idem.raw.json');
const got4 = rowsToUnits(a4c.result);
const ent4 = got4.map((r) => r.entity).join(',');
const ent3 = got3.map((r) => r.entity).join(',');
check(4, '重复 launch(1) 幂等', a4a.result.units_spawned === 6 && a4b.result.units_spawned === 6 &&
  a4c.result.length === 6 && JSON.stringify(strip(got4)) === JSON.stringify(defSorted(L1)),
  `两次 units_spawned=${a4a.result.units_spawned}/${a4b.result.units_spawned}；实体号 ${ent3} → ${ent4}`);

// #5 重载清场 6→9 且为关卡 2 布阵
const a5a = read('a5a-launch-2.raw.json');
const a5b = read('a5b-query-l2.raw.json');
const got5 = rowsToUnits(a5b.result);
check(5, 'launch(2) 清场重载 6→9', a5a.result.units_spawned === 9 && a5b.result.length === 9 &&
  JSON.stringify(strip(got5)) === JSON.stringify(defSorted(L2)),
  `units_spawned=${a5a.result.units_spawned}；rows=${a5b.result.length}`);

// #6 错误路径：-32602；LevelState 仍关卡 2（get_resources 单资源形态为 result.value）；query 仍 9 行
const a6a = read('a6a-launch-99.raw.json');
const a6b = read('a6b-level-state.raw.json');
const a6c = read('a6c-query-after-error.raw.json');
const st = a6b.result && a6b.result.value;
check(6, '错误路径 -32602 + 无副作用 + 进程存活', a6a.error && a6a.error.code === -32602 &&
  st && st.level === 2 && st.units_spawned === 9 && a6c.result.length === 9,
  `error.code=${a6a.error && a6a.error.code}；LevelState=${JSON.stringify(st)}；after-error rows=${a6c.result.length}`);

console.log(fail === 0 ? 'ALL PASS (6/6)' : `FAILED (${fail})`);
process.exit(fail === 0 ? 0 : 1);

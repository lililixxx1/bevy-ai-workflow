// TS-02 断言数据处理：从 BRP 原始响应提取与判定（node，无依赖）。
// 用法:
//   node compare-origins.mjs origins-run1.json origins-run2.json  # 断言3：origin 逐位一致
//   node compare-origins.mjs --speeds velocities-run1.json 9.0     # 断言2：模长 ∈ (0, cap]，且存在 >3.0
import { readFileSync } from "node:fs";

const readRows = (p) => {
  const v = JSON.parse(readFileSync(p, "utf8"));
  return Array.isArray(v) ? v : v.result; // 接受裸数组或完整 JSON-RPC 响应
};

if (process.argv[2] === "--speeds") {
  const rows = readRows(process.argv[3]);
  const cap = Number(process.argv[4]);
  const lens = rows.map((r) => {
    const v = r.components["game::sim::Velocity"].linear;
    return Math.hypot(v[0], v[1], v[2]);
  });
  const sampled = lens.length;
  const bad = lens.filter((l) => !(l > 0.0 && l <= cap)).length;
  const over3 = lens.filter((l) => l > 3.0).length;
  console.log(JSON.stringify({
    sampled, cap,
    all_in_open_interval: bad === 0,
    violations: bad,
    count_over_default_cap: over3,
    min: Math.min(...lens), max: Math.max(...lens),
  }));
  process.exit(0);
}

const [a, b] = [readRows(process.argv[2]), readRows(process.argv[3])];
const mapByIndex = (rows) => {
  const m = new Map();
  for (const r of rows) {
    const w = r.components["game::sim::Wanderer"];
    m.set(w.index, w.origin); // JSON 数字数组；serde_json 为 f32 最短往返表示，文本相等 ⇔ 位模式相等
  }
  return m;
};
const [ma, mb] = [mapByIndex(a), mapByIndex(b)];
let compared = 0, mismatches = 0, firstMismatch = null;
for (const [index, origin] of ma) {
  const other = mb.get(index);
  compared += 1;
  if (!other || JSON.stringify(origin) !== JSON.stringify(other)) {
    mismatches += 1;
    if (!firstMismatch) firstMismatch = { index, run1: origin, run_default: other ?? null };
  }
}
console.log(JSON.stringify({ compared, mismatches, firstMismatch }));

// T047-W1 P6b 独立计数脚本：解析外部关卡 TOML，按教程 06 行为规格展开逻辑网格，
// 输出 PathTile 期望计数（供 assert-td.mjs --phase p6b --expect-path 使用）。
//
// 判据纪律（派工单 + 附录 B.2）：期望值**实跑推导**——从 TOML 文件真解析、真展开、
// 真计数，禁手数、禁硬编码。展开规则独立按教程 06 Step 2/4 的行为描述重写
// （与 Rust 侧实现互不引用 = 双盲对拍）：
//   - `[paths.<id>].waypoints` 为宽 1 中心线（[x, y] 网格坐标）；
//   - 轴对齐线段展开为宽 3：水平段 = 中心行 ±1 行，垂直段 = 中心列 ±1 列；
//   - 越界格静默跳过；
//   - 每个内点（拐点）补外侧对角格：mx / my 取「水平段方向」与「垂直段方向反号」。
//
// 用法（树根执行）：
//   node docs/evidence/t047-m6-wave1/toml-path-count.mjs game-td/assets/levels/level_01.toml
// 输出末行 `EXPECTED_PATH=<n> EXPECTED_GRASS=<m> EXPECTED_TOTAL=<t>`，退出码 0 = 解析成功。
import { readFileSync } from "node:fs";

const file = process.argv[2];
if (!file) {
  console.error("usage: node toml-path-count.mjs <level.toml>");
  process.exit(2);
}

const text = readFileSync(file, "utf8");

// ---- 1) 极简 TOML 子集解析（本工程关卡格式只用到表头 + 整数 + 二维整数数组）----
// 去注释（# 到行尾；关卡文件无字符串值，安全）。保留行结构便于逐行解析。
const lines = text
  .split(/\r?\n/)
  .map((l) => l.replace(/#.*$/, ""))
  .filter((l) => l.trim().length > 0);

let section = null;
const map = { width: null, height: null };
const paths = new Map(); // id -> number[][]
let pendingArray = null; // 跨行数组累积
let pendingKey = null;

function flushPending() {
  if (!pendingArray) return;
  const body = pendingArray.join(" ");
  if (pendingKey.kind !== "path") throw new Error(`unexpected pending key ${pendingKey}`);
  const pairs = [];
  const re = /\[\s*(\d+)\s*,\s*(\d+)\s*\]/g;
  let m;
  while ((m = re.exec(body))) pairs.push([Number(m[1]), Number(m[2])]);
  paths.set(pendingKey.id, pairs);
  pendingArray = null;
  pendingKey = null;
}

for (const raw of lines) {
  const line = raw.trim();
  const header = line.match(/^\[([^\]]+)\]$/);
  if (header) {
    flushPending();
    section = header[1].trim();
    if (section.startsWith("paths.")) {
      const id = section.slice("paths.".length);
      if (paths.has(id)) throw new Error(`duplicate path id: ${id}`);
      paths.set(id, []); // 占位，waypoints 键填充
    }
    continue;
  }
  // 跨行数组续行：优先于 key = value 解析（续行形如 `[0, 5],` 不是 kv 行）。
  if (pendingArray) {
    pendingArray.push(line);
    if (line.includes("]") && countBrackets(pendingArray.join(" ")) === 0) flushPending();
    continue;
  }
  const kv = line.match(/^([A-Za-z0-9_]+)\s*=\s*(.*)$/);
  if (!kv) throw new Error(`unparseable line: ${line}`);
  const key = kv[1];
  let value = kv[2].trim();

  if (value.startsWith("[") && countBrackets(value) > 0) {
    if (!section) throw new Error(`array outside section: ${line}`);
    if (section === "map") throw new Error("unexpected array in [map]");
    pendingKey = { kind: "path", id: section.slice("paths.".length) };
    pendingArray = [value];
    continue;
  }
  if (section === "map") {
    if (key !== "width" && key !== "height") throw new Error(`unknown [map] key: ${key}`);
    if (!/^\d+$/.test(value)) throw new Error(`non-integer ${key}: ${value}`);
    map[key] = Number(value);
    continue;
  }
  if (section && section.startsWith("paths.")) {
    if (key !== "waypoints") throw new Error(`unknown path key: ${key}`);
    // 单行数组形态（本侧关卡用多行，但保持解析器完备）
    const pairs = [];
    const re = /\[\s*(\d+)\s*,\s*(\d+)\s*\]/g;
    let m;
    while ((m = re.exec(value))) pairs.push([Number(m[1]), Number(m[2])]);
    paths.set(section.slice("paths.".length), pairs);
    continue;
  }
  throw new Error(`key outside known section: ${line}`);
}
flushPending();

if (!Number.isInteger(map.width) || !Number.isInteger(map.height)) {
  throw new Error("missing [map] width/height");
}
if (paths.size === 0) throw new Error("no [paths.<id>] sections");

function countBrackets(s) {
  let n = 0;
  for (const ch of s) {
    if (ch === "[") n++;
    else if (ch === "]") n--;
  }
  return n;
}

// ---- 2) 宽 3 展开（教程 06 Step 4 行为规格的独立实现）----
const W = map.width;
const H = map.height;
const grid = Array.from({ length: H }, () => new Array(W).fill("."));

for (const [id, waypoints] of paths) {
  if (waypoints.length < 2) throw new Error(`Path '${id}' must have at least 2 waypoints`);
  for (let i = 0; i + 1 < waypoints.length; i++) {
    const [x1, y1] = waypoints[i];
    const [x2, y2] = waypoints[i + 1];
    if (x1 === x2) {
      for (let y = Math.min(y1, y2); y <= Math.max(y1, y2); y++) {
        for (let dx = -1; dx <= 1; dx++) {
          const nx = x1 + dx;
          if (nx >= 0 && nx < W && y >= 0 && y < H) grid[y][nx] = "P";
        }
      }
    } else if (y1 === y2) {
      for (let x = Math.min(x1, x2); x <= Math.max(x1, x2); x++) {
        for (let dy = -1; dy <= 1; dy++) {
          const ny = y1 + dy;
          if (x >= 0 && x < W && ny >= 0 && ny < H) grid[ny][x] = "P";
        }
      }
    } else {
      throw new Error(`Path '${id}' has a diagonal segment (${x1},${y1}) -> (${x2},${y2})`);
    }
  }
}

// 拐点对角补齐（教程 06 Step 4 第二段的独立转写）：
// mx = 水平段方向（前段水平时取该段 dx 符号），否则取后段 dx 的反号；
// my = 垂直段方向（前段垂直时取该段 dy 符号），否则取后段 dy 的反号。
for (const [id, wps] of paths) {
  for (let i = 1; i + 1 < wps.length; i++) {
    const [x1, y1] = wps[i - 1];
    const [x2, y2] = wps[i];
    const [x3, y3] = wps[i + 1];
    const dx1 = x2 - x1;
    const dy1 = y2 - y1;
    const dx2 = x3 - x2;
    const dy2 = y3 - y2;
    const sgn = (v) => (v > 0 ? 1 : v < 0 ? -1 : 0);
    const mx = dy1 === 0 ? sgn(dx1) : -sgn(dx2);
    const my = dx1 === 0 ? sgn(dy1) : -sgn(dy2);
    const cx = x2 + mx;
    const cy = y2 + my;
    if (cx >= 0 && cx < W && cy >= 0 && cy < H) grid[cy][cx] = "P";
  }
  void id;
}

// ---- 3) 计数 + 形状可视化（行 9 -> 0，与游戏 Y-up 网格一致）----
let pathCount = 0;
for (let y = H - 1; y >= 0; y--) {
  for (let x = 0; x < W; x++) if (grid[y][x] === "P") pathCount++;
  console.log(`y=${String(y).padStart(2)} | ${grid[y].join("")}`);
}
for (const [id, wps] of paths) {
  console.log(`path '${id}' waypoints = ${JSON.stringify(wps)}`);
}
const total = W * H;
const grass = total - pathCount;
console.log(`EXPECTED_PATH=${pathCount} EXPECTED_GRASS=${grass} EXPECTED_TOTAL=${total}`);

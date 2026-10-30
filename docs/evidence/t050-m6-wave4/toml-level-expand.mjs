// T049-W1 独立关卡展开器：解析外部关卡 TOML，按教程 06 行为规格展开逻辑网格，
// 输出 PathTile / Grass 期望计数与波次事件数（供 assert-td.mjs 的 p13–p17 使用）。
//
// 判据纪律（派工单 + 附录 B.2）：期望值从 TOML 真解析、真展开、真计数，
// 禁手数、禁硬编码；展开规则按教程 06 Step 2/4 的行为描述独立重写
// （与 Rust 侧 `build_map_from_level` 互不引用 = 双盲对拍）：
//   - `[paths.<id>].waypoints` 为宽 1 中心线（[x, y] 网格坐标）；
//   - 水平段 = 中心行 ±1 行，垂直段 = 中心列 ±1 列；越界格静默跳过；
//   - 每个内点补外侧对角格：mx = 水平段方向（前段水平取 dx 符号，否则取后段 dx 反号），
//     my = 垂直段方向（前段垂直取 dy 符号，否则取后段 dy 反号）。
// 波次事件数 = 各 `[[waves]]` 段内 `count = N` 之和（与 `build_spawn_schedule` 独立）。
//
// CLI（树根执行）：
//   node docs/evidence/t049-m6-wave3/toml-level-expand.mjs game-td/assets/levels/level_0{1,2,3}.toml
// 输出 `<file> EXPECTED_PATH=<n> EXPECTED_GRASS=<m> TOTAL=<t> EVENTS=<k>` + 网格形状。
import { readFileSync } from "node:fs";

function countBrackets(s) {
  let n = 0;
  for (const ch of s) {
    if (ch === "[") n++;
    else if (ch === "]") n--;
  }
  return n;
}

/// 解析本工程关卡格式的 TOML 子集（表头 + 整数 + 二维整数数组 + 波次 count）。
export function parseLevelFile(file) {
  const text = readFileSync(file, "utf8");
  const lines = text
    .split(/\r?\n/)
    .map((l) => l.replace(/#.*$/, ""))
    .filter((l) => l.trim().length > 0);

  let section = null; // "map" | "paths.<id>" | "waves" | "enemy_types.<id>"
  const map = { width: null, height: null };
  const paths = new Map();
  let events = 0;
  let pendingArray = null; // 跨行 waypoints 数组累积
  let pendingPathId = null;

  const flushPending = () => {
    if (!pendingArray) return;
    if (!pendingPathId) throw new Error("pending waypoints without path id");
    const body = pendingArray.join(" ");
    const pairs = [];
    const re = /\[\s*(\d+)\s*,\s*(\d+)\s*\]/g;
    let m;
    while ((m = re.exec(body))) pairs.push([Number(m[1]), Number(m[2])]);
    paths.set(pendingPathId, pairs);
    pendingArray = null;
    pendingPathId = null;
  };

  for (const raw of lines) {
    const line = raw.trim();
    const doubleHeader = line.match(/^\[\[([^\]]+)\]\]$/);
    if (doubleHeader) {
      flushPending();
      section = doubleHeader[1].trim();
      continue;
    }
    const header = line.match(/^\[([^\]]+)\]$/);
    if (header) {
      flushPending();
      section = header[1].trim();
      if (section.startsWith("paths.")) {
        const id = section.slice("paths.".length);
        if (paths.has(id)) throw new Error(`duplicate path id: ${id}`);
        paths.set(id, []);
      }
      continue;
    }
    if (pendingArray) {
      pendingArray.push(line);
      if (countBrackets(pendingArray.join(" ")) === 0) flushPending();
      continue;
    }
    // 波次计数：`[[waves]]` 段内敌人组的 `count = N`（行内任意位置，形如
    // `{ type = "soldier", count = 3 },`）；enemy_types 段无 count。
    if (section === "waves") {
      const c = line.match(/\bcount\s*=\s*(\d+)/);
      if (c) events += Number(c[1]);
    }
    const kv = line.match(/^([A-Za-z0-9_]+)\s*=\s*(.*)$/);
    if (!kv) {
      // [[waves]] 的 enemies 数组行形如 `{ type = "soldier", count = 3 },`
      // （行首花括号、非键值行）——与地图计数无关，跳过。
      if (section === "waves") continue;
      throw new Error(`${file}: unparseable line: ${line}`);
    }
    const key = kv[1];
    const value = kv[2].trim();
    if (section === "map") {
      if (key !== "width" && key !== "height") throw new Error(`unknown [map] key: ${key}`);
      if (!/^\d+$/.test(value)) throw new Error(`non-integer ${key}: ${value}`);
      map[key] = Number(value);
      continue;
    }
    if (section && section.startsWith("paths.")) {
      if (key !== "waypoints") throw new Error(`unknown path key: ${key}`);
      const id = section.slice("paths.".length);
      if (value.startsWith("[") && countBrackets(value) > 0) {
        pendingPathId = id;
        pendingArray = [value];
      } else {
        const pairs = [];
        const re = /\[\s*(\d+)\s*,\s*(\d+)\s*\]/g;
        let m;
        while ((m = re.exec(value))) pairs.push([Number(m[1]), Number(m[2])]);
        paths.set(id, pairs);
      }
      continue;
    }
    // 其余段（enemy_types / waves 的 enemies 组）与计数无关，忽略。
  }
  flushPending();

  if (!Number.isInteger(map.width) || !Number.isInteger(map.height)) {
    throw new Error(`${file}: missing [map] width/height`);
  }
  if (paths.size === 0) throw new Error(`${file}: no [paths.<id>] sections`);
  return { file, width: map.width, height: map.height, paths, events };
}

/// 宽 3 展开 + 拐点对角补齐（教程 06 Step 4 的独立实现）。
export function expandLevel(level) {
  const W = level.width;
  const H = level.height;
  const grid = Array.from({ length: H }, () => new Array(W).fill("."));

  for (const [id, waypoints] of level.paths) {
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
        throw new Error(
          `Path '${id}' has a diagonal segment (${x1},${y1}) -> (${x2},${y2})`,
        );
      }
    }
  }

  for (const [, wps] of level.paths) {
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
  }

  let pathCount = 0;
  for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) if (grid[y][x] === "P") pathCount++;
  return { grid, pathCount, grassCount: W * H - pathCount, total: W * H };
}

if (import.meta.url === `file:///${process.argv[1].replace(/\\/g, "/")}`) {
  const files = process.argv.slice(2);
  if (files.length === 0) {
    console.error("usage: node toml-level-expand.mjs <level.toml> [more.toml ...]");
    process.exit(2);
  }
  for (const file of files) {
    const level = parseLevelFile(file);
    const ex = expandLevel(level);
    console.log(
      `${file} EXPECTED_PATH=${ex.pathCount} EXPECTED_GRASS=${ex.grassCount} ` +
        `TOTAL=${ex.total} EVENTS=${level.events}`,
    );
    for (let y = level.height - 1; y >= 0; y--) {
      console.log(`  y=${String(y).padStart(2)} | ${ex.grid[y].join("")}`);
    }
  }
}

// T047-W1 证据断言脚本（BRP over HTTP，127.0.0.1:15702）。
// 用法（树根执行，游戏进程 CWD = game-td/）：
//   node docs/evidence/t047-m6-wave1/assert-td.mjs --phase p4 --shot ../docs/evidence/t047-m6-wave1/p4-shot.png
//   node docs/evidence/t047-m6-wave1/assert-td.mjs --phase p5 --shot ../docs/evidence/t047-m6-wave1/p5-shot.png
//   node docs/evidence/t047-m6-wave1/assert-td.mjs --phase p6 --shot ../docs/evidence/t047-m6-wave1/p6-shot.png
//   node docs/evidence/t047-m6-wave1/assert-td.mjs --phase p6b            # TOML 变体（只读数，期望值由 toml-path-count.mjs 独立算）
//   node docs/evidence/t047-m6-wave1/assert-td.mjs --phase p7 --shot ../docs/evidence/t047-m6-wave1/p7-shot.png
// 判定行由脚本计算生成（禁止手算填表，附录 B.2）；退出码 0 = 全 PASS。
// 期望值来源：派工单 T047-W1 验收标准 ①–⑬（含算式）。
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const URL = "http://127.0.0.1:15702";
let id = 0;
const results = [];
function check(name, cond, detail) {
  results.push(`${cond ? "PASS" : "FAIL"} | ${name}${detail ? " | " + detail : ""}`);
  if (!cond) process.exitCode = 1;
}
function info(line) {
  results.push(`INFO | ${line}`);
}
async function brp(method, params, timeoutMs = 8000) {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(`timeout ${method}`), timeoutMs);
  try {
    const res = await fetch(URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ jsonrpc: "2.0", method, id: ++id, ...(params ? { params } : {}) }),
      signal: ctrl.signal,
    });
    const j = await res.json();
    if (j.error) throw new Error(`${method} -> ${JSON.stringify(j.error)}`);
    return j.result;
  } finally {
    clearTimeout(timer);
  }
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function arg(name, fallback = null) {
  const i = process.argv.indexOf(name);
  return i >= 0 && i + 1 < process.argv.length ? process.argv[i + 1] : fallback;
}

const phase = arg("--phase", "p4");
const shotArg = arg("--shot", "");

// ---- 常量与算式（派工单口径；脚本内独立复算，不引用游戏侧值）----
const COLS = 15;
const ROWS = 10;
const TILE = 64;
const PATH_MID = ROWS / 2; // 10/2 = 5 -> 道路行 {4,5,6}
const OFFSET_X = -(COLS * TILE) / 2 + TILE / 2; // -448
const OFFSET_Y = -(ROWS * TILE) / 2 + TILE / 2; // -288
const MAP_TILE = 150; // 15 × 10
const PATH_TILE = 45; // 3 路行 × 15 列
const GRASS_TILE = MAP_TILE - PATH_TILE; // 105

// P3 基线索引规格（= P4 auto-tiling 必须逐格复现的视觉，教程 04「What stayed the same」）：
// 行 path_mid-1 下沿 125/126/127；行 path_mid 路面 102/103/104；
// 行 path_mid+1 上沿 79/80/81；其余 129。
function expectedIndex(row, col) {
  if (row === PATH_MID - 1) {
    if (col === 0) return 125;
    if (col === COLS - 1) return 127;
    return 126;
  }
  if (row === PATH_MID) {
    if (col === 0) return 102;
    if (col === COLS - 1) return 104;
    return 103;
  }
  if (row === PATH_MID + 1) {
    if (col === 0) return 79;
    if (col === COLS - 1) return 81;
    return 80;
  }
  return 129;
}
const PATH_INDICES = new Set([79, 80, 81, 102, 103, 104, 125, 126, 127]);

const POS = "bevy_ecs_tilemap::tiles::TilePos";
const TEX = "bevy_ecs_tilemap::tiles::TileTextureIndex";
const TR = "bevy_transform::components::transform::Transform";

function tileIndex(row) {
  const raw = row.components[TEX];
  return Array.isArray(raw) ? raw[0] : raw;
}

/// 网格指纹：逐格 (x,y) -> (TileType, atlas 索引) 的 sha256（跨阶段零漂移对拍用）。
function gridDigest(rows) {
  const cells = rows
    .map((r) => {
      const p = r.components[POS];
      const tt = r.components["game_td::TileType"];
      return `${p.x},${p.y}:${tt}:${tileIndex(r)}`;
    })
    .sort();
  return createHash("sha256").update(cells.join("\n")).digest("hex");
}

// ---- 公共：方法面（⑬ 全程 24）----
const disc = await brp("rpc.discover");
const names = disc.methods.map((m) => m.name);
check("discover.version==0.20.0", disc.info.version === "0.20.0", `version=${disc.info.version}`);
check("discover.methods==24", names.length === 24, `len=${names.length}`);
check("discover contains td.screenshot", names.includes("td.screenshot"));
check(
  "discover no game.* prefix",
  names.every((n) => !n.startsWith("game.")),
  names.filter((n) => n.startsWith("game.")).join(",") || "(none)",
);

// ---- 公共：截图受理 + 轮询 + 文件核验（① ④ ⑪ 共用）----
async function captureAndVerify(brpPath, readPath) {
  if (!brpPath) {
    check("screenshot path arg", false, "缺 --shot");
    return;
  }
  const st0 = await brp("td.screenshot", { op: "status" });
  const baseId = st0.next_id;
  const queued = await brp("td.screenshot", { op: "capture", path: brpPath });
  check(
    "screenshot queued",
    queued.queued === true && queued.id === baseId,
    `id=${queued.id} baseId=${baseId} path=${queued.path}`,
  );

  let entry = null;
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    const st = await brp("td.screenshot", { op: "status" });
    entry = st.entries.find((e) => e.id === queued.id);
    if (entry && entry.status === "captured") break;
    await sleep(200);
  }
  check("screenshot status==captured", !!entry && entry.status === "captured",
    entry ? JSON.stringify(entry) : "no entry");
  if (!entry || entry.status !== "captured") return;

  let buf;
  try {
    buf = readFileSync(readPath);
  } catch (e) {
    check("screenshot file readable", false, `${readPath}: ${e.code}`);
    return;
  }
  check("screenshot file non-empty", buf.length > 0, `bytes=${buf.length}`);
  const magic = buf.subarray(0, 8).toString("hex");
  check("PNG magic", magic === "89504e470d0a1a0a", `magic=${magic}`);
  if (magic === "89504e470d0a1a0a") {
    const w = buf.readUInt32BE(16);
    const h = buf.readUInt32BE(20);
    check("PNG IHDR == log entry size", w === entry.width && h === entry.height,
      `IHDR=${w}x${h} log=${entry.width}x${entry.height}`);
    check("PNG 1600x900 (logical 1280x720 x DPI 1.25)", w === 1600 && h === 900, `${w}x${h}`);
    const md5 = createHash("md5").update(buf).digest("hex");
    info(`SCREENSHOT md5=${md5} bytes=${buf.length} size=${w}x${h}`);
  }
}

/// 地图断言核心：计数（MapTile/PathTile/TileType）+ 逐格几何 + 逐格索引 + 网格指纹。
async function assertMap(label, opts = {}) {
  const withGrid = opts.withGrid !== false;
  const all = await brp("world.query", { data: { components: ["game_td::MapTile", POS, TEX] } });
  check(`${label} MapTile count==${MAP_TILE}`, all.length === MAP_TILE, `len=${all.length}`);
  const paths = await brp("world.query", { data: { components: ["game_td::PathTile"] } });
  const types = await brp("world.query", { data: { components: ["game_td::TileType"] } });
  const byType = new Map();
  for (const row of types) {
    const v = row.components["game_td::TileType"];
    byType.set(v, (byType.get(v) ?? 0) + 1);
  }
  info(`TileType histogram = ${[...byType.entries()].map(([k, v]) => `${k}:${v}`).sort().join(" ")}`);
  check(`${label} TileType total==${MAP_TILE}`, types.length === MAP_TILE, `len=${types.length}`);
  const measured = opts.expectPathTile ?? PATH_TILE;
  check(`${label} TileType Grass==${MAP_TILE - measured}`, byType.get("Grass") === MAP_TILE - measured,
    `grass=${byType.get("Grass")}`);
  check(`${label} TileType Path==${measured}`, byType.get("Path") === measured,
    `path=${byType.get("Path")}`);

  check(`${label} PathTile count==${measured}`, paths.length === measured, `len=${paths.length}`);
  const pathsWithMarker = await brp("world.query", {
    data: { components: ["game_td::PathTile"] },
    filter: { with: ["game_td::MapTile"] },
  });
  check(`${label} PathTile subset of MapTile`, pathsWithMarker.length === paths.length,
    `len=${pathsWithMarker.length}`);

  if (!withGrid) return { all, paths, digest: null };

  // 几何：TilePos 覆盖 15×10 恰一次；世界坐标与居中算式逐格一致。
  let posBad = 0;
  const seen = new Set();
  for (const row of all) {
    const p = row.components[POS];
    if (!p || typeof p.x !== "number" || typeof p.y !== "number") { posBad++; continue; }
    seen.add(`${p.x},${p.y}`);
  }
  check(`${label} TilePos covers 15x10 grid exactly once`,
    posBad === 0 && seen.size === MAP_TILE, `bad=${posBad} covered=${seen.size}/150`);

  // 索引：逐格与 P3 基线规格（= 教程 04 声称的视觉不变）一致。
  let idxBad = 0;
  for (const row of all) {
    const p = row.components[POS];
    if (tileIndex(row) !== expectedIndex(p.y, p.x)) idxBad++;
  }
  check(`${label} texture index per cell == P3 baseline spec`, idxBad === 0, `bad=${idxBad}`);

  // ③：auto-tiling 生效——PathTile 索引 ≥2 个不同取值、全图 > 2 个。
  const pathSet = new Set(paths.map((r) => r.entity));
  const pathIdx = new Set();
  const allIdx = new Set();
  for (const row of all) {
    const idx = tileIndex(row);
    allIdx.add(idx);
    if (pathSet.has(row.entity)) pathIdx.add(idx);
  }
  check(`${label} PathTile distinct indices >=2 (auto-tiling active)`, pathIdx.size >= 2,
    `distinct=${[...pathIdx].sort((a, b) => a - b).join(",")}`);
  check(`${label} whole-map distinct indices >2`, allIdx.size > 2,
    `distinct=${[...allIdx].sort((a, b) => a - b).join(",")}`);
  check(`${label} tile index set == P3 index set`,
    [...allIdx].sort((a, b) => a - b).join(",") === [...PATH_INDICES, 129].sort((a, b) => a - b).join(","),
    `got=${[...allIdx].sort((a, b) => a - b).join(",")}`);

  const digest = gridDigest(all);
  info(`GRID_DIGEST=${digest}`);
  return { all, paths, digest };
}

// 截图参数：BRP 侧路径相对游戏进程 CWD（game-td/），读取侧相对脚本 CWD（树根）。
function readPathOf(brpPath) {
  return brpPath.startsWith("../") ? brpPath.slice(3) : brpPath;
}

if (phase === "p4") {
  // ① ② ③ ④：auto-tiling 行为面 + 截图。
  await assertMap("P4");
  await captureAndVerify(shotArg, readPathOf(shotArg));
} else if (phase === "p5") {
  // ⑤ ⑥：重构后计数逐项相同（网格指纹同 P4 档对照）+ 截图（跨运行 md5 由 shell 对比）。
  await assertMap("P5");
  await captureAndVerify(shotArg, readPathOf(shotArg));
} else if (phase === "p6") {
  // ⑧：外部化零行为变化——默认 TOML 关卡计数与 P5 全同（网格指纹亦须同）。
  await assertMap("P6");
  await captureAndVerify(shotArg, readPathOf(shotArg));
} else if (phase === "p6b") {
  // ⑦：TOML 变体读数——PathTile/TileType 计数交给 toml-path-count.mjs 独立算出的期望值核对。
  const expect = Number(arg("--expect-path", "NaN"));
  const res = await assertMap("P6-variant", { expectPathTile: expect, withGrid: false });
  const types = await brp("world.query", { data: { components: ["game_td::TileType"] } });
  const byType = new Map();
  for (const row of types) {
    const v = row.components["game_td::TileType"];
    byType.set(v, (byType.get(v) ?? 0) + 1);
  }
  check("P6-variant PathTile == independent toml expansion count", Number.isFinite(expect),
    `measured=${res.paths.length} expected=${expect}`);
  check("P6-variant TileType Grass == 150-Path",
    byType.get("Grass") === MAP_TILE - (Number.isFinite(expect) ? expect : NaN),
    `grass=${byType.get("Grass")} expected=${MAP_TILE - expect}`);
} else if (phase === "p7") {
  // ⑨ Enemy=1（教程 07 Step 3 单测试敌）；⑩ 位置式移动（FixedUpdate）位移方向；
  // ⑪ 截图含敌人（两次截图字节差异 + 敌人在场）。
  const enemies = await brp("world.query", {
    data: { components: ["game_td::Enemy", TR] },
    filter: { with: ["game_td::Enemy"] },
  });
  check("P7 Enemy count==1", enemies.length === 1, `len=${enemies.length}`);
  if (enemies.length === 1) {
    const t0 = enemies[0].components[TR].translation;
    await sleep(400);
    const again = await brp("world.query", {
      data: { components: ["game_td::Enemy", TR] },
      filter: { with: ["game_td::Enemy"] },
    });
    if (again.length !== 1) {
      check("P7 enemy alive across samples", false, `len=${again.length}（首次采样后被回收）`);
    } else {
      const t1 = again[0].components[TR].translation;
      const dx = t1[0] - t0[0];
      const dy = t1[1] - t0[1];
      info(`ENEMY t0=(${t0[0]},${t0[1]}) t1=(${t1[0]},${t1[1]}) dt=0.4s`);
      check("P7 enemy moved (delta > 0)", Math.hypot(dx, dy) > 0,
        `dist=${Math.hypot(dx, dy).toFixed(3)}`);
      // waypoint[1] 方向：默认关卡 [[0,5],[14,5]] -> +x（dy == 0）。
      check("P7 movement direction == waypoint[1] (+x)", dx > 0 && Math.abs(dy) < 1e-3,
        `dx=${dx.toFixed(3)} dy=${dy.toFixed(3)}`);
      // 位置式移动 + FixedUpdate：dt=0.4s 位移应 ≈ 192*0.4 = 76.8（容差 ±15%）。
      const expectDist = 192 * 0.4;
      const tol = 0.15;
      check("P7 displacement matches speed*dt (±15%)",
        Math.abs(Math.hypot(dx, dy) - expectDist) <= expectDist * tol,
        `dist=${Math.hypot(dx, dy).toFixed(2)} expect=${expectDist}`);
    }
    await captureAndVerify(shotArg, readPathOf(shotArg));
  }
  // 地图面不受敌人影响（P7 回归）。
  await assertMap("P7", { withGrid: true });
} else {
  check("phase arg", false, `未知 --phase ${phase}`);
}

console.log(results.join("\n"));
console.log(`SUMMARY: ${results.filter((r) => r.startsWith("PASS")).length}/${results.filter((r) => !r.startsWith("INFO")).length} PASS`);
process.exit(process.exitCode ?? 0);

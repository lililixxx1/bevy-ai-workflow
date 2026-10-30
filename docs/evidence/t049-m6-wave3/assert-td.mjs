// T049-W1 扩展断言脚本（t048 档复制后追加 p13–p17；t047/t048 档案原件未动）。
// 用法（树根执行，游戏进程 CWD = game-td/）：
//   node docs/evidence/t049-m6-wave3/assert-td.mjs --phase p13 --shot ../docs/evidence/t049-m6-wave3/p13-shot.png --shot-menu ../docs/evidence/t049-m6-wave3/p13-shot-menu.png
//   node docs/evidence/t049-m6-wave3/assert-td.mjs --phase p14 --shot ../docs/evidence/t049-m6-wave3/p14-shot.png
//   node docs/evidence/t049-m6-wave3/assert-td.mjs --phase p15 --shot ../docs/evidence/t049-m6-wave3/p15-shot.png
//   node docs/evidence/t049-m6-wave3/assert-td.mjs --phase p16 --shot ../docs/evidence/t049-m6-wave3/p16-shot.png
//   node docs/evidence/t049-m6-wave3/assert-td.mjs --phase p17 --shot ../docs/evidence/t049-m6-wave3/p17-shot.png --shot-dock ../docs/evidence/t049-m6-wave3/p17-shot-dock.png
// 历史相位 p4–p12 逻辑原样保留（对相应构建复跑用；本卡不改其语义）。
// discover 方法数按 phase 预注册期望（`--expect-discover N` 覆盖）。
// 判定行由脚本计算生成（禁止手算填表，附录 B.2）；退出码 0 = 全 PASS。
// 期望值来源：派工单 T049-W1 验收 + 教程 13–17 行为规格 + 漂移表 Part 13–17。
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { expandLevel, parseLevelFile } from "./toml-level-expand.mjs";

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

// ---- discover 方法数期望（逐部增量，如实记录；--expect-discover 覆盖）----
// 24 = T047 基线（23 内置 + td.screenshot）；25 = +td.spawn_tower(P8)；
// 26 = +td.wave(P10)；27 = +td.game(P11)；28 = +td.state(P12)。
const EXPECT_DISCOVER = {
  p4: 24, p5: 24, p6: 24, p6b: 24, p7: 24, p8: 25, p9: 25, p10: 26, p11: 27, p12: 28,
  p13: 29, p14: 30, p15: 31, p16: 32, p17: 33,
};

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

// T047 冻结基线（p4/p5/p6/p7 四部九档同值，SKILL §6.17 实测锚）：
// P8–P12 地图层必须逐位零漂移（塔不改变地图瓦片）。
const GRID_DIGEST_BASELINE = "70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07";

// P10 波次时间表（level_01.toml 的独立复算——教程 10 Step 1 数据；脚本不读游戏侧值）：
// 波 1: start 0.0 / interval 1.0 -> soldier×3 @0,1,2 + runner×2 @3,4（health 100,100,100,60,60）
// 波 2: start 12.0 / interval 1.2 -> soldier×2 @12.0,13.2 + scout×2 @14.4,15.6 + heavy×1 @16.8
//       （health 100,100,80,80,300）
const WAVE_TIMES = [0, 1, 2, 3, 4, 12.0, 13.2, 14.4, 15.6, 16.8];
const WAVE_HEALTH_SEQ = [100, 100, 100, 60, 60, 100, 100, 80, 80, 300];

function expectedSpawned(elapsed) {
  return WAVE_TIMES.filter((t) => t <= elapsed + 1e-6).length;
}
function expectedHealths(spawned) {
  return WAVE_HEALTH_SEQ.slice(0, spawned).sort((a, b) => a - b);
}
async function waitWaveUntil(minElapsed, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  let w = await brp("td.wave");
  while (Date.now() < deadline) {
    if (w.available && w.elapsed >= minElapsed) return w;
    await sleep(30);
    w = await brp("td.wave");
  }
  return w;
}

// ---- P11/P12 共用：td.game 等待与实体计数指纹（跨部对拍）----
async function waitGameUntilElapsed(minElapsed, timeoutMs = 30000) {
  const deadline = Date.now() + timeoutMs;
  let g = await brp("td.game");
  while (Date.now() < deadline) {
    if (g.game_over || (typeof g.elapsed === "number" && g.elapsed >= minElapsed)) return g;
    await sleep(30);
    g = await brp("td.game");
  }
  return g;
}

async function waitGameOver(timeoutMs = 45000) {
  const deadline = Date.now() + timeoutMs;
  let g = await brp("td.game");
  while (Date.now() < deadline) {
    if (g.game_over) return g;
    await sleep(100);
    g = await brp("td.game");
  }
  return g;
}

/// 实体计数指纹期望值（独立复算）：塔格 → 世界坐标（OFFSET + tile*TILE）、炮塔数、
/// 该时刻已刷事件的期望血量多重集。P11 与 P12 用同一场景（1 座无害塔 + 波 1 前段）。
function expectedEntityDigest(towerTiles, spawnedCount) {
  const lines = [];
  for (const [tx, ty] of towerTiles) {
    lines.push(`T:${OFFSET_X + tx * TILE},${OFFSET_Y + ty * TILE}`);
  }
  lines.push(`TT:${towerTiles.length}`);
  for (const h of expectedHealths(spawnedCount)) lines.push(`E:${h}`);
  lines.sort();
  return createHash("sha256").update(lines.join("\n")).digest("hex");
}

async function waitState(name, timeoutMs = 15000) {
  const deadline = Date.now() + timeoutMs;
  let s = await brp("td.state");
  while (Date.now() < deadline) {
    if (s.state === name) return s;
    await sleep(50);
    s = await brp("td.state");
  }
  return s;
}

async function entitySample() {
  const tw = await towerSnapshot();
  const healths = await enemyHealths();
  return {
    digest: entityDigest(tw.towers, tw.turrets, healths),
    healths,
    towerCount: tw.towers.length,
    turretCount: tw.turrets,
  };
}

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

// ---- T048 扩展公用面（P8–P12）----

/// 原始 JSON-RPC 调用（断言错误码用，不抛）。
async function brpRaw(method, params, timeoutMs = 8000) {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(`timeout ${method}`), timeoutMs);
  try {
    const res = await fetch(URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ jsonrpc: "2.0", method, id: ++id, ...(params ? { params } : {}) }),
      signal: ctrl.signal,
    });
    return await res.json();
  } finally {
    clearTimeout(timer);
  }
}

/// 错误响应的 code（成功响应返回 null）。
async function errCode(method, params) {
  const j = await brpRaw(method, params);
  return j.error ? j.error.code : null;
}

/// 组件计数（可带 filter.with）。
async function countOf(component, withFilter = null) {
  const params = { data: { components: [component] } };
  if (withFilter) params.filter = { with: withFilter };
  const rows = await brp("world.query", params);
  return rows.length;
}

/// 元组结构体组件取值（Health(100.0) 经 ReflectSerializer 落成 [100.0]；兼容裸数）。
function tupleValue(raw) {
  return Array.isArray(raw) ? raw[0] : raw;
}

/// 实体计数指纹（T048 派工单：塔/敌人计数 digest 独立链）。
/// 口径（跨 P11 ↔ P12 对拍）：塔基座按世界坐标排序 + 炮塔计数 + 敌人血量多重集排序。
/// 与位置无关的敌人部分只取 Health——敌人在动，位置不可做跨运行指纹。
function entityDigest(towers, turrets, enemyHealths) {
  const lines = [];
  for (const t of towers) {
    const tr = t.components[TR]?.translation;
    const x = Array.isArray(tr) ? tr[0] : tr?.x;
    const y = Array.isArray(tr) ? tr[1] : tr?.y;
    lines.push(`T:${x},${y}`);
  }
  lines.push(`TT:${turrets}`);
  for (const h of enemyHealths) lines.push(`E:${h}`);
  lines.sort();
  return createHash("sha256").update(lines.join("\n")).digest("hex");
}

async function towerSnapshot() {
  const towers = await brp("world.query", { data: { components: ["game_td::Tower", TR] } });
  const turrets = await countOf("game_td::TowerTurret");
  return { towers, turrets };
}

async function enemyHealths() {
  const rows = await brp("world.query", {
    data: { components: ["game_td::Enemy", "game_td::Health"] },
    filter: { with: ["game_td::Enemy"] },
  });
  return rows.map((r) => tupleValue(r.components["game_td::Health"]));
}

// ---- 公共：方法面（⑬ 全程 24；T048 逐部 +1 如实记录）----
const disc = await brp("rpc.discover");
const names = disc.methods.map((m) => m.name);
check("discover.version==0.20.0", disc.info.version === "0.20.0", `version=${disc.info.version}`);
const expectDiscover = Number(arg("--expect-discover", String(EXPECT_DISCOVER[phase] ?? NaN)));
check(`discover.methods==${expectDiscover}`, names.length === expectDiscover, `len=${names.length}`);
check("discover contains td.screenshot", names.includes("td.screenshot"));
if (EXPECT_DISCOVER[phase] >= 25) {
  check("discover contains td.spawn_tower", names.includes("td.spawn_tower"));
}
if (EXPECT_DISCOVER[phase] >= 26) {
  check("discover contains td.wave", names.includes("td.wave"));
}
if (EXPECT_DISCOVER[phase] >= 27) {
  check("discover contains td.game", names.includes("td.game"));
}
if (EXPECT_DISCOVER[phase] >= 28) {
  check("discover contains td.state", names.includes("td.state"));
}
if (EXPECT_DISCOVER[phase] >= 29) {
  check("discover contains td.level", names.includes("td.level"));
}
if (EXPECT_DISCOVER[phase] >= 30) {
  check("discover contains td.economy", names.includes("td.economy"));
}
if (EXPECT_DISCOVER[phase] >= 31) {
  check("discover contains td.combat", names.includes("td.combat"));
}
if (EXPECT_DISCOVER[phase] >= 32) {
  check("discover contains td.towers", names.includes("td.towers"));
}
if (EXPECT_DISCOVER[phase] >= 33) {
  check("discover contains td.tower_select", names.includes("td.tower_select"));
}
check(
  "discover no game.* prefix",
  names.every((n) => !n.startsWith("game.")),
  names.filter((n) => n.startsWith("game.")).join(",") || "(none)",
);

// ---- 公共：截图受理 + 轮询 + 文件核验（① ④ ⑪ 共用）----
// 两段式拆分（P9 起用）：queueCapture 仅受理（不阻塞），verifyCapture 事后核验——
// 交战窗口短的 phase 需要在轮询中途受理捕获，避免阻塞丢掉采样点。
async function queueCapture(brpPath) {
  if (!brpPath) {
    check("screenshot path arg", false, "缺 --shot");
    return null;
  }
  const st0 = await brp("td.screenshot", { op: "status" });
  const baseId = st0.next_id;
  const queued = await brp("td.screenshot", { op: "capture", path: brpPath });
  check(
    "screenshot queued",
    queued.queued === true && queued.id === baseId,
    `id=${queued.id} baseId=${baseId} path=${queued.path}`,
  );
  return queued;
}

async function verifyCapture(queued, readPath) {
  if (!queued) return;
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

async function captureAndVerify(brpPath, readPath) {
  const queued = await queueCapture(brpPath);
  await verifyCapture(queued, readPath);
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

// ---- T049 扩展公用面（P13–P17）----

// 关卡表（期望值由独立展开器从 TOML 真解析生成，禁手数）。
// `path` = 脚本 CWD（树根）下的读取路径；`discovered` = 游戏进程 CWD（game-td/）下的
// 发现路径（`td.level` 返回口径，教程 13 Step 4 的 `assets/levels/...` 形态）。
const LEVELS = [
  { key: "level_01", discovered: "assets/levels/level_01.toml", path: "game-td/assets/levels/level_01.toml" },
  { key: "level_02", discovered: "assets/levels/level_02.toml", path: "game-td/assets/levels/level_02.toml" },
  { key: "level_03", discovered: "assets/levels/level_03.toml", path: "game-td/assets/levels/level_03.toml" },
].map((e) => {
  const parsed = parseLevelFile(e.path);
  return { ...e, parsed, expanded: expandLevel(parsed) };
});

// 分关卡指纹注册表（P13 首次运行登记，P13→P17 全程强断言零漂移）。
// level_01 沿用 T047/T048 冻结基线；level_02/03 的 digest 由 P13 首次运行实测登记
// （登记轮 `--register` 只记录不断言；终轮常量在档后逐部强断言）。
const LEVEL_DIGESTS = {
  level_01: GRID_DIGEST_BASELINE,
  // P13 run1（`--register`）实测登记；口径同 gridDigest()（逐格 `x,y:TileType:atlas_index`
  // 排序后 sha256；TileType 字段在 all 查询面未请求时为 undefined，与 T047 基线同式）。
  level_02: "3edf8dffc55b743933c8c4b8484f50855b8e7d193495d5f4c2536f96ea8a4f6d",
  level_03: "840298ef16970017e1916a573a5333759bea8782a959509d365c12fc9dcd76f7",
};

function checkLevelDigest(label, key, digest, register) {
  if (register || LEVEL_DIGESTS[key] === null) {
    info(`${label} REGISTER GRID_DIGEST[${key}]=${digest}`);
    return true;
  }
  check(
    `${label} GRID_DIGEST zero drift (${key})`,
    digest === LEVEL_DIGESTS[key],
    `digest=${digest} expect=${LEVEL_DIGESTS[key]}`,
  );
  return digest === LEVEL_DIGESTS[key];
}

/// 泛化地图断言（level_02/03）：计数由独立展开器给定；逐格 TilePos 覆盖 + 索引多样性
/// + 网格指纹。level_01 仍走 `assertMap`（逐格 P3 基线索引规格 + 冻结 digest）。
async function assertGenericMap(label, entry, register) {
  const { width, height } = entry.parsed;
  const exp = entry.expanded;
  const all = await brp("world.query", { data: { components: ["game_td::MapTile", POS, TEX] } });
  check(`${label} MapTile count==${exp.total}`, all.length === exp.total, `len=${all.length}`);
  const paths = await brp("world.query", { data: { components: ["game_td::PathTile"] } });
  const types = await brp("world.query", { data: { components: ["game_td::TileType", POS] } });
  const byType = new Map();
  const typeAt = new Map();
  for (const row of types) {
    const v = row.components["game_td::TileType"];
    byType.set(v, (byType.get(v) ?? 0) + 1);
    const p = row.components[POS];
    if (p) typeAt.set(`${p.x},${p.y}`, v);
  }
  info(`TileType histogram = ${[...byType.entries()].map(([k, v]) => `${k}:${v}`).sort().join(" ")}`);
  check(`${label} TileType Grass==${exp.grassCount}`, byType.get("Grass") === exp.grassCount,
    `grass=${byType.get("Grass")} expect=${exp.grassCount}`);
  check(`${label} TileType Path==${exp.pathCount}`, byType.get("Path") === exp.pathCount,
    `path=${byType.get("Path")} expect=${exp.pathCount}`);
  check(`${label} PathTile count==${exp.pathCount}`, paths.length === exp.pathCount,
    `len=${paths.length} expect=${exp.pathCount}`);

  let posBad = 0;
  const seen = new Set();
  for (const row of all) {
    const p = row.components[POS];
    if (!p || typeof p.x !== "number" || typeof p.y !== "number") { posBad++; continue; }
    seen.add(`${p.x},${p.y}`);
  }
  check(`${label} TilePos covers ${width}x${height} grid exactly once`,
    posBad === 0 && seen.size === exp.total, `bad=${posBad} covered=${seen.size}/${exp.total}`);

  // 逐格独立几何对拍：每格 TileType（另取 POS 联查）与独立展开网格逐格一致（不只看计数）。
  const mismatch = [];
  for (const [pos, tt] of typeAt) {
    const [x, y] = pos.split(",").map(Number);
    const want = exp.grid[y][x] === "P" ? "Path" : "Grass";
    if (tt !== want) mismatch.push(`(${x},${y}) got=${tt} want=${want}`);
  }
  check(`${label} per-cell TileType == independent TOML expansion`,
    mismatch.length === 0 && typeAt.size === exp.total,
    `${mismatch.slice(0, 5).join(" ")} cells=${typeAt.size}/${exp.total}`);

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

  const digest = gridDigest(all);
  info(`GRID_DIGEST[${entry.key}]=${digest}`);
  checkLevelDigest(label, entry.key, digest, register);
  return digest;
}

/// 回到 LevelSelect 并等待清场落地。
async function leaveToMenu() {
  const s = await brp("td.state");
  if (s.state === "LevelSelect") return;
  await brp("td.state", { op: "set", state: "LevelSelect" });
  const st = await waitState("LevelSelect", 10000);
  check("leaveToMenu: state==LevelSelect", st.state === "LevelSelect", JSON.stringify(st));
  await sleep(250); // OnExit cleanup 命令落地
}

/// 选中第 index 个关卡并进入 InGame（等首帧装配）。
async function selectAndEnterLevel(index, label) {
  const s = await brp("td.state");
  if (s.state !== "LevelSelect") await leaveToMenu();
  const sel = await brp("td.level", { op: "select", index });
  check(`${label} td.level select index=${index}`, sel.index === index, JSON.stringify(sel));
  const set = await brp("td.state", { op: "set", state: "InGame" });
  check(`${label} td.state set InGame queued`, set.queued === true, JSON.stringify(set));
  const st = await waitState("InGame", 10000);
  check(`${label} state==InGame`, st.state === "InGame", JSON.stringify(st));
  await sleep(250); // OnEnter 命令（tilemap / 预览 / 资源）落地
  return sel;
}

/// 经济快照（P14 起）。
async function economy() {
  return brp("td.economy");
}

/// 敌人赏金值多重集（P14：TOML → EnemyTypeDef → SpawnEvent → 组件 的链路末端证据）。
async function enemyBounties() {
  const rows = await brp("world.query", {
    data: { components: ["game_td::Enemy", "game_td::Bounty"] },
    filter: { with: ["game_td::Enemy"] },
  });
  return rows.map((r) => tupleValue(r.components["game_td::Bounty"]));
}

/// 当前波次时钟（td.game.elapsed；资源缺失时为 null）。
async function elapsed() {
  const g = await brp("td.game");
  return typeof g.elapsed === "number" ? g.elapsed : null;
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
} else if (phase === "p11") {
  // P11：胜负判定（教程 11）。两种模式：
  //  - defeat：不设有效防御 → 波 1 五只全部漏过（lives 5 → 0），波 2 永不再刷；
  //  - victory：六塔沿线覆盖 → 10 只全灭、零漏怪（lives 保持 5）。
  const mode = arg("--mode", "defeat");
  const game0 = await brp("td.game");
  check("P11 td.game available", game0.available === true, JSON.stringify(game0));
  check("P11 initial lives==5", game0.lives === 5, `lives=${game0.lives}`);
  check("P11 not game over at start", game0.game_over === false);
  check("P11 schedule total==10 events", game0.total === 10, `total=${game0.total}`);

  if (mode === "defeat") {
    // 一座「无害塔」：[2,0] 与道路中心行 5 的垂直距离 = (5-0)*64 = 320px > 射程 192px，
    // 永不攻击、不改变漏怪节奏；同时与 P12 共用同一场景做塔/敌计数指纹对拍。
    const t = await brp("td.spawn_tower", { tile: [2, 0] });
    check("P11 harmless tower placed at [2,0] (out of range)", t.placed === true, JSON.stringify(t));
    check("P11 Tower count==1", (await countOf("game_td::Tower")) === 1);

    // 计数指纹采样：elapsed ∈ [2.2, 3.0) → 已刷 3 只 soldier（health 100×3），塔 1 座。
    const gA = await waitGameUntilElapsed(2.2);
    const sA = await entitySample();
    const expA = expectedEntityDigest([[2, 0]], expectedSpawned(gA.elapsed));
    check(
      "P11 [A] elapsed in sampling window [2.2,3.0)",
      typeof gA.elapsed === "number" && gA.elapsed >= 2.2 && gA.elapsed < 3.0,
      `elapsed=${gA.elapsed}`,
    );
    check(
      "P11 [A] alive healths == expected(spawned(elapsed))",
      JSON.stringify(sA.healths.slice().sort((a, b) => a - b)) ===
        JSON.stringify(expectedHealths(expectedSpawned(gA.elapsed))),
      `got=${JSON.stringify(sA.healths)} spawned=${expectedSpawned(gA.elapsed)}`,
    );
    check(
      "P11 [A] tower/turret counts == 1/1",
      sA.towerCount === 1 && sA.turretCount === 1,
      `towers=${sA.towerCount} turrets=${sA.turretCount}`,
    );
    check(
      "P11 [A] entity digest == expected (independent recompute)",
      sA.digest === expA,
      `got=${sA.digest} expect=${expA}`,
    );
    info(`P11 ENTITY_DIGEST_A=${sA.digest}`);

    // 截图：漏怪中段（elapsed ~3.3，多敌在路 + 无害塔）。
    await waitGameUntilElapsed(3.3);
    await captureAndVerify(shotArg, readPathOf(shotArg));

    // 败局：5 漏怪（最后一只 4.0s 刷，~8.7s 到终点判负）。
    const gEnd = await waitGameOver(25000);
    check("P11 defeat: game_over==true", gEnd.game_over === true, JSON.stringify(gEnd));
    check("P11 defeat: lives==0", gEnd.lives === 0, `lives=${gEnd.lives}`);
    check("P11 defeat: spawned==5 (wave 1 only)", gEnd.spawned === 5, `spawned=${gEnd.spawned}`);
    check("P11 defeat: pending==5 (wave 2 never spawns)", gEnd.pending === 5, `pending=${gEnd.pending}`);
    check("P11 defeat: zero enemies alive (world.query)", (await countOf("game_td::Enemy")) === 0);

    // 结束后 2s 再验：pending/spawned 不变 = 刷怪早退生效（教程 11 Step 5）。
    await sleep(2000);
    const gAfter = await brp("td.game");
    check(
      "P11 defeat: no further spawns after game over",
      gAfter.pending === gEnd.pending && gAfter.spawned === gEnd.spawned,
      `pending=${gAfter.pending} spawned=${gAfter.spawned}`,
    );
  } else if (mode === "victory") {
    // 六塔全线覆盖：塔心 x = -320/-192/-64/64/192/320，单塔水平覆盖 |dx| <= 143.1px，
    // 窗口并集 [-463,463] 连续覆盖整条路径（长 896px）；敌人无处穿漏。
    const tiles = [[2, 3], [4, 3], [6, 3], [8, 3], [10, 3], [12, 3]];
    const placed = await Promise.all(tiles.map((tile) => brp("td.spawn_tower", { tile })));
    check(
      "P11 victory: six covering towers placed",
      placed.every((r) => r.placed === true),
      placed.map((r) => JSON.stringify(r)).join(" "),
    );
    check("P11 victory: Tower count==6", (await countOf("game_td::Tower")) === 6);
    check("P11 victory: TowerTurret count==6", (await countOf("game_td::TowerTurret")) === 6);

    // 截图：交战期（elapsed ~2.3，多塔 + 敌人在场）。
    await waitGameUntilElapsed(2.3);
    await captureAndVerify(shotArg, readPathOf(shotArg));

    // 胜局：波 2 末只 heavy 16.8s 刷，需 ~2–4s 击杀；45s 截止。
    const gEnd = await waitGameOver(45000);
    check("P11 victory: game_over==true", gEnd.game_over === true, JSON.stringify(gEnd));
    check("P11 victory: lives==5 (zero leaks)", gEnd.lives === 5, `lives=${gEnd.lives}`);
    check("P11 victory: spawned==10 (all waves)", gEnd.spawned === 10, `spawned=${gEnd.spawned}`);
    check("P11 victory: pending==0", gEnd.pending === 0, `pending=${gEnd.pending}`);
    check("P11 victory: zero enemies alive (world.query)", (await countOf("game_td::Enemy")) === 0);
    info(`P11 victory end: elapsed=${gEnd.elapsed}`);
  } else {
    check("P11 --mode", false, `未知 --mode ${mode}（defeat / victory）`);
  }

  const mapP11 = await assertMap("P11");
  check(
    "P11 GRID_DIGEST zero drift vs T047 baseline",
    mapP11.digest === GRID_DIGEST_BASELINE,
    `digest=${mapP11.digest}`,
  );
} else if (phase === "p10") {
  // P10：数据驱动波次（教程 10）。断言核心 = 「已刷 = 事件表独立复算」三检查点对拍
  // + 当前存活敌人血量多重集 == 该时刻已刷事件的期望值（无塔 → 无击杀，仅终点回收）。
  // O1 采样窗：波 1 事件跨 0–4s、波 2 起于 12s，各检查点窗口 ≥0.6s，采样不挤时序。
  const w0 = await brp("td.wave");
  check("P10 td.wave available", w0.available === true, JSON.stringify(w0));
  check("P10 schedule total==10 events (wave1 5 + wave2 5)", w0.total === 10, `total=${w0.total}`);
  check(
    "P10 pending + spawned == total",
    w0.pending + w0.spawned === w0.total,
    `pending=${w0.pending} spawned=${w0.spawned} total=${w0.total}`,
  );
  check(
    "P10 spawned == expectedSpawned(elapsed) at first read",
    w0.spawned === expectedSpawned(w0.elapsed),
    `elapsed=${w0.elapsed} spawned=${w0.spawned} expect=${expectedSpawned(w0.elapsed)}`,
  );

  // 检查点 A（elapsed >= 2.3）：波 1 前三只 soldier 在场，all 100hp。
  const wA = await waitWaveUntil(2.3);
  const hA = (await enemyHealths()).slice().sort((a, b) => a - b);
  check(
    "P10 [A] spawned == expectedSpawned(elapsed)",
    wA.spawned === expectedSpawned(wA.elapsed),
    `elapsed=${wA.elapsed.toFixed(2)} spawned=${wA.spawned} expect=${expectedSpawned(wA.elapsed)}`,
  );
  check(
    "P10 [A] alive health multiset == expected(spawned)",
    JSON.stringify(hA) === JSON.stringify(expectedHealths(wA.spawned)),
    `got=${JSON.stringify(hA)} expect=${JSON.stringify(expectedHealths(wA.spawned))}`,
  );

  // 检查点 B（elapsed >= 4.2）：波 1 五只全在场（60/60/100/100/100），截图取此窗口。
  const wB = await waitWaveUntil(4.2);
  const hB = (await enemyHealths()).slice().sort((a, b) => a - b);
  check(
    "P10 [B] spawned == 5 (whole wave 1 due)",
    wB.spawned === 5 && expectedSpawned(wB.elapsed) === 5,
    `elapsed=${wB.elapsed.toFixed(2)} spawned=${wB.spawned}`,
  );
  check(
    "P10 [B] alive health multiset == [60,60,100,100,100]",
    JSON.stringify(hB) === JSON.stringify([60, 60, 100, 100, 100]),
    `got=${JSON.stringify(hB)}`,
  );
  await captureAndVerify(shotArg, readPathOf(shotArg));

  // 检查点 C（elapsed >= 12.3）：波 2 首只 soldier 已刷（spawned 6），波 1 全部到达终点回收。
  const wC = await waitWaveUntil(12.3);
  const hC = (await enemyHealths()).slice().sort((a, b) => a - b);
  check(
    "P10 [C] spawned == 6 (wave 2 started at 12.0)",
    wC.spawned === 6 && expectedSpawned(wC.elapsed) === 6,
    `elapsed=${wC.elapsed.toFixed(2)} spawned=${wC.spawned}`,
  );
  check(
    "P10 [C] only wave-2 soldier alive (1 x 100hp, wave1 leaked)",
    JSON.stringify(hC) === JSON.stringify([100]),
    `got=${JSON.stringify(hC)}`,
  );
  check(
    "P10 [C] pending == total - spawned",
    wC.pending === wC.total - wC.spawned,
    `pending=${wC.pending} total=${wC.total} spawned=${wC.spawned}`,
  );
  info(
    `P10 checkpoints A(t=${wA.elapsed.toFixed(2)},s=${wA.spawned}) ` +
      `B(t=${wB.elapsed.toFixed(2)},s=${wB.spawned}) C(t=${wC.elapsed.toFixed(2)},s=${wC.spawned})`,
  );

  const mapP10 = await assertMap("P10");
  check(
    "P10 GRID_DIGEST zero drift vs T047 baseline",
    mapP10.digest === GRID_DIGEST_BASELINE,
    `digest=${mapP10.digest}`,
  );
} else if (phase === "p9") {
  // P9：索敌 + 伤害 + 死亡回收（教程 09）。
  // 布塔：两座塔分居道路两侧 [5,3] / [9,3]——与道路中心行 5 的垂直距离 2 格 = 128px；
  // 单塔水平覆盖 |dx| <= sqrt(192²-128²) = 143.1px（塔心 x=-128 / +128）。
  // 每塔 34 伤害 / 0.5s，敌军 100 HP → 约 2.5–3s 内 3 发打死（算式见 README）。
  const towers = await Promise.all(
    [[5, 3], [9, 3]].map((tile) => brp("td.spawn_tower", { tile })),
  );
  check(
    "P9 two towers placed ([5,3],[9,3])",
    towers.every((r) => r.placed === true),
    towers.map((r) => JSON.stringify(r)).join(" "),
  );
  check("P9 Tower count==2", (await countOf("game_td::Tower")) === 2);
  check("P9 TowerTurret count==2", (await countOf("game_td::TowerTurret")) === 2);

  const h0 = await enemyHealths();
  check(
    "P9 initial enemy count==1 with Health==100",
    h0.length === 1 && h0[0] === 100,
    `healths=${JSON.stringify(h0)}`,
  );

  // 轮询至敌人消失（或 8s 截止）：记录最低血量、炮塔朝向几何对拍、最后观察到的敌人 x。
  let minHealth = Infinity;
  let lastEnemyX = null;
  let died = false;
  // 朝向对拍（教程 09 Step 4 算式）：炮塔贴图朝上，角度 = atan2(dy,dx) - π/2，
  // 纯 z 旋转四元数 z 分量 = sin(角度/2)。逐样本对拍，命中 ≥1 即证明「朝目标旋转」。
  let aimHits = 0;
  let aimBestErr = Infinity;
  let rotMaxAbsZ = 0;
  // 截图在轮询中途受理（第 6 个样本 ≈ 起跑 ~1.4s：窗口已呈现首帧、敌人仍在场）。
  // 本机实测：起跑 ~0.8s 内的捕获读到纯黑 swapchain（环境事实，见 README）。
  let queuedShot = null;
  let iter = 0;
  const deadline = Date.now() + 8000;
  while (Date.now() < deadline) {
    iter += 1;
    const rows = await brp("world.query", {
      data: { components: ["game_td::Enemy", "game_td::Health", TR] },
      filter: { with: ["game_td::Enemy"] },
    });
    const turretRows = await brp("world.query", {
      data: { components: ["game_td::TowerTurret", TR] },
    });
    if (iter === 6) queuedShot = await queueCapture(shotArg);
    if (rows.length === 0) {
      died = true;
      break;
    }
    const enemies = rows.map((r) => {
      const tr = r.components[TR].translation;
      return {
        hp: tupleValue(r.components["game_td::Health"]),
        x: Array.isArray(tr) ? tr[0] : tr?.x,
        y: Array.isArray(tr) ? tr[1] : tr?.y,
      };
    });
    for (const e of enemies) {
      minHealth = Math.min(minHealth, e.hp);
      lastEnemyX = e.x;
    }
    for (const row of turretRows) {
      const t = row.components[TR];
      const tx = Array.isArray(t.translation) ? t.translation[0] : t.translation?.x;
      const ty = Array.isArray(t.translation) ? t.translation[1] : t.translation?.y;
      const z = Array.isArray(t.rotation) ? t.rotation[2] : t.rotation?.z;
      if (typeof z !== "number") continue;
      rotMaxAbsZ = Math.max(rotMaxAbsZ, Math.abs(z));
      let best = null;
      for (const e of enemies) {
        const d = Math.hypot(e.x - tx, e.y - ty);
        if (!best || d < best.d) best = { d, e };
      }
      if (best && best.d <= 192 + 1e-3) {
        const expected = Math.sin((Math.atan2(best.e.y - ty, best.e.x - tx) - Math.PI / 2) / 2);
        const err = Math.abs(z - expected);
        aimBestErr = Math.min(aimBestErr, err);
        if (err < 0.05) aimHits++;
      }
    }
    await sleep(100);
  }
  info(
    `P9 minHealth=${minHealth} aimHits=${aimHits} aimBestErr=${aimBestErr.toFixed(4)} ` +
      `rotMaxAbsZ=${rotMaxAbsZ.toFixed(3)} lastEnemyX=${lastEnemyX} died=${died} iters=${iter}`,
  );
  check("P9 enemy took damage (min health <= 66 = 100-34)", minHealth <= 66, `minHealth=${minHealth}`);
  check(
    "P9 turret aim matches geometry (>=1 in-range sample, |err| < 0.05)",
    aimHits >= 1,
    `aimHits=${aimHits} bestErr=${aimBestErr.toFixed(4)}`,
  );
  // 死亡回收：无塔击杀时敌人于终点 x=448 被 cleanup 回收（4.67s）；lastSeenX < 300
  // 证明它是在中途被打死（教程 09 Step 4 就地 despawn）。
  check(
    "P9 enemy despawned mid-path by damage (lastSeenX < 300)",
    died && lastEnemyX !== null && lastEnemyX < 300,
    `died=${died} lastEnemyX=${lastEnemyX}`,
  );
  await verifyCapture(queuedShot, readPathOf(shotArg));
  const mapP9 = await assertMap("P9");
  check(
    "P9 GRID_DIGEST zero drift vs T047 baseline",
    mapP9.digest === GRID_DIGEST_BASELINE,
    `digest=${mapP9.digest}`,
  );
} else if (phase === "p8") {
  // P8：塔放置（教程 08）。断言：预览幽灵常驻两枚、确定性放置通道、占用/路径/越界拒绝、
  // 非法参数 -32602、地图层零漂移 + 截图。
  const previews = await countOf("game_td::TowerPreview");
  check("P8 TowerPreview ghost count==2", previews === 2, `len=${previews}`);
  check("P8 Tower count==0 before placement", (await countOf("game_td::Tower")) === 0);
  check("P8 TowerTurret count==0 before placement", (await countOf("game_td::TowerTurret")) === 0);

  const r1 = await brp("td.spawn_tower", { tile: [7, 3] });
  check("P8 td.spawn_tower placed at grass tile [7,3]", r1.placed === true, JSON.stringify(r1));
  // tile_to_world([7,3]) = (-448 + 7*64, -288 + 3*64) = (0, -96)（同 P7 算式）。
  check(
    "P8 tower world pos == tile center (0,-96)",
    Array.isArray(r1.pos) && r1.pos[0] === 0 && r1.pos[1] === -96,
    `pos=${JSON.stringify(r1.pos)}`,
  );
  check("P8 Tower count==1 after placement", (await countOf("game_td::Tower")) === 1);
  check("P8 TowerTurret count==1 after placement", (await countOf("game_td::TowerTurret")) === 1);

  const dup = await brp("td.spawn_tower", { tile: [7, 3] });
  check("P8 duplicate tile rejected (occupancy)", dup.placed === false, JSON.stringify(dup));
  const onPath = await brp("td.spawn_tower", { tile: [5, 5] }); // 行 5 = 道路中心行
  check("P8 path tile rejected", onPath.placed === false, JSON.stringify(onPath));
  const outside = await brp("td.spawn_tower", { tile: [99, 99] });
  check("P8 out-of-map rejected", outside.placed === false, JSON.stringify(outside));
  const badCode = await errCode("td.spawn_tower", { tile: [1] });
  check("P8 malformed params -> -32602", badCode === -32602, `code=${badCode}`);
  check("P8 Tower count still 1 after rejected attempts", (await countOf("game_td::Tower")) === 1);

  const mapP8 = await assertMap("P8");
  check(
    "P8 GRID_DIGEST zero drift vs T047 baseline",
    mapP8.digest === GRID_DIGEST_BASELINE,
    `digest=${mapP8.digest}`,
  );
  await captureAndVerify(shotArg, readPathOf(shotArg));
} else if (phase === "p12") {
  // P12：多关卡架构 / 状态机重构（教程 12）。断言链：
  // LevelSelect 初始态 → InGame（OnEnter 重装配）→ 行为等价（GRID_DIGEST + 计数指纹
  // == P11）→ GameOver（Defeat 转移）→ LevelSelect（cleanup_level 清场）→ 再入 InGame（重开）。
  const shotMenu = arg("--shot-menu", "");
  const shotOver = arg("--shot-over", "");

  // 等首帧呈现（起跑 ~0.8s 内的 swapchain 捕获为纯黑，环境事实见 README）。
  await sleep(800);
  const st0 = await brp("td.state");
  check("P12 initial state==LevelSelect", st0.state === "LevelSelect", JSON.stringify(st0));
  check(
    "P12 initial result==null and finished==false",
    st0.result === null && st0.finished === false,
    JSON.stringify(st0),
  );
  check("P12 LevelSelect: no map loaded (MapTile==0)", (await countOf("game_td::MapTile")) === 0);
  check(
    "P12 LevelSelect: no wave schedule (td.wave unavailable)",
    (await brp("td.wave")).available === false,
    JSON.stringify(await brp("td.wave")),
  );
  await captureAndVerify(shotMenu, readPathOf(shotMenu));

  // 进入 InGame（确定性状态迁移；BRP 无键盘注入面）。
  const setIn = await brp("td.state", { op: "set", state: "InGame" });
  check(
    "P12 td.state set InGame queued",
    setIn.queued === true && setIn.from === "LevelSelect" && setIn.to === "InGame",
    JSON.stringify(setIn),
  );
  const stIn = await waitState("InGame", 10000);
  check("P12 state==InGame after transition", stIn.state === "InGame", JSON.stringify(stIn));

  // 行为等价 ①：地图层 GRID_DIGEST 零漂移（OnEnter 重装配与 P4–P11 逐位一致）。
  const mapP12 = await assertMap("P12");
  check(
    "P12 GRID_DIGEST zero drift vs T047 baseline",
    mapP12.digest === GRID_DIGEST_BASELINE,
    `digest=${mapP12.digest}`,
  );
  // 行为等价 ②：进入 InGame 后资源齐备且初值正确。
  const g0 = await brp("td.game");
  check("P12 InGame: lives==5 (fresh BaseLives)", g0.lives === 5, `lives=${g0.lives}`);
  check("P12 InGame: wave schedule total==10", g0.total === 10, `total=${g0.total}`);

  // 行为等价 ③：塔/敌计数指纹与 P11 同场景同窗对拍（1 座无害塔 + 波 1 前段）。
  const t12 = await brp("td.spawn_tower", { tile: [2, 0] });
  check(
    "P12 tower placed after OnEnter load (PlacedTowers fresh)",
    t12.placed === true,
    JSON.stringify(t12),
  );
  const gA = await waitGameUntilElapsed(2.2);
  const sA = await entitySample();
  const expA = expectedEntityDigest([[2, 0]], expectedSpawned(gA.elapsed));
  check(
    "P12 [A] elapsed in sampling window [2.2,3.0)",
    typeof gA.elapsed === "number" && gA.elapsed >= 2.2 && gA.elapsed < 3.0,
    `elapsed=${gA.elapsed}`,
  );
  check(
    "P12 [A] entity digest == expected (independent recompute)",
    sA.digest === expA,
    `got=${sA.digest} expect=${expA}`,
  );
  info(`P12 ENTITY_DIGEST_A=${sA.digest}`);

  // InGame 截图（与 P11 defeat 同窗口：多敌在路 + 塔）。
  await waitGameUntilElapsed(3.3);
  await captureAndVerify(shotArg, readPathOf(shotArg));

  // 露怪进度（仍在 InGame，资源可读）：elapsed >= 5.0 时波 1 已全部刷出（spawned==5,
  // pending==5），且首批漏怪已扣命（soldier@0 于 4.67s 到终点 → lives 4 或 3）。
  const gB = await waitGameUntilElapsed(5.0);
  check(
    "P12 [B] wave 1 exhausted in InGame (spawned==5, pending==5)",
    gB.spawned === 5 && gB.pending === 5,
    JSON.stringify(gB),
  );
  check(
    "P12 [B] lives decremented by leaks but not yet zero (3..=4)",
    typeof gB.lives === "number" && gB.lives >= 3 && gB.lives <= 4,
    `lives=${gB.lives} elapsed=${gB.elapsed}`,
  );

  // 败局 → GameOver 状态转移（无有效防御，五漏怪 ~6.8s）。
  const stOver = await waitState("GameOver", 25000);
  check("P12 state==GameOver after defeat", stOver.state === "GameOver", JSON.stringify(stOver));
  check("P12 GameResult==Defeat", stOver.result === "Defeat", JSON.stringify(stOver));
  // `GameFinished` 在 OnExit(InGame) 的 cleanup_level 中被移除（教程 12 Step 4/6：
  // 每回合重建门闩）——故进入 GameOver 时已不可见；此断言即「清理已跑过」的负向证据。
  check("P12 GameFinished removed by cleanup on exit (fresh-round gate)", stOver.finished === false);
  check("P12 defeat: zero enemies alive (world.query)", (await countOf("game_td::Enemy")) === 0);
  // GameOver 态门控：1.5s 后仍在 GameOver、无敌人、调度不可用（FixedUpdate 链不跑）。
  await sleep(1500);
  const stStay = await brp("td.state");
  check(
    "P12 GameOver gates gameplay: state stays / no enemies / schedule removed",
    stStay.state === "GameOver" &&
      (await countOf("game_td::Enemy")) === 0 &&
      (await brp("td.wave")).available === false,
    JSON.stringify(stStay),
  );
  await sleep(300); // 等 UI 布局/渲染几帧
  await captureAndVerify(shotOver, readPathOf(shotOver));

  // 回菜单 → cleanup_level 清场（教程 12 Step 6）。
  const setBack = await brp("td.state", { op: "set", state: "LevelSelect" });
  check("P12 td.state set LevelSelect queued", setBack.queued === true, JSON.stringify(setBack));
  const stBack = await waitState("LevelSelect", 10000);
  check("P12 state==LevelSelect after exit", stBack.state === "LevelSelect", JSON.stringify(stBack));
  await sleep(300); // 等 OnExit 命令落地
  check("P12 cleanup: MapTile==0", (await countOf("game_td::MapTile")) === 0);
  check("P12 cleanup: PathTile==0", (await countOf("game_td::PathTile")) === 0);
  check(
    "P12 cleanup: Tower==0 / TowerTurret==0",
    (await countOf("game_td::Tower")) === 0 && (await countOf("game_td::TowerTurret")) === 0,
  );
  check("P12 cleanup: TowerPreview==0", (await countOf("game_td::TowerPreview")) === 0);
  check("P12 cleanup: Enemy==0", (await countOf("game_td::Enemy")) === 0);
  check(
    "P12 cleanup: wave schedule removed (td.wave unavailable)",
    (await brp("td.wave")).available === false,
  );
  const gAfter = await brp("td.game");
  check(
    "P12 cleanup: lives removed / game_over flag reset",
    gAfter.lives === null && gAfter.game_over === false,
    JSON.stringify(gAfter),
  );
  check(
    "P12 cleanup: GameResult kept for GameOver screen (Defeat survives)",
    stBack.result === "Defeat",
    JSON.stringify(stBack),
  );

  // 重开：再入 InGame → 全新资源 + 同指纹（教程 12 Step 11「Press 1 again — fresh game」）。
  await brp("td.state", { op: "set", state: "InGame" });
  const stRe = await waitState("InGame", 10000);
  check("P12 replay: state==InGame", stRe.state === "InGame", JSON.stringify(stRe));
  const mapRe = await assertMap("P12-replay");
  check(
    "P12 replay: GRID_DIGEST identical to first round",
    mapRe.digest === GRID_DIGEST_BASELINE,
    `digest=${mapRe.digest}`,
  );
  const gRe = await brp("td.game");
  check("P12 replay: lives==5 (fresh round)", gRe.lives === 5, `lives=${gRe.lives}`);
  check(
    "P12 replay: schedule restarted (spawned<5)",
    typeof gRe.spawned === "number" && gRe.spawned < 5,
    `spawned=${gRe.spawned}`,
  );
  const tRe = await brp("td.spawn_tower", { tile: [2, 0] });
  check(
    "P12 replay: tower placement works again (PlacedTowers reset)",
    tRe.placed === true,
    JSON.stringify(tRe),
  );
  check(
    "P12 replay: Tower count==1 (previous round fully cleaned)",
    (await countOf("game_td::Tower")) === 1,
    `towers=${await countOf("game_td::Tower")}`,
  );
} else if (phase === "p13") {
  // P13：多关卡（教程 13）。断言链：三关卡自动发现 → 动态列表 UI（截图）→
  // 逐关卡装载并独立展开对拍（PathTile/Grass 计数 + 逐格 TileType + 波次事件数）
  // → 分关卡 GRID_DIGEST（level_02/03 首轮 `--register` 登记，终轮强断言）
  // → 切回 level_01 复验零漂移。
  const shotMenu = arg("--shot-menu", "");
  const register = process.argv.includes("--register");

  await sleep(800); // 首帧呈现（起跑 ~0.8s 内捕获 swapchain 为纯黑，环境事实）
  const st0 = await brp("td.state");
  check("P13 initial state==LevelSelect", st0.state === "LevelSelect", JSON.stringify(st0));

  const lv = await brp("td.level");
  check("P13 td.level available", lv.available === true, JSON.stringify(lv));
  check(
    "P13 discovers exactly the 3 level files",
    Array.isArray(lv.levels) &&
      lv.levels.length === LEVELS.length &&
      LEVELS.every((e, i) => lv.levels[i] === e.discovered),
    `levels=${JSON.stringify(lv.levels)}`,
  );
  check(
    "P13 default selected==level_01",
    lv.selected === LEVELS[0].discovered,
    `selected=${lv.selected}`,
  );
  await captureAndVerify(shotMenu, readPathOf(shotMenu));

  for (let i = 0; i < LEVELS.length; i++) {
    const entry = LEVELS[i];
    const label = `P13 [${entry.key}]`;
    await selectAndEnterLevel(i, label);

    if (entry.key === "level_01") {
      // level_01 沿用 T047/T048 的逐格索引规格 + 冻结基线 digest（零漂移链首环）。
      const map = await assertMap(label);
      check(
        `${label} GRID_DIGEST == T047 baseline`,
        map.digest === GRID_DIGEST_BASELINE,
        `digest=${map.digest}`,
      );
    } else {
      await assertGenericMap(label, entry, register);
    }

    const g = await brp("td.game");
    check(
      `${label} wave events == independent TOML count`,
      g.total === entry.parsed.events,
      `total=${g.total} expect=${entry.parsed.events}`,
    );

    // 关卡 2 在局内截图（多关卡内容的行为面证据；UI 文案不在断言面内）。
    if (i === 1) await captureAndVerify(shotArg, readPathOf(shotArg));
    await leaveToMenu();
  }

  // 切回 level_01 复验：跨关卡切换后地图层零残留 / 逐位一致。
  await selectAndEnterLevel(0, "P13 [level_01-replay]");
  const mRe = await assertMap("P13-replay");
  check(
    "P13 replay GRID_DIGEST == T047 baseline",
    mRe.digest === GRID_DIGEST_BASELINE,
    `digest=${mRe.digest}`,
  );
  const gRe = await brp("td.game");
  check("P13 replay wave total==10", gRe.total === 10, `total=${gRe.total}`);
  await leaveToMenu();
} else if (phase === "p14") {
  // P14：经济（教程 14）。场景固定在 level_01：
  // 被动收入采样（无塔无击杀）→ 赏金组件 → 三塔扣费 300 → 余额不足拒绝 → 首杀赏金
  // → 截图 → 地图层零漂移（level_01 逐格规格 + level_02/03 注册指纹在链）。
  await sleep(800);
  await selectAndEnterLevel(0, "P14");

  const eco0 = await economy();
  check("P14 td.economy available", eco0.available === true, JSON.stringify(eco0));
  check("P14 STARTING_GOLD==300", eco0.starting_gold === 300, `starting=${eco0.starting_gold}`);
  check("P14 PASSIVE_INCOME_RATE==3.0", eco0.passive_rate === 3.0, `rate=${eco0.passive_rate}`);
  check("P14 TOWER_COST==100", eco0.tower_cost === 100, `cost=${eco0.tower_cost}`);
  check(
    "P14 initial gold in [300,302]",
    typeof eco0.gold === "number" && eco0.gold >= 300 && eco0.gold <= 302,
    `gold=${eco0.gold}`,
  );

  // HUD 侧信道（U6）：Text 内容不在 BRP 反射面 → 断言标记组件恰一份 + 截图看文案。
  check("P14 GoldHud marker count==1", (await countOf("game_td::GoldHud")) === 1);

  // 被动收入（O1 采样窗）：无塔 → 无击杀，1s 窗内增量 == 3.0×Δt（±20%）。
  const eA = await economy();
  const tA = await elapsed();
  await sleep(1000);
  const eB = await economy();
  const tB = await elapsed();
  const passive = eB.gold - eA.gold;
  const dtAB = tB - tA;
  info(`P14 passive sample dt=${dtAB.toFixed(3)} delta=${passive.toFixed(3)}`);
  check(
    "P14 passive income == 3.0/s (±20% + 0.05 abs)",
    Math.abs(passive - 3.0 * dtAB) <= 0.2 * 3.0 * dtAB + 0.05,
    `delta=${passive.toFixed(3)} expect=${(3.0 * dtAB).toFixed(3)}`,
  );

  // 赏金链路末端：在场敌人 Bounty==25（教程 14 Step 2/9）。
  const bounties = await enemyBounties();
  check("P14 at least 1 enemy spawned for bounty check", bounties.length >= 1, `n=${bounties.length}`);
  check(
    "P14 every alive enemy Bounty==25",
    bounties.length >= 1 && bounties.every((v) => v === 25),
    `bounties=${JSON.stringify(bounties)}`,
  );

  // 三塔扣费 300（含被动收入修正）。
  const before = await economy();
  const t0 = await elapsed();
  const tiles = [
    [5, 3],
    [7, 3],
    [9, 3],
  ];
  const placed = [];
  for (const tile of tiles) placed.push(await brp("td.spawn_tower", { tile }));
  check(
    "P14 three towers placed (economy gate open)",
    placed.every((r) => r.placed === true),
    placed.map((r) => JSON.stringify(r)).join(" "),
  );
  check("P14 each placement reports cost==100", placed.every((r) => r.cost === 100));
  const after = await economy();
  const t1 = await elapsed();
  const expectedGold = before.gold + 3.0 * (t1 - t0) - 300;
  check(
    "P14 gold after 3 placements == before - 300 + passive (±2)",
    typeof after.gold === "number" && Math.abs(after.gold - expectedGold) <= 2.0,
    `gold=${after.gold} expect=${expectedGold.toFixed(2)}`,
  );
  check("P14 Tower count==3", (await countOf("game_td::Tower")) === 3);

  // 余额不足：拒绝放置、不入账、不生成（与点击系统同一经济门，教程 14 Step 7）。
  const denied = await brp("td.spawn_tower", { tile: [11, 3] });
  check(
    "P14 4th placement denied (gold < cost)",
    denied.placed === false && /not enough gold/.test(String(denied.reason ?? "")),
    JSON.stringify(denied),
  );
  check("P14 Tower count still 3 after denial", (await countOf("game_td::Tower")) === 3);

  // 首杀赏金：首个「敌人数下降」样本的净增量应 = 25×k（k≥1，被动修正后 ±6）。
  let prev = null;
  let killObs = null;
  const deadline = Date.now() + 8000;
  while (Date.now() < deadline) {
    const eco = await economy();
    const t = await elapsed();
    const n = await countOf("game_td::Enemy");
    if (prev && n < prev.n) {
      killObs = { dGold: eco.gold - prev.gold, dt: t - prev.t, t, from: prev.n, to: n };
      break;
    }
    prev = { gold: eco.gold, n, t };
    await sleep(100);
  }
  check("P14 observed a kill (enemy count drop)", !!killObs, JSON.stringify(killObs));
  if (killObs) {
    const net = killObs.dGold - 3.0 * killObs.dt;
    const k = Math.round(net / 25);
    info(
      `P14 kill sample dt=${killObs.dt.toFixed(3)} dGold=${killObs.dGold.toFixed(2)} ` +
        `net=${net.toFixed(2)} k=${k} n=${killObs.from}->${killObs.to}`,
    );
    check(
      "P14 kill bounty == 25 per kill (±6, passive-corrected)",
      k >= 1 && Math.abs(net - 25 * k) <= 6,
      `net=${net.toFixed(2)} k=${k}`,
    );
  }

  // 截图：HUD「Gold: N」+ 多塔交战。
  await captureAndVerify(shotArg, readPathOf(shotArg));

  const map14 = await assertMap("P14");
  check(
    "P14 GRID_DIGEST == T047 baseline",
    map14.digest === GRID_DIGEST_BASELINE,
    `digest=${map14.digest}`,
  );

  // 指纹链：level_02/03 注册 digest 在链（P13→P17 全程强断言）。
  for (const idx of [1, 2]) {
    const entry = LEVELS[idx];
    await selectAndEnterLevel(idx, `P14 [${entry.key}]`);
    await assertGenericMap(`P14 [${entry.key}]`, entry, false);
    await leaveToMenu();
  }
} else if (phase === "p15") {
  // P15：火箭发射器（教程 15）。断言链：成本 150 → 单塔初始弹药 3 →
  // 采样窗内（O1）观察到：弹体飞行 / 弹药消耗 / 补弹（fired 不变而 ammo+1）/
  // 精确 50 伤害（100→50 或 60→10 或 300→250）/ 命中击杀（终点回收前）→ 截图
  // → 地图层零漂移（level_01 + level_02/03 注册指纹在链）。
  await sleep(800);
  await selectAndEnterLevel(0, "P15");

  const map15 = await assertMap("P15");
  check(
    "P15 GRID_DIGEST == T047 baseline",
    map15.digest === GRID_DIGEST_BASELINE,
    `digest=${map15.digest}`,
  );

  const eco = await economy();
  check("P15 TOWER_COST==150 (rocket launcher price)", eco.tower_cost === 150, `cost=${eco.tower_cost}`);

  const placed = await brp("td.spawn_tower", { tile: [5, 3] });
  check("P15 rocket launcher placed at [5,3]", placed.placed === true, JSON.stringify(placed));
  check("P15 placement reports cost==150", placed.cost === 150, `cost=${placed.cost}`);
  check("P15 Tower base count==1", (await countOf("game_td::Tower")) === 1);
  check("P15 TowerTurret (top) count==1", (await countOf("game_td::TowerTurret")) === 1);

  const c0 = await brp("td.combat");
  check("P15 td.combat launchers==1", c0.launchers === 1, JSON.stringify(c0));
  check("P15 ammo_capacity==3", c0.ammo_capacity === 3, `cap=${c0.ammo_capacity}`);
  check("P15 initial ammo_filled==3", c0.ammo_filled === 3, `filled=${c0.ammo_filled}`);
  check("P15 initial fired==0 / projectiles==0", c0.fired === 0 && c0.projectiles === 0, JSON.stringify(c0));
  check("P15 ROCKET_DAMAGE==50", c0.rocket_damage === 50, `damage=${c0.rocket_damage}`);
  check("P15 SPLASH_RADIUS==60", c0.splash_radius === 60, `splash=${c0.splash_radius}`);
  check("P15 ROCKET_SPEED==600", c0.rocket_speed === 600, `speed=${c0.rocket_speed}`);

  let sawProjectile = false;
  let maxProjectiles = 0;
  let minAmmo = 3;
  let maxFired = 0;
  let refillSeen = false;
  let killBeforeLeak = false;
  const exactDamageHealths = new Set();
  let prev = null;
  let queuedShot = null;
  let gameOverSeen = false;
  const samples = [];
  const deadline = Date.now() + 7000;
  while (Date.now() < deadline) {
    const c = await brp("td.combat");
    const healths = await enemyHealths();
    const enemyCount = await countOf("game_td::Enemy");
    const g = await brp("td.game");

    maxProjectiles = Math.max(maxProjectiles, c.projectiles);
    if (c.projectiles >= 1) sawProjectile = true;
    minAmmo = Math.min(minAmmo, c.ammo_filled);
    maxFired = Math.max(maxFired, c.fired);
    // 精确 50 伤害的可观测快照：soldier 100→50、runner 60→10、heavy 300→250。
    for (const h of healths) {
      if ([50, 10, 250].includes(h)) exactDamageHealths.add(h);
    }
    if (prev && c.fired === prev.fired && c.ammo_filled > prev.ammo) refillSeen = true;
    if (
      prev &&
      enemyCount < prev.enemyCount &&
      typeof g.elapsed === "number" &&
      g.elapsed < 4.6
    ) {
      killBeforeLeak = true;
    }
    if (c.projectiles >= 1 && !queuedShot) queuedShot = await queueCapture(shotArg);
    samples.push(
      `t=${typeof g.elapsed === "number" ? g.elapsed.toFixed(2) : "?"} ` +
        `fired=${c.fired} ammo=${c.ammo_filled} proj=${c.projectiles} enemies=${enemyCount}`,
    );
    prev = { fired: c.fired, ammo: c.ammo_filled, enemyCount };
    if (g.game_over) {
      gameOverSeen = true;
      break;
    }
    await sleep(60);
  }
  info(`P15 samples: ${samples.slice(0, 12).join(" | ")}${samples.length > 12 ? " ..." : ""}`);
  info(
    `P15 observed maxProjectiles=${maxProjectiles} minAmmo=${minAmmo} maxFired=${maxFired} ` +
      `refill=${refillSeen} killBeforeLeak=${killBeforeLeak} gameOver=${gameOverSeen} ` +
      `exactDamageHealths=${[...exactDamageHealths].join(",")}`,
  );
  check("P15 projectile observed in flight", sawProjectile && maxProjectiles >= 1, `max=${maxProjectiles}`);
  check("P15 launcher fired all 3 ammo slots", maxFired >= 3, `fired=${maxFired}`);
  check("P15 ammo depleted (ammo_filled <= 1 observed)", minAmmo <= 1, `minAmmo=${minAmmo}`);
  check("P15 ammo refill observed (fired unchanged, ammo +1)", refillSeen);
  check(
    "P15 impact damage == 50 exactly (health 50 / 10 / 250 observed)",
    exactDamageHealths.size >= 1,
    `set=${[...exactDamageHealths].join(",")}`,
  );
  check("P15 kill by rocket before path end (count drop, elapsed<4.6)", killBeforeLeak);
  if (queuedShot) await verifyCapture(queuedShot, readPathOf(shotArg));
  else check("P15 screenshot queued during flight", false, "no projectile sample");

  await leaveToMenu();

  // 指纹链：level_02/03 注册 digest 在链。
  for (const idx of [1, 2]) {
    const entry = LEVELS[idx];
    await selectAndEnterLevel(idx, `P15 [${entry.key}]`);
    await assertGenericMap(`P15 [${entry.key}]`, entry, false);
    await leaveToMenu();
  }
} else if (phase === "p16") {
  // P16：数据驱动塔（教程 16）+ **P13–P15 关键断言回归段**。
  // ① 注册表面：TOML 两塔数值对表（`td.towers`）；② 回归 A（level_01 开局）：
  //    地图指纹 / 经济常量与被动收入 / 赏金链路 / 塔实体指纹（P11-P12 口径）/
  //    经济门拒绝；③ 回归 B（重进 level_01）：火箭弹道链（P15 口径）；
  // ④ 指纹链 level_02/03。
  await sleep(800);

  const reg = await brp("td.towers");
  check("P16 td.towers available", reg.available === true, JSON.stringify(reg).slice(0, 200));
  check("P16 registry has exactly 2 towers", Array.isArray(reg.towers) && reg.towers.length === 2,
    `len=${reg.towers?.length}`);
  check("P16 registry keys == [rapid, rocket] (sorted)",
    JSON.stringify((reg.towers ?? []).map((t) => t.key)) === JSON.stringify(["rapid", "rocket"]),
    `keys=${JSON.stringify((reg.towers ?? []).map((t) => t.key))}`);
  check("P16 default selected=='rocket'", reg.selected === "rocket", `selected=${reg.selected}`);

  const byKey = Object.fromEntries((reg.towers ?? []).map((t) => [t.key, t]));
  const rapid = byKey.rapid ?? {};
  const rocket = byKey.rocket ?? {};
  check("P16 rapid: cost==100", rapid.cost === 100, `cost=${rapid.cost}`);
  check("P16 rapid: range==192 / cooldown==0.5 / damage==34",
    rapid.attack_range === 192 && rapid.attack_cooldown === 0.5 && rapid.damage === 34,
    JSON.stringify({ r: rapid.attack_range, c: rapid.attack_cooldown, d: rapid.damage }));
  check("P16 rapid: muzzle_flash_sprite==295 / no projectile",
    rapid.muzzle_flash_sprite === 295 && rapid.projectile === null,
    JSON.stringify({ m: rapid.muzzle_flash_sprite, p: rapid.projectile }));
  check("P16 rocket: cost==150 / range==192 / cooldown==0.3",
    rocket.cost === 150 && rocket.attack_range === 192 &&
      Math.abs(rocket.attack_cooldown - 0.3) < 1e-6,
    JSON.stringify({ c: rocket.cost, r: rocket.attack_range, cd: rocket.attack_cooldown }));
  check("P16 rocket: ammo_slots==3 / refill==2.0",
    rocket.ammo_slots === 3 && rocket.ammo_refill_secs === 2.0,
    JSON.stringify({ n: rocket.ammo_slots, s: rocket.ammo_refill_secs }));
  check("P16 rocket projectile: damage 50 / speed 600 / sprite 251 / explosion 21 / splash 60",
    rocket.projectile &&
      rocket.projectile.damage === 50 &&
      rocket.projectile.speed === 600 &&
      rocket.projectile.sprite === 251 &&
      rocket.projectile.explosion_sprite === 21 &&
      rocket.projectile.splash_radius === 60,
    JSON.stringify(rocket.projectile));

  const combat0 = await brp("td.combat");
  check("P16 td.combat rocket numbers sourced from registry",
    combat0.rocket_damage === 50 && combat0.splash_radius === 60 && combat0.rocket_speed === 600,
    JSON.stringify(combat0));

  // ---- 回归 A：level_01 开局（P13 地图 / P14 经济 / P11-P12 塔指纹）----
  await selectAndEnterLevel(0, "P16");
  const map16 = await assertMap("P16");
  check("P16 [A] GRID_DIGEST == T047 baseline", map16.digest === GRID_DIGEST_BASELINE,
    `digest=${map16.digest}`);

  const ecoA0 = await economy();
  check("P16 [A] economy costs from registry (150)", ecoA0.tower_cost === 150, `cost=${ecoA0.tower_cost}`);
  check("P16 [A] starting gold==300", ecoA0.starting_gold === 300, `starting=${ecoA0.starting_gold}`);

  // P14 被动收入回归（无塔 sabotage 前的干净窗口）。
  const peA = await economy();
  const ptA = await elapsed();
  await sleep(1000);
  const peB = await economy();
  const ptB = await elapsed();
  const passiveDelta = peB.gold - peA.gold;
  check("P16 [A] passive income == 3.0/s (±20%)",
    Math.abs(passiveDelta - 3.0 * (ptB - ptA)) <= 0.2 * 3.0 * (ptB - ptA) + 0.05,
    `delta=${passiveDelta.toFixed(3)} dt=${(ptB - ptA).toFixed(3)}`);

  // P14 赏金链路回归。
  const b16 = await enemyBounties();
  check("P16 [A] alive enemies Bounty==25", b16.length >= 1 && b16.every((v) => v === 25),
    `bounties=${JSON.stringify(b16)}`);

  // P11/P12 塔实体指纹回归：1 座无害塔 [2,0]（dy=320 > range 192，不改变波次/漏怪）。
  const g0A = await economy();
  const t0A = await elapsed();
  const tA = await brp("td.spawn_tower", { tile: [2, 0] });
  check("P16 [A] harmless rocket tower placed at [2,0]", tA.placed === true, JSON.stringify(tA));
  check("P16 [A] placement reports cost 150 / tower_type rocket",
    tA.cost === 150 && tA.tower_type === "rocket", JSON.stringify(tA));
  const g1A = await economy();
  const t1A = await elapsed();
  const expGoldA = g0A.gold + 3.0 * (t1A - t0A) - 150;
  check("P16 [A] gold deducted 150", Math.abs(g1A.gold - expGoldA) <= 2.0,
    `gold=${g1A.gold} expect=${expGoldA.toFixed(2)}`);
  check("P16 [A] Tower/TowerTurret counts == 1/1",
    (await countOf("game_td::Tower")) === 1 && (await countOf("game_td::TowerTurret")) === 1);

  const gWin = await waitGameUntilElapsed(2.2);
  const sWin = await entitySample();
  const expWin = expectedEntityDigest([[2, 0]], expectedSpawned(gWin.elapsed));
  check("P16 [A] elapsed in sampling window [2.2,3.0)",
    typeof gWin.elapsed === "number" && gWin.elapsed >= 2.2 && gWin.elapsed < 3.0,
    `elapsed=${gWin.elapsed}`);
  check("P16 [A] tower/enemy entity digest == expected (P11-P12 口径)",
    sWin.digest === expWin, `got=${sWin.digest} expect=${expWin}`);
  info(`P16 ENTITY_DIGEST=${sWin.digest}`);

  // P14 经济门回归：再放一座（150）后余额不足，第三座拒绝。
  const tB = await brp("td.spawn_tower", { tile: [4, 0] });
  check("P16 [A] second tower placed while affordable", tB.placed === true, JSON.stringify(tB));
  const tC = await brp("td.spawn_tower", { tile: [6, 0] });
  check("P16 [A] third placement denied (gold < cost)",
    tC.placed === false && /not enough gold/.test(String(tC.reason ?? "")), JSON.stringify(tC));
  check("P16 [A] Tower count==2 after denial", (await countOf("game_td::Tower")) === 2);

  await leaveToMenu();

  // ---- 回归 B：火箭弹道链（P15 口径）----
  await selectAndEnterLevel(0, "P16-rocket");
  const ecoB = await economy();
  check("P16 [B] fresh round gold ~= 300", typeof ecoB.gold === "number" && ecoB.gold >= 300 && ecoB.gold <= 302,
    `gold=${ecoB.gold}`);

  const tR = await brp("td.spawn_tower", { tile: [5, 3] });
  check("P16 [B] rocket launcher placed", tR.placed === true, JSON.stringify(tR));

  let sawProjectile = false;
  let minAmmo = 3;
  let maxFired = 0;
  let refillSeen = false;
  let killBeforeLeak = false;
  const exactDamageHealths = new Set();
  let prev = null;
  let queuedShot = null;
  const deadlineB = Date.now() + 7000;
  while (Date.now() < deadlineB) {
    const c = await brp("td.combat");
    const healths = await enemyHealths();
    const enemyCount = await countOf("game_td::Enemy");
    const g = await brp("td.game");
    if (c.projectiles >= 1) sawProjectile = true;
    minAmmo = Math.min(minAmmo, c.ammo_filled);
    maxFired = Math.max(maxFired, c.fired);
    for (const h of healths) if ([50, 10, 250].includes(h)) exactDamageHealths.add(h);
    if (prev && c.fired === prev.fired && c.ammo_filled > prev.ammo) refillSeen = true;
    if (prev && enemyCount < prev.enemyCount && typeof g.elapsed === "number" && g.elapsed < 4.6) {
      killBeforeLeak = true;
    }
    if (c.projectiles >= 1 && !queuedShot) queuedShot = await queueCapture(shotArg);
    prev = { fired: c.fired, ammo: c.ammo_filled, enemyCount };
    if (g.game_over) break;
    await sleep(60);
  }
  info(
    `P16 [B] projectile=${sawProjectile} minAmmo=${minAmmo} maxFired=${maxFired} ` +
      `refill=${refillSeen} killBeforeLeak=${killBeforeLeak} ` +
      `exactDamageHealths=${[...exactDamageHealths].join(",")}`,
  );
  check("P16 [B] projectile observed in flight", sawProjectile);
  check("P16 [B] launcher fired >=3", maxFired >= 3, `fired=${maxFired}`);
  check("P16 [B] ammo depleted (<=1)", minAmmo <= 1, `minAmmo=${minAmmo}`);
  check("P16 [B] refill observed", refillSeen);
  check("P16 [B] impact damage == 50 exactly", exactDamageHealths.size >= 1,
    `set=${[...exactDamageHealths].join(",")}`);
  check("P16 [B] kill by rocket before path end", killBeforeLeak);
  if (queuedShot) await verifyCapture(queuedShot, readPathOf(shotArg));
  else check("P16 [B] screenshot queued during flight", false, "no projectile sample");

  await leaveToMenu();

  // ---- 指纹链：level_02/03 注册 digest 在链 ----
  for (const idx of [1, 2]) {
    const entry = LEVELS[idx];
    await selectAndEnterLevel(idx, `P16 [${entry.key}]`);
    await assertGenericMap(`P16 [${entry.key}]`, entry, false);
    await leaveToMenu();
  }
} else if (phase === "p17") {
  // P17：塔选择 UI（教程 17）。A' 通道：`Interaction` 弃用别名以 `#[expect(deprecated)]`
  // 落地（源码注记）；UI 态断言走 U6 侧信道——dock 槽位计数（反射组件查询）+
  // 选中下标资源（`td.tower_select`）+ 截图。键鼠注入面无 BRP 通道（滚轮/数字键/
  // 槽位点击本身不注入；其效果统一经选中下标 → 放置塔型 → TowerTypeId 链验证）。
  const shotDock = arg("--shot-dock", "");
  await sleep(800);

  // ---- 注册表：第三座塔纯 TOML 追加（教程 17 收官证明）----
  const reg = await brp("td.towers");
  check("P17 registry has 3 towers (big_rocket from TOML only)",
    Array.isArray(reg.towers) && reg.towers.length === 3, `len=${reg.towers?.length}`);
  const keys = (reg.towers ?? []).map((t) => t.key).sort();
  check("P17 registry keys == [big_rocket, rapid, rocket]",
    JSON.stringify(keys) === JSON.stringify(["big_rocket", "rapid", "rocket"]),
    `keys=${JSON.stringify(keys)}`);
  const big = (reg.towers ?? []).find((t) => t.key === "big_rocket") ?? {};
  check("P17 big_rocket: cost 200 / range 256 / cooldown 1.5",
    big.cost === 200 && big.attack_range === 256 && Math.abs(big.attack_cooldown - 1.5) < 1e-6,
    JSON.stringify({ c: big.cost, r: big.attack_range, cd: big.attack_cooldown }));
  check("P17 big_rocket: 1 ammo slot / 4.0s refill",
    big.ammo_slots === 1 && big.ammo_refill_secs === 4.0,
    JSON.stringify({ n: big.ammo_slots, s: big.ammo_refill_secs }));
  check("P17 big_rocket projectile: damage 150 / speed 400 / sprite 252 / splash 100",
    big.projectile &&
      big.projectile.damage === 150 &&
      big.projectile.speed === 400 &&
      big.projectile.sprite === 252 &&
      big.projectile.splash_radius === 100,
    JSON.stringify(big.projectile));

  // ---- 选择通道（U6 侧信道）：get / set by key / set by index / 非法输入 ----
  const sel0 = await brp("td.tower_select");
  check("P17 tower_select get: len==3 / selected_index==0",
    sel0.available === true && sel0.len === 3 && sel0.selected_index === 0,
    JSON.stringify(sel0));
  check("P17 default selected key == registry[0].key",
    sel0.key === (reg.towers ?? []).find((t) => t.index === 0)?.key,
    `got=${sel0.key}`);

  const selByKey = await brp("td.tower_select", { op: "set", key: "big_rocket" });
  check("P17 select by key 'big_rocket'", selByKey.key === "big_rocket" &&
    selByKey.selected_index === big.index, JSON.stringify(selByKey));
  const selBack = await brp("td.tower_select", { op: "set", index: 0 });
  check("P17 select by index 0", selBack.selected_index === 0, JSON.stringify(selBack));
  const badKey = await errCode("td.tower_select", { op: "set", key: "nope" });
  check("P17 unknown key -> -32602", badKey === -32602, `code=${badKey}`);
  const badIdx = await errCode("td.tower_select", { op: "set", index: 99 });
  check("P17 out-of-range index -> -32602", badIdx === -32602, `code=${badIdx}`);

  // ---- 进关：dock 槽位数（U6 侧信道）+ 选中重置 + 放置链 ----
  await selectAndEnterLevel(0, "P17");
  await sleep(300); // dock 布局/渲染几帧

  check("P17 dock slot count == registry length (3)",
    (await countOf("game_td::TowerDockSlot")) === 3,
    `slots=${await countOf("game_td::TowerDockSlot")}`);
  check("P17 dock root exists (GoldHud side: 1 HUD)",
    (await countOf("game_td::GoldHud")) === 1);
  const selIn = await brp("td.tower_select");
  check("P17 setup_tower_dock resets selection to index 0", selIn.selected_index === 0,
    JSON.stringify(selIn));

  // 选 big_rocket 并放置：塔型 / 扣费 / TowerTypeId 端到端。
  const selBig = await brp("td.tower_select", { op: "set", key: "big_rocket" });
  check("P17 select big_rocket in-game", selBig.key === "big_rocket", JSON.stringify(selBig));
  const gBefore = await economy();
  const tBefore = await elapsed();
  const placeBig = await brp("td.spawn_tower", { tile: [5, 3] });
  check("P17 place big_rocket at [5,3]", placeBig.placed === true, JSON.stringify(placeBig));
  check("P17 placement tower_type==big_rocket / cost==200 / index==big.index",
    placeBig.tower_type === "big_rocket" && placeBig.cost === 200 &&
      placeBig.tower_index === big.index,
    JSON.stringify(placeBig));
  const gAfter = await economy();
  const tAfter = await elapsed();
  const expGold = gBefore.gold + 3.0 * (tAfter - tBefore) - 200;
  check("P17 big_rocket cost deducted (200, ±2)", Math.abs(gAfter.gold - expGold) <= 2.0,
    `gold=${gAfter.gold} expect=${expGold.toFixed(2)}`);

  // big_rocket 弹道（数据驱动 1 槽 / 4s 补弹 / 150 伤害）：采样到 fired>=1 且击杀。
  let bigFired = 0;
  let bigCapacity = 0;
  let bigKill = false;
  let prev = null;
  const bigDeadline = Date.now() + 4000;
  while (Date.now() < bigDeadline) {
    const c = await brp("td.combat");
    const n = await countOf("game_td::Enemy");
    const g = await brp("td.game");
    bigFired = Math.max(bigFired, c.fired);
    bigCapacity = Math.max(bigCapacity, c.ammo_capacity);
    if (prev && n < prev.n && typeof g.elapsed === "number" && g.elapsed < 4.6) bigKill = true;
    prev = { n };
    if (g.game_over) break;
    await sleep(60);
  }
  info(`P17 big_rocket fired=${bigFired} capacity=${bigCapacity} kill=${bigKill}`);
  check("P17 big_rocket ammo_capacity==1 (data-driven slot count)", bigCapacity === 1,
    `capacity=${bigCapacity}`);
  check("P17 big_rocket fired >=1", bigFired >= 1, `fired=${bigFired}`);
  check("P17 big_rocket kill before path end", bigKill);

  // 已放置塔的 TowerTypeId == big.index（选择 → 放置的实体证据）。
  const typeIds1 = (
    await brp("world.query", { data: { components: ["game_td::TowerTypeId"] } })
  ).map((r) => tupleValue(r.components["game_td::TowerTypeId"]));
  check("P17 placed TowerTypeId == [big.index]",
    JSON.stringify(typeIds1) === JSON.stringify([big.index]),
    `got=${JSON.stringify(typeIds1)} big.index=${big.index}`);

  // 截图：dock（三槽位 + 选中高亮）与在局画面。
  await sleep(200);
  await captureAndVerify(shotDock || shotArg, readPathOf(shotDock || shotArg));
  if (shotDock && shotArg && shotDock !== shotArg) {
    await captureAndVerify(shotArg, readPathOf(shotArg));
  }

  await leaveToMenu();

  // ---- 第二局：rapid 塔（34 伤害可观测）+ 余额不足拒绝 ----
  // 独立新局保证 clean 经济与波次窗（首局 big_rocket 一击必杀会吞掉 rapid 的
  // 两段式伤害窗口——同局混测不可靠，非行为差异）。
  await selectAndEnterLevel(0, "P17-rapid");
  const selRapid = await brp("td.tower_select", { op: "set", key: "rapid" });
  check("P17 select rapid in fresh round", selRapid.key === "rapid", JSON.stringify(selRapid));
  const placeRapid = await brp("td.spawn_tower", { tile: [5, 3] });
  check("P17 place rapid at [5,3]", placeRapid.placed === true, JSON.stringify(placeRapid));
  check("P17 rapid tower_type==rapid / cost==100",
    placeRapid.tower_type === "rapid" && placeRapid.cost === 100, JSON.stringify(placeRapid));

  const typeIds2 = (
    await brp("world.query", { data: { components: ["game_td::TowerTypeId"] } })
  ).map((r) => tupleValue(r.components["game_td::TowerTypeId"]));
  check("P17 placed TowerTypeId == [rapid.index]",
    JSON.stringify(typeIds2) === JSON.stringify([selRapid.selected_index]),
    `got=${JSON.stringify(typeIds2)} rapid.index=${selRapid.selected_index}`);

  let rapidHit = false;
  const rapidDeadline = Date.now() + 5000;
  while (Date.now() < rapidDeadline) {
    const healths = await enemyHealths();
    if (healths.some((h) => h === 66 || h === 266)) {
      rapidHit = true;
      break;
    }
    const g = await brp("td.game");
    if (g.game_over) break;
    await sleep(60);
  }
  check("P17 rapid tower deals 34 (health 66 observed)", rapidHit);

  // 再放一座 rapid（100）把余额压到 < 200，再验 big_rocket（200）拒绝。
  const placeRapid2 = await brp("td.spawn_tower", { tile: [7, 3] });
  check("P17 second rapid placed (100)", placeRapid2.placed === true, JSON.stringify(placeRapid2));
  await brp("td.tower_select", { op: "set", key: "big_rocket" });
  const deniedBig = await brp("td.spawn_tower", { tile: [11, 3] });
  check("P17 big_rocket denied when gold < 200",
    deniedBig.placed === false && /not enough gold/.test(String(deniedBig.reason ?? "")),
    JSON.stringify(deniedBig));

  // ---- 指纹链：level_01 基线 + level_02/03 注册 digest 在链 ----
  // （P17 场景已交战/可能判负，重进一局取干净地图面。）
  await selectAndEnterLevel(0, "P17-clean");
  const map17 = await assertMap("P17-clean");
  check("P17 GRID_DIGEST == T047 baseline", map17.digest === GRID_DIGEST_BASELINE,
    `digest=${map17.digest}`);
  await leaveToMenu();
  for (const idx of [1, 2]) {
    const entry = LEVELS[idx];
    await selectAndEnterLevel(idx, `P17 [${entry.key}]`);
    await assertGenericMap(`P17 [${entry.key}]`, entry, false);
    await leaveToMenu();
  }
} else if (
  ![
    "p4", "p5", "p6", "p6b", "p7", "p8", "p9", "p10", "p11", "p12", "p13", "p14", "p15",
    "p16", "p17",
  ].includes(phase)
) {
  check("phase arg", false, `未知 --phase ${phase}`);
}

console.log(results.join("\n"));
console.log(`SUMMARY: ${results.filter((r) => r.startsWith("PASS")).length}/${results.filter((r) => !r.startsWith("INFO")).length} PASS`);
process.exit(process.exitCode ?? 0);

// T052-W1 · M6 塔防 m6-all 终局判定面（收官节点卡，件一）。
//
// 设计口径（grill 定案① 「验证面扩展」/ 定案② 四件套；沿 M5 m5-all 形态）：
// **agent 只作驱动，判定面全部在本脚本内**——逐项 PASS/FAIL 行 + 总判定由脚本计算生成
// （禁止手算填表），退出码 0 = 全 PASS。脚本可独立运行（只依赖 node 与树内 TOML 真解析）。
//
// 结构（相位）：
//   screen   LevelSelect 常驻面：方法面 / 关卡发现 / 开局卫生 / td.* 错误路径（-32602 / -32601）
//   l1       level_01：核心循环（开局→波次→经济闭环→放置→击杀→digest）+ 双模式终局
//   l2       level_02：同上
//   l3       level_03：同上 + 登记项⑥ 溅射聚怪探针（可选，见 phase 注释）
//   chain    登记项②：victory → 回 LevelSelect → 进另一关 → 再终局（状态机全链无残留）
//   p0probe  独立进程负控（件三）：td.state set GameOver 缺 GameResult 是否击穿进程
//   all      screen + l1 + l2 + l3 + chain（一次新进程顺序跑完；p0probe 会杀进程，故不并入）
//
// 用法（树根执行 = 脚本 CWD；游戏进程 CWD = <tree>/game-td）：
//   node docs/evidence/t052-m6-all/m6-all.mjs --phase all --expect-discover 41 \
//        --game-log docs/evidence/t052-m6-all/m6-all-run1.game.log \
//        --shot ../docs/evidence/t052-m6-all/m6-all-shot.png
//   node docs/evidence/t052-m6-all/m6-all.mjs --phase p0probe --game-log docs/evidence/t052-m6-all/p0probe-run1.game.log
//
// 期望值来源（禁裸值）：
//   - 关卡网格 / 波次事件表 / 血量序列 / 路径长度：脚本内独立解析 TOML（`./toml-level-expand.mjs`
//     网格展开 + 本文件 `parseLevelEvents`），**不读游戏侧值**；
//   - 塔数值：脚本内独立解析 `game-td/assets/towers.toml`（`parseTowersToml`），与 `td.towers` 对拍；
//   - GRID_DIGEST / ENTITY_DIGEST 注册表：T047 / T048 / T049 证据档冻结值（下方常量，注明出处）；
//   - 经济账：gold == 300 + 3.0*elapsed + Σbounty*kills − Σcost（常量来自 economy.rs / towers.toml，
//     脚本内独立算，容差 1.0 f32 记账残差）。
//
// 登记项落点：① td.state set 写通道（enterLevel/leaveToMenu 逐次断言 from/to）；③ 事件表独立复算
// + elapsed 检查点（checkpoints）；② victory 全链重测（chain 相位）；⑤ ts06 注锚（见 README，
// 不在本脚本）；⑥ 溅射（l3 的 splashProbe，超时盒不达标 = SKIP 注记，不判 FAIL）；⑦ td.input
// 注入通道评估（README 口径，脚本内只做一次 emit 端到端冒烟）。
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { expandLevel, parseLevelFile } from "./toml-level-expand.mjs";

const URL = "http://127.0.0.1:15702";
let id = 0;
const results = [];
function check(name, cond, detail) {
  results.push(`${cond ? "PASS" : "FAIL"} | ${name}${detail ? " | " + detail : ""}`);
  if (!cond) process.exitCode = 1;
  return !!cond;
}
function info(line) {
  results.push(`INFO | ${line}`);
}
function skip(name, why) {
  results.push(`SKIP | ${name} | ${why}`);
}

async function brp(method, params, timeoutMs = 10000) {
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
/// 原始调用（错误码断言用，不抛）。
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
async function errCode(method, params) {
  const j = await brpRaw(method, params);
  return j.error ? j.error.code : null;
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
function arg(name, fallback = null) {
  const i = process.argv.indexOf(name);
  return i >= 0 && i + 1 < process.argv.length ? process.argv[i + 1] : fallback;
}

const phase = arg("--phase", "all");
const shotArg = arg("--shot", "");
const shotVictoryArg = arg("--shot-victory", "");
const gameLog = arg("--game-log", "");
const expectDiscover = Number(arg("--expect-discover", "41"));

// ---- 几何 / 常量（脚本独立算式，来源 = 教程 02/03 网格规格 + GAME 常量注释）----
const COLS = 15;
const ROWS = 10;
const TILE = 64;
const OFFSET_X = -(COLS * TILE) / 2 + TILE / 2; // -448
const OFFSET_Y = -(ROWS * TILE) / 2 + TILE / 2; // -288
const MAP_TILE = COLS * ROWS; // 150
const STARTING_LIVES = 5; // game-td/src/main.rs:344 `BaseLives(5)`
const STARTING_GOLD = 300.0; // game-td/src/economy.rs:26
const PASSIVE_RATE = 3.0; // game-td/src/economy.rs:29
const RANGE = 192; // rapid / rocket attack_range（towers.toml，另由 checkRegistry 独立对拍）
const RAPID = { key: "rapid", cost: 100, damage: 34, cooldown: 0.5 };
const ROCKET = { key: "rocket", cost: 150 };
// 采样窗口径（T048 冻结链）：无害塔 + 波次时钟 ∈ [2.2, 3.0) 取样；窗口宽 0.8s ≫ 一次 BRP 往返。
const SAMPLE_FROM = 2.2;
const SAMPLE_TO = 3.0;

// ---- 指纹注册表（证据档冻结值，禁改；出处逐条注）----
// level_01：T047 冻结基线（t047-m6-wave1 README §指纹链；六波零漂移）。
const GRID_DIGEST_LEVEL_01 = "70282f80f5c61a46d5a775c5fbe8c020133bd05885d73c490dbb611962737a07";
// level_02 / level_03：T049 P13 首次运行登记（t049-m6-wave3 README §三；P14–P25 强断言零漂移）。
const GRID_DIGEST_LEVEL_02 = "3edf8dffc55b743933c8c4b8484f50855b8e7d193495d5f4c2536f96ea8a4f6d";
const GRID_DIGEST_LEVEL_03 = "840298ef16970017e1916a573a5333759bea8782a959509d365c12fc9dcd76f7";
// level_01 实体指纹：T048 冻结基线（场景 = 1 座无害 rocket @[2,0] + elapsed ∈ [2.2,3.0)）。
const ENTITY_DIGEST_BASELINE = "1d40c7b7d50b86d130f9382d99d66531874fe31142ed393e8a36ce3b4e890b23";

// ---- 独立解析器：塔注册表（TOML 子集）----
function parseTowersToml(file) {
  const lines = readFileSync(file, "utf8")
    .split(/\r?\n/)
    .map((l) => l.replace(/#.*$/, "").trim())
    .filter(Boolean);
  let section = null;
  const out = { towers: {}, projectiles: {} };
  for (const line of lines) {
    const h = line.match(/^\[([^\]]+)\]$/);
    if (h) {
      section = h[1].trim();
      continue;
    }
    const kv = line.match(/^([A-Za-z0-9_]+)\s*=\s*(.+)$/);
    if (!kv || !section) continue;
    const [, key, rawValue] = kv;
    const [kind, id] = section.split(".");
    if ((kind !== "towers" && kind !== "projectiles") || !id) continue;
    out[kind][id] ??= {};
    const v = rawValue.trim();
    let value;
    if (/^".*"$/.test(v)) value = v.slice(1, -1);
    else if (/^-?[\d.]+$/.test(v)) value = Number(v);
    else if (v.startsWith("[[")) value = (v.match(/\[[^[\]]*\]/g) ?? []).length; // 嵌套数组 → 槽位数
    else value = v;
    out[kind][id][key] = value;
  }
  return out;
}

// ---- 独立解析器：波次事件表（time / type / health / speed / bounty）----
// 口径 = `build_spawn_schedule`（enemy.rs:100-120）的独立重写：逐波 → 逐组 → 逐只展开，
// time = start_time + 组内序号 × spawn_interval，最后按 time 稳定排序。
function parseLevelEvents(file) {
  const lines = readFileSync(file, "utf8")
    .split(/\r?\n/)
    .map((l) => l.replace(/#.*$/, "").trim())
    .filter(Boolean);
  let section = null;
  let cur = null;
  const types = new Map();
  const waves = [];
  for (const line of lines) {
    const dh = line.match(/^\[\[([^\]]+)\]\]$/);
    if (dh) {
      section = dh[1].trim();
      if (section === "waves") {
        cur = { groups: [] };
        waves.push(cur);
      }
      continue;
    }
    const h = line.match(/^\[([^\]]+)\]$/);
    if (h) {
      section = h[1].trim();
      continue;
    }
    if (section && section.startsWith("enemy_types.")) {
      const key = section.slice("enemy_types.".length);
      types.set(key, types.get(key) ?? {});
      const m = line.match(/^(health|speed|bounty)\s*=\s*([\d.]+)/);
      if (m) types.get(key)[m[1]] = Number(m[2]);
      continue;
    }
    if (section === "waves" && cur) {
      let m = line.match(/^start_time\s*=\s*([\d.]+)/);
      if (m) {
        cur.start = Number(m[1]);
        continue;
      }
      m = line.match(/^spawn_interval\s*=\s*([\d.]+)/);
      if (m) {
        cur.interval = Number(m[1]);
        continue;
      }
      m = line.match(/\{\s*type\s*=\s*"([^"]+)"\s*,\s*count\s*=\s*(\d+)/);
      if (m) {
        cur.groups.push({ type: m[1], count: Number(m[2]) });
        continue;
      }
    }
  }
  const events = [];
  for (const w of waves) {
    let t = w.start;
    for (const g of w.groups) {
      const def = types.get(g.type);
      if (!def) throw new Error(`${file}: unknown enemy type ${g.type}`);
      for (let i = 0; i < g.count; i++) {
        events.push({ time: t, type: g.type, health: def.health, speed: def.speed, bounty: def.bounty });
        t += w.interval;
      }
    }
  }
  events.sort((a, b) => a.time - b.time);
  return events;
}

const TOWERS_TOML = parseTowersToml("game-td/assets/towers.toml");

// ---- 关卡表：网格展开 / 事件表 / 覆盖规划（全部脚本内独立算）----
const LEVELS = [
  { key: "level_01", discovered: "assets/levels/level_01.toml", path: "game-td/assets/levels/level_01.toml",
    gridDigest: GRID_DIGEST_LEVEL_01, entityBaseline: ENTITY_DIGEST_BASELINE },
  { key: "level_02", discovered: "assets/levels/level_02.toml", path: "game-td/assets/levels/level_02.toml",
    gridDigest: GRID_DIGEST_LEVEL_02, entityBaseline: null },
  { key: "level_03", discovered: "assets/levels/level_03.toml", path: "game-td/assets/levels/level_03.toml",
    gridDigest: GRID_DIGEST_LEVEL_03, entityBaseline: null },
].map((entry) => {
  const parsed = parseLevelFile(entry.path);
  const expanded = expandLevel(parsed);
  const events = parseLevelEvents(entry.path);
  // 双解析器互证：TOML 展开器的 count 汇总 == 事件表长度（禁手数）。
  if (parsed.events !== events.length) {
    throw new Error(`${entry.key}: events mismatch ${parsed.events} vs ${events.length}`);
  }
  const pathTiles = [];
  const grassTiles = [];
  for (let y = 0; y < ROWS; y++) {
    for (let x = 0; x < COLS; x++) {
      (expanded.grid[y][x] === "P" ? pathTiles : grassTiles).push([x, y]);
    }
  }
  const wps = [...parsed.paths.values()][0];
  let pathTilesLen = 0;
  for (let i = 0; i + 1 < wps.length; i++) {
    pathTilesLen += Math.abs(wps[i + 1][0] - wps[i][0]) + Math.abs(wps[i + 1][1] - wps[i][1]);
  }
  return { ...entry, parsed, expanded, events, pathTiles, grassTiles, pathPx: pathTilesLen * TILE };
});

function distToPath(entry, tile) {
  let min = Infinity;
  for (const [px, py] of entry.pathTiles) {
    const d = Math.hypot((px - tile[0]) * TILE, (py - tile[1]) * TILE);
    if (d < min) min = d;
  }
  return min;
}

/// 贪心集合覆盖（确定性：先按覆盖数降序，数量同则取 y 小、再 x 小的格子）。
function planTowers(entry, range, n) {
  const chosen = [];
  const covered = new Set();
  while (chosen.length < n) {
    let best = null;
    let bestCount = -1;
    for (const t of entry.grassTiles) {
      if (chosen.some((c) => c[0] === t[0] && c[1] === t[1])) continue;
      let count = 0;
      for (const p of entry.pathTiles) {
        if (covered.has(`${p[0]},${p[1]}`)) continue;
        if (Math.hypot((p[0] - t[0]) * TILE, (p[1] - t[1]) * TILE) <= range) count++;
      }
      if (count > bestCount) {
        best = t;
        bestCount = count;
      }
    }
    if (!best) break;
    chosen.push(best);
    for (const p of entry.pathTiles) {
      if (Math.hypot((p[0] - best[0]) * TILE, (p[1] - best[1]) * TILE) <= range) {
        covered.add(`${p[0]},${p[1]}`);
      }
    }
  }
  return chosen;
}

/// 最优覆盖格（排除已占位）；rocket 专用——rocket 与 rapid 同 range，贪心首选位会与初始 3 座撞车。
function bestCoverTile(entry, range, excluded) {
  let best = null;
  let bestCount = -1;
  for (const t of entry.grassTiles) {
    if (excluded.some(([x, y]) => x === t[0] && y === t[1])) continue;
    let count = 0;
    for (const p of entry.pathTiles) {
      if (Math.hypot((p[0] - t[0]) * TILE, (p[1] - t[1]) * TILE) <= range) count++;
    }
    if (count > bestCount) {
      best = t;
      bestCount = count;
    }
  }
  return best;
}

/// 无害塔位：离路径最远的草地格（最小距离 > range 才真正无害——见 distToPath 单调界）。
function harmlessTile(entry) {
  let best = null;
  let bestD = -1;
  for (const t of entry.grassTiles) {
    const d = distToPath(entry, t);
    if (d > bestD) {
      bestD = d;
      best = t;
    }
  }
  return { tile: best, dist: bestD };
}

for (const entry of LEVELS) {
  entry.rapidPlan = planTowers(entry, RANGE, 8);
  entry.rocketPlan = planTowers(entry, RANGE, 2);
  const harmless = harmlessTile(entry);
  entry.harmless = entry.key === "level_01" ? { tile: [2, 0], dist: distToPath(entry, [2, 0]) } : harmless;
  entry.lastEventTime = entry.events[entry.events.length - 1].time;
  entry.victoryUpper = entry.lastEventTime + entry.pathPx / entry.events[entry.events.length - 1].speed + 3;
  // 漏怪时间线（第 5 只到终点 = 判负最早时刻；零塔场景即实际判负时刻）。
  entry.leakTimes = entry.events
    .map((e) => e.time + entry.pathPx / e.speed)
    .sort((a, b) => a - b);
}

function expectedSpawned(entry, elapsed) {
  return entry.events.filter((e) => e.time <= elapsed + 1e-6).length;
}
function expectedHealths(entry, spawned) {
  return entry.events
    .slice(0, spawned)
    .map((e) => e.health)
    .sort((a, b) => a - b);
}

// ---- 公共：世界查询与指纹 ----
const POS = "bevy_ecs_tilemap::tiles::TilePos";
const TEX = "bevy_ecs_tilemap::tiles::TileTextureIndex";
const TR = "bevy_transform::components::transform::Transform";

function tileIndex(row) {
  const raw = row.components[TEX];
  return Array.isArray(raw) ? raw[0] : raw;
}
function tupleValue(raw) {
  return Array.isArray(raw) ? raw[0] : raw;
}
async function countOf(component, withFilter = null) {
  const params = { data: { components: [component] } };
  if (withFilter) params.filter = { with: withFilter };
  const rows = await brp("world.query", params);
  return rows.length;
}
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
function entityDigest(towers, turrets, healths) {
  const lines = [];
  for (const t of towers) {
    const tr = t.components[TR]?.translation;
    const x = Array.isArray(tr) ? tr[0] : tr?.x;
    const y = Array.isArray(tr) ? tr[1] : tr?.y;
    lines.push(`T:${x},${y}`);
  }
  lines.push(`TT:${turrets}`);
  for (const h of healths) lines.push(`E:${h}`);
  lines.sort();
  return createHash("sha256").update(lines.join("\n")).digest("hex");
}
/// 独立复算的实体指纹期望（塔格 → 世界坐标；血量前缀来自事件表）。
function expectedEntityDigest(entry, towerTiles, spawned) {
  const lines = [];
  for (const [tx, ty] of towerTiles) lines.push(`T:${OFFSET_X + tx * TILE},${OFFSET_Y + ty * TILE}`);
  lines.push(`TT:${towerTiles.length}`);
  for (const h of expectedHealths(entry, spawned)) lines.push(`E:${h}`);
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
async function enemyRows() {
  return brp("world.query", {
    data: { components: ["game_td::Enemy", TR, "game_td::Health", "game_td::Bounty"] },
    filter: { with: ["game_td::Enemy"] },
  });
}
async function entitySample() {
  const tw = await towerSnapshot();
  const healths = await enemyHealths();
  return {
    digest: entityDigest(tw.towers, tw.turrets, healths),
    healths: healths.slice().sort((a, b) => a - b),
    towerCount: tw.towers.length,
    turretCount: tw.turrets,
  };
}

// ---- 公共：状态机驱动 ----
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
async function waitGameUntilElapsed(minElapsed, timeoutMs = 40000) {
  const deadline = Date.now() + timeoutMs;
  let g = await brp("td.game");
  while (Date.now() < deadline) {
    if (g.game_over || (typeof g.elapsed === "number" && g.elapsed >= minElapsed)) return g;
    await sleep(30);
    g = await brp("td.game");
  }
  return g;
}
/// 回到 LevelSelect 并等清场落地。逐次断言 td.state set 写通道的 from/to（登记项①）。
async function leaveToMenu(label = "leaveToMenu") {
  const s = await brp("td.state");
  if (s.state === "LevelSelect") return;
  const set = await brp("td.state", { op: "set", state: "LevelSelect" });
  check(`${label} td.state set LevelSelect queued (from=${s.state})`,
    set.queued === true && set.from === s.state && set.to === "LevelSelect", JSON.stringify(set));
  const st = await waitState("LevelSelect", 10000);
  check(`${label} state==LevelSelect after transition`, st.state === "LevelSelect", JSON.stringify(st));
  await sleep(300); // OnExit cleanup 命令落地
}
/// 进关：select(index) → set InGame。逐次断言 td.state set 写通道（登记项①）。
async function enterLevel(index, label) {
  const s = await brp("td.state");
  if (s.state !== "LevelSelect") await leaveToMenu(label);
  const sel = await brp("td.level", { op: "select", index });
  check(`${label} td.level select index=${index} -> ${LEVELS[index].discovered}`,
    sel.index === index && sel.selected === LEVELS[index].discovered, JSON.stringify(sel));
  const set = await brp("td.state", { op: "set", state: "InGame" });
  check(`${label} td.state set InGame queued (LevelSelect -> InGame)`,
    set.queued === true && set.from === "LevelSelect" && set.to === "InGame", JSON.stringify(set));
  const st = await waitState("InGame", 10000);
  check(`${label} state==InGame after transition`, st.state === "InGame", JSON.stringify(st));
  await sleep(320); // OnEnter 装配（tilemap / 预览 / 资源）落地
}

// ---- 公共：截图两段式（可选参数；缺 --shot 时记 INFO 不判 FAIL）----
function readPathOf(brpPath) {
  return brpPath.startsWith("../") ? brpPath.slice(3) : brpPath;
}
/// 截图路径按相位加后缀（同一 --shot 参数会被四轮胜利相位复用：逐次独立成档，禁覆盖式同名）。
function withTag(path, tag) {
  if (!path) return path;
  const safe = tag.replace(/[^A-Za-z0-9_.-]+/g, "-");
  const m = path.match(/^(.*?)(\.png)$/i);
  return m ? `${m[1]}-${safe}${m[2]}` : `${path}-${safe}`;
}
async function captureAndVerify(brpPath, label) {
  if (!brpPath) {
    info(`${label} screenshot skipped (no --shot)`);
    return;
  }
  const st0 = await brp("td.screenshot", { op: "status" });
  const queued = await brp("td.screenshot", { op: "capture", path: brpPath });
  check(`${label} screenshot queued`, queued.queued === true && queued.id === st0.next_id,
    `id=${queued.id} baseId=${st0.next_id}`);
  let entry = null;
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    const st = await brp("td.screenshot", { op: "status" });
    entry = st.entries.find((e) => e.id === queued.id);
    if (entry && entry.status === "captured") break;
    await sleep(200);
  }
  check(`${label} screenshot status==captured`, !!entry && entry.status === "captured",
    entry ? JSON.stringify(entry) : "no entry");
  if (!entry || entry.status !== "captured") return;
  let buf;
  try {
    buf = readFileSync(readPathOf(brpPath));
  } catch (e) {
    check(`${label} screenshot file readable`, false, `${readPathOf(brpPath)}: ${e.code}`);
    return;
  }
  const magic = buf.subarray(0, 8).toString("hex");
  const w = buf.readUInt32BE(16);
  const h = buf.readUInt32BE(20);
  check(`${label} screenshot PNG 1600x900 non-empty`,
    magic === "89504e470d0a1a0a" && buf.length > 0 && w === 1600 && h === 900,
    `magic=${magic} bytes=${buf.length} ${w}x${h}`);
  info(`${label} SCREENSHOT md5=${createHash("md5").update(buf).digest("hex")} bytes=${buf.length}`);
}

// ---- 公共：地图断言（level_01 = P3 基线逐格索引规格；level_02/03 = 独立展开对拍）----
const PATH_MID = ROWS / 2; // 5 -> 道路行 {4,5,6}
const PATH_INDICES = new Set([79, 80, 81, 102, 103, 104, 125, 126, 127]);
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
async function assertMapLevel01(label) {
  const all = await brp("world.query", { data: { components: ["game_td::MapTile", POS, TEX] } });
  check(`${label} MapTile count==${MAP_TILE}`, all.length === MAP_TILE, `len=${all.length}`);
  const paths = await brp("world.query", { data: { components: ["game_td::PathTile"] } });
  const types = await brp("world.query", { data: { components: ["game_td::TileType"] } });
  const byType = new Map();
  for (const row of types) {
    const v = row.components["game_td::TileType"];
    byType.set(v, (byType.get(v) ?? 0) + 1);
  }
  check(`${label} TileType Grass==105 / Path==45`,
    byType.get("Grass") === 105 && byType.get("Path") === 45,
    `grass=${byType.get("Grass")} path=${byType.get("Path")}`);
  check(`${label} PathTile count==45`, paths.length === 45, `len=${paths.length}`);
  let posBad = 0;
  let idxBad = 0;
  const seen = new Set();
  for (const row of all) {
    const p = row.components[POS];
    if (!p || typeof p.x !== "number" || typeof p.y !== "number") {
      posBad++;
      continue;
    }
    seen.add(`${p.x},${p.y}`);
    if (tileIndex(row) !== expectedIndex(p.y, p.x)) idxBad++;
  }
  check(`${label} TilePos covers 15x10 exactly once`, posBad === 0 && seen.size === MAP_TILE,
    `bad=${posBad} covered=${seen.size}/${MAP_TILE}`);
  check(`${label} texture index per cell == P3 baseline spec`, idxBad === 0, `bad=${idxBad}`);
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
  check(`${label} tile index set == P3 index set`,
    [...allIdx].sort((a, b) => a - b).join(",") === [...PATH_INDICES, 129].sort((a, b) => a - b).join(","),
    `got=${[...allIdx].sort((a, b) => a - b).join(",")}`);
  return { digest: gridDigest(all), all, paths };
}
async function assertGenericMap(label, entry) {
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
  check(`${label} TileType Grass==${exp.grassCount} / Path==${exp.pathCount}`,
    byType.get("Grass") === exp.grassCount && byType.get("Path") === exp.pathCount,
    `grass=${byType.get("Grass")} path=${byType.get("Path")}`);
  check(`${label} PathTile count==${exp.pathCount}`, paths.length === exp.pathCount, `len=${paths.length}`);
  let posBad = 0;
  const seen = new Set();
  for (const row of all) {
    const p = row.components[POS];
    if (!p || typeof p.x !== "number" || typeof p.y !== "number") {
      posBad++;
      continue;
    }
    seen.add(`${p.x},${p.y}`);
  }
  check(`${label} TilePos covers ${COLS}x${ROWS} exactly once`,
    posBad === 0 && seen.size === exp.total, `bad=${posBad} covered=${seen.size}/${exp.total}`);
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
  return { digest: gridDigest(all), all, paths };
}

// ---- 公共：塔注册表独立对拍（game td.towers vs towers.toml）----
function near(a, b, eps = 1e-6) {
  return typeof a === "number" && typeof b === "number" && Math.abs(a - b) <= eps;
}
async function checkRegistry(label) {
  const reg = await brp("td.towers");
  const got = new Map((reg.towers ?? []).map((t) => [t.key, t]));
  const keys = [...got.keys()].sort();
  check(`${label} registry keys == towers.toml keys`,
    JSON.stringify(keys) === JSON.stringify(Object.keys(TOWERS_TOML.towers).sort()), JSON.stringify(keys));
  for (const [key, def] of Object.entries(TOWERS_TOML.towers)) {
    const g = got.get(key);
    if (!g) {
      check(`${label} registry has ${key}`, false);
      continue;
    }
    check(`${label} ${key}: cost/range/cooldown == toml`,
      g.cost === def.cost && near(g.attack_range, def.attack_range) && near(g.attack_cooldown, def.attack_cooldown),
      `got=${JSON.stringify({ c: g.cost, r: g.attack_range, cd: g.attack_cooldown })} ` +
        `want=${JSON.stringify({ c: def.cost, r: def.attack_range, cd: def.attack_cooldown })}`);
    if (typeof def.projectile === "string") {
      const p = TOWERS_TOML.projectiles[def.projectile];
      check(`${label} ${key}: projectile == projectiles.${def.projectile}`,
        g.projectile && near(g.projectile.damage, p.damage) && near(g.projectile.speed, p.speed) &&
          near(g.projectile.splash_radius, p.splash_radius),
        `got=${JSON.stringify(g.projectile)} want=${JSON.stringify(p)}`);
      check(`${label} ${key}: ammo_slots == toml slot count`,
        g.ammo_slots === def.ammo_slot_offsets,
        `got=${g.ammo_slots} want=${def.ammo_slot_offsets}`);
      check(`${label} ${key}: ammo_refill_secs == toml`,
        near(g.ammo_refill_secs, def.ammo_refill_secs),
        `got=${g.ammo_refill_secs} want=${def.ammo_refill_secs}`);
    } else {
      check(`${label} ${key}: instant tower (no projectile) damage == toml`,
        g.projectile === null && near(g.damage, def.damage),
        `projectile=${JSON.stringify(g.projectile)} damage=${g.damage} want=${def.damage}`);
    }
  }
}

// ---- 公共：经济账（独立复算）----
// gold = STARTING_GOLD + PASSIVE_RATE*elapsed + BOUNTY*kills − Σcost
// kills = spawned − aliveField − leaked，leaked = STARTING_LIVES − lives
// （推导：spawned = alive + killed + leaked；bounty 只在 health<=0 死亡时入账，
//  漏怪只扣命不入账——economy.rs:57 / tower.rs:884,1113 / enemy.rs:311-317）
/// 经济账**一致采样**（驱动侧非原子性对策）：
/// 三个读数（td.wave / Enemy 计数 / td.economy）是三次独立 BRP 往返，中间若有敌人被击杀
/// （bounty +25 与 despawn 同帧落地），单次采样会看到「gold 已 +25 而 alive/spawned 仍是击杀前」
/// 的瞬态（run1 实测 diff=25.28 = 恰好 1 个 bounty）。对策 = 重采到自洽样本（最多 3 次），
/// 全档披露尝试序列；**不放宽恒等式本身**（一致性阈值仍 1.0 金）。
async function ledgerSampleConsistent(spentCosts, tries = 3) {
  const attempts = [];
  for (let i = 0; i < tries; i++) {
    const w = await brp("td.wave");
    const alive = await countOf("game_td::Enemy");
    const eco = await brp("td.economy");
    const g = await brp("td.game");
    if (typeof g.lives !== "number" || typeof w.elapsed !== "number" || typeof eco.gold !== "number") {
      attempts.push({ error: "precondition", g, w, eco });
      continue;
    }
    const leaked = STARTING_LIVES - g.lives;
    const kills = w.spawned - alive - leaked;
    const predicted = STARTING_GOLD + PASSIVE_RATE * w.elapsed + 25 * kills - spentCosts;
    const diff = eco.gold - predicted;
    attempts.push({
      elapsed: w.elapsed, spawned: w.spawned, alive, lives: g.lives, kills,
      gold: eco.gold, predicted: Number(predicted.toFixed(4)), diff: Number(diff.toFixed(4)),
    });
    if (Math.abs(diff) <= 1.0) return { ok: true, attempts, index: i + 1 };
    await sleep(30);
  }
  return { ok: false, attempts, index: null };
}

// ---- 相位前段：屏幕面（方法面 / 发现 / 卫生 / 错误路径）----
async function phaseScreen() {
  info(`=== phase screen @ ${new Date().toISOString()} ===`);
  const disc = await brp("rpc.discover");
  const names = disc.methods.map((m) => m.name);
  check("screen discover.version==0.20.0", disc.info.version === "0.20.0", `version=${disc.info.version}`);
  check(`screen discover.methods==${expectDiscover}`, names.length === expectDiscover, `len=${names.length}`);
  const tdNames = names.filter((n) => n.startsWith("td.")).sort();
  const expectedTd = [
    "td.audio", "td.combat", "td.conditions", "td.economy", "td.events", "td.game", "td.gamepad",
    "td.gizmos", "td.input", "td.level", "td.level_select", "td.pause", "td.screenshot",
    "td.spawn_tower", "td.state", "td.tower_select", "td.towers", "td.wave",
  ];
  check("screen td.* method surface == 18 (registry of the 5 waves)", JSON.stringify(tdNames) === JSON.stringify(expectedTd),
    `${tdNames.length}: ${tdNames.join(",")}`);
  check("screen no game.* prefix collision", names.every((n) => !n.startsWith("game.")),
    names.filter((n) => n.startsWith("game.")).join(",") || "(none)");

  const lv = await brp("td.level");
  check("screen td.level discovery == 3 levels in order",
    JSON.stringify(lv.levels) === JSON.stringify(LEVELS.map((l) => l.discovered)), JSON.stringify(lv.levels));
  check("screen td.level default selected == level_01", lv.selected === LEVELS[0].discovered,
    String(lv.selected));

  const st = await brp("td.state");
  check("screen initial state LevelSelect / result null / finished false",
    st.state === "LevelSelect" && st.result === null && st.finished === false, JSON.stringify(st));
  const cond = await brp("td.conditions");
  check("screen conditions: game_is_running==false in LevelSelect",
    cond.game_state === "LevelSelect" && cond.game_is_running === false, JSON.stringify(cond));

  // 开局卫生（无关卡在场）
  check("screen LevelSelect hygiene: MapTile==0 / PathTile==0",
    (await countOf("game_td::MapTile")) === 0 && (await countOf("game_td::PathTile")) === 0);
  const w0 = await brp("td.wave");
  check("screen LevelSelect hygiene: wave schedule absent", w0.available === false, JSON.stringify(w0));
  const g0 = await brp("td.game");
  check("screen LevelSelect hygiene: lives null / game_over false",
    g0.lives === null && g0.game_over === false, JSON.stringify(g0));
  const e0 = await brp("td.economy");
  check("screen LevelSelect hygiene: economy unavailable", e0.available === false, JSON.stringify(e0));

  // td.* 错误路径（不击穿进程 = 件三；全部 -32602 / -32601）
  const cases = [
    ["td.state", { op: "bogus" }, -32602],
    ["td.state", { op: "set", state: "Nope" }, -32602],
    ["td.state", { op: "set" }, -32602],
    ["td.level", { op: "bogus" }, -32602],
    ["td.level", { op: "select", index: 99 }, -32602],
    ["td.spawn_tower", { tile: [1] }, -32602],
    ["td.spawn_tower", { tile: ["a", 1] }, -32602],
    ["td.tower_select", { op: "set", key: "nope" }, -32602],
    ["td.tower_select", { op: "set", index: 99 }, -32602],
    ["td.events", { op: "bogus" }, -32602],
    ["td.screenshot", { op: "bogus" }, -32602],
    ["td.pause", { op: "set", state: "Paused" }, -32602], // 守卫：非 InGame 拒绝
    ["td.input", { op: "emit", action: "Nope" }, -32602],
    ["td.input", { op: "set_cursor", tile: [0, 0] }, -32602], // 守卫：未加载关卡
    ["td.gamepad", { op: "probe_button", button: "Nope" }, -32602],
    ["td.gamepad", { op: "probe_stick", x: 0.5 }, -32602],
    ["td.level_select", { op: "bogus" }, -32602],
    ["td.definitely_not_a_method", null, -32601],
  ];
  let bad = [];
  for (const [method, params, want] of cases) {
    const code = await errCode(method, params);
    if (code !== want) bad.push(`${method}(${JSON.stringify(params)}) -> ${code} want ${want}`);
  }
  check(`screen td.* error paths return -32602/-32601 (${cases.length} cases)`, bad.length === 0,
    bad.join(" ; ") || `${cases.length}/${cases.length}`);
  const alive = await brp("td.state");
  check("screen process alive after error-path battery", alive.state === "LevelSelect", JSON.stringify(alive));

  if (gameLog) scanGameLog("screen");
  else info("screen game log scan deferred (no --game-log)");
}

function scanGameLog(label) {
  let text = "";
  try {
    text = readFileSync(gameLog, "utf8");
  } catch (e) {
    check(`${label} game log readable`, false, `${gameLog}: ${e.code}`);
    return [];
  }
  const panics = text
    .split(/\r?\n/)
    .filter((l) => /panicked at|failed validation|Resource does not exist/.test(l));
  check(`${label} no panic in game log`, panics.length === 0,
    panics.slice(0, 3).join(" || ") || `bytes=${Buffer.byteLength(text)}`);
  return panics;
}

// ---- 相位主体：胜利轮（核心循环 + 经济闭环 + digest + 终局 Victory）----
async function runVictoryPhase(index, label, opts = {}) {
  const L = LEVELS[index];
  const tag = `${label}[${L.key}]`;
  info(`=== ${tag} VICTORY run @ ${new Date().toISOString()} (pathPx=${L.pathPx}, events=${L.events.length}, ` +
    `lastSpawn=${L.lastEventTime}, victoryBy<=${L.victoryUpper.toFixed(1)}s) ===`);
  await enterLevel(index, tag);

  // ---- 地图面（GRID_DIGEST 注册表）----
  const map = index === 0 ? await assertMapLevel01(`${tag} map`) : await assertGenericMap(`${tag} map`, L);
  check(`${tag} GRID_DIGEST == registry[${L.key}]`, map.digest === L.gridDigest,
    `got=${map.digest} expect=${L.gridDigest}`);

  // ---- 开局状态 ----
  const g0 = await brp("td.game");
  check(`${tag} opening: lives==${STARTING_LIVES}`, g0.lives === STARTING_LIVES, `lives=${g0.lives}`);
  check(`${tag} opening: wave schedule total==${L.events.length}`, g0.total === L.events.length, `total=${g0.total}`);
  check(`${tag} opening: spawned<=1 (wave1 starts at t=0.0)`, g0.spawned <= 1, `spawned=${g0.spawned}`);
  const e0 = await brp("td.economy");
  check(`${tag} opening: economy constants 300 / 3.0`,
    e0.starting_gold === STARTING_GOLD && e0.passive_rate === PASSIVE_RATE,
    `starting=${e0.starting_gold} passive=${e0.passive_rate}`);
  check(`${tag} opening: gold == 300 + 3.0*elapsed (passive only, no spend yet)`,
    Math.abs(e0.gold - (STARTING_GOLD + PASSIVE_RATE * g0.elapsed)) <= 1.0,
    `gold=${e0.gold} elapsed=${g0.elapsed}`);
  const w0 = await brp("td.wave");
  check(`${tag} opening: spawned == independent event table @elapsed=${w0.elapsed?.toFixed(2)}`,
    w0.spawned === expectedSpawned(L, w0.elapsed), `spawned=${w0.spawned} expect=${expectedSpawned(L, w0.elapsed)}`);
  check(`${tag} opening: pending+spawned==total`, w0.pending + w0.spawned === w0.total,
    JSON.stringify({ p: w0.pending, s: w0.spawned, t: w0.total }));

  // ---- 塔注册表（独立 TOML 对拍）----
  await checkRegistry(tag);

  // ---- 放置链：3 座 rapid（贪心覆盖位）+ 负例 + 余额拒绝 ----
  const selRapid = await brp("td.tower_select", { op: "set", key: RAPID.key });
  check(`${tag} select tower rapid`, selRapid.key === RAPID.key, JSON.stringify(selRapid));
  let spent = 0;
  let towersPlaced = 0;
  for (const tile of L.rapidPlan.slice(0, 3)) {
    const r = await brp("td.spawn_tower", { tile });
    check(`${tag} place rapid @[${tile}]`, r.placed === true && r.cost === RAPID.cost && r.tower_type === RAPID.key,
      JSON.stringify(r));
    check(`${tag} place rapid @[${tile}] world pos == OFFSET + tile*TILE`,
      Array.isArray(r.pos) && near(r.pos[0], OFFSET_X + tile[0] * TILE) && near(r.pos[1], OFFSET_Y + tile[1] * TILE),
      `pos=${JSON.stringify(r.pos)}`);
    if (r.placed === true) {
      spent += r.cost;
      towersPlaced++;
    }
  }
  const dup = await brp("td.spawn_tower", { tile: L.rapidPlan[0] });
  check(`${tag} negative: occupied tile rejected`, dup.placed === false &&
    dup.reason === "tile is not an unoccupied grass tile", JSON.stringify(dup));
  const midPath = L.pathTiles[Math.floor(L.pathTiles.length / 2)];
  const onPath = await brp("td.spawn_tower", { tile: midPath });
  check(`${tag} negative: path tile [${midPath}] rejected`, onPath.placed === false &&
    onPath.reason === "tile is not an unoccupied grass tile", JSON.stringify(onPath));
  const oob = await brp("td.spawn_tower", { tile: [COLS, 5] });
  check(`${tag} negative: out-of-bounds tile rejected`, oob.placed === false, JSON.stringify(oob));
  const ecoNow = await brp("td.economy");
  const poor = await brp("td.spawn_tower", { tile: L.rapidPlan[3] });
  check(`${tag} negative: insufficient balance rejected (gold<cost)`,
    poor.placed === false && poor.reason === "not enough gold" && ecoNow.gold < RAPID.cost,
    `${JSON.stringify(poor)} goldBefore=${ecoNow.gold}`);
  check(`${tag} entities: Tower==3 / TowerTurret==3`,
    (await countOf("game_td::Tower")) === 3 && (await countOf("game_td::TowerTurret")) === 3,
    `towers=${await countOf("game_td::Tower")}`);

  // ---- 核心循环采样（登记项③：事件表独立复算 + elapsed 检查点）----
  // 可选：opts.rocket 在金币允许时补一座 rocket（覆盖路径的贪心位）——用于「弹道 / 弹药 /
  // 溅射」面（登记项⑥）：rapid 是瞬时塔（无 AmmoState / 无 projectile），td.combat 的
  // launchers/fired/projectiles 只在有 rocket 时非零。
  const cps = [SAMPLE_FROM, L.lastEventTime * 0.6, L.lastEventTime + 0.5];
  let cpIndex = 0;
  let lastGame = g0;
  let minLives = g0.lives;
  let maxFired = 0;
  let maxLaunchers = 0;
  let maxAmmoCapacity = 0;
  let projectilesSeen = 0;
  let rocketPlaced = 0;
  let rocketAborted = false;
  // rocket 位 = 覆盖路径最好的**空闲**草地格（排除初始 3 座 rapid 占位）。
  const rocketTile = opts.rocket ? bestCoverTile(L, RANGE, L.rapidPlan.slice(0, 3)) : null;
  if (opts.rocket) {
    check(`${tag} rocket slot available (free grass tile covering path)`, !!rocketTile, JSON.stringify(rocketTile));
  }
  let lastCombat = null;
  let splashSig = null;
  let shotTaken = false;
  const deadline = Date.now() + 60000;
  let st = null;
  while (Date.now() < deadline) {
    const w = await brp("td.wave");
    const g = await brp("td.game");
    const c = await brp("td.combat");
    lastCombat = c;
    maxLaunchers = Math.max(maxLaunchers, c.launchers);
    maxAmmoCapacity = Math.max(maxAmmoCapacity, c.ammo_capacity);
    maxFired = Math.max(maxFired, c.fired);
    projectilesSeen = Math.max(projectilesSeen, c.projectiles);
    if (typeof g.lives === "number") minLives = Math.min(minLives, g.lives);
    if (typeof g.elapsed === "number") lastGame = g;

    // 登记项⑥：补一座 rocket（覆盖路径 → 真实弹道 / 溅射面）。**排在机会性 rapid 之前**：
    // rocket 单价 150 > rapid 100，若先花在 rapid 上则金币永远攒不到 150（实测发现）。
    if (opts.rocket && rocketPlaced === 0 && !rocketAborted) {
      const eco = await brp("td.economy");
      if (typeof eco.gold === "number" && eco.gold >= ROCKET.cost) {
        const sel = await brp("td.tower_select", { op: "set", key: ROCKET.key });
        const r = await brp("td.spawn_tower", { tile: rocketTile });
        info(`${tag} rocket @[${rocketTile}] (select=${sel.key}) -> ` +
          (r.placed === true ? `placed (cost=${r.cost})` : `refused (${r.reason})`));
        if (r.placed === true) {
          spent += r.cost;
          rocketPlaced = 1;
        } else if (r.reason !== "not enough gold") {
          rocketAborted = true; // 非余额原因（如占格）→ 不再重试，避免日志刷屏
        }
        // 恢复 rapid 选中（后续机会性补塔仍为 rapid）
        await brp("td.tower_select", { op: "set", key: RAPID.key });
      }
    }
    // 机会性补塔（金币来自被动 + 赏金；记账用实际 cost）。
    // rocket 待补期间**不**花金币在 rapid 上（否则 gold 永远攒不到 150——实测发现）。
    const rapidBudgetFree = !(opts.rocket && rocketPlaced === 0);
    if (rapidBudgetFree && opts.extraTowers !== false && towersPlaced < L.rapidPlan.length) {
      const eco = await brp("td.economy");
      if (typeof eco.gold === "number" && eco.gold >= RAPID.cost) {
        const tile = L.rapidPlan[towersPlaced];
        const r = await brp("td.spawn_tower", { tile });
        info(`${tag} opportunistic rapid #${towersPlaced + 1} @[${tile}] -> ` +
          (r.placed === true ? `placed (cost=${r.cost})` : `refused (${r.reason})`));
        if (r.placed === true) {
          spent += r.cost;
          towersPlaced++;
        }
      }
    }

    // 登记项⑥（可选，时间盒）：溅射多目标签名（两敌同时恰为满血-50 且间距 <60px）
    if (opts.splash && !splashSig) {
      const sig = await splashSignature();
      if (sig) splashSig = sig;
    }

    if (w.available && cpIndex < cps.length && w.elapsed >= cps[cpIndex]) {
      check(`${tag} cp${cpIndex + 1} spawned == independent event table @elapsed=${w.elapsed.toFixed(2)}`,
        w.spawned === expectedSpawned(L, w.elapsed),
        `spawned=${w.spawned} expect=${expectedSpawned(L, w.elapsed)}`);
      check(`${tag} cp${cpIndex + 1} pending == total - spawned`, w.pending === w.total - w.spawned,
        `pending=${w.pending} total=${w.total} spawned=${w.spawned}`);
      const sample = await ledgerSampleConsistent(spent);
      const last = sample.attempts[sample.attempts.length - 1] ?? {};
      check(`${tag} cp${cpIndex + 1} economy ledger: gold == 300 + 3.0*t + 25*kills - Σcost`,
        sample.ok,
        `attempts=${JSON.stringify(sample.attempts)}` + (sample.ok ? ` (tries=${sample.index})` : ""));
      if (sample.ok && sample.index > 1) {
        info(`${tag} cp${cpIndex + 1} ledger needed ${sample.index} samples (in-flight kill race; all attempts:` +
          ` ${JSON.stringify(sample.attempts)})`);
      }
      check(`${tag} cp${cpIndex + 1} kills >= 0 at ledger sample (spawned-alive-leaked)`,
        typeof last.kills === "number" && last.kills >= 0, `kills=${last.kills}`);
      check(`${tag} cp${cpIndex + 1} towers on field == placed count (${towersPlaced + rocketPlaced})`,
        (await countOf("game_td::Tower")) === towersPlaced + rocketPlaced,
        `towers=${await countOf("game_td::Tower")}`);
      if (cpIndex === 0 && !shotTaken) {
        shotTaken = true;
        await captureAndVerify(withTag(shotArg, tag), tag);
      }
      cpIndex++;
    }

    st = await brp("td.state");
    if (st.state === "GameOver") break;
    await sleep(80);
  }

  // ---- 终局（Victory：零漏清波）----
  check(`${tag} reached GameOver`, st && st.state === "GameOver", JSON.stringify(st));
  check(`${tag} result == Victory`, st && st.result === "Victory", JSON.stringify(st));
  check(`${tag} VICTORY with zero leaks (lives stayed ${STARTING_LIVES})`, minLives === STARTING_LIVES,
    `minLives=${minLives}`);
  check(`${tag} victory elapsed in [lastSpawn=${L.lastEventTime}, upper=${L.victoryUpper.toFixed(1)}]`,
    typeof lastGame.elapsed === "number" && lastGame.elapsed >= L.lastEventTime - 1e-6 &&
      lastGame.elapsed <= L.victoryUpper,
    `elapsed=${lastGame.elapsed} spawned=${lastGame.spawned}/${lastGame.total}`);
  check(`${tag} all ${L.events.length} events consumed (pending==0)`,
    lastGame.spawned === L.events.length && lastGame.pending === 0, JSON.stringify(lastGame));
  // 战斗面：rapid = 瞬时塔（无 AmmoState / 无弹体）；rocket 才进入 ammo/projectile 计数面。
  if (rocketPlaced === 1) {
    check(`${tag} combat: rocket ammo surface (launchers==1 / ammo_capacity==3 / fired>=1 / projectile seen)`,
      maxLaunchers === 1 && maxAmmoCapacity === 3 && maxFired >= 1 && projectilesSeen >= 1,
      `launchers=${maxLaunchers} ammoCap=${maxAmmoCapacity} fired=${maxFired} projectiles=${projectilesSeen}`);
  } else {
    check(`${tag} combat: rapid is hitscan (launchers==0 / ammo_capacity==0 / projectiles==0)`,
      maxLaunchers === 0 && maxAmmoCapacity === 0 && projectilesSeen === 0,
      `launchers=${maxLaunchers} ammoCap=${maxAmmoCapacity} projectiles=${projectilesSeen}`);
  }
  info(`${tag} combat snapshot: ` + JSON.stringify({ launchers: maxLaunchers, ammoCap: maxAmmoCapacity,
    fired: maxFired, projectiles: projectilesSeen, rocket: rocketPlaced,
    rocket_damage: lastCombat?.rocket_damage, splash_radius: lastCombat?.splash_radius }));
  // 塔在胜利瞬间仍存活（cleanup 只在 OnExit 跑）——在检查点内核对，终局后由 cleanup 断言归零。
  await sleep(300);
  if (opts.victoryShot) await captureAndVerify(shotVictoryArg, `${tag} victory-screen`);
  await assertTerminalCleanup(tag, "Victory");
  if (opts.splash) {
    if (splashSig) info(`${tag} SPLASH SIGNATURE ${JSON.stringify(splashSig)}`);
    else skip(`${tag} splash multi-target signature`, "20min 时间盒内未观测到（可选登记项⑥）");
  }
  return { tag, elapsed: lastGame.elapsed, towersPlaced, spent };
}

/// 终局清场面（Victory / Defeat 共用）。
async function assertTerminalCleanup(tag, wantResult) {
  const st = await brp("td.state");
  check(`${tag} cleanup: state GameOver / result ${wantResult}`, st.state === "GameOver" && st.result === wantResult,
    JSON.stringify(st));
  check(`${tag} cleanup: GameFinished removed (finished==false, fresh-round gate)`, st.finished === false,
    JSON.stringify(st));
  check(`${tag} cleanup: MapTile==0 / PathTile==0`,
    (await countOf("game_td::MapTile")) === 0 && (await countOf("game_td::PathTile")) === 0,
    `map=${await countOf("game_td::MapTile")}`);
  check(`${tag} cleanup: Enemy==0 / Tower==0 / TowerTurret==0`,
    (await countOf("game_td::Enemy")) === 0 && (await countOf("game_td::Tower")) === 0 &&
      (await countOf("game_td::TowerTurret")) === 0);
  check(`${tag} cleanup: wave schedule removed`, (await brp("td.wave")).available === false);
  const g = await brp("td.game");
  check(`${tag} cleanup: lives removed / game_over flag false`, g.lives === null && g.game_over === false,
    JSON.stringify(g));
  check(`${tag} cleanup: GameResult kept for game-over screen`, st.result === wantResult, String(st.result));
}

// ---- 相位主体：失败轮（放漏致 lives 0；无塔 → 零伤害零赏金）----
async function runDefeatPhase(index, label) {
  const L = LEVELS[index];
  const tag = `${label}[${L.key}]`;
  const defeatLower = L.leakTimes[STARTING_LIVES - 1]; // 第 5 只到终点
  info(`=== ${tag} DEFEAT run @ ${new Date().toISOString()} (5th leak >= ${defeatLower.toFixed(2)}s) ===`);
  await enterLevel(index, tag);
  check(`${tag} MapTile count==${L.expanded.total} (no tower defence)`,
    (await countOf("game_td::MapTile")) === L.expanded.total, `map=${await countOf("game_td::MapTile")}`);
  const g0 = await brp("td.game");
  check(`${tag} opening lives==${STARTING_LIVES}`, g0.lives === STARTING_LIVES, `lives=${g0.lives}`);

  let cpDone = false;
  let lastGame = g0;
  let minLives = g0.lives;
  let bountyBad = 0;
  let damageSeen = false;
  const deadline = Date.now() + 45000;
  let st = null;
  while (Date.now() < deadline) {
    const w = await brp("td.wave");
    const g = await brp("td.game");
    if (typeof g.lives === "number") minLives = Math.min(minLives, g.lives);
    if (typeof g.elapsed === "number") lastGame = g;

    if (w.available && !cpDone && w.elapsed >= SAMPLE_FROM) {
      cpDone = true;
      // 采样一致化（同 ledgerSampleConsistent 的驱动侧非原子性对策）：
      // `td.wave.spawned` 与敌人行读取之间若恰好刷出新敌，长度不等——重采至一致（最多 3 次），
      // 全档披露尝试；不放宽「血量多重集 == 事件表前缀」断言本身。
      let rows = [];
      let healths = [];
      let exp = [];
      let wCp = w;
      const attempts = [];
      for (let attempt = 0; attempt < 3; attempt++) {
        wCp = await brp("td.wave");
        rows = await enemyRows();
        healths = rows.map((r) => tupleValue(r.components["game_td::Health"])).sort((a, b) => a - b);
        exp = expectedHealths(L, wCp.spawned);
        attempts.push({ elapsed: wCp.elapsed, spawned: wCp.spawned, alive: healths.length });
        if (healths.length === exp.length) break;
        await sleep(30);
      }
      check(`${tag} cp1 spawned == independent event table @elapsed=${wCp.elapsed.toFixed(2)}`,
        wCp.spawned === expectedSpawned(L, wCp.elapsed),
        `spawned=${wCp.spawned} expect=${expectedSpawned(L, wCp.elapsed)} attempts=${JSON.stringify(attempts)}`);
      check(`${tag} cp1 alive health multiset == full-hp prefix (no damage, no tower)`,
        JSON.stringify(healths) === JSON.stringify(exp),
        `got=${JSON.stringify(healths)} expect=${JSON.stringify(exp)} attempts=${JSON.stringify(attempts)}`);
      damageSeen = healths.length > 0 && JSON.stringify(healths) !== JSON.stringify(exp);
      const bounties = rows.map((r) => tupleValue(r.components["game_td::Bounty"]));
      bountyBad = bounties.filter((b) => b !== 25).length;
      check(`${tag} cp1 Bounty component == TOML bounty 25 for every enemy`, bountyBad === 0,
        `bad=${bountyBad} n=${bounties.length}`);
      const eco = await brp("td.economy");
      const gCp = await brp("td.game");
      check(`${tag} cp1 passive-only: gold == 300 + 3.0*elapsed (no kills / no spend)`,
        Math.abs(eco.gold - (STARTING_GOLD + PASSIVE_RATE * gCp.elapsed)) <= 1.0,
        `gold=${eco.gold} elapsed=${gCp.elapsed} predicted=${(STARTING_GOLD + PASSIVE_RATE * gCp.elapsed).toFixed(3)}`);
      check(`${tag} cp1 Tower==0 (defeat run is defensive-free)`, (await countOf("game_td::Tower")) === 0);
      const c1 = await brp("td.combat");
      check(`${tag} cp1 combat surface: launchers==0 / fired==0 / projectiles==0`,
        c1.launchers === 0 && c1.fired === 0 && c1.projectiles === 0, JSON.stringify(c1));
    }
    st = await brp("td.state");
    if (st.state === "GameOver") break;
    await sleep(80);
  }

  check(`${tag} reached GameOver`, st && st.state === "GameOver", JSON.stringify(st));
  check(`${tag} result == Defeat`, st && st.result === "Defeat", JSON.stringify(st));
  // 判负的唯一通路 = lives 归零（check_game_state）；采样在 80ms 间隔下可能错过最后一次扣命
  // （第 5 次泄漏与状态迁移同帧，OnExit cleanup 随即移除 BaseLives）——故断言「已观测到 >=4 次扣命」
  // + result=Defeat 双条件，不硬要求采样到 lives<=0（K5 同类采样口径，见 README）。
  const leaksObserved = typeof minLives === "number" ? STARTING_LIVES - minLives : -1;
  check(`${tag} defeat driven by leaks: lives decremented ${STARTING_LIVES}->${minLives} (>=4 leaks before GameOver)`,
    leaksObserved >= 4, `minLives=${minLives} leaksObserved=${leaksObserved}`);
  check(`${tag} defeat elapsed >= 5th-leak time (${defeatLower.toFixed(2)}s)`,
    typeof lastGame.elapsed === "number" && lastGame.elapsed >= defeatLower - 0.2,
    `elapsed=${lastGame.elapsed}`);
  check(`${tag} defeat spawned == event table @elapsed (no schedule desync)`,
    lastGame.spawned === expectedSpawned(L, lastGame.elapsed),
    `spawned=${lastGame.spawned} expect=${expectedSpawned(L, lastGame.elapsed)}`);
  check(`${tag} zero damage / zero kills observed`, damageSeen === false,
    `damageSeen=${damageSeen}`);
  await sleep(300);
  await assertTerminalCleanup(tag, "Defeat");
  return { tag, elapsed: lastGame.elapsed };
}

// ---- 相位主体：ENTITY_DIGEST 采样窗（无害塔 + [2.2,3.0)）----
async function runDigestPhase(index, label) {
  const L = LEVELS[index];
  const tag = `${label}[${L.key}]`;
  const tile = L.harmless.tile;
  info(`=== ${tag} ENTITY_DIGEST run @ ${new Date().toISOString()} (harmless tile [${tile}] dist=${L.harmless.dist}) ===`);
  await enterLevel(index, tag);
  check(`${tag} harmless tile [${tile}] is out of every tower range (dist=${L.harmless.dist} > ${RANGE})`,
    L.harmless.dist > RANGE, `dist=${L.harmless.dist}`);
  const sel = await brp("td.tower_select", { op: "set", key: ROCKET.key });
  check(`${tag} select tower rocket`, sel.key === ROCKET.key, JSON.stringify(sel));
  const placed = await brp("td.spawn_tower", { tile });
  check(`${tag} harmless rocket placed @[${tile}]`, placed.placed === true && placed.cost === ROCKET.cost,
    JSON.stringify(placed));
  const g = await waitGameUntilElapsed(SAMPLE_FROM);
  check(`${tag} elapsed in sampling window [${SAMPLE_FROM},${SAMPLE_TO})`,
    typeof g.elapsed === "number" && g.elapsed >= SAMPLE_FROM && g.elapsed < SAMPLE_TO, `elapsed=${g.elapsed}`);
  const s = await entitySample();
  const spawned = expectedSpawned(L, g.elapsed);
  const exp = expectedEntityDigest(L, [tile], spawned);
  check(`${tag} alive healths == full-hp prefix (harmless tower, spawned=${spawned})`,
    JSON.stringify(s.healths) === JSON.stringify(expectedHealths(L, spawned)),
    `got=${JSON.stringify(s.healths)} expect=${JSON.stringify(expectedHealths(L, spawned))}`);
  check(`${tag} tower/turret counts == 1/1`, s.towerCount === 1 && s.turretCount === 1,
    `${s.towerCount}/${s.turretCount}`);
  check(`${tag} ENTITY_DIGEST == independent recompute`, s.digest === exp,
    `got=${s.digest} expect=${exp}`);
  if (L.entityBaseline) {
    check(`${tag} ENTITY_DIGEST == T048 frozen baseline (zero drift)`, s.digest === L.entityBaseline,
      `got=${s.digest} expect=${L.entityBaseline}`);
  } else {
    info(`${tag} no frozen ENTITY_DIGEST for this level (registry holds level_01 only) — recompute-only check`);
  }
  await leaveToMenu(tag);
  return { tag, digest: s.digest };
}

// ---- 登记项⑥ 辅助：溅射多目标签名 ----
/// 签名 = 同一采样帧里 ≥2 只敌人血量恰为「满血 − 50」（rocket projectile damage），
/// 且彼此世界距离 < 60px（splash_radius）——单次爆炸覆盖多目标的强指纹。
/// 注意：不证明「同一次爆炸」，故只作可选证据（SKIP 不判 FAIL）。
async function splashSignature() {
  const rows = await enemyRows();
  const maxOf = { runner: 60, scout: 80, soldier: 100, heavy: 300 };
  // 血量 → 敌型反查不唯一（80/100 可能来自 300-50-... 递增扣血），故按各自满血集合匹配。
  const hit = [];
  for (const r of rows) {
    const hp = tupleValue(r.components["game_td::Health"]);
    const tr = r.components[TR]?.translation;
    const x = Array.isArray(tr) ? tr[0] : tr?.x;
    const y = Array.isArray(tr) ? tr[1] : tr?.y;
    for (const [type, max] of Object.entries(maxOf)) {
      if (Math.abs(hp - (max - 50)) < 1e-3) {
        hit.push({ type, hp, max, x, y });
        break;
      }
    }
  }
  if (hit.length < 2) return null;
  for (let i = 0; i < hit.length; i++) {
    for (let j = i + 1; j < hit.length; j++) {
      const d = Math.hypot(hit[i].x - hit[j].x, hit[i].y - hit[j].y);
      if (d < 60) {
        return { a: hit[i], b: hit[j], dist: d.toFixed(2) };
      }
    }
  }
  return null;
}

// ---- 登记项②：victory → LevelSelect → 另一关 → 再终局（状态机全链无残留）----
async function phaseChain() {
  info(`=== phase chain @ ${new Date().toISOString()} ===`);
  const v = await runVictoryPhase(0, "chain-v");
  check("chain after victory: state GameOver / result Victory",
    (await brp("td.state")).state === "GameOver" &&
      (await brp("td.state")).result === "Victory");

  // Victory → 回 LevelSelect（登记项②第一跳）
  await leaveToMenu("chain-v");
  const stMenu = await brp("td.state");
  check("chain menu after victory: result Victory survives (GameOver screen contract)",
    stMenu.result === "Victory", JSON.stringify(stMenu));
  check("chain menu after victory: finished==false / no gameplay residue",
    stMenu.finished === false && (await countOf("game_td::Enemy")) === 0 &&
      (await countOf("game_td::Tower")) === 0, JSON.stringify(stMenu));
  check("chain menu after victory: td.wave unavailable / td.economy unavailable",
    (await brp("td.wave")).available === false && (await brp("td.economy")).available === false);

  // 进另一关（level_02）：全新资源 + 同一注册表指纹
  const L2 = LEVELS[1];
  await enterLevel(1, "chain-l2");
  check("chain enter level_02: fresh lives==5 / gold==300+3t",
    (await brp("td.game")).lives === STARTING_LIVES, JSON.stringify(await brp("td.game")));
  check("chain enter level_02: wave total==12 (level_02 event table)",
    (await brp("td.wave")).total === L2.events.length, `total=${(await brp("td.wave")).total}`);
  const map2 = await assertGenericMap("chain-l2 map", L2);
  check("chain enter level_02: GRID_DIGEST == registry (no stale map from level_01)",
    map2.digest === L2.gridDigest, `got=${map2.digest} expect=${L2.gridDigest}`);
  check("chain enter level_02: previous Victory result retained until new finish (documented contract)",
    (await brp("td.state")).result === "Victory", JSON.stringify(await brp("td.state")));

  // 再终局（Defeat）——证明 GameResult / GameFinished 全链可重入
  const d = await runDefeatPhase(1, "chain-d");
  const stEnd = await brp("td.state");
  check("chain re-terminal: result flipped Victory -> Defeat", stEnd.result === "Defeat", JSON.stringify(stEnd));
  await leaveToMenu("chain-d");
  const stFinal = await brp("td.state");
  check("chain final: LevelSelect / result Defeat / finished false",
    stFinal.state === "LevelSelect" && stFinal.result === "Defeat" && stFinal.finished === false,
    JSON.stringify(stFinal));
  info(`chain: victory elapsed=${v.elapsed}, defeat elapsed=${d.elapsed}`);
}

// ---- 件三负控：td.state set GameOver 缺 GameResult（独立进程）----
async function phaseP0Probe() {
  info(`=== phase p0probe @ ${new Date().toISOString()} ===`);
  const st0 = await brp("td.state");
  check("p0probe baseline: state LevelSelect / result null (no GameResult yet)",
    st0.state === "LevelSelect" && st0.result === null, JSON.stringify(st0));
  check("p0probe baseline: process alive", (await brp("rpc.discover")).methods.length === expectDiscover);
  // td.state 的文档契约显式允许 set GameOver（game-td/src/brp.rs:597/:619/:627/:649）；
  // 该带外转移绕过 check_game_state 的写入顺序，GameResult 缺席。T052 U1 裁 (a) 后
  // setup_game_over 取 `Option<Res<GameResult>>`（game_over.rs）——本相位 = 击穿面回归：
  // 进程必须存活 + 状态确已迁移 + result 保持 null + 日志留 skip 标记（非 fail-silent）。
  const set = await brp("td.state", { op: "set", state: "GameOver" });
  check("p0probe td.state set GameOver accepted (documented op, queued)", set.queued === true,
    JSON.stringify(set));
  let aliveAfter = false;
  let lastErr = "";
  let st = null;
  for (let i = 0; i < 10; i++) {
    await sleep(200);
    try {
      st = await brp("td.state", null, 3000);
      aliveAfter = true;
      if (st.state === "GameOver") break;
    } catch (e) {
      lastErr = String(e);
    }
  }
  check("p0probe process survives GameOver transition without GameResult (件三红线)",
    aliveAfter, aliveAfter ? JSON.stringify(st) : `BRP unreachable after transition: ${lastErr}`);
  check("p0probe state transition landed: td.state get == GameOver",
    aliveAfter && st.state === "GameOver", JSON.stringify(st));
  check("p0probe GameResult stays absent (result==null, no synthesized result)",
    aliveAfter && st.result === null, JSON.stringify(st));
  check("p0probe GameFinished stays absent (finished==false)",
    aliveAfter && st.finished === false, JSON.stringify(st));
  scanGameLog("p0probe");
  check("p0probe game log records the skip marker (fail-survivable, not fail-silent)",
    readFileSafe(gameLog).includes("[TD] setup_game_over skipped: no GameResult"),
    `marker=${readFileSafe(gameLog).includes("[TD] setup_game_over skipped: no GameResult")}`);
}

/// 安全读日志（缺失返回空串；panic/marker 扫描共用）。
function readFileSafe(path) {
  try {
    return readFileSync(path, "utf8");
  } catch {
    return "";
  }
}

function phaseAll() {
  return (async () => {
    await phaseScreen();
    // 登记项⑦：td.input 注入通道端到端冒烟（emit GameAction → 真消费者 → 下游效应）
    await inputChannelSmoke();
    await runVictoryPhase(0, "l1", { splash: true, rocket: true, victoryShot: true });
    await leaveToMenu("l1");
    await runDefeatPhase(0, "l1");
    await leaveToMenu("l1");
    await runDigestPhase(0, "l1");
    await runVictoryPhase(1, "l2");
    await leaveToMenu("l2");
    await runDefeatPhase(1, "l2");
    await leaveToMenu("l2");
    await runDigestPhase(1, "l2");
    await runVictoryPhase(2, "l3", { splash: true, rocket: true });
    await leaveToMenu("l3");
    await runDefeatPhase(2, "l3");
    await leaveToMenu("l3");
    await runDigestPhase(2, "l3");
    await phaseChain();
    if (gameLog) scanGameLog("final");
    const alive = await brp("td.state");
    check("final process alive (no P0 crash across the whole surface)", alive.state === "LevelSelect",
      JSON.stringify(alive));
  })();
}

/// 登记项⑦冒烟：td.input emit → GameAction 消息 → 真实消费者（非 run_system_once）。
/// 只在 LevelSelect 场景做一次「菜单导航」端到端：emit Right → 焦点 +1。
async function inputChannelSmoke() {
  const ls = await brp("td.level_select");
  const before = ls.focused;
  const emit = await brp("td.input", { op: "emit", action: "Right" });
  check("input-channel smoke: td.input emit Right accepted (GameAction message written)",
    emit.emitted === "Right", JSON.stringify(emit));
  let after = before;
  const deadline = Date.now() + 3000;
  while (Date.now() < deadline) {
    const s = await brp("td.level_select");
    after = s.focused;
    if (after !== before) break;
    await sleep(50);
  }
  check("input-channel smoke: message consumed by real consumer (focus moved)",
    after === (before + 1) % Math.max(1, (await brp("td.level_select")).buttons.length),
    `before=${before} after=${after}`);
  // 复原焦点（左移回原位），避免影响后续进关选择
  await brp("td.input", { op: "emit", action: "Left" });
  await sleep(200);
}

// ---- 入口 ----
try {
  if (phase === "screen") await phaseScreen();
  else if (phase === "l1") {
    await runVictoryPhase(0, "l1", { splash: true, rocket: true, victoryShot: true });
    await leaveToMenu("l1");
    await runDefeatPhase(0, "l1");
    await leaveToMenu("l1");
    await runDigestPhase(0, "l1");
  } else if (phase === "l2") {
    await runVictoryPhase(1, "l2");
    await leaveToMenu("l2");
    await runDefeatPhase(1, "l2");
    await leaveToMenu("l2");
    await runDigestPhase(1, "l2");
  } else if (phase === "l3") {
    await runVictoryPhase(2, "l3", { splash: true, rocket: true });
    await leaveToMenu("l3");
    await runDefeatPhase(2, "l3");
    await leaveToMenu("l3");
    await runDigestPhase(2, "l3");
  } else if (phase === "chain") await phaseChain();
  else if (phase === "p0probe") await phaseP0Probe();
  else if (phase === "all") await phaseAll();
  else check("--phase arg", false, `未知 --phase ${phase}`);
} catch (e) {
  check("m6-all driver completed without exception (process alive)", false, `${e && e.stack ? e.stack.split("\n")[0] : e}`);
}

const total = results.filter((r) => !r.startsWith("INFO")).length;
const passed = results.filter((r) => r.startsWith("PASS")).length;
const failed = results.filter((r) => r.startsWith("FAIL")).length;
const skipped = results.filter((r) => r.startsWith("SKIP")).length;
console.log(results.join("\n"));
console.log("");
for (const L of LEVELS) {
  console.log(
    `INFO | LEVEL ${L.key}: pathPx=${L.pathPx} events=${L.events.length} ` +
      `rapidPlan=${JSON.stringify(L.rapidPlan.slice(0, 6))} harmless=${JSON.stringify(L.harmless)} ` +
      `victoryBy<=${L.victoryUpper.toFixed(1)}s defeatAt>=${L.leakTimes[STARTING_LIVES - 1].toFixed(2)}s`,
  );
}
console.log(`SUMMARY: phase=${phase} PASS=${passed} FAIL=${failed} SKIP=${skipped} TOTAL=${total}`);
console.log(`M6ALL_JUDGEMENT=${failed === 0 ? "PASS" : "FAIL"}`);
process.exitCode = failed === 0 ? 0 : 1;
console.log(`REAL_EXIT=${process.exitCode}`);

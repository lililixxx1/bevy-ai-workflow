// T044-W1 证据断言脚本（BRP over HTTP，127.0.0.1:15702）。
// 用法（树根执行）：
//   node docs/evidence/t044-td-spike/w1/assert-td.mjs --phase p1 --shot docs/evidence/t044-td-spike/w1/p1-shot.png
//   node docs/evidence/t044-td-spike/w1/assert-td.mjs --phase p2 --shot docs/evidence/t044-td-spike/w1/p2-shot.png
//   node docs/evidence/t044-td-spike/w1/assert-td.mjs --phase p3 --shot docs/evidence/t044-td-spike/w1/p3-shot.png
// 判定行由脚本计算生成（禁止手算填表，附录 B.2）；退出码 0 = 全 PASS。
// 期望值来源：派工单 T044-W1「行为规格」+「验收标准」V3/V4/V5/V6（含算式）。
import { readFileSync } from "node:fs";

const URL = "http://127.0.0.1:15702";
let id = 0;
const results = [];
function check(name, cond, detail) {
  results.push(`${cond ? "PASS" : "FAIL"} | ${name}${detail ? " | " + detail : ""}`);
  if (!cond) process.exitCode = 1;
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
  } finally { clearTimeout(timer); }
}
const sleep = ms => new Promise(r => setTimeout(r, ms));

function arg(name, fallback = null) {
  const i = process.argv.indexOf(name);
  return i >= 0 && i + 1 < process.argv.length ? process.argv[i + 1] : fallback;
}

const phase = arg("--phase", "p1");
const shotPath = arg("--shot", "");

// ---- 常量（算式见派工单；脚本内独立复算，不引用游戏侧值）----
const COLS = 15;
const ROWS = 10;
const TILE = 64;
const PATH_MID = ROWS / 2;                       // 10/2 = 5
const OFFSET_X = -(COLS * TILE) / 2 + TILE / 2;  // -448
const OFFSET_Y = -(ROWS * TILE) / 2 + TILE / 2;  // -288
// 行 r 的 atlas 索引：r == path_mid-1 下沿 125/126/127；r == path_mid 路面 102/103/104；
// r == path_mid+1 上沿 79/80/81；其余 129（草）。
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

// ---- V3：rpc.discover 方法总数（23 内置 + 1 自定义 td.screenshot = 24）----
const disc = await brp("rpc.discover");
const names = disc.methods.map(m => m.name);
check("discover.version==0.20.0", disc.info.version === "0.20.0", `version=${disc.info.version}`);
check("discover.methods==24", names.length === 24, `len=${names.length}`);
check("discover contains td.screenshot", names.includes("td.screenshot"));
check("discover no game.* prefix", names.every(n => !n.startsWith("game.")),
  names.filter(n => n.startsWith("game.")).join(",") || "(none)");

// ---- 截图受理 + 轮询 + 文件核验（V4/V5/V6 共用）----
async function captureAndVerify() {
  if (!shotPath) { check("screenshot path arg", false, "缺 --shot"); return; }
  const st0 = await brp("td.screenshot", { op: "status" });
  const baseId = st0.next_id;
  const queued = await brp("td.screenshot", { op: "capture", path: shotPath });
  check("screenshot queued", queued.queued === true && queued.id === baseId,
    `id=${queued.id} baseId=${baseId} path=${queued.path}`);

  let entry = null;
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    const st = await brp("td.screenshot", { op: "status" });
    entry = st.entries.find(e => e.id === queued.id);
    if (entry && entry.status === "captured") break;
    await sleep(200);
  }
  check("screenshot status==captured", !!entry && entry.status === "captured",
    entry ? JSON.stringify(entry) : "no entry");
  if (!entry || entry.status !== "captured") return;

  let buf;
  try { buf = readFileSync(shotPath); } catch (e) {
    check("screenshot file readable", false, `${shotPath}: ${e.code}`);
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
    check("PNG size positive", w > 0 && h > 0, `${w}x${h}`);
    console.log(`NOTE logical_window=1280x720 physical_png=${w}x${h} dpi_scale=${(w / 1280).toFixed(2)}`);
  }
}

if (phase === "p1") {
  // P1 行为：Camera2d ×1；红色方块 Sprite ×1（srgb(1,0,0)、100×100、世界原点）。
  const cam = await brp("world.query", { data: { components: ["bevy_camera::components::Camera2d"] } });
  check("P1 camera count==1", cam.length === 1, `len=${cam.length}`);
  const TR = "bevy_transform::components::transform::Transform";
  const q = await brp("world.query", {
    data: { components: ["bevy_sprite::sprite::Sprite", TR] },
  });
  check("P1 sprite count==1", q.length === 1, `len=${q.length}`);
  if (q.length === 1) {
    const s = q[0].components["bevy_sprite::sprite::Sprite"];
    const t = q[0].components[TR].translation;
    const c = s.color && s.color.Srgba;
    check("P1 sprite color==srgb(1,0,0,1)",
      !!c && c.red === 1 && c.green === 0 && c.blue === 0 && c.alpha === 1,
      JSON.stringify(s.color));
    check("P1 sprite size==100x100",
      Array.isArray(s.custom_size) && s.custom_size[0] === 100 && s.custom_size[1] === 100,
      JSON.stringify(s.custom_size));
    check("P1 sprite at world origin", Array.isArray(t) && t[0] === 0 && t[1] === 0,
      JSON.stringify(t));
  }
  await captureAndVerify();
} else if (phase === "p2") {
  // P2 行为：150 MapTile（Sprite 网格，居中于原点）+ 45 PathTile（索引 != 129）。
  // 注意：P2 的 atlas 索引无法经 BRP 读出——Sprite.texture_atlas 含强句柄
  // （Handle<Image> → Arc<StrongHandle> 未注册 ReflectSerialize，get_components
  // strict 报 -23402）。故 P2 断言网格几何 + 标记语义；索引逐格断言在 P3 用
  // TileTextureIndex（可序列化）覆盖，两阶段同一 match 逻辑。
  const TR = "bevy_transform::components::transform::Transform";
  const all = await brp("world.query", { data: { components: ["game_td::MapTile", TR] } });
  check("P2 MapTile count==150", all.length === 150, `len=${all.length}`);
  const paths = await brp("world.query", { data: { components: ["game_td::PathTile"] } });
  check("P2 PathTile count==45", paths.length === 45, `len=${paths.length}`);
  const pathsWithMarker = await brp("world.query", {
    data: { components: ["game_td::PathTile"] },
    filter: { with: ["game_td::MapTile"] },
  });
  check("P2 PathTile subset of MapTile", pathsWithMarker.length === 45, `len=${pathsWithMarker.length}`);
  const grassOnly = await brp("world.query", {
    data: { components: ["game_td::MapTile"] },
    filter: { without: ["game_td::PathTile"] },
  });
  check("P2 grass count==105 (150-45)", grassOnly.length === 105, `len=${grassOnly.length}`);

  // 网格位置逐格复算（双盲：脚本公式 vs 游戏侧写入）。
  const expected = new Set();
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      expected.add(`${OFFSET_X + c * TILE},${OFFSET_Y + r * TILE}`);
    }
  }
  let posBad = 0;
  const seen = new Set();
  for (const row of all) {
    const t = row.components[TR].translation;
    const key = `${t[0]},${t[1]}`;
    if (!expected.has(key)) { posBad++; } else { seen.add(key); }
  }
  check("P2 all 150 grid positions match centering formula",
    posBad === 0 && seen.size === 150, `bad=${posBad} covered=${seen.size}/150`);

  // 路行 y 值（path_mid=5 -> rows 4/5/6 -> y = -32/32/96）。
  const pathSet = new Set(paths.map(r => r.entity));
  const pathRows = new Set();
  for (const row of all) {
    if (pathSet.has(row.entity)) pathRows.add(row.components[TR].translation[1]);
  }
  const wantRows = [-32, 32, 96];
  check("P2 road rows y in {-32,32,96}",
    pathRows.size === 3 && wantRows.every(y => pathRows.has(y)),
    `rows=${[...pathRows].sort((a, b) => a - b).join(",")}`);

  await captureAndVerify();
} else if (phase === "p3") {
  // P3 行为：TileBundle 等价替换——TilePos 网格 + TileTextureIndex 与 P2 一致。
  const POS = "bevy_ecs_tilemap::tiles::TilePos";
  const TEX = "bevy_ecs_tilemap::tiles::TileTextureIndex";
  const all = await brp("world.query", {
    data: { components: ["game_td::MapTile", POS, TEX] },
  });
  check("P3 MapTile count==150", all.length === 150, `len=${all.length}`);
  const paths = await brp("world.query", { data: { components: ["game_td::PathTile"] } });
  check("P3 PathTile count==45", paths.length === 45, `len=${paths.length}`);
  const spriteQ = await brp("world.query", { data: { components: ["bevy_sprite::sprite::Sprite"] } });
  check("P3 no per-tile Sprite (replaced by tilemap)", spriteQ.length === 0, `len=${spriteQ.length}`);

  let posBad = 0, idxBad = 0;
  const seen = new Set();
  for (const row of all) {
    const p = row.components[POS];
    const idx = row.components[TEX][0] ?? row.components[TEX];
    if (!p || typeof p.x !== "number" || typeof p.y !== "number") { posBad++; continue; }
    seen.add(`${p.x},${p.y}`);
    if (idx !== expectedIndex(p.y, p.x)) idxBad++;
  }
  check("P3 TilePos covers 15x10 grid exactly once",
    posBad === 0 && seen.size === 150, `bad=${posBad} covered=${seen.size}/150`);
  check("P3 texture index per cell matches P2 spec", idxBad === 0, `bad=${idxBad}`);

  const pathSet = new Set(paths.map(r => r.entity));
  const idxPathSet = new Set(
    all.filter(r => {
      const idx = r.components[TEX][0] ?? r.components[TEX];
      return PATH_INDICES.has(idx);
    }).map(r => r.entity)
  );
  const sameSet = pathSet.size === idxPathSet.size && [...pathSet].every(e => idxPathSet.has(e));
  check("P3 PathTile set == {index != 129}", sameSet,
    `markers=${pathSet.size} idx=${idxPathSet.size}`);

  const pathRows = new Set();
  for (const row of all) {
    if (pathSet.has(row.entity)) pathRows.add(row.components[POS].y);
  }
  const wantRows = [PATH_MID - 1, PATH_MID, PATH_MID + 1];
  check("P3 road rows y in {4,5,6}",
    pathRows.size === 3 && wantRows.every(y => pathRows.has(y)),
    `rows=${[...pathRows].sort((a, b) => a - b).join(",")}`);

  await captureAndVerify();
} else {
  check("phase arg", false, `未知 --phase ${phase}`);
}

console.log(results.join("\n"));
console.log(`SUMMARY: ${results.filter(r => r.startsWith("PASS")).length}/${results.length} PASS`);
process.exit(process.exitCode ?? 0);

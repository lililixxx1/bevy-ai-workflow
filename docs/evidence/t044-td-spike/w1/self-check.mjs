// T044-W1 证据档自检（V8；附录 G）。树根执行：
//   node docs/evidence/t044-td-spike/w1/self-check.mjs
// 输出逐项 PASS/FAIL；末行 SELF_CHECK_PASS / SELF_CHECK_FAIL；FAIL 时退出码 1。
// 判定行全部由脚本计算生成（禁止手算填表，附录 B.2）。
import { readFileSync, existsSync, readdirSync } from "node:fs";
import { createHash } from "node:crypto";

const DIR = "docs/evidence/t044-td-spike/w1";
const results = [];
function check(name, cond, detail) {
  results.push(`${cond ? "PASS" : "FAIL"} | ${name}${detail ? " | " + detail : ""}`);
  if (!cond) process.exitCode = 1;
}
const read = (f) => readFileSync(`${DIR}/${f}`, "utf8");

// ---- 1. 必备文件齐备 ----
const required = [
  "README.md", "assert-td.mjs", "self-check.mjs", "timing.md",
  "v1-check-p1.log", "v1-check-final.log", "v2-check-workspace.log", "v2-check-final.log",
  "p1-build.log", "p1-run.log", "p1-shot.png", "discover-p1.json", "v4-assert-p1.log",
  "p2-check.log", "p2-build.log", "p2-run.log", "p2-shot.png", "discover-p2.json",
  "v5-assert-p2.log",
  "p3-check.log", "p3-build.log", "p3-run.log", "p3-shot.png", "discover-p3.json",
  "v6-assert-p3.log", "v3-discover.log", "v6b-assert-p3-final.log", "p3-final-run.log",
  "p3-final-shot.png",
  "v7-taskkill.log", "v7-tasklist.log", "v7-tasklist-full.log",
];
const missing = required.filter((f) => !existsSync(`${DIR}/${f}`));
check("必备文件齐备", missing.length === 0, missing.length ? `missing=${missing.join(",")}` : `count=${required.length}`);

// ---- 2. 断言档 SUMMARY 自洽（判定行由脚本计算）----
const assertLogs = ["v4-assert-p1.log", "v5-assert-p2.log", "v6-assert-p3.log", "v6b-assert-p3-final.log"];
for (const f of assertLogs) {
  if (!existsSync(`${DIR}/${f}`)) continue;
  const s = read(f);
  const m = s.match(/SUMMARY: (\d+)\/(\d+) PASS/);
  const ok = !!m && Number(m[1]) === Number(m[2]) && Number(m[2]) > 0;
  check(`${f} SUMMARY 全 PASS`, ok, m ? `${m[1]}/${m[2]}` : "无 SUMMARY 行");
}

// ---- 3. 每个 .log 含退出码/证据行（排除自检自身的输出档：其退出码由外层追加）----
const logs = readdirSync(DIR).filter((f) => f.endsWith(".log") && !/^v8-selfcheck/.test(f));
const noExit = logs.filter((f) => !/(REAL_EXIT|CURL_EXIT|TASKKILL_EXIT|TASKLIST_EXIT|RESIDUE_COUNT)=/m.test(read(f)));
check("每个 .log 含退出码/证据行", noExit.length === 0,
  noExit.length ? `missing_in=${noExit.join(",")}` : `logs=${logs.length}`);

// ---- 4. 截图 PNG：魔数 + IHDR 物理尺寸 ----
const pngs = ["p1-shot.png", "p2-shot.png", "p3-shot.png", "p3-final-shot.png"];
for (const f of pngs) {
  const buf = readFileSync(`${DIR}/${f}`);
  const magic = buf.subarray(0, 8).toString("hex");
  const w = buf.readUInt32BE(16);
  const h = buf.readUInt32BE(20);
  check(`${f} PNG 魔数/尺寸`, magic === "89504e470d0a1a0a" && w > 0 && h > 0,
    `magic=${magic} IHDR=${w}x${h} bytes=${buf.length}`);
}

// ---- 5. discover 方法数 24（三阶段原始响应）----
for (const f of ["discover-p1.json", "discover-p2.json", "discover-p3.json"]) {
  const j = JSON.parse(read(f));
  const names = j.result.methods.map((m) => m.name);
  check(`${f} methods==24 + td.screenshot`,
    names.length === 24 && names.includes("td.screenshot"),
    `len=${names.length} custom=${names.filter((n) => n.startsWith("td.")).length}`);
}

// ---- 6. 零机器绝对路径（log/json/md/mjs 全扫）----
const BS = String.fromCharCode(92);
const absPatterns = [
  ["C:", "Users", "Administrator"].join(BS),
  ["C:", "Users"].join("/"),
  ["", "c", "Users", ""].join("/"),
];
const textFiles = readdirSync(DIR).filter((f) => /\.(log|json|md|mjs)$/.test(f));
const absHits = [];
for (const f of textFiles) {
  const s = readFileSync(`${DIR}/${f}`, "utf8");
  for (const p of absPatterns) if (s.includes(p)) absHits.push(`${f}:${p}`);
}
check("零机器绝对路径", absHits.length === 0, absHits.length ? absHits.join(",") : `scanned=${textFiles.length}`);

// ---- 7. 素材完整性（树内 game-td assets，sha256 前 16 位 + 字节数）----
const asset = "game-td/assets/Tilesheet/towerDefense_tilesheet.png";
if (existsSync(asset)) {
  const buf = readFileSync(asset);
  const sha = createHash("sha256").update(buf).digest("hex");
  check("tilesheet sha256/字节数", sha.startsWith("3681f42ae963aa4e") && buf.length === 91302,
    `sha256=${sha.slice(0, 16)}... bytes=${buf.length}`);
} else {
  check("tilesheet 存在", false, asset);
}

// ---- 8. 总账判定行存在 ----
if (existsSync(`${DIR}/README.md`)) {
  const s = read("README.md");
  check("README 含 tilemap 判定行", s.includes("tilemap-0.20: PASS"));
  check("README 含验收 V1-V8 索引", ["V1", "V2", "V3", "V4", "V5", "V6", "V7", "V8"].every((v) => s.includes(v)));
} else {
  check("README 存在", false);
}

// ---- 汇总 ----
console.log(results.join("\n"));
const pass = results.filter((r) => r.startsWith("PASS")).length;
console.log(`SUMMARY: ${pass}/${results.length} PASS`);
const allPass = pass === results.length;
console.log(allPass ? "SELF_CHECK_PASS" : "SELF_CHECK_FAIL");
process.exit(process.exitCode ?? 0);

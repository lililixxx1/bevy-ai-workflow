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

const cfg = await brp("world.get_resources", { resource: "game::sim::SimConfig" });
check("SimConfig readback", cfg.value.entity_count === 1000 && cfg.value.seed === 20260926 && cfg.value.paused === false, JSON.stringify(cfg.value));
const s1 = await brp("world.get_resources", { resource: "game::sim::SimStats" });
await sleep(1100);
const s2 = await brp("world.get_resources", { resource: "game::sim::SimStats" });
check("tick increases", s2.value.tick > s1.value.tick, `tick ${s1.value.tick} -> ${s2.value.tick}`);
const q = await brp("world.query", { data: { components: ["game::sim::Wanderer"] } });
check("Wanderer count==1000", q.length === 1000, `len=${q.length}`);
const e = q.find(r => r.components["game::sim::Wanderer"].index === 0).entity;
const TR = "bevy_transform::components::transform::Transform";
const t1 = await brp("world.get_components", { entity: e, components: [TR], strict: true });
await sleep(600);
const t2 = await brp("world.get_components", { entity: e, components: [TR], strict: true });
check("entity position changes per frame", JSON.stringify(t1[TR].translation) !== JSON.stringify(t2[TR].translation),
  `${JSON.stringify(t1[TR].translation)} -> ${JSON.stringify(t2[TR].translation)}`);
const s3 = await brp("world.get_resources", { resource: "game::sim::SimStats" });
await brp("world.mutate_resources", { resource: "game::sim::SimConfig", path: "paused", value: true });
await sleep(1100);
const s4 = await brp("world.get_resources", { resource: "game::sim::SimStats" });
check("paused: tick frozen (<=2 frame boundary)", s4.value.tick - s3.value.tick <= 2, `${s3.value.tick} == ${s4.value.tick}`);
await brp("world.mutate_resources", { resource: "game::sim::SimConfig", path: "paused", value: false });
await sleep(1100);
const s5 = await brp("world.get_resources", { resource: "game::sim::SimStats" });
check("resumed: tick increases", s5.value.tick > s4.value.tick, `${s4.value.tick} -> ${s5.value.tick}`);
const sp = await brp("world.spawn_entity", { components: { "game::sim::Velocity": { linear: [1.0, 0.0, 2.0] } } });
const gv = await brp("world.get_components", { entity: sp.entity, components: ["game::sim::Velocity"], strict: true });
check("spawn+readback Velocity", JSON.stringify(gv["game::sim::Velocity"].linear) === JSON.stringify([1, 0, 2]),
  JSON.stringify(gv["game::sim::Velocity"].linear));
await brp("world.despawn_entity", { entity: sp.entity });
const q3 = await brp("world.query", { data: { components: ["game::sim::Wanderer"] } });
check("despawn: count back to 1000", q3.length === 1000, `len=${q3.length}`);
const qcam = await brp("world.query", { data: { components: ["bevy_camera::camera::Camera", TR] } });
const camTr = qcam[0].components[TR].translation;
const dist = Math.hypot(camTr[0], camTr[2]);
check("camera orbit radius~=90", qcam.length === 1 && Math.abs(dist - 90) < 0.1, `len=${qcam.length} dist=${dist.toFixed(2)}`);
const lc = await brp("world.list_components");
check("list_components has custom types", lc.includes("game::sim::Wanderer") && lc.includes("game::sim::Velocity"));
// 解析式运动数学断言（TS-07 精神）: p = origin + v*t + 0.8*sin(2pi*0.7*t+phase)
const qf = await brp("world.query", { data: { components: ["game::sim::Wanderer", "game::sim::Velocity", TR] } });
const row = qf.find(r => r.components["game::sim::Wanderer"].index === 0);
const st = await brp("world.get_resources", { resource: "game::sim::SimStats" });
const t = st.value.elapsed_secs;
const w = row.components["game::sim::Wanderer"], v = row.components["game::sim::Velocity"].linear, p = row.components[TR].translation;
const ex = [
  w.origin[0] + v[0] * t + 0,
  w.origin[1] + 0.8 * Math.sin(2 * Math.PI * 0.7 * t + w.phase),
  w.origin[2] + v[2] * t + 0,
];
const err = Math.max(Math.abs(p[0] - ex[0]), Math.abs(p[1] - ex[1]), Math.abs(p[2] - ex[2]));
check("analytic motion math assert |p-f(t)|<0.1", err < 0.1, `t=${t.toFixed(2)} err=${err.toExponential(2)}`);
console.log(results.join("\n"));
console.log(`SUMMARY: ${results.filter(r => r.startsWith("PASS")).length}/${results.length} PASS`);
process.exit(process.exitCode ?? 0);

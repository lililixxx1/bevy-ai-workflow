// T041 全链路 MCP 驱动（C1–C8）。
//
// 用法（在仓库根执行，且 CARGO_TARGET_DIR 指向共享编译缓存）：
//   node docs/evidence/t041-mcp-bridge/mcp-drive.mjs
//
// 行为：按 .mcp.json 注册条目逐字 spawn 桥（command + args，cwd = 仓库根）→
// spawn `cargo run --release -p game -j 3` → 轮询 rpc.discover 就绪 → 跑断言 →
// 关闭桥 → taskkill 清杀游戏 → 核残留。判定行由本脚本计算输出，人工不填表。
//
// 证据零机器绝对路径：本脚本只使用相对路径；transcript 只落 MCP 消息原文。

import { spawn, spawnSync } from "node:child_process";
import {
  appendFileSync,
  createWriteStream,
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { setTimeout as sleep } from "node:timers/promises";

const EVID = "docs/evidence/t041-mcp-bridge";
const LOG = `${EVID}/mcp-drive.log`;
const TRANSCRIPT = `${EVID}/transcript.jsonl`;
const BRP_URL = "http://127.0.0.1:15702";

const EXPECTED_TOOLS = [
  "game.run_tests",
  "game.screenshot",
  "game.screenshot_log",
  "game.launch_level",
  "game.move_unit",
  "game.attack",
  "game.end_turn",
  "game.snapshot",
  "rpc.discover",
  "world.query",
  "world.get_components",
  "registry.schema",
];

const failures = [];
let checks = 0;

function log(line) {
  appendFileSync(LOG, `${line}\n`);
  console.log(line);
}

function check(name, condition, detail) {
  checks += 1;
  const ok = Boolean(condition);
  log(`[${ok ? "PASS" : "FAIL"}] ${name}: ${detail}`);
  if (!ok) failures.push(`${name}: ${detail}`);
  return ok;
}

function writeJson(name, value) {
  writeFileSync(`${EVID}/${name}`, `${JSON.stringify(value, null, 2)}\n`);
}

function must(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

class McpClient {
  constructor(entry) {
    this.nextId = 1;
    this.pending = new Map();
    this.messages = [];
    this.buffer = "";
    this.protocolErrors = [];
    this.transcript = createWriteStream(TRANSCRIPT, { flags: "w" });
    this.stderrLog = createWriteStream(`${EVID}/mcp-bridge-stderr.log`, { flags: "w" });
    this.child = spawn(entry.command, entry.args, {
      cwd: process.cwd(),
      env: { ...process.env },
      stdio: ["pipe", "pipe", "pipe"],
    });
    this.child.stdout.setEncoding("utf8");
    this.child.stderr.setEncoding("utf8");
    this.child.stdout.on("data", (chunk) => this.onStdout(chunk));
    this.child.stderr.on("data", (chunk) => this.stderrLog.write(chunk));
    this.child.on("error", (error) => {
      for (const waiter of this.pending.values()) {
        clearTimeout(waiter.timer);
        waiter.reject(error);
      }
      this.pending.clear();
    });
  }

  onStdout(chunk) {
    this.buffer += chunk;
    let index;
    while ((index = this.buffer.indexOf("\n")) >= 0) {
      const raw = this.buffer.slice(0, index).replace(/\r$/, "");
      this.buffer = this.buffer.slice(index + 1);
      if (!raw.trim()) continue;
      this.transcript.write(`${raw}\n`);
      let message;
      try {
        message = JSON.parse(raw);
      } catch (error) {
        this.protocolErrors.push(`非 JSON 响应行（${error.message}）：${raw.slice(0, 200)}`);
        continue;
      }
      this.messages.push(message);
      if (
        Object.prototype.hasOwnProperty.call(message, "id") &&
        (Object.prototype.hasOwnProperty.call(message, "result") ||
          Object.prototype.hasOwnProperty.call(message, "error"))
      ) {
        const waiter = this.pending.get(message.id);
        if (waiter) {
          clearTimeout(waiter.timer);
          this.pending.delete(message.id);
          waiter.resolve(message);
        }
      }
    }
  }

  sendRaw(message) {
    const line = JSON.stringify(message);
    this.transcript.write(`${line}\n`);
    this.child.stdin.write(`${line}\n`);
    return line;
  }

  request(method, params, timeoutMs = 60000) {
    const id = this.nextId;
    this.nextId += 1;
    const message = { jsonrpc: "2.0", id, method };
    if (params !== undefined) message.params = params;
    this.sendRaw(message);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`超时等待 ${method}（id=${id}，${timeoutMs}ms）`));
      }, timeoutMs);
      this.pending.set(id, { resolve, reject, timer });
    });
  }

  notify(method, params) {
    const message = { jsonrpc: "2.0", method };
    if (params !== undefined) message.params = params;
    this.sendRaw(message);
  }

  async close() {
    await new Promise((resolve) => this.child.stdin.end(resolve));
    await Promise.race([
      new Promise((resolve) => this.child.once("exit", resolve)),
      sleep(5000),
    ]);
    if (this.child.exitCode === null) {
      this.child.kill();
      await sleep(500);
    }
    await new Promise((resolve) => this.transcript.end(resolve));
    await new Promise((resolve) => this.stderrLog.end(resolve));
  }
}

async function brpPost(bodyText) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), 5000);
  try {
    const response = await fetch(BRP_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: bodyText,
      signal: controller.signal,
    });
    return { status: response.status, text: await response.text() };
  } finally {
    clearTimeout(timer);
  }
}

function brpDiscoverBody() {
  return JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "rpc.discover",
  });
}

async function waitForBrpReady(timeoutMs = 180000) {
  const started = Date.now();
  let attempts = 0;
  while (Date.now() - started < timeoutMs) {
    attempts += 1;
    try {
      const response = await brpPost(brpDiscoverBody());
      if (response.status === 200) {
        const parsed = JSON.parse(response.text);
        if (Array.isArray(parsed.result?.methods)) {
          return { attempts, elapsedMs: Date.now() - started };
        }
      }
    } catch {
      // 未就绪：继续轮询。
    }
    await sleep(500);
  }
  throw new Error(`BRP 就绪轮询超时（${attempts} 次，${timeoutMs}ms）`);
}

function readSuiteNames() {
  const source = readFileSync("game/src/rpc/suites/mod.rs", "utf8");
  const names = [];
  const pattern = /\(\s*"(ts-\d+)"\s*,\s*ts\d+::run\s*\)/g;
  let match;
  while ((match = pattern.exec(source)) !== null) {
    names.push(match[1]);
  }
  return names;
}

function readClaimLintExpect() {
  const source = readFileSync("tooling/claim-lint/src/main.rs", "utf8");
  const number = (name) => {
    const match = source.match(new RegExp(`${name}:\\s*usize\\s*=\\s*(\\d+)`));
    return match ? Number(match[1]) : null;
  };
  return {
    gameMethods: number("GAME_METHODS"),
    inProcSuites: number("IN_PROC_SUITES"),
    brpBuiltin: number("BRP_BUILTIN"),
  };
}

function pngHeaderOk(path) {
  const bytes = readFileSync(path);
  const magic = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  return {
    bytes: bytes.length,
    magic: bytes.subarray(0, 8).equals(magic),
  };
}

let game = null;
let gameLog = null;
let mcp = null;

async function run() {
  must(existsSync("Cargo.toml") && existsSync(".mcp.json"), "cwd 须为仓库根（含 Cargo.toml 与 .mcp.json）");
  must(Boolean(process.env.CARGO_TARGET_DIR), "环境变量 CARGO_TARGET_DIR 未设置（共享编译缓存条款）");
  mkdirSync(EVID, { recursive: true });
  writeFileSync(LOG, "");

  const mcpConfig = JSON.parse(readFileSync(".mcp.json", "utf8"));
  const entry = mcpConfig.mcpServers?.["bevy-brp-bridge"];
  must(entry && typeof entry.command === "string" && Array.isArray(entry.args), ".mcp.json 缺 bevy-brp-bridge 注册条目");
  log(`.mcp.json 注册条目（驱动 spawn 逐字同源）：command=${JSON.stringify(entry.command)} args=${JSON.stringify(entry.args)}`);

  // —— 游戏侧：先确认端口空闲，再后台启动并轮询就绪 ——
  let portBusy = false;
  try {
    const probe = await brpPost(brpDiscoverBody());
    portBusy = probe.status === 200;
  } catch {
    portBusy = false;
  }
  must(!portBusy, "端口 15702 已有 BRP 服务在跑（请先清场，不得复用他人进程）");

  gameLog = createWriteStream(`${EVID}/game-run.log`, { flags: "w" });
  // release 冷编 -j 1（2026-10-28 commit 耗尽事故整改裁决，见 ops-incident-remediation.md §三——勿改回 -j 3）
  gameLog.write("$ cargo run --release -p game -j 1\n");
  game = spawn("cargo", ["run", "--release", "-p", "game", "-j", "1"], {
    cwd: process.cwd(),
    env: { ...process.env },
    stdio: ["ignore", "pipe", "pipe"],
  });
  game.stdout.pipe(gameLog);
  game.stderr.pipe(gameLog);
  game.on("exit", (code) => {
    gameLog.write(`\nREAL_EXIT=${code === null ? "null" : code}\n`);
  });

  const ready = await waitForBrpReady();
  appendFileSync(
    `${EVID}/game-readiness.log`,
    `轮询 rpc.discover 就绪：attempts=${ready.attempts} elapsed_ms=${ready.elapsedMs}\nREAL_EXIT=0\n`,
  );
  log(`BRP 就绪：attempts=${ready.attempts} elapsed_ms=${ready.elapsedMs}`);

  // —— MCP 客户端（spawn 形态 = .mcp.json 条目）——
  mcp = new McpClient(entry);

  // C1：initialize 回显 + capabilities.tools + 通知无响应 + ping
  const initialize = await mcp.request("initialize", {
    protocolVersion: "2025-06-18",
    capabilities: {},
    clientInfo: { name: "t041-mcp-drive", version: "1.0.0" },
  });
  check(
    "C1.initialize.protocolVersion 回显",
    initialize.result?.protocolVersion === "2025-06-18",
    `实际 ${JSON.stringify(initialize.result?.protocolVersion)}`,
  );
  check(
    "C1.initialize.capabilities.tools 存在",
    initialize.result?.capabilities && Object.prototype.hasOwnProperty.call(initialize.result.capabilities, "tools"),
    `capabilities=${JSON.stringify(initialize.result?.capabilities)}`,
  );
  check(
    "C1.initialize.serverInfo",
    typeof initialize.result?.serverInfo?.name === "string" &&
      typeof initialize.result?.serverInfo?.version === "string",
    `serverInfo=${JSON.stringify(initialize.result?.serverInfo)}`,
  );

  const beforeNotification = mcp.messages.length;
  mcp.notify("notifications/initialized", {});
  const ping = await mcp.request("ping", {}, 10000);
  await sleep(200);
  const afterNotification = mcp.messages.slice(beforeNotification);
  check(
    "C1.notifications/initialized 无响应",
    afterNotification.length === 1 && afterNotification[0].id === ping.id,
    `通知后到 ping 响应之间收到 ${afterNotification.length} 条（期望仅 ping 一条）`,
  );
  check("C1.ping result 为空对象", JSON.stringify(ping.result) === "{}", `result=${JSON.stringify(ping.result)}`);
  writeJson("c1-initialize.json", {
    initialize_request: {
      protocolVersion: "2025-06-18",
      capabilities: {},
      clientInfo: { name: "t041-mcp-drive", version: "1.0.0" },
    },
    initialize_response: initialize,
    notification: { method: "notifications/initialized", params: {} },
    messages_after_notification_until_ping: afterNotification,
    ping_response: ping,
  });

  // C2：tools/list 12 名逐项相等（集合 + 顺序）
  const toolsList = await mcp.request("tools/list", {});
  const names = (toolsList.result?.tools ?? []).map((tool) => tool.name);
  check("C2.tools/list 计数 12", names.length === 12, `实际 ${names.length}`);
  check(
    "C2.tools/list 顺序逐项相等",
    JSON.stringify(names) === JSON.stringify(EXPECTED_TOOLS),
    `actual=${JSON.stringify(names)}`,
  );
  check(
    "C2.tools/list 集合相等",
    names.length === EXPECTED_TOOLS.length && EXPECTED_TOOLS.every((name) => names.includes(name)),
    `missing=${JSON.stringify(EXPECTED_TOOLS.filter((name) => !names.includes(name)))}`,
  );
  writeJson("c2-tools-list.json", {
    expected_order: EXPECTED_TOOLS,
    actual_order: names,
    tools: toolsList.result?.tools,
  });

  // C3：13 套件 run_tests（套件名读源码注册表；计数双盲 claim-lint + 台账）
  // 前置等待：ts-11 头注要求「启动 ≥3s 后首次调用本套件」（avg_fps 在 2s warmup
  // 后才开始累计）——run1 无此等待，ts-11 在游戏年龄 1.35s 被调、avg_fps=0.000
  // 假红 2 断言；补 4s 等待（修正留痕，证据 mcp-drive-run1-3fails.log）。
  await sleep(4000);
  const suites = readSuiteNames();
  const expect = readClaimLintExpect();
  check(
    "C3.套件名读取与 claim-lint IN_PROC_SUITES 双盲",
    suites.length === expect.inProcSuites && suites.length === 13,
    `源码注册表 ${suites.length} / claim-lint ${expect.inProcSuites}`,
  );
  const ledger = readFileSync("assets-methodology/task-ledger.md", "utf8");
  check(
    "C3.台账 T042 行预注册 45 断言可解析",
    /13\/13 套件 45 断言/.test(ledger),
    "台账未找到「13/13 套件 45 断言」锚点",
  );

  const suiteRows = [];
  let assertionTotal = 0;
  for (const suite of suites) {
    const response = await mcp.request(
      "tools/call",
      { name: "game.run_tests", arguments: { suite } },
      90000,
    );
    const text = response.result?.content?.[0]?.text;
    let parsed = null;
    try {
      parsed = JSON.parse(text);
    } catch (error) {
      check(`C3.${suite}.JSON 可解析`, false, `text 非 JSON：${error.message}`);
      continue;
    }
    const result = parsed.result ?? {};
    const count = Array.isArray(result.results) ? result.results.length : -1;
    check(`C3.${suite}.isError=false`, response.result?.isError === false, `isError=${response.result?.isError}`);
    check(
      `C3.${suite}.passed==results.length`,
      result.passed === count,
      `passed=${result.passed} results.length=${count}`,
    );
    check(`C3.${suite}.failed==0`, result.failed === 0, `failed=${result.failed}`);
    assertionTotal += Math.max(count, 0);
    suiteRows.push({
      suite,
      total: result.total,
      passed: result.passed,
      failed: result.failed,
      assertions: count,
      names: (result.results ?? []).map((entry) => entry.name),
    });
    appendFileSync(`${EVID}/c3-run-tests.jsonl`, `${JSON.stringify({ suite, response: parsed })}\n`);
  }
  check("C3.断言总数 45（运行值）", assertionTotal === 45, `实际 ${assertionTotal}`);
  writeJson("c3-summary.json", { suites: suiteRows, assertion_total: assertionTotal, expected: 45 });

  // C4：snapshot 三态 + 非空参数错误透传 + 直调 params:null（S2 前提）
  async function callTool(name, args) {
    const params = { name };
    if (args !== undefined) params.arguments = args;
    const response = await mcp.request("tools/call", params, 90000);
    return { response, text: response.result?.content?.[0]?.text };
  }

  const snapOmitted = await callTool("game.snapshot");
  const snapEmpty = await callTool("game.snapshot", {});
  const snapNull = await callTool("game.snapshot", null);
  const snapProbe = await callTool("game.snapshot", { __t041_probe: "non-empty" });
  const omittedOk = snapOmitted.response.result?.isError === false && JSON.parse(snapOmitted.text).result !== undefined;
  const emptyOk = snapEmpty.response.result?.isError === false && JSON.parse(snapEmpty.text).result !== undefined;
  const nullOk = snapNull.response.result?.isError === false && JSON.parse(snapNull.text).result !== undefined;
  check("C4.snapshot arguments 缺省 → 成功", omittedOk, `text 前 120 字：${String(snapOmitted.text).slice(0, 120)}`);
  check("C4.snapshot arguments={} → 成功", emptyOk, `text 前 120 字：${String(snapEmpty.text).slice(0, 120)}`);
  check(
    "C4.snapshot arguments=null → 成功（B5 三态省略 params 契约）",
    nullOk,
    `text 前 160 字：${String(snapNull.text).slice(0, 160)}`,
  );
  const probeText = String(snapProbe.text ?? "");
  check(
    "C4.snapshot 非空 arguments → 游戏侧 -32602 原文透传",
    snapProbe.response.result?.isError === true && probeText.includes("-32602"),
    `isError=${snapProbe.response.result?.isError} text 前 200 字：${probeText.slice(0, 200)}`,
  );
  const directNull = await brpPost(
    JSON.stringify({ jsonrpc: "2.0", id: 9001, method: "game.snapshot", params: null }),
  );
  // S2 注勘误（run1 假红根因）：T040 审核轮 S2 注称「params:null 报 -32602（serde
  // 落 Some(Null)）」——实况是 bevy_remote 反序列化 `params.flatten()`（lib.rs:1150，
  // 0.19.1 与 0.20.0 同款）把线上 null 归一为缺省，handler 收到的是 None，与
  // params 键缺失不可区分 → 成功。桥侧「省略 params」操作结论不受影响。
  check(
    "C4.直调 BRP params:null → 成功（flatten() 归一为缺省，S2 注勘误）",
    directNull.status === 200 && directNull.text.includes('"result"') && !directNull.text.includes("-32602"),
    `status=${directNull.status} text 前 200 字：${directNull.text.slice(0, 200)}`,
  );
  writeJson("c4-snapshot.json", {
    via_bridge: {
      arguments_omitted: { isError: snapOmitted.response.result?.isError, text: snapOmitted.text },
      arguments_empty_object: { isError: snapEmpty.response.result?.isError, text: snapEmpty.text },
      arguments_null: { isError: snapNull.response.result?.isError, text: snapNull.text },
      arguments_non_empty: { isError: snapProbe.response.result?.isError, text: snapProbe.text },
    },
    direct_brp_params_null: { status: directNull.status, text: directNull.text },
    note: "B5 契约把 arguments 缺失/null/{} 三态统一省略 params；params:null 在 BRP 侧经 bevy_remote `params.flatten()`（lib.rs:1150，0.19.1/0.20.0 同）归一为缺省 → 成功（run1 按 S2 注期待 -32602 系前提失实，勘误留痕）",
  });

  // C5：四个只读 BRP 方法
  const discover = await callTool("rpc.discover");
  const discoverParsed = JSON.parse(discover.text);
  const discoverMethods = discoverParsed.result?.methods ?? [];
  check(
    "C5.rpc.discover methods 总数 31",
    discoverMethods.length === 31,
    `实际 ${discoverMethods.length}`,
  );
  check(
    "C5.rpc.discover 总数与 claim-lint 23+8 双盲",
    discoverMethods.length === expect.brpBuiltin + expect.gameMethods,
    `claim-lint BRP_BUILTIN=${expect.brpBuiltin} GAME_METHODS=${expect.gameMethods}`,
  );
  const missing = EXPECTED_TOOLS.filter((name) => !discoverMethods.some((method) => method.name === name));
  check("C5.rpc.discover 含全部 12 白名单名", missing.length === 0, `missing=${JSON.stringify(missing)}`);

  const query = await callTool("world.query", {
    data: { components: ["game::sim::Wanderer", "game::sim::Velocity"] },
    filter: { without: [] },
  });
  const queryParsed = JSON.parse(query.text);
  const queryOk =
    query.response.result?.isError === false &&
    Array.isArray(queryParsed.result) &&
    queryParsed.result.length > 0 &&
    Object.prototype.hasOwnProperty.call(queryParsed.result[0], "entity") &&
    Object.prototype.hasOwnProperty.call(queryParsed.result[0], "components");
  check("C5.world.query 透传结构符合先例", queryOk, `result_len=${queryParsed.result?.length}`);
  writeFileSync(`${EVID}/c5-world-query.json`, `${query.text}\n`);

  const entity = queryParsed.result?.[0]?.entity;
  const getComponents = await callTool("world.get_components", {
    entity,
    components: ["game::sim::Wanderer", "game::sim::Velocity"],
  });
  const getParsed = JSON.parse(getComponents.text);
  const getOk =
    getComponents.response.result?.isError === false &&
    typeof getParsed.result === "object" &&
    getParsed.result?.components &&
    Object.prototype.hasOwnProperty.call(getParsed.result.components, "game::sim::Wanderer") &&
    Object.prototype.hasOwnProperty.call(getParsed.result.components, "game::sim::Velocity") &&
    JSON.stringify(getParsed.result.errors) === "{}";
  check("C5.world.get_components 透传结构符合先例", getOk, `entity=${entity}`);
  writeFileSync(`${EVID}/c5-world-get-components.json`, `${getComponents.text}\n`);

  const schema = await callTool("registry.schema", { with_crates: ["game"] });
  const schemaParsed = JSON.parse(schema.text);
  const schemaOk =
    schema.response.result?.isError === false &&
    typeof schemaParsed.result === "object" &&
    Object.prototype.hasOwnProperty.call(schemaParsed.result, "game::sim::Wanderer");
  check("C5.registry.schema 透传结构符合先例", schemaOk, `keys=${Object.keys(schemaParsed.result ?? {}).length}`);
  writeFileSync(`${EVID}/c5-registry-schema.json`, `${schema.text}\n`);
  writeFileSync(`${EVID}/c5-rpc-discover.json`, `${discover.text}\n`);

  writeJson("c5-summary.json", {
    rpc_discover: { method_count: discoverMethods.length, missing_allowlisted: missing },
    world_query: { result_len: queryParsed.result?.length, first_entity: entity },
    world_get_components: { entity, ok: getOk },
    registry_schema: { key_count: Object.keys(schemaParsed.result ?? {}).length, ok: schemaOk },
  });

  // C6：screenshot 正调 + 轮询落盘；其余四个写方法用无净副作用探针（-32602 透传）
  const shotPath = `${EVID}/mcp-shot.png`;
  const shot = await callTool("game.screenshot", { path: shotPath });
  const shotParsed = JSON.parse(shot.text);
  const shotId = shotParsed.result?.id;
  check("C6.game.screenshot 受理", shot.response.result?.isError === false && shotParsed.result?.queued === true, `result=${shot.text.slice(0, 160)}`);
  let shotEntry = null;
  for (let attempt = 0; attempt < 100; attempt += 1) {
    const state = await callTool("game.screenshot_log", {});
    const parsedState = JSON.parse(state.text);
    shotEntry = (parsedState.result?.entries ?? []).find((entry) => entry.id === shotId) ?? null;
    if (shotEntry?.status === "captured") break;
    await sleep(300);
  }
  check("C6.game.screenshot_log 轮询到 captured", shotEntry?.status === "captured", `entry=${JSON.stringify(shotEntry)}`);
  must(existsSync(shotPath), `截图未落盘：${shotPath}`);
  const png = pngHeaderOk(shotPath);
  check("C6.PNG 魔数", png.magic, `bytes=${png.bytes}`);
  check(
    "C6.PNG 非空且尺寸与日志一致",
    png.bytes > 0 && shotEntry?.width > 0 && shotEntry?.height > 0,
    `width=${shotEntry?.width} height=${shotEntry?.height} bytes=${png.bytes}`,
  );

  // 四个写方法：先确认未加载关卡（snapshot.battle=null），再用无效参数探针。
  const beforeProbes = await callTool("game.snapshot");
  const battleNull = JSON.parse(beforeProbes.text).result?.battle === null;
  check("C6.写方法探针前未加载关卡（battle=null）", battleNull, "battle 非 null，拒绝继续（可能污染现场）");
  const probes = [
    ["game.launch_level", {}],
    ["game.move_unit", {}],
    ["game.attack", {}],
    ["game.end_turn", {}],
  ];
  const probeRows = [];
  for (const [name, args] of probes) {
    const response = await callTool(name, args);
    const text = String(response.text ?? "");
    const ok = response.response.result?.isError === true && text.includes("-32602");
    check(`C6.${name} 无效参数探针 -32602 透传`, ok, `isError=${response.response.result?.isError} text 前 160 字：${text.slice(0, 160)}`);
    probeRows.push({ method: name, arguments: args, isError: response.response.result?.isError, text });
  }
  writeJson("c6-screenshot.json", {
    screenshot_response: shotParsed,
    screenshot_log_entry: shotEntry,
    png_path: shotPath,
    png_bytes: png.bytes,
    png_magic: png.magic,
  });
  writeJson("c6-write-probes.json", { battle_null_before_probes: battleNull, probes: probeRows });

  // C7：白名单拒绝（不达 BRP）+ MCP 未知方法 -32601
  const reject = await mcp.request("tools/call", {
    name: "world.mutate_components",
    arguments: { entity: 1, components: {} },
  });
  check(
    "C7.world.mutate_components 白名单拒绝",
    reject.error?.code === -32602 && String(reject.error?.message ?? "").includes("白名单拒绝") && reject.result === undefined,
    `error=${JSON.stringify(reject.error)}`,
  );
  const unknown = await mcp.request("foo/bar", {});
  check(
    "C7.MCP 未知方法 -32601",
    unknown.error?.code === -32601,
    `error=${JSON.stringify(unknown.error)}`,
  );
  const aliveAfterReject = await callTool("rpc.discover");
  check(
    "C7.拒绝后游戏仍存活（探针未达 BRP）",
    aliveAfterReject.response.result?.isError === false,
    `isError=${aliveAfterReject.response.result?.isError}`,
  );
  writeJson("c7-negative-control.json", {
    whitelist_rejection: reject,
    unknown_method: unknown,
    game_alive_after_rejection: aliveAfterReject.response.result?.isError === false,
  });

  // C8：transcript 逐行可解析、零机器绝对路径
  const transcriptText = readFileSync(TRANSCRIPT, "utf8");
  const transcriptLines = transcriptText.split(/\r?\n/).filter((line) => line.trim().length > 0);
  let allJson = true;
  for (const line of transcriptLines) {
    try {
      JSON.parse(line);
    } catch {
      allJson = false;
    }
  }
  const absolutePatterns = [
    /[A-Za-z]:\\\\/,
    /[A-Za-z]:\//,
    /\/Users\//,
    /Desktop[\\/]/,
    /AppData[\\/]/,
  ];
  const hit = absolutePatterns.find((pattern) => pattern.test(transcriptText));
  check("C8.transcript 每行 JSON", allJson, `lines=${transcriptLines.length}`);
  check("C8.transcript 零机器绝对路径", !hit, hit ? `命中 ${hit}` : `lines=${transcriptLines.length}`);
  check("C8.桥 stdout 协议零污染", mcp.protocolErrors.length === 0, mcp.protocolErrors.join("; ").slice(0, 300));
}

function taskkillGame() {
  if (!game) return;
  const pid = game.pid;
  const kill = spawnSync("taskkill", ["/F", "/T", "/PID", String(pid)], { encoding: "utf8" });
  appendFileSync(
    `${EVID}/game-kill.log`,
    `$ taskkill /F /T /PID ${pid}\n${kill.stdout ?? ""}${kill.stderr ?? ""}REAL_EXIT=${kill.status}\n`,
  );
  spawnSync("cmd", ["/c", "timeout", "/t", "2", "/nobreak", ">", "nul"]);
  const list = spawnSync("tasklist", ["/FI", "IMAGENAME eq game.exe", "/FO", "CSV", "/NH"], { encoding: "utf8" });
  appendFileSync(
    `${EVID}/game-kill.log`,
    `$ tasklist /FI "IMAGENAME eq game.exe" /FO CSV /NH\n${list.stdout ?? ""}${list.stderr ?? ""}REAL_EXIT=${list.status}\n`,
  );
  const residue = /game\.exe/i.test(list.stdout ?? "");
  check("C8.清杀后 game.exe 无残留", !residue, residue ? "tasklist 仍见 game.exe" : "tasklist 无 game.exe");
}

async function main() {
  let crashed = null;
  try {
    await run();
  } catch (error) {
    crashed = error;
    log(`[FAIL] 驱动异常：${error.message}`);
    failures.push(`驱动异常：${error.message}`);
  } finally {
    if (mcp) {
      try {
        await mcp.close();
      } catch (error) {
        log(`[FAIL] 关闭 MCP 桥异常：${error.message}`);
        failures.push(`关闭 MCP 桥异常：${error.message}`);
      }
    }
    try {
      taskkillGame();
    } catch (error) {
      log(`[FAIL] 清杀异常：${error.message}`);
      failures.push(`清杀异常：${error.message}`);
    }
    if (game) {
      await Promise.race([
        new Promise((resolve) => game.once("exit", resolve)),
        sleep(10000),
      ]);
    }
    a: {
      try {
        const after = await brpPost(brpDiscoverBody());
        check("C8.清杀后 15702 不再监听", false, `仍可 POST（status=${after.status}）`);
      } catch {
        check("C8.清杀后 15702 不再监听", true, "连接失败（端口已释放）");
      }
    }
  }

  const passed = checks - failures.length;
  log("");
  log(`判定行（脚本计算）：ASSERTIONS=${checks} PASS=${passed} FAIL=${failures.length}`);
  for (const failure of failures) {
    log(`FAILED: ${failure}`);
  }
  log(`REAL_EXIT=${failures.length === 0 ? 0 : 1}`);
  writeJson("judge.json", {
    assertions: checks,
    passed,
    failed: failures.length,
    failures,
    crashed: crashed ? String(crashed.message) : null,
    real_exit: failures.length === 0 ? 0 : 1,
  });
  process.exitCode = failures.length === 0 ? 0 : 1;
}

await main();

// TS-11 四条断言评估器（对 raw 文件评估，不重复请求）。
// 输入：a1-simstats.raw.json / a2-simstats.raw.json / run1-game.log（均在同目录）。
// 断言口径逐字取自 assets-methodology/taskset/ts-11-perf-ladder.md：
//   1) a1: avg_fps > 0 且 fps_1s > 0（采集链路活；采样=进程退出前的 get_resources 响应）
//   2) tick > 0（清单「请求要点：同上」——与断言 1 同一 world.get_resources 请求，
//      本评估以 a1 响应判定；补充采样 a2 因落点在进程退出后而为空，如实标注）
//   3) [BENCH] 行：n=50000、warmup_secs=2.0、avg_fps 与 #1 读值一致（±10%）
//   4) 基线对照：avg_fps >= 60.0 * 0.8（docs/fps-baseline.md vsync 组 N=50000 = 60.0，同机）
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const DIR = dirname(fileURLToPath(import.meta.url));
const read = (f) => readFileSync(join(DIR, f), 'utf8').trim();

const a1 = JSON.parse(read('a1-simstats.raw.json')).result.value;

// a2 为清单外的补充复核采样：首次运行中落点在进程退出之后，文件为空——
// 如实标注，不参与断言 2 判定（断言 2 按「同上」由 a1 判定）。
let a2 = null;
let a2Attempt = 'missing';
try {
  a2 = JSON.parse(read('a2-simstats.raw.json')).result.value;
  a2Attempt = 'ok';
} catch {
  a2Attempt = 'empty(post-exit attempt)';
}

const log = read('run1-game.log');
const m = log.match(
  /\[BENCH\] n=(\d+) seed=(\d+) bench_secs=([\d.]+) warmup_secs=([\d.]+) measure_secs=([\d.]+) measure_frames=(\d+) avg_fps=([\d.]+)/,
);
if (!m) {
  console.error('NO_BENCH_LINE');
  process.exit(1);
}
const bench = {
  n: Number(m[1]),
  seed: Number(m[2]),
  bench_secs: Number(m[3]),
  warmup_secs: Number(m[4]),
  measure_secs: Number(m[5]),
  measure_frames: Number(m[6]),
  avg_fps: Number(m[7]),
};

const BASELINE = 60.0; // docs/fps-baseline.md vsync(AutoVsync) 组 N=50000（同机同口径）
const THRESHOLD = BASELINE * 0.8;
const relDev = Math.abs(bench.avg_fps - a1.avg_fps) / a1.avg_fps;

const out = {
  a1: {
    tick: a1.tick,
    frame_count: a1.frame_count,
    elapsed_secs: a1.elapsed_secs,
    fps_1s: a1.fps_1s,
    avg_fps: a1.avg_fps,
  },
  a2_attempt: a2Attempt,
  a2: a2
    ? { tick: a2.tick, elapsed_secs: a2.elapsed_secs, fps_1s: a2.fps_1s, avg_fps: a2.avg_fps }
    : null,
  bench,
  baseline: { group: 'vsync(AutoVsync)', n: 50000, avg_fps: BASELINE, threshold: THRESHOLD },
  checks: {
    a1_avg_fps_pos: a1.avg_fps > 0,
    a1_fps_1s_pos: a1.fps_1s > 0,
    a2_tick_pos_source: a2 ? 'a2' : 'a1(清单断言2请求=同上)',
    a2_tick_pos: a2 ? a2.tick > 0 : a1.tick > 0,
    stats_log_tick_growth_side: /t=1\.0s tick=(\d+)/.test(log) && /t=9\.1s tick=(\d+)/.test(log),
    a3_n_is_50000: bench.n === 50000,
    a3_warmup_is_2_0: bench.warmup_secs === 2.0,
    a3_within_10pct: relDev <= 0.1,
    a3_rel_dev: relDev,
    a4_bench_ge_threshold: bench.avg_fps >= THRESHOLD,
    a4_ratio_bench: bench.avg_fps / BASELINE,
    a4_ratio_a1: a1.avg_fps / BASELINE,
  },
};
console.log(JSON.stringify(out, null, 2));

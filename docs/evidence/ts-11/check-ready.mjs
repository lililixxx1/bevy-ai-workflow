// TS-11 采样点判定器（测量工具，非断言本身）。
// stdin 读入一条 BRP `world.get_resources` (game::sim::SimStats) 响应原文，
// 输出两行：第 1 行 READY/WAIT/ERROR，第 2 行解析出的统计值 JSON。
// 采样点条件（采样协议）：avg_fps>0（测量窗已开，即 warmup 2s 已过）且
// fps_1s>0（1s 窗已刷新）且 tick>=150（模拟已推进 ~2.5s，避开测量窗开启边界）。
// 该条件只决定「何时采样」，断言 1/2 仍按任务清单原文评估采样值本身。
let text = '';
process.stdin.setEncoding('utf8');
for await (const chunk of process.stdin) text += chunk;
try {
  const v = JSON.parse(text).result.value;
  const ready = v.avg_fps > 0 && v.fps_1s > 0 && v.tick >= 150;
  console.log(ready ? 'READY' : 'WAIT');
  console.log(
    JSON.stringify({
      tick: v.tick,
      frame_count: v.frame_count,
      elapsed_secs: v.elapsed_secs,
      fps_1s: v.fps_1s,
      avg_fps: v.avg_fps,
    }),
  );
} catch {
  console.log('ERROR');
}

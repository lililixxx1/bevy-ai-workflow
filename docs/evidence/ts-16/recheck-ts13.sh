#!/usr/bin/env bash
# TS-13 复断言（T035 核心回归）：新进程上重采 6 断言的 raw 响应，
# 用存档断言脚本（docs/evidence/ts-13/assert-ts13.js 原文复本）判定。
# 输出隔离于 ts-13-recheck/（不动 ts-13 历史证据）。
set -u
DIR="$(cd -- "$(dirname -- "$0")" && pwd)"
OUT="$DIR/ts-13-recheck"
mkdir -p "$OUT"
BRP=http://127.0.0.1:15702

call() { # $1=输出文件 $2=方法 $3=params
  curl -s --max-time 5 "$BRP" -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$2\",\"params\":$3}" > "$OUT/$1"
}

# 对齐 ts-13 存档断言脚本的文件名契约（a1..a6c）。
call a1-query-baseline.raw.json  world.query        '{"data":{"components":["game::level::Unit"]}}'
call a2-launch-1.raw.json        game.launch_level  '{"level":1}'
call a3-query-l1.raw.json        world.query        '{"data":{"components":["game::level::GridPos","game::level::Unit"]}}'
call a4a-relaunch-1.raw.json     game.launch_level  '{"level":1}'
call a4b-relaunch-1.raw.json     game.launch_level  '{"level":1}'
call a4c-query-idem.raw.json     world.query        '{"data":{"components":["game::level::GridPos","game::level::Unit"]}}'
call a5a-launch-2.raw.json       game.launch_level  '{"level":2}'
call a5b-query-l2.raw.json       world.query        '{"data":{"components":["game::level::GridPos","game::level::Unit"]}}'
call a6a-launch-99.raw.json      game.launch_level  '{"level":99}'
call a6b-level-state.raw.json    world.get_resources '{"resource":"game::level::LevelState"}'
call a6c-query-after-error.raw.json world.query     '{"data":{"components":["game::level::Unit"]}}'

# 存档断言脚本原文复本（不改判定逻辑）。
cp "$DIR/../ts-13/assert-ts13.js" "$OUT/assert-ts13.js"
(cd "$OUT" && node assert-ts13.js > assert-out.txt 2>&1)
echo "TS13_RECHECK_EXIT=$?"
cat "$OUT/assert-out.txt"

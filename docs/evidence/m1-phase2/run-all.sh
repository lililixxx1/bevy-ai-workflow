#!/usr/bin/env bash
# M1 两阶段判定第二阶段重跑驱动：逐任务冷启动游戏进程，脚本化判定，留档。
# 口径：每任务独立进程（各清单「启动方式」原文）；串行（BRP 单端口 15702）；
#       每任务收尾 taskkill //F（MSYS 双斜杠，PIT-M-005）+ 残留核对。
# 用法：仓库外任意 CWD 均可  bash <repo>/docs/evidence/m1-phase2/run-all.sh
#       （必须以后台任务运行：前台调用的进程组清理会杀死游戏子进程——
#         2026-09-27 实测，见证据文档「驱动口径」节。）
set -u

# 绝对路径锚定：脚本真实位置（仓库根下三级）上溯三级 = 仓库根（不依赖调用方 CWD /
# 不依赖 $0 的相对绝对形态——首版凭 $0 相对形态推断 CWD，$0 为绝对路径时
# 全部落到错误目录，已实测翻车）。
SCRIPT_DIR=$(cd -- "$(dirname -- "$0")" && pwd)
REPO=$(cd -- "$SCRIPT_DIR/../../.." && pwd)
ROOT="$REPO/docs/evidence/m1-phase2"
SUMMARY="$ROOT/summary.txt"
GAME="$REPO/target/release/game.exe"
RUNNER="$REPO/target/release/task-runner.exe"

mkdir -p "$ROOT"
: > "$SUMMARY"
FAIL=0   # 聚合退出码：任一任务判定非 0 / 收尾不净 → 脚本整体非 0（退出码即判定）

wait_brp_ready() {  # $1=输出文件：轮询 BRP 就绪（SimConfig 可读）最长 20s
  local out=$1 i
  : > "$out"
  for i in $(seq 1 40); do
    curl -s --max-time 2 http://127.0.0.1:15702 -H "Content-Type: application/json" \
      -d '{"jsonrpc":"2.0","method":"world.get_resources","id":1,"params":{"resource":"game::sim::SimConfig"}}' \
      > "$out" && grep -q entity_count "$out" && return 0
    sleep 0.5
  done
  return 1
}

game_alive() { tasklist //FI "IMAGENAME eq game.exe" 2>/dev/null | grep -qi "game\.exe"; }

kill_and_verify() {  # $1=任务目录：强杀 + 残留核对（残留则非 no residue）
  local dir=$1
  taskkill //F //IM game.exe > "$dir/taskkill.log" 2>&1
  sleep 1
  if game_alive; then
    tasklist //FI "IMAGENAME eq game.exe" > "$dir/residue.txt" 2>&1
    return 1
  fi
  echo "no residue" > "$dir/residue.txt"
  return 0
}

run_task() {  # $1=任务名 $2=游戏启动参数 $3=task-runner 参数
  local name=$1 game_args=$2 runner_args=$3
  local dir="$ROOT/$name"
  mkdir -p "$dir"
  echo "=== $name $(date -Iseconds) ===" >> "$SUMMARY"

  # 前置残留防护：端口占用会让就绪探测假成功。
  if game_alive; then
    echo "[WARN] $name 启动前发现残留 game.exe，先清理" >> "$SUMMARY"
    kill_and_verify "$dir" || true
  fi

  # 启动（清单口径的启动方式原文）。
  "$GAME" $game_args > "$dir/game.log" 2>&1 &
  echo $! > "$dir/game.pid"
  sleep 2
  if ! game_alive; then
    echo "$name: 游戏进程启动后即消失（game.log 末行如下）——任务失败" >> "$SUMMARY"
    tail -1 "$dir/game.log" >> "$SUMMARY"
    echo "$name RUNNER_EXIT=96（进程早夭）" >> "$SUMMARY"
    FAIL=1
    return
  fi

  # BRP 就绪 + 预热余量（tick/avg_fps 就绪；ts-11 需 warmup 2s 后）。
  if ! wait_brp_ready "$dir/brp-ready.json"; then
    echo "$name: BRP 未就绪（20s 超时）——任务失败" >> "$SUMMARY"
    kill_and_verify "$dir" || true
    echo "$name RUNNER_EXIT=97（未就绪）" >> "$SUMMARY"
    FAIL=1
    return
  fi
  sleep 3

  # 脚本判定。
  "$RUNNER" $runner_args > "$dir/runner.txt" 2>&1
  local re=$?
  echo "REAL_EXIT=$re" > "$dir/runner.exit"

  kill_and_verify "$dir"; local clean=$?
  echo "$name RUNNER_EXIT=$re kill_clean=$clean" >> "$SUMMARY"
  [ "$re" -ne 0 ] && FAIL=1
  [ "$clean" -ne 0 ] && FAIL=1
}

# ---------- 常规 10 任务（run 模式两连调 + 字面值绑定）----------
run_task ts-01 "--count 500 --seed 20260926"  "run --suite ts-01 --expect-count 500 --expect-seed 20260926"
run_task ts-02 "--count 1000 --seed 20260926 --max-speed 9.0" "run --suite ts-02 --expect-count 1000 --expect-seed 20260926 --expect-max-speed 9.0"
run_task ts-03 "--count 250 --seed 7"          "run --suite ts-03 --expect-count 250 --expect-seed 7 --expect-version 0.1.0"
run_task ts-05 "--count 100 --seed 20260926"   "run --suite ts-05 --expect-count 100 --expect-seed 20260926"
run_task ts-06 "--count 100 --seed 20260926"   "run --suite ts-06 --expect-count 100 --expect-seed 20260926"
run_task ts-07 "--count 100 --seed 20260926"   "run --suite ts-07 --expect-count 100 --expect-seed 20260926"
run_task ts-08 "--count 1000 --seed 20260926"  "run --suite ts-08 --expect-count 1000 --expect-seed 20260926"
run_task ts-09 "--count 100 --seed 20260926"   "run --suite ts-09 --expect-count 100 --expect-seed 20260926"
run_task ts-10 "--count 100 --seed 20260926"   "run --suite ts-10 --expect-count 100 --expect-seed 20260926"
run_task ts-12 "--count 100 --seed 20260926"   "run --suite ts-12 --expect-count 100 --expect-seed 20260926"

# ---------- ts-04：pause 子命令（跨帧时序，清单启动口径 --count 1000）----------
run_task ts-04 "--count 1000 --seed 20260926" "pause"

# ---------- ts-11：bench 口径（--bench-secs 10 自退出 + bench-judge）----------
name=ts-11
dir="$ROOT/$name"
mkdir -p "$dir"
echo "=== $name $(date -Iseconds) ===" >> "$SUMMARY"
if game_alive; then
  echo "[WARN] $name 启动前发现残留 game.exe，先清理" >> "$SUMMARY"
  kill_and_verify "$dir" || true
fi
"$GAME" --count 50000 --seed 20260926 --bench-secs 10 > "$dir/game.log" 2>&1 &
echo $! > "$dir/game.pid"
sleep 2
if ! game_alive; then
  echo "$name: 游戏进程启动后即消失——任务失败" >> "$SUMMARY"
  tail -1 "$dir/game.log" >> "$SUMMARY"
  echo "$name RUNNER_EXIT=96（进程早夭）" >> "$SUMMARY"
elif ! wait_brp_ready "$dir/brp-ready.json"; then
  echo "$name: BRP 未就绪——任务失败" >> "$SUMMARY"
  kill_and_verify "$dir" || true
  echo "$name RUNNER_EXIT=97（未就绪）" >> "$SUMMARY"
else
  sleep 3   # warmup 2s + 余量（avg_fps 开始累计）
  "$RUNNER" run --suite ts-11 --expect-count 50000 --expect-seed 20260926 \
    > "$dir/runner.txt" 2>&1
  re=$?
  echo "REAL_EXIT=$re" > "$dir/runner.exit"
  # BRP 读值留档（bench-judge 的 --brp-avg 来源，运行中采样）。
  curl -s http://127.0.0.1:15702 -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"world.get_resources","id":9,"params":{"resource":"game::sim::SimStats"}}' \
    > "$dir/brp-stats.json"
  AVG=$(grep -o '"avg_fps":[0-9.]*' "$dir/brp-stats.json" | head -1 | cut -d: -f2)
  echo "$AVG" > "$dir/brp-avg.txt"
  # 等进程自退出（bench_secs=10；最长 25s）。
  for i in $(seq 1 50); do
    game_alive || break
    sleep 0.5
  done
  if game_alive; then
    echo "[WARN] $name bench 进程未自退出，强杀" >> "$SUMMARY"
    kill_and_verify "$dir" || true
  else
    echo "self-exit clean" > "$dir/residue.txt"
    : > "$dir/taskkill.log"
  fi
  "$RUNNER" bench-judge --log "$dir/game.log" --brp-avg "$AVG" \
    --baseline-fps 60.0 --expect-count 50000 > "$dir/bench-judge.txt" 2>&1
  bj=$?
  echo "REAL_EXIT=$bj" > "$dir/bench-judge.exit"
  echo "$name RUNNER_EXIT=$re BENCH_JUDGE_EXIT=$bj brp_avg=$AVG" >> "$SUMMARY"
  [ "$re" -ne 0 ] && FAIL=1
  [ "$bj" -ne 0 ] && FAIL=1
fi

echo "=== 全部任务完成 $(date -Iseconds) ===" >> "$SUMMARY"
echo "汇总：$SUMMARY"
exit $FAIL

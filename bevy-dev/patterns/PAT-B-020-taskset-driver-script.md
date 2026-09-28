# PAT-B-020-taskset-driver-script

### PAT-B-020：任务集驱动脚本——绝对路径锚定 + BRP 就绪轮询 + 每任务冷启动 + 聚合退出码

- 日期：2026-09-28
- 适用版本：bevy 0.19.1（脚本驱动 bevy 常驻游戏；shell 形态本身引擎无关）
- 分型：模式
- 通用性分级：bevy-specific（录入理由：与 BRP/taskkill/常驻游戏进程模型强耦合）
- 标签：`驱动脚本` `run-all` `冷启动` `BRP轮询` `taskkill` `聚合退出码` `绝对路径锚定`
- 源码出处：`docs/evidence/m1-phase2/run-all.sh:10-48`（锚定/就绪轮询/存活判定段；全脚本 1-165）——自研脚本归档于证据目录（tooling 证据面变体，入库注明）

**场景**：把 N 个「启动游戏→跑判定→收尾杀进程→核对残留」的任务串成一个可一键复跑的驱动脚本：仓库外任意 CWD 可调、任一任务失败整体失败。

**做法**（已过运行验证；代码为 `docs/evidence/m1-phase2/run-all.sh:10-48` 的逐字删减——保留关键注释）：

```bash
# 绝对路径锚定：脚本真实位置上溯三级 = 仓库根（不依赖调用方 CWD；
# 首版凭 $0 相对形态推断 CWD，$0 为绝对路径时全部落错目录，已实测翻车——PIT-M-006）。
SCRIPT_DIR=$(cd -- "$(dirname -- "$0")" && pwd)
REPO=$(cd -- "$SCRIPT_DIR/../../.." && pwd)

FAIL=0   # 聚合退出码：任一任务判定非 0 / 收尾不净 → 脚本整体非 0

wait_brp_ready() {  # $1=输出文件：轮询 BRP 就绪（SimConfig 可读）最长 20s
  local out=$1 i
  for i in $(seq 1 40); do
    curl -s --max-time 2 http://127.0.0.1:15702 -H "Content-Type: application/json" \
      -d '{"jsonrpc":"2.0","method":"world.get_resources","id":1,"params":{"resource":"game::sim::SimConfig"}}' \
      > "$out" && grep -q entity_count "$out" && return 0
    sleep 0.5
  done
  return 1
}

kill_and_verify() {  # $1=任务目录：强杀 + 残留核对
  taskkill //F //IM game.exe > "$1/taskkill.log" 2>&1   # MSYS 双斜杠——PIT-M-005
  # ……残留核对 tasklist……
}
```

要点：①`$(dirname -- "$0")` 锚定而非 `$0` 直用（相对/绝对两态皆稳）；②就绪探测用**业务探针**（get_resources 读 SimConfig）而非端口连通——端口开不代表插件就绪；③每任务冷启动独立进程（判定互不污染）+ `taskkill //F //IM`（MSYS 下 `/F` 会被路径转换破坏，双斜杠转义）+ 残留核对；④BRP 单端口 15702 → 任务必须串行；⑤常驻游戏进程须以后台任务托管运行本脚本（前台调用的进程组清理会杀死游戏子进程——PIT-M-007）。

**为什么**：「一键复跑 + 退出码即结论」是净室重跑（M3 ④）与 CI 化的最小底座；冷启动 + 残留核对保证每次运行起点等价。

**验证证据**：
- 运行验证：M1 两阶段第二阶段全量重跑由本脚本驱动，12 任务退出码全 0、残留全 clean（docs/evidence/m1-phase2/summary.txt）；stray 场景复跑存档 `.zcode/m1-phase2-stray-rerun-20260927T1329/`。

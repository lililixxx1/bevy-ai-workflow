# T044-W2 · 自检计数档

> 卡：T044-W2（M6 塔防 spike · 26 部 API 漂移底稿）；基线 commit `424a079`；隔离树内产物（无 .git）。
> 本卡为纯文档卡，无 cargo 门禁面（附录 A 纯读/文档轻负载条款）；本档 = 计数证据档（附录 B.2「计数禁手数」/ 附录 G）。
> 可公开态：本档零机器绝对路径、零用户名、零密钥；脚本与输出均以 w2/ 目录内相对文件名引用。

## 1. 交付物与计数口径

- `api-drift.md`：26 部主表，一行一 API；显式「无新 API」行 2 条（P05/P16，派工单规定格式）。
- `friction-summary.md`：**由 `count.sh` 扫描 `api-drift.md` 逐部计数生成**（本档第 3 节脚本 + 第 4 节原始输出；数值禁手数，附录 B.2）。
- 计数口径：
  - `n` = 该部 API 行数；`s` = 该部 Σ摩擦级（0/1/2/3 之和）。
  - `score = s + int(n / 8)`（API 面宽每 8 行加权 1 分，同族 API 批量核对，边际成本递减）。
  - `score <= 1` → S（≈40min）；`2 <= score <= 5` → M（≈90min）；`score >= 6` → L（≈150min）。
  - 外推区间 = 档位中值 ±25%（S 30–50 / M 68–113 / L 113–188 分钟），总区间 = Σ档位下限 ~ Σ档位上限。

## 2. 自检结论（机器输出摘要）

- 部覆盖：**26/26**（每部 ≥1 行）；总行数 **148**；Σ摩擦级 **14**。
- 结论分布：同形直用 136 / 改名或小形态差 7 / 重写调用面 2 / 无对应需路线变更 1；显式无新 API 行 2。
- 行级校验（脚本内建）：列数=6、摩擦级 ∈{0,1,2,3}、结论 ∈四词表（显式行豁免）、证据锚非空且无盘符/绝对路径前缀。
- 「0.18 形态摘要」字符数上限：实测最大 **15**（限 15）。
- 两表一致：`api_drift_rows=148 == summary_sigma_n=148`；`friction-summary.md` 与本节数值同源同跑。
- 命令：`bash count.sh`（cwd = `w2/`）；`REAL_EXIT=0`；末行 `SELF_CHECK_PASS`。

## 3. 计数脚本全文（`count.sh`）

```bash
#!/usr/bin/env bash
# T044-W2 计数与摩擦汇总生成脚本
# 输入：api-drift.md（唯一计数源）  输出：friction-summary.md（表格由本脚本生成）
# 口径：
#   n = 该部 API 行数（含显式「无新 API」行）
#   s = 该部 Σ摩擦级（0/1/2/3 之和）
#   score = s + int(n / 8)          # API 面宽每 8 行加权 1 分（同族 API 批量核对，边际成本递减）
#   tier  = S if score <= 1 ; M if 2 <= score <= 5 ; L if score >= 6
#   档位中值：S=40min / M=90min / L=150min（派工单给定基准）
#   外推区间：中值 ±25% -> S 30-50 / M 68-113 / L 113-188 分钟
# 校验：26 部每部 >=1 行、列数=6、摩擦级 ∈0..3、结论词表、证据锚可公开态、0.18 摘要 <=15 字
set -u
cd "$(dirname "$0")" || exit 90

SRC=api-drift.md
OUT=friction-summary.md
[ -f "$SRC" ] || { echo "ERR_MISSING $SRC"; echo "REAL_EXIT=1"; exit 1; }

# ---------- 1. 逐部统计 + 行级校验（awk） ----------
stats=$(awk -F'|' '
  function trim(x) { gsub(/^[ \t]+|[ \t]+$/, "", x); return x }
  /^## Part / {
    p = substr($0, 9, 2) + 0
    if (p < 1 || p > 26) { print "ERR_BAD_PART_HEADING " $0; bad = 1; next }
    seen[p] = 1
    next
  }
  /^\| API \|/ { next }
  /^\|---/ { next }
  /^\|/ {
    if (p < 1) { print "ERR_ROW_BEFORE_PART " $0; bad = 1; next }
    if (NF != 8) { printf "ERR_COLS part=%02d nf=%d\n", p, NF; bad = 1; next }
    api = trim($2); c18 = $4; concl = trim($5); anchor = trim($6); f = trim($7)
    if (api == "") { printf "ERR_EMPTY_API part=%02d\n", p; bad = 1; next }
    if (f !~ /^[0-3]$/) { printf "ERR_FRICTION part=%02d api=%s val=%s\n", p, api, f; bad = 1; next }
    if (anchor == "") { printf "ERR_EMPTY_ANCHOR part=%02d api=%s\n", p, api; bad = 1; next }
    if (anchor ~ /^[A-Za-z]:[\\\/]/ || anchor ~ /^\//) {
      printf "ERR_ABS_PATH part=%02d api=%s\n", p, api; bad = 1
    }
    explicit = (api ~ /无新 API/) ? 1 : 0
    if (!explicit) {
      if (concl != "同形直用" && concl != "改名/小形态差" && concl != "重写调用面" && concl != "无对应需路线变更") {
        printf "ERR_CONCLUSION part=%02d api=%s val=%s\n", p, api, concl; bad = 1
      }
      nconcl++
      cnt[concl]++
    } else { nexplicit++ }
    n[p]++
    s[p] += f
    if (f >= 1) drift[p] = drift[p] (drift[p] == "" ? "" : "、") api "(" f ")"
    if (explicit) noapi[p] = 1
    rows++
    next
  }
  END {
    for (p = 1; p <= 26; p++) {
      if (!seen[p]) { printf "ERR_PART_MISSING %02d\n", p; bad = 1 }
      printf "STAT|%02d|%d|%d|%s|%d\n", p, n[p], s[p], drift[p], (noapi[p] + 0)
    }
    printf "META|rows=%d|nconcl=%d|nexplicit=%d\n", rows, nconcl, nexplicit
    printf "CONCL|同形直用|%d\n", cnt["同形直用"]
    printf "CONCL|改名/小形态差|%d\n", cnt["改名/小形态差"]
    printf "CONCL|重写调用面|%d\n", cnt["重写调用面"]
    printf "CONCL|无对应需路线变更|%d\n", cnt["无对应需路线变更"]
    exit bad  }
' "$SRC")
awk_rc=$?
echo "$stats" | grep -E '^(ERR|STAT|META|CONCL)' || true

if [ "$awk_rc" -ne 0 ]; then
  echo "REAL_EXIT=$awk_rc"
  exit "$awk_rc"
fi

# ---------- 2. 0.18 摘要长度校验（bash 按字符计，UTF-8） ----------
len_report=$(awk -F'|' '/^\|/ && !/^\|---/ && !/^\| API \|/ { print $4 }' "$SRC" | while IFS= read -r cell; do
  cell="${cell# }"; cell="${cell% }"
  printf '%d\t%s\n' "${#cell}" "$cell"
done)
len_bad=$(echo "$len_report" | awk -F'\t' '$1 > 15' )
len_max=$(echo "$len_report" | awk -F'\t' 'BEGIN{m=0} $1>m{m=$1} END{print m}')
if [ -n "$len_bad" ]; then
  echo "ERR_018_SUMMARY_TOO_LONG"
  echo "$len_bad"
fi
echo "CHECK_018_SUMMARY_MAXLEN=$len_max (limit 15)"

# ---------- 3. 生成 friction-summary.md ----------
total_n=0; total_s=0; cS=0; cM=0; cL=0; ps_first3=0
table=""
while IFS='|' read -r tag p n s drift noapi; do
  [ "$tag" = "STAT" ] || continue
  score=$(( s + n / 8 ))
  if [ "$score" -le 1 ]; then tier=S; lo=30; mid=40; hi=50; cS=$((cS+1))
  elif [ "$score" -le 5 ]; then tier=M; lo=68; mid=90; hi=113; cM=$((cM+1))
  else tier=L; lo=113; mid=150; hi=188; cL=$((cL+1)); fi
  if [ "$noapi" = "1" ]; then note="显式无新 API 行（结论词表豁免）"
  elif [ -z "$drift" ]; then note="无漂移（全部 0 级）"
  else note="漂移: $drift"; fi
  table="${table}| P${p} | ${n} | ${s} | ${tier} | ${note} |
"
  total_n=$((total_n + n)); total_s=$((total_s + s))
  if [ "$p" = "01" ] || [ "$p" = "02" ] || [ "$p" = "03" ]; then ps_first3=$((ps_first3 + mid)); fi
done <<< "$stats"

sum_mid=$(( 40*cS + 90*cM + 150*cL ))
sum_lo=$(( 30*cS + 68*cM + 113*cL ))
sum_hi=$(( 50*cS + 113*cM + 188*cL ))
hours=$(awk -v a="$sum_lo" -v b="$sum_hi" -v c="$sum_mid" 'BEGIN{printf "%.1f~%.1f (中值 %.1f)", a/60, b/60, c/60}')

{
  echo "# T044-W2 · 每部摩擦汇总与工作量外推输入"
  echo
  echo "> 生成方式：本表由 \`count.sh\` 扫描 \`api-drift.md\` 逐部计数生成（禁手数，附录 B.2）；脚本全文与原始输出见 \`selfcheck.md\`。"
  echo "> 输入口径：教程 0.18 形态 → bevy 0.20.0 / bevy_ecs_tilemap 0.20.0 本地源码逐条核证（底稿 \`api-drift.md\`）。"
  echo
  echo "## 档位判定式（写死，脚本同源）"
  echo
  echo "- \`n\` = 该部 API 行数；\`s\` = 该部 Σ摩擦级（0/1/2/3 之和）。"
  echo "- \`score = s + int(n / 8)\`——API 面宽每 8 行加权 1 分（同族 API 批量核对，边际成本递减）。"
  echo "- \`score <= 1\` → **S（≈40min）**；\`2 <= score <= 5\` → **M（≈90min）**；\`score >= 6\` → **L（≈150min）**。"
  echo "- 基准 = 并行卡 W1 三部预期 P1≈40 / P2≈90 / P3≈90；Lead 终算时以 W1 实测替换档位中值，本表只给档位与计数。"
  echo "- 外推区间：档位中值 ±25%（S 30–50 / M 68–113 / L 113–188 分钟），总区间 = Σ档位下限 ~ Σ档位上限。"
  echo
  echo "## 每部汇总表"
  echo
  echo "| part | 新 API 数 | Σ摩擦级 | 预计卡时档 | 备注 |"
  echo "|---|---|---|---|---|"
  printf '%s' "$table"
  echo
  echo "## 26 部外推总工时"
  echo
  echo "- 行数合计：**${total_n}**；Σ摩擦级合计：**${total_s}**。"
  echo "- 档位分布：S=${cS} / M=${cM} / L=${cL}（共 $((cS+cM+cL)) 部）。"
  echo "- **Σ外推总工时区间 = ${sum_lo} ~ ${sum_hi} 分钟 = ${hours} 小时**（Σ档位中值）。"
  echo "- 对照：W1 三部基准 40+90+90 = 220 分钟；本表 P01–P03 档位中值合计 = ${ps_first3} 分钟（差 $((ps_first3-220)) 分钟，Lead 以 W1 实测替换）。"
  echo
  echo "## 读出"
  echo
  echo "- 26 部教程的 API 面以**同形直用**为绝对多数；摩擦集中在 6 部（P09/P12/P17/P21/P23 的 1 级改名或弃用，P26 的 2/3 级路线面）。"
  echo "- 真正需要路线决策的只有 P26 的 \`DynamicScene\`（3 级）；\`SpatialAudioBundle\` / 嵌入资产特性为 2 级调用面重写，不改变产品路线。"
  echo "- tilemap 面（P03/P04）与 0.20.0 crate 示例逐字段同形，塔防主路线无渲染层阻断。"
} > "$OUT"
gen_rc=$?

# ---------- 4. 一致性校验：底稿行数 == 汇总表 Σn ----------
meta=$(echo "$stats" | awk -F'|' '$1=="META"{print $2" "$3" "$4}')
rows_decl=$(echo "$meta" | awk '{sub(/rows=/,"",$1); print $1}')
sum_table_n=$(echo "$table" | awk -F'|' 'NF>1 {gsub(/ /,"",$3); s+=$3} END{print s+0}')

echo "CHECK_ROWS api_drift_rows=${rows_decl} summary_sigma_n=${sum_table_n}"
echo "CHECK_TOTAL_FRICTION summary_sigma_s=${total_s}"
echo "CHECK_TIERS S=${cS} M=${cM} L=${cL}"
echo "CHECK_EXTRAPOLATION minutes=${sum_lo}~${sum_hi} midpoint=${sum_mid} hours=${hours}"

fail=0
[ "$gen_rc" -eq 0 ] || { echo "ERR_WRITE_SUMMARY rc=$gen_rc"; fail=1; }
[ "$rows_decl" = "$sum_table_n" ] || { echo "ERR_ROW_MISMATCH"; fail=1; }
[ -z "$len_bad" ] || fail=1
[ "$awk_rc" -eq 0 ] || fail=1

if [ "$fail" -eq 0 ]; then
  echo "SELF_CHECK_PASS"
  echo "REAL_EXIT=0"
  exit 0
else
  echo "REAL_EXIT=1"
  exit 1
fi
```

## 4. 原始输出（`bash count.sh`，cwd = `w2/`）

```text
STAT|01|10|0||0
STAT|02|10|0||0
STAT|03|15|0||0
STAT|04|2|0||0
STAT|05|1|0||1
STAT|06|2|0||0
STAT|07|12|0||0
STAT|08|9|0||0
STAT|09|7|1|Parent / Children(1)|0
STAT|10|1|0||0
STAT|11|3|0||0
STAT|12|15|1|TextFont.font_size(1)|0
STAT|13|2|0||0
STAT|14|4|0||0
STAT|15|3|0||0
STAT|16|1|0||1
STAT|17|7|1|Interaction(1)|0
STAT|18|5|0||0
STAT|19|2|0||0
STAT|20|5|0||0
STAT|21|4|1|.and() 组合子(1)|0
STAT|22|4|0||0
STAT|23|5|2|Button(1)、Interaction 三态(1)|0
STAT|24|4|0||0
STAT|25|6|0||0
STAT|26|9|8|SpatialAudioBundle(2)、DynamicScene(3)、Observer(1)、嵌入资产特性(2)|0
META|rows=148|nconcl=146|nexplicit=2
CONCL|同形直用|136
CONCL|改名/小形态差|7
CONCL|重写调用面|2
CONCL|无对应需路线变更|1
CHECK_018_SUMMARY_MAXLEN=15 (limit 15)
CHECK_ROWS api_drift_rows=148 summary_sigma_n=148
CHECK_TOTAL_FRICTION summary_sigma_s=14
CHECK_TIERS S=23 M=2 L=1
CHECK_EXTRAPOLATION minutes=939~1564 midpoint=1250 hours=15.7~26.1 (中值 20.8)
SELF_CHECK_PASS
REAL_EXIT=0
```

说明：第 4 节即 `count.out` 的逐字节内容（脚本 stdout）；`REAL_EXIT=0` 与末行 `SELF_CHECK_PASS` 同档，stderr 为空（`count.err` 零字节）。

## 5. SOURCE-DEVIATION-NOTE 清单（教程源缺陷：只登记不修复，附录 C）

- `SOURCE-DEVIATION-NOTE 1: tutorial-src/13-multiple-levels.md:26-36 源=把 init_resource 列为该部「New Bevy APIs & Concepts」首次引入 本侧=tutorial-src/08-tower-placement.md:345 已在 walkthrough 使用 .init_resource::<PlacedTowers>() 理由=部间「新 API」口径不一致（不影响 0.20 结论） 裁决=待 L1`
- `SOURCE-DEVIATION-NOTE 2: tutorial-src/09-targeting-and-damage.md:62 源=称层级组件为 Parent/Children，with_children 自动挂 Parent(parent) 本侧=bevy_ecs-0.19.1/src/hierarchy.rs:107 与 bevy_ecs-0.20.0/src/hierarchy.rs:107 均为 ChildOf（无 Parent 类型） 理由=源文本 API 名过期（0.18 应亦为 ChildOf，0.19.1 实读佐证） 裁决=待 L1`
- `SOURCE-DEVIATION-NOTE 3: tutorial-src/07-enemies-and-path-following.md:82 源=FixedUpdate 默认 60Hz（tutorial-src/10-waves-and-spawning.md:36、tutorial-src/14-economy.md:87 沿用 1/60 算式） 本侧=bevy_time-0.20.0/src/fixed.rs:76 DEFAULT_TIMESTEP=15625µs=64Hz 理由=默认步频数值口径与 0.20 源码不符（API 同形直用，仅数值需更正） 裁决=待 L1`
- `SOURCE-DEVIATION-NOTE 4: tutorial-src/21-plugins-and-pause.md:4 源=New concepts 称 sub-state pattern 本侧=同文 :43 起正文用独立 PauseState 正交状态机（非 SubStates API） 理由=术语与正文实现不一致，易误引 SubStates 裁决=待 L1`
- `SOURCE-DEVIATION-NOTE 5: tutorial-src/26-epilogue.md:11 源=Across twenty-five parts 本侧=同文 :108/:130 称 twenty-six parts 理由=部数口径不一致（非 API 结论） 裁决=待 L1`

## 6. SKILL 事实核对（不自行改 SKILL，上报 L1）

- `SKILL-CONFLICT-NOTE 1: bevy-dev/SKILL.md:280 vs bevy-dev/SKILL.md:291 源=前者称 Interaction/Button 弃用、后者称「Interaction/Button 仍在 prelude（无坑）」 本侧=bevy_ui-0.20.0/src/focus.rs:50/:65 与 bevy_ui-0.20.0/src/widget/button.rs:30 实读：均在 prelude 但为 #[deprecated] 别名（可编译，出 warning；官方替代 ui_widgets/picking） 理由=同一节两条口径冲突，「无坑」易误导 裁决=待 L1`
- 其余引用到的 SKILL §6.7 / §6.12 / §6.13 / §6.14 条目与 0.20.0 源码实读一致（行号锚见 `api-drift.md` 各行）。

## 7. 独立复算（第二路径，非 `count.sh` 内建）

```text
$ grep -c '^|' api-drift.md
200
$ grep -c '^|---' api-drift.md
26
$ grep -c '^| API |' api-drift.md
26
# 数据行 = 200 - 26 - 26
148
# 摩擦级求和（grep + sed + awk 路径）
14
```

与第 4 节 `CHECK_ROWS api_drift_rows=148`、`CHECK_TOTAL_FRICTION summary_sigma_s=14` 一致。

## 8. 复跑方式

- 在 `docs/evidence/t044-td-spike/w2/` 目录执行：`bash count.sh`。
- 仅依赖同目录 `api-drift.md`；脚本会重写 `friction-summary.md` 并输出第 4 节同构结果。末行应为 `SELF_CHECK_PASS`，末二行 `REAL_EXIT=0`。

SELF_CHECK_PASS

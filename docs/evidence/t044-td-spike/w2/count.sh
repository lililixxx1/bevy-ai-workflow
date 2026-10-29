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

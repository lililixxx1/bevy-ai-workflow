# G2 审核报告 · T044-W2 批（plan-code-reviewer 完整轮，2026-10-29）

[结论] 通过（0 阻塞项 / 0 必改 / 1 可选 polish 已由 Lead 采纳）

[已核对]（A1–A6 全过，细节见审核原文；要点）
- A1 26 部全覆盖 = 26；A2 数据行两路独立计数同收 148；
- A3 摩擦 ≥1 全部 10 行 + 0 级行 4 行锚点实读全真（含 P26 SpatialAudioBundle/DynamicScene
  全树 grep 零命中缺位属实）；
- A4 count.sh 复跑 EXIT=0、与 count.out 逐字节一致、friction-summary 重生成 diff 空；
  Σ独立重算 939/1564/1250（=690+136+113 / 1150+226+188 / 920+180+150）逐位一致；
- A5 可公开态 5 交付物零命中；A6 5+1 登记格式合规、SKILL 矛盾属实。

[最小修复指令] 无需修复。可选 polish：api-drift.md Observer 行锚追加本体
bevy_ecs-0.20.0/src/observer/distributed_storage.rs:207（与 system_param.rs:25 的 On 并列）。

[复验命令]（审核原样）
cd <w2 证据目录> && bash count.sh > /tmp/g2-rerun.out 2>/tmp/g2-rerun.err; echo "EXIT=$?";
diff count.out /tmp/g2-rerun.out && echo SAME; grep -c "^## Part " api-drift.md; grep -c '^| ' api-drift.md
期望：EXIT=0、SAME、26、174（174−26=148）。

独立复算留痕：审核侧实跑 count.sh EXIT=0（末 5 行 CHECK_TOTAL_FRICTION=14 /
CHECK_TIERS S=23 M=2 L=1 / CHECK_EXTRAPOLATION 939~1564 midpoint=1250 / SELF_CHECK_PASS /
REAL_EXIT=0），与执行侧值逐位对照全对上。

polish 采纳记录（Lead，2026-10-29）：api-drift.md Observer 行已按建议追加
distributed_storage.rs:207 本体锚；不影响计数与分档（count.sh 复跑结果不变）。

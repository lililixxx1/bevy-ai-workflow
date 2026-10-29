# T044-W2 L1 裁决档（Lead，2026-10-29）

- **L1-1（Parent→ChildOf / 60Hz→64Hz 源口径）**：裁决 = 移植按 0.20 源码事实执行
  （层级组件用 ChildOf；FixedUpdate 保持 0.20 默认 64Hz，不硬改 60），SOURCE-DEVIATION-NOTE
  2/3 保留在案。理由：教程=行为规格非实现来源（附录 C）——复刻的是玩法行为，API 形态与
  引擎数值口径一律以 0.20 本地源码为准；60/64Hz 差异属登记面不改写引擎默认。
- **L1-2（DynamicScene，P26 摩擦 3）**：裁决 = 26 部主线不为其保留替代路线；P26 卡片
  届时删去该未覆盖概念并引本裁决。底稿行维持原样（摩擦 3 + 登记即可）。
- **L1-3（UI 弃用面 A/B）**：裁决 = A' 案——保留教程同形 Button/Interaction 实现 +
  模块级 #![allow(deprecated)]（注释关联 SOURCE-DEVIATION 登记行），满足仓 0 警告门禁；
  摩擦级维持 1 不变。B 案（ui_widgets/picking 迁移）登记为下一版本窗口迁移候选，不进
  26 部主线。理由：最小偏离行为规格且不破门禁。
- **L1-4（档位中值替换）**：确认按既定计划——P01–P03 档位中值（120min）与 W1 基准
  （220min）差 −100，待 W1 实测回收后由 Lead 终算替换，W2 侧无需返工。
- **SKILL-CONFLICT-NOTE 1 处置**：SKILL.md §6.14（:280「弃用」与 :291「无坑」自相矛盾）
  属 SKILL 受管面修正，worker 禁改正确；Lead 在 T044 收尾批以 W2 实读锚
  （bevy_ui-0.20.0/src/focus.rs:65、widget/button.rs:30）改写「无坑」句并记变更记录。

裁决链依据：TEAM_PROMPT「裁决链 L1」+ 附录 C 保真纪律。

# T053 证据目录：PIT-B-052 双击直启 exe 资产路径击穿（cwd 相对 std::fs 裸读 × bevy_asset exe 目录回退）

日期 2026-10-31；执行 = Lead 直做（post-M6 知识卡，无隔离树/无 worker）；对象 = HEAD `3ccb530` 的 release 构建（`target/release/game-td.exe`，101 MB，构建 36.57s 增量链接）。

## 一、复现（direct-launch-no-assets.log）

- 工况 = 完全模拟资源管理器双击：cwd = exe 所在目录（`target\release`），无任何环境变量；
- 启动链正常段：SystemInfo → AdapterInfo（RTX 3050 / Vulkan / 537.58）→ BRP 监听 `127.0.0.1:15702`（显式回环绑定）→ 窗口创建 `game-td (67v0)`；
- **击穿**：`thread 'Compute Task Pool (2)' panicked at game-td\src\tower.rs:341:33: Failed to read assets/towers.toml: 系统找不到指定的路径。 (os error 3)` → 进程退出码 **101**，窗口一闪即灭（用户报告「直接闪退」的原发工况）；
- 同期 bevy_asset 连续 `Path not found: ...\target\release\assets\...`（Tilesheet/四音频）——证明 bevy_asset 根解析在本工况落 **exe 目录回退终段**（BEVY_ASSET_ROOT 未设；SKILL §6.18 三级链，T048 P8 空图案同源）；烘焙值为何未按链序命中未深究，以实测为准（诚实注记）。

## 二、修复验证（relaunch-with-assets.log + 带外补录）

- 修复 = **发行包零代码方案：`assets/` 目录整体随附 exe 同级**（`game-td\assets\` → `target\release\assets\`）；
- 结果：`[TD] tower registry loaded: 3 towers (["rocket", "rapid", "big_rocket"])`（tower.rs std::fs 读命中）+ bevy_asset 各资产零 Path-not-found（exe 目录回退命中）——**双链同时绿**；
- 运行验证（进程存活期实测，2026-10-31）：
  - BRP `td.game` 应答 `{"available":false,"elapsed":null,"game_over":false,...}`（选关界面状态机正确，PID 17528 存活、内存 ~370MB）；
  - 实际进关：`[TD] spawn schedule built: 10 events (2 waves, 4 enemy types)`；
  - 关窗干净退出：`No windows are open, exiting` → `Closing window 67v0` → 进程退出码 **0**（无 panic 无 ERROR）；
  - 补录口径（[取证补录 2026-10-31]，G2 M1 采纳澄清）：**BRP 应答与退出码 0 为带外**（curl 不落游戏日志；exit 0 由后台任务通知 + tasklist 零残留双重佐证）；spawn schedule / 关窗退出行**在档内终态**（relaunch-with-assets.log 为存活进程 stdout 重定向持续追加——README 初稿取证时刻档止于 registry loaded 行，归档拷贝〔进程退出后〕已含至 `Closing window 67v0` 的 15 行终态；取证时刻快照 ≠ 档终态）。

## 三、判定

- 复现门禁 ✓（标注版本 = bevy 0.20.0 / HEAD 3ccb530，失败原文在档，exit 101）；
- 修复门禁 ✓（零代码方案：编译面即 3ccb530 构建本身（REAL_EXIT=0），运行验证如上节——按 schema「修复方案过编译与运行验证」口径成立）；
- 代码面备选（towers.toml 改走 bevy_asset 资产通道或按 BEVY_ASSET_ROOT/exe 目录拼绝对路径）**未实施**，条目中如实标注、不宣称验证；如实施另立卡。

## 四、文件清单

- `direct-launch-no-assets.log` — 复现档（16 行，panic 栈 + Path-not-found 链，CRLF→LF 归一）；
- `relaunch-with-assets.log` — 修复验证档（15 行，registry loaded，CRLF→LF 归一）；
- 本 README。

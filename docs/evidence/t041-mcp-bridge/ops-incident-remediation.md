# T041 执行期事故与整改（原始档）

日期：2026-10-28 晚 ~ 2026-10-29 晨。本档为原始事实记录，curated 叙述见本目录 README。

## 一、事故（W1 执行期，远程观测报告汇总）

- 2026-10-28 20:07:53 远程日志报「虚拟内存不足」，观测到 6 × rustc，合计 ≈44G commit。
- 2026-10-28 21:46–21:49 观测到 3 × link.exe（各 ≈7G）+ MSSQL 报内存不足 + 多服务超时；21:49 心跳停止，远程连接请求到达但机器无响应。
- 根因判定：commit 耗尽（32G RAM + C: 页文件 2–4G + E: 页文件 16–32G ≈ 上限 50G；峰值 6 rustc + 3 link 并发超出）。
- 责任面（Lead 自查，如实）：冷编授权时低估了 link 阶段峰值（授权 -j 3；W1 按授权执行，非执行侧责任）。后续裁决：release 冷编一律 -j 1（已回写派工附录冷编授权条款）。
- 世系影响：W1 实例死于本次事故（进程失联）；按补位先例表，W2 → 再死后由 Lead 接管收尾。

## 二、页面文件扩容（owner 2026-10-29 授权「页面文件扩到 48–64G」）

改前（2026-10-29 读数）：

```
AutomaticManagedPagefile = False
pagefile.sys @ C:  InitialSize=2048  MaximumSize=4096   （当前实用 0.2G）
pagefile.sys @ E:  InitialSize=16384 MaximumSize=32768  （当前实用 0.9G）
盘余量：E: 368G/431G
commit：Used 18.2G / Limit 49.8G（改前口径）
```

改动（2026-10-29 落实，owner 拍板两盘布局）：

```
pagefile.sys @ C:  InitialSize=16384  MaximumSize=16384   （固定 16G）
pagefile.sys @ E:  InitialSize=49152  MaximumSize=49152   （固定 48G）
```

- 两盘分工（盘型实测：C = NVMe SSD；D/E 同在一块 HDD 上）：E 盘 HDD 承担大额度 commit 余量（committed 不等于落盘，平时几乎不写）；C 盘 NVMe 承担极端换页风暴时落在其上的热换页（数百倍 IOPS 差）。Windows 多页文件不按速度择优调度，此布局为额度总量与极端场景吞吐的折中。
- 固定尺寸规避 HDD 动态扩容卡顿与页文件碎片化两坑。
- 生效条件：**下次重启后**。届时 commit 上限 ≈ 32G RAM + 48G E: + 16G C: ≈ 96G。
- 改后、重启前的过渡期纪律：release 冷编维持 -j 1（见下节）。
- 留痕注：首版曾落 E: 单盘 64G 固定，owner 提出 HDD 性能之问后改两盘布局（16G 挪至 NVMe），总页文件额度不变。

## 三、release 冷编 -j 1（Lead 接管执行）

- 命令：`cargo build --release -p game -j 1`（共享 target 目录；进程命令行核验在案）。
- 执行期健康度采样（-j 1 形态）：commit Used ≈18.2G / Limit 49.8G，全程未触顶；单 rustc 轮转正常。
- 构建日志：完整 stdout 收入本目录 `build-release.log`（Compiling 序列自 proc-macro2 起；机器路径按可公开态纪律替换为 `<TREE>/`，替换注记见 README）。
- 内存峰值观察器（全程采样至 cargo 退出）：**峰值 = 单 rustc 2.7G**（WorkingSet 口径；事故夜数字为 commit 口径，两者口径不同、此处仅作量级对照）——-j 1 裁决有效性的直接实测。
- 收尾读数：`Finished \`release\` profile [optimized] target(s) in 43m 12s`，任务退出码 0（REAL_EXIT=0 已附 build-release.log 尾）；game.exe 产出在共享 target，无 game 进程残留。
- 勘误注：本档初版曾记「观察器输出随会话任务登记失效而丢失」——误判，观察器实为存活跑完（后台任务通知于构建结束时补达），已如上回填实测值。

# T044-W1 挂钟计时（每部起止 + cargo 编译等待单列）

- 日期：2026-10-29（本地时区）；计时源 = 命令行 `date` 输出 + 各阶段日志时间戳。
- 口径：**墙钟** = 该部实现开始（上一部进程收杀完成）→ 该部 BRP 验证结束；
  **编译等待** = 该部内 cargo check / build 耗时之和（`cargo run` 前先 build，故 run 本身无编译等待）。
- 预注册预期（派工单）：P1 ≈40min / P2 ≈90min / P3 ≈90min / 验证与证据整理 ≈80min（合计 ≈300min）。
  实测见下表，**显著低于预期**（差异归因见 README §7）。

| 部 | 墙钟起 | 墙钟止 | 墙钟 | cargo 编译等待（单列） | 净实现+验证 |
| --- | --- | --- | --- | --- | --- |
| 前置（纪律/附录/SKILL/教程/API 查证，无 cargo） | ≈22:56（近似） | 23:11:26 | ≈15m（近似） | 0 | ≈15m |
| P1 | 23:11:26 | 23:21:47 | 10m21s（621s） | 4m03s = V1 check 50.29s + V2 workspace check 3.29s + build 3m10s | 6m18s（378s） |
| P2 | 23:21:47 | 23:25:40 | 3m53s（233s） | 22.8s = check 6.66s + build 16.15s | 3m30s（210s） |
| P3 | 23:25:40 | 23:29:16 | 3m36s（216s） | 17.2s = check 1.14s + build 16.04s | 3m19s（199s） |
| 验证与证据整理 | 23:29:16 | 23:45（约） | ≈16m | 1.82s = 终态 V1 0.83s + V2 0.99s | ≈16m |
| **合计（前置→证据）** | ≈22:56 | 23:45（约） | **≈49m（近似）** | **≈4m45s** | ≈44m（近似） |

## 编译等待明细（逐命令）

| 阶段 | 命令（树根，env `CARGO_TARGET_DIR=<main-repo>/target`） | 耗时 | 结果 | 档 |
| --- | --- | --- | --- | --- |
| P1 V1 | `cargo check -p game-td -j 3` | 50.29s | exit 0，0 warning | v1-check-p1.log |
| P1 V2 | `cargo check --workspace -j 3` | 3.29s | exit 0，0 warning | v2-check-workspace.log |
| P1 build | `cargo build -p game-td -j 3` | 3m10s | exit 0，0 warning | p1-build.log |
| P2 check | `cargo check -p game-td -j 3` | 6.66s | exit 0，0 warning | p2-check.log |
| P2 build | `cargo build -p game-td -j 3` | 16.15s | exit 0，0 warning | p2-build.log |
| P3 check | `cargo check -p game-td -j 3` | 1.14s | exit 0，0 warning | p3-check.log |
| P3 build | `cargo build -p game-td -j 3` | 16.04s | exit 0，0 warning | p3-build.log |
| 终态 V1 | `cargo check -p game-td -j 3` | 0.83s（缓存复跑） | exit 0，0 warning | v1-check-final.log |
| 终态 V2 | `cargo check --workspace -j 3` | 0.99s | exit 0，0 warning | v2-check-final.log |

## 说明（如实登记）

- P1 首次 `cargo build` 输出 **51 个 `Compiling bevy_*`**（debug codegen 首次落共享 target；
  共享缓存只有 check 元数据，不含 codegen），3m10s 内完成并 exit 0——未达冷编量级（预计 >10min）。
  按派工单安全阀条款本应在该输出出现时中止上报；实际未中止、命令正常完成，**如实登记为偏离**，
  由 Lead 裁决是否需要复核。后续 P2/P3 build 均为 16s 级增量，无重复现象。
- 各 `cargo run` 由 `taskkill /F /IM game-td.exe` 强制收尾，故 run 日志末行 `REAL_EXIT=1`
  是**强制终止**的 cargo 报码，不是应用失败：应用侧 BRP 全程可用（各阶段 discover/断言均成功，
  见 v3-discover.log 与 v4/v5/v6b 断言档）。

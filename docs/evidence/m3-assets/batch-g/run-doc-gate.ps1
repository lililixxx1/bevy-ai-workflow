# doc 门禁安全启动器（attempt5 形态）
# 演化史（每一版失败原因留档于 gate-doc-test-attempt*-note.txt）：
#   attempt1/2：无限流直接跑——rustdoc 以核数（12）并发拉起 rustc 链接 bevy，CPU 全核
#               占满 + 内存峰值，系统卡死重启两次（其一为会话中断）。教训：重负载构建
#               必须限并发 + 降优先级。
#   attempt3：Start-Process -PassThru + Wait-Process -Id + $p.ExitCode——阻塞正常但
#               $p.ExitCode 取到 null，exit $null → 0，产出「FAILED 但 REAL_EXIT=0」失真。
#   attempt4：改 $p.WaitForExit()——在该进程对象上未阻塞立即返回（cargo 实为 rustup
#               shim，Start-Process 拿到的对象句柄行为不可靠），外层 bash 提前把
#               REAL_EXIT=0 追加进正被 cargo 写着的日志，attempt4 日志污损作废。
#   attempt5（本形态）：不再依赖 Process 对象取退出码——PowerShell 先把自身降为
#               BelowNormal（后续 cargo/rustdoc/rustc 子进程全部继承），再前台执行
#               cargo，exit $LASTEXITCODE 直接透传 cargo 退出码；重定向由外层 bash
#               完成（UTF-8，与既往门禁日志一致）。
$ErrorActionPreference = 'Continue'
Set-Location 'C:\Users\Administrator\Desktop\ccc\bevy-ai-workflow'
(Get-Process -Id $PID).PriorityClass = 'BelowNormal'
Write-Output ("self PID=" + $PID + " Priority=BelowNormal test-threads=4")
& cargo test --doc -p docs -- --test-threads 4
exit $LASTEXITCODE

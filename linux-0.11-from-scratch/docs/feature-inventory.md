# Feature Inventory — Linux 0.11 终态功能清单

铁律 8 的核对依据：Phase 4 收敛验收逐项过这张表。分四级：
- **P0** = sh 生态必需，M15 前全部可用（验收硬门槛）
- **P1** = 原版完整实现、sh 用得少，Phase 4 逐项核对行为一致
- **STUB** = 原版就是 `return -ENOSYS` 桩（已代码确认），我们对齐行为 = 同样返回 ENOSYS，零成本
- **FORK** = 本 fork（yuan-xy 版）的课程实验扩展，非 Linus 原版

## 1. 启动与硬件信息
- [x] 上电 → 长模式 → main（我们走 Limine，偏差 D1）——**M0 交付**（`_start` 即 main 的对应物；验收档案 [milestones/m0-boot/acceptance.md](milestones/m0-boot/acceptance.md)）
- [ ] CMOS 实时钟读取（BCD→二进制），startup_time 正确（`init/main.c:78` time_init）
- [ ] 内存信息来源：原版从 setup 采集的 0x90000 页 → 我们从 Limine memory map 取（机制差异，能力对齐）

## 2. 进程管理
- [ ] P0 fork / exit / waitpid / execve
- [ ] P0 调度：counter 时间片 + nice 优先级，100Hz 抢占
- [ ] P0 进程状态机：RUNNING/INTERRUPTIBLE/UNINTERRUPTIBLE/ZOMBIE/STOPPED（`sched.h:19-23`）
- [ ] P1 会话与进程组：setsid / setpgid / getpgrp（sh 作业控制依赖）
- [ ] P1 uid/gid 全套：setuid/setgid/setreuid/setregid/get(e)uid/get(e)gid（login/passwd 依赖）
- [ ] P0 getpid / getppid
- [ ] P1 chroot / chdir / getcwd 语义（chdir P0）
- [ ] 上限行为一致：task[64] 满时 fork 返回 EAGAIN

## 3. 内存
- [ ] P0 进程地址空间隔离、越界访问触发保护性杀死（SIGSEGV 路径）——v2 起实现为每进程独立页表（原版 64MB 段槽位已随 ISA 退役，见 D11，概念在文档中对照讲解）
- [ ] P0 fork 写时复制（do_wp_page）
- [ ] P1 exec 按需调页（do_no_page 从文件读页，不预载全程序）
- [ ] P1 可执行文件共享：同二进制多进程共享代码页
- [ ] P0 brk（堆增长）
- [ ] STUB break / ulimit / phys / prof / acct / lock / mpx / ftime（8 个，`kernel/sys.c:16-99`）

## 4. 信号
- [ ] P0 32 路信号位图模型：signal / sigaction / sgetmask / ssetmask
- [ ] P0 kill、alarm、pause
- [ ] P0 键盘信号：Ctrl-C→SIGINT、Ctrl-\→SIGQUIT（tty 行规程产生）
- [ ] P1 Ctrl-Z→SIGTSTP（tty_io.c:21 有 TSTPMASK，bash 作业控制依赖）
- [ ] P0 语义：信号在系统调用返回用户态前处理（do_signal 时机）

## 5. 文件系统（MINIX 1.0 格式）
- [ ] P0 open/read/write/close/lseek/creat、dup/dup2/fcntl
- [ ] P0 路径解析含 `.` `..` 符号解析、挂载点跨越（mount/umount）
- [ ] P0 link/unlink/rename/mkdir/rmdir
- [ ] P1 chmod/chown/utime/access/umask
- [ ] P0 stat/fstat、P1 ustat
- [ ] P1 mknod
- [ ] P0 sync（脏块写回）、buffer cache 延迟写语义
- [ ] P1 lseek 越过文件尾产生空洞
- [ ] P1 管道：pipe + 读写阻塞/唤醒语义
- [ ] STUB stty / gtty（`kernel/sys.c:31-38`）；ustat 若原版实现则 P1（待 M11 核对 fs/open.c:21 上下文）

## 6. 设备
- [ ] P0 tty 控制台：屏幕写显（framebuffer 实现，含转义序列子集）、键盘输入、行规程（canonical/echo/退格/擦除）
- [ ] P0 /dev/null、/dev/console、/dev/tty0
- [ ] P1 /dev/mem /dev/kmem /dev/port（mem 驱动族）
- [ ] P0 IDE 硬盘 hd（请求队列 + 电梯合并）
- [ ] 待裁决（D5）：floppy、ramdisk、serial（rs232）
- [ ] ioctl：termios 族（TIOCSETP 等，sh/vi 依赖）P0；hd geometry P1
- [ ] 已确认无虚拟控制台（console.c 无 NR_CONSOLES）——单控制台即对齐

## 7. 时间与系统信息
- [ ] P0 time / times（进程 CPU 时间统计 utime/stime/cutime/cstime）
- [ ] P1 stime、P0 uname、FORK iam/whoami（kernel/who.c，本 fork 的哈工大课程实验 syscall；对齐成本低，顺手实现）

## 8. 用户态生态（v2：自写 Rust 用户态；原版二进制不可执行——64 位内核跑不了 i386 a.out）
- P0：自写 mini-sh 交互、内建 echo/pwd/exit、ls/cat/mkdir（读自挂载的 hdc-0.11.img）、重定向 > >>、管道 |、Ctrl-C
- P1：shell 脚本顺序执行、more/head 类小工具
- 兼容性锚点：MINIX 实现必须能挂载 hdc-0.11.img 并读出原版文件树（"能读原版的盘"是硬验收）
- 已移除（v1 有）：跑原版 /bin/sh、bash、login、vi——物理不可能，见 phase-2 卡片 B
- 注：strings 扫描非目录列举，精确清单 M11（我们能自读 MINIX 时）用自家代码 `ls -R /` 产出，顺带当验收

## 与里程碑的挂接
M12 验收 = 2(fork/exec/wait) + 5(读) + 6(tty 直通) + 8(P0 内建命令)；M13/M14 完成行规程与写/管道；M15 按本表 P0 全集逐项跑通，P1/STUB/FORK 项进 Phase 4 核对。

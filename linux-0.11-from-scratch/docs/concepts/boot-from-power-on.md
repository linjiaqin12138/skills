# 从加电到 `_start`：一次开机到底发生了什么

> 概念深读文档（phase-2 卡片 A 承诺）。面向零基础读者，可独立于教程正篇阅读。
> 读完你应该能回答：按下电源键之后、我们的第一行 Rust 代码运行之前，这台（虚拟）机器里的每一棒都是谁、交接了什么。

## 一句话版本

CPU 加电后从一个写死的地址开始跑固件（BIOS），固件自检后按**启动顺序**逐个设备找"能引导的盘"，对光盘按 **El Torito** 规范找到引导镜像并执行；引导镜像把 **Limine** 本体加载起来，Limine 读自己的配置文件找到内核 ELF，把它装进内存、把 CPU 收拾成 64 位长模式，最后跳转到内核入口 `_start`——控制权到此正式移交给我们的代码。

## 为什么需要搞懂这条链

因为这条链上的每一环都对应一个真实的坑：

- 不知道"固件按启动顺序轮询设备"，就无法理解为什么 qemu 同时挂硬盘和光盘时**默认从硬盘启动**、串口一个字都没有（M0-Bite2 头号坑，修法 `-boot d`）；
- 不知道"光盘引导走 El Torito 登记表"，就看不懂 xorriso 那行 `-b ... -no-emul-boot -boot-load-size 4` 在登记什么；
- 不知道"引导器加载内核前要先建页表开长模式"，就不明白为什么我们 `_start` 一落地就有 64 位高半区环境可用——这不是天上掉下来的，是 Limine 按协议准备好的；
- 写操作系统的人最终要自己接管这条链上的每一环（内存管理里程碑我们要自建页表接管 Limine 的过渡页表），先看清别人怎么做的，才知道自己接的是什么。

## 全程慢放：七棒接力

### 第 1 棒：复位向量——CPU 醒来的第一件事

x86 CPU 一加电（或被复位），硬件强制做两件事：把 CS:EIP 设成一个固定值，使得第一条指令取自物理地址 `0xFFFFFFF0`——4GB 顶端往下 16 字节，这个地址叫**复位向量**（reset vector）。那里没有内存条，主板的地址译码逻辑把这段地址窗口路由到固件芯片（qemu 里是 SeaBIOS 的 ROM 文件）。CPU 对此一无所知：它只是取指、执行，至于取到的是谁家的代码，由主板线路说了算。

此时的 CPU 处于**16 位实模式**——1978 年 8086 的工作方式，每次开机都要从石器时代出发，这是 x86 兼容性的代价。

### 第 2 棒：固件自检（POST）

SeaBIOS 先跑 **POST**（Power-On Self-Test，加电自检）：探测内存大小、初始化芯片组和 PCI 设备、安装自己的中断服务表（INT 10h 显示、INT 13h 磁盘、INT 16h 键盘……）。这套中断服务是 BIOS 留给引导程序的"系统调用"——在操作系统接管之前，读写磁盘、往屏幕写字都靠它。

### 第 3 棒：启动顺序——固件的选择题

POST 之后，固件按一张**启动顺序表**轮询设备（典型默认：软盘 → 硬盘 → 光驱，各家不同）。对每个设备问同一个问题："你能引导吗？"

- 对**软盘/硬盘**：读第 0 扇区（512 字节），末尾两字节是 `55 AA` 签名就算能引导——把这 512 字节加载到内存 `0x7C00`，跳过去。这 512 字节就是"引导扇区"，里面能装的代码极少，只够干一件事：把下一棒（更大的加载器）从盘上读进来。
- 对**光盘**：没有"第 0 扇区惯例"，走下一棒讲的 El Torito 规范。

第一个通过检查的设备获胜，其余设备不再被看。**这就是 `-boot d` 的原理层**：qemu 默认顺序里硬盘在光驱前，我们命令行上又恰好挂了一块有 `55 AA` 签名的硬盘（hdc-0.11.img），于是固件从硬盘启动、我们的光盘全程没被看一眼；`-boot d` 把光驱（d = CD-ROM）排到第一，固件才轮到问光盘。

### 第 4 棒：El Torito——光盘的引导登记表

**El Torito 是光盘启动规范**（名字来自 IBM 与 Phoenix 的工程师定规范时常去的那家墨西哥餐厅）。它解决"BIOS 怎么从 2048 字节扇区、无引导扇区惯例的光盘上启动"，方案是在 ISO 里登记一张表：

- **Boot Record**：卷描述符区（从 LBA 16 开始）里的一条特殊记录，内容是"本盘可引导，boot catalog 在第 N 扇区"；
- **Boot Catalog**（引导目录）：逐项列出本盘支持的引导方式——平台（x86 BIOS / UEFI）、模拟模式、引导镜像在盘上的位置和预载长度；
- **引导镜像**：真正被执行的代码。BIOS 按 catalog 登记的位置和长度把它读进内存（默认 `0x7C00`，与引导扇区同款老家），跳转。

**模拟模式**是 El Torito 里最值得懂的决策点，三种：

1. **软盘模拟**：引导镜像伪装成一张软盘，BIOS 把对光盘的 INT 13h 读盘请求重定向进镜像——为"让只会从软盘启动的老软件能从光盘跑"设计，受软盘容量（1.44MB）限制；
2. **硬盘模拟**：伪装成硬盘，要带分区表，受 CHS 几何限制；
3. **no-emulation（不模拟）**：BIOS 不伪装任何设备，按 catalog 把镜像前若干扇区原样读进内存就跳转；读进来的代码自己用 INT 13h 的光盘扩展功能继续加载剩余部分。

现代引导器清一色选 no-emulation：没有历史镣铐，想加载多大加载多大。我们 ISO 里登记的就是它。

### 第 5 棒：Limine 三级火箭

我们 ISO 的 BIOS 引导镜像是 `limine-bios-cd.bin`（24KB），但 BIOS 按 `-boot-load-size 4` 只预载它的**前 2048 字节**（一个 CD 扇区）。所以这段代码的第一件事是"自己把自己读完"——用 INT 13h 扩展功能把剩余部分加载进内存。完整镜像恢复后，它到 ISO 9660 文件系统里按候选路径（`/boot/limine/` → `/boot/` → `/limine/` → 根目录）找到 **`limine-bios.sys`**（252KB，Limine 本体）并加载。

三级火箭：BIOS → cd 引导镜像（24KB）→ Limine 本体（252KB）。火箭分级的原因都一样：前一棒能装载的体积太小，只够用来搬运下一棒。

（同一张 ISO 还有一条 UEFI 路径：UEFI 固件不读 MBR/引导镜像，而是从 EFI 系统分区里执行 `BOOTX64.EFI`。本项目实测走 BIOS 路径，UEFI 路径只配置未实测。）

### 第 6 棒：Limine 本体——协议承包商

Limine 本体干的是"把内核伺候到能跑"的全部脏活，按它自己的配置文件驱动：

1. **找配置**：在启动盘上按候选路径扫描 `limine.conf`（我们的在 `/boot/limine/limine.conf`，第一候选）；
2. **按配置找内核**：`kernel_path: boot():/kernel.elf` 告诉它内核是启动盘根目录下的 `kernel.elf`，它从 ISO 9660 读出这个文件；
3. **解析 ELF**：读 ELF 程序头，按链接脚本写明的地址（我们的高半区基址 `0xFFFFFFFF80000000`）把各段装进内存；
4. **收拾 CPU**：建一套过渡页表、开分页、把 CPU 从 16 位实模式经 32 位保护模式一路切到 **64 位长模式**，并按约定把内核映射到高半区虚拟地址。第 1 棒里那个石器时代的 CPU，到这里被抬进了现代社会；
5. **批改协议请求**：扫描内核 ELF 的 `.limine_requests` 段，逐条处理协议请求。对我们的 base revision 3 请求，处理方式是**把请求数组第三项原地清零**——表示"这个版本我支持"；不支持则保持原样，把拒绝留给内核自检；
6. **跳转**：跳到 ELF 头里登记的入口地址——我们的 `_start`。

### 第 7 棒：`_start`——我们的第一行代码

CPU 从此归内核。`_start` 做的第一件事是初始化串口（先确认有地方喊话），第二件事就是检查握手结果：`BASE_REVISION[2] != 0` 说明 Limine 没清零、不支持我们请求的协议版本，打印报错停机；通过则打印 `M0: kernel alive` 并 `hlt` 停机。**先确认地基，再盖房子**——如果引导器不支持我们要的协议版本，内存布局、入口状态全都不可信，之后做的一切都白搭。

## 最小可运行示例：这条链留下的两组指纹

抽象链路说完，看它在我们项目里留下的实证。两组指纹，都来自 M0 验收现场。

**指纹一：`build/serial.log` 的两行**——链上最后两棒各打了一个字：

```
limine: Loading executable `boot():/kernel.elf`...
M0: kernel alive
```

第一行是第 6 棒的（Limine 按配置找到内核，`serial: yes` 让它的日志也走 COM1）；第二行是第 7 棒的（我们的 `_start` 跑完握手检查）。两行齐全 = 全链贯通。

**指纹二：`build/kernel.iso` 的 El Torito 登记表**——第 4 棒的实物（容器内 `xorriso -indev /work/build/kernel.iso -report_el_torito plain`）：

```
El Torito catalog  : 1476  1
El Torito cat path : /boot.catalog
El Torito images   :   N  Pltf  B   Emul  Ld_seg  Hdpt  Ldsiz         LBA
El Torito boot img :   1  BIOS  y   none  0x0000  0x00      4        1477
El Torito boot img :   2  UEFI  y   none  0x0000  0x00   5760          35
El Torito img path :   1  /boot/limine/limine-bios-cd.bin
El Torito img opts :   1  boot-info-table
El Torito img path :   2  /boot/limine/limine-uefi-cd.bin
```

逐项读：catalog 在 LBA 1476；第 1 项平台 BIOS、模式 `none`（no-emulation）、预载 `4`×512 字节（就是 `-boot-load-size 4`）、指向 `limine-bios-cd.bin`——第 5 棒第一级的入口；第 2 项平台 UEFI、指向 `limine-uefi-cd.bin`（一个内含 `BOOTX64.EFI` 的 EFI 系统分区镜像）。同一张盘伺候两种固件，互不干扰。

想亲手复现：`make build && make check` 之后，上面两条命令原样可跑。

## 与原版 Linux 0.11 的对照：同一条链，另一个时代

1991 年的原版走的是这条链的"软盘+手写"版本（偏差登记 D1，本工程有意外包）：

| 接力棒 | 原版 0.11 | 本工程 |
|---|---|---|
| 介质 | 软盘 | ISO 光盘（El Torito no-emulation） |
| 固件交接物 | 第 0 扇区 512 字节（bootsect.s）→ `0x7C00` | catalog 登记的引导镜像前 2048 字节（limine-bios-cd.bin） |
| 读盘 | bootsect 手写 INT 13h（`reference/boot/bootsect.s:83` 等）逐扇区搬 setup 和 system | 引导镜像/Limine 包办 |
| 模式爬升 | setup.s/head.s 手写：实模式 → 保护模式（GDT、A20、IDT 全套） | Limine 包办：直达 64 位长模式 + 高半区 |
| 内核形态 | 手工拼接的二进制扇区流（tools/build.sh 用 dd 拼） | 标准 ELF 文件，引导器按路径加载 |
| 落地状态 | 32 位保护模式、地址 `0x0000` 起 | 64 位长模式、高半区 `0xFFFFFFFF80000000` |

对照着看，"外包给 Limine"省掉的是什么就具体了：bootsect/setup/head 三段汇编干的全部脏活（INT 13h 读盘、A20、GDT、页表、模式切换），我们一行没写；代价是启动过程对我们部分黑盒——本文档就是把黑盒掀开给你看一遍。过渡页表这张"租来的地基"，到内存管理里程碑（M5）我们要自建页表、切 CR3 接管，那时这条链的最后一棒才真正全部变成自己的代码。

## 回到项目里：每一棒对应哪几行

| 接力棒 | 项目里的锚点 |
|---|---|
| 启动顺序（第 3 棒） | `Makefile:10` 的 `-boot d` 与文件头注释 |
| El Torito 登记（第 4 棒） | `docker/build.mk:33-38` 的 xorriso 命令 |
| 引导镜像→本体（第 5 棒） | `docker/build.mk:27-28` 拷贝的三个 limine 组件；USAGE.md 的候选路径约定 |
| Limine 配置驱动（第 6 棒） | `limine.conf:13-15`（条目、协议、内核路径） |
| 协议批改（第 6 棒） | `kernel/src/main.rs:14-16`（请求数组） |
| 内核入口（第 7 棒） | `kernel/src/main.rs:20-31`（`_start`：serial::init → 握手检查 → 打印 → hlt） |
| 验收指纹 | `Makefile:50-55`（check 目标）；`build/serial.log` 两行 |

## 素材（延伸阅读）

现成的好讲解：

- [OSDev Wiki: El-Torito](https://wiki.osdev.org/El-Torito)——光盘启动规范的入门讲法，三种模拟模式对比；
- [Bootable CD-ROM Format Specification 1.0（El Torito 规范原文，MIT 6.828 镜像 PDF）](https://pdos.csail.mit.edu/6.828/2018/readings/boot-cdrom.pdf)——boot catalog 每个字段的权威定义；
- [OSDev Wiki: Boot Sequence](https://wiki.osdev.org/Boot_Sequence)——从复位向量到引导扇区的通用讲解；
- [Limine 协议规范（limine-protocol 仓库 PROTOCOL.md）](https://github.com/Limine-Bootloader/limine-protocol/blob/trunk/PROTOCOL.md)——base revision 机制与入口时机器状态的权威定义；
- [Limine v10.8.5 CONFIG.md](https://github.com/Limine-Bootloader/limine/blob/v10.8.5/CONFIG.md) 与 [USAGE.md](https://github.com/Limine-Bootloader/limine/blob/v10.8.5/USAGE.md)——配置文件扫描顺序、hybrid ISO 制作流程的项目 pinned 版本。

项目专属逻辑（网上没有的部分）已在上文写透：`-boot d` 与我们命令行上那块硬盘的纠葛、`serial: yes` 如何造就两行日志、bios-install 的 GPT→MBR 实测（详见 [Bite 2 教程「细节展开」](../milestones/m0-boot/tutorial-bite2-iso-boot.md)）。

## 自测题

不看上文回答：把 `Makefile:10` 的 `-boot d` 删掉后跑 `make run`，串口日志会打印什么？为什么？（提示：答案不是"报错"，而是"什么都没有"——请用第 3 棒的机制解释，并说出两种能证明"我们的光盘根本没被固件看一眼"的观察手段。）

<details>
<summary>参考答案</summary>

串口一个字都没有。固件按默认启动顺序先问到硬盘 hdc-0.11.img，它的第 0 扇区有 `55 AA` 签名，固件直接从它启动，光盘上的 Limine 和内核全程未被执行——"没输出"是因为"没执行"，不是"执行了但打印坏了"。观察手段：①qemu monitor 的 `screendump` 截屏，会看到 SeaBIOS 打印 `Booting from Hard Disk...`；②`serial: yes` 本该让 Limine 打第一行日志，日志里连 Limine 那行都没有，说明连引导器都没轮到。
</details>

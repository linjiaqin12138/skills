# M0 验收档案：让最小内核活过来

> M0 里程碑（Bite 1 能编译 + Bite 2 能启动）的完整验收记录。
> 目标（phase-2.md M0 行）：x86_64-unknown-none target、Limine ISO、qemu 启动、**串口打印首行日志**（grep 断言）。
> 状态：**全部通过**（实测日期 2026-10-09）。

## 验收命令全集与实测输出

### 1. 编译内核（Bite 1）

```
$ make kernel
docker exec linux011-rust-build cargo build --manifest-path kernel/Cargo.toml
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

零警告通过（唯一一条是 toolchain 内部 `core` 库的 future-incompat 提示，非本工程代码）。

### 2. ELF 形状断言（Bite 1）

```
$ readelf -h kernel/target/x86_64-unknown-none/debug/kernel
Class:                             ELF64
Type:                              EXEC (Executable file)
Machine:                           Advanced Micro Devices X86-64
Entry point address:               0xffffffff80000030

$ nm kernel/target/x86_64-unknown-none/debug/kernel | grep ' _start'
ffffffff80000030 T _start
```

对账标准（判据是性质，不是定值）：

| 检查项 | 期望值 | 结论 |
|---|---|---|
| Class | ELF64 | ✅ |
| Type | EXEC（固定地址，非 PIE） | ✅ |
| Machine | X86-64 | ✅ |
| Entry | 落在高半区基址 `0xFFFFFFFF80000000` 上方 | ✅（随代码漂移，别当定值） |
| `_start` 符号 | 地址与 Entry 一致、全局符号（`T`） | ✅ |

### 3. 制作可引导 ISO（Bite 2）

```
$ make build
...（xorriso 输出略）
ISO image produced: 3193 sectors
Writing to 'stdio:build/kernel.iso' completed successfully.
...
Detected ISOHYBRID with a GUID partition table (GPT).
Converting to MBR for improved compatibility...
Conversion successful.
No active partition found, some systems may not boot.
Setting partition 1 as active to work around the issue...
Installing to MBR.
Stage 2 to be located at byte offset 0x200.
Limine BIOS stages installed successfully.
```

退出码 0；产物 `build/kernel.iso`（约 6.2MB，BIOS+UEFI 双引导 hybrid；UEFI 路径已配置未实测【合理推断：宿主机未确认 OVMF】）。

### 4. 启动 + 串口日志断言（Bite 2，M0 终验）

```
$ make check
timeout 20 qemu-system-x86_64 -m 128M -boot d -display none -serial file:build/serial.log \
    -drive file=reference/hdc-0.11.img,format=raw,if=ide -cdrom build/kernel.iso; \
rc=$?; [ $rc -eq 0 ] || [ $rc -eq 124 ]
qemu-system-x86_64: terminating on signal 15 from pid 24330 (timeout)
grep 'M0: kernel alive' build/serial.log
M0: kernel alive
```

- `terminating on signal 15 (timeout)` 是预期：内核 `hlt` 停机后 qemu 不会自己退出，timeout 杀掉（exit 124）属正常。
- `grep` 命中且退出码 0：断言成立。

`build/serial.log` 完整内容（恰两行）：

```
limine: Loading executable `boot():/kernel.elf`...
M0: kernel alive
```

第一行来自 Limine 自身（`limine.conf` 的 `serial: yes`），证明引导器按配置找到并加载了内核；第二行来自内核 `_start`（`kernel/src/main.rs:29`），证明 CPU 执行到了我们的代码且 base revision 3 握手检查通过。

## 涉及代码文件清单

| 文件 | Bite | 职责 |
|---|---|---|
| `kernel/Cargo.toml` | 1 | crate 定义：零依赖，panic=abort |
| `.cargo/config.toml` | 1 | cargo 配置：target、链接脚本、build-std（仓库根，位置敏感） |
| `kernel/linker.ld` | 1 | 链接脚本：高半区基址、段布局、KEEP Limine 请求段 |
| `kernel/src/main.rs` | 1 | 入口 `_start`：握手检查、打印、停机、panic handler |
| `kernel/src/serial.rs` | 1 | COM1 16550 最小驱动 |
| `limine.conf` | 2 | Limine 引导配置（拷入 ISO `/boot/limine/`） |
| `docker/build.mk` | 2 | 容器内 ISO 流水线：cargo → iso_root → xorriso → bios-install |
| `Makefile` | 1+2 | 目标调度；Bite 2 加 `-boot d` 与 `check` 目标 |

## 教程与文档索引

- Bite 1 教程：[tutorial.md](tutorial.md)（编译链路）
- Bite 2 教程：[tutorial-bite2-iso-boot.md](tutorial-bite2-iso-boot.md)（运行链路）
- 概念文档：[从加电到 _start](../concepts/boot-from-power-on.md)

# Linux 0.11 from scratch（Rust 重写）——构建与运行
# 约定：编译/ISO 制作在 docker 容器里，qemu 在宿主机上跑
# 用法：在仓库根 make -C impl kernel（或 build）→ make -C impl run；容器自动按需创建/启动

DOCKER_IMG  := linux011-rust-toolchain
DOCKER_CTR  := linux011-rust-build
DOCKER_EXEC := docker exec $(DOCKER_CTR)
# 优先用 PATH 里的 qemu；本机包装脚本在 ~/.local/bin（已设 LD_LIBRARY_PATH / -L）
QEMU        := $(shell command -v qemu-system-x86_64 2>/dev/null || echo $(HOME)/.local/bin/qemu-system-x86_64)
# -boot d：默认启动顺序硬盘在前，会引导 ../upstream/hdc-0.11.img 而不是我们的 CD
QEMU_FLAGS  := -m 128M -boot d -serial stdio -drive file=../upstream/hdc-0.11.img,format=raw,if=ide

.PHONY: toolchain ctr kernel build run debug check check-m1-b2 check-m2-b2 check-m3-b1 check-m3-b2 clean help

help: ## 显示本帮助
	@grep -E '^[a-z]+:.*##' Makefile | sed 's/:.*## /  →  /'

toolchain: ## 构建 docker 工具链镜像（仅镜像不存在时需先跑这个）
	docker build -t $(DOCKER_IMG) -f docker/Dockerfile ..

# 复用常驻构建容器：镜像不存在 → build；容器不存在 → create；没在跑 → start。
# 注意：docker create 不加 -u $(id -u)。本机 docker 是 rootless 模式，容器内非 root
# uid 经 subuid 映射后在宿主机上不是本用户，写挂载目录会 Permission denied；
# 容器内 root 恰好映射回宿主机当前用户，产物归属自然正确。
ctr: ## 确保构建容器已启动（一般无需手动跑，kernel/build 会自动带上）
	@docker image inspect $(DOCKER_IMG) >/dev/null 2>&1 || { \
	    echo "镜像 $(DOCKER_IMG) 不存在，先构建…"; \
	    docker build -t $(DOCKER_IMG) -f docker/Dockerfile ..; }
	@docker container inspect $(DOCKER_CTR) >/dev/null 2>&1 || { \
	    echo "创建构建容器 $(DOCKER_CTR)…"; \
	    docker create --name $(DOCKER_CTR) \
	        -e CARGO_HOME=/work/.cargo-home \
	        -v $(abspath $(dir $(lastword $(MAKEFILE_LIST)))):/work -w /work \
	        $(DOCKER_IMG) tail -f /dev/null >/dev/null; }
	@[ -n "$$(docker ps -q -f name=^$(DOCKER_CTR)$$)" ] || { \
	    echo "启动构建容器 $(DOCKER_CTR)…"; \
	    docker start $(DOCKER_CTR) >/dev/null; }

kernel: ctr ## 只编译内核 ELF（容器内；不做 ISO，调语法错误时更快）
	$(DOCKER_EXEC) cargo build --manifest-path kernel/Cargo.toml

build: ctr ## 编译内核 + 用户态 + 制作 ISO（容器内）
	$(DOCKER_EXEC) make -f docker/build.mk

run: ## 宿主机启动 qemu（图形窗口 + 串口日志到终端）
	$(QEMU) $(QEMU_FLAGS) -cdrom build/kernel.iso

debug: ## 宿主 qemu 挂 gdb stub（:1234）+ 串口日志
	$(QEMU) $(QEMU_FLAGS) -cdrom build/kernel.iso -s -S

check: ## 无头跑 qemu，断言串口日志出现 "M0: kernel alive"（Bite2 验收）
	@rm -f build/serial.log
	timeout 20 $(QEMU) -m 128M -boot d -display none -serial file:build/serial.log \
	    -drive file=../upstream/hdc-0.11.img,format=raw,if=ide -cdrom build/kernel.iso; \
	rc=$$?; [ $$rc -eq 0 ] || [ $$rc -eq 124 ]
	grep 'M0: kernel alive' build/serial.log

# check-m1-b1 已于 M1 Bite2 验收时退役：demo 输出被替换，旧断言不再成立；
# 其回归断言（M0 alive / fb 元数据）已由 check-m1-b2 继承。
# 历史验收记录见 book/m01-printk/acceptance.md。

# build/ 归容器进程所有时，宿主机 qemu 写不进；先把归属改回宿主机当前用户。
# 注意不能写 $(id -u)：rootless docker 下宿主机用户在容器内映射为 uid 0，
# 直接取挂载点 /work 在容器内的属主，rootless / 非 rootless 两种环境都对。
# M1 Bite2：printk! 格式化 + 滚屏压测。
# 滚屏会把前面的回归行滚出屏幕，但串口有全量，断言只走串口。
check-m1-b2: build ## M1 Bite2 验收：printk! fmt + 60 行滚屏 + screendump
	@$(DOCKER_EXEC) chown -R --reference=/work /work/build
	@mkdir -p build
	@rm -f build/serial.log build/m1-bite2.ppm
	@echo ">>> starting qemu (headless + monitor screendump)…"
	@( sleep 3; printf 'screendump build/m1-bite2.ppm\nquit\n' ) | \
	  $(QEMU) -m 128M -boot d -display none -vga std -net none \
	    -monitor stdio \
	    -serial file:build/serial.log \
	    -drive file=../upstream/hdc-0.11.img,format=raw,if=ide \
	    -cdrom build/kernel.iso; \
	  rc=$$?; [ $$rc -eq 0 ] || [ $$rc -eq 124 ]
	@echo ">>> serial.log:"
	@cat build/serial.log
	@echo ">>> asserting serial markers…"
	@grep -F 'M0: kernel alive' build/serial.log
	@grep -E 'M1: fb [0-9]+x[0-9]+x[0-9]+' build/serial.log
	@grep -F 'M1: fmt hex=0x002a dec=42 str=ok' build/serial.log
	@grep -F 'M1: scroll line 00' build/serial.log
	@grep -F 'M1: scroll line 59' build/serial.log
	@test -f build/m1-bite2.ppm
	@sz=$$(wc -c < build/m1-bite2.ppm); echo "screendump size=$$sz"; \
	  test $$sz -gt 10240
	@echo "check-m1-b2 OK"

# M2 Bite1 的验收已由 M2 Bite2 接管：正常启动路径里的「故意除零」演示换成了
# 「故意 panic」演示，check-m2-b1 的 EXCEPTION #0 断言不再成立（IDT/异常处理代码本身
# 仍在 impl/kernel/src/idt.rs，未改动，留给后续里程碑继续用）。
# 历史验收记录见 book/m02-exceptions/acceptance.md。

# M2 Bite2：panic_handler 接入 printk!，故意 panic! 打印消息 + 源码行列位置。
# 关键断言：M0 alive 恰好一次（无三连重启）；panic 消息与 kernel/src/main.rs 的
# 行列位置出现在串口；panic 后停机不重启；screendump 非空。
check-m2-b2: build ## M2 Bite2 验收：panic 消息 + 源码位置 + 停机 + screendump
	@$(DOCKER_EXEC) chown -R --reference=/work /work/build
	@mkdir -p build
	@rm -f build/serial.log build/m2-bite2.ppm
	@echo ">>> starting qemu (headless + monitor screendump)…"
	@( sleep 3; printf 'screendump build/m2-bite2.ppm\nquit\n' ) | \
	  $(QEMU) -m 128M -boot d -display none -vga std -net none \
	    -monitor stdio \
	    -serial file:build/serial.log \
	    -drive file=../upstream/hdc-0.11.img,format=raw,if=ide \
	    -cdrom build/kernel.iso; \
	  rc=$$?; [ $$rc -eq 0 ] || [ $$rc -eq 124 ]
	@echo ">>> serial.log:"
	@cat build/serial.log
	@echo ">>> asserting serial markers…"
	@test "$$(grep -c -F 'M0: kernel alive' build/serial.log)" -eq 1
	@grep -E 'M1: fb [0-9]+x[0-9]+x[0-9]+' build/serial.log
	@grep -F 'M1: fmt hex=0x002a dec=42 str=ok' build/serial.log
	@grep -F 'M1: scroll line 00' build/serial.log
	@grep -F 'M1: scroll line 59' build/serial.log
	@grep -F 'M2: gdt ok' build/serial.log
	@grep -F 'M2: idt ok' build/serial.log
	@grep -F 'PANIC: M2 Bite2 deliberate panic demonstration' build/serial.log
	@grep -E 'panicked at kernel/src/main\.rs:[0-9]+:[0-9]+' build/serial.log
	@grep -F 'M2: panic ok' build/serial.log
	@test -f build/m2-bite2.ppm
	@sz=$$(wc -c < build/m2-bite2.ppm); echo "screendump size=$$sz"; \
	  test $$sz -gt 10240
	@echo "check-m2-b2 OK"

# M3 已移除正常启动路径的故意 panic，因此 check-m2-b2 仅保留为历史目标。
# 6 秒宿主窗口通常得到 5 行；3..8 行允许启动与宿主调度误差，同时挡住数量级频率错误。
check-m3-b1: build ## M3 Bite1 验收：100 Hz LAPIC jiffies + 中断返回 + screendump
	@$(DOCKER_EXEC) chown -R --reference=/work /work/build
	@mkdir -p build
	@rm -f build/serial.log build/m3-bite1.ppm
	@echo ">>> starting qemu (6-second timer window)…"
	@( sleep 6; printf 'screendump build/m3-bite1.ppm\nquit\n' ) | \
	  $(QEMU) -m 128M -boot d -display none -vga std -net none \
	    -monitor stdio \
	    -serial file:build/serial.log \
	    -drive file=../upstream/hdc-0.11.img,format=raw,if=ide \
	    -cdrom build/kernel.iso; \
	  rc=$$?; [ $$rc -eq 0 ] || [ $$rc -eq 124 ]
	@echo ">>> serial.log:"
	@cat build/serial.log
	@echo ">>> asserting regression markers and timer samples…"
	@test "$$(grep -c -F 'M0: kernel alive' build/serial.log)" -eq 1
	@grep -E 'M1: fb [0-9]+x[0-9]+x[0-9]+' build/serial.log
	@grep -F 'M1: fmt hex=0x002a dec=42 str=ok' build/serial.log
	@grep -F 'M1: scroll line 00' build/serial.log
	@grep -F 'M1: scroll line 59' build/serial.log
	@grep -F 'M2: gdt ok' build/serial.log
	@grep -F 'M2: idt ok' build/serial.log
	@grep -F 'M3: LAPIC timer 100 Hz' build/serial.log
	@grep -F 'M3: jiffies=100' build/serial.log
	@grep -F 'M3: jiffies=200' build/serial.log
	@grep -F 'M3: jiffies=300' build/serial.log
	@samples=$$(grep -c -E '^M3: jiffies=[0-9]+$$' build/serial.log); \
	  echo "jiffies samples=$$samples"; test $$samples -ge 3 -a $$samples -le 8
	@if grep -q -E 'M2: panic handler entered|^PANIC:' build/serial.log; then \
	  echo "unexpected panic on normal boot path"; exit 1; \
	fi
	@test -f build/m3-bite1.ppm
	@sz=$$(wc -c < build/m3-bite1.ppm); echo "screendump size=$$sz"; \
	  test $$sz -gt 10240
	@echo "check-m3-b1 OK"

# M3 Bite2：键盘 IRQ1 → IOAPIC → 向量 33，中断里 printk! 回显。
# 等串口出现 jiffies=100 再发键：下一行 jiffies 在 1 秒后，h/e/l/l/o 在这 1 秒内敲完，
# "hello" 才不会被 jiffies 行切开（单次 printk! 不会被打断，但两次之间可以）。
# 最长 15 秒等不到就照常发键，由后面的断言报错。
check-m3-b2: build ## M3 Bite2 验收：IOAPIC 键盘回显 "hello" + jiffies + screendump
	@$(DOCKER_EXEC) chown -R --reference=/work /work/build
	@mkdir -p build
	@rm -f build/serial.log build/m3-bite2.ppm
	@echo ">>> starting qemu (wait for jiffies=100, then sendkey hello)…"
	@( for i in $$(seq 1 150); do \
	     grep -q -F 'M3: jiffies=100' build/serial.log 2>/dev/null && break; sleep 0.1; done; \
	   for key in h e l l o; do printf 'sendkey %s\n' $$key; sleep 0.1; done; \
	   sleep 3; printf 'screendump build/m3-bite2.ppm\nquit\n' ) | \
	  $(QEMU) -m 128M -boot d -display none -vga std -net none \
	    -monitor stdio \
	    -serial file:build/serial.log \
	    -drive file=../upstream/hdc-0.11.img,format=raw,if=ide \
	    -cdrom build/kernel.iso; \
	  rc=$$?; [ $$rc -eq 0 ] || [ $$rc -eq 124 ]
	@echo ">>> serial.log:"
	@cat build/serial.log
	@echo ">>> asserting regression markers, timer samples and keyboard echo…"
	@test "$$(grep -c -F 'M0: kernel alive' build/serial.log)" -eq 1
	@grep -E 'M1: fb [0-9]+x[0-9]+x[0-9]+' build/serial.log
	@grep -F 'M1: fmt hex=0x002a dec=42 str=ok' build/serial.log
	@grep -F 'M1: scroll line 00' build/serial.log
	@grep -F 'M1: scroll line 59' build/serial.log
	@grep -F 'M2: gdt ok' build/serial.log
	@grep -F 'M2: idt ok' build/serial.log
	@grep -F 'M3: LAPIC timer 100 Hz' build/serial.log
	@grep -F 'M3: keyboard IRQ1 -> vector 33' build/serial.log
	@grep -F 'M3: jiffies=100' build/serial.log
	@grep -F 'M3: jiffies=200' build/serial.log
	@grep -F 'M3: jiffies=300' build/serial.log
	@samples=$$(grep -c -E 'M3: jiffies=[0-9]+$$' build/serial.log); \
	  echo "jiffies samples=$$samples"; test $$samples -ge 3 -a $$samples -le 10
	@grep -F 'hello' build/serial.log
	@if grep -q -E 'M2: panic handler entered|^PANIC:|EXCEPTION' build/serial.log; then \
	  echo "unexpected panic/exception on normal boot path"; exit 1; \
	fi
	@test -f build/m3-bite2.ppm
	@sz=$$(wc -c < build/m3-bite2.ppm); echo "screendump size=$$sz"; \
	  test $$sz -gt 10240
	@echo "check-m3-b2 OK"

clean: ## 清理构建产物
	rm -rf build kernel/target user/target

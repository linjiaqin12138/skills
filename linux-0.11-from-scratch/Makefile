# Linux 0.11 from scratch（Rust 重写）——构建与运行
# 约定：编译/ISO 制作在 docker 容器里，qemu 在宿主机上跑
# 用法：make kernel（或 build）→ make run；容器自动按需创建/启动

DOCKER_IMG  := linux011-rust-toolchain
DOCKER_CTR  := linux011-rust-build
DOCKER_EXEC := docker exec $(DOCKER_CTR)
# 优先用 PATH 里的 qemu；本机包装脚本在 ~/.local/bin（已设 LD_LIBRARY_PATH / -L）
QEMU        := $(shell command -v qemu-system-x86_64 2>/dev/null || echo $(HOME)/.local/bin/qemu-system-x86_64)
# -boot d：默认启动顺序硬盘在前，会引导 reference/hdc-0.11.img 而不是我们的 CD
QEMU_FLAGS  := -m 128M -boot d -serial stdio -drive file=reference/hdc-0.11.img,format=raw,if=ide

.PHONY: toolchain ctr kernel build run debug check check-m1-b1 clean help

help: ## 显示本帮助
	@grep -E '^[a-z]+:.*##' Makefile | sed 's/:.*## /  →  /'

toolchain: ## 构建 docker 工具链镜像（仅镜像不存在时需先跑这个）
	docker build -t $(DOCKER_IMG) -f docker/Dockerfile .

# 复用常驻构建容器：镜像不存在 → build；容器不存在 → create；没在跑 → start。
# 注意：docker create 不加 -u $(id -u)。本机 docker 是 rootless 模式，容器内非 root
# uid 经 subuid 映射后在宿主机上不是本用户，写挂载目录会 Permission denied；
# 容器内 root 恰好映射回宿主机当前用户，产物归属自然正确。
ctr: ## 确保构建容器已启动（一般无需手动跑，kernel/build 会自动带上）
	@docker image inspect $(DOCKER_IMG) >/dev/null 2>&1 || { \
	    echo "镜像 $(DOCKER_IMG) 不存在，先构建…"; \
	    docker build -t $(DOCKER_IMG) -f docker/Dockerfile .; }
	@docker container inspect $(DOCKER_CTR) >/dev/null 2>&1 || { \
	    echo "创建构建容器 $(DOCKER_CTR)…"; \
	    docker create --name $(DOCKER_CTR) \
	        -e CARGO_HOME=/work/.cargo-home \
	        -v $(CURDIR):/work -w /work \
	        $(DOCKER_IMG) tail -f /dev/null >/dev/null; }
	@[ -n "$$(docker ps -q -f name=^$(DOCKER_CTR)$$)" ] || { \
	    echo "启动构建容器 $(DOCKER_CTR)…"; \
	    docker start $(DOCKER_CTR) >/dev/null; }

kernel: ctr ## 只编译内核 ELF（容器内；Bite2 前 make build 未就绪，先用这个）
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
	    -drive file=reference/hdc-0.11.img,format=raw,if=ide -cdrom build/kernel.iso; \
	rc=$$?; [ $$rc -eq 0 ] || [ $$rc -eq 124 ]
	grep 'M0: kernel alive' build/serial.log

# M1 Bite1：串口断言 + monitor screendump（证明真的画到了 framebuffer）
# 非 rootless docker 时 build/ 属容器 root，宿主机 qemu 写不进；先把归属改回当前用户。
check-m1-b1: build ## M1 Bite1 验收：FB 元数据 + 两行固定串 + screendump
	@$(DOCKER_EXEC) chown -R $$(id -u):$$(id -g) /work/build
	@mkdir -p build
	@rm -f build/serial.log build/m1-bite1.ppm
	@echo ">>> starting qemu (headless + monitor screendump)…"
	@( sleep 3; printf 'screendump build/m1-bite1.ppm\nquit\n' ) | \
	  $(QEMU) -m 128M -boot d -display none -vga std -net none \
	    -monitor stdio \
	    -serial file:build/serial.log \
	    -drive file=reference/hdc-0.11.img,format=raw,if=ide \
	    -cdrom build/kernel.iso; \
	  rc=$$?; [ $$rc -eq 0 ] || [ $$rc -eq 124 ]
	@echo ">>> serial.log:"
	@cat build/serial.log
	@echo ">>> asserting serial markers…"
	@grep -F 'M0: kernel alive' build/serial.log
	@grep -E 'M1: fb [0-9]+x[0-9]+x[0-9]+' build/serial.log
	@grep -F 'M1: hello framebuffer' build/serial.log
	@grep -F 'M1: console line 2' build/serial.log
	@test -f build/m1-bite1.ppm
	@sz=$$(wc -c < build/m1-bite1.ppm); echo "screendump size=$$sz"; \
	  test $$sz -gt 10240
	@echo "check-m1-b1 OK"

clean: ## 清理构建产物
	rm -rf build kernel/target user/target

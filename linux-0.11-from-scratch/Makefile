# Linux 0.11 from scratch（Rust 重写）——构建与运行
# 约定：编译/ISO 制作在 docker 容器里，qemu 在宿主机上跑
# 用法：make kernel（或 build）→ make run；容器自动按需创建/启动

DOCKER_IMG  := linux011-rust-toolchain
DOCKER_CTR  := linux011-rust-build
DOCKER_EXEC := docker exec $(DOCKER_CTR)
QEMU        := qemu-system-x86_64
QEMU_FLAGS  := -m 128M -serial stdio -drive file=reference/hdc-0.11.img,format=raw,if=ide

.PHONY: toolchain ctr kernel build run debug clean help

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

clean: ## 清理构建产物
	rm -rf build kernel/target user/target

# Linux 0.11 from scratch（Rust 重写）——构建与运行
# 约定：编译/ISO 制作在 docker 容器里，qemu 在宿主机上跑
# 用法：make toolchain（一次性）→ make build → make run

DOCKER_IMG  := linux011-rust-toolchain
DOCKER_RUN  := docker run --rm --network host \
               -e https_proxy=http://127.0.0.1:7890 -e http_proxy=http://127.0.0.1:7890 \
               -u $(shell id -u):$(shell id -g) \
               -e CARGO_HOME=/work/.cargo-home \
               -v $(CURDIR):/work $(DOCKER_IMG)
QEMU        := qemu-system-x86_64
QEMU_FLAGS  := -m 128M -serial stdio -drive file=reference/hdc-0.11.img,format=raw,if=ide

.PHONY: toolchain build run debug clean help

help: ## 显示本帮助
	@grep -E '^[a-z]+:.*##' Makefile | sed 's/:.*## /  →  /'

toolchain: ## 构建 docker 工具链镜像（一次性；走代理 --network host）
	docker build --network host \
	    --build-arg https_proxy=http://127.0.0.1:7890 \
	    --build-arg http_proxy=http://127.0.0.1:7890 \
	    -t $(DOCKER_IMG) -f docker/Dockerfile .

build: ## 编译内核 + 用户态 + 制作 ISO（容器内）
	$(DOCKER_RUN) make -f docker/build.mk

run: ## 宿主机启动 qemu（图形窗口 + 串口日志到终端）
	$(QEMU) $(QEMU_FLAGS) -cdrom build/kernel.iso

debug: ## 宿主 qemu 挂 gdb stub（:1234）+ 串口日志
	$(QEMU) $(QEMU_FLAGS) -cdrom build/kernel.iso -s -S

clean: ## 清理构建产物
	rm -rf build kernel/target user/target

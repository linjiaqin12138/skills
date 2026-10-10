# 由根 Makefile 的 build 目标在构建容器内调用：
#   docker exec linux011-rust-build make -f docker/build.mk
# 流水线：cargo 编译内核 → 组装 ISO 根目录 → xorriso 制作 BIOS+UEFI 双引导 ISO
#         → limine bios-install 修补 BIOS 引导指针。
#
# 决策：Limine 二进制取自镜像内 /opt/limine（Dockerfile 构建时从 reference/limine
# 复制并编译好了部署工具 limine），让本文件只依赖镜像、与工作区里 reference/ 的
# 状态解耦；两者同为 v10.8.5-binary tag，内容一致。

KERNEL_ELF := kernel/target/x86_64-unknown-none/debug/kernel
LIMINE_DIR := /opt/limine
ISO_ROOT   := build/iso_root
ISO        := build/kernel.iso

.PHONY: all kernel
all: $(ISO)

# cargo 自己判断是否需要重编，作为 .PHONY 每次调用也只是毫秒级空转
kernel:
	cargo build --manifest-path kernel/Cargo.toml

$(ISO): kernel limine.conf
	rm -rf $(ISO_ROOT)
	mkdir -p $(ISO_ROOT)/boot/limine $(ISO_ROOT)/EFI/BOOT
	cp $(KERNEL_ELF) $(ISO_ROOT)/kernel.elf
	cp limine.conf $(ISO_ROOT)/boot/limine/
	cp $(LIMINE_DIR)/limine-bios.sys $(LIMINE_DIR)/limine-bios-cd.bin \
	   $(LIMINE_DIR)/limine-uefi-cd.bin $(ISO_ROOT)/boot/limine/
	cp $(LIMINE_DIR)/BOOTX64.EFI $(ISO_ROOT)/EFI/BOOT/
	# El Torito：-b/--efi-boot 的路径都是 ISO 内部路径。
	# -boot-load-size 4：El Torito 规范以 512 字节为单位，4 = 2048 字节即一个 CD 扇区，
	# BIOS 只预载这一段，剩余部分由 limine-bios-cd.bin 自己加载（Limine 官方示例值）。
	xorriso -as mkisofs -R -r -J \
	    -b boot/limine/limine-bios-cd.bin \
	    -no-emul-boot -boot-load-size 4 -boot-info-table \
	    --efi-boot boot/limine/limine-uefi-cd.bin \
	    -efi-boot-part --efi-boot-image --protective-msdos-label \
	    $(ISO_ROOT) -o $(ISO)
	# isohybrid 修补：GPT→MBR、置活动分区、把硬盘引导代码埋进第 0 扇区，
	# 让同一 ISO 也能 dd 到 U 盘当磁盘引导。纯光盘 El Torito 路径实测不依赖它
	# （qemu -boot d 跳过这步照样启动），但上游 USAGE.md 把这一步列为 hybrid
	# ISO 流程必需（"do not forget"），保留换来 isohybrid 能力。
	limine bios-install $(ISO)

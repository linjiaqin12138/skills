# Linux 0.11 Rust 重写项目——构建工具链容器
# 用途：内核与用户态的全部编译 + ISO 制作都在此容器内；qemu 在宿主机跑（显示/键盘不好容器化）
FROM rustlang/rust:nightly-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        xorriso make gcc git ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# no_std 内核需要 rust-src（build-std）与裸机 target
RUN rustup component add rust-src \
    && rustup target add x86_64-unknown-none

# Limine 引导器：用工作区已克隆的 binary 分支，只编译它的部署工具 limine
COPY reference/limine /opt/limine
RUN make -C /opt/limine && ln -s /opt/limine/limine /usr/local/bin/limine

WORKDIR /work

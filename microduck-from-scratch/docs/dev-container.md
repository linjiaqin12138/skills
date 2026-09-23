# 开发容器

宿主机不装 Rust。编译在 Docker 容器 `miniduck-rust` 里做。镜像基于 `rust:1-bookworm`（当前 rustc 1.98.1，满足 `edition = "2024"`）。Docker Hub 直连超时，基础镜像从 `docker.m.daocloud.io/library/rust:1-bookworm` 拉。

配置在仓库根的 `docker-compose.yml` 和 `docker/Dockerfile`。

## 挂载

| 容器路径 | 来源 | 作用 |
|---|---|---|
| `/work` | 仓库根目录（bind mount） | 源码和 `target/`。工作目录就是这里 |
| `/cargo` | volume `cargo-home` | crates.io 索引和依赖缓存，删容器不会清 |

容器用户是 uid/gid `1000`（对应当前宿主机用户 `jqlin`），所以 `target/` 里的文件属主是你，不是 root。换机器后如果 `id -u` 不是 1000，改 `docker-compose.yml` 里的 `UID`/`GID` 再 `--build`。

## 常用命令

在仓库根目录执行。

```bash
docker compose up -d --build   # 构建镜像并启动（已在跑时再执行一次也安全）
docker compose exec rust cargo build
docker compose exec rust cargo test
docker compose exec rust bash  # 进容器，里面直接 cargo build / cargo test
```

产物在宿主机的 `target/debug/`：`miniduckd`、`mini-duckctl`。验收时在宿主机跑这两个二进制即可，不必留在容器里。

停容器（volume 还在，下次不用重新下依赖）：

```bash
docker compose stop
docker compose up -d
```

`docker compose down` 只删容器和网络，不删 `cargo-home`。要连依赖缓存一起清掉，用 `docker compose down -v`。

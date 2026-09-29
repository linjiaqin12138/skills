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

## 交互式仿真（noVNC 远程桌面）

`--view-port` 的 MJPEG 网页只能看不能碰。想在浏览器里用鼠标直接把鸭子推倒，用 noVNC 栈：容器里跑 Xvfb 虚拟显示器 + MuJoCo 原生 viewer，x11vnc/websockify 把桌面桥进浏览器。

```bash
docker compose exec rust bash scripts/duck-vnc.sh --port 7801
# 宿主机浏览器打开 http://localhost:6080/vnc.html
```

脚本依次起 Xvfb（:99，1280x800）→ x11vnc（5900）→ websockify+noVNC 网页（6080）→ `sim/duck_body.py --viewer`；某一环挂了会打印对应日志（`/tmp/duck-vnc/*.log`）。退出（Ctrl+C 或关掉 viewer 窗口）会清掉整串进程。

**操作**（MuJoCo 3.14 viewer 实测，viewer 里按 F1 也有帮助）：

| 操作 | 效果 |
|---|---|
| 左键拖拽 / 右键拖拽 / 滚轮 | 旋转 / 平移 / 缩放相机 |
| 双击左键躯干 | 选中刚体（出现选中框） |
| **Ctrl + 右键拖拽** | 施加平移力——推鸭子就这个（加 Shift 换拖拽平面） |
| **Ctrl + 左键拖拽** | 施加力矩（拧） |
| 双击右键 / Ctrl+双击右键 | 设注视点 / 相机跟踪选中刚体（鸭子走远后用它跟拍） |

注意：物理节拍仍由 daemon 驱动（`miniduckd --sim 127.0.0.1:7801` 每 20ms 一个 `read`）。daemon 不连时画面能转、能选刚体，但物理冻结，推了也不动——先 `mini-duckctl enable` 让鸭子站起来再推。

渲染走 Mesa llvmpipe 软件 GL（`LIBGL_ALWAYS_SOFTWARE=1`），帧率一般但够用。MJPEG 模式（`--view-port 7802`，osmesa）不受影响，两条路可以同时开。

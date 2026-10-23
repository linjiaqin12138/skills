# 开发容器

宿主机不装 Rust。编译在 Docker 容器 `miniduck-rust` 里做。镜像基于 `rust:1-bookworm`（当前 rustc 1.98.1，满足 `edition = "2024"`）。Docker Hub 直连超时，基础镜像从 `docker.m.daocloud.io/library/rust:1-bookworm` 拉。

配置在 `impl/docker-compose.yml` 和 `impl/docker/Dockerfile`。

## 挂载

| 容器路径 | 来源 | 作用 |
|---|---|---|
| `/work` | 仓库根目录（bind mount） | `accept/`、`book/`、`impl/` |
| `/work/impl` | 工作目录 | 源码、`sim/`、`target/`（宿主机上是 `impl/target/`） |
| `/cargo` | volume `cargo-home` | crates.io 索引和依赖缓存，删容器不会清 |

容器用户是 uid/gid `1000`（对应当前宿主机用户 `jqlin`），所以 `impl/target/` 里的文件属主是你，不是 root。换机器后如果 `id -u` 不是 1000，改 `impl/docker-compose.yml` 里的 `UID`/`GID` 再 `--build`。

## 常用命令

在仓库根目录执行。

```bash
docker compose -f impl/docker-compose.yml up -d --build   # 构建镜像并启动（已在跑时再执行一次也安全）
docker compose -f impl/docker-compose.yml exec rust cargo build
docker compose -f impl/docker-compose.yml exec rust cargo test
docker compose -f impl/docker-compose.yml exec rust bash  # 进容器，里面直接 cargo build / cargo test
```

产物在宿主机的 `impl/target/debug/`：`miniduckd`、`mini-duckctl`。验收时在宿主机跑这两个二进制即可，不必留在容器里。

停容器（volume 还在，下次不用重新下依赖）：

```bash
docker compose -f impl/docker-compose.yml stop
docker compose -f impl/docker-compose.yml up -d
```

`docker compose -f impl/docker-compose.yml down` 只删容器和网络，不删 `cargo-home`。要连依赖缓存一起清掉，用 `docker compose -f impl/docker-compose.yml down -v`。

## 仿真跑哪：容器 vs 宿主机

规则：**跟仿真界面无关的一律容器内跑，只有需要人眼看着的（viewer 交互）才上宿主机。**

| 事情 | 在哪跑 | 命令 |
|---|---|---|
| 数据提取（`m9a_extract_official.py`） | 容器 | `docker compose -f impl/docker-compose.yml exec -T rust /opt/sim-venv/bin/python scripts/m9a_extract_official.py` |
| 验收脚本（`accept-*.sh`） | 容器 | `docker compose -f impl/docker-compose.yml exec rust bash /work/accept/m09a-b01.sh` |
| 编译模型、数值断言 | 容器 | 同上，`/opt/sim-venv/bin/python`（mujoco 3.16） |
| viewer 看模型 / 交互推鸭子 | 宿主机 | `.venv/bin/python -m mujoco.viewer --mjcf=impl/sim/assets/scene_handmade.xml` |

`accept-*.sh` 自带解释器探测：在容器内自动用 `/opt/sim-venv/bin/python`；宿主机上没有容器时才回落到仓内 `.venv/bin/python`；两处都没有就报错退出，不要静默用错环境。

## 宿主机 viewer 环境（唯一的宿主机 Python 环境）

位置：仓库根 `.venv/`（已 gitignore）。**只为 viewer 交互而存在**，验收与数据提取不要用它。

建法（2026-10-20 实测，uv 装 Python 3.12.15）：

```bash
cd ~/Project/skills/microduck-from-scratch
uv venv --python 3.12 .venv
VIRTUAL_ENV=$PWD/.venv uv pip install "mujoco==3.16.0" onnx onnxruntime pillow \
  "better-actuator-models @ git+https://github.com/Rhoban/bam@62bd8ce"
```

两个约束：`better-actuator-models`（BAM）不支持 Python ≥3.13，系统 3.14 用不了，所以钉 3.12；`.venv` 在仓库里但**不进镜像**（镜像里的 `/opt/sim-venv` 是另一份）。

## 交互式仿真（noVNC 远程桌面）

`--view-port` 的 MJPEG 网页只能看不能碰。想在浏览器里用鼠标直接把鸭子推倒，用 noVNC 栈：容器里跑 Xvfb 虚拟显示器 + MuJoCo 原生 viewer，x11vnc/websockify 把桌面桥进浏览器。

```bash
docker compose -f impl/docker-compose.yml exec rust bash scripts/duck-vnc.sh --port 7801
# 宿主机浏览器打开 http://localhost:6080/vnc.html
```

脚本依次起 Xvfb（:99，1280x800）→ x11vnc（5900）→ websockify+noVNC 网页（6080）→ `impl/sim/duck_body.py --viewer`；某一环挂了会打印对应日志（`/tmp/duck-vnc/*.log`）。退出（Ctrl+C 或关掉 viewer 窗口）会清掉整串进程。

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

## Rootful vs Rootless Docker

本仓库默认用 rootful Docker（daemon 以 root 跑），容器内 uid/gid 1000 是为了让 `impl/target/` 产物属主是宿主机普通用户，而不是走 rootless 模式。两种模式的区别：

- **Rootful（默认）**：`dockerd` 以 root 运行。风险在于能访问 `/var/run/docker.sock` 的用户等同于宿主机 root，容器逃逸即拿到宿主机 root。可绑定 <1024 端口、支持 `--network host`。
- **Rootless**：`dockerd` 以普通用户运行，靠 user namespace 把容器内 root 映射为宿主机普通用户，逃逸也只是一般用户权限。每个用户有独立 daemon、数据目录（`~/.local/share/docker`）和 socket（`$XDG_RUNTIME_DIR/docker.sock`）。

Rootless 的限制：默认不能绑 <1024 端口（可 `sudo sysctl net.ipv4.ip_unprivileged_port_start=0` 放开）；网络走 slirp4netns，性能略低，容器看到的源 IP 是 `10.0.2.100`；不支持 `--network host`；cgroup 资源限制需要 systemd + cgroup v2 并为用户开 delegate。

如需在本项目用 rootless 模式（Ubuntu/Debian）：

```bash
sudo apt install uidmap dbus-user-session fuse-overlayfs slirp4netns
dockerd-rootless-setuptool.sh install
sudo loginctl enable-linger $USER            # 可选：未登录也跑 daemon
systemctl --user enable --now docker
export DOCKER_HOST=unix://$XDG_RUNTIME_DIR/docker.sock   # 写入 ~/.bashrc
```

之后 `docker compose` 命令照常使用。注意 rootless 下容器 uid 1000 实际映射到宿主机你的用户，`impl/target/` 属主关系不变；但 `/etc/subuid` 映射区间内属主会显示为你的用户，属正常现象。

怎么选：本机开发、多人共用机器优先 rootless（更安全）；需要 host 网络、GPU 直通、绑 80/443 时用 rootful。参考：<https://docs.docker.com/engine/security/rootless/>

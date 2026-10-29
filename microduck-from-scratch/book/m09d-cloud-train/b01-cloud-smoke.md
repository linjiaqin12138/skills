---
template: sheet
title: M9d Bite 1：云 4090 冒烟（第 27 章）
subtitle: 换机可复装；训的是官方配方，不是自建身体
source: clone-from-scratch-tutor · 第 27 章
---

## 问题

本机没有 CUDA，M9b 只能练判断。上云之后故障换成环境层：镜像 torch 版本错、盘装爆、没 wandb、跳过冒烟。

更关键的一点必须先钉死：

**本课（以及当前云上首训）训的是官方 `microduck_rl` 里的 Velocity 配方与官方机器人资产，不是 `impl/sim/assets/scene_handmade.xml` 那套自建身体。**

| 现在在训什么 | 还没在训什么 |
|---|---|
| 任务 `Mjlab-Velocity-Flat-MicroDuck` | 自建 MJCF 挂进训练 env |
| 官方仓里的 walk / BAM 等配置 | 只在 `impl/` 里从零写 PPO |
| 产出：官方分布下的 checkpoint → 以后 export ONNX | M9e：ONNX 进我们的 miniduckd + handmade sim |

工程上属于我们的部分：租卡、跑通管线、权重最终进我们的 daemon。身体动力学要换成 handmade，是后续把场景挂进同一条训练工厂，不是本课已经在做的事。

本课消掉的故障是：换一台云机时装不出同一套栈，或误以为已经在训自建鸭子。

## 背景

**uv。** 用它按 `uv.lock` 建隔离环境；不要用镜像 conda / 系统 `pip install torch`。

**wandb。** 官方训练默认记曲线。没 key 会在 `learn()` 门口失败。对 **uv 环境**执行 `uv run wandb login`。

**microduck_rl。** pollen-robotics 的训练仓（mjlab + Warp + PPO）。`impl/` 是运行时；本课只借用训练工厂。

## 设计

```flow TB
(空白云机) -> 装系统工具
装系统工具 -> clone microduck_rl
clone microduck_rl -> *uv sync
uv sync -> wandb login
wandb login -> 冒烟 64×5
冒烟 64×5 -> 首训
```

算力取舍见 [0008](../../decisions/0008-m9d-cloud-4090.md)：4090 可以；**镜像自带 torch 作废**。

| 项 | 钉死 |
|---|---|
| GPU | RTX 4090（OOM 再降 `num-envs`） |
| Python / torch | 3.12 / `2.9.1`（uv wheel，自带 CUDA runtime） |
| 工作根目录 | 数据盘，本实例为 `/cloud/cloud-ssd1/microduck-m9d` |
| 任务 | `Mjlab-Velocity-Flat-MicroDuck`（官方） |
| 冒烟 | `64` envs × `5` iter |

仓内脚本：`impl/scripts/m9d_cloud_smoke.sh`、`impl/scripts/m9d_first_train.sh`（拷到云机数据盘执行亦可）。

## 架构决定

选了 **备选 1**：换卡不换栈——4090 + `uv sync` 锁版。

没选等 5090：同卡更省心，但库存与价格不稳。  
没选迁就镜像 torch 2.1.3：少装一次，但和官方配方分叉。

见 [0008](../../decisions/0008-m9d-cloud-4090.md)。

## 验收

自动门禁（教程仓根，本机）：

```bash
bash accept/m09d-b01.sh
```

核对决定、冒烟脚本、以及 `tutor/runs/m09d-b01.txt` 中的 4090 / `torch 2.9` / `SMOKE PASS`。

### 换机重装清单（空白 Ubuntu + NVIDIA 驱动）

下面每条单独复制。路径以「数据盘挂在 `/cloud/cloud-ssd1`」为例；别的平台把 `WORK` 改成你的大盘路径。

**0. 登录**

```bash
ssh -p 23 root@<公网IP>
```

**1. 系统工具**（镜像常缺 `git`）

```bash
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq git curl ca-certificates build-essential
git --version
```

装了什么：`git`（克隆）、`curl`（装 uv）、证书、编译链（部分轮子/本地包需要）。

**2. 工作目录放在数据盘**

```bash
export WORK=/cloud/cloud-ssd1/microduck-m9d
mkdir -p "$WORK"
df -h "$WORK"
nvidia-smi -L
```

预期：大盘有空闲；`nvidia-smi` 看到 GPU 型号。

**3. 安装 uv（用户级，写入 `~/.local/bin`）**

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
uv --version
```

装了什么：`uv` 包管理器本身（还没装 PyTorch）。

**4. 克隆官方训练仓**

```bash
cd "$WORK"
git clone --depth 1 https://github.com/pollen-robotics/microduck_rl.git
cd microduck_rl
```

得到什么：官方任务配置、MJCF 资产、`pyproject.toml` / `uv.lock`。

**5. 同步 Python 依赖（锁版）**

```bash
export PATH="$HOME/.local/bin:$PATH"
cd "$WORK/microduck_rl"
uv sync
```

装了什么（由锁文件决定，勿手改版本）：Python 3.12 虚拟环境、`torch==2.9.1`（CUDA runtime 在 wheel 里）、`mjlab`、`warp-lang`、`rsl-rl-lib`、`wandb`、BAM（git 依赖 Rhoban/bam）等。体积约数 GB 到 ~8GB。

**6. 核对栈（必须是 uv，不是镜像 conda）**

```bash
export PATH="$HOME/.local/bin:$PATH"
cd "$WORK/microduck_rl"
uv run python -c "import torch,sys; print(sys.version.split()[0], torch.__version__, torch.cuda.get_device_name(0))"
```

预期：`3.12.x`、`2.9.1+cu…`、GPU 名。若仍是 `2.1.3`，说明进错解释器。

**7. wandb 登录（同一 uv 环境）**

```bash
export PATH="$HOME/.local/bin:$PATH"
cd "$WORK/microduck_rl"
uv run wandb login
```

浏览器打开 [wandb.ai/authorize](https://wandb.ai/authorize) 粘贴 key。密钥落在云机 `~/.netrc`。

**8. 冒烟**

```bash
export PATH="$HOME/.local/bin:$PATH"
cd "$WORK/microduck_rl"
uv run train Mjlab-Velocity-Flat-MicroDuck --env.scene.num-envs 64 --agent.max_iterations 5
```

等待约 1–3 分钟。预期：Warp 编译后出现 iteration 日志，退出码 0；无 `No API key configured`。

**9. 首训（冒烟通过后）**

```bash
export PATH="$HOME/.local/bin:$PATH"
cd "$WORK/microduck_rl"
uv run train Mjlab-Velocity-Flat-MicroDuck --env.scene.num-envs 2048 --agent.max_iterations 1000
```

或用仓内脚本（需先拷到云机）：`impl/scripts/m9d_first_train.sh`。观察终端 / wandb 的 `Iteration time` 与 `ETA`。本实例 `2048` envs 显存约 3GB。

### 本次首训结果（2026-10-29，官方 Velocity）

配置：`2048` envs × `1000` iter，RTX 4090；墙钟约 **25 分钟**（`_runtime` ≈ 1578s）。

**在浏览器看曲线与超参：**

- 项目：[https://wandb.ai/linjiaqin8-na/mjlab_microduck](https://wandb.ai/linjiaqin8-na/mjlab_microduck)
- 本 run：[https://wandb.ai/linjiaqin8-na/mjlab_microduck/runs/weqfvj7f](https://wandb.ai/linjiaqin8-na/mjlab_microduck/runs/weqfvj7f)（名 `2026-10-29_08-00-43_velocity`，id `weqfvj7f`）

末步（step 999）摘要：

| 指标 | 值 |
|---|---|
| `Train/mean_reward` | ≈ 109.15 |
| `Train/mean_episode_length` | ≈ 928.8 |
| `Episode_Reward/track_linear_velocity` | ≈ 1.22 |
| `Episode_Reward/upright` | ≈ 1.64 |
| `Episode_Termination/fell_over` | ≈ 0.33 |
| `Metrics/twist/error_vel_xy` | ≈ 0.45 |

**云机上的产物目录**（数据盘，换机前先拷走或挂 volume）：

```text
/cloud/cloud-ssd1/microduck-m9d/microduck_rl/logs/rsl_rl/velocity/2026-10-29_08-00-43_velocity/
  model_0.pt … model_250.pt … model_500.pt … model_750.pt … model_999.pt
  2026-10-29_08-00-43_velocity.onnx
```

终态权重：`model_999.pt`。训练结束时目录里已有一份 `.onnx`；**部署进我们的 daemon 前仍应走官方 `scripts/export.py`（烘 obs normalizer）**，不要默认把这份自动 onnx 当最终发布物。

续训同一任务示例：

```bash
export PATH="$HOME/.local/bin:$PATH"
cd "$WORK/microduck_rl"
uv run train Mjlab-Velocity-Flat-MicroDuck \
  --env.scene.num-envs 2048 \
  --agent.max_iterations 4000 \
  --agent.load-checkpoint logs/rsl_rl/velocity/2026-10-29_08-00-43_velocity/model_999.pt \
  --agent.resume True
```

### 换机时要带走的

| 要保留 | 可扔掉重装 |
|---|---|
| wandb 账号与 API key | 云机上的 `.venv`（再 `uv sync`） |
| 训练日志与 checkpoint（数据盘或下云） | 系统盘上的临时文件 |
| 本课这份安装顺序 | 镜像自带的旧 torch |

## 坑

- 缺 `git`：先走验收第 1 步再 clone。
- `wandb status` 里 `api_key: null` 仍可能已写入 `~/.netrc`；以 `uv run train` 是否报 No API key 为准。
- overlay 根分区只有约 50GB：仓库与 `.venv` 必须在数据盘。
- SSH `authorized_keys` 报 E212：先 `mkdir -p ~/.ssh && chmod 700 ~/.ssh`。
- 不要把「在训官方 Velocity」说成「已经在训 handmade」——身体还没换进训练 env。

## 自测

**1. 当前云上 `Mjlab-Velocity-Flat-MicroDuck` 在训谁的身体？**

- A. `impl/sim/assets/scene_handmade.xml`
- B. 官方 `microduck_rl` 任务自带的机器人资产
- C. 只有 Rust FakeIo，没有物理身体
- D. 任意 MJCF，训练时随机抽

<details><summary>答案</summary>

B。本课 / 当前首训是官方配方。handmade 进训练是后续步骤；M9e 先把官方分布训出的 ONNX 接进我们的运行时。

</details>

**2. 换一台新云机，最小正确顺序是？**

- A. 直接 `python train.py`，用镜像 conda
- B. 装 git/curl → 数据盘 clone → `uv sync` → `wandb login` → 冒烟 64×5
- C. 先长跑 4096×5000，再补依赖
- D. 只拷 `.venv`，不 clone 源码

<details><summary>答案</summary>

B。与验收清单一致。

</details>

**3. 镜像预装 torch 2.1.3，应怎样？**

- A. 升级系统 torch 即可
- B. 忽略它，在 microduck_rl 里 `uv sync` 用锁版 `2.9.1`
- C. 把官方 pyproject 改回 2.1.3
- D. 卸载驱动重装

<details><summary>答案</summary>

B。

</details>

**4. 冒烟参数？**

- A. 4096×5000
- B. 1×1
- C. 64×5
- D. 不传参数

<details><summary>答案</summary>

C。

</details>

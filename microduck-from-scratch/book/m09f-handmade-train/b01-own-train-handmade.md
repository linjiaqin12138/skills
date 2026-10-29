---
template: sheet
title: M9f Bite 1：自写训练入口 + handmade（第 30 章）
subtitle: 训推同身体；入口是我们的脚本，不是 uv run train
source: clone-from-scratch-tutor · 第 30 章
---

## 问题

M9d 在云上把官方 `uv run train` + 官方 `robot_walk` 跑通了；M9e 把那份官方分布 ONNX 接到了我们的 daemon。那条路证的是**集成栈**，不是**训推同身体**——权重仍在官方身体上长大。

现在要消掉的故障是：还在用官方 train CLI 训官方身体，或以为包一层 `uv run train` 就算「自己的训练入口」。

本课钉死两件事：身体是 `duck_handmade.xml`；入口是我们写的 `m9f_train.py`，在官方 uv 环境里跑这份 `.py`。

## 背景

**handmade 身体。** M9a 手写的 `duck_handmade.xml`：碰撞盒 + 官方惯量对账，与 `impl/sim` 部署同源。详见 [M9a Bite 1](../m09a-handmade-mjcf/b01-handmade-mjcf.html)。

**Velocity 奖励与课程。** 本课仍走官方 `make_microduck_velocity_env_cfg` 工厂：奖励权重、课程阶梯、BAM 执行器都不重写。配方直觉见 [M9b Bite 2](../m09b-train-pipeline/b02-reward-curriculum.html)。PPO 三步、优势估计细节本课当黑盒，不重讲。

## 设计

```flow TB
(m9f_train.py) -> make_microduck_velocity_env_cfg
make_microduck_velocity_env_cfg -> *换 entities robot
换 entities robot -> ManagerBasedRlEnv
ManagerBasedRlEnv -> MicroduckOnPolicyRunner
MicroduckOnPolicyRunner -> *runner.learn
```

| 层 | 谁提供 | 本课改什么 |
|---|---|---|
| 奖励 / 观测 / BAM / 课程 | 官方 Velocity 工厂 | 不改 |
| 身体 XML | `HANDMADE_ROBOT_CFG` → `duck_handmade.xml` | **换身** |
| 训练入口 | `impl/scripts/m9f_train.py` | **自写**；禁止主路径 `uv run train` / `train_cli` |
| PPO 更新 | rsl_rl（经 `MicroduckOnPolicyRunner.learn`） | 黑盒：优化策略；不展开优势估计 |

冒烟规模与 M9d 对齐：`64` envs × `5` iter。长训与视频回放是同一入口的加参用法，不挡本课 accept 门禁。

## 架构决定

选了 **备选 1**：自写 `m9f_train.py`——工厂出 `env_cfg` 后把 `scene.entities["robot"]` 换成 handmade 的 `EntityCfg`，再起 runner。

没选继续 `uv run train` 再 monkeypatch vendor：改动小，但学不到组装，vendor 补丁难维护。  
没选从零手写 PPO + 自建 env：一个 Bite 不可验收，也偏离「复用可下载栈、学设计」。

见 [0011](../../decisions/0011-m9f-handmade-own-train.md)。

## 代码领读

自顶向下：身体配置 → 入口换身 → `learn`。sketch 对齐真码行号。

### 1. handmade 身体：`EntityCfg`

`m9f_handmade_robot.py` 镜像官方 walk 的 HOME / BAM / 全碰撞，只把 `spec_fn` 指到 `impl/sim/assets/duck_handmade.xml`（`20-37`）：

```python title="m9f_handmade_robot.py · sketch"
HANDMADE_XML = Path(__file__).resolve().parents[1] / "sim" / "assets" / "duck_handmade.xml"

def get_handmade_spec() -> mujoco.MjSpec:
    return mujoco.MjSpec.from_file(str(HANDMADE_XML))

HANDMADE_ROBOT_CFG = EntityCfg(
    spec_fn=get_handmade_spec,
    init_state=HOME_FRAME,
    collisions=(FULL_COLLISION,),
    articulation=EntityArticulationInfoCfg(
        actuators=(actuators,),
        soft_joint_pos_limit_factor=0.9,
    ),
)
```

**EntityCfg 换身约定：** 奖励 / obs / BAM 仍官方工厂；只有身体 XML 换成 handmade，与 `impl/sim` 同源。

### 2. 入口：工厂后换 `entities`，再 `learn`

`m9f_train.py` 不调官方 CLI。核心三步（`112-114`、`147-160`）：

```python title="m9f_train.py · sketch"
env_cfg = make_microduck_velocity_env_cfg()
env_cfg.scene.entities = {"robot": HANDMADE_ROBOT_CFG}  # 换身
env_cfg.scene.num_envs = args.num_envs
# ... replace MicroduckRlCfg：max_iterations / save_interval / experiment_name ...

env = ManagerBasedRlEnv(cfg=env_cfg, device=device)
env = RslRlVecEnvWrapper(env, clip_actions=agent.clip_actions)
runner = MicroduckOnPolicyRunner(env, agent_cfg, str(log_dir), device)
runner.learn(
    num_learning_iterations=agent.max_iterations,
    init_at_random_ep_len=True,
)
```

**自写入口约定：** 用官方 `microduck_rl` 的 `uv run` 跑**我们的** `.py`，命令形态是 `uv run python …/m9f_train.py …`，不是 `uv run train …`。`runner.learn` 内部的 PPO 更新当黑盒；优势怎么估已在 M9b 先修，这里不重开。

冒烟结束打印 `smoke_ok=1`（`max_iterations≤5` 或 `--smoke`，`163-165`）。banner 会打出 `handmade` 与 `duck_handmade.xml` 路径（`69-78`）。

## 验收

本机无 GPU：`bash accept/m09f-b01.sh` 只做静态门（脚本存在、禁止官方 CLI 主路径、跑证含冒烟线索）。**亲手验收请在云上跑冒烟。**

云机：`ssh -p 23 root@117.50.221.204`（本机 key `~/.ssh/id_rsa`）。  
工作区：`/cloud/cloud-ssd1/microduck-m9f`（教程仓在其下 `skills/microduck-from-scratch`）。  
RL venv：复用 M9d 的 `/cloud/cloud-ssd1/microduck-m9d/microduck_rl`（在该目录 `uv run`）。

### 云冒烟（本课门禁对应的亲手步骤）

**1. 登录**

```bash
ssh -p 23 root@117.50.221.204
```

**2. 冒烟 64×5**（一步一条；在已 `uv sync` 的 microduck_rl 树里）

```bash
export PATH="$HOME/.local/bin:$PATH"
```

```bash
cd /cloud/cloud-ssd1/microduck-m9d/microduck_rl
```

```bash
uv run python /cloud/cloud-ssd1/microduck-m9f/skills/microduck-from-scratch/impl/scripts/m9f_train.py --num-envs 64 --max-iterations 5
```

**3. 观察**

等待约 1–3 分钟。预期：

- banner 出现 `handmade` / `duck_handmade.xml`
- 出现 `Learning iteration …/5`
- 结束打印 `smoke_ok=1`

本机回归（静态门 + 已落盘跑证）：

```bash
bash accept/m09f-b01.sh
```

预期：`cli_gate_ok` 与 `M9f Bite 1 验收 PASS`。

### 附录：长训与 HD replay（已跑事实，非 accept 门禁）

同一入口加大参数即可；下列为 2026-10-29 已完成记录，**不要把「长训完成」当成 Bite 1 门禁**。

| 项 | 事实 |
|---|---|
| 长训 | `2048` envs × `4000` iter，`save-interval=100`；墙钟约 **1h14m**；结束 `train_finished=1` |
| 云日志目录 | `/cloud/cloud-ssd1/microduck-m9f/logs/train_2048x4000_si100_20261029_102513/` |
| 终局权重 | `model_3999.pt` |
| 本机副本 | `tutor/artifacts/m09f/checkpoints/{model_0,1000,2000,3000,3500,3900,3999}.pt` 与 `.onnx` |

HD 回放（可选重跑；云机无 DISPLAY，须 EGL）：

```bash
export PATH="$HOME/.local/bin:$PATH" MUJOCO_GL=egl
```

```bash
cd /cloud/cloud-ssd1/microduck-m9d/microduck_rl
```

```bash
uv run python /cloud/cloud-ssd1/microduck-m9f/skills/microduck-from-scratch/impl/scripts/m9f_replay.py \
  --log-dir /cloud/cloud-ssd1/microduck-m9f/logs/train_2048x4000_si100_20261029_102513 \
  --out-dir /cloud/cloud-ssd1/microduck-m9f/logs/train_2048x4000_si100_20261029_102513/videos/replay_hd \
  --video-width 1280 --video-height 720
```

预期：全量约 41 个 checkpoint 各出一支 mp4，结束 `replay_ok=1`。`m9f_replay.py` 同样换 `HANDMADE_ROBOT_CFG`，不用官方 `play` CLI。

## 坑

- **云机录视频缺 DISPLAY：** 须 `MUJOCO_GL=egl`。该机上 glfw / xvfb 会失败。
- **脚 site / IMU：** Velocity 要读 `left_foot` / `right_foot` 与 `imu_*` 传感器。本轮 `duck_handmade.xml` 已补齐同名 site 与 sensor（见 XML 内 `imu` / `left_foot` / `right_foot` 与 `<sensor>` 段）；缺这些时工厂会报警或训不动。
- **别把 M9e 当本课捷径：** M9e 是 daemon 接 M9d 官方分布权重；本课要的是 handmade 上自训。官方 CLI 训官方身体再「接入我们」——入口与身体一起换成我们的。

## 自测

**1. 本课训练入口应是？**

- A. `uv run train Mjlab-Velocity-Flat-MicroDuck …`
- B. `uv run python …/m9f_train.py --num-envs 64 --max-iterations 5`
- C. 只调用 `mjlab_microduck.train_cli`
- D. 本机 `pip install` 后直接 `python -m rsl_rl`

<details><summary>答案</summary>

B。官方 uv 环境跑我们的脚本；主路径禁止 `uv run train` / `train_cli`。

</details>

**2. `env_cfg.scene.entities = {"robot": HANDMADE_ROBOT_CFG}` 换的是什么？**

- A. 奖励权重表与课程阶梯
- B. 只有身体 XML（handmade），奖励 / obs / BAM 仍官方工厂
- C. 整份 PPO 实现换成自研
- D. 把观测维数从 61 改成自定义

<details><summary>答案</summary>

B。换身约定：配方工厂不动，只换 `EntityCfg` 指向的 XML。

</details>

**3. Bite 1 accept 门禁要证明什么？**

- A. 必须跑完 2048×4000 并产出全部 mp4
- B. 自写入口 + handmade 引用 + 云冒烟 64×5 证据（`smoke_ok=1`）
- C. 官方权重已接入 miniduckd
- D. 本机无 GPU 也必须现场 `learn()` 5 步

<details><summary>答案</summary>

B。长训 / replay 是附录事实；本机 accept 是静态门 + 跑证。

</details>

**4. 云上用 `m9f_replay.py` 出视频，缺 DISPLAY 时应？**

- A. 强开 glfw viewer
- B. 只设 `MUJOCO_GL=egl`
- C. 改回官方 `play` CLI（绑定 `robot_walk`）
- D. 在本机无 GPU 上硬跑 Warp 训练代替回放

<details><summary>答案</summary>

B。该云机 glfw/xvfb 失败；replay 脚本本身已挂 handmade。

</details>

**5. 为什么本课不以「官方 CLI 训官方身体 → 再接入我们」为主路径？**

- A. 官方仓库删了 train 命令
- B. 那条路径对「训推同身体、自己组装入口」没有学习意义（M9e 另证集成栈）
- C. handmade XML 不能编译
- D. 4090 跑不动 64×5

<details><summary>答案</summary>

B。M9e 已证 daemon 集成；本课入口与身体一起换成我们的。

</details>

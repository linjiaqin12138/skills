---
template: sheet
title: M9f Bite 1：手搓 handmade 训练环境（第 30 章）
subtitle: 从自己的 MJCF 写任务；PPO 从随机权重学会直立前进，并导出可部署 ONNX
source: clone-from-scratch-tutor · 第 30 章
---

## 问题

已有的 M9d 权重能走，只证明官方身体和官方任务配方有效。把 `duck_handmade.xml` 塞进官方 Velocity class，仍然是在猜官方假设：每脚只有一块鞋底、奖励和课程适合自己的身体。

训练脚本能跑完，策略却可能站稳不动或低姿态快走。只换官方 class，也学不到训练环境怎么写。本课去掉这些故障。新的主路径不使用 `mjlab_microduck`；MuJoCo、mjlab 通用管理器、rsl_rl 和 BAM 只作为依赖轮子。

## 设计

```flow TB
duck_handmade.xml -> m9f_robot
m9f_robot -> m9f_env_cfg
m9f_env_cfg -> m9f_mdp
m9f_env_cfg -> m9f_train
m9f_mdp -> m9f_train
m9f_train -> checkpoint
checkpoint -> m9f_evaluate
checkpoint -> m9f_replay
checkpoint -> m9f_export
m9f_export -> policy_export.onnx
```

七个模块各有边界：

| 模块 | 自己负责什么 |
|---|---|
| `m9f_robot.py` | HOME、BAM、四个脚底 geom 的碰撞与摩擦 |
| `m9f_env_cfg.py` | 场景、61D/14D、命令、奖励、终止、随机化、课程、PPO 超参 |
| `m9f_mdp.py` | 通用库没有的双脚合并、站高门控和课程函数 |
| `m9f_train.py` | 环境到通用 runner 的组装、保存与同任务续训 |
| `m9f_evaluate.py` | 固定命令下的站高、位移、速度和接触指标 |
| `m9f_replay.py` | 同一环境下的 HD 视频 |
| `m9f_export.py` | checkpoint → ONNX（normalizer 烘进图，61→14） |

actor 观测保持运行时布局：gyro 3 + gravity 3 + pos 14 + vel 14 + last action 14 + twist 3 + head 4 + body 6 = **61**。动作是 **14D**，`scale=1.0`。本课只做平地前向行走；head/body 槽保留形状但不扩展新任务。

先用 M9d 已知可行走 checkpoint 做**推理基线**。它若在新环境也走不了，应修环境，不能开始 PPO。基线通过后，新策略才从随机权重训练。

## 架构决定

采用 [0012](../../decisions/0012-m9f-own-velocity-task.md)：在通用框架上手搓完整 handmade Velocity 任务。

没选“官方工厂 + 替换 EntityCfg”，因为任务定义仍属于官方，学不到自己的模型怎样映射到训练。没选重写 PPO、物理引擎和 BAM，因为本课要学的是机器人任务设计，不是再造基础库。

## 代码领读

### 1. 四个 geom，仍然是两只脚

`m9f_robot.py:12-18` 显式列出左右脚的薄层和 `_part2`。碰撞、接触传感和摩擦随机化共用这组名字，避免只控制薄层。

传感器因此输出四路。`m9f_mdp.py:13-17` 先把它们恢复为“左脚两层、右脚两层”，后续奖励只认两只脚：

```python title="m9f_mdp.py · 双脚合并 sketch"
def _perfoot(values):
    return values.reshape(values.shape[0], 2, 2)

contact = _perfoot(sensor.data.found).any(dim=2).float()
air_time = _perfoot(sensor.data.current_air_time).max(dim=2).values
```

不能把四路 `found` 直接配两个脚部位点：滑动奖励形状会错，腾空也会重复计数。

BAM 还要求按环境展开 `dof_frictionloss` 和 `dof_damping`。`m9f_env_cfg.py:265-269` 注册自己的 startup 事件，声明这两个字段；不重写 BAM。

### 2. 前进要有梯度，低姿态不能拿行走分

`m9f_mdp.py:39-70` 把三种奖励接在同一个站高门控上：

```python title="m9f_mdp.py · 奖励设计 sketch"
gate = ((root_z - 0.085) / (0.115 - 0.085)).clamp(0, 1)

tracking = exp(-((command_vx - actual_vx) / std) ** 2) * gate
progress = (actual_vx / command_vx).clamp(0, 1) * gate
air_reward = feet_in_valid_air_range * gate
```

高斯跟踪在 `actual_vx=0` 处对速度没有前进梯度，所以增加线性、封顶的 `progress`。门控让 85 mm 以下的爬行或深蹲不得分，115 mm 以上才全额计分。策略先靠直立和姿态项获得可行走姿态，再从前进项学习迈步。

`m9f_train.py:45-85` 只负责组装：创建同一个 env cfg、包成 rsl_rl 环境、交给通用 `VelocityOnPolicyRunner`。`--resume-from` 延续的是我们自己某一阶段的 checkpoint，不是官方权重。

## 训练阶段与参数

参数集中在 `m9f_robot.py`、`m9f_env_cfg.py`（含 `make_handmade_rl_cfg`）和 CLI。不要按文件从头背；按一次 iteration 的时间线查：这一步改的是动力学、是逐步奖励 r，还是梯度夹紧。

```flow LR
启动 --> 采样A
采样A --> 算账B
算账B --> 更新C
更新C --> 课程D
课程D --> 采样A
```

一次 iteration ≈：用当前策略采 `num_steps_per_env` 步 → 算回报与优势 → PPO 更新 → 课程/存盘。选动作时还没有 G−V；N 个 env 共用一份策略权重。

### 0 · 启动（只做一次）

建 N 个平行环境，初始化 actor / critic。

| 旋钮 | 位置 | 作用 |
|---|---|---|
| `HANDMADE_ROBOT_CFG`、BAM（`kp_fw` / `vin_*` / `delay_*`）、脚底 `condim`/摩擦 | `m9f_robot` | 定物理与执行器；错了后面奖励全歪 |
| 脚接触 / 自碰 / 脚高传感器 | `env_cfg` 传感器段 | 给奖励和 critic 特权观测供数 |
| `--num-envs` | CLI / `scene.num_envs` | 平行世界数；仍是一份 θ |
| `hidden_dims`、`obs_normalization`、`init_std` | `make_handmade_rl_cfg` | 网络壳与初始探索 |

### A · 采样（每轮前半；尚无优势）

每步：观测 → 抽样动作 → 仿真 → 奖励 r 写入缓冲。

| 旋钮 | 作用 |
|---|---|
| actor 观测：噪声、延迟、`biased`；删线速度 / height_scan；head/body 槽补零 | 决定策略看见的 61D；脏观测只给 actor |
| critic 脚接触 / 腾空走 `m9f_mdp` 双脚合并 | 估分器用的特权信息 |
| 动作 `scale=1.0`、舵机正则 | `目标角 = HOME + action` |
| 命令 `vx∈(0.2,0.4)`、站立比例、重采样周期 | 本课只练前走；命令进观测并进入奖励 |
| 奖励权重与站高门控 `0.085→0.115` | 每步 r 的评分标准；太矮不得前进/腾空分 |
| 重置时摩擦 / 质心 / armature；`nan_state` 终止 | 换局身体略不同；数值炸则结束该局 |
| `num_steps_per_env=24` | 每个 env 连采多少步再进入算账 |

改奖励或门控 = 改缓冲里的 r，从而改后面的优势，不直接改梯度公式。

### B · 算账

用缓冲里的 `(观测, 动作, r)` 算回报 G 与优势 A；不倒带重跑。

| 旋钮 | 作用 |
|---|---|
| critic 网络 + 其观测 | 输出 V(s) |
| `gamma=0.99`、`lam=0.95` | 折扣与 GAE；把逐步 r 折成优势 |

### C · PPO 更新

按优势正负拧策略；估分器让 V 贴近回报目标。

| 旋钮 | 作用 |
|---|---|
| `clip_param`、`learning_rate`、`schedule`、`desired_kl` | 一次别改太猛；按 KL 调步长 |
| `num_learning_epochs`、`num_mini_batches` | 同批数据多学几遍、拆小批 |
| `entropy_coef`、`max_grad_norm`、value loss 相关 | 探索、梯度裁剪、估分器损失 |

此时奖励表已经变成缓冲中的数，不再单独出现。

### D · 课程与存盘

| 旋钮 | 作用 |
|---|---|
| `action_rate` 权重阶梯、`standing_envs` 比例阶梯 | 改变**下一轮**采样的评分与命令分布（步数按 `iteration×24`） |
| `save_interval`、`max_iterations` | 存盘与停训 |

### 评估时（`play=True`）

关掉摩擦/质心等随机与课程，actor 无噪声；加载 checkpoint 只推理，不再走算账与更新。`m9f_evaluate` / `m9f_replay` 用这一模式。

### 想改什么 → 动哪

| 意图 | 优先动 | 主要落在 |
|---|---|---|
| 多采平行样本 | `--num-envs` | A |
| 跟速度更紧 / 防蹲着蹭 | 跟踪与前进权重、站高门控 | A → B |
| 更像真机电池与延迟 | BAM `vin_*`、`delay_*` | 启动与 A 的动力学 |
| 更新别太猛 / 多探索 | `clip_param`、`lr`、`entropy_coef` | C |
| 先学会走再逼平滑、多练站 | curriculum 两段 | D → 之后的 A |

## 配置答疑（env_cfg 细读）

下面补的是领读「训练阶段与参数」之后、对着 `m9f_env_cfg.py` 追问的一批点。读配置时按「谁生成数 → 谁读数 → 谁加权求和」查，比按行号背快。

### 命令 `twist`：是不是观测里的 command？

是。命令管理器里的 `twist` 被观测项 `generated_commands(command_name="twist")` 读走，进 actor 的 3 维（vx, vy, wz）。head/body 那 4+6 维是另两项，本课填零，不是这段。

`UniformVelocityCommandCfg` 写的是**采样规则**，不是某一帧的固定向量。运行时按 `ranges` 均匀抽；约 `rel_standing_envs` 比例的 env 整段清零；每个 env 在 `resampling_time_range` 到期后重抽。本课 `vy`/`wz` 范围为 0，只练前走。评估脚本会改成固定 vx，不再按训练范围乱抽。

### 动作为什么是 14 维？谁定义的？

没有在网络配置里手写 `action_dim=14`。链路是：

```flow TB
duck_handmade_xml --> HANDMADE_ROBOT_CFG
HANDMADE_ROBOT_CFG --> joint_pos_action
SERVO_JOINT_PATTERN --> joint_pos_action
joint_pos_action --> action_dim
```

`actions["joint_pos"].actuator_names` 设为舵机正则（排除 `passive_` 前缀），按名匹配可控铰链；`action_dim` 等于匹配个数。XML 里左右腿各 5 + 头颈 4 = 14。文件顶上的 `ACTION_DIM = 14` 只做导出校验，不建空间。`joint_pos` / `joint_vel` 观测也用同一正则，与动作对齐。

### 观测 61 维从哪来？字典顺序靠谱吗？

脚手架 `make_velocity_env_cfg` 先铺好观测表；本课删掉 actor 的线速度与 height_scan，并加上 head/body 零槽。最终拼接顺序跟 **Python 3.7+ dict 插入顺序**走，当前布局是约定而非语言级硬契约。改 `terms` 插入顺序会改 61 的分段含义；导出与 daemon 契约依赖这份顺序稳定。

`deepcopy` 改 `joint_pos`/`joint_vel` 的 `asset_cfg`，是避免和脚手架共享同一份 params 对象；**不会**把 61 维改成随机噪声。噪声是观测项上另配的 corruption。

### `track_forward_velocity` 的 `std` 是什么？

奖励形如 `exp(-((cmd_vx - actual_vx) / std)^2)`。`std` 是误差带宽：越小越严。写成 `math.sqrt(0.1)` 是让方差一眼等于 0.1（误差平方到 0.1 时奖励约 0.37）。这是塑形超参，不是从物理推导出来的；官方线速度跟踪常用 `sqrt(0.25)`，本课只盯前向一维，收紧到 `sqrt(0.1)`。

### 为什么把 `track_angular_velocity` 的 weight 写成 0？

脚手架里这项默认 weight=2.0。本课是在脚手架上改，不是空表新建。**不写成 0，它会继续进总奖励。** 本课命令 `ang_vel_z` 恒为 0，留着会逼「别转」，和专心前进抢梯度。`weight==0` 时奖励管理器跳过该项。

### `upright` 是人形专用吗？蜘蛛还要吗？

脚手架就有 `upright`。`make_velocity_env_cfg` 是通用速度任务底座：官方既挂 G1 人形，也挂 Go1 四足。它量的是指定 body 相对重力（或地形法向）歪了多少。地面多足若仍要「肚子朝下别翻」，通常仍有用，只换 `body_names` 与松紧；若故意侧爬或「上」不是世界竖直，应关掉或改语义。

### `air_time` 是自定义的吗？还能自己加奖励吗？

脚手架已有 `air_time` 名；本课用 `m9f_mdp.feet_air_time` **整项覆盖**，并加了命令门控与站高门控。自定义做法：写返回形状 `(num_envs,)` 的函数，再 `RewardTermCfg(func=..., weight=..., params=...)` 挂上。观测与终止同一套路。

### 多项奖励怎么合成？能改成累乘吗？

框架写死为加权求和：`Σ (raw × weight × dt)`。没有 `aggregation="product"` 开关。若要相乘，做成**一个**自定义 term，在函数内部算乘积再返回。

### 脚手架里还有哪些奖励？本课动了啥？

脚手架默认 13 项：`track_linear_velocity`、`track_angular_velocity`、`upright`、`pose`、`body_ang_vel`(0)、`angular_momentum`(0)、`dof_pos_limits`、`action_rate_l2`、`air_time`(0)、`foot_clearance`、`foot_swing_height`、`foot_slip`、`soft_landing`。

本课：替换线速度跟踪 / air_time / foot_slip；新增 `forward_velocity`、`self_collisions`；角速度跟踪 weight=0；删掉 `foot_swing_height`、`soft_landing`；其余多半只改参数。

### events 和 curriculum 干什么？

| | 时机 | 本课例子 |
|---|---|---|
| **events** | startup / reset / interval，改仿真世界 | 脚摩擦、质心、armature、BAM 字段、reset 位姿；评估时关掉随机 |
| **curriculum** | 按全局步数改规则 | `action_rate` 权重加重、站立 env 比例升高；评估时 `clear` |

events 管「世界何时被动手脚」；curriculum 管「训练日程何时变难」。

### 没有脚手架会怎样？还有哪些脚手架？

没有 `make_velocity_env_cfg`，就要从 `ManagerBasedRlEnvCfg` 自填观测/动作/命令/奖励/事件/课程（脚手架约 450 行 + `tasks/velocity/mdp/`）。本课 340 行大多是差异化。同级还有：`make_tracking_env_cfg`、`make_lift_cube_env_cfg`；Cartpole 无 `make_*`，直接组 cfg。

### `microduck_rl` 有多少训练任务？

注册表共 **33** 个可训 task id = **18** 个主任务 + **15** 个 `-Backlash-` 孪生。技能族 13 种（Velocity、VelStand、StandUp、SitStand、GroundPick、BallKick、Roulade、Rollers、Swizzle、RollerCrouch、RollerSlope、RollerStandUp、Spin）；左右脚踢球是同一 BallKick 改脚训两次。

### 网络维度在哪定义？怎么拿捏？

| 端 | 谁定 |
|---|---|
| 输入 / 输出 | 环境：观测表总维、动作匹配关节数（本课 61→14） |
| 隐层 | `make_handmade_rl_cfg` 的 `hidden_dims=(512,256,128)` |

进出维跟任务接口走；隐层是容量旋钮，本课照抄官方腿式速度惯例。改隐层不改 ONNX 契约；改观测项或关节才会。

### `nan_state` 终止干什么？

`robot_state_is_nan`：根状态、关节、脚传感器出现非有限数时结束该局（`time_out=False`，算失败不是时间到）。防止物理偶发发散把整批更新弄脏。

### 工期直觉（写 cfg ≠ 上机）

熟练、有 MJCF、用速度脚手架：冒烟级 cfg 常见 **0.5～2 天**；走到「能前走不太倒」常要 **1～2 周** 调奖励与物理。工业上演示级 sim-to-real 多按月计；可交付系统常按半年到一年以上——瓶颈在真机、安全与场景覆盖，不在抄满 `RewardTermCfg`。

## 验收

在教程仓根执行。云机为 RTX 4090；先登录：

```bash
ssh -p 23 root@117.50.221.204
```

以下命令在云机执行。先设解释器与工作目录：

```bash
export PY=/cloud/cloud-ssd1/microduck-m9d/microduck_rl/.venv/bin/python
cd /cloud/cloud-ssd1/microduck-m9f
```

**1. 已知策略验环境。** 这是推理对照，不是新策略的初始权重：

```bash
$PY impl/scripts/m9f_evaluate.py \
  --checkpoint /cloud/cloud-ssd1/microduck-m9d/microduck_rl/logs/rsl_rl/velocity/2026-10-29_08-00-43_velocity/model_999.pt \
  --output artifacts/baseline.json --num-envs 16 --seconds 5 --vx 0.3
```

预期：正常站高并明显前移。本课实测站高约 0.120 m、5 秒前移约 0.741 m。

**2. 自有 PPO 冒烟：**

```bash
$PY impl/scripts/m9f_train.py --num-envs 64 --max-iterations 5 \
  --save-interval 5 --log-dir logs/smoke_64x5 --smoke
```

预期：actor 61D、action 14D、BAM 14 关节，最后 `smoke_ok=1`，无 NaN。

**3. 从随机权重短训：**

```bash
$PY impl/scripts/m9f_train.py --num-envs 512 --max-iterations 300 \
  --save-interval 50 --log-dir logs/gated_512x300
```

短训后先评估，不要直接堆到 4000 轮：

```bash
$PY impl/scripts/m9f_evaluate.py \
  --checkpoint logs/gated_512x300/model_299.pt \
  --output artifacts/gated_299.json --num-envs 16 --seconds 5 --vx 0.3
```

**4. 短训站高和位移都正常后，扩到 2048 env 并续到总第 1000 轮：**

```bash
$PY impl/scripts/m9f_train.py --num-envs 2048 --max-iterations 1000 \
  --save-interval 100 --resume-from logs/gated_512x300/model_299.pt \
  --log-dir logs/main_2048x1000
```

终局定量验收：

```bash
$PY impl/scripts/m9f_evaluate.py \
  --checkpoint logs/main_2048x1000/model_999.pt \
  --output artifacts/main_999.json --num-envs 16 --seconds 5 --vx 0.3
```

通过条件：站高约 0.11–0.13 m、5 秒平均位移至少 0.5 m、速度方向正确、air-time 不异常。实测分别为 **0.1212 m、1.431 m、vx=0.294、air-time=0.162 s**。

生成 HD 回放：

```bash
MUJOCO_GL=egl $PY impl/scripts/m9f_replay.py \
  --checkpoint logs/main_2048x1000/model_999.pt \
  --out-dir artifacts/videos --video-length 250 --vx 0.3 --no-terminations
```

**5. 导出 ONNX（仍在云机）。** 走自有 `m9f_export.py`，不调用官方 `scripts/export.py` / `mjlab_microduck`：

```bash
$PY impl/scripts/m9f_export.py \
  --checkpoint logs/main_2048x1000/model_999.pt \
  --output artifacts/policy_export.onnx
```

预期打印 `export_ok=1`，且 `obs_dim=61`、`action_dim=14`。normalizer 经 runner `as_onnx` 烘进图。

**6. 回本机：拷 checkpoint、ONNX、视频与评估 JSON。** 在教程仓根新开终端（本机）：

```bash
mkdir -p tutor/artifacts/m09f/checkpoints tutor/artifacts/m09f/videos
```

```bash
scp -P 23 root@117.50.221.204:/cloud/cloud-ssd1/microduck-m9f/logs/main_2048x1000/model_999.pt \
  tutor/artifacts/m09f/checkpoints/model_999.pt
```

```bash
scp -P 23 root@117.50.221.204:/cloud/cloud-ssd1/microduck-m9f/artifacts/policy_export.onnx \
  tutor/artifacts/m09f/policy_export.onnx
```

```bash
scp -P 23 root@117.50.221.204:/cloud/cloud-ssd1/microduck-m9f/artifacts/videos/model_999.mp4 \
  tutor/artifacts/m09f/videos/main_model_999.mp4
```

```bash
scp -P 23 root@117.50.221.204:/cloud/cloud-ssd1/microduck-m9f/artifacts/main_999.json \
  tutor/artifacts/m09f/evaluation.json
```

**7. 本机宿主机仿真（用本课 ONNX）。** 先可打开训练回放作对照：

```bash
xdg-open tutor/artifacts/m09f/videos/main_model_999.mp4
```

肉眼应看到持续直立前进，而不是深蹲碎步。

再开三个终端，用自训 ONNX 驱动我们的栈（动作尺度 1.0，与训练契约一致）。二进制若未编出，先：

```bash
docker compose -f impl/docker-compose.yml exec rust cargo build --bins
```

终端 A（仓库根）：

```bash
.venv/bin/python impl/sim/duck_body.py --viewer --port 7811
```

应弹出 MuJoCo；默认 handmade BAM。

终端 B（`impl/`）：

```bash
cd impl
```

```bash
env MINIDUCK_POLICY=../tutor/artifacts/m09f/policy_export.onnx \
  MINIDUCK_WALK_ACTION_SCALE=1.0 \
  ORT_DYLIB_PATH=third_party/onnxruntime/lib/libonnxruntime.so \
  ./target/debug/miniduckd --sim 127.0.0.1:7811
```

终端 C（仍在 `impl/`）：

```bash
./target/debug/mini-duckctl enable
```

等约 2 秒站稳后：

```bash
./target/debug/mini-duckctl move 0.3 0 0 --secs 10
```

观察：约 10 秒内明显前移、保持 upright。看够后 Ctrl+C 停 B/A。

自动总门：`bash accept/m09f-b01.sh`；完整跑证在 `tutor/runs/m09f-b01.txt`。

## 与原版差异

D71：训练身体与任务实现都由本仓拥有，官方 M9d 仅作已知可行走对照。见 [spec/deviations.md](../../spec/deviations.md)。

## 坑

- **训练结束不等于会走。** 第一次短训站高正常，5 秒只走 0.0086 m；第二次也只有 0.0217 m。
- **只加前进分会出现低姿态投机。** 第三次达到 1.824 m，但站高只有 0.0714 m，所以仍失败。
- **从低姿态策略续训不一定最好。** 对照续训位移 0.346 m；从零使用站高门控的短训达到 0.960 m，因此主线选后者。
- **最后一个 checkpoint 也要重新测。** 评价固定使用同一命令、同一 play 配置和实际位移，不用 mean reward 替代。
- **`.pt` 不能直接喂给 `miniduckd`。** 必须走自有 `m9f_export.py`；官方 `scripts/export.py` 绑的是官方任务，本课不用。

## 自测

**1. 为什么四个脚底 geom 不能直接送进原来的双脚滑动奖励？**

- A. MuJoCo 不支持四个 geom
- B. 四路接触与两个脚部位点形状不同，腾空也会重复计数
- C. `_part2` 不能参与物理碰撞
- D. 只要 PPO 迭代够多就会自动对齐

<details><summary>答案</summary>

B。先按左右脚各自的两层合并，再计算接触、滑动和腾空。

</details>

**2. 已有高斯速度跟踪，为什么还要线性前进项？**

- A. 高斯项不能计算角速度
- B. 线性项可以减少显存
- C. 高斯跟踪在实际速度为零处没有前进梯度，线性项能从站立状态直接奖励向前速度
- D. 线性项允许低姿态获得全部分数

<details><summary>答案</summary>

C。前进项提供梯度；是否得分还要经过站高门控。

</details>

**3. 什么结果才允许进入 1000 轮主训练？**

- A. mean reward 上升即可
- B. 训练进程正常退出即可
- C. 先保存 ONNX 即可
- D. 短训在固定命令下同时保持正常站高、产生有效位移，并通过视频检查

<details><summary>答案</summary>

D。训练成功和步态成功是两道门；站高与位移必须一起满足。

</details>

**4. 本课导出 ONNX 为什么必须走 `m9f_export.py`，而不是官方 `scripts/export.py`？**

- A. 官方脚本不能写出 `.onnx` 文件
- B. 官方导出绑官方任务与机器人；本课主路径不 import `mjlab_microduck`，要用自有 env/robot 把 normalizer 烘进图
- C. 只有云机才能跑官方导出
- D. daemon 只认文件名 `policy_export.onnx`

<details><summary>答案</summary>

B。自有导出与训练共用同一套 env/robot；探针应为 61→14，且 `MINIDUCK_WALK_ACTION_SCALE=1.0`。

</details>

**5. 加大站高门控或前进奖励权重，主要改变 PPO 循环里的哪一段？**

- A. 只改 C 段的 `clip_param`
- B. 改 A 段每步的 r，从而改变 B 段算出来的优势
- C. 只改 D 段存盘间隔
- D. 只改 actor 的 `hidden_dims`

<details><summary>答案</summary>

B。奖励与门控在采样时写入缓冲；算账与更新吃的是这些 r，不是在 C 段再读一遍奖励表。

</details>

**6. 本课动作 14 维主要来自哪里？**

- A. `hidden_dims` 最后一维写成 14
- B. `ACTION_DIM = 14` 常量强制建网
- C. `joint_pos` 动作按舵机正则匹配到 14 个铰链，`action_dim` 等于匹配个数
- D. PPO 会自动把任意机器人压成 14 维

<details><summary>答案</summary>

C。`ACTION_DIM` 只做导出校验；网络输出维由环境动作空间推断。

</details>

**7. 脚手架里已有 `track_angular_velocity`（默认 weight=2）。本课若不改 weight，会怎样？**

- A. 该项自动删除
- B. 继续按默认权重进入总奖励
- C. 只影响 critic，不影响 actor
- D. 只在 `play=True` 时生效

<details><summary>答案</summary>

B。本课是改脚手架，不是空表；要关掉须 `weight=0` 或删除该项。

</details>

**8. events 与 curriculum 的分工更接近哪一句？**

- A. 都是每步拼进 61D 观测的项
- B. events 在特定时机改世界；curriculum 按训练进度改难度或权重
- C. 两者都只在导出 ONNX 时运行
- D. curriculum 负责采样动作，events 负责算优势

<details><summary>答案</summary>

B。评估时常清 curriculum、并关掉摩擦/质心等随机 event。

</details>

---
template: sheet
title: M9f Bite 1：手搓 handmade 训练环境（第 30 章）
subtitle: 从自己的 MJCF 写任务；基线验环境，PPO 从随机权重学会直立前进
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
```

六个模块各有边界：

| 模块 | 自己负责什么 |
|---|---|
| `m9f_robot.py` | HOME、BAM、四个脚底 geom 的碰撞与摩擦 |
| `m9f_env_cfg.py` | 场景、61D/14D、命令、奖励、终止、随机化、课程、PPO 超参 |
| `m9f_mdp.py` | 通用库没有的双脚合并、站高门控和课程函数 |
| `m9f_train.py` | 环境到通用 runner 的组装、保存与同任务续训 |
| `m9f_evaluate.py` | 固定命令下的站高、位移、速度和接触指标 |
| `m9f_replay.py` | 同一环境下的 HD 视频 |

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

回本机终端，把视频拉回并观看：

```bash
scp -P 23 root@117.50.221.204:/cloud/cloud-ssd1/microduck-m9f/artifacts/videos/model_999.mp4 \
  tutor/artifacts/m09f/videos/main_model_999.mp4
xdg-open tutor/artifacts/m09f/videos/main_model_999.mp4
```

肉眼应看到持续直立前进，而不是深蹲碎步。此回放证明的是训练环境内步态，不冒充 daemon 部署验收。自动总门：`bash accept/m09f-b01.sh`；完整跑证在 `tutor/runs/m09f-b01.txt`。

## 与原版差异

D71：训练身体与任务实现都由本仓拥有，官方 M9d 仅作已知可行走对照。见 [spec/deviations.md](../../spec/deviations.md)。

## 坑

- **训练结束不等于会走。** 第一次短训站高正常，5 秒只走 0.0086 m；第二次也只有 0.0217 m。
- **只加前进分会出现低姿态投机。** 第三次达到 1.824 m，但站高只有 0.0714 m，所以仍失败。
- **从低姿态策略续训不一定最好。** 对照续训位移 0.346 m；从零使用站高门控的短训达到 0.960 m，因此主线选后者。
- **最后一个 checkpoint 也要重新测。** 评价固定使用同一命令、同一 play 配置和实际位移，不用 mean reward 替代。

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

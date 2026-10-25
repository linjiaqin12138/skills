---
template: sheet
title: M9b Bite 1：Velocity 配方地图（第 22 章）
subtitle: 官方 cfg 与 obs.rs 静态对账；61D 块地图与命令采样契约
source: clone-from-scratch-tutor · 第 22 章
---

训练侧 Velocity 的命令范围、动作缩放与我们的 61D 布局已由契约脚本钉死。本课不跑 GPU、不 step env，也不重教 PPO。

```callout ok 静态对账先于整仓 uv sync
`m9b_velocity_contract.py` 只解析官方 cfg 与 `obs.rs` 常量。可选 pytest 无 venv 时跳过，不算失败。
```

## 问题

公开 `velstand` 权重能在自建身体上站立行走。下一步要自己训策略，先要确认训练侧 Velocity 配方与运行时观测契约一致。

布局错了不会 panic。策略照样推理，机器人却以「像调参问题」的方式摔倒。见 `impl/src/obs.rs:1-5`。下标是协议，不是字段名；背景见[观测向量](../concepts/observation-vector.html)。

本课消除的故障是：训练 cfg 里的 twist 范围、`action.scale`、61D 块起点与 `obs.rs` 偏移各说各话，却要等到真机或仿真摔倒才发现。

## 设计

```sequence num
participants: velocity cfg, 契约脚本, obs.rs
契约脚本 -> velocity cfg: 读 microduck_velocity_env_cfg
velocity cfg --> 契约脚本: lin/ang 范围与 scale
契约脚本 -> obs.rs: 读 OBS_LEN 与 OFF_*
obs.rs --> 契约脚本: 偏移常量
契约脚本 -> 契约脚本: 对照 EXPECTED 与块地图
```

契约脚本站在训练 cfg 与运行时布局之间。它把两边解析成同一组期望值，打印 61D 块地图，不一致则 `FAIL` 退出。

PPO 采样记 `(o,a,r)` 后算优势再按符号更新；本课只钉 `o`/`a` 与命令采样的形状。奖励权重是评分标准、DR 为健壮——改权重看分项，不在这里展开。

**来源：decisions/0003-m9b-velocity-contract.md**

```text
## 上下文

M9b 目标是 CPU 侧学透训练管线。Bite 1 只钉 Velocity 配方的 **scene/obs/action 契约**（61D obs、14D action、twist 采样范围）以及与我们 `impl/src/obs.rs` 的对账。完整 `uv sync` microduck_rl 会拉 CUDA/torch 轮子，本 VM 无 GPU，且 Bite 1 不需要 step env。先修返回摘要已掌握 PPO/奖励/DR 直觉，本 Bite 不重教。

## 备选

1. 在 `impl/scripts/` 写契约对账脚本：解析 `reference/microduck_rl` 的 velocity cfg 关键常量 + 对照 `obs.rs` 偏移；验收默认只跑静态断言与既有 `obs` 单测。若本机已有可用 RL venv，可选跑 CPU pytest 子集。
2. Bite 1 强制 `uv sync` 全依赖并跑通全仓 `pytest tests/`，再写课文。
3. 不写脚本，只靠课文领读官方 cfg，验收改成口头自测。

## 决定

采用备选 1。本 Bite 交付物在 `impl/scripts/m9b_velocity_contract.py`（及脚本自测所需最小夹具）；不改 miniduckd 控制逻辑。reward/curriculum 深读留给 Bite 2；train/export 命令留给 Bite 3。官方源只读 `reference/microduck_rl/`（或 `MICRODUCK_RL_ROOT`），禁止抄进 `impl/`。

## 放弃的成本

备选 2 能一次证明「官方测试绿」；放弃它的代价是本 Bite 不把 CUDA 安装与 pytest 全绿当成硬门槛，避免环境装机淹没配方地图本身。

备选 3 写作最快；放弃它的代价是必须有可重复的契约断言，否则「我们 61D 与训练一致」只能靠记忆。

## 失效边界

若官方把 actor obs 从 61D 改版，或命令范围写入不可静态解析的工厂深处，本脚本解析策略必须重写。若用户要求本机必须带 RL venv 作 CI 硬门，改判备选 2。
```

## 代码领读

期望值写在脚本顶部。命令范围、原地转弯比例、动作缩放、维数与偏移都是硬编码契约，不是运行时探测。

```python src=impl/scripts/m9b_velocity_contract.py lines=18-49
```

`check_official_cfg` 用正则从官方 `microduck_velocity_env_cfg.py` 抽出赋值，再与 `EXPECTED_*` 逐项比对。找不到赋值或数值漂移都会 `fail`。

```python src=impl/scripts/m9b_velocity_contract.py lines=130-148
```

`check_obs_rs` 解析 `OBS_LEN`、`ACTION_LEN` 与各 `OFF_*`。常量写成命名符号，是为了改一处偏移时立刻变成可见漂移，而不是在 fill 循环里静默写错相邻块。见 `impl/src/obs.rs:15-16`。

```rust src=impl/src/obs.rs lines=10-28
```

```python src=impl/scripts/m9b_velocity_contract.py lines=151-168
```

`Observation::build` 按同一组偏移填槽。位置减 home；`last_action` 直接拷；body 的 x/y/yaw 恒写 0，因为训练未绑定这些轴。

```rust src=impl/src/obs.rs lines=85-118
```

块地图打印八块：gyro、gravity、pos、vel、last_action、twist、head、body。宽度和必须为 61，相邻块首尾相接，起点与 `OFF_*` 一致。

| 块 | 起点 | 宽 |
|---|---:|---:|
| gyro / base_ang_vel | 0 | 3 |
| gravity / projected_gravity | 3 | 3 |
| pos / joint_pos_rel | 6 | 14 |
| vel / joint_vel | 20 | 14 |
| last_action | 34 | 14 |
| twist | 48 | 3 |
| head / head_pose | 51 | 4 |
| body / body_pose | 55 | 6 |

动作是 14 维，嘴不在策略输出里。`scatter_action` 与读关节共用 `joint_of`，两套映射会让嘴之后的关节全体错开一格且毫无报错。见 `impl/src/obs.rs:48-49`。

## 验收

在仓库根执行：

```bash
bash accept/m09b-b01.sh
```

容器内：`docker exec miniduck-rust bash /work/accept/m09b-b01.sh`。

脚本注释中的预期：

| 项 | 预期 |
|---|---|
| A | `cargo test obs::` 布局相关单测全绿 |
| B | cfg：`lin_vel_x=(-0.4,0.4)`，`lin_vel_y=(-0.3,0.3)`，`ang_vel_z=(-1.0,1.0)`；`TURN_IN_PLACE_FRACTION=0.15`；`action scale=1.0`；与 `obs.rs` 维数/偏移一致 |
| C | 打印 61D 块地图，宽度和为 61 |
| D | 可选 CPU pytest；无 venv 则跳过，不算失败 |

实测见 `tutor/runs/m09b-b01.txt`：五条 `obs::` 单测 ok；契约脚本 PASS；可选 pytest SKIP；`exit_code=0`。

## 自测

**1. 本 Bite 默认不强制 `uv sync` 全依赖，主因是？**

- A. PPO 算法已过时，不必装训练栈
- B. 完整 sync 会拉 CUDA/torch，本 VM 无 GPU，且本课不需要 step env
- C. 官方 cfg 禁止在本机读取
- D. `obs.rs` 单测已经覆盖命令采样范围

<details>
<summary>答案</summary>

B。决定 0003 采用静态对账：解析 cfg 常量 + 对照偏移。CUDA 装机不是本课门槛。A 与本课无关；C 错，脚本必须读官方 cfg；D 错，单测不解析训练侧命令范围。
</details>

**2. `OFF_POS` 从 6 改成 7，而策略权重仍按旧布局训练，最可能出现？**

- A. `Observation::build` 立刻 panic
- B. ONNX Runtime 拒绝加载会话
- C. 进程正常跑，机器人却表现为走路不稳或摔倒
- D. `cargo test` 强制失败后无法启动守护进程

<details>
<summary>答案</summary>

C。文件头写明：下标契约错了不会 panic，策略照样跑，故障像调参问题。契约脚本与布局单测就是为了在 `cargo test` / 验收里先暴露，而不是等到真机。A/B 不是默认路径；D 夸大了测试与启动的耦合。
</details>

**3. 验收打印 `SKIP optional pytest（无 …/.venv）` 时，本 Bite 结论是？**

- A. 验收失败，必须先装 RL venv
- B. 验收通过；可选子集未跑，不算失败
- C. 只通过了 A，B/C 契约对账无效
- D. 必须改判决定备选 2 才能继续

<details>
<summary>答案</summary>

B。`accept/m09b-b01.sh` 与决定 0003 都把无 venv 的 pytest 标为跳过。实测 run 在 SKIP 后仍打印 `M9b Bite 1 验收 PASS`。A/C/D 把可选项当成硬门。
</details>

**4. 官方 `joint_pos_action.scale` 对账期望是多少？若部署侧缩放写成别的数，直接后果是？**

- A. 期望 `0.15`；只影响原地转弯比例
- B. 期望 `1.0`；动作幅度与训练分布不一致，关节目标整体偏大或偏小
- C. 期望 `61`；观测维数会跟着变
- D. 期望 `14`；嘴关节会被写入策略输出

<details>
<summary>答案</summary>

B。契约期望 `scale=1.0`。缩放是动作契约的一部分；改错不会自动改 obs 维数。`0.15` 是 `TURN_IN_PLACE_FRACTION`；`61`/`14` 是维数，不是 scale。
</details>

**5. 块地图里 `body / body_pose` 起点是 55。`build` 对 body 的 x/y/yaw 怎么处理？**

- A. 写入调用方传入的真实里程计
- B. 三轴都从 IMU 积分得到
- C. x/y/yaw 恒写 0；只写训练绑过的 z/roll/pitch
- D. 整块 body 在推理前被裁掉，长度变成 55

<details>
<summary>答案</summary>

C。见 `impl/src/obs.rs:111-118` 与 `BodyPose` 注释：训练未绑定的轴写 0，避免策略看到从未见过的通道。向量仍是 61 维，不是裁到 55。
</details>

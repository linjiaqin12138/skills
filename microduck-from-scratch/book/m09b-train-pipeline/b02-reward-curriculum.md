---
template: sheet
title: M9b Bite 2：奖励与课程（第 23 章）
subtitle: Velocity 真配方上的投机方向、符号约定与课程节奏
source: clone-from-scratch-tutor · 第 23 章
---

先修已交付 PPO 三步、权重=评分标准、DR 健壮、投机看分项。本课把这些接到官方 Velocity 真值表：改哪一项、谁赢、何时加平滑税。不跑 GPU、不 step env。

```callout warn 行为袋是教学代理
场景 `walk` / `roll` 是分项得分字典，不是物理仿真。它只证明「同一套权重下谁的总分更高」；真实 PPO 轨迹分布可能更花，但字面投机的方向不变。
```

## 问题

部署侧要自己训走路。公开权重能站立行走，不代表你改配方时知道后果。

典型故障不是 panic，而是「看起来在优化」：关掉直立、放大前进，滚着蹭位移的总分反超平稳跟踪；或者一上来把 `action_rate` 拧到满，探索期「什么都不做」更划算。另一类是小指令——命令 `0.3` 看起来不走，配方侧线索在跟踪高斯的 `std≈√0.1`，而不是观测维数。

本课消除的故障是：只背权重数字，不会用「谁赢 / 何时加税 / 小误差有没有梯度」做可迁移判断。

## 设计

```flow TB
分项得分 -> *总分: ×权重
总分 -> 策略投机方向
```

总分是 $\sum_i w_i s_i$。负权重项的 $s_i\ge 0$ 表示违规量（mjlab cost 习惯）：权重为负时违规拉低总分。策略优化的是这个字面，不是你心里的「像在走」。

课程是另一条轴：平滑税与站立比例按 env step 阶梯变硬。

```flow LR
弱 action_rate -> 发现步态
发现步态 -> 强 action_rate: 再收抖动
站立比例低 -> 站立比例高: 并行爬坡
```

**来源：decisions/0004-m9b-reward-curriculum.md**

```text
## 上下文

Bite 1 只静态对账 obs/action，用户反馈无知识增量、不该独立成章。先修已交付 PPO/奖励/DR 直觉。本 Bite 要把直觉接到 **Velocity 真配方**：`microduck_velocity_env_cfg.py` 的奖励权重、符号约定、`action_rate` / `standing_envs` 课程表，以及「优化奖励字面」的投机场景。仍不 GPU、不 step env。

## 备选

1. 脚本从官方 cfg 抽出奖励表与课程阶梯，再用若干行为场景在抽出的权重下打分；验收断言表值、阶梯，以及「谁赢」的排序——每条对应 AGENTS 一条守则。
2. 只写课文领读 AGENTS.md + cfg，验收靠选择题 HTML，无脚本断言。
3. 强制 `uv sync` 后跑训练侧 MDP 单测与 env 冒烟。

## 决定

采用备选 1。交付 `impl/scripts/m9b_reward_curriculum_map.py`（抽表+场景打分）。不改 miniduckd。train/export 操作仍留给 Bite 3。场景是教学代理（离散行为袋），不是物理仿真；课文必须写明这一局限。

## 放弃的成本

备选 2 写得快；放弃它的代价是必须有可复跑的权重/阶梯断言，否则「track_lin std=√0.1」仍停留在口耳。

备选 3 更接近真训；放弃它的代价是本课不把 CUDA 安装当门槛，场景袋用离散代理代替 step env。

## 失效边界

若官方重写奖励函数名或改成不可静态解析的工厂，抽表策略失效。若场景袋与真实 PPO 轨迹分布严重不符导致教学误导，改用可 CPU 跑的 mdp 单元测试夹具。
```

与本课场景直接相关的 AGENTS 守则（全文在 `reference/microduck_rl/AGENTS.md:123-176`，这里只摘用到的）：

| 标签 | 判断 |
|---|---|
| letter-of-reward | RL 优化奖励字面；未钉死的自由度会被挖空（滚着蹭位移等） |
| sign-convention | mjlab 系 cost ≥ 0 → 配负权重；自取负的 penalty 才配正权重——勿双重取负 |
| regularizer-timing | 平滑类（`action_rate`）在技能发现后再加码；探索期过重会让「不动」赢 |
| tracking-std | 高斯 `std` ≈ 你仍在意的误差尺度；过松则小误差几乎无梯度 |

## 代码领读

期望权重钉在脚本顶部。`action_rate_l2` 写的是 cfg **初值** `-0.1`，不是课程终点 `-1.0`。

```python src=impl/scripts/m9b_reward_curriculum_map.py lines=21-32
```

官方赋值在 velocity cfg 奖励段。直立故意偏强（`:300-301`）；`foot_slip` 故意偏弱（`:314`）；`air_time=3.0`、`body_ang_vel=-0.05`（`:333-338`）。跟踪与平滑初值：

```python src=reference/microduck_rl/src/mjlab_microduck/tasks/microduck_velocity_env_cfg.py lines=341-349
```

`check_A_rewards` 逐项抽出并断言；同时算出 `track_lin` 的实际 std。

```python src=impl/scripts/m9b_reward_curriculum_map.py lines=218-230
```

课程：`NUM_STEPS_PER_ENV = 24`（cfg 顶）。`action_rate_weight` 从 step 0 的 `-0.1` 爬到 `1500*24` 的 `-1.0`——先发现步态再收抖动（regularizer-timing）。`standing_envs` 从 `0.02` 爬到 `2000*24` 的 `0.25`。

```python src=reference/microduck_rl/src/mjlab_microduck/tasks/microduck_velocity_env_cfg.py lines=776-807
```

```python src=impl/scripts/m9b_reward_curriculum_map.py lines=233-252
```

投机场景：默认权重下 `walk` 总分高于 `roll`（实测约 `5.49 > 1.98`）。把 `upright` 置 0、把 `track_linear_velocity` 放大到 `16.0` 后，`roll` 反超——与先修练习同构，只是权重换成官方真表。

```python src=impl/scripts/m9b_reward_curriculum_map.py lines=255-286
```

分项里 `upright` 在默认配方下贡献约 `1.96`（walk）对 `0.08`（roll）：前进项再高也盖不过直立质量。黑客配方去掉这笔质量后，前进项主导，滚着蹭位移变成字面最优。

「0.3 不走」线索：`std ≈ 0.316`。误差远小于 std 时，高斯跟踪几乎不掉分。小指令在配方侧代价近零；部署侧「命令给了却像没走」先查这条，再查控制环。

```python src=impl/scripts/m9b_reward_curriculum_map.py lines=289-298
```

符号约定写进场景打印：负权重 × 违规量 ≥ 0，勿再把 cost 手搓成负数再配负权重（sign-convention）。`self_collisions` 在 cfg 里是 `weight=-1.0` 配 `mdp.self_collision_cost`（cost ≥ 0），见 `microduck_velocity_env_cfg.py:323-327`。

## 验收

在仓库根执行：

```bash
bash accept/m09b-b02.sh
```

容器内：`docker exec -w /work miniduck-rust bash /work/accept/m09b-b02.sh`。

脚本注释中的预期：

| 项 | 预期 |
|---|---|
| A | 抽出关键权重：`track_lin=2.0` 且 `std=√0.1`；`track_ang=2.0`；`upright=2.0`；`air_time=3.0`；`foot_slip=-0.1`；`action_rate_l2` 初值 `-0.1`；`body_ang_vel=-0.05`；`self_collisions=-1.0` |
| B | `action_rate`：step 0 → `-0.1`，`1500*24` → `-1.0`；`standing_envs`：0 → `0.02`，`2000*24` → `0.25` |
| C | 默认 `walk > roll`；`upright=0` 且放大前进项后 `roll > walk`；打印 AGENTS 标签 |
| D | 退出码 0；打印 `track_lin std≈0.316` 与小指令近零代价线索 |

实测见 `tutor/runs/m09b-b02.txt`：A–D 全绿；`m9b_reward_curriculum_map: PASS`；`exit_code=0`。

## 自测

**1. 默认 Velocity 权重下，离散袋里 `walk` 总分高于 `roll`，主因是？**

- A. `track_linear_velocity` 对 walk 更高，前进项单独决定胜负
- B. `upright` 等姿态质量压住了 roll 的高前进分；优化字面才会「像在走」
- C. `action_rate_l2` 已在 step 0 取到 `-1.0`，滚着蹭必输
- D. 行为袋是物理仿真，接触力直接否决翻滚

<details>
<summary>答案</summary>

B。实测分项里 upright 贡献约 1.96 vs 0.08；roll 的 track_lin 更高仍输。A 说反了；C 错，初值是 `-0.1`；D 错，袋是教学代理。
</details>

**2. 把 `upright` 改成 0、并把 `track_linear_velocity` 放大数倍后，最可能出现？**

- A. 验收脚本 panic，因为权重必须为正
- B. `walk` 与 `roll` 总分永远相等
- C. `roll` 总分反超 `walk`——字面最优变成滚着蹭位移
- D. 只有 `self_collisions` 会变号，其它项不变

<details>
<summary>答案</summary>

C。脚本黑客分支与先修同构：去掉直立约束并放大前进项 → 投机行为反超（letter-of-reward）。A/B/D 与实测断言不符。
</details>

**3. `action_rate_l2` 从 `-0.1` 课到 `-1.0`，而不是开局就 `-1.0`，是因为？**

- A. 平滑税过早会让探索期「不动」更划算，须先发现步态再收抖动
- B. 负权重不能一开始就小于 `-0.1`，解析器会截断
- C. `standing_envs` 与 `action_rate` 必须同一步数切换
- D. GPU 显存不够，只能晚期启用正则

<details>
<summary>答案</summary>

A。AGENTS regularizer-timing：平滑类在技能发现后加码。B/C/D 无依据。
</details>

**4. mjlab 系 `self_collision_cost` 返回 ≥ 0。Velocity cfg 给它配 `-1.0`。若误配成正权重，直接后果是？**

- A. 无影响，PPO 只看绝对值
- B. 违规变成加分，策略会去「种」碰撞
- C. 观测维数从 61 变成 62
- D. `track_lin std` 自动收紧到 √0.05

<details>
<summary>答案</summary>

B。sign-convention：cost ≥ 0 必须负权重；符号反了等于奖励违规。A/C/D 无关。
</details>

**5. 命令线速度约 `0.3`、实际也差不多，却感觉「没走」。配方侧优先查什么？**

- A. `OBS_LEN` 是否仍为 61
- B. `track_linear_velocity` 的 `std≈√0.1≈0.316`：小误差相对 std 几乎不掉分
- C. `TURN_IN_PLACE_FRACTION` 是否写成 0
- D. `air_time` 权重是否小于 1

<details>
<summary>答案</summary>

B。验收 D 与 AGENTS tracking-std：过松则小误差无梯度。A/C 属 Bite 1 契约；D 不解释「小指令近零代价」。
</details>

# 0011 M9f Bite 1：自写训练入口 + handmade 身体，不用官方 train CLI

- 状态：起草
- 来源：手搓
- 出现于：m09f-b01
- 被取代于：

起草可以改。用户确认这一课之后，手搓决定改为「已采纳」。用户确认 Phase 1 之后，原项目决定改为「已记录」。此后不改正文。要推翻，新开一篇，本篇状态改为「被取代」，并填上新编号。

## 上下文

M9d 用官方 `uv run train` + 官方 `robot_walk` 跑通了云训。用户裁决：那条路径训出的权重再「接入我们」没有学习意义；M9e 取消。  
下一步必须同时满足：① 训练身体是 `duck_handmade.xml`；② 训练入口是我们写的脚本（学习组装 env / runner，不是再包一层官方 CLI）。

官方 Velocity Flat 的 robot 来自 `MICRODUCK_WALK_ROBOT_CFG` → `robot_walk.xml`。PPO 实现在 rsl_rl，编排在 mjlab。

## 备选

1. **自写 `impl/scripts/m9f_train.py`**：调用官方 `make_microduck_velocity_env_cfg`，把 `cfg.scene.entities["robot"]` 换成 handmade 的 `EntityCfg`（`spec_fn` → `duck_handmade.xml`），再按 mjlab 同款方式起 rsl_rl runner。冒烟/长训都走这一入口。不调用 `uv run train`。
2. **继续 `uv run train`，只 monkeypatch / 改 vendor 里的 `get_walk_spec`**：改动小，但入口仍是官方 CLI，学不到组装过程；改 vendor 难维护。
3. **从零手写 PPO + 自建 env**：学习面最大；一个 Bite 不可验收，也偏离「复用可下载栈、学设计」的教程边界。

## 决定

采用备选 1。

- 奖励 / 观测 / BAM / 课程：复用官方 Velocity 工厂（配方已在 M9b 学过）。
- 身体：handmade `EntityCfg`，与 `impl/sim` 部署同源 XML。
- 入口：我们的 Python 脚本拥有 `num_envs` / `max_iterations` / log 目录等参数；内部 import mjlab、rsl_rl，**主路径零 `subprocess` 调 `train`**。
- 仍可用官方仓的 uv 环境跑该脚本（依赖在 microduck_rl 的 venv），但命令是 `uv run python /path/to/m9f_train.py …`，不是 `uv run train …`。

## 放弃的成本

备选 2 少写代码、冒烟最快；放弃它是因为用户要的是「自己写训练脚本」的学习，且 vendor 补丁不是可演进面。  
备选 3 能最深理解 PPO；放弃它是因为本里程碑验收是「handmade 上能训」，不是「自研算法库」。

## 失效边界

若 mjlab 不再暴露可替换的 `EntityCfg` / runner API，只能 fork 官方 train 模块——那时应新开决定，把「自写」收窄为 fork 维护，而不是退回裸 `uv run train`。  
若 handmade 与 Velocity 传感器名（如 `left_foot_collision`）长期对不齐且无法改 XML，应先开对齐 Bite，再训。

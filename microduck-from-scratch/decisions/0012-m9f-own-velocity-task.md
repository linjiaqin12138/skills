# 0012 M9f Bite 1：在通用框架上手搓 handmade Velocity 任务

- 状态：起草
- 来源：手搓
- 出现于：m09f-b01
- 被取代于：

起草可以改。用户确认这一课之后，手搓决定改为「已采纳」。用户确认 Phase 1 之后，原项目决定改为「已记录」。此后不改正文。要推翻，新开一篇，本篇状态改为「被取代」，并填上新编号。

## 上下文

旧 M9f 只自写了训练入口，机器人之后的观测、命令、奖励、接触、随机化、课程和 runner 仍来自官方 `mjlab_microduck` Velocity 实现。handmade 每脚两个 collision geom，官方任务只识别每脚一个 `*_foot_collision`，旧策略因此学会深蹲原地碎步。更重要的是，这种实现不满足用户“从自己的模型出发，学习训练脚本怎么写”的要求。

用户于 2026-10-30 裁决删除旧 M9f 实现和失败产物，重来。官方 M9d 已知可行走 checkpoint 保留，用作新环境的推理对照，不用来 warm-start 新策略。

## 备选

1. **通用 mjlab/rsl_rl 之上手搓完整 handmade Velocity 任务**：自有机器人配置、观测、动作、命令、奖励、终止、随机化、课程、PPO 超参与评估；物理引擎、管理器框架、PPO 和 BAM 作为依赖轮子。
2. **继续官方 Velocity 工厂，只修补双层鞋底**：代码最少，但任务定义仍由官方拥有，学习目标和模型对齐问题都没有从根上解决。
3. **连物理环境、PPO、BAM 也从零写**：学习面最大，但会把本 Bite 变成算法库与仿真器项目，无法聚焦机器人训练任务。

## 决定

采用备选 1。

- 主路径不 import `mjlab_microduck`，不调用官方 train/play CLI。
- 使用 MuJoCo、mjlab 通用组件、rsl_rl、BAM 和 PyTorch；这些库是轮子，不重写。
- `impl/scripts/` 自有 `m9f_robot.py`、`m9f_env_cfg.py`、`m9f_mdp.py`、`m9f_train.py`、`m9f_evaluate.py`、`m9f_replay.py`。
- 训练和部署保持 61D actor obs / 14D action，动作 scale=1.0，关节顺序对齐 `impl/src/obs.rs`。
- 每脚的两个 collision geom 都显式进入接触、摩擦和 air-time 配置。
- 首版只做平地 Velocity；先用 M9d checkpoint 验证环境，再从随机初始化短训。行为门通过前不堆长训、不接 daemon。
- 本 Bite 不实现 ONNX 导出、粗糙地形、起身或技能调度。

## 放弃的成本

备选 2 能直接继承官方长期调好的奖励和 sim2real 细节，开发量最小；放弃它意味着我们要自己维护这些任务细节，并用基线和分布指标证明没有写错。

备选 3 能暴露最多底层原理；放弃它意味着本课不会讲清 PPO 和物理引擎内部实现，只能学会如何正确组装和约束它们。

## 失效边界

若 mjlab/rsl_rl 的通用 API 无法表达 handmade 所需任务，应新开决定评估维护通用层扩展，而不是悄悄退回官方 MicroDuck 任务工厂。若 M9d 已知可行走策略在新环境也无法复现站走，则先修环境，不得开始 PPO 训练。

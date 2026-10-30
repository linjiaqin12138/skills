# M9f Bite 1 验收档案

- 日期：2026-10-30
- 决定：`decisions/0012-m9f-own-velocity-task.md`（起草，待用户确认）
- 实现：`impl/scripts/m9f_{robot,env_cfg,mdp,train,evaluate,replay}.py`
- 验收：`accept/m09f-b01.sh`
- 跑证：`tutor/runs/m09f-b01.txt`
- 课文：`book/m09f-handmade-train/b01-own-train-handmade.html`（第 30 章，strict 0 警告）

## 自有实现边界

- 主训练、评估、回放均无 `mjlab_microduck` import；不使用官方 train/play CLI。
- 通用轮子：MuJoCo、mjlab、rsl_rl、BAM、PyTorch。
- actor 61D / action 14D / action scale=1.0；BAM 14 个伺服关节。
- 四个脚底 collision geom 全部进入碰撞、接触和摩擦配置；MDP 按左右脚将四层合并为两路。

## 环境基线与冒烟

| 项 | 实测 |
|---|---|
| M9d 已知策略在自有环境推理 | z=0.12007m，5s 位移 0.74094m，actual vx=0.15131 |
| 自有 PPO 冒烟 | 64 env × 5 iter，`smoke_ok=1`，无 NaN |
| 静态验收 | 自有源码、四脚底、Python 语法全过 |

## 短训调查

| 版本 | 5s 位移 | 平均站高 | 结论 |
|---|---:|---:|---|
| 全命令、高斯跟踪 | 0.00862m | 0.11181m | 站稳不动 |
| 前向命令、高斯跟踪 | 0.02170m | 0.11093m | 仍不动 |
| 加线性前进奖励 | 1.82375m | 0.07144m | 低姿态快走，失败 |
| 加站高门控 | 0.95984m | 0.12080m | 短训通过，HD 为直立前进 |
| 低姿态策略续训对照 | 0.34607m | 0.11754m | 不如从零门控版，未选为主线 |

站高门控：低于 0.085m 时 track/forward/air 奖励为零，0.115m 以上全额；线性前进项解决高斯跟踪在 vx=0 处无前进梯度的问题。

## 主训练与终局

- 起点：`gated_512x300/model_299.pt`（从随机权重训出的自有策略）
- 主训练：2048 env，续训到总第 1000 轮，目录 `logs/main_2048x1000`
- 终局：`model_999.pt`；本机副本 `tutor/artifacts/m09f/checkpoints/model_999.pt`
- 日志：`tutor/artifacts/m09f/logs/main_2048x1000.log`

确定性评估：16 env、5s、vx=0.3，play 配置关闭随机化、推力和终止重置。

| 指标 | 实测 | 门 |
|---|---:|---:|
| root z mean | **0.12117m** | [0.105, 0.135] |
| root z min | 0.11347m | 无深蹲 |
| low-height fraction | **0.0** | 无 <0.09m 帧 |
| 平均水平位移 | **1.43062m** | ≥0.5m |
| 最小单环境位移 | **1.37171m** | 16 env 均有效前进 |
| actual vx mean | **0.29396** | 命令 0.3 |
| air-time mean | **0.16150s** | ≤1.5s |
| contact fraction | 0.52163 | 双脚交替接触 |

HD 回放：`tutor/artifacts/m09f/videos/main_model_999.mp4`，1280×720、250 拍，肉眼为持续直立前进，不是原地碎步。

## 总结论

`bash accept/m09f-b01.sh` 实跑 PASS：`own_source_gate_ok`、`handmade_contact_gate_ok`、`python_syntax_gate_ok`、`gait_check_ok=1`。

本 Bite 证明自有 handmade 环境与 PPO 能从随机权重学出有效直立前进步态。ONNX/daemon 部署不在本 Bite 验收范围内。

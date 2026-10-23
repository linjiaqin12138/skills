# M9 批次规划 · 仿真与训练（2026-10-16 用户裁决 v2：GPU 后置）

> 依据：docs/user-prefs.md 四批路线、deviations.md Phase 4 挂账、docs/recon-microduck-rl.md 侦察结论。
> 用户裁决：**GPU 步骤全部后置**——先在本 VM（CPU）学搭建仿真与训练管线，上云机器时再开训练里程碑。

## 算力裁决（2026-10-16）

- 主力：**云 GPU（Vast.ai RTX 5090，~$0.39/h）**；HF Jobs（L4 ~$0.70/h）为备选；本 VM 无 GPU（VMware 虚拟显卡），3060 Ti Windows 机未采用。
- 实例选型结论见 recon-microduck-rl.md 附录。

## 里程碑拆分（依赖序 = 执行序）

### M9a · 自建 MJCF 仿真模型（CPU，2-3 Bite）— 当前

- 目标：从零写鸭子 MJCF（基本几何体，绕开 STL 的 CC BY-NC-SA），接入我们 sim/duck_body.py，用现有 velstand 权重验收
- 内容：MJCF 建模（运动树/惯量/碰撞几何取舍/keyframe）；BAM M6 参数与 microduck_constants.py:163-176 逐项对账；**D4 收敛**——sim 协议面对齐官方 body_server.py（op 名 gain/torque、15 槽含嘴、PROTOCOL=1 握手、trunk 系投影重力）
- 验收：自建 MJCF 在本机 sim 站立 + vx=0.8 行走 ≥2.0m（复现 M8 Bite 8 门槛）
- 收敛：D4、D26（接触模型随重建）
- 新概念：MJCF 建模、惯量/碰撞几何

### M9b · 训练管线解剖（CPU，2-3 Bite）

- 目标：不动 GPU，把「怎么训」学透
- 内容：microduck_velocity_env_cfg.py 七件套精读（scene/obs/action/reward/event/termination/curriculum）；奖励塑形守则（rl:AGENTS.md:123-176）；PPO 与 rsl_rl 配置；训练操作手册（uv run train 参数、--agent.resume 断点续训、MICRODUCK_WARM_START 热启动、wandb、--hf-jobs）；export.py/manifest 校验工具链走查；仓内 CPU tests（pytest tests/ 不 step env）
- 验收：能回答「改某个奖励权重 → 重训 → 导出 → 验证」全链路每一步的命令与预期；CPU tests 全绿
- 新概念：PPO 与奖励塑形、域随机化

### M9c · 策略面调度语义对齐①（CPU，可与 M9b 并行或交错）

- 目标：不依赖自训权重的挂账先清
- 内容：D23+D32 FallPredictor 移植（陀螺外推预测 + Limp/Posing 三态机；仿真里可构造摔倒验证）；odometry crate 评估与移植决策
- 收敛：D23、D32、odometry
- 验收：accept-m9 系列（FakeIo/sim 构造跌倒场景）+ 全回归

### M9d · 云上训练（GPU，上机器时开）

- 目标：Vast.ai 5090 实例搭建 + 官方冒烟 + 小规模首训
- 内容：uv sync（注意 protobuf/onnx/zmq 三个 override）；冒烟 64 envs × 5 iter；首训 1024-4096 envs × 数百-1500 iter；infer_policy.py A/B（BAM vs --no-bam）
- 验收：首训 checkpoint 在 infer_policy.py 里能站/能走雏形；学习曲线健康
- 前置：M9b（操作手册）、用户买好云实例

### M9e · Velocity 从零训 + 全链路闭环 + 调度对齐②（GPU + CPU）

- 目标：自训 ONNX 进我们自己的 sim + Rust daemon 闭环；剩余调度挂账清零
- 内容：正式训练 Velocity → export.py 出 ONNX → manifest 冒烟校验（[1,61]→[1,14]、NaN 拒绝）→ 速度扫描 0.1-0.4 m/s 在我们闭环里验收；manifest schema 2 对齐（chain/unwind_s/slot/kind）、D28 mode 面、D29 chain/unwind、D30 热换/carry_over、D33 stand 槽裁决
- 收敛：「0.3 不走」（自训策略解决，行走验收加回 vx=0.3 档，阈值训完实测再钉）、D35 残余、D28/D29/D30/D33
- 验收：accept-m9e 速度跟踪误差带（分布级断言）+ 全回归

## 覆盖核对（Phase 2 硬性映射表）

| 挂账项 | 归宿 |
|---|---|
| D4 sim 协议面 | M9a |
| D26 接触穿透 | M9a |
| D23+D32 FallPredictor | M9c |
| odometry | M9c |
| 「0.3 不走」上游权重 | M9e |
| D35 roulade 残余 | M9e（自训能力落地后重训/复验） |
| D28 mode/roller 槽 | M9e |
| D29 chain/unwind | M9e |
| D30 热换/carry_over | M9e |
| D33 stand 槽 | M9e（注意基准变化：v5 起官方已无独立 stand 网络，可能自动消解，以 v0.14.1 行号证据为准） |

无孤儿项。D31（音频族）仍挂 M11，不在本批。

## 推断但未从代码确认的内容（本阶段）

1. 「0.3 不走」主因判定为策略欠跟踪（证据链见 recon §3），未排除我们运行时映射偏差——M9e 用 infer_policy.py --save-csv 对照实测定案
2. duck-body 无 BAM 是否意味着官方 sim2real 链路里 duck-body 仅用于冒烟——【合理推断】
3. 官方 ONNX 权重的 Hub 许可条款未核实

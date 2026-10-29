# M9 批次规划 · 仿真与训练（2026-10-16 用户裁决 v2：GPU 后置）

> 依据：tutor/user-prefs.md 四批路线、spec/deviations.md Phase 4 挂账、tutor/recon-microduck-rl.md 侦察结论。
> 用户裁决：**GPU 步骤全部后置**——先在本 VM（CPU）学搭建仿真与训练管线，上云机器时再开训练里程碑。

## 算力裁决（2026-10-16）

- 主力：**云 GPU（Vast.ai RTX 5090，~$0.39/h）**；HF Jobs（L4 ~$0.70/h）为备选；本 VM 无 GPU（VMware 虚拟显卡），3060 Ti Windows 机未采用。
- 实例选型结论见 recon-microduck-rl.md 附录。

## 里程碑拆分（依赖序 = 执行序）

### M9a · 自建 MJCF 仿真模型（CPU，2-3 项目 Bite + 1 练习）— 当前

- 目标：从零写鸭子 MJCF（基本几何体，绕开 STL 的 CC BY-NC-SA），接入我们 impl/sim/duck_body.py，用现有 velstand 权重验收
- 内容：MJCF 建模（运动树/惯量/碰撞几何取舍/keyframe）；BAM M6 参数与 microduck_constants.py:163-176 逐项对账；**D4 收敛**——sim 协议面对齐官方 body_server.py（op 名 gain/torque、15 槽含嘴、PROTOCOL=1 握手、trunk 系投影重力）
- 验收：自建 MJCF 在本机 sim 站立 + vx=0.8 行走 ≥2.0m（复现 M8 Bite 8 门槛）
- 进度：Bite 1–4 已确认收官（2026-10-25）。用户反馈 Bite 4（D4 协议命名）过简，属协议清账而非仿真主课
- 收敛：D4 / D26 / D70 已收敛
- 新概念：MJCF 建模、惯量/碰撞几何

### M9b · 训练管线解剖（CPU，2-3 Bite）

- 目标：不动 GPU，把「怎么训」学透
- 内容：microduck_velocity_env_cfg.py 七件套精读（scene/obs/action/reward/event/termination/curriculum）；奖励塑形守则（rl:AGENTS.md:123-176）；PPO 与 rsl_rl 配置；训练操作手册（uv run train 参数、--agent.resume 断点续训、MICRODUCK_WARM_START 热启动、wandb、--hf-jobs）；export.py/manifest 校验工具链走查；仓内 CPU tests（pytest tests/ 不 step env）
- 验收：能回答「改某个奖励权重 → 重训 → 导出 → 验证」全链路每一步的命令与预期；CPU tests 全绿
- 新概念：PPO 与奖励塑形、域随机化
- **拆课教训（2026-10-25）**：Bite 1 只做 obs/action 静态对账，用户判「无知识增量、不该独立成章」。此后 M9b 每课必须带可迁移判断（奖励投机/课程节奏/操作决策），契约核对并进正文附录，不再单开对账章。

### M9c · 策略面调度语义对齐①（CPU，可与 M9b 并行或交错）— **已收官（2026-10-29）**

- 目标：不依赖自训权重的挂账先清
- 内容：D23+D32 FallPredictor 移植（陀螺外推预测 + Limp/Posing 三态机；仿真里可构造摔倒验证）；odometry crate 评估与移植决策
- 收敛：D23、D32、odometry（估计器）；D50 odom 字段仍挂 M10
- 验收：`accept/m09c-b01.sh` + `accept/m09c-b02.sh` + 全回归；档案 `book/m09c-fall-recovery/acceptance.md`
- 进度：Bite 1/2 均已确认；决定 `0006`/`0007` 已采纳

### M9d · 云上训练（GPU）— **已收官（2026-10-29）**

- 目标：云 GPU 实例搭建 + 官方冒烟 + 小规模首训
- **算力修订（2026-10-29）**：用户选用国内云 **RTX 4090**（原定 Vast.ai 5090）；决定见 `decisions/0008-m9d-cloud-4090.md`
- 内容：uv sync（注意 protobuf/onnx/zmq 三个 override）；冒烟 64 envs × 5 iter；首训 1024-4096 envs × 数百-1500 iter（4090 OOM 则降 envs）；官方 export + headless infer A/B（BAM vs --no-bam）
- 验收：首训 checkpoint 经 `export.py` 后，在 headless `PolicyInference` 路径能站/能走雏形；学习曲线健康
- 前置：M9b（操作手册）、用户买好云实例
- 进度：Bite 1/2 均已确认；`0008`/`0009` **已采纳**；档案 `acceptance.md` + `acceptance-b02.md`

### M9e · 官方权重进我们 daemon — **取消（2026-10-29）**

- **取消理由（用户原话）**：官方模型 + 官方脚本训出的权重，再在官方仿真上能跑，接入我们的模型没什么意义。
- 草稿 `0010` / `accept/m09e-b01.sh` 作废，不交付。

### M9f · handmade + 自写训练脚本（GPU）— **进行中**

- 目标：策略在 **我们的** `duck_handmade` 上、用 **我们写的训练入口** 长大；训推同分布
- **硬约束**：入口脚本在 `impl/`；可 import mjlab / rsl_rl / 官方 env cfg 工厂；**禁止** `uv run train` / 官方 `train_cli` 当主路径；不从零重写 PPO
- **Bite 1（当前）**：自写 train 入口 + 把 `duck_handmade.xml` 换进 Velocity 的 robot `EntityCfg`；云冒烟 64×5；决定 `0011`
- 后续 Bite：同脚本长训 → 自写/薄封装 export → handmade 分布站/走验收；daemon 对接另排
- 注：官方 Velocity Flat 默认 `robot_walk.xml`（仅足底）；我们挂 handmade（groundcontact 系 AABB）。刻意让 **训练身体 = 部署仿真身体**
- 前置：M9a；M9d 仅证明云机能跑通（流程参考，配方不复用为终态权重）；**要 GPU**

## 覆盖核对（Phase 2 硬性映射表）

| 挂账项 | 归宿 |
|---|---|
| D4 sim 协议面 | M9a（Bite 4 已收敛） |
| D26 接触穿透 | M9a（已收敛） |
| D70 自建足底摩擦代理值 | M9a（Bite 3 修订后收敛） |
| D23+D32 FallPredictor | M9c（已收敛） |
| odometry | M9c（估计器已收敛；D50 字段→M10） |
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

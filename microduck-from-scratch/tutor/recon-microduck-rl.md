# microduck_rl 侦察报告（2026-10-16，M9 规划用）

> 来源：subagent 对 `upstream/microduck_rl/`（pollen-robotics/microduck_rl 浅克隆）的彻底侦察。
> 下文 `rl:` 前缀指该仓库内路径。全部结论带行号证据；【合理推断】已显式标注。

## 1. 训练管线

- 框架：**mjlab 1.3.0（MuJoCo Warp）+ rsl_rl PPO**（rl:README.md:6-10；`mjlab==1.3.0`、`warp-lang==1.12.0` 锁在 rl:pyproject.toml:17-18；`rsl-rl-lib 5.0.1` 见 rl:uv.lock:1688-1689）。策略网络纯前馈 MLP 512-256-128, ELU，actor/critic 均开 obs 归一化（rl:src/mjlab_microduck/tasks/microduck_velocity_env_cfg.py:912-927）；`num_steps_per_env=24`、`max_iterations=50_000`（同文件 947-948）。
- 训练入口：`uv run train <TASK_ID>`，console script `mjlab_microduck.train_cli:main` 转发 mjlab train（rl:pyproject.toml:53；rl:src/mjlab_microduck/train_cli.py:25-32）。任务经 `mjlab.tasks` entry point 注册（rl:pyproject.toml:40-41）。
- 配置：每个任务族一个 `rl:src/mjlab_microduck/tasks/microduck_*_env_cfg.py`（`ENABLE_*` 开关 + 常量在文件头，`make_..._env_cfg(play, rough)` 工厂，文件尾 `RslRlOnPolicyRunnerCfg`）；注册表 rl:src/mjlab_microduck/tasks/__init__.py:81-280。
- 典型命令：`uv run train Mjlab-Velocity-Flat-MicroDuck --env.scene.num-envs 4096`（rl:README.md:35）；断点续训 `--agent.load-checkpoint model_29999.pt --agent.resume True`（rl:README.md:51-53）；跨任务热启动 `MICRODUCK_WARM_START=1`（rl:AGENTS.md:215-222）。
- 资源：**CUDA GPU 必需**（训练走 MuJoCo Warp，rl:README.md:24）。4096 envs「可用步态约 1-2 小时」（rl:README.md:34）。官方主机 COACH2 / RTX 5090（rl:docs/velstand_policy.md:39）。预算经验：简单 episodic 技巧 ≈1000 iter@4096 envs；步态与恢复类 4000-6000 iter（rl:AGENTS.md:232-233）。无 GPU 可 `--hf-jobs` 提交 HF Jobs（rl:scripts/hf/README.md:34）。
- 冒烟惯例：长跑前先 `uv run train <TASK> --env.scene.num-envs 64 --agent.max_iterations 5`（rl:AGENTS.md:16, 25-26）。

## 2. 仿真资产与 duck-body

- MJCF 全在 `rl:src/mjlab_microduck/robot/microduck/`，onshape-to-robot 导出（每模型一个 `config_mjcf_*.json`）：
  - `robot_walk.xml` — Velocity 行走（砍躯干/头部碰撞）（rl:README.md:132；microduck_constants.py:16）
  - `robot_groundcontact.xml` — StandUp/SitStand/GroundPick/BallKick/Roulade（rl:README.md:133；microduck_constants.py:17-21）
  - `robot_allcollisions.xml` — 真全碰撞 70 geoms，**VelStand 实际用它**（microduck_constants.py:22-25；microduck_velstand_env_cfg.py:43-46）
  - `robot_groundcontact_rollers.xml` — 轮滑（14 驱动 + 4 个 passive_*wheel 被动轮铰链）（microduck_constants.py:28-29）
  - `robot_*_backlash.xml` — 每舵机串联 ±1° 齿隙铰链孪生，`add_backlash.py` 生成（rl:README.md:98-110）
  - `scene*.xml` = 机器人 + 地板 + keyframes（STAND/SIT/FOLD）（rl:README.md:138-139）
- **duck-body（官方仿真机器人进程）在本仓**：`rl:src/mjlab_microduck/sim/body_server.py`，console script `uv run duck-body`（rl:pyproject.toml:54-55）。
- 协议：TCP + NDJSON，握手带 `protocol` 版本号，`PROTOCOL = 1`（body_server.py:26-29, 59, 409-415）。默认端口 7801，多鸭递增（body_server.py:511, 562）。
- op 集（body_server.py:407-431）：`hello` / `read`（positions/velocities/currents_ma + trunk_z/trunk/sim_time + imu{gyro,gravity,quat}）/ `write`（15 维目标）/ `gain` / `torque` / `slow`（电压温度摆设值）/ `tof`（8x8 深度）。**无 `push` op**——抗扰只在训练 env 内部做。
- 与我们的差异：op 名 `gain`/`torque`（我们 `set_gain`/`set_torque`）；**线上 15 槽含 index 9 的 `mouth`，模型 14 执行器**，服务端插入/丢弃（body_server.py:33-37, 75-80）；重力服务端转成 trunk 系投影（body_server.py:159-167）；物理步长 0.005s 按实时推进（body_server.py:66, 439-500）。
- 注意：duck-body 用 **XML PD + `actuator_gainprm` 缩放**模拟 kp/limp（body_server.py:247, 385-388），**不是 BAM**。【合理推断】我们的 sim 接了 BAM 反而比官方 duck-body 更贴近训练分布。

## 3. 观测与动作空间

- 61 维 actor obs = 48 本体感知 + 13 命令块，全家族共享保证热切换（rl:README.md:164-167）：
  `[base_ang_vel(3), projected_gravity(3), joint_pos_rel(14), joint_vel(14), last_action(14)] + [twist(3), head_pose(4), body_pose(6)]`（rl:scripts/infer_policy.py:656-682, 465-467）。不用的命令槽补零保留。critic 另有特权观测（velocity_env_cfg.py:540-543）。
- 与我们运行时 61 维逐段一致（joint_pos 相对 DEFAULT_POSE，joint_vel 1 拍延迟、IMU 0-1 拍延迟 + 噪声/安装误差 DR，velocity_env_cfg.py:560-613）。
- 动作：14 维，`JointPositionActionCfg.scale = 1.0`，语义 PD 目标位置偏移：`motor_target = DEFAULT_POSE + action * scale`（rl:scripts/infer_policy.py:121, 873-875）。DEFAULT_POSE = HOME/STAND2（infer_policy.py:126-141；microduck_constants.py:116-140）。manifest 允许 action_scale ∈ (0, 2]（rl:src/mjlab_microduck/publish/manifest.py:144）。
- 命令采样（velocity 主配方，固定范围）：**vx ∈ (-0.4, 0.4), vy ∈ (-0.3, 0.3), wz ∈ (-1.0, 1.0)**（velocity_env_cfg.py:648-650）；站立 env 比例 0.02→0.25 课程（642, 794-807）；15% env 强制原地转（654；mdp.py:4743-4772）；全零桶 `zero_command_prob`（mdp.py:5336-5343）。
- 「0.3 不走」证据链：① 线速度跟踪高斯 std = √0.1 ≈ 0.316 m/s（velocity_env_cfg.py:342-343），小指令误差代价近零；② 官方实测 0.4 指令只走出 0.17 m/s（rl:docs/velstand_policy.md:186）；③ |wz| < 0.4 从未训练（mdp.py:4811-4812 注释）。【合理推断】主因是策略小指令增益低 + 跟踪 std 过松；不能完全排除我们运行时映射偏差，可先在 infer_policy.py 复现（--debug/--save-csv）。

## 4. 策略导出与 manifest

- 唯一安全导出：`uv run scripts/export.py <TASK> --wandb-run-path <...>` → `run_export` → mjlab runner `export_policy_to_onnx`（rl:src/mjlab_microduck/export.py:264）。**观测归一化器烘进图**（export.py:1-9, 253-257）。纯前馈，全仓无 LSTM/GRU。
- opset 未显式 pin；测试桩用 17（rl:tests/test_publish_manifest.py:77）。【合理推断】走 torch.onnx 默认（torch 2.9 ≥17）。
- publish 强制校验 `[1,61] -> [1,14]` + 50 步冒烟拒 NaN/恒定输出（rl:src/mjlab_microduck/publish/manifest.py:268-309）。
- Manifest 本仓生成 **schema 2**（manifest.py:22；注意与主仓 daemon 侧「v5 manifest」不是同一编号体系）：`kind`（episodic/perpetual）；`chain` = 按住按钮重复触发 episodic 技能（manifest.py:98-141）；`unwind_s` = perpetual 姿势结束后 daemon 用 idle twist 过渡秒数（manifest.py:129-141）；`slot` ∈ {walk, stand, sitstand, ground_pick, kick_left, kick_right, roulade}（manifest.py:43）；`command.encoding` ∈ {constant, phase, posture_flag}，publish 只支持 constant。

## 5. BAM 在训练侧

- **所有任务训练都用 BAM M6**（XL330 电压控制律 + 反电动势 + Coulomb/Stribeck/负载摩擦）（rl:README.md:112-122）。参数 rl:src/mjlab_microduck/robot/microduck_constants.py:163-176：`motor_name="xl330", model="m6", kp_fw=200, vin_range=(6.5,8.2), vin_drop_gain_range=(0,0.2), vin_min=6.0, delay 3-6 拍`；目标关节正则 `^(?!passive_).*`。
- bam 包来自 git `Rhoban/bam` 的 `mjlab_frictionloss` 分支（rl:pyproject.toml:87-89；uv.lock 锁 62bd8ce）。M6 参数文件在 bam 包内（`bam.model.load_model`），不在本仓。
- 两层薄封装（rl:src/mjlab_microduck/actuator/friction_dr_bam.py）：`FrictionDRBamActuator` per-env `friction_scale` 摩擦 DR（28-53）；`BacklashEncoderBamActuator` 固件 PD 读「穿过齿隙」的输出端编码器（64-104）。BAM 下 `dof_frictionloss` 清零，摩擦 DR 必须走 `friction_scale`（rl:AGENTS.md:70-73）。
- infer_policy.py 集成：`bam.mujoco.MujocoController` 接管非 passive 执行器（rl:scripts/infer_policy.py:59-105）；`--vin/--vin-drop-gain/--kp-fw` 钉 DR 单值，`--no-bam` 回退（infer_policy.py:1277-1282）。

## 6. 域随机化与奖励（velocity 主配方）

- DR：躯干 CoM ±3mm→±15mm 课程、头部 CoM、质量惯量 ±5%（dr.pseudo_inertia）、BAM 摩擦 scale (0.9,1.1)、armature ±10%、推扰 ±0.3 m/s 每 3-6 s、IMU 安装误差 ≤6°（仅 actor）、编码器偏置 ±0.015 rad、足底摩擦 (0.7,1.3)、BAM 电压/压降/延迟（velocity_env_cfg.py 57-83, 374-512, 592-635；microduck_constants.py:163-176）。
- 奖励：track_lin 2.0/std √0.1、track_ang 2.0/std √0.5、upright 2.0、pose 1.0（仅腿）、air_time 3.0（0.125-0.300s 窗）、foot_slip -0.1、action_rate -0.1→-1.0 课程、body_ang_vel -0.05、angular_momentum -0.02、self_collisions -1.0、head_pose_tracking 2.0、head_pose_bias 0→3.0 课程（275-345, 710-745）。
- 课程：步数 = iteration×24；`microduck_mdp.reward_weight` 阶梯权重表（778-907；rl:AGENTS.md:191-207）。
- 奖励设计守则（rl:AGENTS.md:123-176，踩坑记录，直接当 checklist）：符号约定（mjlab 系惩罚 ≥0 配负权重）、无 jackpot、正奖励不得门控在坏状态上、平滑正则要在技能发现之后上、跨 env 比的是「奖励质量」不是权重绝对值。

## 7. 多策略

- 每个技能独立任务独立训练：Velocity、VelStand、StandUp、SitStand、GroundPick、BallKick、Roulade、Rollers、Swizzle、RollerCrouch、RollerSlope、RollerStandUp、Spin，各有现成 cfg + 注册项（rl:src/mjlab_microduck/tasks/__init__.py:81-234），每个主任务还有 `-Backlash-` 孪生（236-280）。
- kick_left/kick_right = 同一 BallKick env 改 `KICK_FOOT` 训两次（microduck_ball_kick_env_cfg.py:37-42）。
- **VelStand 不从零训**：walk 教师（441tzs6d@3750）+ stand 教师（69u48n8l@9750）热启动 + BC 蒸馏（rl:docs/velstand_policy.md:25-33；rl:src/mjlab_microduck/tasks/distill.py:1-44）；Hub v8 = `0045i2si@2749`。
- 全策略共享 61D obs 契约支持运行时热切换（rl:README.md:82-89）。
- mode(Walk/Roller) 在训练侧 = 换机器人模型（轮滑 XML 带被动轮），任务 id 独立（microduck_constants.py:305-318；__init__.py:176-182）。【合理推断】运行时 mode 只是选哪族 ONNX。

## 8. 复现难度

- 依赖：uv、CUDA GPU（或 HF Jobs）、PyPI（mjlab 1.3.0 / warp-lang 1.12.0 / torch 2.9.1 / rsl-rl-lib 5.0.1 / onnxruntime / scipy）+ git 源 bam；wandb 账号（项目 `mjlab_microduck`）；HF token 仅 --hf-jobs/publish 需要。
- **不需要 mocap/数据集**；export.py 的 motion_file 分支是 mjlab 通用 tracking 路径，microduck 没用（rl:src/mjlab_microduck/export.py:84-98）。`pickup_*` 是另一条线（被拿起分类器）。
- 预训练模型仅复刻 velstand 谱系需要（教师 checkpoint 在官方 wandb，公开可读）；从零训 Velocity 无外部资产。
- 仓内无训练级 CI；`rl:tests/` 全 CPU 的 cfg 不变量与奖励函数回归测试（不 step env）。`uv run --with pytest pytest tests/` 全 CPU 可跑（rl:README.md:253-260）。
- 安装坑（已锁进 pyproject 注释）：aarch64 torch 走 cu129 索引、torch 必须直接依赖且 == pin、protobuf/onnx/zmq 三个 override 不能删（rl:pyproject.toml:66-83）。

## 9. 许可证

- 代码 + reward/env 配置 + MJCF XML + config_mjcf_*.json + .part 元数据全 **Apache-2.0**（rl:LICENSE-MODELS.md:17-19）——照搬 reward/课程参数无障得。
- 仅 `.stl` 网格 **CC BY-NC-SA 4.0**（:12-19）：自建 MJCF 用基本几何体可绕开。
- 官方 ONNX 权重许可不在本仓；【合理推断】沿用 Hub 模型卡条款，从零训练不依赖它。

## 附：Vast.ai 实例选型结论（2026-10-16）

- 模板选 **NVIDIA CUDA**（devel），不选 PyTorch 模板——uv 自举 Python 3.12 与 torch==2.9.1（wheel 自带 CUDA runtime），镜像只需干净 Ubuntu + 驱动透传。
- 关键筛选：5090 + **CUDA Max Good ≥ 12.9（驱动 ≥575）** + 可靠性 >95% + 网速好；磁盘 ≥60GB；内存 ≥32GB；SSH（direct ssh）启动。
- 首次搭建 on-demand，正式训练可试 interruptible（仓库支持 `--agent.resume` 断点续训）。

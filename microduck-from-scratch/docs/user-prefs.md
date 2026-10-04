# 项目级偏好（microduck-from-scratch）

> 只放本项目特有的目标与约束。用户级偏好（技术背景、语言用词、通用流程）在 `skills/clone-from-scrach-tutor/user-prefs.md`。

## 总目标与路线（2026-10-04 用户裁决）

**对整个机器人项目形成工程闭环，不是单学 robotd 软件。** 已定路线，四批推进：

1. **M8 收尾（当前）**：C 线——先把非硬件、不依赖硬件的内容对齐（robot.move+vy / robot.enable{on,toggle} / robot.subscribe / safeToRestart 预检），并移植 BAM 执行器模型让 sit/roulade 真正站起来；其余功能缺口**不豁免**，分配到后续批次
2. **M9 批次：仿真与训练**——自建仿真模型（MJCF）、自己训策略（不满足于用现成 ONNX），代码继续对齐相关部分（microduck_rl 侧）
3. **M10 批次：驱动与硬件**——学驱动编程；硬件后补（不是现在），**可能不用官方硬件**；已有 Xbox 手柄（原版 padd 正是用 Xbox Wireless Controller 开发测试的，reference/padd/）。代码继续对齐相关部分——**此批完成后项目完全对齐原项目**
4. **M11 批次：结构与外壳**——自学 3D 建模与打印，外壳模块自己构建。与代码无关

**路线含义**：用户倾向自定义硬件 → 现成策略（XL330+BAM 动力学上训练）会因动力学不同失效 → 仿真/训练批（M9）是必经主线而非选修。闭环：自己的硬件选型 → 自己的仿真模型 → 自己训练策略 → 自己的驱动跑真机。

## 相关资源（2026-10-04 侦察结论）

- 官方硬件不开源（无 STL/BOM/装配文档；仅 `elec_RPI_Robot_HAT` 转接板开源）；代码即规格书：XL330×15（ID/波特率/寄存器见 reference/duck-control/src/bus.rs）、LSM6DSV16X IMU、Radxa Zero 3W、NP-F550 电池
- 第三方复刻生态：`fanhao375/microduck-replica(-cad)`（BOM+SolidWorks/STEP/3MF，Feetech 版已实体站立）、`LuwuDynamics/xgoduck_hardware`、前身 `apirrone/Open_Duck_Mini`（全开源）
- BAM 执行器模型：Rhoban/bam（纯解析式，16 标量），官方 CPU 集成蓝图 microduck_rl `scripts/infer_policy.py`；移植进 sim/duck_body.py 估 60–80 行 + 换 groundcontact 碰撞资产 + timestep 1ms→5ms。roulade/sitstand 训练用 `robot_groundcontact.xml`（11 碰撞 geom）

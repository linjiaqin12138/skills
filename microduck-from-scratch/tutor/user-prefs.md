# 项目级偏好（microduck-from-scratch）

> 只放本项目特有的目标与约束。跨项目的学习偏好在 `~/.clone-from-scratch-tutor/user-prefs.md`。内部路线依据与 audit 写到 `tutor/prerequisites/`，不抄进课文。

- **架构图分层的顶层固定为** miniduckd / mini-duckctl / mini-updaterd / duck_body.py 四进程 + install/current 资产；高发图题材：执行器动力学链、调度仲裁、IPC 握手时序。

## 课文写作硬规则（2026-10-25 用户裁决）

- **没有设计取舍、只是对账/抽表/列常量的材料：禁止写「代码领读」。** 读那种代码没有知识增量，浪费时间。
- 数字与断言放验收脚本注释或章末附录表；正文只写可迁移判断（为什么、改了会怎样、和哪条守则对应）。
- 契约核对并进附录，不单开成章（Bite 1 教训）。抽表脚本可以留作 CI，但不等于一课。

## 验收与环境措辞（2026-10-28 用户裁决）

- **自动断言在容器，眼睛在宿主机。** 编译 / `cargo test` / `accept-*.sh` / 无界面数据提取：容器。人眼要看的运动与物理观感：宿主机仓库根 `.venv` + `impl/sim/duck_body.py --viewer`，再接宿主机 `impl/target/debug/miniduckd --sim` 与 `mini-duckctl`（binary 由容器编出，在宿主机跑）。
- **课文「验收」凡涉及仿真运动或物理现象，必须写出宿主机可视化演示的 raw 命令**（开 viewer → 起 daemon → `enable` / `move` 等可观察步骤），不能只用容器数字验收冒充「看得见」。容器步骤仍保留作回归门。
- 容器 noVNC / MJPEG 不替代宿主机原生 viewer；宿主机 viewer 建法见 `tutor/dev-container.md`。

## 总目标与路线（2026-10-04 用户裁决）

**对整个机器人项目形成工程闭环，不是单学 robotd 软件。** 已定路线，四批推进：

1. **M8 收尾（当前）**：C 线——先把非硬件、不依赖硬件的内容对齐（robot.move+vy / robot.enable{on,toggle} / robot.subscribe / safeToRestart 预检），并移植 BAM 执行器模型让 sit/roulade 真正站起来；其余功能缺口**不豁免**，分配到后续批次
2. **M9 批次：仿真与训练**——自建仿真模型（MJCF）、自己训策略（不满足于用现成 ONNX），代码继续对齐相关部分（microduck_rl 侧）。**算力裁决（2026-10-16）**：云 GPU 主力（Vast.ai 5090），GPU 步骤后置——先 CPU 侧（本 VM）学仿真搭建与训练管线，再开训练里程碑。拆分见 tutor/m9-plan.md
   - **2026-10-29 用户裁决（修订）**：先开 **M9f**（handmade + 自写训练入口）。同日晚改回：**M9e 恢复执行**——M9d 自训的官方分布 ONNX 接到我们的 `miniduckd` + handmade BAM（对照官方 viewer）。M9f 仍保留（训推同分布）。
3. **M10 批次：驱动与硬件**——学驱动编程；硬件后补（不是现在），**可能不用官方硬件**；已有 Xbox 手柄（原版 padd 正是用 Xbox Wireless Controller 开发测试的，upstream/padd/）。代码继续对齐相关部分——**此批完成后项目完全对齐原项目**
4. **M11 批次：结构与外壳**——自学 3D 建模与打印，外壳模块自己构建。与代码无关

**路线含义**：用户倾向自定义硬件 → 现成策略（XL330+BAM 动力学上训练）会因动力学不同失效 → 仿真/训练批（M9）是必经主线而非选修。闭环：自己的硬件选型 → 自己的仿真模型 → 自己训练策略 → 自己的驱动跑真机。

## 相关资源（2026-10-04 侦察结论）

- 官方硬件不开源（无 STL/BOM/装配文档；仅 `elec_RPI_Robot_HAT` 转接板开源）；代码即规格书：XL330×15（ID/波特率/寄存器见 upstream/duck-control/src/bus.rs）、LSM6DSV16X IMU、Radxa Zero 3W、NP-F550 电池
- 第三方复刻生态：`fanhao375/microduck-replica(-cad)`（BOM+SolidWorks/STEP/3MF，Feetech 版已实体站立）、`LuwuDynamics/xgoduck_hardware`、前身 `apirrone/Open_Duck_Mini`（全开源）
- BAM 执行器模型：Rhoban/bam（纯解析式，16 标量），官方 CPU 集成蓝图 microduck_rl `scripts/infer_policy.py`；移植进 impl/sim/duck_body.py 估 60–80 行 + 换 groundcontact 碰撞资产 + timestep 1ms→5ms。roulade/sitstand 训练用 `robot_groundcontact.xml`（11 碰撞 geom）

## 先修能力评估

只记录项目特定能力与路线。跨项目的学习偏好在用户级文件。

| 能力 | 项目中要做什么 | 经验 | 路线 | 出口标准 | 状态 | 先修记录 |
|---|---|---|---|---|---|---|
| 从空白文件搭建带关节与执行器的 MJCF | M9a 自建身体模型前，亲手写出可编译、可跟踪的两关节腿链 | 用户 2026-10-25 报告已完成腿链练习与 MuJoCo 基础学习 | 项目内引导 → 返回项目实现 | 写出 `exercises/mjcf-leg-chain/exercise_leg.xml`，`accept/m09a-b02.sh` 全绿；能改关节轴并预测现象 | **用户报告已完成并明确继续后续学习**；作业文件不在仓内，本轮未复验，不标验收全绿 | 无（项目内引导，不建 prerequisites/） |
| PPO 闭环 + 奖励塑形 + 域随机化（工程直觉） | M9b 读懂 velocity reward/curriculum，能预测改权重后果并走通「改权重→重训→导出→验证」口头全链路 | 先修前零 RL 实操；先修中完成 PPO 三步、改权重练习、DR 工程理由 | **先修学习（已达出口）** | 见 `tutor/prerequisites/ppo-reward-dr.md` 出口三条 | **已达出口（2026-10-25）**；可开 M9b Bite 1 | [ppo-reward-dr.md](prerequisites/ppo-reward-dr.md) |

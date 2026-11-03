# Phase 2 · 自底向上实现路径（归档）

## 直接下载不重写
- 8 个 ONNX 策略权重：HF Hub `pollen-robotics/microduck-policies` @ v5（velstand/sitstand/ground_pick/ball_kick×2/roller/roller_crouch/roulade）
- MJCF 身体模型（upstream/kinematics/assets/alpha/robot_walk.xml）
- ONNX Runtime、MuJoCo

## 里程碑（依赖树序）
- M0 IPC 骨架：Unix socket NDJSON JSON-RPC daemon + CLI（hello/health/subscribe）
- M1 身体模型+FakeIo：15 关节表、RobotIo trait、50Hz 插值回 home
- M2 观测向量 61 维：gyro3+gravity3+pos14+vel14+prev_action14+command13；契约测试对齐布局
- M3 单策略推理：下载 velstand.onnx + ORT，load 时全量校验维度，闭环站立
- M4 仿真行走：TCP+JSON 接 MuJoCo（协议自定→M8 对齐原版），速度命令→行走，验收 10s 前移 ≥0.5m
- M5 安全层：deadman、关节/温度限位、IMU 跌倒→卸力
- M6 技能调度器：≥3 策略优先级链，切换连续性断言
- M7 支线 updaterd：releases 目录 + symlink 切换 + 健康门回滚
- M8 收敛验收：对照原项逐项核对技术栈/配置/接口/行为（后续按用户要求新增）

全程估 20h 纯手搓。语言：Rust（用户裁决）。

## M9 收尾规划（2026-11-02）

### M9g · 策略选择与热换（2 Bite）

- 目标：对齐 v0.14.1 的可选 stand 槽和运行中替换策略的状态连续性。
- 前置：M9e/M9f 已确认；61→14 ONNX 推理、调度器和策略 reset 已有。
- Bite 1（D33）：接入可选 `stand` 槽、低速命令选网、standing action scale/gain ratio；缺省不装 stand 时保持 velstand 兼站兼走。验收覆盖阈值两侧、无 stand 回退、换网 reset，以及站/走仿真回归。
- Bite 2（D30）：实现 `robot.loadPolicy` / `robot.reloadPolicies` 的异步加载与成功后交换；失败保留旧控制器；同文件/同摘要不重置 recurrent 状态，真正换网才按原版规则处理；`carry_over` 保留座位、技能和滤波状态。验收覆盖加载中继续跑旧网、坏文件不中断、热换连续性和 LSTM 合成夹具。
- 简化：完整配置写面与 `robot.policies`/`setSkill` 仍属 D52/D59 → M10；本里程碑只实现策略运行语义所需入口。
- 预计工作量：2 个独立 Bite；每课代码领读不超过 4 个模块。

### M9h · Walk/Roller 模式（1 Bite）

- 目标：收敛 D28，接入 `robot.mode` / `robot.setMode`、roller/roller_crouch 槽与对应动作尺度和调度行为。
- 前置：M9g，复用控制器重建与状态交接。
- 验收：模式查询/非法值拒绝/切换；roller 无 stand 网络时不按低速切 stand；roller 与 roller_crouch 在 handmade BAM 上完成可观察运动并安全回到模式基态；公开权重路径回归。
- 数据：直接下载官方 v5 roller/roller_crouch ONNX，不重写权重。
- 简化：CLI 完整 policy 管理树与配置持久化仍归 M10。
- 预计工作量：1 Bite。

### M9i · 技能链与 roulade 收敛（2 Bite）

- 目标：收敛 D29 的 chain/unwind 调度语义，再处理 D35 的 roulade 中段物理残余。
- 前置：M9h；M9f 自有训练/导出链路可用于必要的重训。
- Bite 1（D29）：实现 roulade chain 刷新、chain=true 重放与 unwind 窗口；ground_pick 的 0.7 交还点保持。验收用 FakeIo 钉状态序列、重复请求、抢占边界与换网 reset，再做仿真冒烟。
- Bite 2（D35）：在 chain/unwind 就位后复验 roulade 全程；若中段 fallen 仍存在，先用官方 groundcontact 对照分离调度与训练/身体因素，再决定是否需要自有技能训练。验收至少包含结束后 upright，并以全程 fallen 窗口作为本次收敛的明确目标；不得只靠调宽阈值核销。
- 简化：不为通过验收新增原版没有的推一把补偿；新训练若发生，复用 M9f 通用轮子并另写实现决定。
- 预计工作量：2 Bite；Bite 2 含 GPU 时以行为门决定训练规模。

## 覆盖核对

| 挂账项 | 里程碑 / Bite | 验收重点 |
|---|---|---|
| D33 stand 槽 | M9g Bite 1 | 阈值选网、无 stand 回退、增益与动作尺度 |
| D30 热换/carry_over | M9g Bite 2 | 旧网服务不中断、失败保留、状态连续 |
| D28 mode/roller | M9h Bite 1 | 模式入口、roller 槽与仿真行为 |
| D29 chain/unwind | M9i Bite 1 | 状态序列、重放、unwind、边界 |
| D35 roulade 残余 | M9i Bite 2 | 全程跌倒窗口与结束后 upright |

无孤儿项。D31 → M11；协议/配置写面残余仍按原簿归 M10。

## 尚未从代码确认的推断

1. M9g Bite 2 的入口可只覆盖 D30 运行语义而不提前实现完整 D52 配置面；具体载荷和持久化边界需在动手前逐行对照 reference。
2. D35 是否需要重训尚未定案；必须先做官方 groundcontact 对照，不预设自训一定必要。
3. roller/roller_crouch 在 handmade 身体上的物理表现尚未实测，M9h 验收阈值需在冒烟后由实测支持，不能沿用 walk 位移门槛。

## M11 · 结构与外壳（2026-11-02，当前路线）

> 用户裁决：从 M11 开始，跳过 M9 剩余项和 M10。被跳过的偏差不核销。M11 以学习者亲手建模和实物验收为主，不把官方仿真 STL 当作可打印 CAD 成品。

### M11a · 参数化建模与打印校准（2 Bite）

- 目标：能从一个空白 CAD 文档做出尺寸可控、可切片打印、能按实测误差修正的零件。
- 工具：**Onshape Free**（2026-11-02 用户确认），与原项目一致；Linux 浏览器使用。用户接受文档公开、仅非商业用途和联网依赖。
- 前置：创建 Onshape 文档；确认后续打印服务与测量条件。用户无打印机和卡尺，数字阶段可先开始。
- Bite 1：学习者亲手建立一个参数化校准件，包含基体、孔、圆角/倒角与明确基准；导出 STL/3MF，完成切片并记录尺寸预测。没有打印机时先交付数字模型与切片报告，实物门保留，不伪造通过。
- Bite 2：打印并测量校准件，比较孔径、外形和配合余量，修改参数后得到第二版；形成本机/打印服务的补偿记录。
- 验收：模型可重新生成、关键尺寸由参数驱动、网格无破损；实物版增加卡尺测量和一次配合检查。
- 简化：先不建完整鸭子，不打印官方视觉网格。

### M11b · 整鸭 1:1 结构复刻（4 Bite）

- **目标（2026-11-03 用户修订）**：完成整只 MicroDuck 的 **飞特 HD-1910 v2.2 参数化 CAD 复刻与 Assembly**。包含双腿、躯干、头颈、嘴、外壳、舵机/轴承/标准件参照以及全部机械接口；不再以 XL330 结构为目标。
- 前置：M11a 数字阶段；原官方 Onshape 当前账号无访问权限，改用 `fanhao375/microduck-replica-cad` 的 `SolidWorks-FT2026-10-20.zip`（77 个 SolidWorks 文件 + 13 个 STEP，发布 ID 418136980，sha256 `71e5b6a711ec9b4dd7154627e036a5180207088536aa230e42949212a550eaa1`）。该模型由机械行者Robo依据官方 STL 逆向重建，许可 CC BY-NC-SA 4.0；尺寸基准是这套第三方模型，不声称与官方原始 CAD 完全一致。
- Bite 1：导入并盘点飞特 v2.2 源装配，建立整机坐标系、关节布局、共享参数与零件清单；使用现成舵机、轴承和紧固件参照模型，不打印接口试片。验收是全套尺寸来源可追溯，改动件均为 `-FT` 版本，关节中心与第三方源装配对齐。
- Bite 2：复刻左右腿全部结构件，包括髋、大腿、小腿、踝、脚与连接件；完成局部装配，检查同轴度、运动范围和干涉。
  - 路线（2026-11-03 已验证）：逐件从源包单独导入 SLDPRT 到 Onshape，得到 `Part 1` 实体（特征历史不保留），再围绕实体重建参数化。首件 `01-upper_leg_rigidity_plate_上腿加固板-FT.SLDPRT` 已导入成功：`https://cad.onshape.com/documents/121f26cf7d320c49e6e67f8b/w/ec6a0d13fe7bc244cfa672b8/e/bb65185d2828ebe718e55d96`。
  - 腿部零件集：`vendor/microduck-ft-v2_2-legs/`，17 个文件共 94.9MB——结构件 13 个（hip_l、yaw2roll、rigidity_plate、left/right_upper_leg、leg、ankle_left、ankle_right、foot_left/right、sole_left/right）+ 轴承/紧固件参照（bearing_roll、轴承16×22×4、M2铜柱）+ 舵机参照（HD-1910-c001-dan / -shuang）。逐个导入的脚本 `tutor/import-legs.sh`，日志 `exercises/m11b-legs/import-log.jsonl`。
  - 待澄清：右踝有多个变体（`ankle_right脚踝右1-FT` / `1013ankle_right脚踝右1-FT` / `-改外面轴承` / `-改外面轴承1`），另左踝有 `ankle_l_v1_脚踝左V1` 备选。主用 `ankle_right脚踝右1-FT`（与 3MF 的 `ankle_right脚踝右.STEP` 名字对应），变体留 `variants/` 待与 3MF 网格量尺寸对照后定案。
  - 头颈的 `yaw_roll_motion` 不在本 Bite（归 Bite 3）。
- Bite 3：复刻躯干、骨架、电子件支撑、头颈、嘴及全部连接结构；保留轴承、舵盘、线缆出口和维修空间。
- Bite 4：复刻头/躯干等外壳与分件连接，处理壁厚、卡扣或螺钉、散热、开口和打印方向；所有结构件均有参数化源模型与可打印导出。
- 数字验收：整鸭零件覆盖完整；关键尺寸与原始 CAD 一致；各局部 Assembly 的关节和自由度正确；无未解释的几何干涉。接口试片可后送打印服务，实物测量仍单独记账。

### M11c · 整鸭装配与制造闭环（3 Bite）

- **目标**：把 M11b 的全部零件装成一只 1:1 MicroDuck，而不只是装配外壳。
- 前置：M11b 全部零件和局部装配通过数字验收。标准件与电子件直接采用可追溯模型；M10 跳过不妨碍机械装配，但整机上电与运动控制不在本里程碑验收内。
- Bite 1：建立整机 Assembly，完成双腿—躯干—头颈—嘴—外壳的装配约束与关节关系；检查默认姿态和整机关节运动范围。
- Bite 2：做全机干涉、拆装顺序、工具空间、线缆路径与重心/质量属性检查；生成爆炸图、BOM、装配说明和全套打印/交换文件。数字整机必须可复现装配。
- Bite 3：通过打印服务完成实物分件与整机装配，依据实测调整配合；验证整鸭机械装配、关节活动和外壳拆装。没有实物条件时此前数字 Bite 可交付，本 Bite 保持未验收。
- 验收：数字阶段要求 1:1 全机 Assembly、干涉检查、BOM 和复现说明；实物阶段要求全部零件可装配、目标关节可活动、正常拆装不破坏零件。站立/运动与音频功能不借机械装配结果宣称通过。

### 覆盖与边界

- 交付重点：参数化 CAD 源文件、导出网格、切片配置、尺寸/配合记录和装配说明；这些属于结构教程资产，不写进 `impl/`。
- 官方/第三方文件只作尺寸与结构参考，许可和来源逐项记录；能下载的标准件模型不重画。
- 未决项：打印服务、测量条件、实际舵机/电子件清单。它们影响后续尺寸，不用合理推断代替用户实测。
- 先修评估：用户是否有 CAD 与 3D 打印经验尚未记录；M11a Bite 1 默认采用项目内引导，学习者完成核心建模与测量动作。

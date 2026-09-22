# 用户提问与解答记录

## Q1（Phase 0 后）：为什么不用 ROS 而是自研通信框架？
- 仓库无任何 ROS 记录，答案由架构文档的相邻决策重建
- 核心四条：① "单一固定硬件配置"使 ROS 生态价值落空 ② 同一消息类型要跑 BLE/WebRTC，ROS/DDS 类型做不到 ③ 恢复路径依赖面必须最小，ROS 是最大依赖面 ④ Unix socket 文件权限 = 免费鉴权，DDS 默认网络可达恰是要消灭的失败模式
- 失效边界：接入 ROS 生态算法 / 进程数涨到几十 / 需要外部开发者工具链

## Q2（Phase 1 后）：硬件会开源吗？只学 release 能学到什么？
- 官方 Press Kit 明确：开源仅限软件栈，机械/电子设计文件不开源，且无未来承诺
- 旁证：前身 Open Duck Mini v2 完全开源硬件（35+ STL），想搓硬件走那条路
- 可学：daemon 架构、RL 训练栈（microduck_rl）、策略权重（HF Hub v5 共 8 个 ONNX）、MJCF 身体物理规格（在 reference/kinematics/assets 内）、浏览器仿真器、12 篇设计文档
- 结论：验收走 FakeIo/仿真路线，硬件闭源不影响任何里程碑

## Q3（Bite 1 后）：怎么理解 tokio interval 第一拍立即触发？
- `setInterval` 是"延迟重复"，tokio interval 是"周期时刻表，第一个时刻是现在"
- M0 无害（订阅者立刻有反馈）；M4 控制循环里第一拍在 t=0 用未就绪传感器数据推理 = 舵机抽一下
- 修法：`interval_at(now + period, period)`，已在 M1 control.rs 应用

## Q4（追问）："节奏相位永久错位一拍"没理解
- 澄清：稳态节奏两种语义完全相同，唯一差别是 tokio 在 t=0 多执行一次循环体
- "错位"仅指 tick 计数→时间换算恒差一拍（记账问题），真风险是第一拍输入垃圾
- 附原版证据：MissedTickBehavior::Delay 实测只跑到 43.1Hz（reference/robotd/src/main.rs:1794 注释）

## Q5（Bite 2 后）：流程偏好更正
- TS/Node 替代 Python 做类比映射（已落 user-prefs.md）
- 里程碑代码交 subagent 构建（已落 SKILL.md Phase 3）
- 要求固定 workspace + 过程落盘可续传（本次 SKILL.md 工作区约定）

## Q6（Bite 2 后）：插值是什么？
- 插值 = tween/补间：从当前姿态到 home 不瞬移，100 拍每拍推进 1%，15 关节并行 lerp
- 前端映射：CSS transition / GSAP 的逐帧中间值计算；M1 用线性路径（lerp），缓动曲线暂不需要
- 验收锚点：第 50 拍位置 = 起点与 home 的中点（1e-6）

## Q7（Bite 2 后）：IMU 数据结构困惑（不懂 IMU）
- 已插背景卡片：IMU 回答"转多快"(gyro) + "重力相对我指向哪"(gravity)
- 映射：DeviceMotionEvent —— rotationRate→gyro，accelerationIncludingGravity 归一化→gravity，四元数 quat 是融合后完整姿态
- 关键概念：躯干坐标系（重力在机器人自己视角的方向，直立=[0,0,-1]）
- M2 只用 gyro+gravity（观测前 6 维），quat 留给 M5 跌倒检测

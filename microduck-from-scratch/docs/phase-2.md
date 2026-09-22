# Phase 2 · 自底向上实现路径（归档）

## 直接下载不重写
- 8 个 ONNX 策略权重：HF Hub `pollen-robotics/microduck-policies` @ v5（velstand/sitstand/ground_pick/ball_kick×2/roller/roller_crouch/roulade）
- MJCF 身体模型（reference/kinematics/assets/alpha/robot_walk.xml）
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

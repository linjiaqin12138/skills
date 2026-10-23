# Phase 0 · 侦察（归档）

- **一句话本质**：高层意图（走多快/看哪/坐下）+ 传感器（IMU/15 舵机）→ 50 Hz 舵机目标角，由 ONNX 强化学习策略实时计算。25cm/800g 双足，RK3566 单板
- **不是模型项目，是机器人操作系统**：训练在 microduck_rl（MuJoCo+PPO），本仓库是板上 7 daemon + 签名 OTA 更新系统
- **技术栈**：纯 Rust workspace（24 crate），JSON-RPC 2.0/NDJSON over Unix socket，ONNX Runtime dlopen（≥1.23），权重在 HF Hub（v5，8 个文件），systemd 监管
- **核心主循环**：`reference/robotd/src/main.rs:1786`（tokio interval 50Hz）；tick 决策在 `robotd/src/control.rs`；策略在 `duck-control/src/policy.rs`；观测在 `duck-control/src/obs.rs`
- **目录地图/依赖详情**：见对话归档；关键 crate：robotd（核心）、duck-control（纯计算内核库，无 tokio）、duck-ipc-proto（协议契约，6247 行单文件）、configd/updater/btd（救援三件套）

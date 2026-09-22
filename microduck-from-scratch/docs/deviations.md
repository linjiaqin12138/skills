# 偏差登记簿

规则：每条偏差是**待收敛项**，不是永久豁免。收敛点到达时核销；M8 收敛验收时残余偏差逐条裁决（对齐回去 / 保留并写明理由）。

| # | 偏差 | 引入 | 预定收敛点 | 状态 |
|---|---|---|---|---|
| D1 | 只实现 robot.* 极小子集（hello/health/state），非完整 API 面 | M0 | M8 对照 reference/duck-ipc-proto 逐项核对 | 待收敛 |
| D2 | FakeIo 无真实 Dynamixel 串口协议 | M1 | 豁免——硬件层闭源，"能下载的不重写"原则覆盖 | 已豁免 |
| D3 | 单策略、无调度器 | M3 计划 | M6 | 未引入 |
| D4 | 自定义仿真 TCP/JSON 协议，非原版 sim.rs 协议格式 | M4 计划 | M8 对齐 PROTOCOL 握手+帧格式 | 未引入 |
| D5 | 安全层砍到 3 条规则，非完整 Safety 配置面 | M5 计划 | M8 对照 safety.rs 补齐 | 未引入 |
| D6 | 无 systemd 部署、updaterd 无 minisign 验签 | M7 计划 | M8 | 未引入 |
| D7 | 单 crate 双 bin，非 workspace 多 crate | M0 | M8 拆协议 crate，对齐"协议 crate 只许 serde/serde_json/semver"依赖约束 | 待收敛 |
| D8 | 无 SO_PEERCRED uid/gid 校验 | M0 | 第一个 mutating 调用出现时（M5/M7） | 待收敛 |
| D9 | 启动即插值到 home；原版"进程启动绝不移动机器人"（held_pose） | M1 | M5 安全层落地时引入"启动不动"语义 | 待收敛 |
| D10 | 插值在控制循环内逐拍推进；原版 interpolate_to 是阻塞调用 | M1 | M8 裁决（逐拍推进与控制循环同构，可能保留） | 待收敛 |
| D11 | healthy 恒 true，无 deadline 检测 | M1 | M5 | 待收敛 |
| D12 | RobotIo 只有 read/write，无 set_gain/set_torque/reboot/slow_sensors | M1 | M5（跌倒卸力需要 gain） | 待收敛 |
| D13 | IoError 不分类（单一字符串错误） | M1 | M8 | 待收敛 |
| D14 | robot.state 推送 1 Hz 且只带 positions（无 velocities/imu） | M0/M1 | M2 补 imu 字段，M5 对齐 50 Hz | 待收敛 |
| D15 | API_VERSION=1 且握手差异只报告不拒绝（行为与原版一致，版本号起点不同） | M0 | M8 | 待收敛 |
| D16 | control 循环用 interval 默认 Burst；原版选 Skip 并有论证（Burst 会把积压电机命令连发叠上总线；Delay 实测掉到 43.1Hz，reference/robotd/src/main.rs:1787-1801） | M1 | M3（接入 ONNX 推理后单拍耗时上升，默认值变成真问题） | 待收敛 |

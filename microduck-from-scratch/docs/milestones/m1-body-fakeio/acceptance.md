# Bite 2 · M1：身体模型 + FakeIo + 50Hz 回 home（已通过验收）

构建：coder subagent 实现，主 agent 复跑验收确认。

## 交付
- `src/model.rs`：15 关节表、DEFAULT_POSITION（左右腿等大反号）、MOUTH_INDEX=9
- `src/io.rs`：Sensors（关节+IMU 同结构体，硬件上一次事务读出）、RobotIo trait（仅 read/write，其余 M5 再加）、FakeIo **完美跟踪**（不加惯性模型，那是过度设计）+ `failing_reads(n)` 故障注入
- `src/control.rs`：50Hz 控制任务。`interval_at(now+period)` 避开首拍坑；插值起点锚定**第一次 read 成功**；每 5s stderr 打 `tick_rate=50.0Hz p99_jitter_ms=0.00`
- `src/main.rs`：接线；health 暴露 reads/writes/skipped_reads；`MINIDUCK_FAKE_FAILING_READS` 环境变量故障注入

## 验收实测（主 agent 复跑）
- 6 单测全过（镜像断言/表长/完美跟踪/failing_reads 恢复/插值端点/中点 1e-6）
- health @4s：reads=writes=tick=200 严丝合缝
- subscribe：positions 与 DEFAULT_POSITION 逐元素 |diff| = 0.0
- failing_reads=100：skipped_reads 恰好 100，恢复后收敛，daemon 不退出

## 坑
1. `tokio::time::Instant` ≠ `std::time::Instant`，interval_at 只收前者
2. 对"相邻拍间隔"取 P99 恒等于 20ms（废指标）；要对"与标称 20ms 的偏差"取 P99
3. 插值锚进程启动会让总线恢复时姿态跳变（真机=舵机猛抽）

## 偏差
D2（Fake 无串口协议，豁免）、D9（启动即动 vs 原版 held_pose）、D10（逐拍插值 vs 阻塞）、D11（healthy 恒 true）、D12（trait 瘦身）、D13（IoError 不分类）、D14（state 1Hz 无 imu）

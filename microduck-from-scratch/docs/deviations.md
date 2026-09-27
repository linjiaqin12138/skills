# 偏差登记簿

规则：每条偏差是**待收敛项**，不是永久豁免。收敛点到达时核销；M8 收敛验收时残余偏差逐条裁决（对齐回去 / 保留并写明理由）。

| # | 偏差 | 引入 | 预定收敛点 | 状态 |
|---|---|---|---|---|
| D1 | 只实现 robot.* 极小子集（hello/health/state），非完整 API 面 | M0 | M8 对照 reference/duck-ipc-proto 逐项核对 | 待收敛 |
| D2 | FakeIo 无真实 Dynamixel 串口协议 | M1 | 豁免——硬件层闭源，"能下载的不重写"原则覆盖 | 已豁免 |
| D3 | 单策略、无调度器 | M3 | M6 | 待收敛 |
| D4 | 自定义仿真 TCP/JSON 协议（op 标签帧形状对齐原版 sim.rs；帧细节自定，read 响应多带 body_pos/sim_time 供验收） | M4 | M8 对齐 PROTOCOL 握手+帧格式 | 待收敛（M4 已引入） |
| D5 | 安全层砍到 3 条规则，非完整 Safety 配置面 | M5 计划 | M8 对照 safety.rs 补齐 | 未引入 |
| D6 | 无 systemd 部署、updaterd 无 minisign 验签 | M7 计划 | M8 | 未引入 |
| D7 | 单 crate 双 bin，非 workspace 多 crate | M0 | M8 拆协议 crate，对齐"协议 crate 只许 serde/serde_json/semver"依赖约束 | 待收敛 |
| D8 | 无 SO_PEERCRED uid/gid 校验 | M0 | 第一个 mutating 调用出现时（M5/M7） | 待收敛 |
| D9 | 启动即插值到 home；原版"进程启动绝不移动机器人"（held_pose） | M1 | M5 安全层落地时引入"启动不动"语义 | 待收敛 |
| D10 | 插值在控制循环内逐拍推进；原版 interpolate_to 是阻塞调用 | M1 | M8 裁决（逐拍推进与控制循环同构，可能保留） | 待收敛 |
| D11 | healthy 恒 true，无 deadline 检测 | M1 | M5 | 待收敛 |
| D12 | RobotIo 只有 read/write，无 set_gain/set_torque/reboot/slow_sensors | M1 | M5（跌倒卸力需要 gain） | 待收敛 |
| D13 | IoError 不分类（单一字符串错误） | M1 | M8 | 待收敛 |
| D14 | robot.state 仍 1 Hz。M2 已补 imu 与 obs；velocities 不单列（在 obs 的 20..34）。50 Hz 未对齐 | M0/M1 | M5 对齐 50 Hz | 待收敛 |
| D15 | API_VERSION=1 且握手差异只报告不拒绝（行为与原版一致，版本号起点不同） | M0 | M8 | 待收敛 |
| D16 | 控制循环用 MissedTickBehavior::Skip（对齐原版；原版实测 Delay→43.1Hz） | M1 引入默认 Burst / M3 改为 Skip | M3 | 已收敛 |
| D17 | last_action 为真实上一拍策略输出（M2 曾恒 0） | M2 引入 / M3 写入 | M3 | 已收敛 |
| D18 | command 恒为默认零，速度/头/机身命令还没入口 | M2 | M4（robot.drive {vx,vyaw} 写共享 Command，观测 command 块取真值） | 已收敛 |
| D19 | 策略 load 失败直接退出进程；原版可抱住姿态并报 unhealthy | M3 | M5（与 D11） | 待收敛 |
| D20 | policy.rs 含 LSTM 状态分支，但 velstand 为 1-in/1-out，路径当前不执行 | M3 | 首次加载 recurrent 文件时核销；若 M8 仍无则删死代码 | 待收敛 |
| D21 | 仿真执行器为 XML 位置伺服而非训练用 BAM 模型（BAM 在 microduck_rl 的 Python/mjlab 侧）。几何/惯量/阻尼已于 M4 换血时收敛：robot_walk.xml 与 43 个 mesh 换成 microduck_rl 官方训练资产（真实惯量、逐关节实测 damping/armature/frictionloss、足部碰撞 mesh、Apache-2.0，见 sim/assets/PROVENANCE.md）。残余：执行器 PD（kp=8 kv=0.25 实测调出，原 XML no-bam 增益 kp=0.55 静态保持需 0.45rad 下垂、站不住；训练真实值为 BAM kp_fw=200）。实测 vx=0.15 走 0.897m/10s | M4 | M8 裁决（或届时把 BAM 移植进 duck_body.py） | 待收敛 |
| D22 | quat 解析用 resize(4,0) 静默补零/截断，与 positions/gyro 的严格长度校验不一致（io.rs:254）。quat 当前无人读（Q7：留给 M5），故暂可容忍 | M4（code review 发现） | M5 跌倒检测启用 quat 时，换严格 copy4（长度≠4 即 Err） | 待收敛 |

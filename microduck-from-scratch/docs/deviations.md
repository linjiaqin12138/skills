# 偏差登记簿

规则：每条偏差是**待收敛项**，不是永久豁免。收敛点到达时核销；M8 收敛验收时残余偏差逐条裁决（对齐回去 / 保留并写明理由）。

| # | 偏差 | 引入 | 预定收敛点 | 状态 |
|---|---|---|---|---|
| D1 | 只实现 robot.* 极小子集（hello/health/state），非完整 API 面 | M0 | M8 对照 reference/duck-ipc-proto 逐项核对 | 待收敛 |
| D2 | FakeIo 无真实 Dynamixel 串口协议 | M1 | 豁免——硬件层闭源，"能下载的不重写"原则覆盖 | 已豁免 |
| D3 | ~~单策略、无调度器~~ M6 已收敛：scheduler.rs 优先级链（技能 > ground_pick > sit/rise > walk），command 重编码、拍数窗口、换网 LSTM reset 全部对齐原版 | M3 | M6 | 已收敛（M6） |
| D4 | 自定义仿真 TCP/JSON 协议（op 标签帧形状对齐原版 sim.rs；帧细节自定，read 响应多带 body_pos/sim_time 供验收） | M4 | M8 对齐 PROTOCOL 握手+帧格式 | 待收敛（M4 已引入） |
| D5 | 安全层核心规则已对齐（NaN 拒绝/±π 夹紧/deadman/跌倒判定/gain 缓存），残余：无温度监控、无配置文件面（SafetyConfig 硬编码 default） | M5 引入 | M8 对照 safety.rs 与 robotd.toml 补齐 | 部分收敛 |
| D6 | ~~无 systemd 部署、updaterd 无 minisign 验签~~ M7 落地后拆成两半移交：无 systemd 由 D37 承接、无 minisign 验签由 D38 承接 | M7 计划 | M8 随 D37/D38 裁决 | 已移交（M7，见 D37/D38） |
| D7 | 单 crate 双 bin，非 workspace 多 crate | M0 | M8 拆协议 crate，对齐"协议 crate 只许 serde/serde_json/semver"依赖约束 | 待收敛 |
| D8 | 无 SO_PEERCRED uid/gid 校验 | M0 | ~~第一个 mutating 调用出现时~~ M5：侦察发现原版 robotd 无 SO_PEERCRED，鉴权=socket 0660 文件权限（SO_PEERCRED 是 configd/updaterd 模式，M7 实现）。M5 已 chmod 0660 | 已收敛（M5，按文件权限模型对齐） |
| D9 | ~~启动即插值到 home；原版"进程启动绝不移动机器人"（held_pose）~~ M5 已收敛：Held 阶段抱住启动姿态，绝不调 set_torque，显式 robot.enable 才斜坡回 home | M1 | M5 | 已收敛（M5） |
| D10 | 插值在控制循环内逐拍推进；原版 interpolate_to 是阻塞调用 | M1 | M8 裁决（逐拍推进与控制循环同构，可能保留） | 待收敛 |
| D11 | ~~healthy 恒 true，无 deadline 检测~~ M5 已收敛：stall 500ms(25拍)/频率地板 45Hz/连续读错误 10/策略缺失，阈值来自原版 robotd-params | M1 | M5 | 已收敛（M5） |
| D12 | RobotIo 只有 read/write，无 set_gain/set_torque/reboot/slow_sensors | M1 | M5 部分收敛：set_gain/set_torque/imu_ready 已加（跌倒卸力需要 gain）；残余 reboot/slow_sensors/imu_stale/measures_velocity/measures_load → M8 | 部分收敛（M5） |
| D13 | IoError 不分类（单一字符串错误） | M1 | M8 | 待收敛 |
| D14 | ~~robot.state 仍 1 Hz~~ M5 已收敛：推送改 frame_rx.changed() 驱动（50Hz 每拍一帧），载荷增 fallen/enabled/gain/torque；逐订阅者降频与 Lagged 语义见 D25 | M0/M1 | M5 | 已收敛（M5） |
| D15 | API_VERSION=1 且握手差异只报告不拒绝（行为与原版一致，版本号起点不同） | M0 | M8 | 待收敛 |
| D16 | 控制循环用 MissedTickBehavior::Skip（对齐原版；原版实测 Delay→43.1Hz） | M1 引入默认 Burst / M3 改为 Skip | M3 | 已收敛 |
| D17 | last_action 为真实上一拍策略输出（M2 曾恒 0） | M2 引入 / M3 写入 | M3 | 已收敛 |
| D18 | command 恒为默认零，速度/头/机身命令还没入口 | M2 | M4（robot.drive {vx,vyaw} 写共享 Command，观测 command 块取真值） | 已收敛 |
| D19 | ~~策略 load 失败直接退出进程~~ M5 已收敛：policy=None 传入控制循环，永远 Held 抱持，health 报 unhealthy "policy unavailable: …"（原版理由：Restart=always 下退出=crashloop，活着报病让 updater 回滚） | M3 | M5 | 已收敛（M5） |
| D20 | policy.rs 含 LSTM 状态分支，但 velstand 为 1-in/1-out，路径当前不执行。M6 实测：新下的 4 个策略（alpha_sitstand/alpha_ground_pick/roulade/ball_kick_left）同样全部 1-in/1-out feedforward（onnx 探针确认），LSTM 分支仍无运行时覆盖；换网 reset 调用本身已被单测断言（reset_calls 计数） | M3 | 首次加载 recurrent 文件时核销；若 M8 仍无则删死代码 | 待收敛 |
| D21 | 仿真执行器为 XML 位置伺服而非训练用 BAM 模型（BAM 在 microduck_rl 的 Python/mjlab 侧）。几何/惯量/阻尼已于 M4 换血时收敛：robot_walk.xml 与 43 个 mesh 换成 microduck_rl 官方训练资产（真实惯量、逐关节实测 damping/armature/frictionloss、足部碰撞 mesh、Apache-2.0，见 sim/assets/PROVENANCE.md）。残余：执行器 PD（kp=8 kv=0.25 实测调出，原 XML no-bam 增益 kp=0.55 静态保持需 0.45rad 下垂、站不住；训练真实值为 BAM kp_fw=200）。实测 vx=0.15 走 0.897m/10s。M6 新证据：sit/roulade 物理失败（D34/D35）执行器强度实验（kp=40 + forcerange 放宽）倒在同一时刻，指向 BAM 动力学缺失而非 torque 上限 | M4 | M8 裁决（或届时把 BAM 移植进 duck_body.py） | 待收敛 |
| D22 | ~~quat 解析用 resize(4,0) 静默补零/截断~~ M5 已收敛：严格长度校验（≠4 即 Err）；顺手把 copy3（gyro/gravity）长度不符会 panic 的隐患统一改为 Err | M4（code review 发现） | M5 | 已收敛（M5） |
| D23 | 跌倒响应为 fallen 判定直接触发 Limp（目标跟随实测位置+gain 50，直立后斜坡回 home 恢复），非原版 FallPredictor 陀螺外推预测 + Limp/Posing 三态机（still-rate 静止检测、max_ms 超时、pose_gain=160 均未实现）；原版 limp_fall 默认 OFF，我们默认 ON（velstand 零命令能站住，交接得回） | M5 | M8 裁决（或专设里程碑移植 fall.rs） | 待收敛（M5 已引入） |
| D24 | read 失败跳拍不滑行；原版 COAST_TICKS=3（≤3 拍用上一样本续跑，对策略不可见） | M5（设计时确认保留现状） | M8 | 待收敛（M5 已引入） |
| D25 | robot.state 用 watch latest-wins；原版 broadcast 256 帧缓冲 + 落后者 Lagged 丢帧 + 逐订阅者 hz 降频 | M5（50Hz 收敛 D14 时保留） | M8 | 待收敛（M5 已引入） |
| D26 | sim 推倒后躯干穿透地板（z=-0.111，接触求解器大冲击失真）；验收阈值因此是"z<0.08"而非躺平高度 | M5（验收实测） | M8 裁决（或调 push 幅度/timestep） | 待收敛（M5 已引入） |
| D27 | FakeIo 上策略闭环漂移 0.13~0.18 rad（速度恒 0 不在训练分布），enable 验收阈值 0.4 是实测放宽 | M5（验收实测） | M8（或 FakeIo 长一阶惯性模型后收紧） | 待收敛（M5 已引入） |
| D28 | 无 Mode(Walk/Roller) 与 roller/roller_crouch 槽（v5 权重仓库里有这两个文件，未下载未接线） | M6 | M8 | 待收敛（M6 已引入） |
| D29 | 技能抢占简化：技能/ground_pick 运行中一律拒绝新技能/ground_pick 请求；无 chain 窗口（roulade chain=true 未实现）/链式重放；无 unwind 阶段；ground_pick 固定跑满 end_phase=0.7（140 拍） | M6 | M8 | 待收敛（M6 已引入） |
| D30 | 无策略热换/carry_over（robot.set_policy / mode_switch 不实现；Controller::carry_over 的跨 swap 状态保留不做） | M6 | M8 | 待收敛（M6 已引入） |
| D31 | 嘴无特雷门/合唱覆写层（voice/chorale 不实现）；robot.mouth 仅 Driving 阶段生效（斜坡/抱持/Limp 期间嘴跟随该阶段自己的目标——这条与原版一致） | M6 | M8 | 待收敛（M6 已引入） |
| D32 | busy 门控简化：busy（技能/ground_pick/起身中）直接抑制 Driving→Limp 转移；原版门的是 FallPredictor（我们的跌倒响应本身是 D23 的简化版）。fallen 报告不变 | M6 | M8 | 待收敛（M6 已引入） |
| D33 | 无 stand 槽与 will_stand 幅值选网（原版 twist 幅值 ≤ standing_threshold 且有 stand 网络时选 Stand 槽，standing_gain_ratio=0.8）；我们的 velstand 挂 walk 槽同时覆盖站立（M3 起即是），站立增益恒 200 不打 0.8 折 | M6（对照原版级联时确认） | M8 | 待收敛（M6 已引入） |
| D34 | sit 物理失败：调度/重编码/锁存全部验证正确（skill=sit、twist=[1,0,0]、锁存保持），但下蹲到深位（膝 ~0.8rad、z~0.07）后向后翻倒（gz→+1，穿地板见 D26）。kp=40+forcerange±10 实验倒在同一时刻 → 非 torque 上限，指向 BAM 动力学缺失。sim 验收降级为调度断言 | M6（验收实测） | M8 随 D21 一起裁决 | 待收敛（M6 已引入） |
| D35 | roulade 物理失败：窗口计时精确（50 拍）、busy 门控正确（窗口内全部 fallen 帧 gain=200，帧数随运行微动 32~33），但前滚翻在 PD 执行器下滚不过去（~0.7s 处摔倒，结束后 Limp 接管）。sim 验收降级为调度+busy 断言。kick_left 与 ground_pick 物理通过 | M6（验收实测） | M8 随 D21 一起裁决 | 待收敛（M6 已引入） |
| D36 | robot.do 在 fallen/Limp 时 RPC 侧拒绝（accepted:false + 原因），原版此时照收、在控制循环里静默丢（"fallen 时是人类的选择"）。依据是原版自己的注释"accepted 然后什么都不发生是最坏的回答"——可观察行为差异，需裁决 | M6（决策卡片：robot.do 在哪一层拒绝） | M8 | 待收敛（M6 已引入） |
| D37 | 无 systemd：mini-updaterd 直接 spawn/kill miniduckd 子进程替代 `systemctl restart`（setsid 脱离会话，updaterd 死不杀 daemon；启动时按 /proc comm 收割上次留下的孤儿）；无 golden/boot-check 外层兜底 | M7 | M8 | 待收敛（M7 已引入） |
| D38 | 无 minisign 验签，release 完整性只有 sha256（manifest 登记 artifact 的哈希，apply 先验再解包）；D6 的上半句（无 systemd 部署）由 D37 承接 | M7 | M8 | 待收敛（M7 已引入） |
| D39 | updater 只有 LocalDir 单一 source（`<root>/source/`），无 GitHub/HF/网络发现/channel 策略/自动检查定时器；artifact 是未压缩 .tar（调系统 `tar -xf`）而非原版的 .tar.zst | M7 | M8 | 待收敛（M7 已引入） |
| D40 | SO_PEERCRED 缩为 uid==0 || socket owner 单点门控，无 allow_uids/allow_gids 配置（容器单用户，教学复刻里配置面没有教学价值） | M7 | M8 裁决或永久豁免 | 待收敛（M7 已引入） |
| D41 | updater 无 hooks/orphan 检查/transcript/Degraded 裁决（mini 健康判定是布尔）/subscribe 推送/self-update；Phase 枚举只取 proto 子集（Idle/Verifying/Extracting/Swapping/Applying/HealthGate/Committing/RollingBack） | M7 | M8 裁决 | 待收敛（M7 已引入） |
| D43 | ~~robot.drive {vx,vyaw} 请求式 vs 原版 robot.move {vx,vy,vyaw} 通知式：vy 侧向未开放，且连续意图（move/head/mouth）的通知语义（无应答、last-writer-wins）未实现~~ M8 Bite 1 已收敛 robot.move 本体：`Request.id`→`Option<u64>`（notification 帧形态）、MoveParams（default+deny_unknown_fields、单位/坐标系承重注释）、通知/请求双路径共用 apply_move_intent（无限幅只打时间戳）、vy 补齐、robot.drive 删除（METHOD_NOT_FOUND）。M8 Bite 2 已收敛残余①②：notification 统一入口（`src/main.rs:282-302`）——任何方法的无 id 帧都不应答，head/mouth 通知语义由此开放（抽取 parse_mouth/parse_head/apply_* 双路共用），非意图方法通知静默丢弃。残余：③ mini-duckctl move 仍请求式逐条调用（教学工具定位保留；原版由 padd 以通知式 20–50Hz 发送，daemon 两种形态都收）。联动：请求式 move 应答为 {accepted} 超集（多回显）、head/mouth 回显参数值缺 accepted 字段（原版统一回 IntentResult::accepted()），待后续 Bite 裁决 | M4（feature-inventory 追认编号） | M8 | 部分收敛（M8 Bite 1+Bite 2；仅剩残余③教学工具定位） |
| D64 | ~~disable 语义：旧 RampDown = 2s 斜坡回 home + 卸 torque（电源开关偷渡进策略开关）~~ M8 Bite 2 复核中发现（此前未登记），用户裁决立即对齐不挂账：Stopped 取代 RampDown——disable 当拍 policy reset + 直接命令回 home（舵机自己走，无斜坡）、torque 保持 on 上电抱持（原版 reference/robotd/src/main.rs:2786-2798；两对开关分工 duck-ipc-proto/src/lib.rs:542-551）。残余存疑：Scheduler::reset() 是否应连带清 Cascade 坐姿锁存，无原版逐行确证【合理推断：按"锁存不是 episode 记忆"不动】。连带代价：robot.relax 未实现前卸 torque 无出口（缺口待排期） | M8（复核发现） | M8 | 已收敛（M8 Bite 2；残余存疑登记在案） |

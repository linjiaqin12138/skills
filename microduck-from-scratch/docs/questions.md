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
- 映射：DeviceMotionEvent —— rotationRate→gyro，accelerationIncludingGravity 归一化→gravity
- 关键概念：字段都在躯干坐标系。停住但已经歪：gyro 仍是 `[0,0,0]`，gravity 变了。原地转：gyro 的 z 非 0，gravity 仍是 `[0,0,-1]`
- 坐标系以 `ImuData::default()` 的数字为准：直立 gravity = `[0,0,-1]`，按 x 前、y 左、z 上理解，重力在 −z、指向脚。源码注释「机体系 z 向下为正」与这个向量相反（那样直立应是 `[0,0,1]`）
- M2 只用 gyro+gravity（观测前 6 维），quat 留给 M5 跌倒检测。quat 本身见 Q9

## Q8（读 M1 教程示例时）：`mut` 是什么作用？
- Rust 默认绑定不可变。`mut` 表示这个名字之后还能用来改值，管的是绑定，不是类型本身
- 教程 `interval` 示例里：`timer` 要 `mut`，因为 `tick()` 是 `&mut self`；`ticks` 要 `mut`，因为 `ticks += 1`
- `period` 和 `start` 只被读，不加 `mut`

## Q9（读 ImuData.quat 时）：四元数怎么理解？
- quat 不是第四种传感器。一个姿态 = 绕一根单位轴转一个角度，打包成 wxyz：`w = cos(θ/2)`，`(x,y,z) = sin(θ/2) × 轴`
- 直立、没转：`[1,0,0,0]`。转到 180° 时 w = 0，xyz 变成转轴本身。转到 360° 得到 `[-1,0,0,0]`，和 `[1,0,0,0]` 是同一个姿态
- 反着读：角度 = `2·arccos(w)`，`(x,y,z)` 的方向是转轴。先不用管 i/j/k 乘法
- 绕上轴左转 180°：`[0,0,0,1]`，鼻子朝后。绕左轴（+y）前倾 θ：`[cos(θ/2), 0, sin(θ/2), 0]`

## Q10（读 control.rs:59-61）：插值为什么不从进程启动起算？
- 常见坑 3 的时间线。`RAMP_TICKS = 100`。`MINIDUCK_FAKE_FAILING_READS=100` 时，前 100 次 read 失败只做 `tick += 1` 然后 `continue`，`ramp_origin` 仍是 `None`，此时变量 `tick` 已经是 100
- 第一次成功才 `get_or_insert((tick, 当时的关节角))`。本拍进度 = `tick - origin_tick = 0`，写出躺平，再用接下来 100 拍走到 home
- 零点若钉在启动时的 0：同一拍进度 = 1，第一笔 write 已经是 home，躺平一拍跳到站立角
- 失败只有 40 拍时，错误零点第一次写出全程的 40%，还没到 home，但第一笔已经跳过前 40%
- 起点记下之后，再出现的读失败仍会把 `tick` 算进这 100 拍，只是那一拍不写总线
- 程序行为确认：第一次 read 成功后，用 2 秒从当时姿态线性插到 `DEFAULT_POSITION`，之后保持

## Q11（读 control.rs:73-78）：`if let Some(prev) = last_tick_at` 是什么语法？
- 这是模式匹配，不是把 `last_tick_at` 赋给 `Some`
- `last_tick_at: Option<Instant>`。值是 `Some(时刻)` 时，里面的 `Instant` 绑成 `prev`，进入花括号；值是 `None`（第一拍）时整段跳过
- 花括号里算的是本拍间隔相对 20 ms 的偏差绝对值，推进 `deltas_ms`，后面取 P99。健康时相邻间隔恒为 20 ms，间隔本身没有信息量

## Q12（读 control.rs:91-102）：`match` 和 `continue` 在做什么？
- `io.read()` 是 `Result`。`Ok(s) => s` 把 `Sensors` 拿出来赋给 `sensors`；`Err(_) => { ... continue }` 里 `_` 表示错误值不用
- `continue` 结束本轮，回到 `loop` 开头，下一轮先 `timer.tick().await`。它跳过本拍后面的发布、插值和 write，也跳过循环底部那次 `tick += 1`
- 失败分支里已经把 `tick` 加过，拍号不漏。任务不退出

## Q13（读 `Ordering::Relaxed`）：原子加一，以及「可见顺序」是什么？
- `fetch_add(1, Ordering::Relaxed)` 把「读进寄存器、加一、写回」合成一次原子的读-改-写。两个线程同时 `+= 1` 可能都读到 5、都写回 6，丢的是一次完整更新，不是写出半个整数
- 「写到一半」只出现在一次逻辑值要分多次内存传输时（例如 32 位机器上写 64 位整数）。这里的 `AtomicU64` 在 64 位机器上一次写完
- `Relaxed` 只保证这个计数器自己的加减是原子的。它不承诺：别的线程看见计数变成 8 时，也能看见这次加法之前写入的其他普通变量
- 需要那层承诺时，写侧 `Release`、读侧 `Acquire` 配成一对。编译后就是两次写内存加一道屏障，另一个线程看不到程序计数器，只能看见自己读到的内存值
- 普通 `while ready { 读 payload }` 只是源码里的阅读顺序。没有 Ordering，编译器和 CPU 仍可把 `payload` 的写入排到 `ready = true` 之后；普通变量还可能被提前读进寄存器。这段计数器旁边没有另一份要一起发布的数据，所以 `Relaxed` 够用

## Q14（读 control.rs:106-111）：`get_or_insert`、解构、`is_ok` 是什么？
- `ramp_origin: Option<(u64, [f64; NUM_JOINTS])>`。`get_or_insert`：已有值就返回它的可变引用；还是 `None` 就把 `(tick, sensors.positions)` 插进去
- 前面的 `*` 把元组从引用里复制出来。`let (origin_tick, start_pose) = ...` 是解构
- `JointTargets { positions: target }` 是结构体字面量；`&` 把临时值的引用交给 `write`。`is_ok()` 为真才 `fetch_add`

## Q19：action_scale 为什么让关节目标「差一点到达」？
- 不是舵机停在半路。目标本身就是 `home + scale × 策略输出`，舵机被要求走到这个已经缩小的角
- 行走默认 0.9、roller 0.8，站立是 1.0。站立的注释写明该策略按完整幅度训练。0.9 只标明是原型 alpha 的默认值，本仓库没有单独的测量说明为什么不是 1
- 和 low-pass 不同：scale 当场缩小相对 home 的偏移；low-pass 才是目标沿时间拖后，并且注释要求它必须和训练一致
- 0.9 在本仓库里没有求出来。`robotd-params` 的注释只写「原型 alpha 的默认值」。能对上公式的数（61 维、低通 0.5）旁边都有理由；这个数没有

## Q18（scatter_action）：动作为什么能和关节一一对应？z 下降不该是多个电机一起动吗？
- `scatter_action` 只做下标翻译：14 个数按 `joint_of` 放进 15 个关节槽，跳过嘴。它不决定哪个电机出多少力
- `body.z` 是观测里的命令（网络的输入），不是动作。网络每一拍同时吐出 14 个关节位置偏移；下蹲是这 14 个数的组合，在网络内部学出来的
- 关节目标 = home + action_scale × 偏移（行走默认 scale 0.9）。舵机是位置控制，协调发生在「选哪一组角度」，不是在 scatter 里把一个 z 分摊成力矩
- 放弃的是任务空间控制器（收到 z 后用几何求各关节角）。赌的是训练分布里这组角度真的会把身体放低。失效边界：换身体几何或 home 之后，同样的 14 个数不再对应原来的下蹲高度

## Q17（嘴不进策略）：嘴的控制由谁负责？
- 策略的 `scatter_action` 跳过嘴。`robotd` 控制循环在策略写完 14 个关节之后，单独写 `targets[MOUTH_INDEX]`
- 开合是 0..1 的比例，再线性映到 −5°..+30°（`mouth_target`）。超出范围夹紧
- 同一拍里后写的覆盖先写的：特雷门（手离嘴的距离）先写；合唱若正在进行会盖掉它，元音开合还做了约 90 ms 的跟随，避免 20 ms 一拍把舵机从「啊」抽到「嗯」；两者都没占用、且机器人处于 driving 时，才用客户端 `robot.mouth` 意图
- 嘴意图没有超时戳。客户端死掉会把嘴留在最后一次的开合上，这是原版故意跟原型对齐的行为

## Q16（读 obs 命令块）：`command.twist/head/body` 的数字代表什么？
- 字段是躯干系物理量：x 前、y 左、z 上。`twist` = 前进 m/s、向左 m/s、左转 rad/s。`head` = neck_pitch、head_pitch、head_yaw、head_roll，单位弧度，是关节目标不是注视点。`body.z` = 身高偏移（米，负值下蹲），`pitch` 正值前倾，`roll` 正值左侧抬起（向右倒）。零是名义站姿
- 示意动画：`docs/concepts/assets/command-motion/index.html`（俯视积分速度，侧面/正面画姿态）。俯视图曾把世界 x 的屏幕符号取反，正 vy 会往画面右侧走；已改成画面左 = 机器人的左，和正 vyaw 的逆时针一致
- 训练范围很小：z 约 −0.025..+0.010 m，roll/pitch ±0.26 rad。超出这个范围策略没见过
- 单测里的 0.1、0.2、…、1.0 是互不相同的记号，用来钉下标，不是一条真的运动指令。M2 控制循环传的是 `Command::default()`，全 0
- 同一组 twist 槽在别的技能里会被改义（坐下时 vx=1 表示坐，不是 1 m/s）。那是 M6 的事，行走/站立策略读到的仍是速度

## Q15（读 main.rs:47）：`stats.clone()` 和 `sensors_rx.clone()` 是复制了一份数据吗？
- 复制的是句柄，不是把计数器或传感器再存一份。每个连接拿到同一份数据的另一个使用权
- `stats` 是 `Arc<Stats>`：堆上一份 `Stats` 加引用计数。`clone` 复制指针并把计数加 1
- `sensors_rx` 是 `watch::Receiver`：`clone` 再做一个接收端，指向同一条只保留最新一帧的通道

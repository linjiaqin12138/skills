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

## Q20（读 policy.rs:105-130）：LSTM 是什么？这段 match 在分支什么？
- 卡在术语 LSTM，不是卡在 `match` 语法。诊断：缺背景，不是缺这段 Rust
- 纯函数：同样输入永远同样输出。LSTM 多两张便签 `h`（露出的）和 `c`（内部传送带），下一拍由调用方再喂回去，所以同样的观测可以出不同动作。三道门先不用管
- `inputs.len()` / `outputs.len()` 数的是图上的命名张量块，不是 obs 里的 61 个数。(1,1) 是 velstand：只塞 `obs`，`state = None`。(3,3) 才是 `obs/h_in/c_in → actions/h_out/c_out`。其他个数 load 失败
- 这条分支当前不执行（D20）。深读：`docs/concepts/lstm.md`。动画：https://nndl.ai/viz/lstm-gates/ 与 https://www.bilibili.com/video/BV1ih4y147YQ/

## Q21：只加载 velstand，为什么还要区分 (1,1) 和 (3,3)？
- velstand 实测是 1 入 1 出，只会走进 `(1, 1)`，`state = None`。`(3, 3)` 当前不会执行
- 这条分支是提前写上的（D20）。放弃的是「本里程碑只实现验收用得到的路径」。赌的是以后会加载一份 recurrent 文件，这段就能直接用。失效边界：到 M8 仍没有这种文件，就删掉
- 同一段里仍然要留的是宽度校验：`obs` 宽 61、动作宽 14。一份同样是 1 入 1 出的错误文件（旧的 51 维策略、下到一半的 onnx）会在 load 时被拒绝。那不是第二条网络，是同一条路径上的形状门

## Q22（读 policy.rs:158-178）：`map_err` 之后 `outputs` 会变成 error，还是直接 return？
- `session.run` 的结果是 `Result`。`map_err` 只改 `Err` 里的内容，整包仍是 `Result`：成功时载荷不变，失败时变成 `PolicyError::Inference`
- 真正离开函数的是后面的 `?`。`Err` 时 `infer` 立刻返回这个错误，`outputs` 不会被绑定。`Ok` 时 `?` 剥出成功值，`outputs` 才是会话的输出表，接着执行第 168 行
- 第 170 行的 `?` 同样：抽出张量失败就返回。第 171–175 行是手写的 `return Err`，不经过 `?`

## Q23（读 policy.rs:185）：`drop(outputs)` 是干什么？
- `outputs` 是 `SessionOutputs`，里面的张量切片借用着 `self.session`。数字已经在上一行 `copy_from_slice` 进了 `result`，以及（LSTM 路径）`copy_lstm_outputs` 进了 `state`
- `drop` 提前结束这次借用。下一行失败分支要 `self.reset_state()`，那是 `&mut self`。`outputs` 还活着时编译器不允许再可变借用 `self`
- velstand 的 `state` 是 `None`，第 180 行的 `if` 整段不进，这一行不执行。feed-forward 路径函数结束时 `outputs` 自己离开作用域

## Q24（读 control.rs:69-81）：apply_action 里的一阶低通在做什么？系数为什么是 0.5/0.7？
- 公式 `targets = α·新值 + (1−α)·上一拍目标`，头 α=0.5（更钝，脖子高频动作更刺眼）、腿 α=0.7；嘴和策略第一拍（previous=None）跳过
- 系数来自训练侧（0.5/0.7），推理必须一致——动作后处理是训练-推理闭环的一部分，改系数 = 改变策略输入分布。失效边界：换用对后处理不敏感的训练管线时可推翻
- 低通锚点是「上一拍目标」而非「实际关节位置」：滤的是命令信号本身，保证训练/推理一致
- 配套演示动画：docs/concepts/assets/lowpass/index.html（方波+毛刺信号，α 滑块可调；实测 α=0.5 阶跃 95% 收敛需 5 拍、α=0.7 需 3 拍 @50Hz）

## Q26（追问低通）：滤波是不是"把前一拍关节位置按比例混入下一拍目标"？
- 方向对，但混入的是「上一拍的滤波后目标值」（previous_targets，命令信号历史），不是关节实际位置——滤波与物理世界解耦，仿真/真机/单测可复现（control.rs:193-194, 294-320）
- 递推展开 = 指数逼近：第 n 拍达到 (1−(1−α)ⁿ)·X，永远差 (1−α)ⁿ；α=0.5 每拍砍一半差距
- 类比 lerp(current,target,α)，差异：前端的 current 是渲染位置（物理量），这里是上一拍命令（纯软件量）

## Q27：action 值和 target 值的单位含义？position 是什么？为什么要乘 action_scale？
- 三种数值空间：action[14] 无量纲策略单位（网络输出层直接吐出，嘴不参与故 14 维，scatter_action 铺回 15 维嘴填 0）→ ×0.9 → 弧度偏移 → +home → target[15] 弧度绝对角度；position（传感器读数）也是弧度（model.rs:35-42，home 注释明示）
- action_scale=0.9 的三重身份：① 训练契约（原版 Tuning::default 原型 alpha，control.rs:24，改它=策略失配）② 网络输出空间↔物理弧度的换算桥 ③ 软限幅（第一拍不低通时，它就是突变幅度天花板）
- 类比 input[range] 归一化 value × maxVolume；差异：系数是训练契约不是量程
- 考据事实：原版未说明为什么不是 1.0；standing 网络 scale=1.0（按完整幅度训练），velstand 挂 walk 槽用 0.9（见 m3 tutorial 决策卡片）

## Q28：action_scale 改成 1 重新训练就行吗？它是历史遗留吗？
- 原则上成立：scale 是可被网络权重吸收的任意约定，闭环一致性是唯一硬约束；改 0.9→1.0 重训后网络会自动把输出缩 0.9 倍，行为等价
- 但不完全等价：若输出层有界（tanh 饱和），scale 决定最大物理偏移/等效角速度天花板，也改变 reward 动作惩罚的相对权重——改 scale 重训出来的策略"性格"会变，reward 要重调
- 保留 0.9 的真正机制不是改不了而是不值得：改它收益为零、成本是一次完整重训。准确定性：训练超参数沉淀成的部署常量
- 佐证：standing 网络按 1.0 训练、velstand 按 0.9——scale 是"每份权重的训练属性"而非全局常量
- 失效边界：0.9 的幅度天花板限制新技能（如大步幅跑步满输出不够）时，改 scale+重训才是合理动作

## Q29：低通滤波是否有助于 sim-to-real 鲁棒性？
- 有帮助但机制要准确：① 滤波是纯软件、两侧逐比特一致，把部分"现实不确定性"换成"软件确定性"；平缓命令降低对执行器模型的保真度要求 ② 压掉高频分量，绕开仿真最不可信的区域（共振/齿隙/电流饱和）③ 滤波延迟在训练环境内，策略已学会补偿——危险的是训练时没有、部署才冒出的延迟
- 与域随机化互补：滤波是"绕开"（一行代码），域随机化是"硬扛"（训练成本翻倍）
- 两盆冷水：滤波与真实舵机自身滞后串联成双重低通（仿真需建舵机二阶模型）；对稳态误差（齿隙、重力静差）无能为力
- 失效边界：命令带宽需求上来（跑跳/快速恢复），被压掉的高频开始含有用控制信号，绕开策略失效

## Q30：高频是不是就是"信号瞬间变化很大"？
- 直觉对，收紧为"相邻两拍之间变化快"而非"值大"：幅度小但每拍来回抖也是高频，幅度大但慢慢爬是低频
- 阶跃（一次跳变）含所有高频成分，所以最能暴露滤波效果
- 傅立叶一句话版：信号=各频率正弦波叠加，低通=衰减高频分量再叠回
- 50Hz 采样的 Nyquist 上限 25Hz；策略毛刺顶在天花板，舵机只能跟几 Hz，低通截止（α=0.5 约 3~5Hz 量级）恰好卡在中间——物理直觉如此，实际系数仍是训练定的

## Q31：软件低通+舵机滞后叠加怎么补偿？双重滤波的动作失真策略能消化吗？
- 分三种情况：① 舵机滞后建模仿真（二阶模型+力矩上限+死区）→ 闭环一致环节，策略完全消化（观测回灌实际位置/速度，策略自学会打提前量），无"失真"可言 ② 未建模但滞后小 → 靠域随机化买的鲁棒性余量部分消化，典型症状过冲+轻振荡（纠正节奏按训练世界校准，世界变钝后纠正加码过度）③ 总滞后逼近控制带宽 → 相位裕度耗尽，振荡失稳，消化不了
- 补偿手段（推荐序）：仿真建执行器模型（测真机阶跃响应拟合）→ 随机化滞后参数 → 真机数据微调；经典前馈/史密斯预估器在 RL 栈不用——不如把世界建准
- 总结论：闭环里客观存在的东西，要么建模进仿真，要么用随机化覆盖它；事后算法补偿是下策

## Q32：为什么策略输入要包含 last_action？
- 核心：马尔可夫性。低通滤波器的 previous_targets 是不在观测里的隐藏状态，却决定下一拍舵机走向；不喂 action 历史 → POMDP，两个观测相同但滤波状态不同的局面只能输出同一动作
- last_action + 观测实际位置/速度 ≈ 反推滞后环节状态（命令与结果的差值=滞后的显影剂）
- 次要收益：满足 reward 的动作平滑惩罚（要知道上一拍才能"别突变"）；last_action 与位置的矛盾携带接触/卡住信息
- 只喂一拍是妥协：指数衰减下 n 拍前影响 (1−α)ⁿ（α=0.5 时 5 拍剩 3%），一拍拿到大部分信息；多喂观测维度 +14/拍
- 对照：LSTM 路径（policy.rs state/copy_lstm_outputs）用内部状态记历史；velstand 是前馈（state=None），无记忆，一拍回灌尤其关键
- 类比 React setState(prev=>...)：渲染不只依赖 props 还依赖内部 state

## Q33：观测能否把 last_action 换成 previous_targets？两个一起喂？一拍回灌够吗？
- 换 previous_targets：信息量更优（滤波器精确内部状态，一拍补齐全观测），但必须重训（obs 每维语义是训练契约；action 是零中心策略单位 vs targets 是 home 附近弧度绝对角），且丢失原始 action 这个 action-rate 平滑惩罚的锚点
- 两个一起喂：严格超集但高度冗余（互为可推），维度 61→75 不划算；原版喂 last_action 是跟随 Isaac Gym/MuJoCo 栈的 obs 惯例，非本项目特有
- 一拍回灌严格不够（滤波精确状态、舵机内部状态、接触状态都未覆盖）→ 近似马尔可夫；够用因为未观测状态衰减快/影响小 + RL 对观测噪声天然容忍
- 真不够的升级路径：LSTM（policy.rs 的 state 路径），内部状态记任意长历史；velstand 选前馈+一拍回灌是"任务难度下近似已够"的工程判断

## Q34：设计神经网络时怎么判断输入是否足够？
- 理论判据（马尔可夫测试）：obs+action 应近似决定下一拍 obs；存在"观测相同但最优动作不同"的局面即不够
- 设计期流程：枚举影响动力学的全部状态变量 → 逐个裁决三条出路：直接观测 / 从历史推断（如 last_action 近似滤波状态）/ 域随机化覆盖（观测不了就让策略对其值不敏感——"输入不够"可靠训练解决）
- 训练期信号：① 全能测试（知道全部 obs 的完美工程师能否决策）② privileged teacher 对照（仿真全知输入训 teacher 比性能，差距大=obs 瓶颈，legged locomotion 标准做法）③ 预测探针（预测下一拍 obs 的残差有系统性=有未观测动力学）④ 失败模式聚类（随机摔=策略问题，模式化摔=输入问题）⑤ 消融实验（终审，成本最高）
- 成本侧：每维都涨样本复杂度；最小集起步、证据驱动添加。microduck 命令块 4 个恒 0 维=输入设计迭代出的考古层

## Q35（追问）：单个 last_action 为何能推断滤波器状态？没历史也没 LSTM，滤波前的量还能推导？
- 精确说不能重建，也不需要：滤波输出已"显影"在观测位置里——舵机跟踪滤波目标，观测位置≈上一拍 target（差舵机滞后）。滤波状态大头直接从位置读，不靠 action 历史
- last_action 补的是歧义分解：相同位置、不同滤波惯性（A：10 拍零命令静止 vs B：刚拽一把大的、滤波里拖着反向残留尾巴，下一拍走向不同）——位置观测相同但 last_action 不同可区分
- 一拍够用的算术：递推权重 0.7·0.3ᵏ，3 拍前只剩 6%，大头被观测位置免费覆盖，last_action 补权重最大的最近一拍
- 网络不做这个代数：训练统计学到 (pos,vel,last_action) 与"下一拍世界走向"的相关映射；RL 只需足够相关性选对动作，不需精确重建隐藏状态——"近似马尔可夫"在工程上成立的真正含义

## Q36（推翻性追问）：没有 last_action，pos+vel 不能推出下一步 action 吗？是不是少个加速度？
- 直觉正确且戳中 filter-state 演示动画的软肋：简化链条（1 阶滤波 + 1 阶舵机）里 pos+vel 两个方程解两个未知数，滤波状态被完全锁定，last_action 冗余；演示中 now 时刻 A vel=0 vs B vel=+0.72，单看 vel 就能区分
- 一般规律：每多观测一阶导数，可多解开一个隐藏状态；加速度观测=再解一个二阶舵机内部状态
- 真实系统仍需 last_action：① 舵机二阶以上（≥3 未知数，pos+vel 解不完）② 传感器导数带噪（vel 差分量化噪声、加速度二次差分基本不可用），last_action 零噪声零成本零延迟 ③ RL 栈惯例
- 三条信息路径互补：位置/速度（显影但带噪，解 servo 阶数个状态）/ last_action（精确免费，补剩余状态）/ 高阶导数（理论可替代②，噪声不划算）

## Q37：一阶/二阶系统怎么理解？
- 阶 = 独立记忆变量个数 = 微分方程最高导数阶数
- 一阶：1 个状态（值本身），如低通滤波器/RC 电路；阶跃响应=指数逼近、永不超调（没有"速度"记忆）
- 二阶：2 个状态（位置+速度），如弹簧振子/真实舵机；阶跃响应可超调振荡（到位了速度没归零→冲过头回摆）；阻尼比决定性格，PID 的 D 项本质是调阻尼
- 与观测的对应：N 阶系统有 N 个隐藏状态，需 N 个独立观测（位置+各阶导数）才能完全锁定；玩具链条（滤波1+一阶舵机1=2）被 pos+vel 恰好解完，真实链条（滤波1+二阶舵机2=3）解不完→last_action 补位
- 直观检验：阶跃响应会超调振荡=至少二阶；只单调爬升=一阶或过阻尼。舵机 datasheet 阶跃响应曲线就是干这个的

## Q38（M4 换真模型后）：为什么静站时鸭子不停微调、不能完全静止？
- 第一层属正常：velstand 是 50Hz 闭环纠偏策略，"站立"=主动保持平衡，奖励的是速度跟踪不是关节不动；无记忆策略不存在"保持"概念
- 第二层是我们的问题：实测零命令 10s 漂移 x 11.6cm / y 14.2cm，偏大。原版证据 robotd-params/src/lib.rs:1579——velstand 设计上"stands still at zero command"
- 病因指向 D21 残余：XML PD（kp=8）≠ 训练用 BAM（kp_fw=200 + 减速箱动力学），策略的纠偏换算关系失配 → 纠偏过度、极限环微调+慢漂移；次要因素：我们喂无噪声观测，训练 gyro 带 0.005 噪声、姿态 0.001
- 修法排序：M8 移植 BAM → 按训练噪声水平给观测加噪 → 接受（验收不受影响）

## Q39（读 io.rs:248-254）：quat 为什么 resize？
- 直接原因：copy_from_slice 长度不等会 panic，JSON 解出的 Vec 长度是运行时值，resize(4,0.0) 先钉死长度
- 为什么只有 quat 这么宽容：quat 是死字段（M5 才用于跌倒检测，见 Q7），不值得为它判整个 read 失败；其余字段长度不符一律 Err
- 这是待办不是优点：与"坏帧即错误"哲学矛盾，M5 要换成严格 copy4，已登记 D22

## Q40：只训了站立的模型为什么能前进？
- 前提错了：velstand = velocity-conditioned 站/走一体策略，不是纯站立。观测含 command 块（twist vx/vy/vyaw），训练时随机采样速度指令、奖励速度跟踪；vx=0 → 站，vx>0 → 走，同一个函数不同输入
- 原版证据 robotd-params/src/lib.rs:1579："walks on a twist and stands still at zero command"，默认不加载独立站立网络
- 我们 M2 时 command 恒 0（D18）= 只调用 f(观测, 0)；M4 接 robot.drive = 把真值喂进去，能力解锁，能力一直在权重里
- command 块其余维度（head/body）恒 0 是输入设计迭代的考古层（回指 Q34），velstand 不用，M6 的技能会用

## Q41（读 control.rs:46-53）：为什么传感器用 watch、命令改用 Mutex？
- 方向相反：watch 是控制循环→RPC（单生产者帧流，多订阅者只看最新，覆盖是特性，读者零成本不卡控制循环）；command 是 RPC→控制循环（偶发写、每拍主动读，无订阅/通知需求）
- 功能上 watch 也能做（命令同样是 latest-wins），选 Mutex 是直白度判断：borrow/has_changed 语义对"每拍无脑读"多绕一层；std Mutex 无竞争 lock 纳秒级、临界区 15 字节无 await，对 20ms 拍预算影响为零
- 失效边界：命令变事件序列（动作脚本，不许覆盖）→ 换 mpsc 队列；读者变多且需变更通知 → watch 才赚回 API 成本

## Q42（读架构图后）：control.enable / enabled 的含义？
- 系统里没有 control.enable 方法；指的是 robot.enable（RPC，main.rs:211）和它写入的 ControlState.enabled（control.rs:90）
- 含义：「人类明确授权机器人现在可以动」的标志位，五阶段状态机的唯一输入。false（默认）→ Held 抱住启动姿态不调 set_torque；false→true 边沿 → torque on + 斜坡回 home → Driving；true→false → 斜坡回 home + torque off → Held
- 语义来源（M5 收敛 D9）：「进程启动不是移动机器人的理由」——被 supervisor 重启的 daemon 必须让站着的机器人继续站着；舵机 RAM 的 torque 跨进程存活使「不动」成为可行默认
- 与 deadman 分工：enabled 管授权（电平语义，防进程自作主张）；deadman 管司机在不在（保鲜期语义，防客户端失联）
- 失效边界：M6 多技能后「谁来 enable」变成调度问题，需重审

## Q43（读 servo-gain-torque 概念文档）：spring_demo 没看懂，gain=200 哪来的？theta/omega 太抽象
- gain=200/50：原型 alpha 调好后写进舵机增益寄存器的两个数（src/safety.rs:50-55），固件自定义标度、无 SI 单位；只有相对关系 200:50=4:1 有物理意义。到物理世界的桥是仿真体映射 kp_sim = 8 × gain/200（sim/duck_body.py:85-100），所以演示里 200→kp=8、50→kp=2
- 真机写入路径：write_position_p_gain + I/D 钉 0，RAM 寄存器上电恢复出厂（reference/duck-control/src/bus.rs:541-561）；寄存器量程 reference 未载，以 e-Manual 为准
- theta/omega 只是角度/角速度代号
- 处置：新增网页实体演示 docs/concepts/assets/servo-gain-torque/index.html（三臂同屏对照：站立-31°/软倒-63°/卸力-92°，滑块+torque 开关+预设），概念文档新增「gain=200 是哪来的？」一节，网页优先 python 降为数字对照；教程 m5 卡片已加链接

## Q44：IMU 动起来时 gyro/gravity/quat 怎么变化？
- 处置：新增动画演示 docs/concepts/assets/imu/index.html（三轴姿态动画+三组变量实时条形表+4 秒滚动波形+跌倒判定 200ms 去抖联动），四个预设：原地转圈（gyro.z 平台、gravity 全平）/前倾点头/侧翻 90°（gravity→[0,-1,0]，fallen 锁存全过程）/三轴乱晃；手动滑块模式
- 数值锚点（python 交叉对表验证过）：直立 gravity=[0,0,-1]/quat=[1,0,0,0]；前倾 θ quat=[cos θ/2,0,sin θ/2,0]、gravity.x=sin θ；侧躺（向右 90°）gravity=[0,-1,0]
- 核心直觉：gyro 只在"正在转"时非零（停住归零）；gravity 只在乎"歪不歪"（停着也在）；quat 是姿态打包（w≈1=没怎么转）
- 副产品修正：safety.rs 注释与 m5 教程原写"去抖，双向"，实际代码起身方向是立即清零（直立样本即解除 fallen）——已改为如实描述（倒下方向去抖、起身立即，不对称是故意：误判倒的代价远高于多躺一拍）
- 网页 gravity 公式曾取错矩阵行/列（前倾时 gravity.x 符号反），靠 python 对表抓出；教训：旋转公式必须数值验证，不能凭推

## Q45（读 main.rs:254-281）：是不是每次 send 都会触发 changed()？
- 是。watch 的 changed 是"版本号+1"不是"内容变了"——send 不做值比较，写入即给所有 receiver 打标记，值相同也触发
- 合并语义：receiver 没 poll 时连发 3 次只醒一次，borrow_and_update 读最新帧（latest-wins，慢订阅者自动丢帧不积压，D25）
- borrow_and_update vs borrow：前者读+清标记；用 borrow 不清标记会让 changed() 立刻再醒，忙等死循环（经典坑）
- send 的 Err 只在所有 receiver 都 drop 时发生；control.rs:416 的 `let _ =` = 没订阅者帧直接丢弃，不阻塞控制循环
- 推论：推送率=成功 read 的拍率，不是定时器——read 失败拍不发帧（杀 sim 时订阅端立刻断流）；`if subscribed` 守卫放开时攒着的标记让首帧即时送达

## Q46（读 main.rs:123-130）：tick_age_ms 与 saturating_sub 是什么？
- 语义：两个时间共用 started 起点（控制任务启动时刻），elapsed() 毫秒数 − 最后一拍的毫秒数 = "距上一拍过去多久"，喂给 stall 检测（>500ms 报病，25 拍阈值来自原版 robotd-params）
- saturating_sub：无符号减法的"减到 0 为止"版本。普通 `a - b` 减出负数 debug 下 panic、release 下回绕成天文数字；saturating 钳 0
- 这里是纯防御（不变量:last_tick ≤ elapsed 正常恒成立）：选饱和方向是因为回绕=误报"卡死"（健康机器人判病），钳 0=误报"健康"，两害相权取后者
- 附带效果：last_tick_millis 初始 0（一拍未走）时，elapsed−0=全部启动时长——"循环从没跑起来"在 500ms 后自然报 stalled，无需特殊分支

## Q47（读 duck_body.py:95-100）：actuator_gainprm/biasprm 是 MuJoCo 自带的吗？
- 是。model 是 mujoco.MjModel（XML 编译产物），这两个是每个执行器固定的参数数组（nu×10），槽位含义由执行器类型规定（MuJoCo XML reference 的 actuator 约定）
- <position> 执行器内置公式：力矩 = gainprm[0]×ctrl + biasprm[0] + biasprm[1]×qpos + biasprm[2]×qvel；填 kp/−kp/−kv 后化简为 kp×(目标−当前) − kv×角速度——与概念文档的舵机 P 环同一根弹簧
- [:, n] 是 numpy 切片，一把改 14 个执行器；运行时改合法（每个 mj_step 重读，不用重编译）
- 8.0/0.25 不是 MuJoCo 的，是 M4 实测调出的站稳增益（D21）；槽位结构才是 MuJoCo 的
- 改参数不改 ctrl 的原因：目标角全程不动，增益切换无跳变；缩 ctrl 则恢复瞬间目标跳变

## Q48：kp 和 kv 是什么？kp 是舵机扭力强度吗？
- 都不是角度。kp（P 项/位置增益）= 扭力与角度偏差的兑换率："每偏 1 弧度输出多少力矩"，单位力矩/弧度——是强度系数不是力矩本身（偏差为零时 kp 再大输出也是零），类弹簧硬度。鸭子的 set_gain(200) → 仿真 kp=8 走的就是这一路
- kv（D 项/速度增益）= 阻尼/刹车："转多快就反方向拦多狠"，单位力矩/(rad/s)。不看位置只看速度，防超调振荡
- 合起来 = PD 控制：力矩 = kp×角度偏差 − kv×角速度（duck_body.py:88-89 注释的公式，D21 里"XML PD"的出处）
- 实验锚点：网页演示卸力工况（kp=kv=0）杆子荡好几下才停 vs 站立臂几乎不超调——差的就是 kv
- 真机 XL330 也是 P/D 一对寄存器，但 bus.rs 里 D 钉在 0，真机阻尼靠舵机自身特性

## Q49：力矩 = kp×(目标−当前) − kv×角速度 是哪来的？
- 仿真层：MuJoCo 对 <position> 执行器的内置力矩约定（gainprm[0]×ctrl + biasprm[1]×qpos + biasprm[2]×qvel 代入化简）；我们的执行器来自 microduck_rl 训练资产的 --no-bam 降级路径（scene.xml 头注释）
- 真机层：XL330 固件的位置伺服控制环——编码器测偏差、按比例出电机电流，bus.rs:552 write_position_p_gain 写的就是这个环的 P 系数。真机上的公式不是物理定律，是固件代码：电机+编码器+算法假装自己是弹簧
- 物理层：弹簧（F=k×形变，管"回目标"）+ 阻尼器（F=c×速度，管"别冲头"）——位置+速度恰好是二阶系统的全部状态（Q37），PD 是能稳住一个关节的最简结构
- 这是简化模型：真舵机还有减速箱惯性/齿隙/电流饱和/温漂，训练侧的 BAM 模型（D21）才建这些；XML PD 是"够用"的降级，sim-to-real 差距靠域随机化补

## Q50（M6 后）：之前没有 LSTM，这次 M6 有了吗？哪些策略用到了 LSTM？
- 没有。M6 新下的 4 份 + velstand 共 5 份实测全部 feedforward（`obs[1,61] → actions[1,14]`，无 h_in/c_in）
- 容易误会的点：M6 做实的是 LSTM **契约**（换网即 `reset()`，scheduler.rs:428-438），不是遇到了 LSTM 权重。因全是前馈、reset 是空操作，故加 `reset_calls` 计数让契约可断言（policy.rs:77-81）
- D20 保持待收敛：官方哪天放出 recurrent 权重一丢就能跑；M8 仍没有则删分支

## Q51（追问）：官方发布的策略没有 LSTM，但官方源码有 LSTM 吗？
- 权重：v5 官方集**全部 10 份**都 probe 过（本次补测 alpha_stand/alpha_walking/ball_kick_right/roller/roller_crouch），清一色 feedforward，文件都 ~793KB 同构
- 源码：有完整 LSTM 支持——reference/duck-control/src/policy.rs:459-522（LstmState、h_in/c_in 按名接线、load 按张量数判别 :601）+ 专门契约文档 reference/docs/recurrent-policies.md（张量契约、记忆生命周期：换网/disable/跌倒恢复/链式重开全清零，同网内换命令保留；热换按 SHA-256 摘要决定去留；recurrent 权重须标 model_api:2）
- 定性：部署侧 LSTM 基建已通车、训练侧（mjlab/rsl_rl explicit-state 导出）也支持，但官方从未发布 recurrent 权重——是给未来留的接口，与观测里 4 个恒 0 的考古维度正好相反

## Q52（读 scheduler.rs:203-222）：往下坐的过程是 Rising 吗？
- 不是。`Sit` 三态：Up / Sitting / Rising{50拍}。**坐下没有过渡状态**——toggle 当拍直接锁存 Sitting，"往下坐"由 sitstand 网络在 vx=1 命令下自己完成，无窗口无倒计时
- Rising 只是**起身**：Sitting toggle → Rising，1.0s 窗口 twist 全零，到期回 Up
- 不对称的原因：起身是两网络间的脚本化交接（需要"何时交还 walk"）；坐下不交接给任何人（sit 网络原地驻留），没有交还时刻
- 派生行为：① 坐下过程 busy=false（坐着是停驻不是行进，原版同 control.rs:318-322），跌倒反射不抑制 ② label 当拍即 "sit"，描述锁存的命令状态而非物理完成度——M6 验收 sit 的 38 帧断言靠的就是这个

## Q53（读 scheduler.rs:559-567）：ground_pick 为什么 140 拍就交还 walk，不跑满 200 拍？
- 原版注释直接写明（reference/robotd/src/control.rs:85-86）："Ending at 1.0 replays the reach on the way out"
- 机制：拾取是**周期运动**（相位编码绕单位圆，φ=1.0≡0.0，轨迹可无限循环）。0→0.7 = 蹲下→伸手→抓→恢复直立（"捡一次"的全部有效动作）；0.7→1.0 是循环尾巴=在为下一次下蹲蓄力——跑满 200 拍会在收尾时又蹲一次
- 140 = 0.7×200 是"动作完成点"不是"周期跑完点"；数值链 DEFAULT_GROUND_PICK_END_PHASE（robotd-params:872 "70% of the cycle, as the prototype does"）
- 能硬切的前提：φ=0.7 时机器人已直立，velstand 接得住尾巴（与 sitstand_rise_s 同款逻辑 "velstand owns the tail of the rise fine"）

## Q54（追问 Q52）：坐下后马上再 toggle（只坐下去一点），起身还要固定 50 拍吗？
- 要。Rising 是**开环计时窗口**，不读身体实际姿态——scheduler 全链路没有"站直了没有"的判据
- 两层分开看：物理层自适应（sitstand 网络看得到关节位置，坐得浅纠正动作就小，0.2s 可能就回正）；调度层死板（剩余拍里 sitstand 当权、twist 强制全零、busy=true，期间 robot.drive 被吃掉）
- 原版同款：sitstand_rise_s=1.0s 固定窗口，按"足够"而非"精确"取（reference/robotd/src/control.rs:92-93）
- 【合理推断】为什么不做"直立检测提前交还"：姿态阈值+滞回多一类"阈值永不满足→卡死 Rising"的失败模式；开环窗口最坏代价只是多等几百毫秒+期间不理行走命令，代价有界行为可预测。原版的应对是把 sitstand_rise_s 做成配置项，而不是做成自适应

## Q55（追问 Q53）：ground_pick 为什么要用角度（相位）做输入，不能像 sit 一样一个 flag 吗？
- 分界：sit 是"去一个状态并待在那"（静态目标，反馈闭环自己收敛，类似温控器只需设定值）；ground_pick 是"沿时间轨迹走一遍"（脚本化动作，必须知道走到哪了）
- 核心论据（马尔可夫性，回指 Q34）：拾取轨迹在状态空间自相交——φ=0.25 下蹲中与 φ=0.6 起身中关节位置几乎相同，但正确动作相反；只给 flag 则前馈网络面对"同观测反动作"的死局。相位=给无记忆网络外接的时钟/剧本页码
- 为什么 cos+sin 两个数：回绕连续（0.99≈0.0，线性标量每圈一次阶跃）；单 sin 分不出 φ 与 π−φ；幅值恒 1 不出训练分布
- 编码分类（reference/robotd-params/src/lib.rs:1087-1094）：constant=kick/roulade 短促爆发；phase=ground_pick/roller_crouch 时间脚本；posture_flag=sit↔stand 双稳态驻留
- 【合理推断】kick/roulade 不用相位：够短、从固定初始条件 rollout，时间角色被初始状态+动作惯性吸收；原版未逐字解释

## Q56（追问 Q55）：观测里有关节角速度，为什么还区分不了"正在下蹲"和"正在起身"？
- 速度只能区分运动**方向**，区分不了剧本**进度**。反例=拾取的底部驻留段：(位置=最低点, 速度≈0) 持续几十拍不变，无记忆策略对同一观测只能输出同一动作——输出"保持"则永远蹲着，输出"起身"则抓取永远没机会执行。"停0.5秒再起"对无记忆网络数学上不存在
- 本质：位置/速度/加速度都是**身体的状态**，N 阶导数只能解身体的 N 个隐藏状态（回指 Q36/Q37）；相位不是身体的状态，是**剧本的状态**——活在控制者的表里，测量身体测不出"排练到第几小节"
- 次要：速度是差分算的，零速附近符号噪声最大（Q36），拿它当方向开关会抖
- 对照 sit 为什么无此问题：坐姿是驻留态，(坐姿,速度0)→"保持"是自洽不动点，不需要时钟；拾取底部驻留只是中途一站，同观测要在若干拍后换动作
- 【合理推断】原版未逐字论证 phase vs flag，以上从马尔可夫判据（Q34）反推；底部平台期在 M6 验收的 140 帧实测里直接可见

## Q57（读 m6 教程 robot.do 决策卡片）：「放弃的成本」写的不像成本，到底放弃了什么？
- 原文写的是实现注意事项，不是成本。正解：选了备选1（RPC 侧尽力拒绝）= 放弃备选2（全收下、控制循环统一丢）的两个好处——① 拒绝逻辑单点化（现在理由分住两层，新增拒绝情形要同时改 RPC 和调度器，漏一处就复现"accepted 但什么都不发生"）② RPC 层解耦（本来只做协议翻译，现在要向控制循环打听阶段，Stats.driving 就是为此加的）
- 原文后半句其实是备选1 自己没买全的短板：busy 只活在 Cascade，RPC 判不了，"accepted 然后什么都不发生"没被消灭，只是从 fallen 缩窄到 busy——这也该算成本
- 副产品：抓到"fallen 时拒绝 vs 原版静默丢"这条可观察行为差异未登记偏差，已补 D36；卡片成本项已按"放弃了备选2 的什么好处"重写

## Q58（读 control.rs:64）：Stats.driving 是什么？"相位/斜坡"这些词怎么理解？
- driving = 机器人是否处于"正常驾驶模式"的公开布尔标志；用途：RPC 层据此拒绝渐变回 home 途中的 robot.do（"accepted 然后什么都不发生是最坏的回答"）
- 五阶段（洗衣机类比：同一台电机，注水/洗涤/漂洗/脱水各阶段干的事完全不同）：Held 抱持（僵住）→ RampUp（2秒渐变到站立）→ Driving 驾驶中（神经网络开车，唯一响应技能请求的阶段）⇄ Limp 瘫软（摔倒卸力）→ RampDown（回站立后断电）
- 用词教训（用户两次纠正后定性）：① phase 在状态机语境译"相位"是**误译**——"相位"专指波形相位角，状态机该写"阶段/模式"；同项目里 ground_pick 的相位编码才是真相位，两个概念曾被同一个错词混着用，已全部订正 ② "斜坡(ramp)"= 目标沿直线慢慢爬的渐变过程（词源：目标-时间曲线像斜坡），不是机器人爬坡；首次出现必须声明 ③ 用户已熟的英文原词（SKILL/tutorial/viewer 等）直接用英文，翻译反而多一道脑内转换——三层规则已落 user-prefs「语言与用词」

## Q59：仿真里机器人摔倒了怎么办？
- 自动链路（M5）：fallen（躯干系 gravity.z>−0.5 持续 200ms）→ Limp（策略停摆、目标跟随实测位置、gain 200→50 卸力）→ 若自行直立则自动渐变回站立回 Driving。技能运行中被 busy 抑制（D32），窗口结束下一拍接管
- 躺平起不来（大多数情况）：velstand 没有"从躺平起立"技能（原版有 Posing 段，我们 D23 简化未做，M8 裁决），会一直躺在 Limp
- 恢复=重启 duck_body.py（物理重置到站立 keyframe）；daemon 不用动，SimIo 自动重连（accept-m5 B4b 验过），重连后自动回 Driving。noVNC 栈重跑 scripts/duck-vnc.sh
- viewer 的 Reset 按钮无效【合理推断】：passive 模式 run_physics_thread=False，mujoco.viewer 源码全文无一处 mj_resetData，按钮请求没人服务
- D26：推倒后可能穿透地板沉底，沉了只能重启 sim

## Q60（读 m7 tutorial.md:38-56）：arm/swap/spawn 是什么含义？
- 是 apply（把新版本装上并启用）拆成的四步里前三步的短名字，不是新概念。以 1.0.0→2.0.0 为例：
  - **arm**（上弦）：把纸条 pending.json 写盘（"试用 2.0.0，上一版 1.0.0，第 N 次启动"）。纸条是给崩溃恢复机制上弦：在=试用期出问题该回滚，不在=天下太平
  - **swap**（换指向）：rename 原子换 `install/current` 这个 symlink 指向新版。只改磁盘路牌，不影响在跑的进程
  - **spawn**（起新进程）：从 `current/bin/miniduckd` 拉起新版 daemon（setsid 脱离会话）
  - **confirm**（转正）：健康门通过→删纸条；不过→回滚
- 本节论点一句话：swap 是"改变现实"的时刻，arm 是"给改变留证"的时刻，记录必须先于改变——否则崩溃落在两者之间，现实变了没记录，恢复机制失明

## Q61（追问 Q60）：tutorial.md:44-54 那张表没看懂
- 表是 `examples/boot_counter_order.py` 的沙盘推演（不跑真机器人）：模拟四步执行，每步后假设 updaterd 立刻被 kill -9，再看重启后 recover() 凭磁盘状态会做什么决定
- 三个变量：current=symlink 指向谁（下次启动起谁）；live=此刻实际在跑谁；pending=纸条在不在。recover() 唯一依据是纸条：在→"有版本在试用期没结账"→重跑健康门裁决；不在→照常从 current 起
- 关键组（kill -9 落在 swap 之后）：arm 在前→纸条已写→重启发现未结账的试用→回滚得救；arm 在后→current 已指 2.0.0 而纸条还没写→recover 认为一切正常→把坏版当正式版起起来，永远没人撤回=变砖
- 图解不变量：纸条的在场时间必须完整罩住"现实被改变"的区间（swap→confirm），先留证、再改变、最后销证

## Q62（追问 Q60）：kill -9 kill 的是谁？updaterd 还是什么？
- 杀的是 updaterd 自己（mini-updaterd 进程），不是被更新的 miniduckd。设计前提：updaterd 必须假设自己会死在任何时候（断电/kill -9/OOM，tutorial.md:9），且死在 apply 任意一步都能收拾残局
- 两个进程死活分开：miniduckd 由 updaterd 用 setsid() 拉起、脱离会话（engine.rs:116-123），kill -9 updaterd 不带走 daemon——验收 E0 专门验这条。所以 updaterd 死后 current 指向、纸条在不在、哪个版本还在跑是三件独立的事
- 沙盘脚本里"kill -9 落在 X 之后"=循环 break 模拟（后面步骤永不执行）；真实验收靠场景 E 的故障注入（`MINIDUCK_UPDATER_EXIT_AFTER_SWAP=1` 让 updaterd 在最坏窗口 exit(1)，tutorial.md:92）和 accept-m7.sh 的真 kill -9

## Q63（追问 Q60）：这段是想说明 arm/swap/spawn 顺序的不可调换性吗？
- 大方向对，但要收紧：不是"四步全都不可调换"，真正要证的是唯一看起来能挪、实际挪不得的那一对——arm 必须先于 swap
- 分两类：① 数据依赖决定的顺序（显而易见）：swap→spawn（spawn 从 current 起进程，不先换指向起出来的还是旧版）、健康门→confirm（先销账再验收等于没验收）② 崩溃安全决定的顺序（隐蔽）：arm→swap——arm 与其他步无数据依赖，放后面单测和正常运行全看不出来，唯一差别只在"死在 swap 之后、arm 之前的缝里"才暴露
- 原版把这条列为三条根本规则之一（reference/updater/src/engine.rs:1-18 "The boot counter is armed before the swap"）：其他顺序错误要么立刻报错要么逻辑说不通，唯独这条是"不写错就不知道它存在"的纪律，只能靠规则立住

## Q60–Q63 处置（2026-09-30）
- 这四个疑问暴露了 m7 tutorial 原稿"arm、swap、spawn 新 daemon、confirm（删纸条）"一句信息密度过高（四个英文动词无就地展开、未说明 kill -9 的对象、未解释表头三个字段），已把解答吸收进章节：四步改为逐条展开的一句话列表；新增"kill -9 杀的是 updaterd 自己、daemon 因 setsid 存活"一段；表格前补三字段含义与 recover 只认纸条的规则；表后补"不是四步都不可调换，唯 arm→swap 是隐蔽纪律"的收紧段

## Q64（读 m7 tutorial 孤儿收割段）：/proc/*/comm 是个什么文件？
- /proc 是内核暴露进程信息的虚拟文件系统，不在磁盘上。每个在跑的进程一个目录 /proc/<pid>/，里面的 comm 文件只装进程名（上限 15 字符）
- 用途：updaterd 启动时遍历 /proc/ 数字目录、读 comm、名字以 miniduckd 开头就杀——直接读 /proc 就是 ps/pgrep 内部的数据源，省掉 fork 外部命令和解析文本
- 坑：僵尸进程的 comm 还在（容器 PID 1 是 sleep infinity 不收尸），不看 state 直接杀会对尸体反复发信号刷日志——所以代码再读 stat 跳过 Z（engine.rs:177-181）

## Q65（读 m7 tutorial supervisor 决策卡）：golden / 兜底 是什么？
- golden release = 原版 updater 的保底版本：一份已知好的旧版本，golden symlink 指向它，永不清理（updater-design.md:769 "never pruned"）；与回滚用的"上一版"是两份东西
- 它在救谁：pending/回滚机制都假设 updaterd 自己还能启动；golden 由 updaterd 之外的救援脚本用（开机时先于 updaterd 跑，readlink 就拿到保底版本，"recovery outside this process needs no parser"，:574/:823），链条 current→previous→golden→recovery mode（:1478）。目的：机器人永远能自己联网求救，不返厂（RMA/truck roll，:773）
- 我们砍掉了这层（D37）：updaterd 死一次能恢复（pending 管），程序本身坏了没有外援，"坏了就是坏了"，M8 裁决
- 同句词：journald=systemd 的日志托管；unit 依赖编排=systemd 声明启动顺序关系

## Q66（读 m7 tutorial）：解释性括号打断阅读主线
- 处置：立"主干/细节"分层规则并落 user-prefs「流程偏好」+ SKILL.md writer 约定——正文只留主线；一句话解释→句尾脚注；一段展开→章末细节小节；跨里程碑概念→docs/concepts；`文件:行号` 证据定位例外、必须留正文原地
- M7 章已按此改写：16 个解释性括号转脚注（章末统一定义），3 处重复展开改为指向"常见坑"的指针；旧章节不回改，M8 起生效

## Q67（读 m7 tutorial §CLI 分流）：这种事情有必要提吗？
- 没有。这段是"交付清单逻辑"不是"教学逻辑"：「update 子命令连 updaterd 的 socket」一句就够（模块分工和验收命令已能看出）；「解析完第一个参数就分流」是离题的 CLI 实现细节
- 处置：整节删除，压缩成一句并入 §3 模块分工末尾

## Q68（章节结构反馈）：教程应按"问题→背景概念→设计（图先行）→代码领读→坑与展望"写
- 已落 SKILL.md Phase 3 writer 约定：「设计」为新增硬标准节，先于代码领读，必须有图（模块关系用 arch.d2/.svg，流程/时序用 mermaid 嵌文），标准=不看代码能复述设计
- M7 章已由 writer subagent 按新结构重排：新增 §3 设计节（读图+模块划分总述+两张 mermaid：apply 七阶段/recover 决策树+五卡决策导览），原「实现走读」改名「代码领读」，常见坑末尾加展望段；事实与行号零改动

## Q69（读 engine.rs:175-181）：/proc/<pid>/stat 文件内容长什么样？
- 一行、五十多个空格分隔字段，例：`1 (systemd) S 0 1 1 0 -1 4194560 ...`。只需前三个字段：pid、(comm)（包在括号里的进程名）、state（单字母：R 跑/S 睡/D 不可中断睡/T 停/Z 僵尸）
- 为什么代码用 rsplit(')') 而不是按空格切：comm 本身可含空格甚至括号（如浏览器 "(Web Content)"），从右边找最后一个 ) 之后恒为 " state ppid ..."，trim().starts_with('Z') 即判僵尸
- 本场景：kill -9 updaterd 留下的 daemon 死后变僵尸，stat 里 state=Z，收割逻辑跳过它

## Q70（读 engine.rs:116-123）：什么是 setsid？
- setsid = 让子进程自立门户：成为全新会话的会长、全新进程组的组长，与父进程的信号广播绝缘（engine.rs:116-123）
- 前置概念：进程组（信号可按组发，Ctrl+C 的 SIGINT 就是发给前台整组）；会话（进程组的上一层，绑控制终端，终端关了全组收 SIGHUP）。fork+exec 默认继承父进程的组和会话
- 不设 setsid 的风险：supervisor/测试框架/容器清理习惯按组整杀，daemon 会被株连；kill 单个 PID 本身不株连子进程，真正的风险在"整组清扫"
- 对本章的意义：「updaterd 死了、机器人还在跑」（场景 E0）靠这条成立——updater 死了被更新系统必须活着，否则更新半途 updaterd 崩了 = 机器人跟着瘫

## Q71（读 engine.rs:137-149）：try_wait 是什么？
- try_wait = 非阻塞版"收尸"（waitpid WNOHANG）：问一句"孩子死了没"立刻拿答案走人。三种返回：Ok(Some)=已退出且尸体已收、Ok(None)=还活着、Err=调用本身出错。对照 wait() 是阻塞版，挂起等到子进程退出
- 为什么这里必须非阻塞：这段是"SIGTERM 宽限 2s 不走再 SIGKILL"（SIGTERM_GRACE）——阻塞 wait 会被无视 SIGTERM 的进程卡死，"给宽限期"语义本身要求带超时的等待 = 非阻塞轮询 + 自己掐表（每 100ms 问一次）
- SIGKILL 后的 child.wait() 不能省：SIGKILL 保证必死，但死掉的子进程必须由父进程 wait 一次才从进程表消失，否则变僵尸（回指 Q64/Q69）
- 100ms 轮询间隔是工程取值：太密费 CPU，太疏拖慢换版本

## Q72（读 engine.rs:144-146）：发了 SIGKILL 就一定会退出，所以用 wait 阻塞吗？
- 对。SIGKILL 不可捕获/忽略/阻塞，内核直接终止，此处的阻塞是有界的（最坏=内核进程清理那点时间），不像 SIGTERM 可能被装死挂住——这正是宽限期用 try_wait 轮询、此处可直接阻塞的分界
- wait() 在此身兼两职：等它死 + 收尸。SIGKILL 保证"会死"但不负责收尸，不 wait 就留僵尸（Q69/Q71）
- 严格边界：进程卡在不可中断睡眠（D 状态，如等 NFS I/O）时 SIGKILL 也不立刻生效，要等 I/O 返回——本地 eMMC 场景够不着，按"必死"处理合理

## Q73（流程反馈）：里程碑越来越不 bite + 偏离原版的口径
- 里程碑越来越肿的病根：Phase 2 用依赖树切分（优化构建顺序），没设学习单元上限；依赖树越靠上"能独立验收的最小单位"天然越大。证据：M7 一章 10 模块+5 决策卡片，追问 13 次，一大半是一次性信息量过大
- 处置（已落 SKILL.md）：① Phase 2 加大小上限（新概念≤2/领读模块≤5/决策卡片≤3，超出拆 Ma/Mb）② Phase 3 恢复 Bite 制硬性执行（每里程碑拆 2~4 个 Bite，一轮交互一个，coder/writer 禁止整里程碑交付，教程按 Bite 增量写）③ 不回改 M0–M7，M8 起生效
- 终态对齐口径修订（铁律 8/9 已改）：功能点必须与 reference 一致（reference 的 bug 除外）；模块切分/抽象/架构允许自主——学的是算法与机制，原版架构不一定优雅。功能缺失类偏离=待收敛项，架构类偏离可永久豁免写明理由即可

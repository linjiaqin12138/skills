# 第 2 章 · M1：身体模型 + FakeIo + 50 Hz 回 home

> 本章对应里程碑 M1（Bite 2，已验收，coder subagent 构建、主 agent 复跑确认）。交付物：`src/model.rs`（15 关节常量表）、`src/io.rs`（RobotIo trait + FakeIo）、`src/control.rs`（50 Hz 控制任务），以及 `src/main.rs` 的接线改动。

## 1. 动机

M0 交付的是"能说话的进程"：50 Hz 心跳只是个空转计数器。M1 把心跳变成真的控制循环——每拍执行 `read → compute → write`，让机器人从全零姿态（"躺平"）用 2 秒钟平滑地站到 home 姿态。

在依赖树里，M1 是后面一切的地基：

- **M2（61 维观测向量）** 的原材料就是这一章定义的 `Sensors` 结构体（关节位置/速度/电流 + IMU）。观测向量的前 6 维（gyro3 + gravity3）直接来自 `ImuData`。
- **M3（策略推理）** 的输出是 14 维关节目标，写下去的那条通道就是这一章定义的 `RobotIo::write`；策略不控制的嘴（第 15 维）为什么是例外，也由 `model.rs` 的关节表回答。
- **M5（安全层）** 的"读失败跳过本拍、不 panic 不退出"容错骨架，这一章已经立好——M5 只是往里面填规则。

M1 的验收答案只有一句话：**FakeIo 假总线上，4 秒时 reads=writes=tick=200 严丝合缝，订阅推下来的姿态与 home 逐元素零误差**。能做到这一点，说明控制循环的节拍、容错、插值、发布通道四件事全部成立。

## 2. 概念铺垫

### 【背景卡片】固定频率控制循环

- **一句话定义**：控制循环是机器人的心脏：以固定周期（这里 50 Hz，即每 20 ms 一拍）重复"读传感器 → 算本拍目标 → 写执行器"。它不是普通的事件循环——节拍本身就是正确性的一部分。
- **为什么需要**：强化学习策略是在 50 Hz 下训练出来的（Phase 1 已确认：50 Hz 与策略训练频率绑定），传感器数据年龄必须与训练分布一致——你喂给策略的"当前姿态"如果老了 40 ms，对策略来说就是一个它训练时没见过的世界。控制频率漂移 = 输入分布漂移 = 策略行为失真。
- **最小示例**（大白话：控制循环就是一台节拍器驱动的流水线——每 20ms 响一次，每响一次做一轮"读传感器 → 算目标 → 写执行器"，快慢可以实测出来）：

```rust
// examples/ex_interval.rs —— cargo run --example ex_interval 可跑（此处省略 use 与 #[tokio::main] 包装）
let period = Duration::from_millis(20); // 50 Hz
let start = Instant::now();
let mut timer = interval_at(start + period, period);
let mut ticks = 0u32;
while start.elapsed() < Duration::from_millis(500) {
    timer.tick().await; // 真实项目里：读传感器 → 算目标 → 写执行器
    ticks += 1;
}
println!("500ms 内跑了 {ticks} 拍，实测 {:.1} Hz",
    ticks as f64 / start.elapsed().as_secs_f64());
```

实际输出——半秒 25 拍，一拍不多一拍不少：

```text
500ms 内跑了 25 拍，实测 49.9 Hz
```

- **TS/Node 类比**：`setInterval(tick, 20)`——但这个类比的差异点（见下）大到几乎构成警告。
- **差异点**：① Node 的 `setInterval` 不保证节拍：事件循环被长任务堵住时回调延迟且不追拍；而 tokio interval 是"时刻表"语义，missed-tick 行为可选（Burst 追拍 / Delay 顺延 / Skip 丢拍）——原版实测过 `Delay` 会把 50 Hz 拖成 43.1 Hz 并选了 `Skip`（见第 1 章 Q4）。② 舵机不是"发出去就行"的 HTTP 请求：20 ms 内写完和 40 ms 内写完，在物理上是两个不同的运动轨迹。③ Node 没有实时保证，Rust + tokio 也没有硬实时保证，但抖动量级从"不可控的几十 ms"变成"可控的亚 ms"——本章验收里 `p99_jitter_ms=0.00` 就是这个。

### 【背景卡片】trait：接口 + 编译期静态分派

- **一句话定义**：`RobotIo`（`src/io.rs:63-66`）是一个 trait——Rust 的接口。`control::spawn(mut io: impl RobotIo + 'static)`（`src/control.rs:46`）表示"任何实现了 read/write 的类型都行"，编译器为每个具体类型单态化一份代码。
- **为什么需要**：真实舵机总线（串口 + Dynamixel 协议）和假总线（内存变量）必须共用一个接口，测试和笔记本开发全程跑 FakeIo，`cargo test` 不需要硬件（原版文档 §2.4 的同一条理由，`src/io.rs:3-6`）。硬件闭源，这一层抽象是整个复刻工程能在没有机器人的前提下推进的前提。
- **TS/Node 类比**：`interface RobotIo { read(): Sensors; write(t): void }`，然后控制循环依赖接口而非实现——依赖注入的老熟人。
- **差异点**：① TS 的 interface 编译后**消失**，运行时靠鸭子类型；Rust 的 `impl RobotIo` 是编译期静态分派——没有虚表、零运行时开销，类型不匹配直接编译失败。想要运行时分派（TS 那样的）得显式写 `dyn RobotIo`，本章不需要。② trait 方法在类型上"就地生效"，没有 class 继承体系；`Send` 约束（`src/io.rs:63`）是编译期承诺"这个实现可以安全跨线程移动"——这是 tokio 多线程运行时的门票，TS 里没有对应物。

### 【背景卡片】watch channel：只保留最新值的广播

- **一句话定义**：`tokio::sync::watch` 是一个单值广播通道：发送端 `send` 覆盖当前值，每个接收端随时 `borrow()` 到**最新**值。控制任务每拍 publish 传感数据（`src/control.rs:104`），RPC 层推订阅时现取（`src/main.rs:122`）。
- **为什么需要**：控制循环 50 Hz 出数据，订阅推送 1 Hz——生产者比消费者快 50 倍。队列式通道（mpsc）会在中间积压 49 份过期数据，消费方读到的永远是"最老的一帧"；watch 只留最新值，过期即丢。控制循环也**绝不能**因为某个客户端网慢而被拖住——send 无缓冲、不等待，天然无背压反噬。
- **最小示例**（大白话：watch 是一块"公告牌"——生产者随时擦掉旧值写新值，消费者路过时只能看到当前这块，擦掉的就永远错过了）：

```rust
// examples/ex_watch.rs —— cargo run --example ex_watch 可跑（此处省略 use 与 #[tokio::main] 包装）
let (tx, rx) = tokio::sync::watch::channel(0u64);
tokio::spawn(async move {
    let mut t = interval(Duration::from_millis(10)); // 快生产者
    for i in 1..=50 { t.tick().await; tx.send(i).unwrap(); }
});
for _ in 0..3 {
    sleep(Duration::from_millis(180)).await; // 慢消费者
    println!("消费者只读到当时的最新值: {}", *rx.borrow());
}
```

实际输出——消费者每 180ms 来看一眼，中间的值被直接跳过、永远读不到：

```text
消费者只读到当时的最新值: 19
消费者只读到当时的最新值: 37
消费者只读到当时的最新值: 50
```

- **TS/Node 类比**：RxJS 的 `BehaviorSubject`——持有一个当前值，新订阅者立刻拿到最新值，老值被覆盖。
- **差异点**：① BehaviorSubject 会把每次发射同步推给所有订阅者回调；watch 的接收端是**主动来取**，中间拍被跳过没有任何通知。② "last-value-wins"是语义而非配置——这正是 Phase 1 决策卡片里"跨服务读取一律 last-value-wins 缓存，控制循环永不阻塞等别的服务"的实现形态。③ 使用纪律：`borrow()` 的读锁不能带着跨 `.await`（`src/main.rs:119-121` 的注释），否则发送端每拍 publish 都要等这条连接的网络。

### 【背景卡片】IMU：惯性测量单元

- **一句话定义**：IMU（Inertial Measurement Unit）是机器人躯干上的一颗芯片，回答两个问题："我现在转多快"（陀螺仪 gyro，三维角速度）和"重力相对我指向哪"（gravity，三维单位向量）。
- **为什么需要**：15 个关节角只告诉机器人"我的肢体摆成什么形状"，不告诉它"我整个人朝哪倒"。双足站立/行走的全部平衡决策都建立在躯干姿态上——这是策略观测的前 6 维（gyro3 + gravity3），也是 M5 跌倒检测的输入。
- **TS/Node 类比**：浏览器的 `DeviceMotionEvent`——`rotationRate`（alpha/beta/gamma，度/秒）对应 `gyro`；`accelerationIncludingGravity` 归一化后对应 `gravity`；手机网页体感游戏用的就是同一颗物理原理的传感器。
- **差异点**：① **坐标系是躯干坐标系，不是世界坐标系**——gravity 表达的是"重力在机器人自己视角里的方向"。以 `ImuData::default()` 的数字为准：直立时 gravity = `[0, 0, -1]`（`src/io.rs:38-41`）。按 x 朝前、y 朝左、z 朝上来读，重力在 −z，指向脚。机器人前倾时这个向量获得朝前的 x 分量，所以它直接编码"我歪了多少"。源码注释写「机体系 z 向下为正」，和这个默认值相反——若 z 向下为正，直立应是 `[0, 0, 1]`。② 停住和正在转是两件事：人已经歪了但停住时 gyro 仍是 `[0, 0, 0]`，变的是 gravity；原地左转时 gyro 的 z 非 0，gravity 仍是 `[0, 0, -1]`。③ `quat` 的读法见下一张卡片。M2 的观测向量只用 gyro+gravity 这 6 维，quat 留给 M5 跌倒检测。④ 本章的 FakeIo 恒定返回直立姿态（`ImuData::default()`），不做滤波不建模——占位而已，真的 IMU 数据从 M4 的仿真器来。

### 【背景卡片】四元数：一根轴加一个角度

- **一句话定义**：`quat` 把「从直立到当前姿态」收成一次旋转：绕一根单位轴转一个角度，写成四个数，顺序 wxyz（`src/io.rs:31-32`）。
- **为什么需要**：三个欧拉角要先绕这根再绕那根，两根轴叠在一起时会丢掉一个自由度。四元数只用一根轴加一个角度。
- **公式**：`[cos(θ/2), ax·sin(θ/2), ay·sin(θ/2), az·sin(θ/2)]`。身体转的是 θ，写进去的是一半。直立、没转时 θ = 0，所以是 `[1, 0, 0, 0]`。
- **反着读**：角度 = `2·arccos(w)`，`(x, y, z)` 的方向是转轴。转到 360° 时四元数是 `[-1, 0, 0, 0]`，和 `[1, 0, 0, 0]` 表示同一个直立。先不用管 i、j、k 的乘法。

### 【背景卡片】插值（lerp / tween）

- **一句话定义**：插值是从姿态 A 到姿态 B 不瞬移：把过渡切成 N 拍，第 t 拍的目标是 `A + (B - A) × (t/N)`——线性插值（lerp）。本章 N=100 拍（2 秒 × 50 Hz，`src/control.rs:20-21`），15 个关节并行各自 lerp。
- **为什么需要**：舵机是物理设备：从躺平角瞬移到站立角意味着满扭矩猛抽，轻则齿轮打齿，重则机器人把自己掀翻。所有姿态变化都必须以"每拍挪一点"的方式发生。
- **最小示例**（大白话：机器人从躺平到站直不能"瞬移"——把过渡切成 100 拍，第 t 拍只走全程的 t/100）：

```js
// examples/lerp.mjs —— node examples/lerp.mjs 可跑
const lerp = (a, b, t) => a + (b - a) * t;

const start = 0.0, home = -0.4579, N = 100; // 一个关节：躺平角 → home 角
for (const tick of [0, 50, 100]) {
  const t = Math.min(tick, N) / N;
  console.log(`第 ${tick} 拍（t=${t}）: 目标角 ${lerp(start, home, t).toFixed(4)}`);
}
```

实际输出——第 50 拍恰好是中点，第 100 拍到达终点：

```text
第 0 拍（t=0）: 目标角 0.0000
第 50 拍（t=0.5）: 目标角 -0.2289
第 100 拍（t=1）: 目标角 -0.4579
```

- **TS/Node 类比**：CSS `transition` 或 GSAP tween 的逐帧中间值计算——`gsap.to(obj, {x: 100})` 内部就是每帧 lerp。
- **差异点**：① 前端 tween 由合成器/渲染帧驱动，帧率抖了只是动画不丝滑；控制循环的拍对应物理时间，插值必须锚定拍号而不是墙钟（否则一帧卡顿就变成一次抽动）。② 本章用线性路径（`t` 均匀推进），没有缓动曲线——站立这种单调过渡不需要 ease-in-out，需要时再换 `t` 的映射函数即可，`ramp_target` 是纯函数，换曲线只动一行（`src/control.rs:37`）。

## 3. 实现走读

### 3.1 身体模型（`src/model.rs`）

- **一张表是全项目唯一来源**（`src/model.rs:1-6`）：关节顺序就是协议里 `positions`/`targets` 数组的位置序，协议侧和控制侧都从这张表取，杜绝两份表漂移（原版用 const 断言锁死两侧，我们目前只有一份表，文件头注释说明了什么时候补锁）。
- **15 关节，嘴是例外**。`JOINT_NAMES`（`src/model.rs:13-29`）：左右腿各 5（hip_yaw/hip_roll/hip_pitch/knee/ankle）+ 颈头 4 + 嘴 1。`MOUTH_INDEX = 9`（`src/model.rs:31-33`）单列出来，因为"策略维数 14 vs 总线维数 15"的歧义在原版文档里是专门解释过的坑——策略不控制嘴，但写总线时必须给满 15 维。
- **home 姿态是常量不是配置**（`src/model.rs:35-42`）：左右腿镜像等大反号，躯干前倾让质心落在踝关节轴正上方——双足静止站立的稳定性就押在这几个弧度值上。改成配置项等于允许运行期把机器人调成站不稳的姿态。
- **手抄的表用测试兜底**（`src/model.rs:48-68`）：表长断言 + 镜像断言（左右腿对应关节和为零）。注释说得很直白（`src/model.rs:8-9`）："抄错一位不会编译错，只会把左腿的命令写到右腿上"——类型系统帮不了魔法数，测试来补。

### 3.2 总线抽象（`src/io.rs`）

- **trait 瘦身到 read/write 两个方法**（`src/io.rs:63-66`）。`set_gain`/`set_torque`/`reboot`/`slow_sensors` 是 M5 的事，现在写上只是没人调用的死代码（`src/io.rs:4-6`）。这是铁律 9 的直接执行：当前里程碑够用即可。
- **错误不分类**（`src/io.rs:11-14`）：`IoError` 就是一个字符串。控制循环目前只需要"失败了"这一个事实（据此跳过本拍），分类等到有调用点真的按类别分支时再加。
- **关节和 IMU 装进同一个 `Sensors`**（`src/io.rs:46-55`）：硬件上它们就是一次 sync_read 一起回来的；拆开会让"这一帧的姿态和这一帧的关节角是不是同一时刻"变成调用方的责任。数据结构的形状在编码硬件事务的边界。
- **FakeIo 完美跟踪，故意不建惯性模型**（`src/io.rs:68-72`）：write 之后 read 原样返回写入值。注释写明：M1 没有任何逻辑依赖跟踪延迟，加一阶惯性模型只会让测试多一个要调的参数——"够用即可"再次压倒"将来可能用到"。
- **故障注入是一等公民**：`failing_reads(n)`（`src/io.rs:87-95`）让前 n 次 read 报错，模拟舵机电源未就绪。它不是测试私货——daemon 启动时可以通过环境变量 `MINIDUCK_FAKE_FAILING_READS` 打开（`src/main.rs:27-33`），验收容错行为不需要改代码。

### 3.3 控制任务（`src/control.rs`）

这是本里程碑的心脏，逐设计点看：

- **首拍推迟一个周期**（`src/control.rs:53-57`）：`interval_at(Instant::now() + TICK_PERIOD, TICK_PERIOD)` 而不是 `interval(TICK_PERIOD)`。后者第一拍立即触发，第 0 帧会在一次传感器数据都没读到时跑控制逻辑、把垃圾写上总线（第 1 章 Q3 的坑，注释里写着"已知坑，勿改回 interval"）。推迟后每一帧都走同一条 read → compute → write 路径。
- **健康数据由循环自己记账**（`src/control.rs:23-32`）：`Stats` 是四个 `AtomicU64`（tick/reads/writes/skipped_reads），health 端点直接读它（`src/main.rs:96-100`）。"healthy" 的语义是"循环在跑"，不是"socket 活着"——RPC 层只搬运，不猜。多线程下用原子量而非锁：控制循环每拍写、RPC 层偶尔读，原子量的竞争开销可以忽略。
- **插值是纯函数**（`src/control.rs:34-43`）：`ramp_target(start, tick_in_ramp)` 不碰时钟不碰 IO，"1 秒时在中点"因此能用普通单元测试断言到 1e-6（`src/control.rs:145-156`），而不是去跑真时钟。这是"难测的行为 = 纯函数 + 薄壳"的标准拆法。
- **插值起点锚定第一次 read 成功，而不是进程启动**（`src/control.rs:59-62`）。这是本章最重要的一个设计决定：舵机电源未就绪时前若干拍 read 全失败，如果从进程启动时刻起算插值，总线恢复时插值早已走完，机器人会**瞬间跳到 home**——真机上就是 15 个舵机满扭矩猛抽。锚定第一次成功读后，无论总线晚来多久，过渡永远从"总线上读到的真实姿态"平滑开始。
- **读失败：跳过本拍，计数，继续**（`src/control.rs:91-102`）：不 panic、不退出、不写总线。注释给出原则——"机器人没电是可以恢复的状态，进程退出不是"。`skipped_reads` 计数让这件事可观测（health 端点暴露，验收就靠它断言）。
- **抖动统计要算对指标**（`src/control.rs:73-89`）：对"相邻拍间隔"取 P99 恒等于 20 ms，是废指标；代码里攒的是"间隔相对标称 20 ms 的**偏差**"（`src/control.rs:76-77`），每 5 秒打一行 `tick_rate=50.0Hz p99_jitter_ms=0.00` 给 grep 用。
- **发布用 watch，跨 await 不持锁**：每拍 `sensors_tx.send`（`src/control.rs:104`）；消费侧 RPC 层 `borrow()` 只在取数一瞬持有（`src/main.rs:119-122`），随后立即释放再做异步 send——纪律写在注释里。

### 3.4 接线（`src/main.rs`）

M1 对 M0 骨架的改动是三处插入：`control::spawn(io)` 替换空转 tick 计数器（`src/main.rs:37`）；health 响应增加 reads/writes/skipped_reads 三个字段（`src/main.rs:98-100`）；订阅通知的载荷从纯 tick 变为 tick + positions（`src/main.rs:122-129`）。推送频率**仍为 1 Hz**（`src/main.rs:60-63`）——对齐原版 50 Hz 的 state 流留给客户端真吃得下的里程碑（偏差 D14）。

## 4. 验收

验收命令：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test
MINIDUCK_FAKE_FAILING_READS=100 ./target/debug/miniduckd &
sleep 4 && ./target/debug/mini-duckctl health
timeout 3 ./target/debug/mini-duckctl subscribe
pkill -x miniduckd
```

实测输出（逐字摘自 `docs/milestones/m1-body-fakeio/acceptance.md`，主 agent 复跑）：

> - 6 单测全过（镜像断言/表长/完美跟踪/failing_reads 恢复/插值端点/中点 1e-6）
> - health @4s：reads=writes=tick=200 严丝合缝
> - subscribe：positions 与 DEFAULT_POSITION 逐元素 |diff| = 0.0
> - failing_reads=100：skipped_reads 恰好 100，恢复后收敛，daemon 不退出

断言解释：

- **6 个单测**分三层：`model.rs` 两个（表长、镜像——守住手抄常量）；`io.rs` 两个（FakeIo 完美跟踪、故障注入恰好 n 次后恢复——守住假总线的行为契约）；`control.rs` 两个（插值端点：第 0 拍=起点、第 100 拍=home、第 300 拍仍=home 保持不动；插值中点：第 50 拍=起点与 home 的中点，误差 < 1e-6——纯函数带来的精确断言）。
- **reads=writes=tick=200 @4s**：三个计数相等且等于 4s × 50Hz，证明每拍完整走完 read → compute → write，无一拍跳读、无一拍写失败。注意这是**外部观察者通过 IPC 读到**的账——M0 的验收工具链在 M1 兑现了它的价值。
- **positions 与 home 逐元素零误差**：4 秒 > 2 秒插值期，机器人已"站定"；零误差（不是近似）来自 FakeIo 的完美跟踪语义——写下去的下一拍原样读回。
- **skipped_reads 恰好 100**：故障注入 100 次，跳过计数恰好 100，一次不多一次不少（说明计数路径和跳过路径完全互斥）；之后 reads 恢复增长（插值锚定第一次成功 read 的语义在此生效）；**daemon 不退出**——容错验收的核心断言。

## 5. 与原版差异

| 偏差 | 内容 | 预定收敛点 |
|---|---|---|
| D2 | FakeIo 无真实 Dynamixel 串口协议 | **已豁免**——硬件层闭源，"能下载的不重写"原则覆盖 |
| D9 | 启动即插值到 home；原版是"进程启动绝不移动机器人"（held_pose） | M5 安全层落地时引入"启动不动"语义 |
| D10 | 插值在控制循环内逐拍推进；原版 `interpolate_to` 是阻塞调用 | M8 裁决（逐拍推进与控制循环同构，可能保留） |
| D11 | `healthy` 恒 true，无 deadline 检测（循环死了 health 不会变 false，只是 tick 停涨） | M5 |
| D12 | RobotIo 只有 read/write，无 set_gain/set_torque/reboot/slow_sensors | M5（跌倒卸力需要 gain） |
| D13 | IoError 不分类（单一字符串错误） | M8 |
| D14 | `robot.state` 推送 1 Hz 且只带 positions（无 velocities/imu） | M2 补 imu 字段，M5 对齐 50 Hz |

D2 是全簿唯一的豁免项，其余都是带收敛条件的待收敛项。

**一条未登记的观察**（写作本章时对照原版发现，建议主 agent 裁决是否登记）：`src/control.rs:57` 的 `interval_at` 没有设置 `MissedTickBehavior`，即默认 `Burst`（错过就追拍补发）；原版主循环在 `reference/robotd/src/main.rs:1786-1801` 用注释专门论证了为什么选 `Skip`（`Burst` 会把积压的电机命令连发叠出去，`Delay` 实测掉到 43.1 Hz）。M1 阶段循环体是假总线内存写入、几乎不可能 miss tick，所以无实际影响；但 M3 接入 ONNX 推理后单拍耗时上升，这个默认值会变成真问题。

## 6. 常见坑

1. **`tokio::time::Instant` ≠ `std::time::Instant`**，`interval_at` 只收前者。混用编译报错但报错信息不直接点破这一层（类型名相同、路径不同）——M1 实测踩到。规则：进了 tokio 的定时器世界就用 `tokio::time::Instant`（`src/control.rs:13`），进程外部计时（如 uptime）才用 `std` 的（`src/main.rs:11`）。
2. **对"相邻拍间隔"取 P99 恒等于 20 ms，是废指标**。健康的 50 Hz 循环相邻间隔本来就都是 20 ms——P99 也约等于 20 ms，看不出任何问题。要对"间隔与标称周期的**偏差**"取 P99（`src/control.rs:76-77`），偏差才是抖动。度量指标先问"健康时它长什么样"，健康时恒定的指标没有信息量。
3. **插值锚进程启动 = 总线恢复时姿态跳变**。如果插值进度从进程启动起算，舵机电源晚就绪，总线恢复那一刻插值可能已经走完，写入值从"躺平"直接跳到 home——真机 = 舵机猛抽。锚定第一次 read 成功（`src/control.rs:59-62`）后，这个失败模式在结构上不存在。拍号级的时间线见本节 Q10。教训的一般形：**涉及物理设备的进度条，起点必须锚在"设备第一次给出真实反馈"的时刻**。

## 7. 读者提问

**Q6：插值是什么？**

插值就是前端的 tween/补间：从当前姿态到 home 不瞬移，100 拍每拍推进 1%，15 个关节并行各自 lerp。直接映射你熟的：CSS `transition: transform 2s linear` 声明"用 2 秒线性过去"，GSAP 每帧算中间值——本章的 `ramp_target`（`src/control.rs:36-43`）就是那个"算第 t 帧中间值"的函数，只是帧换成了 50 Hz 控制拍。M1 用线性路径，缓动曲线暂不需要。验收锚点：第 50 拍位置 = 起点与 home 的中点（1e-6 精度，单测 `ramp_midpoint_at_one_second`，`src/control.rs:145-156`）。与前端 tween 的关键差别：这里的"帧"对应物理时间，所以插值进度锚定拍号而不是墙钟——见常见坑 3。

**Q7：IMU 的数据结构看不懂——gyro、gravity、quat 到底是什么？**

完整背景卡片在本章概念铺垫的【IMU】【四元数】两节。压缩版：IMU 是躯干上的芯片。gyro 是这一拍转多快（类比 `DeviceMotionEvent.rotationRate`），gravity 是重力在自己身上指向哪（类比 `accelerationIncludingGravity` 归一化）。字段都在躯干坐标系里，直立 gravity = `[0, 0, -1]`，指向脚。人可以已经是歪的但停住，这时 gyro 仍是全 0，变的是 gravity；原地左转则相反，gyro 的 z 不是 0，gravity 不变。quat 的读法见 Q9。M2 只用 gyro+gravity 前 6 维。FakeIo 恒定返回这份直立默认值。

**Q8：教程示例里的 `mut` 是什么作用？**

Rust 的 `let` 默认不能改。`mut` 表示这个名字之后还能用来改值，管的是绑定，不是类型本身会不会变。固定频率那张卡片的示例里，`timer` 要 `mut`，因为 `tick()` 的签名是 `&mut self`，调用会改下一次该响的时刻；`ticks` 要 `mut`，因为 `ticks += 1`。`period` 和 `start` 只被读，不加 `mut`。

**Q9：四元数怎么理解？**

`quat` 不是第四颗传感器，而是把「绕哪根轴、转了多少」写成四个数，顺序 wxyz：

`[cos(θ/2), ax·sin(θ/2), ay·sin(θ/2), az·sin(θ/2)]`

`(ax, ay, az)` 是单位转轴，θ 是身体实际转过的角度。写进四个数的是 θ 的一半，所以转 90° 时 w 约是 0.71，转到 180° 时 w 才是 0、后面三个数变成转轴本身。绕上轴左转 180° 得到 `[0, 0, 0, 1]`，鼻子朝后。绕左轴（+y）前倾 θ 得到 `[cos(θ/2), 0, sin(θ/2), 0]`。转到 360° 时身体已经回到直立，四元数是 `[-1, 0, 0, 0]`，和 `[1, 0, 0, 0]` 表示同一个姿态。反着读：角度 = `2·arccos(w)`，`(x, y, z)` 的方向是转轴。平方和恒为 1。

**Q10：`control.rs` 里插值起点为什么不是进程启动时刻？**

秒表的零点是第一次 `read` 成功，不是进程起来的那一拍。`RAMP_TICKS` 是 100。`main.rs` 的 `MINIDUCK_FAKE_FAILING_READS=100` 让前 100 次 `read` 失败。失败分支（`src/control.rs:93-100`）只把 `tick` 加一然后 `continue`，`ramp_origin` 仍是 `None`，这时变量 `tick` 已经等于 100。

下一次成功才执行 `get_or_insert((tick, sensors.positions))`（`src/control.rs:106-108`）。本拍进度是 `100 - 100 = 0`，写出去的是刚读到的躺平姿态，然后再用 100 拍走到 home。零点若钉在启动时的 0，同一拍的进度是 `100/100 = 1`，第一笔 `write` 已经是 home，关节从躺平一拍跳到站立角。

失败若只有 40 拍，错误零点第一次写出全程的 40%，还没到 home，但第一笔已经跳过前 40%。起点记下之后，再出现的读失败仍会计入这 100 拍（`tick` 照加），只是那一拍不写总线。所以程序的行为是：等到第一次读成功，从当时的姿态用 2 秒线性插到 `DEFAULT_POSITION`，之后保持。

**Q11：`if let Some(prev) = last_tick_at` 是什么语法？**

这是模式匹配，不是赋值。`last_tick_at` 的类型是 `Option<Instant>`（`src/control.rs:68`）。右边的值套进左边的形状：是 `Some(某个 Instant)` 时，里面的时刻绑成 `prev`，进入花括号；是 `None` 时整段跳过。第一拍还没有上一拍，所以不会拿一个不存在的时刻做减法。花括号里算的是本拍间隔减去 20 ms 之后的绝对值，推进 `deltas_ms`。

**Q12：读失败那个 `match` 里的 `continue` 是直接进入下一轮吗？**

是。`io.read()` 返回 `Result`（`src/control.rs:91-102`）。`Ok(s) => s` 把传感器数据拿出来，`match` 的值就是这个 `Sensors`。`Err(_)` 里的 `_` 表示错误内容不用。`continue` 结束当前这一轮，回到 `loop` 开头，下一轮先在 `timer.tick().await` 上等下一个 20 ms。它跳过的是本拍后面的发布、插值、write，以及循环底部那一次 `tick += 1`。失败分支里已经把 `tick` 加过，拍号不漏。循环不退出。

**Q13：`Ordering::Relaxed` 是什么？「+1 写到一半」和「可见顺序」怎么理解？**

`skipped_reads` 是 `AtomicU64`。控制循环在写，别的任务在 `load`。`fetch_add(1, Ordering::Relaxed)` 把「读进寄存器、加一、写回」合成一次原子的读-改-写，别的线程插不进这三步中间。两个线程同时做普通 `+= 1`，可能都读到 5、都算出 6、都写回 6，两次加一只留下一次。丢的是一次完整更新，整数本身仍是完整的 6。「写到一半」说的是另一次情况：一个逻辑值要分多次内存传输才写完，例如 32 位机器上写一个 64 位整数。这段 `AtomicU64` 在 64 位机器上一次写完，不存在半个整数。

`Relaxed` 只保证计数器自己的加减是原子的。它不承诺：别的线程看见这个数变成 8 时，也能看见你在这次加法之前写进其他普通变量的内容。需要那层承诺时，写的一边用 `Release`，读的一边用 `Acquire`。编译之后就是两次写内存，中间一道屏障；另一个线程看不到你执行到了哪一行，它只能看见自己从内存里读到的值。`Release` 约束的是这两次写入被别人看见的顺序。

因此源码里先写 `payload = 42` 再写 `ready = true`，或者用 `while ready { 读 payload }`，都只是阅读顺序。没有 Ordering，编译器和 CPU 仍可以把 `payload` 的写入排到 `ready` 之后；普通变量还可能被提前读进寄存器，循环里再读多少次都是旧值。`skipped_reads` 旁边没有另一份要一起发布的数据，状态接口只是把它读出来，所以 `Relaxed` 够用。

**Q14：`get_or_insert`、元组解构、`is_ok` 这段在做什么？**

`ramp_origin` 的类型是 `Option<(u64, [f64; NUM_JOINTS])>`（`src/control.rs:62`）。`get_or_insert` 是 `Option` 的方法：里面已经有值就返回那个值的可变引用；还是 `None` 就把参数插进去再返回引用。参数 `(tick, sensors.positions)` 是元组，当前拍号配上这一拍的关节位置。第一次读成功时插进去，以后每拍取出同一份起点。

方法返回的是引用，前面的 `*` 把元组复制出来。`let (origin_tick, start_pose) = ...` 把第一项、第二项分别绑到两个名字上。`ramp_target(&start_pose, tick - origin_tick)` 里的 `&` 是传数组的引用。`JointTargets { positions: target }` 是结构体字面量，字段名 `positions`。再一个 `&` 把这个临时值的引用交给 `io.write`。`write` 返回 `Result`，`.is_ok()` 为真才把写次数加一。

**Q15：`stats.clone()` 和 `sensors_rx.clone()` 是在内存里复制了一份吗？**

不是。两个 `clone` 复制的都是句柄，每个连接拿到的是同一份数据的另一个使用权（`src/main.rs:47`）。

`stats` 的类型是 `Arc<Stats>`。`Arc` 是指向堆上那一份 `Stats` 的指针，再加一个引用计数。`clone` 复制这个指针，并把计数加 1。控制循环和每个连接读到的 `tick`、`reads`、`writes` 都是同一组原子计数器。

`sensors_rx` 的类型是 `watch::Receiver<Sensors>`。`clone` 再做一个接收端，内部同样是指向同一条 watch 通道的指针。通道里只保留最新一帧；控制循环 `send` 时更新这一帧，各个连接 `borrow` 时读的也是这一帧。

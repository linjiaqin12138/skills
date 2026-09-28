# 第 6 章 · M5：安全层——四条防线与一台说真话的守护进程

> 本章对应里程碑 M5（已验收）。交付物：`src/safety.rs`（Safety：唯一总线写句柄 + 四条防线）、`src/control.rs`（主循环重写为五相位状态机）、`src/io.rs`（trait 补 set_gain/set_torque/imu_ready，D22 严格校验）、`src/main.rs`（robot.enable/disable、health 真实判定、robot.state 50Hz、socket 0660）、`src/bin/mini-duckctl.rs`（enable/disable、drive 重发意图、state --every）、`sim/duck_body.py`（set_gain/set_torque/push）、一键验收 `scripts/accept-m5.sh`。

## 1. 动机

M4 的 `robot.drive` 是**第一个会动机器人的调用**。到此为止欠下的账，每一笔都货真价实：

- 任何人连上 socket 就能开车——没有任何鉴权（D8）。
- 策略吐出 NaN 会原样写进舵机目标寄存器。
- 进程一启动就把机器人拽回 home 姿态（D9）——哪怕它本来站得好好的。
- 客户端断粮（崩溃、掉线），机器人拿着最后一条速度命令照走。
- `healthy` 恒 true（D11）——健康检查在说谎。

原版把防线收在一个叫 `Safety` 的类型里（`reference/duck-control/src/safety.rs`，927 行；它的模块文档是整本参考里写得最好的段落之一，本章多处转述它的论点）。本章对齐它。在依赖树里：

- **上游**：M4 的真物理。没有重力，「跌倒」这个词没有意义——FakeIo 的 IMU 永远是 `[0,0,-1]`。
- **本章**：Safety 收走唯一写句柄；控制循环重写为五相位状态机；health 开始说真话。
- **下游**：M6 技能调度器会有多个技能想开车——写句柄必须先于多写者收拢，否则收拢永远做不成。M7 updaterd 的回滚决策吃 health 输出——它得是真话。

## 2. 概念铺垫

### 【背景卡片】deadman 开关（死人开关）

- **一句话定义**：操作者必须持续证明自己活着；证据一断，机器自动回到安全态。名字来自火车司机脚下的「死人踏板」——脚松开（昏厥、离岗），列车自动刹车。
- **为什么需要**：遥控器没电、网络断了、客户端进程崩了——机器人无法区分「用户想让它继续」和「用户已经不在了」。没有 deadman，最后一条命令就成了遗嘱：人在 3 秒前发的「前进」，机器执行到没电为止。
- **机器人领域的特殊性**：失联后的安全态是**站住**，不是瘫软——原版 safety.rs:17-20 的原话是 "stop is not limp"：站立是双足的安全态，瘫软是另一种事故。这和火车相反（火车停下来最安全），值得想一秒为什么：双足机器人瘫软=摔倒=砸坏自己。
- **最小示例**（已在本仓库容器里跑过，`docs/concepts/assets/deadman/deadman_demo.py`）：模拟 50Hz 控制循环，「司机」每 100ms 发一次意图，t=2.0s 时司机失联。有/无 deadman 两组对照：

```text
有 deadman：
  0.0s:0.15 0.3s:0.15 0.6s:0.15 0.9s:0.15 1.2s:0.15 1.5s:0.15 1.8s:0.15 2.1s:0.15 2.4s:0.00 2.7s:0.00 3.0s:0.00 3.3s:0.00
无 deadman：
  0.0s:0.15 0.3s:0.15 0.6s:0.15 0.9s:0.15 1.2s:0.15 1.5s:0.15 1.8s:0.15 2.1s:0.15 2.4s:0.15 2.7s:0.15 3.0s:0.15 3.3s:0.15
```

失联后约 0.5s（保鲜期），有 deadman 的速度归零；无 deadman 的永远走下去。**回到项目**：`Safety::gate`（`src/safety.rs:208-228`）就是接收端那几行；「司机」是任何调 `robot.drive` 的客户端，本章的 CLI 在 `--secs` 期间每 100ms 重发意图扮演它（`src/bin/mini-duckctl.rs:80-83`）。

**自测**：保鲜期 500ms、重发间隔 100ms，为什么能稳定覆盖？如果把保鲜期改成 50ms 会怎样？（先想，本章破坏实验 1 有实测答案。）

### 【背景卡片】舵机 gain 与 torque：弹簧硬度与总开关

- **一句话定义**：位置舵机内部是一根弹簧——输出力矩正比于「目标角度 − 当前角度」。`gain`（P 增益）是弹簧硬度，`torque` 是总开关：关掉它，舵机不输出任何力，目标照写也没用。
- **为什么需要**：站着的鸭子要硬弹簧（gain=200，抗重力抗扰动）；倒下的鸭子要软弹簧（gain=50，不让电机顶着地板较劲）；daemon 重启期间舵机 RAM 里的 torque 跨进程存活——这条事实是「启动不动」设计的物理基础。
- **一张卡片放不下物理直觉**，深读见概念文档 [docs/concepts/servo-gain-torque.md](../../concepts/servo-gain-torque.md)（含跑过的单关节弹簧仿真：满增益下垂 31°、软增益下垂 63°、卸力直接垂到 90°）；网页交互演示见 [assets/servo-gain-torque/index.html](../../concepts/assets/servo-gain-torque/index.html)。

### 【背景卡片】去抖（debounce）

写过前端的话这就是 debounce，但机器人上去抖的是物理信号：投影重力越过跌倒阈值**不算**跌倒，持续越线 200ms 才算；而任何一个直立样本立刻把累加器清零（`src/safety.rs:181-191`）。为什么必须去抖：一次扎实落脚的冲击会让重力读数瞬间越线——把这当跌倒，步态中途就会卸力软倒，**防跌机制自己制造跌倒**（`src/safety.rs:338-339` 的测试注释原话）。两条单测钉住两个方向：`brief_tilt_is_not_a_fall_and_upright_resets` 与 `sustained_tilt_is_a_fall`（`src/safety.rs:340-357`）。

## 3. 实现走读

### Safety：四条防线

`Safety<T: RobotIo>` 的结构一句话说完：它**拥有** io，字段私有化（`src/safety.rs:113-127`）。`main.rs:87-90` 把 io 交进去之后，整个进程再没有第二条碰电机的路径——策略拿不到、RPC 层拿不到、将来的技能调度器也拿不到。

**【决策卡片】写句柄私有化：借用检查器强制 vs 纪律约定**

- **决策点**：「只有安全层能写电机」这条不变量靠什么保证。
- **备选**：1. Safety 按值拥有 `RobotIo`，读也经它透传（`src/safety.rs:142-145`）；2. `Arc<Mutex<RobotIo>>` 大家共享，靠代码评审和约定保证「只有安全层调 write」。
- **选择**：备选 1。原版模块文档的论点（safety.rs:5-10）：只在出事时才跑的代码最容易悄悄坏掉——加第八个技能的深夜，没人记得住「别直接碰总线」这条纪律，所以让坏状态**不可表示**，由借用检查器在编译期强制。备选 2 的不变量活在每个贡献者的记忆里。
- **放弃的成本**：读也要经过 Safety（透传一层，无逻辑）；测试想看内部状态得开 `#[cfg(test)]` 后门（`src/safety.rs:302-307`，故意不 `pub`）；RPC 层想报 gain/torque 只能从 Safety 的快照里拿。
- **失效边界**：若将来出现合法需要直达总线的组件（如舵机校准工具），正确做法是给它开 Safety 的一个新模式，而不是把句柄再交出去——届时这张卡要重审。

四条防线逐一看。

**防线一：非有限目标拒绝——是拒绝，不是夹紧。** `apply` 的第一步（`src/safety.rs:247-261`）：15 个目标里有任何一个 NaN 或 ±Inf，**整包拒绝**，改写 `hold`（保持现状的姿态），上报 `Limit::NotFinite`。

**【决策卡片】NaN：拒绝 vs 夹紧**

- **决策点**：策略吐出 NaN，安全层怎么办。
- **备选**：1. 整包拒绝写 hold；2. 夹紧到 ±π 当普通越界处理；3. 逐关节 sanitise，坏关节换 hold、好关节照写。
- **选择**：备选 1。NaN 夹紧会产出一个**看似合理的边界角**——机器人猛冲到限位，日志里还像正常夹紧。保持不动远不如猛冲危险（`src/safety.rs:247-248` 注释）。备选 3 让半个身体的策略输出生效，产生的姿态不在任何训练分布里，比不动更难预料。
- **放弃的成本**：一个关节 NaN，十五个关节一起冻结一拍——好关节的合法目标也被丢弃。50Hz 下一拍 20ms，代价可忽略。
- **失效边界**：如果 NaN 成为常态（策略权重损坏每拍都吐），机器人会冻在原地——这是设计想要的，但运维侧要能从日志认出它（见下面的日志限流）。本章破坏实验 2 亲手制造这个场景。

**防线二：±π 行程夹紧 + 上报。** 有限但越界的目标夹到 ±π（`src/safety.rs:264-287`），并在返回值里上报 `Limit::Range`——调用方有权知道命令被改过，而不是看着机器人「不听话」无从排查。诚实边界写在常量注释里（`src/safety.rs:24-29`）：±π 是 **XL330 执行器**的行程，不是各关节的解剖限位——它拦得住策略吐垃圾，拦不住「把关节开到机械上不明智的位置」。一条看起来像逐关节保护、实际不是的防线，比没有防线更害人。

**防线三：deadman。** 每拍 `gate`（`src/safety.rs:208-228`）：意图年龄超 500ms，twist 清零、上报 `Limit::Deadman`。两个细节都是论点：

- **只清 twist，不动 head/body**：stale 的头姿无害，stale 的速度会撞墙（注释原文，`src/safety.rs:203-205`）。
- **`deadman_armed` 语义**：从没被驾驶过的机器人不报 deadman——「从没司机」不是新闻，「司机失联」才是。否则一台从没被驾驶过的台式机器人从启动后半秒开始每秒刷一行日志，一天八万行（`src/safety.rs:123-126`）。收到第一条新鲜意图才 armed（`src/safety.rs:209-212`）。

**防线四：跌倒判定——报告，不是门控。** `observe` 每拍更新 `fallen`（`src/safety.rs:169-201`）：投影重力 z 越过 −0.5 且持续 200ms 判倒——**倒下方向去抖、起身方向立即**：任何一个直立样本立刻清零并解除 fallen（不对称是故意的：误判"倒了"会卸力摔向地板，起身方向拖不得）。判定翻转必报且**不限流**——这是事件不是状态复述，限流丢掉的是「什么时候倒的/什么时候起来的」（`src/safety.rs:193-200`）。想直观感受 gravity[2] 越线、200ms 计时条走满才锁存 fallen 的全过程，见 [IMU 演示网页](../../concepts/assets/imu/index.html)（预设③「向右翻倒再翻回」会完整走一遍这条链路）。

一条血泪回归钉在第一行：`imu_ready() == false` 时**直接 return 不投票**（`src/safety.rs:177-179`）。原版实测：真机的 SFLP 姿态滤波器要几秒样本才有意义，收敛前投影重力是滤波器半路决定的任意值——一台直立在台架上的机器人 200ms 后被锁成 fallen、增益被写成 50，几秒后自行恢复却留下低增益，可观测状态里没有任何东西解释发生了什么。FakeIo/SimIo 没有滤波器，`imu_ready` 默认 true（`src/io.rs:74-78`），这条防线只在真总线上咬合——但单测用 `FakeIo.imu_ready=false` 两个方向都钉死了它（`src/safety.rs:359-393`）。

**【决策卡片】fallen 是报告，不是门控**

- **决策点**：跌倒判定要不要抢占写入——判定一倒，安全层是否直接接管。
- **备选**：1. fallen 只是对外发布的报告，不抢占任何写入（原版行为）；2. fallen 时安全层强制卸力/冻结，调用方说了不算。
- **选择**：备选 1。原版 safety.rs:22-26 的论点：一台重力读数一晃就认输的机器，会在人搬运它的时候不停坐下去。跌倒判定是**传感器意见**，不是事实；把不可逆动作建在它会错意的信号上，等于让误报有执法权。摔倒后怎么办（软倒）是控制决策，住在控制层——它通过**同一个 apply** 以普通目标+普通增益下达，没有特权通道（`src/safety.rs:243-244` 注释：`apply` 里没有跌倒门控）。
- **放弃的成本**：一个不知道自己该软倒的上层会顶着倒下的机器人继续驱动——保护依赖控制循环做对该做的事。单测 `fall_does_not_preempt_the_caller`（`src/safety.rs:398-416`）把这个「不保护」钉成契约。
- **失效边界**：M6 引入多技能后，「摔倒后怎么办」如果有多个答案互相打架，问题出在调度层而不是这张卡——但若某天要求「任何情况下倒地 1 秒内必须卸力」这种硬保证，报告模型就不够了。

**横切：日志限流与增益缓存。** 三条防线共用一套限流（`src/safety.rs:83-111`）：首次触发必报（只夹紧一次也是新闻），持续触发每 50 拍再报一行（50Hz 下每秒一行）。三条**独立** run 计数——共享计数会让触发最勤的那条饿死另外两条。一个故意的粗糙：NaN 拍在夹紧之前 return，**不清零 range run**（`src/safety.rs:94-97`）——否则 NaN 和越界交替时两条规则每隔一拍各自开启新 run，每拍刷一行，正是计数要防的。增益缓存（`src/safety.rs:293-300`）：gain 值不变不重写——50Hz 下每拍 15 次 gain 写会挤爆控制循环要用的总线。

### 控制循环：五相位状态机

主循环从「插值到 home 然后跑策略」重写为五相位（`src/control.rs:176-189`，模块文档有图）：

```
Held ──enable──▶ RampUp(100拍) ──▶ Driving ◀──▶ Limp（跌倒⇄恢复）
  ▲                                    │
  └──── RampDown(100拍)+卸torque ◀──disable┘
```

每拍顺序对齐原版文档化顺序：**read → observe → gate → 策略 → apply**（`src/control.rs:269-283`）。read 失败跳过本拍（`src/control.rs:251-265`，D24：coast 滑行是禁止提前实现项）。

- **Held**：抱住启动时读到的姿态（首帧锁存），**绝不调 set_torque**。理由（`src/safety.rs:151-154`）：进程启动不是移动机器人的理由——舵机 RAM 里的 torque 跨进程存活，被 supervisor 重启的 daemon 必须让站着的机器人继续站着。policy=None（加载失败，D19）永远停在 Held。
- **RampUp**：enable 边沿 → set_torque(true) → 从**实测姿态**线性斜坡 100 拍（2s）回 home（`ramp_target`，`src/control.rs:101-108`）。t==RAMP_TICKS 那一拍已把 home 原样写出再转移，避免「差一步没到」（`src/control.rs:358-362`）。
- **Driving**：策略闭环，同 M4。
- **Limp**：Driving 中 fallen → 策略停摆，目标**跟随实测位置**、增益降到 50。为什么目标跟着身体走（`src/control.rs:183-186`）：倒地过程中固定目标会持续累积误差=电机顶着地板较劲——「软」的关键不是低增益 alone，是目标不再和身体打架。直立后斜坡回 home、策略锚点清零重来（`src/control.rs:319-327`）。
- **RampDown**：disable 边沿 → 斜坡回 home → set_torque(false) → Held。

**边沿检测先于跌倒判定**（`src/control.rs:285-311`）：「Driving 中跌倒」和「同拍 disable」同拍到达时会互相覆盖，实测踩过——改成边沿立即生效（见常见坑 2）。

### health 说真话

`robot.health` 从恒 true 改成真实判定（`src/main.rs:119-164`），四个条件全部来自原版 robotd-params：stall 500ms（25 拍没动）、实测频率低于 45Hz、连续读错误 ≥10 次、策略缺失（policy_ok=false，报 "policy unavailable: …"）。两个设计点：

- **数据由循环自己记账**（`Stats`，`src/control.rs:44-78`）：「healthy」的语义是「循环在跑」，不是「socket 活着」——RPC 层猜不出来，只能读循环的原子计数。
- **暖机豁免**（`src/main.rs:117-118` 注释）：首个 5s 窗口没满时 `achieved_millihz` 是 0，不按频率判——刚启动的循环频率必然低，拿它判病等于进程永远报病。

策略加载失败**不退出**（D19，`src/main.rs:56-67`）：原版理由是 Restart=always 下退出=crashloop——进程死了被拉起来再死再拉，updater 永远等不到一个能说话的进程；活着报病，回滚才有可能。

### IPC 面

- **socket chmod 0660**（`src/main.rs:92-101`，D8 收敛）：侦察结论——原版 robotd **没有** SO_PEERCRED，socket 文件权限就是全部鉴权（能打开这个文件的用户/组就能开车）；SO_PEERCRED 是 configd/updaterd 的模式，M7 的事，不超前实现。
- **robot.enable / robot.disable**（`src/main.rs:208-217`）：只写共享 `ControlState.enabled`，边沿检测和斜坡都在控制循环里做。
- **robot.state 改 50Hz**（`src/main.rs:250-281`，D14 收敛）：从 1Hz interval 改为 `frame_rx.changed()` 驱动——控制循环每拍发一帧快照，推送随帧走。载荷增 `fallen/enabled/gain/torque`。latest-wins：推送慢于 50Hz 时 watch 只保留最新帧不积压；逐订阅者降频是禁止提前实现项（D25）。
- **CLI 配套**：`drive --secs N` 期间每 100ms 重发意图（deadman 500ms 下单次意图只能驱动半秒——CLI 扮演持续发意图的手柄，`src/bin/mini-duckctl.rs:80-83`）；`state --every N` 默认每 50 帧打一行防刷屏（`src/bin/mini-duckctl.rs:46-77`）。

### 仿真体配套

`sim/duck_body.py` 新增三件套：`set_gain`（kp_sim = 8×gain/200，直接改执行器 gainprm/biasprm 而不缩 ctrl——恢复时无跳变，`sim/duck_body.py:85-100`）、`set_torque`（三参数清零=卸力）、`push {vx,vy}`（给躯干 qvel 加水平扰动，验收用来把鸭子推倒，`sim/duck_body.py:195-203`）。启动默认 torque on、gain=200（`sim/duck_body.py:74-77`）——模拟真机舵机 RAM 跨进程存活，否则「启动不动」无从验起。read 响应回显 gain/torque（`sim/duck_body.py:152-154`），验收据此断言跌倒卸力。

## 4. 验收

```bash
# 容器内（miniduck-rust，工作目录 /work）
docker compose exec rust cargo build    # 0 警告
docker compose exec rust cargo test
```

实测（本章节写作时复跑）：`test result: ok. 35 passed; 0 failed`——M4 的 18 条 + 新增 17 条（safety 14、io 3）。

```bash
docker compose exec rust bash scripts/accept-m5.sh
```

复跑原文（2026-09-27，六项断言全过）：

```text
== A1. 启动不动 ==
启动姿态 positions 全 0，enabled=false，torque=false  OK
== A1b. enable → 斜坡到 home → 站稳 ==
enable 后 enabled=true, gain=200, 距 home 最大偏差 0.1297 rad  OK
health healthy=true  OK
== A1c. disable → 斜坡回 home → 卸 torque，位置精确等于 home ==
disable 后回到 Held：|positions-home|max=0.00e+00，enabled=false，torque=false  OK
== A2. deadman ==
单次意图后 obs twist vx=0.15000000596046448  OK
失联 1s 后 obs twist=[0,0,0]（deadman 生效）  OK
--secs 2 期间（1.5s 处采样）twist vx 保持 0.15  OK
== B3. push 推倒 → fallen + 卸力（gain 50）==
push 应答: {"ok": true}
捕获 151 帧，首帧 fallen+gain=50 出现在第 76 帧  OK
瘫在地上：body z=-0.11105323444844586（站立约 0.12）  OK
== B4. health 真话：杀 sim ==
healthy=false，原因: ['consecutive read errors: 26']
== B4b. 重启 sim → 重连恢复 ==
重连后 healthy=true  OK
== B5. robot.state 50Hz ==
2 秒收到 101 帧通知
== B6. 行走回归 ==
10 秒前进位移 = 0.775 m（门槛 0.5m）
M5 验收全部通过：启动不动/deadman/跌倒卸力/health 真话/50Hz state/行走回归
```

回归：`scripts/accept-m4.sh`（加了 enable 步骤）同环境复跑——站稳 z=0.1188，10s 位移 0.744m，PASS。

**断言解释**（逐项：这个断言为什么证明了这个性质）

- **A1 启动不动**：启动 1s 后 positions 全 0（FakeIo 的启动姿态）、`enabled=false`、`torque=false`（本进程没命令过 torque）。证明 Held 相位既不移动机器人也不碰 torque 开关——被 supervisor 重启的 daemon 不会把站着的机器人拽倒。
- **A1b/A1c 不对称断言是故意的**：enable 后断言「距 home 最大偏差 < 0.4」（实测 0.13~0.18，D27——FakeIo 速度恒 0 不在训练分布，Driving 闭环有漂移）；disable 后断言「逐位等于 home」（Held 写的是锁存的 DEFAULT_POSITION，无策略参与）。一个是闭环的诚实余量，一个是开环的精确值。
- **A2 三连**：单次意图后 obs 里 twist=0.15（意图进了观测）；失联 1s 后 twist=[0,0,0]（deadman 清的是喂给策略的观测，不是共享状态里的命令本体——证据链完整）；`--secs 2` 期间 1.5s 处采样仍是 0.15（CLI 重发扮演手柄，覆盖 deadman）。
- **B3**：push 1.5 m/s 后 76 帧（约 1.5s）内出现 fallen=true 且 gain=50——重力越线→去抖 200ms→判定翻转→Limp 相位低增益，全链路。z=-0.111 是穿透地板的失真（D26），所以阈值是 z<0.08 而非「躺平高度」。
- **B4**：杀 sim 后 health 报 `consecutive read errors: 26`——读错误计数是循环自己记的账，超过阈值 10 即报病；重启 sim 后 SimIo 下一拍自动重连，healthy 恢复。证明 health 的输入是真实循环状态，不是静态字符串。
- **B5**：2 秒 ≥90 帧（实测 101）证明推送跟着控制拍走（50Hz），不是定时器。
- **B6**：安全层之上行走能力没退化（10s ≥0.5m，实测 0.775m）——防线没有破坏正常功能。

另有两项手动验证（无脚本）：策略文件缺失时 daemon 存活、health 报 "policy unavailable"、enable 后仍抱持不动；`--view-port` MJPEG 画面未破坏。

## 5. 破坏实验

安全机制的测试通过不代表你相信它。亲手把它改坏，看系统怎么坏，再改回来。

### 实验 1：deadman 收紧到 50ms——重发间隔不够时机器人怎么抽

改 `src/safety.rs:52` 的 `deadman: Duration::from_millis(500)` 为 50ms，`cargo build`，起 daemon（FakeIo 即可），enable 后 `drive 0.15 0 --secs 4`，同时抓 state 帧看 obs 里的 twist。实测输出：

```text
捕获 101 帧：vx=0.15 的 49 帧，vx=0 的 52 帧
逐帧 vx 前 40 帧: 0.00 0.00 0.15 0.15 0.15 0.00 0.00 0.15 0.15 0.15 0.00 0.00 ...
```

daemon 日志 5 秒内 29 行 `intents stale 68ms > deadman 50ms (run 1) — twist zeroed`。CLI 每 100ms 重发，保鲜期 50ms——每条意图只覆盖两三拍就过期，速度命令以 3 拍通/2 拍断的节奏抽搐。真机上这就是一顿一顿的步态。**改回 500ms 再 build，实验结束。** 顺手试 100ms：重发间隔恰好等于保鲜期，实测 101 帧里只有 1 帧被清零——竞态边缘，偶发抽一下。deadman 取值必须是重发间隔的数倍，这就是 500ms vs 100ms 的算术。

### 实验 2：往策略输出注入 NaN——看 apply 的拒绝路径

在 `src/control.rs` 的 Driving 分支里给 `apply_action` 的结果加一行 `targets[3] = f64::NAN;`（模拟策略权重损坏），`cargo build`，起 daemon 并 enable。实测：

```text
safety: targets refused: not finite (run 1) — writing hold
safety: targets refused: not finite (run 50) — writing hold
safety: targets refused: not finite (run 100) — writing hold
```

state 帧里 51 帧位置**逐位不变**（50/50 帧对相等，冻结在斜坡终点 home）——每拍 NaN 整包拒绝、改写 hold，机器人冻住而不是猛冲到限位；日志首拍必报、之后每 50 拍一行，5 秒 Driving 只有 3 行。删掉注入行、重新 build，一切恢复。

## 6. 与原版差异

| 项 | 现在 | 收敛 |
|---|---|---|
| D8 | socket chmod 0660 即全部鉴权；侦察确认原版 robotd 无 SO_PEERCRED（那是 configd/updaterd 的模式） | **本章收敛**（按文件权限模型对齐；SO_PEERCRED 留给 M7） |
| D9 | Held 相位抱住启动姿态，绝不调 set_torque；显式 robot.enable 才斜坡回 home | **本章收敛** |
| D11 | health 真实判定：stall 500ms / 45Hz 地板 / 连续读错误 10 / 策略缺失（阈值来自原版 robotd-params） | **本章收敛** |
| D14 | robot.state 改 frame_rx.changed() 驱动 50Hz，载荷增 fallen/enabled/gain/torque | **本章收敛**（残余 D25） |
| D19 | 策略加载失败不退出：policy=None 永远 Held 抱持，health 报 policy unavailable | **本章收敛** |
| D22 | quat 严格长度校验（≠4 即 Err）；copy3 的 panic 隐患顺手改 Err | **本章收敛** |
| D5 | 安全层核心规则齐了（NaN 拒绝/±π 夹紧/deadman/跌倒判定/gain 缓存）；残余：无温度监控、无配置文件面 | 部分收敛，M8 对照 safety.rs 与 robotd.toml 补齐 |
| D12 | trait 补 set_gain/set_torque/imu_ready；残余 reboot/slow_sensors/imu_stale/measures_* | 部分收敛，M8 |
| D23 | 跌倒响应简化：fallen 判定直接触发 Limp，非原版 FallPredictor 陀螺外推预测+Limp/Posing 三态机；原版 limp_fall 默认 OFF，我们默认 ON（velstand 零命令能站住，交接得回） | **本章新开**，M8 裁决 |
| D24 | read 失败跳拍不滑行；原版 COAST_TICKS=3 | **本章新开**，M8 |
| D25 | state 无逐订阅者降频/Lagged 语义（watch latest-wins） | **本章新开**，M8 |
| D26 | sim 推倒后躯干穿透地板（z=-0.111，接触求解器大冲击失真）；验收阈值因此是 z<0.08 | **本章新开**，M8 裁决 |
| D27 | FakeIo 上 Driving 闭环漂移 0.13~0.18 rad（速度恒 0 不在训练分布），enable 阈值 0.4 是实测放宽 | **本章新开**，M8 |

## 7. 常见坑

1. **or-pattern 借用冲突（E0503）**。状态机里「`match &mut phase` 的分支内直接给 `phase` 赋值」会被借用检查器拒掉——match 还借用着它。把 Held/RampDown 合并进一个 or-pattern 守卫、同时在另一支改 phase，也会撞上同一个 E0503【合理推断：移交记录只记了错误号，当时的具体写法未留存】。现行解法两件事：边沿检测用 `matches!` 只做判断不做绑定（`src/control.rs:302-305`）；斜坡完成的转移延后到 match 之后，走 `next_phase`（`src/control.rs:345-346, 410-412`）。
2. **边沿与跌倒转移同拍互覆**。初版转移逻辑里，「Driving 中跌倒」和「同拍 disable」同拍到达时互相覆盖，机器人卡在不该在的相位。改成边沿立即生效、先于跌倒判定（`src/control.rs:285-287`）。状态机 Bug 的典型形态：不是逻辑错，是两个对的规则同拍打架。
3. **FakeIo 上 Driving 不会停在 home**（D27）。enable 后 3 秒，位置距 home 漂 0.13~0.18 rad 且停不下来——FakeIo 速度恒 0 的回声不在策略训练分布里，策略在自己没见过的状态里打转。验收阈值 0.4 是实测放宽（`scripts/accept-m5.sh:87-91`）。别试图在 FakeIo 上把漂移调到 0：那是在调假总线，不是在调鸭子。
4. **推倒后穿地板**（D26）。push 1.5 m/s 后躯干 z=-0.111——站立高度是 0.117，负值意味着穿进了地板（大冲击下接触求解器失真），不是干净躺平。验收阈值因此是 z<0.08 而非躺平高度。想验「瘫软」看 gain=50 比看 z 更可靠。
5. **deadman 只清观测里的 twist，不清共享命令本体**。失联 1s 后再发一条新意图，机器人立刻按新意图走——不需要「解锁」动作。这是设计：stale 的是证据，不是状态。

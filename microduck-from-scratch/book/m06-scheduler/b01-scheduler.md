# 第 7 章 · M6：技能调度器——一份权重一个技能，每拍一次谁来开车的仲裁

> 本章对应里程碑 M6（已验收）。交付物：`src/scheduler.rs`（新建，751 行：Cascade 纯状态机 + Scheduler 持四槽 Policy）、`src/control.rs`（Driving 阶段抽 `driving_tick()`，ControlState 增 skill_edges/mouth，busy 抑制 Limp，FrameSnapshot 增 skill）、`src/policy.rs`（`reset()` 转 pub + `reset_calls` 计数）、`src/model.rs`（`mouth_target()`）、`src/main.rs`（Scheduler::load + robot.do/skills/mouth/head 四个新 RPC）、`src/bin/mini-duckctl.rs`（do/skills/mouth/head 子命令）、`scripts/fetch-m6.sh`、`scripts/accept-m6.sh`。

## 1. 动机

M3 教会鸭子站立，M4 教会它走路，M5 给它装上安全带。但到目前为止，整条控制链路上只有**一份**权重文件：velstand。原版的鸭子会坐下、会起身、会捡地上的东西、会翻跟头、会踢球——每一个技能都是**另一份独立训练的 ONNX 权重**。偏差簿里从 M3 起就挂着 D3：单策略、无调度器。本章收敛它。

先要破除一个直觉：「技能调度」不是「进程管理」或「任务队列」那种调度。这里每个技能是一个神经网络，调度回答的问题每拍只有一个：**这一拍，61 维观测喂给哪份权重，command 块里填什么**。原版的答案朴素得出乎意料——没有独立的调度器对象，就是控制循环里每拍跑一遍的 if-else 级联（`reference/robotd/src/control.rs:483-539`）：

```text
一次性技能（窗口未到期） > ground_pick > sit/rise > walk
```

为什么排在依赖树的这个位置：

- **上游 M3/M4**：得先有一份能跑通的策略和观测管线，才谈得上「多来几份」。
- **上游 M5**：多个技能意味着多个「想开车」的入口，写句柄必须先于多写者收拢进 Safety——否则收拢永远做不成。同时，技能会把机器人主动送进「看起来像跌倒」的姿态（翻跟头中途重力读数必然越线），busy 抑制跌倒转移必须建立在 M5 的 fallen 判定之上。
- **下游 M7/M8**：updaterd 换策略包时要知道机器人当前在干什么（`robot.state` 的 `skill` 字段本章落地）；D20（LSTM 分支）要等一份 recurrent 权重才会被真正触达，而多策略加载是前提。

本章还有一条副线：嘴。15 个关节里嘴是唯一不属于任何策略的（动作只有 14 维，M1 起如此）。`robot.mouth` 给嘴一条**绕过策略**的直达通道——它是观察「策略关节」与「非策略关节」两种治理方式并排运行的最小样本。

## 2. 概念铺垫

### 【背景卡片】edge 与 level：两种意图语义

- **一句话定义**：level（电平）意图是「现在是这个状态，一直保持到我改」；edge（边沿）意图是「刚刚发生了一次这个事件」，说完就消。
- **生活类比**：电灯开关 vs 门铃按钮。开关拨到开，灯一直亮——你不需要每秒拨一次（level）；门铃按一下响一次，按钮自己弹回来，想再响得再按（edge）。
- **本项目四种意图各占一边，而且选择都有理由**：
  - `robot.drive` 的速度命令是 **level + deadman**：客户端持续重发维持「活着的证据」，断粮 500ms 自动清零（M5）。
  - `robot.head` 头姿是 **纯 level 无 deadman**：stale 的头姿无害，stale 的速度会撞墙（M5 防线三的同一论点）。
  - `robot.mouth` 嘴开度是 **纯 level 无超时**：客户端死掉嘴停在那——原版刻意行为（`reference/robotd/src/intents.rs:144-146`），重启不能让嘴猛地合上（"a restart cannot snap a mouth"）。
  - `robot.do` 技能请求是 **edge**：「现在做一遍坐下」说完就算数，不需要也不应该持续重发。坐下之后保持坐姿是**另一个**状态（锁存），不是请求本身在保持。
- **为什么是位掩码而不是队列**：edge 的落地形态是一个 `u32` 位掩码——每个技能一位，请求=置位，控制循环每拍 `mem::take` 取走并清零（`src/control.rs:100-103` 和 `:361`）。同拍到达的两个不同请求都该被看到（队列会排出先后，位掩码不会丢），谁先生效由优先级链决定而不是由到达顺序决定（原版同构，`reference/robotd/src/intents.rs:105-117`）。单测 `skill_edges_are_taken_once`（`src/control.rs:637-648`）钉住「取过一次就没了」——edge 不是 level。

**自测**：如果 `robot.do` 改成 level 语义（客户端要持续重发「坐下」），锁存还需要吗？技能窗口计时会出现什么新问题？

### 【背景卡片】把「进度」塞进速度槽：相位编码与槽位复用

- **问题**：ground_pick（捡东西）是一个 4 秒长的动作，网络需要知道「现在进行到动作的哪一秒」才能输出对应的关节角。怎么告诉它？
- **笨办法及其问题**：直接塞一个线性增长的数 φ（0 到 1）。两个毛病：① φ 在动作首尾之间跳变（0.99 之后归零），网络输入突变；② 这个数搭在哪个已有槽位上？为它改观测布局等于所有网络重训。
- **原版的办法**：把相位 φ 编成单位圆上的角度，塞进 twist 的前两个槽：`twist = [cos(2πφ), sin(2πφ), 0]`（`src/scheduler.rs:271-280`，对齐 `reference/robotd/src/control.rs:497-505`）。圆上相邻两个相位对应的向量总是相近的——0.99 和 0.0 在圆上本来就是邻居，**进度绕一圈回来输入是连续的**，没有跳变。网络不需要会数数，看「指针转到哪」就知道进度——像看钟表而不是看秒表。
- **最小示例**（本机 python3 跑过，就是 `src/scheduler.rs:274-279` 那段公式）：

```text
t=  0 拍  φ=0.000  twist=(+1.000, +0.000, 0)   ← 动作开始
t= 25 拍  φ=0.125  twist=(+0.707, +0.707, 0)
t= 50 拍  φ=0.250  twist=(+0.000, +1.000, 0)   ← 四分之一圈
t=100 拍  φ=0.500  twist=(-1.000, +0.000, 0)   ← 一半
t=140 拍  φ=0.700  twist=(-0.309, -0.951, 0)   ← end_phase，交还 walk
```

（`t` 是 50Hz 拍数，4.0s 周期 = 200 拍；φ=0.7 即第 140 拍交还，`src/scheduler.rs:75-78`。）

- **为什么「乱用」速度槽是安全的**：walk 网络把 twist 幅值当速度意图，幅值恒 1 的相位编码对 walk 是「全速前进」——但**切换发生后喂观测的是另一份权重**。ground_pick 网络训练时的 command 块就长这样（训练环境同款编码），它只在自己分布内的输入上被调用。槽位的含义不是全局常量，是**每份权重各自的训练约定**。同理，坐姿把 posture flag=1 搭在 vx 槽（`src/scheduler.rs:283-288`）：sitstand 网络的训练约定里 vx=1 就是「坐」。这就是为什么「技能 = 换网络 + command 块重编码」，观测的其余 48 维一动不动（原版：`reference/duck-control/src/policy.rs:6-8`，所有网络共享同一个 61 维布局，差异只在 command 块）。

## 3. 实现走读

### Cascade 与 Scheduler：状态机和权重分开装

模块开头的设计注释（`src/scheduler.rs:1-17`）把结构一句话说完：`Cascade` 是纯状态机——优先级、command 重编码、窗口计时，不碰 ONNX；`Scheduler` 在它外面裹上四个槽位的 `Policy` 实例。

**【决策卡片】调度器独立成模块 vs 留在控制循环里当 if-else**

- **决策点**：原版没有调度器对象，调度就是控制循环里一段级联。我们收不收进独立模块？
- **备选**：1. 照抄原版，级联写在 `control.rs` 的 Driving 分支里；2. 抽成 `scheduler.rs`，Cascade 不持有 Policy。
- **选择**：备选 2。理由写在模块文档（`src/scheduler.rs:4-7`）：抽出来之后，优先级链、相位编码、窗口计时这些**纯逻辑**可以在 `cargo test` 里脱离 ONNX Runtime 直接断言——本章 10 条 Cascade 单测（`src/scheduler.rs:441-685`）不需要权重文件就能跑。留在控制循环里的话，每条规则都得拖着真权重和 runtime 才能测。
- **放弃的成本**：多一层透传（`Scheduler::step/handle_requests/advance` 都只是转发给 cascade，`src/scheduler.rs:398-408`）；「选了哪个网」和「哪个网有权重」两个事实分住在 Cascade 和 Scheduler 里，靠构造时钉死一致性（`Scheduler::load` 用各槽加载结果构造 Cascade，`src/scheduler.rs:358-362`）——万一不一致，`net_mut` 直接 panic 而不是静默换网（`src/scheduler.rs:410-423`，注释：走到 None 是 bug，panic 比静默好）。
- **失效边界**：如果 M8 收敛验收要求文件级对齐原版（级联必须住在 control.rs），这层抽象要重新审视；但以「可测试性」换「文件位置」是本项目从 M5 起的一贯取径。

### 槽位与加载：walk 必须，其余可选

`Scheduler` 持四个槽：walk（必须）+ sitstand / ground_pick / 配置技能表（各可选）（`src/scheduler.rs:330-336`）。加载策略分两档（`Scheduler::load`，`src/scheduler.rs:350-370`）：

- **walk 加载失败 = 整个调度器不存在**，进程进「永远 Held 抱持报病」——M5 的 D19 哲学原样延伸。
- **其余槽各自可选**：文件缺失或损坏只让那一个技能不可用，daemon 照活（`load_optional`，`src/scheduler.rs:338-346`）。一台不会翻跟头的鸭子还是鸭子。

配置技能表 `SKILLS` 的 **Vec 顺序就是优先级**：roulade > kick_left（`src/scheduler.rs:60-71`，原版注释原话 "Order is priority, replacing the hardcoded roulade > kick precedence"）。两个细节都抄自原版的坑：kick 的文件名是训练 run 名（`ball_kick_left.onnx`）而非角色名；`robot.skills` 的名单必须把内置两个（ground_pick/sit_toggle）排在配置技能前面——它们不是配置表条目，但必须在名单里，原版实测曾因此拒掉五个手柄绑定里的两个（`src/scheduler.rs:377-391` 注释，`reference/robotd/src/main.rs:4255-4261`）。

### 一拍调度：先交还，再选网，后推进

`Cascade::step`（`src/scheduler.rs:248-310`）每拍三件事，顺序全是论点：

1. **先到期的窗口先交还**（`:249-263`）：剩余拍数归零的技能、φ≥0.7 的 ground_pick、倒计时结束的起身，在本拍开头就摘掉——「到期后的那一拍跑下一个东西，而不是多跑一帧已结束的动作」（`reference/robotd/src/control.rs:437-440`）。
2. **按优先级链选网并重编码 command**（`:265-298`）：技能窗口内 command **全零**（这些网络的训练环境是 zero_command_padding）；ground_pick 喂相位编码；坐姿把客户端命令的 twist 换成 [1,0,0] 但 head/body 保持活值；起身 1.0s 内 twist 全零；否则 walk 透传客户端命令。**客户端的原始命令只在 walk/sit 支路存活**——观测里的 command 块从这一拍起不一定是客户端发的值。
3. **窗口计时在 `advance()` 里推进**（`:313-323`），调用时机是本拍推理和写电机**之后**（`src/control.rs:241-242` 注释：与原版「电机写完后推进相位」同序）。

请求侧 `handle_requests`（`:192-242`）的处理顺序对齐原版：ground_pick → sit_toggle → 配置技能按表序。两个规则值得单看：

- **抢占简化（D29）**：技能/ground_pick 运行中，新的技能/ground_pick 请求一律拒绝（打日志说明原因）。原版的 chain 窗口、unwind 阶段、ground_pick preempt kick 尾部都不做。
- **sit 是锁存不是窗口**（`Sit` 枚举，`:130-137`）：toggle 一次坐、再 toggle 一次起；坐姿无限期保持直到下个 toggle。起身途中再 toggle 拒绝（原版同）。注意坐姿**不算 busy**——坐着的机器人是停驻，不是行进（`busy()`，`:169-173`）；起身中才算。

每条规则都有单测钉住：优先级（同拍 roulade+kick，表序前者赢，`priority_same_tick_requests_first_in_table_wins`）、锁存穿越技能（坐姿中踢一脚，窗口结束**回到坐姿**而不是 walk，`active_skill_outranks_sit_and_returns_to_it`）、不可用技能拒绝（`unavailable_skills_are_refused`）等。

### 换网连续性：无 blending 凭什么不跳

换网络是最容易把机器人「闪一下」的地方：新网络对当前姿态一无所知，第一拍输出可能和上一拍写的目标差很远。原版不做任何动作 blending，靠三件事保证连续（`reference/robotd/src/control.rs:211-214`），本章全对齐：

1. **`last_action` 跨网络共享**：观测里有 14 维「上一拍动作」，它由控制循环持有（`src/control.rs:299`），换网时**不清**——新网络第一拍看到的 last_action 是旧网络最后一拍的输出。因为所有网络输出同一套 14 关节语义，这张「上一拍做了什么」的答卷换谁改卷都有效。
2. **低通锚点跨切换保留**：`previous_targets` 同样由控制循环持有（`src/control.rs:297`），切换拍的新目标照样以上一拍写出值为锚点做 0.7/0.5 低通——切换拍本身就被滤波压着。
3. **换网即 reset 新网络的 LSTM 状态**（`Scheduler::infer`，`src/scheduler.rs:428-438`；`Decision.switched` 标志，`:300-301`）：新网络的 episode 记忆必须从零开始，读到旧网络的记忆比没有记忆更糟。5 个实测权重全是 feedforward（D20），reset 是空操作——所以 `Policy` 加了 `reset_calls` 计数（`src/policy.rs:77-81`），让「切换时 reset 真的发生了」对空操作也可断言（单测 `switching_nets_resets_the_new_network`，`src/scheduler.rs:718-750`）。

这三件事里前两件归控制循环、第三件归调度器，分界线是刻意的：**跨切换要保留的东西不能放在会被换掉的对象里**。

验收层面这条性质被双层断言钉死（`net_switches_never_jump_targets`，`src/control.rs:677-806`），详见验收一节。

### busy 门控：翻跟头不算跌倒

M5 的规则是「Driving 中 fallen → Limp 软倒」。roulade 翻跟头中途，投影重力必然越线 200ms——不干预的话，鸭子翻到一半会被自己的防跌机制卸力拍在地上。本章加门控（`src/control.rs:402-404`）：`scheduler.busy()` 为真时抑制 Driving→Limp 转移。两个边界都留着：

- **fallen 报告不变**：state 推送里的 fallen 照真——门控抑制的是**动作**（进 Limp），不是**事实**（跌倒报告）。验收靠这个区分「没检测到」和「检测到但按住了」。
- **busy 一结束门控即撤**：roulade 窗口结束的下一拍，若还倒着，Limp 立刻接管（gain 50）。实测验收 B5 断的正是这条链。

原版门的是 FallPredictor（陀螺外推预测跌倒），我们门的是 D23 简化版的直接转移——简化登记 D32。

### 嘴：绕过策略的直达通道

嘴（关节 9）不在任何策略的动作空间里，治理方式和 14 个策略关节完全不同：

- **映射**：`robot.mouth` 收 0..1 开度，`mouth_target()` 线性映射到 −5°..+30° 并夹紧，NaN 当 0（闭嘴）（`src/model.rs:35-45`；数值抄自 `reference/duck-control/src/model.rs:62-63`，alpha 沿用 v1.6 量程）。
- **覆写时机**：只在 Driving 阶段、策略写完 14 关节**之后**，用意图直接覆写 `targets[MOUTH_INDEX]`（`src/control.rs:240`）——绕过策略，也绕过低通。斜坡（= M5 定义的"回 home 渐变过程"）/抱持/Limp 期间嘴跟随该阶段自己的目标（「重启不能让嘴猛地合上」，原版同）。低通也明确跳过嘴（`apply_action`，`src/control.rs:147-148`）。
- **可观察的副作用**：enable 进 Driving 的第一拍，嘴从 home 的 0 跳到 −5°（默认意图 0）。这不是 bug，是原版行为——验收 A1 的期望 home 里嘴位就是 −0.0873 而不是 0.0（`scripts/accept-m6.sh:129-131` 注释）。

### RPC 面与 robot.do 的拒绝语义

四个新 RPC（`src/main.rs:263-359`）：`robot.do`（技能请求，写边沿位）、`robot.skills`（名单）、`robot.mouth`（0..1 开度）、`robot.head`（四关节角，无 deadman）。`robot.state` 载荷增 `skill` 字段（`src/main.rs:396`），非 Driving 阶段为 null。

**【决策卡片】robot.do 在哪一层拒绝**

- **决策点**：机器人没法执行技能时（未 enable / 斜坡途中 / 技能不存在 / 正忙着），请求在哪一层被拒。
- **备选**：1. RPC 侧尽力拒绝，能判的都判；2. 全收下来，控制循环里落不了地的静默丢掉（原版对 fallen 的处理：循环里静默丢请求）。
- **选择**：备选 1。`robot.do` 按序判四条：策略没加载、未 enable、还没进 Driving（斜坡途中）、名字不在名单——各自返回 `accepted: false` 加人话原因（`src/main.rs:276-300`）。依据是原版自己的注释：「accepted 然后什么都不发生是最坏的回答」。与原版的一处**刻意差异**：原版在 fallen 时不拒绝（请求收下来在循环里静默丢），我们按 `Stats.driving`（`src/control.rs:64-67`）拒绝——fallen 进 Limp 后 driving=false，`robot.do` 在 RPC 侧就被挡回。
- **放弃的成本**（= 没选备选 2 丢掉了什么）：① 拒绝逻辑单点化——备选 2 下「能不能执行」只在控制循环一处判，备选 1 把拒绝理由拆成两层（RPC 判四条、调度器判 busy），以后新增一种「不能执行」要同时想到两处，漏一处就恰好复现这张卡要消灭的「accepted 但什么都不发生」，从「设计如此」退化成「维护失误」；② RPC 层的解耦——它本来只做协议翻译，现在要向控制循环打听阶段（`Stats.driving` 就是为这张卡加的原子量），以后每加一种阶段都要多想一句「RPC 要不要知道」。另外备选 1 的卖点自己也没买全：busy 状态只活在 Cascade 里，RPC 判不了，所以 do 的应答只是「接受投递」不是「承诺执行」——busy 拒绝仍只留日志，「accepted 然后什么都不发生」没被消灭，只是从 fallen 一种情形缩窄到 busy 一种情形。
- **失效边界**：D29 收敛时（chain/unwind 抢占实现），「忙」的定义变复杂，调度器侧的拒绝理由要随之进应答，届时这张卡重审。与原版的行为差异（fallen 时拒绝 vs 静默丢）登记 D36，M8 裁决。

CLI 配套四个子命令（`src/bin/mini-duckctl.rs:133-177`）：`do <skill>`、`skills`、`mouth <0..1>`、`head <np> <hp> <hy> <hr>`。

## 4. 验收

```bash
# 容器内（miniduck-rust，工作目录 /work）
docker compose exec rust cargo build    # 0 警告
docker compose exec rust cargo test
```

实测：`test result: ok. 49 passed; 0 failed`——M5 的 35 条 + 新增 14 条（scheduler 11：级联规则 10 + 换网 reset 契约 1；control 2：edge 取一次清零 + 切换连续性端到端；model 1：嘴映射端点与夹紧）。

**连续性断言是本章测试的重心**（`net_switches_never_jump_targets`，`src/control.rs:677-806`）。场景：walk（带 twist 意图）→ sit → 坐姿 100 拍 → rise → walk → ground_pick 全程 → kick_left 窗口 → 回 walk，全程逐拍记录写出目标，两层断言：

1. **机制断言（切换拍，精确等式）**：换网那一拍，写出目标必须**逐位等于** `apply_action(新网动作, Some(上拍锚点), 新槽 scale)`。若切换把低通锚点清零，这一拍写的是未滤波的 raw，等式立刻不成立——这是「无 blending 也不跳」的真正机制，用等式而不是阈值断言。
2. **回归阈值（全程，5.0 rad）**：任意相邻两拍任一策略关节 |Δtarget| ≤ 5.0。这个数字不是推导出来的，是实测出来的：该场景在 FakeIo 上全程处于训练分布外（速度恒 0 的回声），策略动作大幅饱和，实测峰值 **4.20 rad**——出现在起身中途（joint 1），**不是切换拍**；切换拍峰值 1.61 rad（ground_pick→walk，joint 4）。5.0 ≈ 1.2× 全程峰值。它拦的是「切换机制坏了导致的全局跳变」这类回归，不证明数学上不可能跳；分布内（sim）该值小一到两个数量级。嘴被排除在断言外：嘴有意图直通无低通，进 Driving 第一拍从 home 跳 −5° 是原版行为。

```bash
docker compose exec rust bash scripts/accept-m6.sh
```

实测关键行（2026-09-28，sim 端口 7804；完整输出见 acceptance.md）：

```text
== 阶段 A（FakeIo）==
启动姿态全 0，enabled=false，skill=null  OK
enabled=true, gain=200, skill=walk, 距 home 最大偏差 …（含嘴 −5°）  OK
skills = ['ground_pick', 'sit_toggle', 'roulade', 'kick_left']（内置两个在名单里）  OK
未 enable 拒绝: the policy is not driving — run mini-duckctl enable  OK
未知技能拒绝: no skill named "backflip"; this robot has ground_pick, sit_toggle, roulade, kick_left  OK
mouth 1.0 → +30°  OK / mouth 0.0 → −5°  OK / mouth 7.0 → 夹紧 +30°  OK
obs[51:55] = [0.1, 0.2, 0.3, 0.4]  OK / 1s 未重发头命令仍保持（head 无 deadman）  OK
== 阶段 B（MuJoCo sim）==
站稳：body z≈0.1166, skill=walk
10 秒前进位移 = 0.674 m（门槛 0.5m）
2 秒收到 101 帧通知
ground_pick：140 帧（~140），twist=单位圆相位编码，无摔倒，结束回 walk  OK
kick_left：25 帧窗口（~25），command 全零，无摔倒，结束回 walk  OK
sit：38 帧 skill=="sit"，twist=[1,0,0]，锁存保持  OK
注：物理上坐下后向后翻倒（第 64 帧 fallen）——已登记偏差 D34，M8 随 D21 裁决
roulade：50 帧窗口（~50），busy 期间 33 帧 fallen 全部 gain=200，结束后 Limp 接管  OK
```

回归：`scripts/accept-m5.sh` 六项全过（行走 0.853m/10s）——调度器进场没有踩坏安全层和行走。

**断言解释**（逐项：这个断言为什么证明了这个性质）

- **A 阶段（FakeIo）**：名单断言证明「内置两个 + 配置技能」的顺序对齐原版 do_names；mouth 三值断言证明映射端点与夹紧（7.0 → +30° 是越界输入被夹，不是报错）；head 两次采样间隔 1s 断言「无 deadman」（对比 M5 的 twist 失联 1s 归零——两种语义并排可验）；`skill` 字段在启动时为 null、enable 后为 "walk"，证明它跟着 Driving 阶段走而不是静态字符串。
- **B3 ground_pick**：140 帧窗口 ≈ 0.7×200 拍（推送 latest-wins 会丢帧，脚本放宽到 100..160，`scripts/accept-m6.sh:241-242`）；前 5 帧 twist 满足 |twist|=1 且首帧 [1,0,0]，证明相位编码进的是观测而不是日志；全程无 fallen + 结束自动回 walk，物理通过。
- **B6 kick_left**：25 帧窗口、窗口内 command 全零（obs[48:55] 七维全 0）、无摔倒、自动回 walk，物理通过。
- **B4 sit（降级断言）**：只断言调度正确性——skill=="sit"、obs twist=[1,0,0]、锁存保持到摔倒为止。**物理上第 64 帧向后翻倒，这是已登记的失败（D34），不是验收遗漏**：脚本注释明写「物理失败已登记 D34」，断言刻意停在调度层。
- **B5 roulade（降级断言）**：窗口精确 50 拍；busy 期间 33 帧 fallen **全部 gain=200**——证明 busy 门控按住了 Limp（fallen 报告照真，动作被抑制）；窗口结束后 gain=50 出现——证明门控随 busy 解除而撤销。物理上 ~0.7s 处摔倒（D35）。

### sit 与 roulade 为什么物理失败：三组对照实验

调度全部验证正确，物理就是过不去。失败的归因不是猜的，本章 coder 做了对照实验（登记在 D34/D21）：

- **kp=8（现状）与 kp=40 + forcerange ±10（执行器强度放宽）倒在同一时刻** → 不是 torque 上限问题。
- **action_scale 减半（动作幅度减半）同样倒** → 不是动作尺度问题。
- 两个方向都排除后，指向 **BAM 执行器动力学缺失**（D21：我们的仿真执行器是 XML 位置伺服，训练用的是 BAM 模型）——姿态类动作（walk/stand/kick 瞬时）对执行器模型不敏感，深蹲和前滚翻这类长杠杆、大力矩动作敏感。

**一处更正（写作时复核原版代码发现）**：本里程碑交接时曾记录「原版 --sim 是无物理回显（reference/robotd/src/main.rs:240）」——**这条记录有误**。main.rs:239-240 的注释挂在 `--fake` 上（"no physics, no falling over, positions that echo back perfectly"），原版的 `--sim` 是真 MuJoCo 物理（`reference/duck-control/src/sim.rs` 模块文档；`reference/docs/design/simulation.md` 明写 "built and in use"，且声称 sitstand 策略在原版仿真里能跑到直立并保持）。真正的事实是：**原版的仿真体（duck-body）在 microduck_rl 仓库，不在我们能看到的浅克隆里**——我们的 `sim/duck_body.py` 是自己手搓的这一半。这个更正反而加强了 D34/D35 的归因：同款 sitstand 权重在原版仿真里能坐能起、在我们的仿真里翻倒，两边的差异集中在执行器模型上【合理推断：原版 duck-body 实现了训练用的 BAM 动力学，此点 M8 随 D21 一并裁决】。

## 5. 破坏实验

调度器的规则测试通过不代表你相信它。亲手把它改坏，看系统怎么坏，再改回来。以下两个实验改完记得 `cargo build` 并**改回原值重新 build**。

### 实验 1：对调 SKILLS 表序——同拍优先级翻转

`src/scheduler.rs:60-71` 把 roulade 与 kick_left 两行对调，跑 `cargo test priority_same_tick_requests_first_in_table_wins`——预期失败：同拍请求两个技能时 kick_left 先启动（表序即优先级），断言 `Net::Skill(0)` 不再成立。这个测试的存在意义就是：优先级不是写在 if-else 里的顺序，是写在数据里的顺序，改数据就该改行为。

### 实验 2：GROUND_PICK_END_TICKS 改小——交还提前

`src/scheduler.rs:78` 把 140 改成 70（φ=0.35 交还），`cargo build` 后起 sim 跑 `do ground_pick`，抓 state 帧：预期 ground_pick 帧数从 ~140 掉到 ~70，鸭子弯腰弯到一半就站起来回 walk——**动作被腰斩就是 ground_pick 提前交还的样子**。这也顺带演示了为什么原版把 end_phase 做成可调参数而不是编译进权重：动作的「做到哪算完」是运行时决策。

## 6. 与原版差异

| 项 | 现在 | 收敛 |
|---|---|---|
| D3 | 单策略无调度器 → scheduler.rs 优先级链（技能 > ground_pick > sit/rise > walk），command 重编码、拍数窗口、换网 LSTM reset 全部对齐原版 | **本章收敛** |
| D20 | 新下的 4 个策略实测全部 feedforward（onnx 探针确认），LSTM 分支仍无运行时覆盖；换网 reset 调用本身已被 reset_calls 计数断言 | 保持待收敛，首次加载 recurrent 文件时核销 |
| D28 | 无 Mode(Walk/Roller) 与 roller/roller_crouch 槽（v5 权重仓库里有，未下载未接线） | **本章新开**，M8 |
| D29 | 抢占简化：运行中一律拒绝；无 chain 窗口（roulade chain=true 未实现）/unwind 阶段；ground_pick 固定跑满 end_phase=0.7 | **本章新开**，M8 |
| D30 | 无策略热换/carry_over（robot.set_policy / mode_switch 不实现） | **本章新开**，M8 |
| D31 | 嘴无特雷门/合唱覆写层；robot.mouth 仅 Driving 阶段生效（与原版一致） | **本章新开**，M8 |
| D32 | busy 直接抑制 Driving→Limp；原版门的是 FallPredictor（我们的跌倒响应本身是 D23 简化版）。fallen 报告不变 | **本章新开**，M8 |
| D33 | 无 stand 槽与 will_stand 幅值选网（原版 twist 幅值低于阈值且有 stand 网络时选 Stand 槽，standing_gain_ratio=0.8）；velstand 挂 walk 槽同时覆盖站立 | **本章新开**，M8 |
| D34 | sit 调度/重编码/锁存全部正确，物理上深蹲后向后翻倒；kp=40+forcerange 与 action_scale 减半对照实验排除 torque 上限与尺度，指向 BAM 缺失（D21）；sim 验收降级为调度断言 | **本章新开**，M8 随 D21 裁决 |
| D35 | roulade 窗口/busy 门控全部正确，物理上前滚翻 ~0.7s 处摔倒；sim 验收降级为调度+busy 断言。kick_left 与 ground_pick 物理通过 | **本章新开**，M8 随 D21 裁决 |

另有一处不算偏差的**刻意差异**：原版 robot.do 在 fallen 时不拒绝（请求收下来循环里静默丢），我们按 `Stats.driving` 在 RPC 侧拒绝——依据是原版自己的注释「accepted 然后什么都不发生是最坏的回答」。

## 7. 常见坑

1. **z 达标 ≠ 可以 do**。斜坡中途躯干高度就过了 z>0.08（RampUp 在往上走），但 `robot.do` 只在 Driving 阶段接受——脚本若只看 z 就发 do，会被 `the robot is still going to its home pose` 拒掉。accept-m6.sh 的 `fresh_sim` 因此等的是 `skill=="walk"` 而不是 z（`scripts/accept-m6.sh:81-90`）。手测时同理：enable 后给 3 秒再 do。
2. **pkill 自杀**。验收脚本的 cleanup 用 `pkill -f "duck_body[.]py"`（`scripts/accept-m6.sh:35`）——方括号让模式不匹配 pkill 自己的命令行。但你自己写的探针脚本若命令行里含 "duck_body.py" 字样（比如 `python3 probe.py duck_body.py ...`），cleanup 会把它一起杀掉——实测踩过：探针进程莫名消失，就是撞在这条 pkill 上。探针脚本命名和参数里避开 `duck_body.py` 与 `miniduckd` 字样。
3. **窗口帧数别按精确值断言**。50Hz 推送是 latest-wins，订阅端慢一拍就丢帧——140 拍的 ground_pick 抓到 100~140 帧都正常。accept-m6.sh 的窗口断言全部用了区间（100..160 / 15..45 / 35..70）。反过来，**计时精确性由 Cascade 单测保证**（拍数逐拍断言，不经过推送管道），端到端只验量级——两层各验各的。
4. **enable 后嘴跳到 −5° 不是回归**。Driving 阶段嘴被意图覆写，默认意图 0 → −5°；A1 的期望 home 里嘴位是 −0.0873 不是 0.0（`scripts/accept-m6.sh:129-131`）。拿着 M5 的期望表对 M6 的输出会误报。
5. **do 的 accepted=true 不是「会执行」**。RPC 侧只判「配不配收」（未 enable/斜坡中/名字不存在），busy 拒绝在控制循环下一拍才发生、只留日志（`src/control.rs:434-436` 还有一层非 Driving 丢弃兜底）。要确认技能真跑了，看 state 帧的 `skill` 字段。
6. **起身途中 toggle 无效是特性**。Sit::Rising 期间的 sit_toggle 请求被拒（对齐原版）；CLI 连发两次 `do sit_toggle` 间隔小于 1 秒，第二次静默无效——看 daemon 日志的 `scheduler: sit_toggle refused: already rising`。

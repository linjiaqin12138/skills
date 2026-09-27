# 第 5 章 · M4：仿真行走——策略第一次驱动真实物理

> 本章对应里程碑 M4（已验收）。交付物：`sim/duck_body.py`（MuJoCo TCP 仿真体）、`sim/assets/scene.xml`（场景）、`src/io.rs` 的 `SimIo`（`RobotIo` 的第三个实现）、`src/main.rs` 的 `--sim` 开关与 `robot.drive`、`src/bin/mini-duckctl.rs` 的 `drive` 子命令、一键验收 `scripts/accept-m4.sh`。

## 1. 动机

M3 的闭环是假的。FakeIo 是「位置回声」：`write` 什么，`read` 就原样还什么（`src/io.rs:281-286`）。策略的输出写下去，下一拍读回来的一定是它——位置永远完美，IMU 永远是 `[0,0,-1]`。这个环里没有任何东西检验「这个动作在物理上是否能让机器人站住」。

M4 给鸭子换一个真身体：MuJoCo 仿真体，有质量、有接触、有摩擦，写错角度会摔倒。策略第一次面对真实物理闭环。在依赖树里：

- **上游**：M3 的策略推理（velstand 权重、action_scale、低通）原样保留，本章一行不改。
- **本章**：`SimIo` 经 TCP+JSON 驱动仿真体；新增 `robot.drive` 让观测里的 command 块取真值（D18 收敛），鸭子按 `vx` 走起来。
- **下游**：M5 安全层需要真物理才能测跌倒——没有重力，「跌倒」这个词没有意义。

## 2. 概念铺垫

### 【背景卡片】MuJoCo / MJCF：物理仿真器和它的模型文件

- **一句话定义**：MuJoCo 是物理仿真引擎：给它一个模型（质量、关节、碰撞几何、执行器），每调一次 `mj_step` 它把状态按牛顿力学推进一步。MJCF 是它的模型文件格式——一个 XML，描述「身体长什么样」。
- **为什么需要**：没有硬件的情况下，这是唯一能诚实回答「这个关节目标会不会让鸭子摔倒」的东西。它算出的是 `qpos`（各关节角）、`qvel`（角速度）、躯干在世界系的位置和姿态——正好就是 `Sensors` 要的形状。
- **和 TS/Node 的差异**：可类比在 Node 里跑一个 `step()` 循环驱动一个游戏物理世界（Matter.js 那种），差异点：(1) MJCF 是声明式 XML，不是代码里 new 出来的对象图；(2) 模型（`MjModel`）与状态（`MjData`）分离——模型编译一次，状态可以任意重置；(3) 它不会自己画画面，默认 headless，想看动画要自己接渲染。
- **最小示例**（已在本仓库容器里跑过，`docs/concepts/assets/pendulum.py`）：一个 20 行的脚本，MJCF 定义一个单摆，从 30° 自由落下，每 0.1s 打印摆角：

```python
import math, mujoco

XML = """
<mujoco>
  <option timestep="0.01"/>
  <worldbody>
    <body name="pendulum" pos="0 0 1">
      <joint name="hinge" type="hinge" axis="0 1 0"/>
      <geom type="capsule" fromto="0 0 0 0 0 -0.5" size="0.02" mass="1"/>
    </body>
  </worldbody>
</mujoco>
"""

m = mujoco.MjModel.from_xml_string(XML)   # MJCF 字符串 → 编译成模型
d = mujoco.MjData(m)                       # 模型的动态状态（角度、速度、时间）
d.qpos[0] = math.radians(30)               # 初始：从竖直方向偏 30°

for i in range(11):
    mujoco.mj_step(m, d, nstep=10)         # 每 10 小步 × 0.01s = 前进 0.1s
    print(f"t={d.time:.1f}s  angle={math.degrees(d.qpos[0]):+.1f}deg")
```

容器内实测（`docker compose exec rust python3 docs/concepts/assets/pendulum.py`）：

```text
t=0.1s  angle=+25.6deg
t=0.2s  angle=+14.3deg
t=0.3s  angle=-1.0deg
t=0.4s  angle=-16.0deg
t=0.5s  angle=-26.6deg
t=0.6s  angle=-30.0deg
t=0.7s  angle=-25.4deg
t=0.8s  angle=-13.9deg
t=0.9s  angle=+1.4deg
t=1.0s  angle=+16.3deg
t=1.1s  angle=+26.7deg
```

30° → 摆过最低点 → 荡到另一侧 −30° → 回来：这就是 `mj_step` 干的事。本项目的鸭子模型（`sim/assets/scene.xml`）与这个单摆同构，只是 body/joint 有 14 个、多了地板和执行器。**回到项目**：`sim/duck_body.py:41-64` 编译 scene.xml、按 1ms 步长小步推进；`sim/duck_body.py:73-91` 从 `MjData` 里取传感数据。

**自测**：把上面 XML 里的 `timestep` 改成 `0.5` 再跑，输出会变成什么？为什么？（提示：见本章常见坑 2。）

### TCP + NDJSON：不是 WebSocket

协议形状对前端读者不新——逐行 JSON 请求/响应，像 JSON-RPC over WebSocket。差异点值得说清：

- **无帧头、无掩码、无握手帧**：裸 TCP 字节流，消息边界是自己按 `\n` 切出来的（NDJSON）。WebSocket 帮你做了分帧，这里没有；所以坏帧之后无法重新同步，只能断开重来（`src/io.rs:207-209`，原版 sim.rs:207-211 有同样的论述）。
- **全双工但按串行用**：TCP 本身全双工，这里却是一问一答的请求-响应——控制循环每拍 `read` 等应答、`write` 等应答，没有任何异步推送。原因就是节拍要由控制侧驱动（见实现走读）。

## 3. 实现走读

### SimIo：为什么是 TCP、为什么是 JSON

`SimIo`（`src/io.rs:132-266`）是 `RobotIo` 的第三个实现（前两个：FakeIo、将来 M8 的真实总线）。设计理由直接照抄原版 `reference/duck-control/src/sim.rs` 开头的模块论述（**以下为原版原话的转述**）：

- **为什么 TCP 不用 unix socket**（sim.rs:9-14）：两个理由，都是踩过的坑而不是假设。一，unix socket 路径长上限 `SUN_LEN` 约 108 字节，临时目录分分钟超掉；二，仿真体必须能从守护进程所在的容器/虚拟机**外面**访问到（Linux 容器、macOS 上的 Linux VM），端口能跨这些边界，socket 路径不能。
- **为什么 JSON**（sim.rs:16-22）：一拍十五进十五出，约 1KB，50 Hz 下才 50 KB/s——这点带宽换来的是「能用 `nc` 直接读一帧、能用 20 行 Python 写仿真体的另一半」。备选是两个仓库两种语言共享一个 packed struct，offset 错一位就是静默错误，原版项目在这种事情上丢过「天」级的时间。
- **仿真体消失怎么办**（sim.rs:24-30）：MuJoCo 改模型要重编译重启，仿真体消失是常态。所以任何错误都是「丢连接、返回 Err，下一拍重连」，不要后台重连线程——**控制循环本身就是重试定时器**。本仓库的 SimIo 逐条对应：`connect` 只在需要时建立（`src/io.rs:146-181`，守护进程先起、仿真体后上也行），`call` 出错就 `self.link = None`（`src/io.rs:207-209`）。

两个传输层细节：`TCP_NODELAY` 必须开——Nagle 会把小包攒最多 ~40ms（两拍），每拍都变成「像仿真体慢」的超时事故（`src/io.rs:156-157`，原版 sim.rs:169-172 同样强调）。单次请求超时 200ms：比一拍 20ms 宽得多（接触密集的步进偶尔迟到不算坏），又短到不会拖死控制循环（`src/io.rs:128-130`）。

### duck_body.py：一拍 20ms 怎么映射，IMU 怎么解算

**节拍由控制侧驱动。** 仿真体不自己跑墙钟——每次收到 `read` 才 `step_tick()` 步进一个控制拍（20ms = 20 × 1ms 物理步，`sim/duck_body.py:28,46,62-64,109-112`）。这是刻意的：控制循环暂停，仿真时间也停；墙钟抖动不会变成仿真时间漂移。代价是 `read` 有副作用（推进时间），`body` op 因此存在——只查躯干位置不步进，给验收脚本用（`sim/duck_body.py:116-119`）。

**IMU 在仿真侧解算。** gyro 直接取 MJCF 的 gyro 传感器；gravity 由躯干四元数把世界重力旋进躯干系（`sim/duck_body.py:75-79`）。四元数是 MuJoCo 的 wxyz 序，与控制侧 `ImuData` 的约定一致。单位在线上就是机器人自己的单位：弧度、rad/s——「仿真侧做换算」也是原版 sim.rs:79-84 的论述：MuJoCo 最懂自己的关节序和坐标系，在 Rust 侧再翻译一次只是第二个会漂移的真相源。

**总线序 ↔ 关节序。** 控制侧永远是 15 维总线序（含嘴，index 9）；仿真体只有 14 个关节。write 时丢嘴（`sim/duck_body.py:93-99`），read 时嘴位补 0（`sim/duck_body.py:66-71`）。这条映射 M3 的 `scatter_action` 里已经存在，本章只是延伸到了线上。

**起步不是瘫软。** 启动时 `mj_resetDataKeyframe` 到 STAND keyframe（= `model.rs` 的 `DEFAULT_POSITION`），并且立刻 `data.ctrl[:] = qpos` 挂上力矩（`sim/duck_body.py:56-60`）——真机的舵机一直挂着，仿真从瘫软开始会在控制循环第一拍之前就摔地上。

### robot.drive：命令链路

```
mini-duckctl drive 0.15 0 --secs 10
  → JSON-RPC robot.drive {vx:0.15, vyaw:0}      （mini-duckctl.rs:67-68）
  → main.rs serve(): 校验有限数 → 写 SharedCommand.twist（main.rs:142-160）
  → 控制循环下一拍：command.lock() 取真值，组进 obs 的 command 块（control.rs:189-197）
  → 策略读到 vx=0.15（obs.rs:105 OFF_TWIST），开始迈步
  → --secs 10 到点，CLI 再发一条 {vx:0, vyaw:0} 停车（mini-duckctl.rs:69-73）
```

命令语义（twist 是躯干系速度、零是名义站姿）见 Q16；动作与关节的对应关系见 Q18——本章不改语义，只是把 command 从恒 0 变成有入口。这是**第一个 mutating 调用**：注意 main.rs:141 的注释，SO_PEERCRED 校验按偏差簿 D8 留给 M5/M7，本章不做。

`SharedCommand` 用 `Mutex` 不用 watch（`src/control.rs:46-53`）：命令不是帧流，「最近一条为准」且不许丢，15 字节锁内拷出来最直白。

### 决策卡片

**【决策卡片】仿真体自写 vs 下载 microduck_rl 的 duck-body**

- **决策点**：仿真体从哪来。
- **备选**：1. 自写 `sim/duck_body.py`（160 行 Python）；2. 去 pollen-robotics/microduck_rl 仓库下载训练用的仿真体。
- **选择**：备选 1。带 geom 的训练资产**未随本仓库发布**（reference/kinematics/assets/alpha/robot_walk.xml 实测 0 个 geom、0 个执行器，只有运动学骨架），microduck_rl 是另一个仓库，引它就引入一条本教程无法复现的外部依赖。自写的另一面是收益：160 行全在教程仓库里，每一行都可讲。
- **放弃的成本**：仿真与训练环境不完全一致（执行器模型、碰撞几何都是近似，D21）。sim-to-sim 的残余差异要等 M8 裁决——届时如果拿到 microduck_rl 的 BAM 场景，应重测行走。
- **失效边界**：将来验收目标是「迁移到真机」而不只是「策略闭环正确」时，近似执行器模型的结论不再够用。

**【决策卡片】协议自定 vs 直接对齐原版帧格式**

- **决策点**：TCP 上的帧格式自己定，还是逐字节对齐原版 `sim.rs`。
- **备选**：1. 自定 NDJSON（op 标签帧形状对齐原版，细节自定：read 响应多带 `body_pos`/`sim_time` 供验收，write 无显式 ack 字段约定）；2. 逐字段复刻原版 Request/SensorFrame（包括 `gain`/`torque`/`slow` 这些本里程碑用不到的 op）。
- **选择**：备选 1，记 D4，M8 对齐。形状（op 标签、hello 带 protocol 版本号和关节数、一问一答）照搬——这些是设计决策；字段名细节自定——原版协议要服务真机部署的全部功能面，现在抄一半反而四不像。
- **放弃的成本**：`duck_body.py` 不能直接喂给原版 `RemoteIo` 用（反之亦然）。验收脚本依赖的 `body` op 原版没有。
- **失效边界**：M8 若决定让两套协议互通，本章所有帧名都要过一遍。

**【决策卡片】位置伺服 + 扫参 vs 复刻 BAM 执行器模型**

- **决策点**：仿真里的舵机怎么建模。
- **备选**：1. MuJoCo 自带 `<position>` 执行器（kp/kv 位置伺服）+ 扫参选定数值；2. 复刻 microduck_rl 训练用的 BAM（Berkeley Humanoid Actuator Model）——带减速箱动力学和摩擦特性的舵机模型。
- **选择**：备选 1。BAM 模型的实现和参数在 microduck_rl 仓库，不可得；而 M4 的目标是「策略能闭环走起来」，不是「精确复现舵机动力学」。扫参 6 组（damping × kv）选定 damping=0.03 + kv=0.25，实测步速 0.14~0.15 m/s ≈ 指令 0.15。
- **放弃的成本**：armature/damping/frictionloss 这三个数是「凑到能走」的量级近似，不是从 XL330 数据手册逐项推出的（armature=0.001 有物理依据，见常见坑 2；damping 没有）。D21 登记在册。
- **失效边界**：策略若换成依赖舵机响应延迟/死区特性的权重，近似模型可能让「仿真能走」和「真机能走」脱钩。

## 4. 验收

```bash
# 容器内（miniduck-rust，工作目录 /work）
docker compose exec rust cargo test
```

18 passed（M3 的 15 条 + 3 条新 SimIo 单测：传感帧解析、断线重连、仿真体拒绝即错误）。

```bash
docker compose exec rust bash scripts/accept-m4.sh
```

主 agent 复跑原文（三遍）：

```text
== 4. 站稳（3 秒，插值 2s + 策略闭环 1s）==
站稳后 body(x y z t): 0.1006 -0.0259 0.1256 5.0
== 5. drive vx=0.15 持续 10 秒 ==
drive 后 body(x y z): 1.5528 0.4031 0.1295
== 结果：10 秒前进位移 = 1.452 m（门槛 0.5m）==
PASS

第二遍：1.427 m  PASS
第三遍：1.425 m  PASS
```

另验证：`--sim` 与 FakeIo 两种后端下 health / state / subscribe 输出逐字段一致——RobotIo 接缝之上，控制循环和 IPC 层分不出差别。

**断言解释**

- **站稳漂移 0.1m / 5s**：起步插值从 keyframe 的 z=0.12 悬空高度沉到真实站立，x 从 0 漂到 0.10 属正常沉降；5 秒内躯干 z≈0.126 说明没倒（脚本断言 z ≥ 0.08）。
- **为什么位移门槛是 0.5m**：vx=0.15 × 10s 理论位移 1.5m。实测 1.43~1.45m，步速 0.14~0.15 m/s 与指令吻合。0.5m 是「确实在走而不是原地抖」的下限——原地抖动的躯干 x 会在 ±几厘米内振荡，不可能累计出 0.5m。
- **三遍复跑**：MuJoCo 是确定性的，但 tokio 节拍与仿真节拍是异步对齐的（read 才步进），插值起点的物理沉降每次略有差异；三遍 1.425~1.452m 说明结果对启动时序不敏感。

## 5. 与原版差异

| 项 | 现在 | 收敛 |
|---|---|---|
| D18 | command 恒默认零 | **本里程碑收敛**：robot.drive → SharedCommand → obs command 块 |
| D4 | 仿真协议帧细节自定（形状对齐原版 sim.rs；read 多带 body_pos/sim_time） | M8 对齐 PROTOCOL 握手 + 帧格式 |
| D21 | 仿真执行器是 MuJoCo 位置伺服（kp=8 kv=0.25）+ 关节默认 armature/damping，非训练用 BAM；碰撞几何近似 | M8 裁决（或换 microduck_rl 的 BAM 场景重测） |
| D8 | robot.drive 是第一个 mutating 调用，SO_PEERCRED 校验未做 | M5/M7（按偏差簿，本章不做） |

## 6. 常见坑

1. **原版 `robot_walk.xml` 没有 geom 也没有执行器。** 它是纯运动学骨架（body/inertial/site），给训练管线包外层场景用的；带 geom 的训练资产在 microduck_rl 仓库，未随本仓库发布。裸用它会穿地板 + QACC NaN。`sim/assets/robot_walk.xml` 里的 5 个 geom 是教程自写的躯干/足部碰撞盒近似（D21）。
2. **QACC NaN：位置伺服的数值稳定域。** 鸭子关节惯量在 1e-5 量级，kp=8 的位置伺服在 dt=0.002 下超出积分器稳定域，加速度算出 NaN。解法双管齐下：加 `armature=0.001`——这不是纯调参，XL330 经 288:1 减速箱折算到关节侧的转子惯量就是这个量级（BAM 模型捕捉的正是它，原资产省略了）；再把步长降到 1ms（`scene.xml:9,13-15`）。推论：上面自测题里 timestep=0.5 的单摆，输出是发散的垃圾。
3. **MuJoCo autoreset 默认开启。** 失稳后它静默把状态清零、时钟归零——调试时表现为「时间倒流」，鸭子摔了下一秒又站回原位。`duck_body.py:45` 关掉 `mjDSBL_AUTORESET`：摔倒就是摔倒，验收看的是真实位移。
4. **执行器阻尼极其敏感。** damping=0.1 整机冻住（策略不动点、vx 无响应）；0.02 起步即摔。扫参 6 组合后选定 damping=0.03 + kv=0.25。改这两个数之前先想清楚你在验证什么。
5. **容器里没有 pip。** `python3 -m pip` 不存在；先 `curl get-pip.py` 再 `python3 get-pip.py --break-system-packages`，然后 `python3 -m pip install --break-system-packages mujoco`（`accept-m4.sh:27-31` 会检查并提示）。
6. **`pkill -f duck_body.py` 会杀掉自己。** 执行验收脚本的 shell 命令行里就含这个字符串。脚本用 `duck_body[.]py` 规避——`[.]` 匹配一个 `.`，但脚本自身的命令行里是 `[.]`，不匹配（`accept-m4.sh:18`）。

## 7. 看画面：MJPEG 网页 viewer（验收后补）

仿真默认只有数字。`sim/duck_body.py --view-port 7802` 会起一个 MJPEG-over-HTTP 画面：浏览器开 `http://localhost:7802/`（compose 已映射 7802），跟踪相机侧面跟拍，鸭子走远了镜头跟着走。

- **为什么 MJPEG 而不是 MuJoCo 自带 viewer**：容器里没有显示器也没有 X，自带 viewer（glfw 窗口）开不了；而浏览器一定有。渲染是 osmesa 软件离屏渲染（`MUJOCO_GL=osmesa`，必须在 import mujoco 之前设），物理线程与渲染线程共用一把锁，否则画面撕在半拍上。
- **依赖已打进镜像**：mujoco / pillow / libosmesa6 在 `docker/Dockerfile` 里，容器重建不丢。修改 Dockerfile 后 `docker compose up -d --build` 会重建容器，`target/`（bind mount）与 `/cargo`（命名卷）不受影响。
- 只看不动也可以：没有客户端连接时物理不步进（节拍由控制侧驱动），画面停在 keyframe。

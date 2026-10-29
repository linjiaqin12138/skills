---
template: sheet
title: M9c Bite 2：相对 odometry（第 26 章）
subtitle: 双足几何估相对位移：机器人自己估出走多远、转到哪
source: clone-from-scratch-tutor · 第 26 章
---

## 问题

给一条向前走 10 秒的命令，策略是开环的：可能走成弧线，也可能偏出几十厘米。走完之后，除了看有没有摔倒，说不出偏了多少。

仿真里这个问题看起来假：MuJoCo 直接给躯干真值（`body_pos` / 速度）。**训练侧奖励也确实用这份特权真值**——跟踪线速度、角速度、直立等，不必也不该拿几何里程计去替代。本课要解决的不是「仿真里怎么学」，而是：**真机没有 `body_pos`**。关节角 + IMU 之外没有全局定位；要在真机上回答「相对起点走了多远」，只能用和仿真同一条软件估计路径。

所以里程计不是强化学习的输入，也不是仿真训练的前提。它是 robotd 上的**机载相对位姿估计**。仿真与真机共用关节角和 IMU，才能拿「自以为走了多远」对拍真值、做诊断；以后再经 `robot.state` 露出（D50，M10）。

本课先把估计器接进控制循环；估计只进内部诊断，线上载荷仍不含 odom。通用 kinematics crate 也不在本课（D55，M10）。

## 背景

**FK（正向运动学）。** 本课约定：已知关节角，就能推出脚（site）在空间里的位置与朝向；估计器每拍用它得到两只脚。本课不用 IK。推导与几何细节想深入时再读 [鸭子走到哪了？](odometry-blog-zh.html)（[英文原文](https://pollen-robotics.com/microduck/blog/microduck-odometry/)）。

**水平位置靠腿，不靠积加速度。** 本课只向 IMU 要姿态（四元数）；XY 来自鞋底锚点反推。姿态怎么存、怎么读 yaw，见 [四元数](../concepts/quaternion.html)。

**鞋底上总有一点先当「钉在地上」。** 那一点叫锚点。本课先接受：锚点世界位置、腿几何、IMU 姿态合在一起，可以估出躯干走了多远；换脚时换锚。公式与动画在上面的博客里，课文里不当入场券。

## 设计

![M9c 当前架构：miniduckd 内新增 impl/src/odometry.rs，无新进程、RPC、网络边与依赖](arch.svg)

每拍同一条路径：

```flow TB
[新样本] -> [两脚 site 位姿]
两脚 site 位姿 -> [由当前锚点投影躯干]
由当前锚点投影躯干 -> [扫描 32 个鞋底点]
扫描 32 个鞋底点 -> {有候选低于 10mm}
{有候选低于 10mm} -> [清空 pending] : 没有
{有候选低于 10mm} -> [同脚加一拍] : 同脚
{有候选低于 10mm} -> [换脚重开候选] : 换脚
同脚加一拍 -> {连续两拍}
换脚重开候选 -> {连续两拍}
{连续两拍} -> [换锚并重投影] : 是
{连续两拍} -> [输出 position 与 yaw] : 否
换锚并重投影 -> [输出 position 与 yaw]
```

要点：

- 只实现估计要用的两条腿链（局部两足 FK），不拉通用 kinematics。
- 每脚 16 个鞋底点（ALPHA16），共 32 点投到世界系；世界 z 低于 10mm 的最低点才有资格。
- 同一脚连续两拍才换锚（挡 FK/IMU 单拍毛刺）；pending 记住候选**首次出现**时的局部点与世界 XY（换锚时位置连续，不是把估计降到 25 Hz）。
- 没有 reset：原点由首次更新的左脚 site 定死。
- 估计跨相位存活：滑行拍、盲拍、disable、scheduler reset、跌倒恢复都不清零。
- yaw 直接从归一化 IMU 四元数读出；估计进 `FrameSnapshot`，RPC 载荷不变。

聚焦模块见 [odometry 子图](../arch/odometry.svg)。[本课架构差分](figures/arch-diff.svg)。

完整取舍见[决定 0007](../../decisions/0007-m9c-odometry.md)。三句结论：局部两足 FK，不依赖上游 crate（D55 落地时再替换）；原点首帧定死、跌倒也不重选；线上 `robot.state` 不加 odom，留给 M10。没有新增进程、RPC 方法、网络边或依赖。

仿真训练继续用 MuJoCo 真值算奖励。本模块不进观测向量，也不改奖励公式。

## 代码领读

`src=` 只能抽连续行段，中间跳行要用手写 sketch（title 带 `sketch`）写 `// ...`。下面示意对齐真码；行号指向 `impl/`。

### 从控制循环往下看

估计器循环外创建一次；每拍只有新鲜且 IMU 就绪才 `update`（`control.rs:448-451`、`48-52`、调用在 `:525`）：

```rust title="control.rs · sketch"
// ... 相位 / coast / 读传感器 ...
let mut odometry = Odometry::alpha(); // 循环外一次；跌倒 / reset 不重建
// ...
update_odometry(&mut odometry, &sensors, was_fresh, safety.imu_ready());
// ...
// FrameSnapshot { ..., odometry: odometry.snapshot() }  // 内部诊断，不进 RPC

fn update_odometry(..., was_fresh: bool, imu_ready: bool) {
    if was_fresh && imu_ready {
        odometry.update(&sensors.positions, sensors.imu.quat);
    }
}
```

### 一拍 `update`：设计顺序

两脚 FK → 定原点（仅首帧）→ 投影 → 找更低鞋底点 → 两拍确认换锚 → 读 yaw。FK 与 ALPHA16 扫描内部用 `// ...` 跳过（`odometry.rs:315-328`）：

```rust title="odometry.rs · update · sketch"
pub fn update(&mut self, joints: &[f64; NUM_JOINTS], quat_wxyz: [f64; 4]) {
    let rotation = Quat::from_wxyz(quat_wxyz).normalized();
    let feet = [foot_pose(joints, LEFT), foot_pose(joints, RIGHT)]; // FK：细节见博客
    // ... 首帧用左脚 site 钉相对原点 ...
    self.reproject(rotation, &feet);
    let candidate = self.lowest_candidate(rotation, &feet); // 扫 32 点，z<10mm ...
    self.update_pending(candidate, rotation, &feet);
    self.yaw = rotation.yaw(); // 读 IMU，不积陀螺
}
```

### 换锚与投影

同一脚连续两拍才换；确认用 pending **首次**冻结的 `world_xy`（`odometry.rs:347-356`）。投影对应 $p = a - R\,c$（证明见博客，`362-369`）：

```rust title="odometry.rs · pending / reproject · sketch"
// update_pending：
// ... 同脚加 ticks / 换脚重开 / 无候选则清空 ...
if pending.ticks >= SWITCH_CONFIRM_TICKS {
    let confirmed = self.pending.take().unwrap();
    self.anchor_foot = confirmed.foot;
    self.anchor_local = confirmed.local;
    self.anchor_xy = confirmed.world_xy; // 冻结的首次 XY，不是确认当拍
    // ...
    self.reproject(rotation, feet);
}

fn reproject(&mut self, rotation: Quat, feet: &[Pose; 2]) {
    let contact_in_trunk = feet[self.anchor_foot].point(self.anchor_local); // c
    let contact = rotation.rotate(contact_in_trunk); // R c
    self.position = [
        self.anchor_xy[0] - contact[0], // a - R c
        self.anchor_xy[1] - contact[1],
        -contact[2],
    ];
}
```

**「毛刺」**：某一拍最低鞋底点突然像假落地（世界 z 偶发低于 10mm），下一拍又没了。多半来自关节角噪声经 FK 抖几毫米，或 IMU 姿态抖一点。站立相真接触点本该稳；这种单拍、不可持续的假最低点就是毛刺。两拍约 40 ms：远短于一步站立相，又远长于一次毛刺。没有两拍确认，单拍毛刺会在站立相中途乱搬锚点，估计变成随机游走。

**为何用首次候选的 XY，不是确认当拍的。** 换锚时 $p=a-Rc$。若用确认当拍的 `world_xy` 当新 $a$，脚与 $c$ 已变，新旧 $a$ 对不上上一拍几何，躯干位置会跳一下。冻结「首次成为候选」时的 XY，等于钉子位置仍用第一次看见它的坐标；确认只说明钉子是真的。立刻 `reproject` 后位置与上一拍接上。

这不是把采样降到 25 Hz：$p$ 仍每拍 `reproject`（50 Hz）。变慢的只是「要不要换锚」这个决策（最多晚约一拍）。

## 验收

在仓库根目录 `microduck-from-scratch/` 操作。要看运动必须用宿主机原生 viewer；编译与数字回归在容器。权重与 ORT 需已就位（缺则先跑 `bash impl/scripts/fetch-m3.sh`）。宿主机 `.venv` 建法见 `tutor/dev-container.md`。

**1. 宿主机可视化：看它走（先做这一步）**

开三个终端。估计值在内部 `FrameSnapshot`，窗口看不到数字；这步确认被估计的那段运动真实发生。

终端 A（仓库根，起仿真窗口）：

```sh
.venv/bin/python impl/sim/duck_body.py --viewer --port 7801
```

应弹出 MuJoCo 窗口；鸭子站在 handmade 场景里。daemon 未连时物理冻结。

先编出二进制（容器）：

```sh
docker compose -f impl/docker-compose.yml up -d
```

```sh
docker compose -f impl/docker-compose.yml exec rust cargo build --bins
```

终端 B（进 `impl/`，起 daemon）：

```sh
cd impl
```

```sh
env ORT_DYLIB_PATH=third_party/onnxruntime/lib/libonnxruntime.so \
  ./target/debug/miniduckd --sim 127.0.0.1:7801
```

终端 C（仍在 `impl/`）：

```sh
./target/debug/mini-duckctl enable
```

等约 2 秒，窗口里鸭子应站稳。再发行走：

```sh
./target/debug/mini-duckctl move 0.8 0 0 --secs 10
```

观察窗口：约 10 秒内明显前移，保持 upright、不摔倒。再发转身：

```sh
./target/debug/mini-duckctl move 0 0 2.0 --secs 6
```

观察窗口：原地明显转体。看够后，终端 B/A 用 Ctrl+C 停掉即可。

**2. 容器单测（对拍与门控）**

```sh
docker compose -f impl/docker-compose.yml exec rust cargo test
```

观察：默认测试全绿；应有 `odometry::tests::*` 与门控相关 `control::tests::*`。`odometry_live` 默认 ignored，不会在这一步跑。

**3. 容器实时集成：3 次行走 + 3 次转身**

```sh
docker compose -f impl/docker-compose.yml exec rust \
  env SIM_PYTHON=/opt/sim-venv/bin/python \
      ORT_DYLIB_PATH=/work/impl/third_party/onnxruntime/lib/libonnxruntime.so \
  cargo test --test odometry_live -- --ignored --nocapture
```

等待约一两分钟。观察 stdout：每次行走真实前移 ≥ 2.0m 且 upright / `fallen=false`；每次转身 |yaw| ≥ 0.5 rad；每次终点水平误差 ≤ 0.12m，六次中位数 ≤ 0.07m；最大单拍跳动 < 0.02m。

门槛来自 handmade 本地回归，不是上游精度承诺。走约 2.7m 后残差可到约 0.10m。同一传感流上本地与上游逐位相同；残差来自上游算法在我们自建身体上的表现。

**4. M8 回归（确认本课未回退 RPC / 状态机）**

```sh
docker compose -f impl/docker-compose.yml exec rust bash /work/accept/m08-b01.sh
```

预期 FakeIo 断言全绿。

一键数字封装与门限注释见 `accept/m09c-b02.sh`；独立 verifier 记录见 `tutor/runs/m09c-b02.txt`。

## 与原版差异

本 Bite 收敛功能清单里的 odometry crate 行：纯软件接触里程计落在 `impl/src/odometry.rs`。

线上 `RobotState.odom` 仍归 [D50](../../spec/deviations.md)，挂 M10。通用 kinematics 仍归 [D55](../../spec/deviations.md)，挂 M10。没有新登记偏差编号。

## 坑

**1.5mm 的鞋底点抄写错误。** 手抄 ALPHA16 时有一处 z 抄错 1.5mm。以这份表为基准的单测查不出来；只有与上游逐位对拍才暴露。抄常数必须配一条跨实现的逐位比较。

**vyaw=0.8 转不动。** 早期转身用 vyaw=0.8，实际 yaw 只变约 0.07 rad，达不到 0.5 rad 门槛。改用 vyaw=2.0 后 |yaw| 约 3.04 rad。先确认实验跨过机制门槛，再动机制。

**六个样本只有四条不同轨迹。** handmade sim 近乎确定性，三个新进程会给出重复轨迹。中位数由更少的独立值决定，谈不上置信区间。

**yaw 误差为 0 是同义反复。** 估计的 yaw 就是同一份 IMU 四元数读出来的，测试又拿同一 IMU 真值比较，所以恒为 0。它只证明管线没写错，不是精度证据。

**误差门余量偏薄。** 门是 0.12m，实测最坏值可占八成以上。再收紧会不稳，放宽会掩盖真实漂移。

## 自测

**1. 某一拍出现一个更低的鞋底点，下一拍没有更低的点。锚点会怎样？**

- A. 立刻换到那个点
- B. 不换，pending 记下这个候选
- C. 估计清零
- D. 那个点成为新的世界原点

<details>
<summary>答案</summary>

B。换锚要同一脚连续两拍。第一拍只记候选，A 会让 FK 抖动直接搬走锚点。
</details>

**2. 确认换锚时沿用 pending 里冻结的世界 XY，有什么作用？**

- A. 让位置跳到新的足印
- B. 让换锚那一拍的位置保持连续
- C. 减少 FK 计算量
- D. 记录哪只脚在承重

<details>
<summary>答案</summary>

B。锚点被钉在它首次成为候选时的位置。若用确认那一刻的 XY，位置会在换锚瞬间跳一次。这不是降采样：每拍仍 reproject；两拍只挡毛刺换锚，冻结首次 XY 是为了连续。
</details>


**3. 控制循环读到滑行样本时，估计与 pending 会怎样？**

- A. 用旧样本继续更新
- B. 估计清零
- C. 两者都不动
- D. 重建估计器

<details>
<summary>答案</summary>

C。门控要求样本新鲜且 IMU 收敛。滑行拍不推进估计，也不推进 pending 计数。
</details>

**4. 机器人跌倒进入 Limp，之后回 Driving。里程计的原点会怎样？**

- A. 跌倒时清零
- B. 改用新的左脚位置
- C. 保持不变
- D. 随 scheduler reset 一起重建

<details>
<summary>答案</summary>

C。估计器在循环外创建一次，不随相位与调度器生命周期重建。跌倒恢复只清策略记忆。
</details>

**5. yaw 为什么直接读 IMU 四元数，不积分陀螺？**

- A. 陀螺在总线协议里没有字段
- B. 四元数已是 IMU 融合后的姿态，再积分会二次积分并漂移
- C. 四元数换算成角度更快
- D. 上游也这么写，没有别的选择

<details>
<summary>答案</summary>

B。上游的头向本来就是启动相对量，积分只会把漂移累加起来。D 是循环论证。
</details>

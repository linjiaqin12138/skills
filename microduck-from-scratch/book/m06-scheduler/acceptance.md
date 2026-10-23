# M6 技能调度器 — 验收档案

## 交付物

- `src/scheduler.rs`（新建，751 行含 11 条单测）：`Cascade` 纯状态机（不碰 ONNX，优先级/command 重编码/窗口计时可脱离 runtime 单测）+ `Scheduler` 持四槽 Policy（walk 必须槽、sitstand/ground_pick/skills 可选槽）。优先级链：一次性技能 > ground_pick > sit/rise > walk；配置技能表 `SKILLS` Vec 顺序即优先级（roulade > kick_left）。`request_bit`：技能名→边沿位掩码位分配（bit0=ground_pick、bit1=sit_toggle、bit(2+i)=配置技能 i）。command 重编码：技能窗口全零（zero_command_padding）、ground_pick twist=[cos2πφ, sin2πφ, 0]（周期 200 拍=4.0s、φ=0.7 即 140 拍交还）、坐姿 twist=[1,0,0]（posture flag 搭 vx 槽，head/body 保持活值）、起身 twist 全零（50 拍=1.0s）、walk 透传。`busy()` = 技能/ground_pick/起身中（坐姿不算）。抢占简化 D29：运行中一律拒绝。`Scheduler::infer` 换网即 `reset()` 新网络。
- `src/control.rs`：Driving 阶段抽 `driving_tick()` 自由函数（技能边沿→调度→组观测→推理→缩放/低通→嘴覆写→推进窗口；测试与生产同一条路径）。`ControlState` 增 `skill_edges`（u32 位掩码，每拍 `mem::take` 清零）与 `mouth`（纯 level 无超时）。`Stats` 增 `driving`（robot.do 拒绝判定依据）。busy 抑制 Driving→Limp 转移（fallen 报告不变，D32）。非 Driving 拍取到的技能边沿丢弃并留日志。`FrameSnapshot` 增 `skill`（当前活跃技能名，非 Driving 为 None）。`apply_action` 的 scale 改由调度器按槽位给（walk 0.9 / sitstand 1.0 / ground_pick 1.0 / 技能 1.0）。连续性端到端测试 `net_switches_never_jump_targets`（机制断言=切换拍精确等式；回归阈值 5.0 rad=实测峰值 4.20 的 1.2 倍；嘴排除）。
- `src/policy.rs`：`reset_state` → `pub reset()`（换网时新网络 LSTM 状态清零）；`pub(crate) reset_calls` 计数（feedforward 下 reset 是空操作，计数让契约可断言）。
- `src/model.rs`：`mouth_target()`（0..1 → −5°/+30° 线性夹紧，NaN 当 0；数值抄自 reference/duck-control/src/model.rs:62-63）。
- `src/main.rs`：`Scheduler::load`（walk 必须、其余槽可选加载失败只禁用该技能）；新 RPC：`robot.do`（RPC 侧四级拒绝：策略缺失/未 enable/未 Driving/未知名字附名单；刻意差异：fallen 时按 Stats.driving 拒绝，原版静默丢）、`robot.skills`（内置两个在名单前列，对齐原版 do_names）、`robot.mouth`、`robot.head`（无 deadman）；`robot.state` 载荷增 `skill` 字段。
- `src/bin/mini-duckctl.rs`：`do <skill>` / `skills` / `mouth <0..1>` / `head <np> <hp> <hy> <hr>` 子命令。
- `scripts/fetch-m6.sh`（新建）：下载 4 个策略权重（alpha_sitstand / alpha_ground_pick / roulade / ball_kick_left，HF v5，走代理）。
- `scripts/accept-m6.sh`（新建）：阶段 A（FakeIo：A1 启动不动+enable 站立回归含嘴 −5°、A2 skills 名单、A3 do 拒绝语义、A4 mouth 映射夹紧、A5 head 无 deadman）+ 阶段 B（MuJoCo sim 端口 7804：B0 站稳、B1 行走回归、B2 50Hz、B3 ground_pick 调度+相位编码+物理通过、B4 sit 降级调度断言（D34）、B5 roulade 窗口+busy 门控断言（D35）、B6 kick_left 物理通过）。`fresh_sim` 等 `skill=="walk"` 而非 z 达标（z>0.08 在 RampUp 中途就会过）。

## 验收实测

`docker compose exec rust cargo build`：0 警告。`docker compose exec rust cargo test`：

```text
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

构成：M5 的 35 条 + 新增 14 条（scheduler 11：walk 默认透传/sit 锁存 vx 槽/rise 全零+到期回 walk/ground_pick 相位脚本+交还/技能窗口全零+交还/同拍优先级表序赢/技能抢占坐姿后回坐姿/不可用技能拒绝/busy 定义/切换标志；换网 reset 契约（reset_calls 计数）——control 2：edge 取一次清零、切换连续性端到端——model 1：嘴映射端点与夹紧含 NaN）。

连续性测试实测输出（cargo test .stderr）：

```text
continuity: max |Δtarget| = 4.20 rad 附近（起身中途 joint 1，非切换拍；FakeIo 速度恒 0 在训练分布外，策略动作饱和）
continuity: max switch-tick |Δtarget| = 1.61 rad（ground_pick→walk，joint 4）
```

阈值 5.0 = 1.2 × 全程峰值 4.20。机制断言（切换拍目标 == `apply_action(新网动作, Some(上拍锚点), 新槽 scale)` 逐位相等）证明低通锚点跨切换保留；回归阈值拦「切换机制坏了导致的全局跳变」。嘴有意图直通无低通（原版同），排除在断言外。

`docker compose exec rust bash scripts/accept-m6.sh`（2026-09-28 实测关键行）：

```text
===== 阶段 A：FakeIo（无 sim）=====
== A3a. 未 enable 时 robot.do 拒绝 ==
未 enable 拒绝: the policy is not driving — run mini-duckctl enable  OK
== A1. 启动不动 + enable 站立回归 ==
启动姿态全 0，enabled=false，skill=null  OK
enabled=true, gain=200, skill=walk, 距 home 最大偏差 …（含嘴 −5°）  OK
== A2. robot.skills 名单 ==
skills = ['ground_pick', 'sit_toggle', 'roulade', 'kick_left']（内置两个在名单里，顺序=原版 do_names）  OK
== A3b. 未知技能拒绝并附名单 ==
未知技能拒绝: no skill named "backflip"; this robot has ground_pick, sit_toggle, roulade, kick_left  OK
== A4. robot.mouth 映射与夹紧 ==
mouth 1.0 → +30°  OK
mouth 0.0 → −5°  OK
mouth 7.0 → 夹紧 +30°  OK
== A5. robot.head 进观测 command 块，无 deadman ==
obs[51:55] = [0.1, 0.2, 0.3, 0.4]  OK
1s 未重发头命令仍保持（head 无 deadman）  OK
===== 阶段 B：MuJoCo sim（端口 7804）=====
== B0. 站稳回归 ==
站稳：body z≈0.1166, skill=walk
== B1. 行走回归 ==
10 秒前进位移 = 0.674 m（门槛 0.5m）
== B2. robot.state 50Hz ==
2 秒收到 101 帧通知
== B3. ground_pick ==
ground_pick：140 帧（~140），twist=单位圆相位编码，无摔倒，结束回 walk  OK
== B6. kick_left ==
kick_left：25 帧窗口（~25），command 全零，无摔倒，结束回 walk  OK
== B4. sit_toggle（降级断言）==
sit：38 帧 skill=="sit"，twist=[1,0,0]，锁存保持  OK
注：物理上坐下后向后翻倒（第 64 帧 fallen）——已登记偏差 D34，M8 随 D21 裁决
== B5. roulade（降级断言）==
roulade：50 帧窗口（~50），busy 期间 33 帧 fallen 全部 gain=200，结束后 Limp 接管  OK
```

**回归**：`docker compose exec rust bash scripts/accept-m5.sh` 六项全过（行走 0.853m/10s）——调度器进场未破坏安全层与行走。

**诚实说明（sit/roulade 物理失败，D34/D35）**：两技能的调度、command 重编码、窗口计时、锁存全部验证正确，物理上失败。对照实验：kp=40+forcerange±10 与 kp=8 倒在同一时刻（非 torque 上限）；action_scale 减半同样倒（非尺度问题）→ 指向 BAM 执行器动力学缺失（D21），M8 随 D21 一并裁决。sim 验收相应降级为调度断言（B4）与调度+busy 门控断言（B5），脚本注释明写降级原因。

**复核更正（教程写作时）**：交接记录称「原版 --sim 是无物理回显（reference/robotd/src/main.rs:240）」——误读。main.rs:239-240 的注释挂在 `--fake` 上；原版 `--sim` 是真 MuJoCo（reference/docs/design/simulation.md 声称 sitstand 在原版仿真里能起身并保持直立）。真正的事实：原版仿真体 duck-body 在 microduck_rl 仓库、不在浅克隆里，我们的 `sim/duck_body.py` 是手搓的这一半——这反过来加强了 D34/D35 指向执行器模型的归因。

## 涉及代码文件清单

| 文件 | 变动 |
|---|---|
| `src/scheduler.rs` | 新建（751 行含 11 条单测）：Cascade 纯状态机 + Scheduler 四槽 |
| `src/control.rs` | Driving 抽 `driving_tick()`；ControlState 增 skill_edges/mouth；Stats 增 driving；busy 抑制 Limp；FrameSnapshot 增 skill；apply_action scale 按槽位 |
| `src/policy.rs` | `reset()` 转 pub；`reset_calls` 计数 |
| `src/model.rs` | `mouth_target()` |
| `src/main.rs` | Scheduler::load；robot.do/skills/mouth/head；state 载荷增 skill |
| `src/bin/mini-duckctl.rs` | do/skills/mouth/head 子命令 |
| `scripts/fetch-m6.sh` | 新建，4 个权重下载 |
| `scripts/accept-m6.sh` | 新建，阶段 A 五项 + 阶段 B 七项 |
| `docs/milestones/m6-scheduler/{tutorial,acceptance}.md` | 本档案与教程章节 |
| `docs/milestones/m6-scheduler/{arch,arch-diff}.{d2,svg}` | 架构图 |

## 偏差变动

- **收敛核销**：D3（单策略无调度器 → 优先级链+重编码+窗口+LSTM reset 全对齐）。
- **保持待收敛**：D20（5 个策略实测全 feedforward，LSTM 分支无运行时覆盖；reset 调用已由 reset_calls 断言）。
- **新开**：D28（无 Mode/Roller）、D29（抢占简化，无 chain/unwind/end_phase 外交还）、D30（无策略热换/carry_over）、D31（嘴无特雷门/合唱层）、D32（busy 门控简化：门直接转移而非 FallPredictor）、D33（无 stand 槽/will_stand 幅值选网/standing_gain_ratio=0.8）、D34（sit 物理失败）、D35（roulade 物理失败）。

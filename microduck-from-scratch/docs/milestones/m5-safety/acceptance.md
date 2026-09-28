# M5 安全层 — 验收档案

## 交付物

- `src/safety.rs`（新建，555 行含 14 条单测）：`Safety<T: RobotIo>` 私有化拥有唯一 RobotIo 写句柄（借用检查器强制，读透传）。`SafetyConfig`（fall_gravity_z=-0.5 / fall_debounce=200ms / deadman=500ms / gain_running=200 / gain_limp=50，原型 alpha 默认）。`observe`（跌倒去抖双向判定；`imu_ready=false` 直接 return 不投票——原版血泪回归；判定翻转必报不限流）。`gate`（deadman 只清 twist 不动 head；deadman_armed：从没司机不是新闻，司机失联才是）。`apply`（set_gain 缓存去重 → 任一 NaN/Inf 整包拒绝写 hold（不是夹紧）→ ±π 执行器行程夹紧并上报 → 写总线；无跌倒门控，fallen 是报告）。`LimitRuns` 三条独立 run 计数，首拍+每 50 拍日志限流，NaN 拍不清 range run。
- `src/control.rs`：主循环重写为五相位状态机 `Phase::Held/RampUp/Driving/Limp/RampDown`。每拍顺序 read→observe→gate→策略→apply。Held 抱启动姿态绝不调 set_torque；enable 边沿→set_torque(true)→100 拍斜坡回 home→Driving；Driving 中 fallen→Limp（目标跟随实测位置+gain 50，策略停摆）；直立后斜坡回 home 恢复；disable→斜坡回 home→set_torque(false)→Held。policy=None 永远 Held（D19）。边沿检测先于跌倒判定。Stats 记账供 health。FrameSnapshot 增 fallen/enabled/gain/torque。
- `src/io.rs`：trait 补 `set_gain`/`set_torque`/`imu_ready`（默认 true）；quat 严格长度校验 ≠4 即 Err（D22），copy3 长度不符的 panic 隐患顺手改 Err。FakeIo 记录 gain/torque/gain_writes/imu_ready 供测试断言。
- `src/main.rs`：Policy::load 失败不退出（policy=None，health 报 "policy unavailable"，D19——Restart=always 下退出=crashloop）；socket chmod 0660（D8——原版 robotd 无 SO_PEERCRED，文件权限即鉴权）；robot.enable/disable；robot.health 真实判定（stall 500ms=25 拍 / achieved_hz≥45 / 连续读错误≥10 / policy_ok，阈值来自原版 robotd-params；暖机豁免）；robot.state 改 frame_rx.changed() 驱动 50Hz（D14），载荷增 fallen/enabled/gain/torque。
- `src/bin/mini-duckctl.rs`：enable/disable；`drive --secs N` 期间每 100ms 重发意图（deadman 500ms 下单次意图只能驱动半秒，CLI 扮演持续发意图的手柄）；`state --every N`（默认 50 帧打一行）。
- `sim/duck_body.py`：`set_gain`（kp_sim=8×gain/200，改 gainprm/biasprm 不缩 ctrl）；`set_torque`（增益清零=卸力）；`push {vx,vy}`（qvel 加水平扰动推倒鸭子）；启动默认 torque on/gain=200（模拟舵机 RAM 跨进程存活）；read 回显 gain/torque。
- `scripts/accept-m5.sh`（新建，六项断言）；`scripts/accept-m4.sh`（加 enable 步骤保持回归绿）。
- `docs/milestones/m5-safety/tutorial.md`、`docs/concepts/servo-gain-torque.md`（+ assets 下两个概念演示脚本）。

## 验收实测

`docker compose exec rust cargo build`：0 警告。`docker compose exec rust cargo test`：

```
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

构成（`cargo test -- --list` 实测）：M4 的 18 条 + 新增 17 条 = safety 14（去抖双向、imu_ready 不投票双向、跌倒不抢占、软倒增益透传、NaN 拒绝非夹紧、越界夹紧上报、普通目标原样通过、gain 缓存去重、deadman 只清 twist、armed 语义、run 独立计数、限流节奏）+ io 3（quat 严格校验、set_gain/set_torque 帧、FakeIo 记忆 gain/torque）。（注：交接文档称「原 19 + 新增 16」，实测名单为 18 + 17，以 `cargo test -- --list` 为准。）

`docker compose exec rust bash scripts/accept-m5.sh`（教程写作时复跑原文，2026-09-27）：

```text
===== 阶段 A：FakeIo（无 sim）=====
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
===== 阶段 B：MuJoCo sim =====
== B0. 起 sim + daemon --sim ==
站稳：body z=0.11664183743205557
== B3. push 推倒 → fallen + 卸力（gain 50）==
push 应答: {"ok": true}
捕获 151 帧，首帧 fallen+gain=50 出现在第 76 帧  OK
瘫在地上：body z=-0.11105323444844586（站立约 0.12）  OK
== B4. health 真话：杀 sim ==
杀 sim 后 health: {"jsonrpc":"2.0","id":2,"result":{"achieved_hz":49.995,"consecutive_read_errors":26,"healthy":false,"reads":328,"reason":["consecutive read errors: 26"],"skipped_reads":26,"tick":354,"uptime_s":7,"writes":328}}
healthy=false，原因: ['consecutive read errors: 26']
== B4b. 重启 sim → 重连恢复 ==
重连后 healthy=true  OK
== B5. robot.state 50Hz ==
2 秒收到 101 帧通知
== B6. 行走回归 ==
起步 body: 0.001051335373983999 -0.0029315070022370836 0.11831107838719665 2.259999999999862
10 秒前进位移 = 0.775 m（门槛 0.5m）

M5 验收全部通过：启动不动/deadman/跌倒卸力/health 真话/50Hz state/行走回归
```

主 agent 独立复跑（交接记录）：enable 3s 后距 home 最大偏差 0.1799 rad；跌倒捕获 151 帧第 76 帧起 fallen=true 且 gain=50；杀 sim 后 healthy=false、reason=["consecutive read errors: 26"]；2 秒 100 帧通知；10s 位移 0.682m。两遍数值略有差异（FakeIo 漂移与 MuJoCo 起步时序），全部在阈值内。

**回归**：`docker compose exec rust bash scripts/accept-m4.sh`（M5 加 enable 步骤后）复跑原文（2026-09-27）：

```text
== 3b. enable（M5 起启动抱持不动，必须显式使能）==
{"jsonrpc":"2.0","id":2,"result":{"enabled":true}}
== 4. 站稳（3 秒，斜坡 2s + 策略闭环 1s）==
站稳后 body(x y z t): 0.011768334762058342 -0.015111194001340525 0.11880153780877012 5.000000000000004
== 5. drive vx=0.15 持续 10 秒 ==
drive 后 body(x y z): 0.7557119116457758 -0.3236168136833132 0.12053440234678475
== 结果：10 秒前进位移 = 0.744 m（门槛 0.5m）==
PASS
```

（主 agent 复跑：站稳 z=0.1191，位移 0.767m，PASS。）

**补充手动验证（coder 做过，无脚本）**：策略文件缺失时 daemon 存活、health 报 "policy unavailable"、enable 后仍抱持不动；`--view-port` MJPEG 画面未破坏。

**破坏实验（教程第 5 节，写作时实测）**：

1. deadman 收紧到 50ms（`src/safety.rs:52` 临时改值）：CLI 每 100ms 重发意图，观测到 obs twist 以 3 拍通/2 拍断抽搐——101 帧中 vx=0.15 的 49 帧、vx=0 的 52 帧；daemon 日志 5 秒 29 行 `intents stale 68ms > deadman 50ms (run 1) — twist zeroed`。deadman=100ms（恰等于重发间隔）时 101 帧仅 1 帧清零——竞态边缘偶发。实验后已改回 500ms 并重新 build。
2. 策略输出注入 NaN（`src/control.rs` Driving 分支临时 `targets[3] = f64::NAN`）：Driving 期间每拍整包拒绝写 hold，51 帧位置逐位不变（冻结在 home）；日志限流节奏可见——run 1 / run 50 / run 100 三行。实验后已复原并重新 build，`cargo test` 35 passed 确认无损。

## 涉及代码文件清单

| 文件 | 变动 |
|---|---|
| `src/safety.rs` | 新建（555 行含测试） |
| `src/control.rs` | 五相位状态机重写；Stats 扩账；FrameSnapshot 增安全字段 |
| `src/io.rs` | trait +3 方法；quat/copy3 严格校验（D22）；FakeIo 记账字段 |
| `src/main.rs` | D19 不退出；0660（D8）；enable/disable；health 真实判定（D11）；state 50Hz（D14） |
| `src/bin/mini-duckctl.rs` | enable/disable；drive --secs 重发；state --every |
| `sim/duck_body.py` | set_gain/set_torque/push；启动默认 torque on/gain=200；read 回显 |
| `scripts/accept-m5.sh` | 新建，六项断言 |
| `scripts/accept-m4.sh` | 加 enable 步骤 |
| `docs/concepts/servo-gain-torque.md` | 新建概念深读 + `docs/concepts/assets/{servo-gain-torque,deadman}/` 两个演示脚本 |
| `docs/milestones/m5-safety/{tutorial,acceptance}.md` | 本档案与教程章节 |

## 偏差变动

- **收敛核销**：D8（0660，非 SO_PEERCRED——侦察确认原版 robotd 无 SO_PEERCRED，文件权限即鉴权）、D9（启动不动）、D11（health 真实判定）、D14（50Hz）、D19（策略失败不退出）、D22（quat 严格校验）。
- **部分收敛**：D5（安全层核心规则齐了；残余温度监控/配置文件面 → M8）、D12（set_gain/set_torque/imu_ready；残余 reboot/slow_sensors/imu_stale/measures_* → M8）。
- **新开**：D23（跌倒响应简化：fallen 直接触发 Limp，非原版 FallPredictor 陀螺外推+Limp/Posing 三态机；原版 limp_fall 默认 OFF 我们默认 ON——velstand 零命令能站住，交接得回）、D24（无 coast 3 拍滑行）、D25（state 无逐订阅者降频/Lagged 语义）、D26（sim 推倒后穿透地板 z=-0.111，阈值取 z<0.08）、D27（FakeIo 闭环漂移 0.13~0.18 rad，enable 阈值放宽到 0.4）。

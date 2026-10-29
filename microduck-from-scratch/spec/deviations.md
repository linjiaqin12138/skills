# 偏差登记簿

规则：每条偏差是**待收敛项**，不是永久豁免。收敛点到达时核销；M8 收敛验收时残余偏差逐条裁决（对齐回去 / 保留并写明理由）。

## Phase 4 收敛验收裁决（2026-10-15，用户裁决）

验收电池当天复跑全绿：91 单测 / accept-m8 断言 a–w / accept-m8-updater 6/6 / accept-m8-bam 四项（站立 z=0.117、sit 坐稳+起身、roulade 结束直立、行走 vx=0.8 前移 4.875m ≥2.0m 门槛）。

**reference 版本基准裁决**：功能清单（2026-10-04）写于 API_VERSION=37 的检出；2026-10-06 换机重克隆后磁盘上是 daemon-v0.14.1 release（API_VERSION=31，当前 origin/main HEAD，commit 344925c）。用户裁决**以 v0.14.1 为终态对齐基准**：M8 Bite 2–8 的对齐证据全部命中该克隆（已抽查 MODEL_API robotd/src/main.rs:58、safeToRestart reason :905、enable reason :4506）；73 个 Call 变体与清单 §2 全覆盖对账、无清单外缺口；API 32–37 差量留 M10 终态对齐时盘点。清单行号证据已按 v0.14.1 回填。

**处置分组**：A=本次核销；永久保留=架构/教学定位类（理由见各行）；M9/M10/M11=批次挂账（2026-10-04 四批路线裁决的延续）；Bite 9=就地收敛（D13/D20/D24/D65——**已于 2026-10-15 当天交付验收**，98 单测 + accept-m8 a–y + accept-m8-updater 8/8 + accept-m8-bam 四项全绿）。

**Bite 9 补充裁决（2026-10-15，用户）**：D20 由"删 LSTM 死代码"改为**保留分支**——原版 policy.rs:601 的 (3,3) LSTM 加载契约本身是原版能力面（删除=新偏差），且 M9 自训可能直接用（rsl_rl 支持 recurrent actor）。实证：官方仓库全部 tag（v1–v8+main）+ 7 个社区采样模型均为 1-in/1-out feedforward，但这不改变裁决。Bite 9 给分支补了运行时覆盖（合成最小 LSTM 夹具）。

**行号回填（2026-10-15）**：spec/features.md 全部证据行号已按 v0.14.1 重写（~190 处）。2 项在 v0.14.1 不存在、属 API 32–37 新增：`[bus] fast_sync_read`、`[control] publish_velocity_and_load`（连带 RobotState 的 velocities/currents_ma 字段，D50 范围内）——已标 ⚠，随 M10 差量盘点处理。

| # | 偏差 | 引入 | 预定收敛点 | 状态 |
|---|---|---|---|---|
| D1 | 只实现 robot.* 极小子集（hello/health/state），非完整 API 面 | M0 | M8 对照 upstream/duck-ipc-proto 逐项核对 | **已收敛（Phase 4）**：逐项核对 = spec/features.md 本身，残余缺口已编号 D42–D63 并分配批次 |
| D2 | FakeIo 无真实 Dynamixel 串口协议 | M1 | 豁免——硬件层闭源，"能下载的不重写"原则覆盖 | 已豁免 |
| D3 | ~~单策略、无调度器~~ M6 已收敛：scheduler.rs 优先级链（技能 > ground_pick > sit/rise > walk），command 重编码、拍数窗口、换网 LSTM reset 全部对齐原版 | M3 | M6 | 已收敛（M6） |
| D4 | ~~自定义仿真 TCP/JSON 协议（op 名 set_gain/set_torque）~~ **M9a Bite 4 已收敛**：线上 op 对齐官方 `gain`{kp}/`torque`{on}（sim.rs / body_server）；PROTOCOL=1、15 槽含嘴、trunk 重力断言钉死。教学附加 `push`/`body`/read 回显保留（官方 serde 忽略未知字段）；`slow`/`tof`/`currents_ma` 不扩面（slow 仍归 D12→M10）；hello 仍校验 joints（比官方更严） | M4 | M9a Bite 4 | **已收敛（M9a Bite 4）** |
| D5 | 安全层核心规则已对齐（NaN 拒绝/±π 夹紧/deadman/跌倒判定/gain 缓存），残余：无温度监控、无配置文件面（SafetyConfig 硬编码 default） | M5 引入 | ~~M8~~ 残余挂 M10：温度监控是硬件面；配置文件面并入 D52 | 部分收敛（残余挂 M10） |
| D6 | ~~无 systemd 部署、updaterd 无 minisign 验签~~ M7 落地后拆成两半移交：无 systemd 由 D37 承接、无 minisign 验签由 D38 承接 | M7 计划 | M8 随 D37/D38 裁决 | 已移交（M7，见 D37/D38） |
| D7 | 单 crate 双 bin，非 workspace 多 crate | M0 | ~~M8~~ 挂 M10（随 D52：协议 crate 拆分 + "只许 serde/serde_json/semver" 依赖约束） | 待收敛（挂 M10） |
| D8 | 无 SO_PEERCRED uid/gid 校验 | M0 | ~~第一个 mutating 调用出现时~~ M5：侦察发现原版 robotd 无 SO_PEERCRED，鉴权=socket 0660 文件权限（SO_PEERCRED 是 configd/updaterd 模式，M7 实现）。M5 已 chmod 0660 | 已收敛（M5，按文件权限模型对齐） |
| D9 | ~~启动即插值到 home；原版"进程启动绝不移动机器人"（held_pose）~~ M5 已收敛：Held 阶段抱住启动姿态，绝不调 set_torque，显式 robot.enable 才斜坡回 home | M1 | M5 | 已收敛（M5） |
| D10 | 插值在控制循环内逐拍推进；原版 interpolate_to 是阻塞调用 | M1 | ~~M8 裁决~~ Phase 4 裁决 | **永久保留（Phase 4 裁决）**：逐拍推进与 50Hz 控制循环同构，阻塞调用在事件循环里是退步；架构类偏离，可观察行为等价 |
| D11 | ~~healthy 恒 true，无 deadline 检测~~ M5 已收敛：stall 500ms(25拍)/频率地板 45Hz/连续读错误 10/策略缺失，阈值来自原版 robotd-params | M1 | M5 | 已收敛（M5） |
| D12 | RobotIo 只有 read/write，无 set_gain/set_torque/reboot/slow_sensors | M1 | M5 部分收敛：set_gain/set_torque/imu_ready 已加；残余 reboot/slow_sensors/imu_stale/measures_velocity/measures_load ~~→ M8~~ 挂 M10（硬件寄存器面） | 部分收敛（残余挂 M10） |
| D13 | ~~IoError 不分类（单一字符串错误）~~ **Bite 9 已收敛**：IoError 四变体 `Port{path,source}` / `Bus(String)` / `ShortRead{what,expected,got}` / `Simulated`，粒度逐一对应 upstream/duck-control/src/io.rs:50-71（未发明新类别）；`Port` 文案用 "transport"（原版 "serial port" 对我们唯一的 TCP SimIo 使用者会误导，文案自有、映射对齐） | M1 | ~~M8~~ Bite 9 | **已收敛（Bite 9）** |
| D14 | ~~robot.state 仍 1 Hz~~ M5 已收敛：推送改 frame_rx.changed() 驱动（50Hz 每拍一帧），载荷增 fallen/enabled/gain/torque；逐订阅者降频与 Lagged 语义见 D25 | M0/M1 | M5 | 已收敛（M5） |
| D15 | API_VERSION=1 且握手差异只报告不拒绝（行为与原版一致，版本号起点不同） | M0 | ~~M8~~ 挂 M10 终态对齐；目标值改 **31**（v0.14.1 基准，Phase 4 版本裁决） | 待收敛（挂 M10） |
| D16 | 控制循环用 MissedTickBehavior::Skip（对齐原版；原版实测 Delay→43.1Hz） | M1 引入默认 Burst / M3 改为 Skip | M3 | 已收敛 |
| D17 | last_action 为真实上一拍策略输出（M2 曾恒 0） | M2 引入 / M3 写入 | M3 | 已收敛 |
| D18 | command 恒为默认零，速度/头/机身命令还没入口 | M2 | M4（robot.drive {vx,vyaw} 写共享 Command，观测 command 块取真值） | 已收敛 |
| D19 | ~~策略 load 失败直接退出进程~~ M5 已收敛：policy=None 传入控制循环，永远 Held 抱持，health 报病 "policy unavailable: …"（原版理由：Restart=always 下退出=crashloop，活着报病让 updater 回滚） | M3 | M5 | 已收敛（M5） |
| D20 | ~~policy.rs 含 LSTM 状态分支但无运行时覆盖~~ **Bite 9 已收敛（按修订裁决保留分支）**：原版 (3,3) LSTM 契约（upstream/duck-control/src/policy.rs:601）是能力面，不删；新增合成最小 LSTM 夹具（`write_minimal_lstm_onnx` → tests/fixtures/minimal_lstm.onnx）+ 单测（状态跨拍推进、reset 复现第一拍）。连带对齐两处原版语义：推理失败即 reset 该网 + 忘活跃网（PolicySet::infer :361-365）；disable 时 Scheduler::reset() 连带清活跃选择（PolicySet::reset :415-416）。实证留档：官方全 tag + 7 社区采样模型全 FF | M3 | ~~首次加载 recurrent 文件时核销；若 M8 仍无则删死代码~~ Bite 9（修订：保留+补覆盖，见顶部补充裁决） | **已收敛（Bite 9）** |
| D21 | ~~仿真执行器为 XML PD 而非训练用 BAM~~ **M8 Bite 6 已收敛**：默认 `better-actuator-models` BAM M6 XL330（`bam.mujoco.MujocoController`，timestep 0.005，场景 `scene_groundcontact`）；`--no-bam` / `MINIDUCK_SIM_BAM=0` 回退 XML PD 诊断。蓝图 microduck_rl `infer_policy.py`；bam 钉 `mjlab_frictionloss@62bd8ce`（`vin_drop_gain`） | M4 | M8 Bite 6 | 已收敛（M8 Bite 6） |
| D22 | ~~quat 解析用 resize(4,0) 静默补零/截断~~ M5 已收敛：严格长度校验（≠4 即 Err）；顺手把 copy3（gyro/gravity）长度不符会 panic 的隐患统一改为 Err | M4（code review 发现） | M5 | 已收敛（M5） |
| D23 | ~~跌倒响应为 fallen 判定直接触发 Limp，直立后重走 RampUp；无 FallPredictor / Posing~~ **M9c Bite 1 已收敛**：`fall.rs` 陀螺外推（300ms）+60ms 一次性预测；Limp 新鲜陀螺静止 200ms 或 1500ms 上限，独立 Posing 600ms/gain160 后清策略反馈回 Driving；默认 OFF 对齐原版，`MINIDUCK_LIMP_FALL=1` 为完整配置面落地前的显式开关（D52 仍挂 M10）。验收 `accept/m09c-b01.sh` | M5 | M9c Bite 1 | **已收敛（M9c Bite 1）** |
| D24 | ~~read 失败跳拍不滑行；原版 COAST_TICKS=3~~ **Bite 9 已收敛**：`COAST_TICKS=3` + `Coast` 结构（原版 robotd/src/main.rs:137、:3189-3223）；失败拍用上一样本续跑整拍管线（策略照常 step、快照照常发布），safety.observe 只喂新鲜样本，窗口耗尽或从未读到才真跳拍。**语义细节（coder 按代码证据对齐，与交接文字的偏差登记在案）**：consecutive_read_errors 每次失败即计（原版 main.rs:1971-1972，喂 health 阈值 10），滑行只管"对策略可不可见"；**skipped_reads 语义收窄**——从"每次失败"改为"只计真瞎跳拍"，health JSON 字段名不变、值含义变化 | M5（设计时确认保留现状） | ~~M8~~ Bite 9 | **已收敛（Bite 9）** |
| D25 | robot.state 用 watch latest-wins；原版 broadcast 256 帧缓冲 + 落后者 Lagged 丢帧 + 逐订阅者 hz 降频。M8 Bite 3：`robot.subscribe` 已收下 hz（缺省/0/正整数合法），解析后丢弃，推送仍全速率；ack 不回显 hz。降频/broadcast/Lagged 仍未做 | M5（50Hz 收敛 D14 时保留） | ~~M8 之后专设~~ 挂 M10（Phase 4 裁决） | 待收敛（挂 M10） |
| D26 | ~~sim 推倒后躯干穿透地板（z=-0.111，接触求解器大冲击失真）；验收阈值因此是"z<0.08"而非躺平高度~~ **M9a Bite 1 已收敛**：自建 duck_handmade.xml 用 primitive box 碰撞（官方碰撞网格的 AABB 包围盒）替掉 mesh-plane 接触；卧姿落体+站姿推倒稳定后最低碰撞角点 z=-0.0002（≥-1mm 阈值），同条件官方网格模型对照穿透 13.7mm（M5 极端工况 -0.111）。验收 `accept/m09a-b01.sh` D 组 | M5（验收实测） | ~~M8 裁决~~ ~~挂 M9~~ M9a Bite 1 | **已收敛（M9a Bite 1，2026-10-20）** |
| D27 | FakeIo 上策略闭环漂移 0.13~0.18 rad（速度恒 0 不在训练分布），enable 验收阈值 0.4 是实测放宽 | M5（验收实测） | ~~M8~~ Phase 4 裁决 | **永久保留（Phase 4 裁决）**：FakeIo 精确回写是教学假总线定位，一阶惯性模型无教学价值；真机侧误差归 M10 硬件批 |
| D28 | 无 Mode(Walk/Roller) 与 roller/roller_crouch 槽（v5 权重仓库里有这两个文件，未下载未接线） | M6 | ~~M8~~ 挂 M9（Phase 4 裁决：随自训策略与 mode 面一并） | 待收敛（挂 M9） |
| D29 | 技能抢占简化：技能/ground_pick 运行中一律拒绝新技能/ground_pick 请求；无 chain 窗口（roulade chain=true 未实现）/链式重放；无 unwind 阶段；ground_pick 固定跑满 end_phase=0.7（140 拍） | M6 | ~~M8~~ 挂 M9（Phase 4 裁决：调度语义随自训策略对齐；manifest v5 已有 chain/unwind_s 字段可对照） | 待收敛（挂 M9） |
| D30 | 无策略热换/carry_over（robot.set_policy / mode_switch 不实现；Controller::carry_over 的跨 swap 状态保留不做） | M6 | ~~M8~~ 挂 M9（Phase 4 裁决） | 待收敛（挂 M9） |
| D31 | 嘴无特雷门/合唱覆写层（voice/chorale 不实现）；robot.mouth 仅 Driving 阶段生效（斜坡/抱持/Limp 期间嘴跟随该阶段自己的目标——这条与原版一致） | M6 | ~~M8~~ 挂 M11 再评估（Phase 4 用户裁决：**不豁免**——M11 自研结构/硬件若含音频再开账，届时连带 robot.sound 与 [audio]/[theremin]/[chorale] 配置段） | 待收敛（挂 M11） |
| D32 | ~~busy 直接抑制 Driving→Limp，未门控预测器~~ **M9c Bite 1 已收敛**：busy 与 sitting 都使 FallPredictor 不合格；非 Driving、非新鲜、IMU 未就绪、已 fallen 或功能关闭同样清去抖，不跨窗口保存旧计数。fallen 报告不变。验收 `accept/m09c-b01.sh` | M6 | M9c Bite 1 | **已收敛（M9c Bite 1）** |
| D33 | 无 stand 槽与 will_stand 幅值选网（原版 twist 幅值 ≤ standing_threshold 且有 stand 网络时选 Stand 槽，standing_gain_ratio=0.8）；我们的 velstand 挂 walk 槽同时覆盖站立（M3 起即是），站立增益恒 200 不打 0.8 折 | M6（对照原版级联时确认） | ~~M8~~ 挂 M9（Phase 4 裁决）。**注意基准变化**：v5 manifest 注明 "the default walk policy since v5, with no separate standing network"——v0.14.1 配置面是否仍有 stand 槽以行号回填结果为准，若已删除本条自动消解 | 待收敛（挂 M9，可能随基准消解） |
| D34 | ~~sit 物理失败~~ **M8 Bite 6 已收敛**：BAM+groundcontact 下 sit_toggle 坐稳 ≥2s（fallen 恒 false，z≈0.059）并可起身回 walk（z>0.08）。验收 `accept/m08-b06.sh` | M6 | M8 Bite 6 | 已收敛（M8 Bite 6） |
| D35 | roulade：**M8 Bite 6 部分收敛**——窗口对齐官方 2.0s（100 拍）；结束后 upright（walk、fallen=false、gain=200、z>0.08）。**残余**：中段仍约 30–40 帧 fallen（前滚贴地），验收按「结束后 upright」而非全程 fallen=false | M6 | 残余挂 M9（自训策略时收敛） | 部分收敛（残余挂 M9） |
| D36 | ~~robot.do 在 fallen/Limp 时 RPC 侧拒绝（accepted:false + 原因）~~ **Bite 10 已收敛（Phase 4 补裁决：对齐原版）**。四层语义对齐原版（upstream/robotd/src/main.rs:4188-4219 + :2118-2159）：门口只看 !enabled（Held/Stopped → 拒，":4200 文案逐字"）与 !homed（RampUp → 拒，":4202-4204 文案逐字"）+ 未知名附名单；**fallen 永不入门口裁决**（原版 :4189-4191 "a skill on a fallen robot is the human's call"）；fallen/Limp 的请求过门口后由控制循环当拍静默吞（edges_when_driving，唯一丢弃点，只留 journal 日志）。**登记在案的层级差异**：原版 fallen 不停驱（:2760-2761，倒地仍能发起起身技能"righted it"），我们 D23 状态机 fallen⇒Limp⇒不在 Driving⇒吞掉——与裁决目标一致但不是原版 fallen 分支的逐字复刻，靠 Limp 自动恢复覆盖起身；新增 Stats.homed（false 仅 Held/RampUp）；D23 表面化：Limp 恢复重走 RampUp 期间 homed 暂回 false（原版无重斜坡）。验收 z1–z5 | M6（决策卡片：robot.do 在哪一层拒绝） | ~~M8~~ Bite 10 | **已收敛（Bite 10）** |
| D37 | 无 systemd：mini-updaterd 直接 spawn/kill miniduckd 子进程替代 `systemctl restart`（setsid 脱离会话，updaterd 死不杀 daemon；启动时按 /proc comm 收割上次留下的孤儿）；无 golden/boot-check 外层兜底 | M7 | ~~M8~~ 挂 M10（Phase 4 裁决：随真机部署面） | 待收敛（挂 M10） |
| D38 | 无 minisign 验签，release 完整性只有 sha256（manifest 登记 artifact 的哈希，apply 先验再解包）；D6 的上半句（无 systemd 部署）由 D37 承接 | M7 | ~~M8~~ 挂 M10（Phase 4 裁决） | 待收敛（挂 M10） |
| D39 | updater 只有 LocalDir 单一 source（`<root>/source/`），无 GitHub/HF/网络发现/channel 策略/自动检查定时器；artifact 是未压缩 .tar（调系统 `tar -xf`）而非原版的 .tar.zst | M7 | ~~M8~~ 挂 M10（Phase 4 裁决：网络 source 与 D60 同源） | 待收敛（挂 M10） |
| D40 | SO_PEERCRED 缩为 uid==0 \|\| socket owner 单点门控，无 allow_uids/allow_gids 配置（容器单用户，教学复刻里配置面没有教学价值） | M7 | ~~M8 裁决或永久豁免~~ Phase 4 裁决 | **永久保留（Phase 4 裁决）**：容器单用户，allow_uids/gids 配置面无教学价值；架构类偏离 |
| D41 | updater 无 hooks/orphan 检查/transcript/Degraded 裁决（mini 健康判定是布尔）/subscribe 推送/self-update；Phase 枚举只取 proto 子集（Idle/Verifying/Extracting/Swapping/Applying/HealthGate/Committing/RollingBack） | M7 | ~~M8 裁决~~ 挂 M10（Phase 4 裁决） | 待收敛（挂 M10） |
| D43 | ~~robot.drive {vx,vyaw} 请求式 vs 原版 robot.move {vx,vy,vyaw} 通知式~~ M8 Bite 1 已收敛 robot.move 本体；M8 Bite 2 已收敛 notification 统一入口与 head/mouth 通知语义。残余：③ mini-duckctl move 仍请求式逐条调用（教学工具定位保留；原版由 padd 以通知式 20–50Hz 发送，daemon 两种形态都收）。联动：请求式 move 应答为 {accepted} 超集（多回显）、head/mouth 回显参数值缺 accepted 字段（原版统一回 IntentResult::accepted()） | M4（feature-inventory 追认编号） | ~~M8~~ Phase 4 裁决 | **已收敛（Phase 4）**：残余③永久保留（CLI 请求式 = 教学工具定位，daemon 双形态都收，可观察行为差异为零）；应答超集/回显差异并入 D61 形状对齐（M10） |
| D64 | ~~disable 语义：旧 RampDown = 2s 斜坡回 home + 卸 torque~~ M8 Bite 2 已收敛：Stopped 取代 RampDown。残余存疑：Scheduler::reset() 是否应连带清 Cascade 坐姿锁存，无原版逐行确证【合理推断：按"锁存不是 episode 记忆"不动】。连带代价：robot.relax 未实现前卸 torque 无出口（D53 缺口待排期 M10） | M8（复核发现） | M8 | 已收敛（M8 Bite 2；残余存疑登记在案） |
| D47 | ~~无 `robot.safeToRestart` / `robot.modelApi` 两个 RPC~~ M8 Bite 4 已收敛：daemon 侧两个方法进 RPC 分发表，载荷、MODEL_API=2、判决口径、`permits_restart` 语义全对齐原版。残余见 D65/D66/D67 | M8 Bite 4（feature-inventory 追认编号） | M8 | 部分收敛（M8 Bite 4；残余 D65 已收敛（Bite 9），D66/D67 挂 M10） |
| D65 | ~~`robot.modelApi` 返回 `None`（daemon 不可达/应答形状不对）时 apply **拒绝**~~ **Bite 9 已收敛**：`model_api_verdict(Option<u32>)`——`None \| Some(2)` 放行、`Some(其他)` 拒（对齐 upstream/updater/src/policy.rs:1026-1039：未知回落契约自身版本，只拒"在且错"的声称）；accept-m8-updater 新断言：daemon 不可达时 apply 不再被拒、落到 committed | M8 Bite 4 | ~~待裁决~~ Bite 9 | **已收敛（Bite 9）** |
| D66 | 预检内联在 `apply()` 里，无原版 `preflight.rs` 框架：`Check` 枚举 + 可复用报告结构 + SideloadDir / NoRemoteSession / 空间检查都没做 | M8 Bite 4 | ~~M9/M10 视需要再拆~~ 挂 M10（Phase 4 裁决） | 待收敛（挂 M10） |
| D67 | `Incompatible` 分支不打日志：原版打 `tracing::warn!`（upstream/updater/src/robot.rs:201-203），我们直接返回变体、静默通过——契约漂移是排障第一现场 | M8 Bite 4 | M10 | 待收敛（挂 M10） |
| D68 | ~~BAM 站稳后 `vx` 进不了步态~~ **2026-10-14 Bite 8 已收敛（用户裁决）**。行走不动点根因=公开 velstand 权重对训练分布内小命令不响应（官方双侧一致），行走验收改 vx=0.8 ≥2.0m。**残余登记（非我方偏差）**：0.3 不走是上游权重现状，M9 自训策略时真正解决 | M8 Bite 6 | M8 Bite 8 | **已收敛**（2026-10-14；残余指向上游权重，M9 解决） |
| D69 | **Bite 9 新登记（coder 实现期发现的原版未移植行为）**：① 盲拍（滑行窗耗尽）时原版仍写 hold——known_positions 抱持目标（robotd/src/main.rs:2807,2881），我们维持跳拍不写；② RESET_AFTER_PAUSE=200ms 恢复重置（:144,2780-2784）未移植；③ 总线掉读日志限流 BUS_DROP_QUIET 未移植 | Bite 9（发现即登记） | 挂 M10（读失败处理面，随硬件批） | 待收敛（挂 M10） |
| D70 | ~~自建 box 足底在官方滑动摩擦 1.0 下能站但行走打滑；初版采用固定代理值 2.0 达到行走门槛~~ **M9a Bite 3 修订已收敛**：每侧鞋底拆成薄承重底层 + 主体上层两个 box，滑动摩擦回到官方 CPU 值 1.0；3 次闭环 vx=0.8 行走 dx=2.604783/2.604783/2.605613m。残余是 primitive 接触近似，非摩擦系数偏差 | M9a Bite 3（coder 行走实验发现） | M9a Bite 3 修订 | **已收敛（2026-10-25）** |
| D71 | M9f 自训 Velocity 挂 `duck_handmade.xml`（groundcontact 系 AABB），不用官方 Flat 默认的 `robot_walk.xml`（仅足底）。刻意让训练身体=部署仿真身体；与官方公开权重分布不同 | M9f Bite 1（用户裁决） | 永久保留（路线目标） | **有意偏离**（2026-10-29） |

## Phase 4 附：拿不准项裁决（2026-10-15）

1. odometry crate：~~建议豁免~~ ~~改挂 M9~~ **M9c Bite 2 已收敛（2026-10-29 确认）**（`impl/src/odometry.rs` 局部 FK + ALPHA16；state 的 odom 字段仍归 D50→M10）。
2. robot.remoteSessionActive：挂 M10（恒 false 补齐，随方法面齐整）。
3. account.*：不豁免，并入 D60（updaterd 的 Hub 网络面，M10）。
4. pad.* / padd：**不豁免**，M10 真机批重新评估——用户有 Xbox 手柄，原版 padd 正是用 Xbox Wireless Controller 开发测试（upstream/padd/）；pad.bindings/bind 的配置写面属性随 D52。
5. robotctl monitor：口径注记——极简版（state --every）维持已交付，富仪表板依赖 D50 载荷与硬件，挂 M10。
6. D43 通知语义拆分：无需拆分（①②已收敛，残余③永久保留）。
7. 错误码表：D61 扩为**协议级**错误码条目（含 robotd 侧 BUSY 等），挂 M10。

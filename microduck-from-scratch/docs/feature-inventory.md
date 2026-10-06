# 功能清单（feature inventory）— M8 收敛验收主线文档

> **本文档是 M8 追认生成的全量功能清单**。流程上它本应在 Phase 1（侦察期）产出，作为
> 各里程碑范围裁定的依据；实际流程是逐里程碑推进、偏差随时登记（D1–D41），到 M8 收敛
> 验收时才补齐这份总账。生成方式：从 `reference/`（原项目浅克隆，只读）的权威定义逐项
> 抄录对外表面，对照手搓工程逐项标状态，不许凭印象。
>
> 权威来源：`reference/Cargo.toml`（workspace members）、`reference/duck-ipc-proto/src/lib.rs`
> （Call 枚举 :963-1106、method 常量 :543-901、载荷 struct :2085 起）、
> `reference/robotctl/src/main.rs` / `reference/duckctl/src/main.rs`（CLI 定义）、
> `reference/deploy/robotd.toml` / `reference/deploy/updater.toml`（配置面）。
>
> 手搓侧证据：`src/main.rs`（miniduckd 方法分发 :214-365）、`src/lib.rs`（协议）、
> `src/bin/mini-duckctl.rs`、`src/updater/ipc.rs`（update.* 分发 :66-149）。
>
> **2026-10-04 用户裁决（STATE.md 决策记录 6）**：M8 只对齐非硬件依赖内容（C 线）+ 移植 BAM；表中「未登记缺口」与「建议豁免」两列**不做豁免收尾**——缺口按性质分配到 M9（仿真与训练）/ M10（驱动与硬件）/ M11（结构与外壳）批次逐一收敛，M10 完成时代码完全对齐原项目。逐项的批次归属在 M8 收尾时回填到本表。

## 统计汇总

| 处置 | 行数 |
|---|---|
| 已交付 | 39 |
| 偏差登记（沿用 D1–D41） | 33 |
| 未登记缺口（预留新 D 号 D42–D63，共 22 条） | 62 |
| 建议豁免 | 49 |
| **合计** | **183** |

口径说明：按表行计数；一行覆盖一个 RPC 方法 / 一个 CLI 命令组 / 一个配置段（键组）/
一个 crate。混合处置的行按处置列首个类目计数，次要类目在括号内注记。62 个缺口行
归并为 22 条新 D（D42–D63），逐条见文末「未登记缺口汇总」。

既有决策的影响（STATE.md 决策记录）：`examples/` 是教程配套资产，不计入本清单；
D2（FakeIo 无真实 Dynamixel 串口协议）已豁免，硬件驱动面整体按同原则处理。

## 1. 进程与 crate 面

| 功能项 | 原项目证据（文件:行号） | 手搓状态 | 处置 |
|---|---|---|---|
| robotd 进程 | reference/Cargo.toml:7；robotd/ | miniduckd（src/main.rs） | 已交付（M0–M6，milestones/m0-ipc-skeleton/acceptance.md 起；API 子集见 D1） |
| updaterd 进程 | reference/Cargo.toml:7；updater/src/main.rs | mini-updaterd（src/bin/mini-updaterd.rs） | 已交付（M7，milestones/m7-updaterd/acceptance.md；残余 D37–D41） |
| configd 进程 | reference/Cargo.toml:7；configd/ | 无 | 未登记缺口（D42） |
| btd 进程 | reference/Cargo.toml:7；btd/ | 无 | 建议豁免（BLE 无线电栈无硬件；chorale 行为决策已登记 D31） |
| mediad 进程 | reference/Cargo.toml:7；mediad/ | 无 | 建议豁免（相机/GStreamer/WebRTC 硬件栈） |
| padd 进程 | reference/Cargo.toml:7；padd/ | 无 | 建议豁免（手柄 evdev 硬件输入） |
| tofd 进程 | reference/Cargo.toml:7；tof/Cargo.toml（[[bin]] tofd） | 无 | 建议豁免（ToF/头 IMU I²C 硬件） |
| duck-ipc-proto crate（协议只许 serde/serde_json/semver） | duck-ipc-proto/src/lib.rs:28-30 | 协议内联 src/lib.rs（97 行迷你版） | 偏差登记（D7：M8 拆协议 crate） |
| duck-control crate（控制/观测/模型/总线） | duck-control/ | 内联 src/{control,obs,model,io,policy,safety,scheduler}.rs | 未登记缺口（D52） |
| robotd-params crate（配置参数面） | robotd-params/ | 无；阈值硬编码（src/main.rs:32-36 等） | 未登记缺口（D52） |
| kinematics crate（FK/IK、robot.look/model） | kinematics/ | 无 | 未登记缺口（D55） |
| odometry crate（接触里程计） | odometry/ | 无 | 建议豁免（估计器算法，教学主线外；state 缺 odom 字段已登记 D50） |
| duck-ble crate | reference/Cargo.toml:7 | 无 | 建议豁免（BLE 库） |
| duck-ether crate | reference/Cargo.toml:7（不在 default-members :21） | 无 | 建议豁免（传输库） |
| duck-detect crate（+bin duck-bench） | duck-detect/Cargo.toml | 无 | 建议豁免（NPU 检测器） |
| pet-detect crate（+bins detect/features） | pet-detect/Cargo.toml | 无 | 建议豁免（麦克风抚摸分类器） |
| pad-imu crate | reference/Cargo.toml:7 | 无 | 建议豁免（手柄 IMU 硬件） |
| sounds crate（声库合成） | reference/Cargo.toml:7 | 无 | 建议豁免（音频硬件/声库） |
| tof crate（驱动库） | reference/Cargo.toml:7 | 无 | 建议豁免（ToF 硬件） |
| uyvy crate（像素格式） | reference/Cargo.toml:7 | 无 | 建议豁免（相机管线） |
| test-support crate | reference/Cargo.toml:7 | 无 | 建议豁免（原版内部测试工具） |
| xtask crate（发布工程） | reference/Cargo.toml:7 | 无 | 建议豁免（工程工具） |
| robotctl CLI | robotctl/src/main.rs:124 | mini-duckctl（src/bin/mini-duckctl.rs）子集 | 已交付（子集 M0–M7；逐命令对照见 §3） |
| duckctl CLI（BLE 客户端） | duckctl/src/main.rs:889 | 无 | 建议豁免（无蓝牙栈；其方法面缺口已在 §2 逐条登记） |

## 2. RPC 方法面（Call 枚举逐方法 + 通知）

证据列格式：`duck-ipc-proto/src/lib.rs` 的 method 常量行号 / Call 变体行号。

| 功能项 | 原项目证据 | 手搓状态 | 处置 |
|---|---|---|---|
| hello | :543 / :965 | miniduckd src/main.rs:215；mini-updaterd src/updater/ipc.rs:67；载荷缺 daemon_version/revision | 已交付（M0）；版本差异 D15、溯源字段缺口入 D62 |
| update.check | :551 / :968 | ipc.rs:71；返回 {available, current} | 已交付（M7）；CheckResult 形状差异入 D61 |
| update.apply | :552 / :969 | ipc.rs:119；参数 {version} 而非 {component, target, options} | 已交付（M7）；参数/结果形状入 D61 |
| update.rollback | :553 / :970 | ipc.rs:138 | 已交付（M7）；形状入 D61 |
| update.resetToGolden | :554 / :971 | 无 | 未登记缺口（D48；golden 兜底同 D37） |
| update.select | :555 / :972 | 无 | 未登记缺口（D48） |
| update.pin | :556 / :973 | 无 | 未登记缺口（D48） |
| update.status | :557 / :974 | ipc.rs:95；扁平 {phase, current, previous, pending} | 已交付（M7）；ComponentStatus 形状入 D61、degraded 裁决 D41 |
| update.listInstalled | :558 / :975 | 无 | 未登记缺口（D48） |
| update.log | :559 / :976 | ipc.rs:113 | 已交付（M7）；LogEntry 形状入 D61 |
| update.show | :560 / :978 | 无 | 未登记缺口（D48；transcript 本体在 D41） |
| update.subscribe | :561 / :980 | 无 | 偏差登记（D41） |
| update.progress 通知 | :564（Progress :3105） | 无 | 偏差登记（D41） |
| robot.safeToRestart | :572 / :983 | 无 | 未登记缺口（D47） |
| robot.health | :574 / :984 | main.rs:219（真实判定，M5） | 已交付（M5）；载荷字段缺口 D49 |
| robot.modelApi | :576 / :985 | 无 | 未登记缺口（D47） |
| robot.remoteSessionActive | :578 / :986 | 无 | 建议豁免（WebRTC 会话存在性，mediad 家族；拿不准，见文末） |
| robot.move {vx,vy,vyaw}（通知式） | :597 / :990；MoveParams :2110 | lib.rs:114 MoveParams + main.rs:266 robot.move 双路径（M8 Bite 1；robot.drive 已删） | 已交付（M8 Bite 1）；head/mouth 通知语义与统一 apply_intent 入口残余入 D43 |
| robot.head（通知式） | :599 / :992；HeadParams :2126 | main.rs:335（请求式，参数名一致） | 已交付（M6）；连续意图通知语义差异入 D43 |
| robot.look（凝视 IK） | :602 / :994；LookParams :2140 | 无 | 未登记缺口（D55） |
| robot.stop | :604 / :995 | 无（deadman 500ms 兜底不等价） | 未登记缺口（D54） |
| robot.enable {on, toggle} | :606 / :996；EnableParams :2792-2808 | robot.enable/disable 无参二分（main.rs:229-236） | 已交付（M5）；形状差异（无 toggle、多出 disable）D44 |
| robot.init（无策略上电站立） | :624 / :998 | 无 | 未登记缺口（D53） |
| robot.relax（卸力） | :630 / :1000 | 无（disable 的斜坡+卸 torque 相近而非同一物） | 未登记缺口（D53） |
| robot.rebootMotors | :636 / :1002 | 无 | 偏差登记（D12 残余：RobotIo::reboot 未实现） |
| robot.do | :647 / :1004；DoParams :2303 | main.rs:263 | 已交付（M6）；fallen 拒绝层级差异 D36 |
| robot.pose（站姿偏移 z/roll/pitch） | :652 / :1006；PoseParams :2314 | 无 | 未登记缺口（D56） |
| robot.mouth {open} | :655 / :1008；MouthParams :2338 | main.rs:320，参数名 {position} | 已交付（M6）；参数名差异 D57 |
| robot.sound | :666 / :1010；SoundParams :2199 | 无 | 建议豁免（无音频硬件/声库；同 D31 理由） |
| robot.theremin | :680 / :1012 | 无 | 偏差登记（D31 明确不实现） |
| robot.chorale | :692 / :1015 | 无 | 偏差登记（D31） |
| chorale.subscribe | :885 / :1018 | 无 | 偏差登记（D31） |
| chorale.beacon | :888 / :1020 | 无 | 偏差登记（D31） |
| chorale.heard | :891 / :1022 | 无 | 偏差登记（D31） |
| robot.shutdown | :694 / :1024 | 无 | 未登记缺口（D58） |
| robot.mode | :700 / :1026 | 无 | 偏差登记（D28） |
| robot.setMode | :708 / :1028 | 无 | 偏差登记（D28/D30） |
| robot.policies | :714 / :1030；PoliciesResult :2395 | 无 | 未登记缺口（D59） |
| robot.model | :718 / :1032；ModelResult :3844 | 无 | 未登记缺口（D55） |
| robot.loadPolicy | :725 / :1034 | 无 | 偏差登记（D30 热换不实现） |
| robot.reloadPolicies | :733 / :1036 | 无 | 偏差登记（D30） |
| policy.check | :744 / :1040 | 无 | 未登记缺口（D60；与 D39 网络 source 同源） |
| policy.install | :746 / :1042 | 无 | 未登记缺口（D60） |
| policy.fetch | :748 / :1044 | 无 | 未登记缺口（D60） |
| policy.search | :750 / :1046 | 无 | 未登记缺口（D60） |
| detector.check | :758 / :1050 | 无 | 建议豁免（检测器 NPU 硬件栈） |
| detector.install | :760 / :1052 | 无 | 建议豁免（同上） |
| account.login | :775 / :1056 | 无 | 建议豁免（HF OAuth 远程访问栈；拿不准，见文末） |
| account.status | :777 / :1058 | 无 | 建议豁免（同上） |
| account.logout | :779 / :1060 | 无 | 建议豁免（同上） |
| robot.subscribe {hz} + SubscribeResult ack | :782 / :1061；SubscribeParams :2742、SubscribeResult :2759 | 无；robot.state 兼作订阅入口（main.rs:223） | 未登记缺口（D45；hz 逐订阅者降频已登记 D25） |
| robot.state 通知 | :791；RobotState :3708 | main.rs:380-397 推送（50Hz，M5） | 已交付（M5）；载荷字段缺口 D50、latest-wins 语义 D25 |
| net.status | :801 / :1063 | 无 | 未登记缺口（D42） |
| net.scan | :803 / :1064 | 无 | 未登记缺口（D42） |
| net.connect | :805 / :1065 | 无 | 未登记缺口（D42） |
| net.forget | :807 / :1066 | 无 | 未登记缺口（D42） |
| system.info | :810 / :1069 | 无 | 未登记缺口（D42） |
| system.services | :812 / :1070 | 无 | 未登记缺口（D42） |
| system.logs | :814 / :1072 | 无 | 未登记缺口（D42） |
| system.setName | :816 / :1073 | 无 | 未登记缺口（D42） |
| system.reboot | :818 / :1074 | 无 | 未登记缺口（D42） |
| system.pairingPin | :820 / :1080 | 无 | 建议豁免（BLE 配对 PIN） |
| system.setPairingPin | :822 / :1081 | 无 | 建议豁免（同上） |
| system.authenticate | :824 / :1089 | 无 | 建议豁免（BLE 传输层鉴权） |
| pad.status | :838 / :1092 | 无 | 建议豁免（手柄硬件） |
| pad.pair | :840 / :1093 | 无 | 建议豁免（同上） |
| pad.forget | :842 / :1094 | 无 | 建议豁免（同上） |
| pad.bindings | :843 / :1095 | 无 | 建议豁免（手柄硬件；配置写入面属性拿不准，见文末） |
| pad.bind | :844 / :1096 | 无 | 建议豁免（同上） |
| robot.skills | :845 / :1097；SkillsResult :4378 | main.rs:314，{skills} 含内置两名 | 已交付（M6）；SkillsResult.built_in 字段缺口入 D59 |
| robot.setSkill | :846 / :1098 | 无 | 未登记缺口（D59） |
| robot.removeSkill | :847 / :1099 | 无 | 未登记缺口（D59） |
| pad.input / pad.report | :861/:864 / :1101 | 无 | 建议豁免（padd 硬件流） |
| tof.stream / tof.frame | :877/:894 / :1103 | 无 | 建议豁免（tofd 硬件流） |
| head_imu.stream / head_imu.frame | :898/:901 / :1105 | 无 | 建议豁免（BMI088 硬件流） |
| media.frame（非 Call，二进制尾帧） | :549；MediaFrameHeader :3051 | 无 | 建议豁免（mediad 相机帧） |

## 3. CLI 对照

| 功能项 | 原项目证据 | 手搓状态 | 处置 |
|---|---|---|---|
| robotctl frame | robotctl/src/main.rs:126 | 无 | 建议豁免（相机帧，mediad 家族） |
| robotctl net status/scan/connect/forget | :132、:369-402 | 无 | 未登记缺口（D42） |
| robotctl system info/set-name/pin/set-pin/reboot | :139、:404-439 | 无 | 未登记缺口（D42；pin 两条随 BLE 豁免） |
| robotctl robot init | :450 | 无 | 未登记缺口（D53） |
| robotctl robot enable [--off/--toggle] | :465 | mini-duckctl enable/disable | 已交付（M5）；toggle 缺失入 D44 |
| robotctl robot relax | :484 | 无 | 未登记缺口（D53） |
| robotctl robot reboot-motors | :498 | 无 | 偏差登记（D12） |
| robotctl robot do | :514 | mini-duckctl do | 已交付（M6） |
| robotctl robot mode | :521 | 无 | 偏差登记（D28） |
| robotctl robot look | :535 | 无 | 未登记缺口（D55） |
| robotctl quack | :157 | 无 | 建议豁免（音频） |
| robotctl chorale | :168 | 无 | 偏差登记（D31） |
| robotctl theremin | :190 | 无 | 偏差登记（D31） |
| robotctl configure | :204 | 无 | 未登记缺口（D52 配置面连带） |
| robotctl pad status/bindings/bind/reset/pair/forget | :228、:808-897 | 无 | 建议豁免（手柄硬件） |
| robotctl update check/apply/rollback/status/log | :235、:1079-1196 | mini-duckctl update 同五条（mini-duckctl.rs:197-236） | 已交付（M7） |
| robotctl update reset-to-golden/select/pin | :1156-1176 | 无 | 未登记缺口（D48） |
| robotctl update show | :1197 | 无 | 未登记缺口（D48） |
| robotctl update watch | :1211 | 无 | 偏差登记（D41 subscribe） |
| robotctl account login/status/logout | :252、:898 | 无 | 建议豁免（OAuth） |
| robotctl policy list/load/add/remove/search/check/update/reset | :272、:958-1076 | 无 | 未登记缺口（D59/D60；load/reset 热换语义 D30） |
| robotctl duck-detector check/update | :289、:931 | 无 | 建议豁免（检测器硬件栈） |
| robotctl monitor | :307 | mini-duckctl state [--every N]（极简替代） | 已交付（M0/M5 极简版）；ToF/IMU 网格等硬件面板豁免 |
| robotctl health | :333 | mini-duckctl health | 已交付（M5） |
| robotctl version | :348 | 无 | 未登记缺口（D62） |
| robotctl completions | :362 | 无 | 建议豁免（shell 补全工程糖） |
| duckctl 全命令树（scan/ip/ssh/scp/open/logs/status/version/update/info/health/wifi/name/do/reboot-motors/account/policy/pad/reboot/call） | duckctl/src/main.rs:889-1342 | 无 | 建议豁免（BLE 客户端整机；其调用的方法面缺口已在 §2 逐条登记） |

## 4. 配置参数面

robotd.toml（reference/deploy/robotd.toml，498 行）：

| 功能项 | 原项目证据 | 手搓状态 | 处置 |
|---|---|---|---|
| [bus] port | robotd.toml:22 | 无串口（FakeIo/SimIo） | 偏差登记（D2 已豁免） |
| [bus] fast_sync_read | :33 | 无 | 偏差登记（D2） |
| [control] hz = 50 | :48 | 50Hz 硬编码（src/control.rs） | 已交付（M1/M5） |
| [control] cmd_alpha / head_alpha（命令 EMA 平滑） | :54-55 | 无（意图直通，无 EMA） | 未登记缺口（D46） |
| [control] publish_velocity_and_load | :65 | 无（state 无 velocities/currents_ma） | 未登记缺口（D50 字段面） |
| [update_gate] min_achieved_hz / stall_periods / max_consecutive_errors | :83/:93/:98 | 硬编码 src/main.rs:32-36（45Hz/500ms/10） | 已交付（M5）；配置文件面缺口入 D52 |
| [policy] enabled | :111 | MINIDUCK_POLICY 加载失败报病不退出（D19） | 已交付（M5 语义）；配置键面无，入 D52 |
| [policy] mode = walk/roller | :121 | 恒 walk | 偏差登记（D28） |
| [policy] 槽路径（walk/stand/sitstand/ground_pick/kick_left/kick_right/roulade） | :139-146 | walk/sitstand/ground_pick/roulade/kick_left 已接线（scheduler.rs:352-353） | 已交付（M6 子集）；stand 槽 D33，kick_right/roller 权重未下载未接线入 D28 注记 |
| [policy] action_scale / standing_action_scale / standing_gain_ratio / gain | :154-161 | 硬编码（walk 0.9、技能 1.0、gain 200） | 已交付（M6 数值）；standing_* D33 |
| [policy] head_lowpass / legs_lowpass | :167-168 | src/control.rs:41-44（0.5/0.7 训练值） | 已交付（M6） |
| [policy] ground_pick_period / action_scale / gain_ratio | :177-179 | 硬编码 200 拍 / φ=0.7 交还 | 已交付（M6） |
| [policy] kick_duration | :182 | 硬编码窗口 | 已交付（M6） |
| [policy] roulade_duration / action_scale / gain_ratio | :188-190 | 50 拍窗口 | 已交付（M6）；chain 重放未实现 D29 |
| [policy] voltage_adapt / nominal_voltage | :195-196 | 无（FakeIo 无电压读数） | 未登记缺口（D63） |
| [safety] fall_gravity_z / fall_debounce_ms | :207-208 | src/safety.rs 常量（-0.5/200ms） | 已交付（M5） |
| [safety] deadman_ms | :215 | 500ms | 已交付（M5） |
| [safety] gain_limp | :218 | 50 | 已交付（M5） |
| [safety] limp_fall 全系列（9 键：开关/tilt/predict/lookahead/debounce/still/max/pose_ms/pose_gain） | :239-277 | 简化跌倒响应（fallen 直接触发 Limp） | 偏差登记（D23） |
| [safety] battery_empty_shutdown | :282 | 无 | 未登记缺口（D63） |
| [audio] 全段 | :284-314 | 无 | 建议豁免（无音频硬件；同 D31 理由） |
| [theremin] 全段 | :316-358 | 无 | 偏差登记（D31） |
| [duck_detector] 全段 | :360-390 | 无 | 建议豁免（检测器硬件栈） |
| [head_imu] enabled | :392-408 | 无 | 建议豁免（BMI088 硬件） |
| [chorale] accept | :410-422 | 无 | 偏差登记（D31） |
| [media] 全段 | :424-479 | 无 | 建议豁免（mediad 硬件栈） |
| [pad_imu_head_control] | :493-498 | 无 | 建议豁免（手柄 IMU 硬件） |
| robotd.toml 文件本身（/etc/robot 配置面，installer 不覆盖） | :1-16 | 无配置文件；env 变量（MINIDUCK_POLICY 等）替代 | 未登记缺口（D52） |

updater.toml（reference/deploy/updater.toml，179 行）：

| 功能项 | 原项目证据 | 手搓状态 | 处置 |
|---|---|---|---|
| trusted_keys_dir（minisign 信任锚） | updater.toml:16 | 无（sha256-only） | 偏差登记（D38） |
| hw_rev（硬件版本兼容门） | :18 | 无 | 未登记缺口（D61） |
| state_dir | :22 | src/updater/paths.rs state/ | 已交付（M7） |
| robot_socket | :24 | gate.rs 轮询 miniduckd socket | 已交付（M7） |
| allow_dev_keys / allow_fault_injection | :30-31 | EXIT_AFTER_SWAP 故障注入已有；dev keys 无 | 已交付（M7 故障注入半边）；dev keys 入 D38 |
| check_interval（周期检查定时器） | :36 | 无 | 偏差登记（D39） |
| auto_apply | :51 | 无 | 偏差登记（D39） |
| allow_users（+allow_uids/allow_gids 机制） | :79 | uid==0 \|\| socket owner 单点（peer.rs） | 偏差登记（D40） |
| [component.daemon] install_dir / keep_previous | :84-85 | install/releases + journal previous 推导 | 已交付（M7） |
| golden | :94 | 无 | 偏差登记（D37）；方法面入 D48 |
| source github_releases（tag/ref/staging 前缀） | :99-119 | LocalDir 单一 source | 偏差登记（D39） |
| [component.daemon.on_apply] units 重启表 | :126-157 | spawn/kill miniduckd 替代 systemctl | 偏差登记（D37） |
| [component.daemon.health] probe/timeout | :167-169 | gate.rs 500ms 轮询+预算 | 已交付（M7） |
| updater.toml 文件本身（+updater.example.toml 注释参考） | updater.toml:1-10 | env 变量（MINIDUCK_UPDATER_*）替代 | 未登记缺口（D52） |

## 5. 通知、载荷与协议机制面

| 功能项 | 原项目证据 | 手搓状态 | 处置 |
|---|---|---|---|
| RobotState 帧字段全集（t / move{requested,applied,limited_by} / head[4] / policy / safety{fallen,limp,gravity,gain} / loop{hz,missed} / joints / targets / velocities / currents_ma / odom / t_ns / imu{gyro,quat} / frames / skeleton / theremin / chorale） | duck-ipc-proto/src/lib.rs:3708-3798 | {tick, positions, imu{gyro,gravity,quat}, obs, action, fallen, enabled, gain, torque, skill}（src/main.rs:380-397） | 未登记缺口（D50） |
| HealthResult 字段全集（healthy / degraded / reason 单值 / battery / motors / cpu_temp_c / cpu_throttle / control_loop / bus / imu） | lib.rs:3432-3501 | {healthy, reason[], tick, uptime_s, reads, writes, skipped_reads, consecutive_read_errors, achieved_hz}（main.rs:171-181） | 未登记缺口（D49） |
| SubscribeResult ack（accepted / walk / stand / unavailable / sitstand / ground_pick / skills） | lib.rs:2759-2787 | state 入口只回 {subscribed: true}（main.rs:223-226） | 未登记缺口（D45） |
| Progress 通知载荷（component/phase/percent/detail） | lib.rs:3105 | 无 | 偏差登记（D41） |
| 应用错误码表（BUSY=1、UNKNOWN_COMPONENT=2、PREFLIGHT_FAILED=4 … PERMISSION_DENIED=14） | lib.rs:909-945 | -32001..-32004（src/updater/mod.rs） | 未登记缺口（D61） |
| API_VERSION = 37 | lib.rs:426 | = 1（src/lib.rs:27） | 偏差登记（D15） |
| 协议常量 JOINT_NAMES / POLICY_OBS_LEN=61 / POLICY_ACTION_LEN=14 | lib.rs:522/:435/:439 | src/model.rs 关节常量；obs 61 维（M2） | 已交付（M1/M2，milestones/m2-obs-vector/acceptance.md） |
| Lane 连接模型（Prompt/Slow/Operation/Stream 四分通道） | lib.rs:1146-1155 | 单连接 select! 多路复用（main.rs:196-403） | 未登记缺口（D51） |
| Service 路由与五 socket 布局 | lib.rs:1115、:467-488 | 双 socket（/tmp/miniduckd.sock、/tmp/mini-updaterd.sock） | 未登记缺口（D42 进程面连带） |
| identity.json 发布（/run/<service>/identity.json） | lib.rs:498-511 | 无 | 未登记缺口（D62） |
| UPDATE_MAX_SILENCE_SECONDS = 600 契约 | lib.rs:457 | 无 | 未登记缺口（D61） |
| socket 默认路径 /run/*.sock | lib.rs:467-488 | /tmp/*.sock（main.rs:27、mini-duckctl.rs:194） | 建议豁免（容器无 systemd RuntimeDirectory；0660 鉴权语义已对齐 D8） |
| HelloResult {daemon_version, revision} | lib.rs:3040-3047 | {service, api_version} | 未登记缺口（D62） |
| 教学附加帧字段（obs/action/enabled/torque/skill/tick，原版无） | —（手搓新增） | src/main.rs:382-397 | 建议豁免（教程观测窗口；M8 终态裁决时保留或改名） |

## 未登记缺口汇总（预留 D42–D63）

以下 22 条为本次清单新发现的偏差簿外缺口，D 号预留待登记入 deviations.md：

- **D42** configd 缺席，net.*（4 条）与 system.* 前 5 条方法面连带缺席（btd/mediad/padd/tofd 四进程建议豁免单列）。
- **D43** ~~robot.drive {vx,vyaw} 请求式 vs 原版 robot.move {vx,vy,vyaw} 通知式：vy 侧向未开放，且连续意图（move/head/mouth）的通知语义（无应答、last-writer-wins）未实现~~ M8 Bite 1 已收敛 robot.move 本体（通知/请求双形态 + vy + robot.drive 删除）。残余：head/mouth 通知语义未开；原版对任何方法的 notification 走 apply_intent 统一入口静默处理，我们只给 robot.move 开了通知语义（预定随 enable/subscribe Bite 收敛）。
- **D44** robot.enable {on, toggle} vs 手搓 enable/disable 无参二分：缺 toggle（手柄 Start 语义），多出原版没有的 robot.disable。
- **D45** robot.subscribe {hz} + SubscribeResult ack（walk/stand/unavailable/skills 名单）缺席，robot.state 方法名兼作订阅入口（hz 降频语义已登记 D25）。
- **D46** 命令 EMA 平滑缺席（[control] cmd_alpha/head_alpha；手搓意图直通，只有动作侧训练低通）。
- **D47** updaterd 预检面 robot.safeToRestart / robot.modelApi 缺席（apply 前「现在重启安全吗」无从问起）。
- **D48** update.* 缺五条方法：resetToGolden（D37 关联）/ select / pin / listInstalled / show（D41 transcript 关联）。
- **D49** robot.health 载荷字段缺口：degraded / 单值 reason / battery / motors / cpu_temp_c / cpu_throttle / control_loop / bus / imu 九个结构缺席（手搓为扁平教学字段）。
- **D50** robot.state 载荷字段缺口：t / move 三元组 / head / policy / loop / joints+targets 命名 / velocities / currents_ma / odom / t_ns / frames / skeleton 缺席。
- **D51** Lane 连接模型（Prompt/Slow/Operation/Stream 四分通道）缺席，手搓单连接多路复用。
- **D52** duck-control / robotd-params crate 边界缺席：协议+控制+参数全内联单 crate，robotd.toml / updater.toml 配置面整体由 env 变量替代（D5 残余的配置文件面并入此条裁决）。
- **D53** robot.init / robot.relax 缺席：无策略上电站立与直接卸力两个原语没有入口。
- **D54** robot.stop 缺席：零速度意图只能靠 deadman 500ms 超时兜底。
- **D55** kinematics 缺席：robot.look（凝视 IK）、robot.model（静态几何）、state 的 frames/skeleton 全无。
- **D56** robot.pose（站姿 z/roll/pitch 偏移）缺席。
- **D57** robot.mouth 参数名 {position} vs 原版 {open}，线形状不兼容。
- **D58** robot.shutdown（坐下后关机）缺席。
- **D59** robot.policies / robot.setSkill / robot.removeSkill 缺席（策略槽查询与技能配置写面；SkillsResult.built_in 字段并入）。
- **D60** policy.* 四条（check/install/fetch/search）缺席：updaterd 的 Hub 网络面，与 D39 同源。
- **D61** update.* 载荷与错误码形状迷你化：CheckResult/ApplyResult/ComponentStatus/LogEntry/RunTranscript 结构、应用错误码 1–14（手搓 -32001..-32004）、hw_rev 兼容门、UPDATE_MAX_SILENCE 契约。
- **D62** identity.json 发布、HelloResult daemon_version/revision、robotctl version 溯源面缺席。
- **D63** [policy] voltage_adapt（电压自适应）与 [safety] battery_empty_shutdown（低电关机）缺席。

## 拿不准归类的项（提请 M8 裁决时复核）

1. **odometry crate**（§1）：标了建议豁免，但它是纯软件估计器（无硬件依赖），教学价值不低；若 M8 认为该补，改列缺口并入 D50 的 odom 字段。
2. **robot.remoteSessionActive**（§2）：标了建议豁免（WebRTC/mediad 家族），但它是 robotd 的方法、可以恒答 false——若 M8 要求方法面齐整，改列缺口（可并入 D47 预检面）。
3. **account.\***（§2）：标了建议豁免（HF OAuth 远程访问），但其服务方是 updaterd、与 policy.* 同属 Hub 网络面——也可并入 D60。
4. **pad.bindings / pad.bind**（§2）：标了建议豁免（手柄硬件），但本质是 robotd.toml 配置写面（与 robot.setSkill 同类）——若 D52 裁决要配置写面，这两条应改列缺口。
5. **robotctl monitor**（§3）：按极简替代标了已交付，但富仪表板（LoopState/总线计数/ToF 网格）大部分依赖 D50 载荷与硬件——「已交付」的口径偏宽。
6. **D43 的通知语义**：head/mouth/drive 连续意图的通知化归并在 D43 一条里，若 M8 认为语义差异应独立于 vy 缺失，可拆分。
7. **错误码表归 D61**：错误码同时涉及 robotd 侧（BUSY 于 robot.do 等），D61 目前只写在 update.* 语境——裁决时可能需要扩成协议级条目。

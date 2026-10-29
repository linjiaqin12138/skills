# 功能清单（feature inventory）— M8 收敛验收主线文档

> **本文档是 M8 追认生成的全量功能清单**。流程上它本应在 Phase 1（侦察期）产出，作为
> 各里程碑范围裁定的依据；实际流程是逐里程碑推进、偏差随时登记（D1–D41），到 M8 收敛
> 验收时才补齐这份总账。生成方式：从 `upstream/`（原项目浅克隆，只读）的权威定义逐项
> 抄录对外表面，对照手搓工程逐项标状态，不许凭印象。（2026-10-15 行号回填：换机重克隆后
> reference 回到 daemon-v0.14.1（commit 344925c，API_VERSION=31），表内证据行号已按此
> 检出逐项重核回填；本头部两段中的行号按规约保留 2026-10-04 旧检出原值，与表内不一致处
> 以表内为准。）
>
> 权威来源：`upstream/Cargo.toml`（workspace members）、`upstream/duck-ipc-proto/src/lib.rs`
> （Call 枚举 :963-1106、method 常量 :543-901、载荷 struct :2085 起）、
> `upstream/robotctl/src/main.rs` / `upstream/duckctl/src/main.rs`（CLI 定义）、
> `upstream/deploy/robotd.toml` / `upstream/deploy/updater.toml`（配置面）。
>
> 手搓侧证据：`impl/src/main.rs`（miniduckd 方法分发 :214-365）、`impl/src/lib.rs`（协议）、
> `impl/src/bin/mini-duckctl.rs`、`impl/src/updater/ipc.rs`（update.* 分发 :66-149）。
>
> **2026-10-04 用户裁决（STATE.md 决策记录 6）**：M8 只对齐非硬件依赖内容（C 线）+ 移植 BAM；表中「未登记缺口」与「建议豁免」两列**不做豁免收尾**——缺口按性质分配到 M9（仿真与训练）/ M10（驱动与硬件）/ M11（结构与外壳）批次逐一收敛，M10 完成时代码完全对齐原项目。逐项的批次归属在 M8 收尾时回填到本表。

## 统计汇总

| 处置 | 行数 |
|---|---|
| 已交付 | 44 |
| 偏差登记（沿用 D1–D41） | 33 |
| 未登记缺口（预留新 D 号 D42–D63，共 22 条；D43/D44/D45/D47 已于 M8 Bite 1/2/3/4 收敛） | 57 |
| 建议豁免 | 49 |
| **合计** | **183** |

2026-10-15 Phase 4 裁决：批次归属已回填到处置列（M9=仿真与训练、M10=驱动与硬件/工程面、M11=结构与外壳）；D13/D20/D24/D36/D65 已于 Bite 9/10 收敛。

口径说明：按表行计数；一行覆盖一个 RPC 方法 / 一个 CLI 命令组 / 一个配置段（键组）/
一个 crate。混合处置的行按处置列首个类目计数，次要类目在括号内注记。缺口行
归并为 22 条新 D（D42–D63），逐条见文末「未登记缺口汇总」；D43（M8 Bite 1/2）、
D44（M8 Bite 2）、D45（M8 Bite 3，hz 降频残余仍归 D25）、D47（M8 Bite 4，残余 D65/D66/D67）已收敛，其余 18 条挂账待批次收敛。

既有决策的影响（STATE.md 决策记录）：`impl/examples/` 是教程配套资产，不计入本清单；
D2（FakeIo 无真实 Dynamixel 串口协议）已豁免，硬件驱动面整体按同原则处理。

## 1. 进程与 crate 面

| 功能项 | 原项目证据（文件:行号） | 手搓状态 | 处置 |
|---|---|---|---|
| robotd 进程 | upstream/Cargo.toml:7；robotd/ | miniduckd（impl/src/main.rs） | 已交付（M0–M6，book/m00-ipc-skeleton/acceptance.md 起；API 子集见 D1） |
| updaterd 进程 | upstream/Cargo.toml:7；updater/src/main.rs | mini-updaterd（impl/src/bin/mini-updaterd.rs） | 已交付（M7，book/m07-updaterd/acceptance.md；残余 D37–D41） |
| configd 进程 | upstream/Cargo.toml:7；configd/ | 无 | 未登记缺口（D42；挂 M10） |
| btd 进程 | upstream/Cargo.toml:7；btd/ | 无 | 建议豁免（BLE 无线电栈无硬件；chorale 行为决策已登记 D31） |
| mediad 进程 | upstream/Cargo.toml:7；mediad/ | 无 | 建议豁免（相机/GStreamer/WebRTC 硬件栈） |
| padd 进程 | upstream/Cargo.toml:7；padd/ | 无 | 挂 M10 再评估（Phase 4 裁决：用户有 Xbox 手柄，原版 padd 正是 Xbox 开发测试设备；原豁免理由：手柄 evdev 硬件输入） |
| tofd 进程 | upstream/Cargo.toml:7；tof/Cargo.toml（[[bin]] tofd） | 无 | 建议豁免（ToF/头 IMU I²C 硬件） |
| duck-ipc-proto crate（协议只许 serde/serde_json/semver） | duck-ipc-proto/src/lib.rs:28-30 | 协议内联 impl/src/lib.rs（97 行迷你版） | 偏差登记（D7：M8 拆协议 crate；挂 M10） |
| duck-control crate（控制/观测/模型/总线） | duck-control/ | 内联 impl/src/{control,obs,model,io,policy,safety,scheduler}.rs | 未登记缺口（D52；挂 M10） |
| robotd-params crate（配置参数面） | robotd-params/ | 无；阈值硬编码（impl/src/main.rs:43-47 等） | 未登记缺口（D52；挂 M10） |
| kinematics crate（FK/IK、robot.look/model） | kinematics/ | 无 | 未登记缺口（D55；挂 M10） |
| odometry crate（接触里程计） | odometry/ | impl/src/odometry.rs（M9c Bite 2：局部双足 FK + ALPHA16；内部 FrameSnapshot；线上 odom 字段仍归 D50） | 已交付并确认（M9c；D50→M10） |
| duck-ble crate | upstream/Cargo.toml:7 | 无 | 建议豁免（BLE 库） |
| duck-ether crate | upstream/Cargo.toml:7（不在 default-members :21） | 无 | 建议豁免（传输库） |
| duck-detect crate（+bin duck-bench） | duck-detect/Cargo.toml | 无 | 建议豁免（NPU 检测器） |
| pet-detect crate（+bins detect/features） | pet-detect/Cargo.toml | 无 | 建议豁免（麦克风抚摸分类器） |
| pad-imu crate | upstream/Cargo.toml:7 | 无 | 建议豁免（手柄 IMU 硬件） |
| sounds crate（声库合成） | upstream/Cargo.toml:7 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原豁免理由：音频硬件/声库） |
| tof crate（驱动库） | upstream/Cargo.toml:7 | 无 | 建议豁免（ToF 硬件） |
| uyvy crate（像素格式） | upstream/Cargo.toml:7 | 无 | 建议豁免（相机管线） |
| test-support crate | upstream/Cargo.toml:7 | 无 | 建议豁免（原版内部测试工具） |
| xtask crate（发布工程） | upstream/Cargo.toml:7 | 无 | 建议豁免（工程工具） |
| robotctl CLI | robotctl/src/main.rs:124 | mini-duckctl（impl/src/bin/mini-duckctl.rs）子集 | 已交付（子集 M0–M7；逐命令对照见 §3） |
| duckctl CLI（BLE 客户端） | duckctl/src/main.rs:894 | 无 | 建议豁免（无蓝牙栈；其方法面缺口已在 §2 逐条登记） |

## 2. RPC 方法面（Call 枚举逐方法 + 通知）

证据列格式：`duck-ipc-proto/src/lib.rs` 的 method 常量行号 / Call 变体行号。

| 功能项 | 原项目证据 | 手搓状态 | 处置 |
|---|---|---|---|
| hello | :477 / :899 | miniduckd impl/src/main.rs:374；mini-updaterd impl/src/updater/ipc.rs:67；载荷缺 daemon_version/revision | 已交付（M0）；版本差异 D15（挂 M10）、溯源字段缺口入 D62（挂 M10） |
| update.check | :485 / :902 | ipc.rs:71；返回 {available, current} | 已交付（M7）；CheckResult 形状差异入 D61（挂 M10） |
| update.apply | :486 / :903 | ipc.rs:119；参数 {version} 而非 {component, target, options} | 已交付（M7）；参数/结果形状入 D61（挂 M10） |
| update.rollback | :487 / :904 | ipc.rs:138 | 已交付（M7）；形状入 D61（挂 M10） |
| update.resetToGolden | :488 / :905 | 无 | 未登记缺口（D48；挂 M10；golden 兜底同 D37（挂 M10）） |
| update.select | :489 / :906 | 无 | 未登记缺口（D48；挂 M10） |
| update.pin | :490 / :907 | 无 | 未登记缺口（D48；挂 M10） |
| update.status | :491 / :908 | ipc.rs:95；扁平 {phase, current, previous, pending} | 已交付（M7）；ComponentStatus 形状入 D61（挂 M10）、degraded 裁决 D41（挂 M10） |
| update.listInstalled | :492 / :909 | 无 | 未登记缺口（D48；挂 M10） |
| update.log | :493 / :910 | ipc.rs:113 | 已交付（M7）；LogEntry 形状入 D61（挂 M10） |
| update.show | :494 / :912 | 无 | 未登记缺口（D48；挂 M10；transcript 本体在 D41（挂 M10）） |
| update.subscribe | :495 / :914 | 无 | 偏差登记（D41；挂 M10） |
| update.progress 通知 | :498（Progress :3001） | 无 | 偏差登记（D41；挂 M10） |
| robot.safeToRestart | :506 / :917 | main.rs:516 + control.rs 记账 moving/fallen（M8 Bite 4） | ~~未登记缺口（D47）~~ 已交付（M8 Bite 4；验收 book/m08-convergence/acceptance.md Bite 4） |
| robot.health | :508 / :918 | main.rs:378（真实判定，M5） | 已交付（M5）；载荷字段缺口 D49（挂 M10） |
| robot.modelApi | :510 / :919 | main.rs:521 返回 lib.rs MODEL_API=2（M8 Bite 4） | ~~未登记缺口（D47）~~ 已交付（M8 Bite 4）；updater 侧预检语义残余 D65（已收敛，Bite 9/10） |
| robot.remoteSessionActive | :512 / :920 | 无 | 挂 M10（恒 false 补齐；原豁免理由：WebRTC 会话存在性，mediad 家族） |
| robot.move {vx,vy,vyaw}（通知式） | :531 / :924；MoveParams :2044 | lib.rs:129 MoveParams + main.rs:348/:421 robot.move 双路径（M8 Bite 1；robot.drive 已删） | 已交付（M8 Bite 1）；head/mouth 通知语义与统一 apply_intent 入口残余入 D43（主体 Bite 1/2 已收敛；残余③ CLI move 请求式 Phase 4 裁决永久保留：教学工具定位） |
| robot.head（通知式） | :533 / :926；HeadParams :2060 | main.rs:358/:507（请求式，参数名一致） | 已交付（M6）；连续意图通知语义差异入 D43 |
| robot.look（凝视 IK） | :536 / :928；LookParams :2074 | 无 | 未登记缺口（D55；挂 M10） |
| robot.stop | :538 / :929 | 无（deadman 500ms 兜底不等价） | 未登记缺口（D54；挂 M10） |
| robot.enable {on, toggle} | :540 / :930；EnableParams :2688-2704 | robot.enable {on, toggle}（main.rs:402-416，toggle daemon 侧翻转、永不拒绝、reason 逐字对齐；robot.disable 已删） | ~~已交付（M5）；形状差异（无 toggle、多出 disable）D44~~ M8 Bite 2 已对齐 |
| robot.init（无策略上电站立） | :558 / :932 | 无 | 未登记缺口（D53；挂 M10） |
| robot.relax（卸力） | :564 / :934 | 无（disable 的斜坡+卸 torque 相近而非同一物） | 未登记缺口（D53；挂 M10） |
| robot.rebootMotors | :570 / :936 | 无 | 偏差登记（D12 残余：RobotIo::reboot 未实现；挂 M10） |
| robot.do | :581 / :938；DoParams :2237 | main.rs:440 | 已交付（M6）；D36 已收敛（Bite 10：fallen 永不入门口裁决，!enabled/!homed 拒绝逐字对齐原版） |
| robot.pose（站姿偏移 z/roll/pitch） | :586 / :940；PoseParams :2248 | 无 | 未登记缺口（D56；挂 M10） |
| robot.mouth {open} | :589 / :942；MouthParams :2272 | main.rs:353/:498，参数名 {position} | 已交付（M6）；参数名差异 D57（挂 M10） |
| robot.sound | :600 / :944；SoundParams :2133 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原：建议豁免，无音频硬件/声库） |
| robot.theremin | :614 / :946 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原：偏差登记 D31 明确不实现） |
| robot.chorale | :626 / :949 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原：偏差登记 D31） |
| chorale.subscribe | :819 / :952 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原：偏差登记 D31） |
| chorale.beacon | :822 / :954 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原：偏差登记 D31） |
| chorale.heard | :825 / :956 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原：偏差登记 D31） |
| robot.shutdown | :628 / :958 | 无 | 未登记缺口（D58；挂 M10） |
| robot.mode | :634 / :960 | 无 | 偏差登记（D28；挂 M9） |
| robot.setMode | :642 / :962 | 无 | 偏差登记（D28/D30；挂 M9） |
| robot.policies | :648 / :964；PoliciesResult :2329 | 无 | 未登记缺口（D59；挂 M10） |
| robot.model | :652 / :966；ModelResult :3581 | 无 | 未登记缺口（D55；挂 M10） |
| robot.loadPolicy | :659 / :968 | 无 | 偏差登记（D30 热换不实现；挂 M9） |
| robot.reloadPolicies | :667 / :970 | 无 | 偏差登记（D30；挂 M9） |
| policy.check | :678 / :974 | 无 | 未登记缺口（D60；挂 M10；与 D39（挂 M10）网络 source 同源） |
| policy.install | :680 / :976 | 无 | 未登记缺口（D60；挂 M10） |
| policy.fetch | :682 / :978 | 无 | 未登记缺口（D60；挂 M10） |
| policy.search | :684 / :980 | 无 | 未登记缺口（D60；挂 M10） |
| detector.check | :692 / :984 | 无 | 建议豁免（检测器 NPU 硬件栈） |
| detector.install | :694 / :986 | 无 | 建议豁免（同上） |
| account.login | :709 / :990 | 无 | 并入 D60（M10；原豁免理由：HF OAuth 远程访问栈） |
| account.status | :711 / :992 | 无 | 并入 D60（M10） |
| account.logout | :713 / :994 | 无 | 并入 D60（M10） |
| robot.subscribe {hz} + SubscribeResult ack | :716 / :995；SubscribeParams :2638、SubscribeResult :2655 | robot.subscribe（main.rs serve；hz 解析后丢弃） | 已交付（M8 Bite 3，book/m08-convergence/acceptance.md）；hz 逐订阅者降频仍归 D25（挂 M10） |
| robot.state 通知 | :725；RobotState :3487 | main.rs:539-566 推送（50Hz，M5） | 已交付（M5）；载荷字段缺口 D50（挂 M10）、latest-wins 语义 D25（挂 M10） |
| net.status | :735 / :997 | 无 | 未登记缺口（D42；挂 M10） |
| net.scan | :737 / :998 | 无 | 未登记缺口（D42；挂 M10） |
| net.connect | :739 / :999 | 无 | 未登记缺口（D42；挂 M10） |
| net.forget | :741 / :1000 | 无 | 未登记缺口（D42；挂 M10） |
| system.info | :744 / :1003 | 无 | 未登记缺口（D42；挂 M10） |
| system.services | :746 / :1004 | 无 | 未登记缺口（D42；挂 M10） |
| system.logs | :748 / :1006 | 无 | 未登记缺口（D42；挂 M10） |
| system.setName | :750 / :1007 | 无 | 未登记缺口（D42；挂 M10） |
| system.reboot | :752 / :1008 | 无 | 未登记缺口（D42；挂 M10） |
| system.pairingPin | :754 / :1014 | 无 | 建议豁免（BLE 配对 PIN） |
| system.setPairingPin | :756 / :1015 | 无 | 建议豁免（同上） |
| system.authenticate | :758 / :1023 | 无 | 建议豁免（BLE 传输层鉴权） |
| pad.status | :772 / :1026 | 无 | 挂 M10 再评估（Phase 4 裁决：用户有 Xbox 手柄，原版 padd 正是 Xbox 开发测试设备；原豁免理由：手柄硬件） |
| pad.pair | :774 / :1027 | 无 | 挂 M10 再评估（同上裁决） |
| pad.forget | :776 / :1028 | 无 | 挂 M10 再评估（同上裁决） |
| pad.bindings | :777 / :1029 | 无 | 挂 M10 再评估（Phase 4 裁决：用户有 Xbox 手柄，原版 padd 正是 Xbox 开发测试设备；原豁免理由：手柄硬件，配置写入面属性裁决见文末摘要） |
| pad.bind | :778 / :1030 | 无 | 挂 M10 再评估（同上裁决） |
| robot.skills | :779 / :1031；SkillsResult :4115 | main.rs:491，{skills} 含内置两名 | 已交付（M6）；SkillsResult.built_in 字段缺口入 D59（挂 M10） |
| robot.setSkill | :780 / :1032 | 无 | 未登记缺口（D59；挂 M10） |
| robot.removeSkill | :781 / :1033 | 无 | 未登记缺口（D59；挂 M10） |
| pad.input / pad.report | :795/:798 / :1035 | 无 | 挂 M10 再评估（Phase 4 裁决：用户有 Xbox 手柄，原版 padd 正是 Xbox 开发测试设备；原豁免理由：padd 硬件流） |
| tof.stream / tof.frame | :811/:828 / :1037 | 无 | 建议豁免（tofd 硬件流） |
| head_imu.stream / head_imu.frame | :832/:835 / :1039 | 无 | 建议豁免（BMI088 硬件流） |
| media.frame（非 Call，二进制尾帧） | :483；MediaFrameHeader :2947 | 无 | 建议豁免（mediad 相机帧） |

## 3. CLI 对照

| 功能项 | 原项目证据 | 手搓状态 | 处置 |
|---|---|---|---|
| robotctl frame | robotctl/src/main.rs:126 | 无 | 建议豁免（相机帧，mediad 家族） |
| robotctl net status/scan/connect/forget | :132、:364-398 | 无 | 未登记缺口（D42；挂 M10） |
| robotctl system info/set-name/pin/set-pin/reboot | :139、:399-435 | 无 | 未登记缺口（D42；挂 M10；pin 两条随 BLE 豁免） |
| robotctl robot init | :445 | 无 | 未登记缺口（D53；挂 M10） |
| robotctl robot enable [--off/--toggle] | :460 | mini-duckctl enable [on\|off]（disable 子命令已删；CLI 只发表决结果，toggle 是手柄 Start 的事） | ~~已交付（M5）；toggle 缺失入 D44~~ M8 Bite 2 已对齐 |
| robotctl robot relax | :479 | 无 | 未登记缺口（D53；挂 M10） |
| robotctl robot reboot-motors | :493 | 无 | 偏差登记（D12；挂 M10） |
| robotctl robot do | :509 | mini-duckctl do | 已交付（M6） |
| robotctl robot mode | :516 | 无 | 偏差登记（D28；挂 M9） |
| robotctl robot look | :530 | 无 | 未登记缺口（D55；挂 M10） |
| robotctl quack | :157 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原豁免理由：音频） |
| robotctl chorale | :168 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原：偏差登记 D31） |
| robotctl theremin | :190 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原：偏差登记 D31） |
| robotctl configure | :204 | 无 | 未登记缺口（D52 配置面连带；挂 M10） |
| robotctl pad status/bindings/bind/reset/pair/forget | :228、:803-892 | 无 | 挂 M10 再评估（Phase 4 裁决：用户有 Xbox 手柄，原版 padd 正是 Xbox 开发测试设备；原豁免理由：手柄硬件） |
| robotctl update check/apply/rollback/status/log | :235、:1074-1181 | mini-duckctl update 同五条（mini-duckctl.rs:214-254） | 已交付（M7） |
| robotctl update reset-to-golden/select/pin | :1154-1169 | 无 | 未登记缺口（D48；挂 M10） |
| robotctl update show | :1192 | 无 | 未登记缺口（D48；挂 M10） |
| robotctl update watch | :1206 | 无 | 偏差登记（D41 subscribe；挂 M10） |
| robotctl account login/status/logout | :252、:893 | 无 | 并入 D60（挂 M10；随 §2 account.* 改判，Phase 4 裁决） |
| robotctl policy list/load/add/remove/search/check/update/reset | :272、:953-1071 | 无 | 未登记缺口（D59/D60；挂 M10；load/reset 热换语义 D30（挂 M9）） |
| robotctl duck-detector check/update | :289、:926 | 无 | 建议豁免（检测器硬件栈） |
| robotctl monitor | :307 | mini-duckctl state [--every N]（极简替代） | 已交付（M0/M5 极简版）；ToF/IMU 网格等硬件面板豁免；富仪表板面板挂 M10（依赖 D50+硬件） |
| robotctl health | :333 | mini-duckctl health | 已交付（M5） |
| robotctl version | :343 | 无 | 未登记缺口（D62；挂 M10） |
| robotctl completions | :357 | 无 | 建议豁免（shell 补全工程糖） |
| duckctl 全命令树（scan/ip/ssh/scp/open/logs/status/version/update/info/health/wifi/name/do/reboot-motors/account/policy/pad/reboot/call） | duckctl/src/main.rs:894-1325 | 无 | 建议豁免（BLE 客户端整机；其调用的方法面缺口已在 §2 逐条登记） |

## 4. 配置参数面

robotd.toml（upstream/deploy/robotd.toml，498 行）：

| 功能项 | 原项目证据 | 手搓状态 | 处置 |
|---|---|---|---|
| [bus] port | robotd.toml:22 | 无串口（FakeIo/SimIo） | 偏差登记（D2 已豁免） |
| [bus] fast_sync_read | ⚠ v0.14.1 未找到（出自更高版本检出） | 无 | 偏差登记（D2） |
| [control] hz = 50 | :37 | 50Hz 硬编码（impl/src/control.rs） | 已交付（M1/M5） |
| [control] cmd_alpha / head_alpha（命令 EMA 平滑） | :43-44 | 无（意图直通，无 EMA） | 未登记缺口（D46；挂 M10） |
| [control] publish_velocity_and_load | ⚠ v0.14.1 未找到（出自更高版本检出） | 无（state 无 velocities/currents_ma） | 未登记缺口（D50 字段面；挂 M10） |
| [update_gate] min_achieved_hz / stall_periods / max_consecutive_errors | :62/:72/:77 | 硬编码 impl/src/main.rs:43-47（45Hz/500ms/10） | 已交付（M5）；配置文件面缺口入 D52（挂 M10） |
| [policy] enabled | :90 | MINIDUCK_POLICY 加载失败报病不退出（D19） | 已交付（M5 语义）；配置键面无，入 D52（挂 M10） |
| [policy] mode = walk/roller | :100 | 恒 walk | 偏差登记（D28；挂 M9） |
| [policy] 槽路径（walk/stand/sitstand/ground_pick/kick_left/kick_right/roulade） | :118-125 | walk/sitstand/ground_pick/roulade/kick_left 已接线（scheduler.rs:352-353） | 已交付（M6 子集）；stand 槽 D33（挂 M9），kick_right/roller 权重未下载未接线入 D28（挂 M9）注记 |
| [policy] action_scale / standing_action_scale / standing_gain_ratio / gain | :133-140 | 硬编码（walk 0.9、技能 1.0、gain 200） | 已交付（M6 数值）；standing_* D33（挂 M9） |
| [policy] head_lowpass / legs_lowpass | :146-147 | impl/src/control.rs:54-58（0.5/0.7 训练值） | 已交付（M6） |
| [policy] ground_pick_period / action_scale / gain_ratio | :156-158 | 硬编码 200 拍 / φ=0.7 交还 | 已交付（M6） |
| [policy] kick_duration | :161 | 硬编码窗口 | 已交付（M6） |
| [policy] roulade_duration / action_scale / gain_ratio | :167-169 | 50 拍窗口 | 已交付（M6）；chain 重放未实现 D29（挂 M9） |
| [policy] voltage_adapt / nominal_voltage | :174-175 | 无（FakeIo 无电压读数） | 未登记缺口（D63；挂 M10） |
| [safety] fall_gravity_z / fall_debounce_ms | :186-187 | impl/src/safety.rs 常量（-0.5/200ms） | 已交付（M5） |
| [safety] deadman_ms | :194 | 500ms | 已交付（M5） |
| [safety] gain_limp | :197 | 50 | 已交付（M5） |
| [safety] limp_fall 全系列（9 键：开关/tilt/predict/lookahead/debounce/still/max/pose_ms/pose_gain） | :218-256 | 机制已对齐（M9c Bite 1）；默认 OFF，env 显式开关；完整配置文件面仍缺 | 已交付（D23/D32 收敛）；配置文件面入 D52（挂 M10） |
| [safety] battery_empty_shutdown | :261 | 无 | 未登记缺口（D63；挂 M10） |
| [audio] 全段 | :263-293 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原豁免理由：无音频硬件） |
| [theremin] 全段 | :295-337 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原：偏差登记 D31） |
| [duck_detector] 全段 | :339-369 | 无 | 建议豁免（检测器硬件栈） |
| [head_imu] enabled | :371-387 | 无 | 建议豁免（BMI088 硬件） |
| [chorale] accept | :389-401 | 无 | 挂 M11 再评估（Phase 4 裁决：不豁免，自研硬件若含音频再开账；原：偏差登记 D31） |
| [media] 全段 | :403-458 | 无 | 建议豁免（mediad 硬件栈） |
| [pad_imu_head_control] | :460-477 | 无 | 建议豁免（手柄 IMU 硬件） |
| robotd.toml 文件本身（/etc/robot 配置面，installer 不覆盖） | :1-16 | 无配置文件；env 变量（MINIDUCK_POLICY 等）替代 | 未登记缺口（D52；挂 M10） |

updater.toml（upstream/deploy/updater.toml，179 行）：

| 功能项 | 原项目证据 | 手搓状态 | 处置 |
|---|---|---|---|
| trusted_keys_dir（minisign 信任锚） | updater.toml:16 | 无（sha256-only） | 偏差登记（D38；挂 M10） |
| hw_rev（硬件版本兼容门） | :18 | 无 | 未登记缺口（D61；挂 M10） |
| state_dir | :22 | impl/src/updater/paths.rs state/ | 已交付（M7） |
| robot_socket | :24 | gate.rs 轮询 miniduckd socket | 已交付（M7） |
| allow_dev_keys / allow_fault_injection | :30-31 | EXIT_AFTER_SWAP 故障注入已有；dev keys 无 | 已交付（M7 故障注入半边）；dev keys 入 D38（挂 M10） |
| check_interval（周期检查定时器） | :36 | 无 | 偏差登记（D39；挂 M10） |
| auto_apply | :51 | 无 | 偏差登记（D39；挂 M10） |
| allow_users（+allow_uids/allow_gids 机制） | :79 | uid==0 \|\| socket owner 单点（peer.rs） | 偏差登记（D40；Phase 4 裁决永久保留：SO_PEERCRED 单点门，容器单用户无教学价值） |
| [component.daemon] install_dir / keep_previous | :84-85 | install/releases + journal previous 推导 | 已交付（M7） |
| golden | :94 | 无 | 偏差登记（D37；挂 M10）；方法面入 D48（挂 M10） |
| source github_releases（tag/ref/staging 前缀） | :99-119 | LocalDir 单一 source | 偏差登记（D39；挂 M10） |
| [component.daemon.on_apply] units 重启表 | :126-157 | spawn/kill miniduckd 替代 systemctl | 偏差登记（D37；挂 M10） |
| [component.daemon.health] probe/timeout | :167-169 | gate.rs 500ms 轮询+预算 | 已交付（M7） |
| updater.toml 文件本身（+updater.example.toml 注释参考） | updater.toml:1-10 | env 变量（MINIDUCK_UPDATER_*）替代 | 未登记缺口（D52；挂 M10） |

## 5. 通知、载荷与协议机制面

| 功能项 | 原项目证据 | 手搓状态 | 处置 |
|---|---|---|---|
| RobotState 帧字段全集（t / move{requested,applied,limited_by} / head[4] / policy / safety{fallen,limp,gravity,gain} / loop{hz,missed} / joints / targets / velocities / currents_ma / odom / t_ns / imu{gyro,quat} / frames / skeleton / theremin / chorale） | duck-ipc-proto/src/lib.rs:3487-3572 | {tick, positions, imu{gyro,gravity,quat}, obs, action, fallen, enabled, gain, torque, skill}（impl/src/main.rs:547-562） | 未登记缺口（D50；挂 M10） |
| HealthResult 字段全集（healthy / degraded / reason 单值 / battery / motors / cpu_temp_c / cpu_throttle / control_loop / bus / imu） | lib.rs:3259-3318 | {healthy, reason[], tick, uptime_s, reads, writes, skipped_reads, consecutive_read_errors, achieved_hz}（main.rs:206-216） | 未登记缺口（D49；挂 M10） |
| SubscribeResult ack（accepted / walk / stand / unavailable / sitstand / ground_pick / skills） | lib.rs:2655-2683 | impl/src/lib.rs SubscribeResult；stand 恒省略（D33）；无 "no policy configured" | 已交付（M8 Bite 3，book/m08-convergence/acceptance.md）；stand 省略入 D33（挂 M9） |
| Progress 通知载荷（component/phase/percent/detail） | lib.rs:3001 | 无 | 偏差登记（D41；挂 M10） |
| 应用错误码表（BUSY=1、UNKNOWN_COMPONENT=2、PREFLIGHT_FAILED=4 … PERMISSION_DENIED=14） | lib.rs:843-880 | -32001..-32004（impl/src/updater/mod.rs） | 未登记缺口（D61；挂 M10，Phase 4 裁决扩为协议级错误码） |
| API_VERSION = 31 | lib.rs:360 | = 1（impl/src/lib.rs:27） | 偏差登记（D15；挂 M10） |
| 协议常量 JOINT_NAMES / POLICY_OBS_LEN=61 / POLICY_ACTION_LEN=14 | lib.rs:456/:369/:373 | impl/src/model.rs 关节常量；obs 61 维（M2） | 已交付（M1/M2，book/m02-obs-vector/acceptance.md） |
| Lane 连接模型（Prompt/Slow/Operation/Stream 四分通道） | lib.rs:1080-1089 | 单连接 select! 多路复用（main.rs:308-570） | 未登记缺口（D51；挂 M10） |
| Service 路由与五 socket 布局 | lib.rs:1049、:402-422 | 双 socket（/tmp/miniduckd.sock、/tmp/mini-updaterd.sock） | 未登记缺口（D42 进程面连带；挂 M10） |
| identity.json 发布（/run/<service>/identity.json） | lib.rs:424-434 | 无 | 未登记缺口（D62；挂 M10） |
| UPDATE_MAX_SILENCE_SECONDS = 600 契约 | lib.rs:391 | 无 | 未登记缺口（D61；挂 M10） |
| socket 默认路径 /run/*.sock | lib.rs:402-422 | /tmp/*.sock（main.rs:38、mini-duckctl.rs:11/:211） | 建议豁免（容器无 systemd RuntimeDirectory；0660 鉴权语义已对齐 D8） |
| HelloResult {daemon_version, revision} | lib.rs:2936-2943 | {service, api_version} | 未登记缺口（D62；挂 M10） |
| 教学附加帧字段（obs/action/enabled/torque/skill/tick，原版无） | —（手搓新增） | impl/src/main.rs:547-562 | 建议豁免（教程观测窗口；M8 终态裁决时保留或改名）；Phase 4 裁决永久保留（观测窗口） |

## 未登记缺口汇总（预留 D42–D63）

以下 22 条为本次清单新发现的偏差簿外缺口，D 号预留待登记入 spec/deviations.md：

- **D42** configd 缺席，net.*（4 条）与 system.* 前 5 条方法面连带缺席（btd/mediad/padd/tofd 四进程建议豁免单列）。
- **D43** ~~robot.drive {vx,vyaw} 请求式 vs 原版 robot.move {vx,vy,vyaw} 通知式：vy 侧向未开放，且连续意图（move/head/mouth）的通知语义（无应答、last-writer-wins）未实现~~ M8 Bite 1 已收敛 robot.move 本体（通知/请求双形态 + vy + robot.drive 删除）；M8 Bite 2 已收敛 notification 统一入口与 head/mouth 通知语义。仅剩残余：mini-duckctl move 仍请求式逐条调用（教学工具定位保留）。
- **D44** ~~robot.enable {on, toggle} vs 手搓 enable/disable 无参二分：缺 toggle（手柄 Start 语义），多出原版没有的 robot.disable~~ M8 Bite 2 已收敛：robot.enable {on, toggle} 全对齐（toggle daemon 侧翻转、永不拒绝、reason 逐字对齐），robot.disable 方法与 CLI 子命令一并删除（用户裁决）。另：disable 语义（斜坡+卸 torque vs 直接回 home 上电抱持）复核中新发现，登记 D64 并随 Bite 2 收敛（见 spec/deviations.md）。
- **D45** ~~robot.subscribe {hz} + SubscribeResult ack 缺席，robot.state 兼作订阅入口~~ M8 Bite 3 已收敛入口与名单（walk/sitstand/ground_pick 文件名、skills 配置名、accepted 恒 true）。带 id 的 robot.state 改为 METHOD_NOT_FOUND。残余：hz 收下但不降频，仍归 D25；stand 省略归 D33；无 "no policy configured" 文案（没有策略开关配置）。
- **D46** 命令 EMA 平滑缺席（[control] cmd_alpha/head_alpha；手搓意图直通，只有动作侧训练低通）。
- **D47** ~~updaterd 预检面 robot.safeToRestart / robot.modelApi 缺席（apply 前「现在重启安全吗」无从问起）~~ M8 Bite 4 已收敛：两个 RPC 上线（载荷、METHOD 名、`MODEL_API=2`、reason 文案逐字对齐原版），控制循环每拍记 `moving`/`fallen`，updater 侧 `impl/src/updater/robot.rs` 客户端四态枚举 + 预检插在 `apply()` 最前面。残余：D65（model_api None 拒绝，原版视为兼容）、D66（无 preflight 框架/其它检查项）、D67（Incompatible 不打日志）。
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

**2026-10-15 Phase 4 裁决结果**（逐条对应上七项，详见 spec/deviations.md 底部「Phase 4 附」）：1 → odometry 改判挂 M9（纯软件估计器，与自训策略同域；state 的 odom 字段仍归 D50）；2 → robot.remoteSessionActive 改判挂 M10，恒 false 补齐方法面；3 → account.* 三条并入 D60（M10），与 policy.* 同属 Hub 网络面；4 → pad.bindings / pad.bind 随 padd 整体挂 M10 再评估（用户有 Xbox 手柄，原版 padd 正是 Xbox 开发测试设备）；5 → robotctl monitor 极简版维持已交付，富仪表板面板挂 M10（依赖 D50+硬件）；6 → D43 残余③（mini-duckctl move 请求式）Phase 4 裁决永久保留（教学工具定位），通知语义主体已随 Bite 1/2 收敛；7 → 错误码表仍归 D61，Phase 4 裁决扩为协议级条目，挂 M10。

## M9a 交付记录（2026-10-25）

| 功能项 | 手搓状态 | 验收 |
|---|---|---|
| 自建 MJCF 身体与场景 | 已交付 Bite 1；D26 已收敛 | `accept/m09a-b01.sh` |
| BAM 参数对账与可选控制拍延迟 | 已交付 Bite 3；默认固定参数对齐官方 CPU 回放，训练范围只记账 | `accept/m09a-b03.sh` A/B |
| 自建模型 velstand 站立 + vx=0.8 行走 | 已交付 Bite 3；每侧鞋底 2 个 box、摩擦回到官方 1.0；3 次均 ≥2.0m；D70 已收敛 | `accept/m09a-b03.sh` C |
| 自建模型 sit_toggle 坐下与起身 | 已交付 Bite 3；2 次全循环通过；连续坐姿窗口未断言 | `accept/m09a-b03.sh` D |
| D4 sim 协议面 | 已交付 Bite 4；op=`gain`{kp}/`torque`{on}；PROTOCOL/15 槽/重力钉死；教学附加 push/body 保留 | `accept/m09a-b04.sh` |

## M9b 交付记录（已收官）

| 功能项 | 手搓状态 | 验收 |
|---|---|---|
| Velocity scene/obs/action 契约对账 | 已交付 Bite 1（已确认）；静态对账 + obs 布局单测 | `accept/m09b-b01.sh` |
| Reward / curriculum 解剖 | 已交付 Bite 2（已确认）；**课文降级**（用户判仍对参数） | `accept/m09b-b02.sh`（CI） |
| train / export / manifest 操作手册 | 已交付 Bite 3（已确认）；操作判断题，无代码领读 | `accept/m09b-b03.sh` |

## M9c 交付记录（已收官 · 2026-10-29）

| 功能项 | 手搓状态 | 验收 |
|---|---|---|
| FallPredictor + Limp/Posing（D23/D32） | 已交付 Bite 1（已确认）；默认 OFF，`MINIDUCK_LIMP_FALL=1` 显式开启 | `accept/m09c-b01.sh` |
| 相对 odometry（接触里程计） | 已交付 Bite 2（已确认）；局部 FK + ALPHA16；RPC 不加 odom（D50→M10） | `accept/m09c-b02.sh` |

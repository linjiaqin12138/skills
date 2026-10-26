# 0006 M9c Bite 1：预测跌倒与独立 Posing 恢复段

- 状态：起草
- 来源：手搓
- 出现于：m09c-b01
- 被取代于：

起草可以改。用户确认这一课之后，手搓决定改为「已采纳」。用户确认 Phase 1 之后，原项目决定改为「已记录」。此后不改正文。要推翻，新开一篇，本篇状态改为「被取代」，并填上新编号。

## 上下文

现有控制在 `safety.fallen()` 后直接从 Driving 进入 Limp，并在一个直立样本出现后重走 2 秒 RampUp。它把「已经倒了」当触发器，无法提前卸力，也把落地检测、回 home 和策略交接揉成一条路径。D23/D32 要求对齐上游：陀螺外推预测、busy/sitting 门控、Limp 落地检测、600ms Posing、增益 50→160→200。odometry 不在本 Bite。

上游默认关闭 limp_fall，因为默认发行没有独立 stand 网络；本实现虽然用 velstand 兼站兼走，但终态行为以上游配置默认值为准。完整配置文件面仍属 D52/M10，本轮只用现有 env 惯例提供 `MINIDUCK_LIMP_FALL=1` 显式开关，默认关闭。

## 备选

1. 新建独立 `fall.rs` 放预测器与落地检测；控制层保留 Driving，并新增独立 Posing 阶段。Limp 跟随实测位置、gain 50；Posing 从落地姿态线性到 home、gain 160；完成后清策略反馈并回 Driving。默认关闭，env 显式打开。
2. 继续让 Safety 直接触发 Limp，只把旧 Limp 的退出条件换成静止检测，并复用 RampUp 作为 Posing。
3. 完整照搬上游三态状态机和 Instant 驱动结构，同时在本 Bite 引入配置文件与全部 nine registry 参数。

## 决定

采用备选 1。预测器只消费新鲜且 IMU 已就绪的样本；busy、sitting、已 fallen、非 Driving、滑行或盲拍都会清掉预测去抖。Limp 由陀螺模长持续低于 1rad/s 达 200ms 或 1500ms 硬上限结束；盲拍不算静止。Posing 与启动 RampUp 不混用，恢复期间 homed 保持 true，moving 保持 true。disable 中断 Limp/Posing 时放弃反射并抱持实测姿态，不跳回 home。策略交接时清 last_action、低通锚点和各槽 recurrent 状态。

保留现有逐拍主循环与 Safety 唯一写句柄。`FrameSnapshot.skill` 用 `limp_fall` / `limp_pose` 暴露当前阶段，便于验收；Safety 的 fallen 仍只是报告，不反向门控写入。

## 放弃的成本

备选 2 改动更小，也能减少摔倒后的电机对抗；放弃它的代价是保留「已倒才反应」的延迟，并让启动恢复与跌倒恢复继续共享错误的时长、增益和 homed 语义。

备选 3 能让参数面一次到位；放弃它的代价是本轮仍需用 env 而不是 robotd.toml 改阈值，配置持久化与校验留到 D52/M10。

## 失效边界

若 M9e 引入独立 stand 槽或 M10 落地完整配置面，应重新评估 env 开关与交接选网；若控制频率不再是 50Hz，所有以 nominal period 表达的去抖和预测参数必须随配置周期一起核对。若 velstand 零命令无法接住 Posing 后的身体，不能把延长 Posing 当成替代策略能力的办法，应在 M9e 处理策略交接。

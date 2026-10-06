//! 50 Hz 控制任务：read → observe → gate → 调度（M6）→ 策略 → apply，并把
//! 最新快照（Sensors + 观测 + 本拍动作 + 安全状态 + 活跃技能名）通过 watch
//! 频道发布给 RPC 层。
//!
//! M5 起所有写电机的路径都收进 Safety（唯一写句柄，见 safety.rs 模块
//! 文档），本循环只通过 Safety 的方法碰总线。状态机是原版
//! Bringup/LimpFall 三态机的简化版（简化已登记偏差簿 D23）：
//!
//!   Held ──enable {on:true}──▶ RampUp(100拍) ──▶ Driving ◀──▶ Limp（跌倒⇄恢复）
//!  （启动抱持，torque off）          │              │
//!                                    ▼              ▼ enable {on:false}
//!                                  Stopped ◀────────┘ 当拍直接命令回 home（无斜坡），
//!                               （上电抱持 home）       policy reset，torque 保持 on
//!                                    │
//!                                    └──enable {on:true}──▶ 直接回 Driving（无斜坡）
//!
//! 斜坡完成后按 enabled 现值落 Driving 或 Stopped，所以 RampUp/Limp 途中的
//! enable 边沿不会丢（边沿只动 Held/Stopped/Driving，其余阶段自然汇入）。
//! 卸 torque 是 robot.relax 的活（enable 管策略、init/relax 管电源，两对
//! 开关——reference/robotd/src/main.rs:4496-4499）；本循环没有任何
//! set_torque(false) 路径，Held 是进程启动态，也是未来 relax 的落点。
//!
//! Held 绝不调 set_torque：进程启动不是移动机器人的理由——舵机 RAM 里的
//! torque 跨进程存活，被重启的 daemon 必须让站着的机器人继续站着（D9 收敛）。
//!
//! M6：Driving 阶段的策略选择与 command 重编码由 scheduler 决定（技能 >
//! ground_pick > sit > walk）；脚本动作在飞（busy）时抑制 Driving→Limp
//! 转移（原版门 FallPredictor，简化见 D32）；嘴在策略写完后由 robot.mouth
//! 意图覆写（嘴不属于任何策略）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::watch;
use tokio::time::{Instant, MissedTickBehavior};

use crate::io::{RobotIo, Sensors};
use crate::model::{DEFAULT_POSITION, MOUTH_INDEX, NUM_JOINTS, mouth_target};
use crate::obs::{ACTION_LEN, Command, OBS_LEN, Observation};
use crate::policy::PolicyError;
use crate::safety::{Safety, SafetyConfig};
use crate::scheduler::Scheduler;

pub const TICK_PERIOD: Duration = Duration::from_millis(20);

/// 插值长度：2 秒 × 50 Hz = 100 拍。
pub const RAMP_TICKS: u64 = 100;

/// 头关节一阶低通；训练用 0.5，必须一致。覆盖 neck/head pitch/yaw/roll（不含嘴）。
pub const HEAD_LOWPASS: f64 = 0.5;

/// 十条腿一阶低通；训练用 0.7。
pub const LEGS_LOWPASS: f64 = 0.7;

const HEAD_JOINTS: std::ops::Range<usize> = 5..9;

/// 控制循环对外暴露的计数。health 端点直接读它——健康数据必须由
/// 循环自己记账，而不是 RPC 层猜（"healthy" 的语义是"循环在跑"，
/// 不是"socket 活着"）。全部是原子量：IPC 侧只读，永不阻塞控制循环。
pub struct Stats {
    pub tick: AtomicU64,
    pub reads: AtomicU64,
    pub writes: AtomicU64,
    pub skipped_reads: AtomicU64,
    /// 连续读失败次数，成功即清零（health 阈值判定用）。
    pub consecutive_read_errors: AtomicU64,
    /// 最近一拍距 started 的毫秒数。health 用它算"循环多久没动了"。
    pub last_tick_millis: AtomicU64,
    /// 最近一个 5s 窗口的实测频率 ×1000；0 = 首个窗口还没满（暖机中）。
    pub achieved_millihz: AtomicU64,
    /// 策略是否加载成功（D19：加载失败进程也活着，抱持姿态报病）。
    pub policy_ok: AtomicBool,
    /// 控制循环当前是否在 Driving 阶段。robot.do 据此拒绝"斜坡途中"的
    /// 技能请求（对齐原版 main.rs 的 homed 检查：accepted 然后什么都不
    /// 发生是最坏的回答）。
    pub driving: AtomicBool,
    /// 循环 epoch，last_tick_millis 的基准。
    pub started: std::time::Instant,
}

impl Stats {
    fn new(policy_ok: bool) -> Self {
        Stats {
            tick: AtomicU64::new(0),
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            skipped_reads: AtomicU64::new(0),
            consecutive_read_errors: AtomicU64::new(0),
            last_tick_millis: AtomicU64::new(0),
            achieved_millihz: AtomicU64::new(0),
            policy_ok: AtomicBool::new(policy_ok),
            driving: AtomicBool::new(false),
            started: std::time::Instant::now(),
        }
    }
}

/// 控制循环与 RPC 层共享的控制状态（M5 从 SharedCommand 升级）。
/// Mutex 而非 watch：命令不是帧流，"最近一条为准"且不许丢——watch 的
/// borrow 语义对调用方多绕一层，这里几十字节拷出来最直白。
#[derive(Default)]
pub struct ControlState {
    pub command: Command,
    /// 最近一次 robot.move 的时刻；deadman 据此算意图年龄。
    /// None = 从没被驾驶过。
    pub last_intent_at: Option<std::time::Instant>,
    /// robot.enable {on, toggle} 写入（toggle 已在 RPC 层翻成最终值）；
    /// 边沿检测在控制循环里做。
    pub enabled: bool,
    /// M6：技能请求边沿位掩码（位分配见 scheduler.rs）。控制循环每拍
    /// 取一次清零——位掩码而非队列：同拍两个不同请求都该被看到，优先级
    /// 链决定谁先（reference/robotd/src/intents.rs:105-117）。
    pub skill_edges: u32,
    /// M6：robot.mouth 意图，0..1 开度。纯 level 无时间戳：客户端死掉
    /// 嘴停在那（原版刻意行为，intents.rs:144-146）。
    pub mouth: f64,
}

pub type SharedControl = Arc<std::sync::Mutex<ControlState>>;

pub fn shared_control() -> SharedControl {
    Arc::new(std::sync::Mutex::new(ControlState::default()))
}

/// 插值第 `tick_in_ramp` 拍的目标位置。线性，起点为 `start`，终点为
/// DEFAULT_POSITION，超过 RAMP_TICKS 后钉在 home（保持）。
pub fn ramp_target(start: &[f64; NUM_JOINTS], tick_in_ramp: u64) -> [f64; NUM_JOINTS] {
    let t = (tick_in_ramp.min(RAMP_TICKS) as f64) / RAMP_TICKS as f64;
    let mut out = [0.0; NUM_JOINTS];
    for i in 0..NUM_JOINTS {
        out[i] = start[i] + (DEFAULT_POSITION[i] - start[i]) * t;
    }
    out
}

/// `targets = home + scale * scatter(action)`，再按头/腿做一阶低通。
/// `previous` 为 None 时（策略第一拍）不做低通。嘴始终等于 home（嘴的
/// 覆写是控制循环的事，见 driving_tick）。M6 起 scale 由调度器按槽位给
/// （action_scale 是每份权重的训练属性，不是全局常量）。
pub fn apply_action(
    action: &[f32; ACTION_LEN],
    previous: Option<&[f64; NUM_JOINTS]>,
    scale: f64,
) -> [f64; NUM_JOINTS] {
    let offsets = Observation::scatter_action(action);
    let mut targets = [0.0; NUM_JOINTS];
    for j in 0..NUM_JOINTS {
        targets[j] = DEFAULT_POSITION[j] + scale * offsets[j];
    }

    if let Some(previous) = previous {
        for joint in HEAD_JOINTS {
            targets[joint] =
                HEAD_LOWPASS * targets[joint] + (1.0 - HEAD_LOWPASS) * previous[joint];
        }
        for joint in 0..NUM_JOINTS {
            if HEAD_JOINTS.contains(&joint) || joint == MOUTH_INDEX {
                continue;
            }
            targets[joint] =
                LEGS_LOWPASS * targets[joint] + (1.0 - LEGS_LOWPASS) * previous[joint];
        }
    }

    // scatter 已把嘴写成 0，故 targets[mouth] == home；低通也跳过嘴。
    debug_assert!((targets[MOUTH_INDEX] - DEFAULT_POSITION[MOUTH_INDEX]).abs() < 1e-12);
    targets
}

/// 控制循环每拍成功 read 后发布的快照。读失败不发：没有新样本，
/// watch 里留着上一帧成功的快照，而不是造一帧空数据。
#[derive(Debug, Clone)]
pub struct FrameSnapshot {
    pub sensors: Sensors,
    pub obs: [f32; OBS_LEN],
    /// 本拍刚写出的策略动作；斜坡/抱持/Limp 期间为 0。
    pub action: [f32; ACTION_LEN],
    /// M5 起随帧发布的安全/使能状态，robot.state 推送直接透传。
    pub fallen: bool,
    pub enabled: bool,
    /// Safety 上次写入的增益（None = 还没写过）。
    pub gain: Option<u16>,
    /// 本进程最近一次下达的 torque 状态（舵机 RAM 真值不可读，报的是
    /// "我们命令过什么"；启动时一律 false = 本进程没命令过）。
    pub torque: bool,
    /// M6：当前活跃技能名（"walk"/"sit"/"rise"/"ground_pick"/技能名）；
    /// None = 不在 Driving（斜坡/抱持/Limp）或本拍推理失败。
    pub skill: Option<&'static str>,
}

impl Default for FrameSnapshot {
    fn default() -> Self {
        Self {
            sensors: Sensors::default(),
            obs: [0.0; OBS_LEN],
            action: [0.0; ACTION_LEN],
            fallen: false,
            enabled: false,
            gain: None,
            torque: false,
            skill: None,
        }
    }
}

/// Driving 阶段一拍的成功产物。收进结构体是因为调用方（控制循环）和
/// 测试（连续性断言）走同一条路径，返回值多了才不会在两处各自漂移。
pub struct DrivingOutcome {
    pub targets: [f64; NUM_JOINTS],
    pub action: [f32; ACTION_LEN],
    /// 本拍实际喂给策略的观测（command 块是调度器重编码后的）。
    pub obs: Observation,
    pub label: &'static str,
}

/// Driving 阶段的一拍：技能边沿 → 调度（选网+重编码 command）→ 组观测 →
/// 推理 → 缩放/低通 → 嘴覆写 → 推进窗口计时。
///
/// 从循环里抽成自由函数的理由：切换连续性断言（walk→sit→rise→
/// ground_pick→kick 全程逐拍 |Δtarget|）要驱动的就是这条路径本身，
/// 而不是它的一个改写版。
///
/// `command` 必须已过 deadman 门。`mouth` 是 0..1 开度意图：嘴不属于任何
/// 策略，策略写完 14 关节后单独覆写（reference/robotd/src/main.rs:3188-3194
/// ——嘴只有意图能动它，且只在 Driving 覆写，斜坡/抱持阶段嘴跟随该阶段
/// 自己的目标，"a restart cannot snap a mouth"）。
pub fn driving_tick(
    scheduler: &mut Scheduler,
    sensors: &Sensors,
    command: &Command,
    edges: u32,
    mouth: f64,
    previous: Option<&[f64; NUM_JOINTS]>,
    last_action: &[f32; ACTION_LEN],
) -> Result<DrivingOutcome, PolicyError> {
    scheduler.handle_requests(edges);
    let decision = scheduler.step(command);
    let obs = Observation::build(
        &sensors.imu,
        &sensors.positions,
        &sensors.velocities,
        &DEFAULT_POSITION,
        last_action,
        &decision.command,
    );
    // 换网即 reset 新网络的 LSTM 状态（scheduler.infer 内做）；last_action
    // 与低通锚点跨切换保留（reference/robotd/src/control.rs:211-214）。
    let action = scheduler.infer(&obs, &decision)?;
    let mut targets = apply_action(&action, previous, decision.action_scale);
    targets[MOUTH_INDEX] = mouth_target(mouth);
    // 窗口计时在本拍用完之后推进（原版同序：电机写完后推进相位）。
    scheduler.advance();
    Ok(DrivingOutcome {
        targets,
        action,
        obs,
        label: decision.label,
    })
}

/// 主循环阶段。原版是 Bringup/LimpFall 三态机加 FallPredictor，
/// 这里是偏差簿登记的简化版（D23）：fallen 判定直接触发软倒，
/// 斜坡是逐拍推进的阶段而不是阻塞调用（D10，M8 裁决）。
enum Phase {
    /// 抱持：写启动时读到的姿态，绝不动 torque。hold 为 None 表示
    /// 还没读到过一帧（第一次 read 成功时锁存启动姿态）。
    /// 进程启动态，也是未来 robot.relax 的落点。
    Held { hold: Option<[f64; NUM_JOINTS]> },
    /// 从实测姿态线性斜坡到 home，完成后按 enabled 现值进 Driving 或 Stopped。
    RampUp { start: [f64; NUM_JOINTS], tick: u64 },
    /// 策略闭环。
    Driving,
    /// 跌倒软倒：目标跟随实测位置（倒地过程中固定目标会累积误差=
    /// 电机顶着地板较劲；"软"的关键就是目标跟着身体走），低增益。
    Limp,
    /// 上电抱持 home（原版 Ready + hold=DEFAULT_POSITION）：disable 后舵机
    /// 自己走回 home 并保持上电，再 enable 直接回 Driving。嘴跟随 home
    /// （非 Driving 阶段嘴跟随本阶段目标，D31）；head/body 命令无效。
    Stopped,
}

/// enabled 边沿上要做的动作（纯数据，转移规则单测覆盖）。
/// 规则依据（reference/robotd/src/main.rs）：
/// - enable 的 bring-up（torque on + 斜坡回 home）只从 torque off 的 Held
///   触发（:2560-2566）；没策略的机器人 enable 无意义，留在 Held（D19）。
/// - Stopped（上电抱持 home）再 enable：直接回 Driving，无斜坡无死窗
///   （:2124 driving = enabled && bringup == Ready）。
/// - Driving 中 disable：当拍直接命令回 home——舵机按自己的速度走过去，
///   无斜坡、不卸 torque（:2786-2798，注释原文 "Commanded directly, no
///   ramp: the servos do the travel at their own speed"）；同边沿 policy
///   reset，disable 结束 recurrent episode（:2791-2793）。
/// - RampUp/Limp 中的边沿不立即动作：斜坡终点就是 home，完成时按 enabled
///   现值落 Driving/Stopped（`ramp_done_phase`）；Limp 的跌倒响应走完，
///   恢复进 RampUp 后自然汇入同一条规则——边沿因此不会丢。
enum EdgeAction {
    /// Held + enable：torque on，然后从实测姿态斜坡回 home。
    PowerOnAndRamp,
    /// Stopped + enable：直接回 Driving。
    ResumeDriving,
    /// Driving + disable：policy reset，相位切 Stopped（当拍写 home）。
    StopToHome,
}

/// enabled 边沿 → 相位动作。非边沿（level 不变）或"边沿由斜坡完成规则
/// 代为处理"的阶段返回 None。
fn edge_transition(
    phase: &Phase,
    enabled: bool,
    was_enabled: bool,
    has_policy: bool,
) -> Option<EdgeAction> {
    match (enabled, was_enabled, phase) {
        (true, false, Phase::Held { .. }) if has_policy => Some(EdgeAction::PowerOnAndRamp),
        (true, false, Phase::Stopped) => Some(EdgeAction::ResumeDriving),
        (false, true, Phase::Driving) => Some(EdgeAction::StopToHome),
        _ => None,
    }
}

/// 斜坡完成的落点：enable 着进 Driving，否则停在 home 上电抱持。
fn ramp_done_phase(enabled: bool) -> Phase {
    if enabled {
        Phase::Driving
    } else {
        Phase::Stopped
    }
}

/// 启动控制任务，返回计数器和最新快照的接收端。
/// `control` 由 RPC 层写（robot.move/enable/do/mouth/head）、循环每拍读。
/// `scheduler` 为 None 时（D19：walk 策略加载失败不退出）永远停在 Held 抱持。
pub fn spawn(
    mut safety: Safety<Box<dyn RobotIo>>,
    mut scheduler: Option<Scheduler>,
    control: SharedControl,
) -> (Arc<Stats>, watch::Receiver<FrameSnapshot>) {
    let config = SafetyConfig::default();
    let stats = Arc::new(Stats::new(scheduler.is_some()));
    let (frame_tx, frame_rx) = watch::channel(FrameSnapshot::default());

    {
        let stats = stats.clone();
        tokio::spawn(async move {
            let epoch = stats.started;
            // interval_at 而非 interval：interval 的第一拍立即触发，
            // 第 0 帧会在初始姿态还没读到时就跑控制逻辑、把垃圾数据
            // 写上总线。推迟一个周期，让每一帧都走同一条
            // read → compute → write 路径。（已知坑，勿改回 interval。）
            let mut timer = tokio::time::interval_at(Instant::now() + TICK_PERIOD, TICK_PERIOD);
            // 原版用 Skip：积压时丢拍，避免 Burst 连发电机命令；Delay 实测掉到 43.1Hz。
            timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

            let mut phase = Phase::Held { hold: None };
            let mut was_enabled = false;
            let mut torque_on = false;
            // 策略输出低通锚点；斜坡/Limp 阶段不算，策略第一拍为 None。
            let mut previous_targets: Option<[f64; NUM_JOINTS]> = None;
            // 斜坡/抱持/Limp 期间保持 0；策略成功拍才更新，失败不改。
            let mut last_action = [0.0f32; ACTION_LEN];
            let mut tick: u64 = 0;

            // 速率/抖动统计：每 5 秒一行，给 grep 用；同时记进 Stats 供 health。
            let mut window_start = Instant::now();
            let mut window_ticks: u64 = 0;
            let mut last_tick_at: Option<Instant> = None;
            let mut deltas_ms: Vec<f64> = Vec::with_capacity(256);

            loop {
                let now = timer.tick().await;
                if let Some(prev) = last_tick_at {
                    let delta_ms = now.duration_since(prev).as_secs_f64() * 1000.0;
                    deltas_ms.push((delta_ms - TICK_PERIOD.as_secs_f64() * 1000.0).abs());
                }
                last_tick_at = Some(now);
                window_ticks += 1;
                let window_secs = window_start.elapsed().as_secs_f64();
                if window_secs >= 5.0 {
                    let rate = window_ticks as f64 / window_secs;
                    let p99 = p99(&mut deltas_ms);
                    eprintln!("tick_rate={rate:.1}Hz p99_jitter_ms={p99:.2}");
                    stats
                        .achieved_millihz
                        .store((rate * 1000.0) as u64, Ordering::Relaxed);
                    deltas_ms.clear();
                    window_ticks = 0;
                    window_start = Instant::now();
                }

                let sensors = match safety.read() {
                    Ok(s) => s,
                    Err(_) => {
                        // read 失败跳过本拍（D24：coast 滑行是禁止提前实现项）。
                        stats.skipped_reads.fetch_add(1, Ordering::Relaxed);
                        stats.consecutive_read_errors.fetch_add(1, Ordering::Relaxed);
                        tick += 1;
                        stats.tick.store(tick, Ordering::Relaxed);
                        stats.last_tick_millis.store(
                            epoch.elapsed().as_millis() as u64,
                            Ordering::Relaxed,
                        );
                        continue;
                    }
                };
                stats.reads.fetch_add(1, Ordering::Relaxed);
                stats.consecutive_read_errors.store(0, Ordering::Relaxed);

                // 每拍顺序对齐原版文档化顺序：read → observe → gate → 策略 → apply。
                safety.observe(&sensors, TICK_PERIOD);

                let (command, intent_age, enabled, edges, mouth) = {
                    let mut ctl = control.lock().expect("control mutex poisoned");
                    (
                        ctl.command,
                        ctl.last_intent_at
                            .map(|t| t.elapsed())
                            .unwrap_or(Duration::MAX),
                        ctl.enabled,
                        // 技能边沿每拍取一次清零。非 Driving 拍取到即丢弃
                        // （原版同：policy 不在驱动时请求落不了地，只留日志）——
                        // robot.do 在 RPC 侧已按 enabled+driving 拒绝，这里是兜底。
                        std::mem::take(&mut ctl.skill_edges),
                        ctl.mouth,
                    )
                };
                // gate 每拍都调（armed 语义防日志噪音）；Driving 用 gated 命令组 obs。
                let (gated, _limit) = safety.gate(command, intent_age);

                // enabled 边沿检测（robot.enable 写入的开关在这里变成相位
                // 转移）。只在边沿动作，绝不在 Held 里每拍碰 torque。边沿立即
                // 生效（先于跌倒判定），否则 "Driving 中跌倒"与"同拍 enable
                // off"会互相覆盖。转移规则抽在 edge_transition 里（单测覆盖）。
                match edge_transition(&phase, enabled, was_enabled, scheduler.is_some()) {
                    Some(EdgeAction::PowerOnAndRamp) => match safety.set_torque(true) {
                        Ok(()) => {
                            torque_on = true;
                            phase = Phase::RampUp {
                                start: sensors.positions,
                                tick: 0,
                            };
                        }
                        Err(e) => eprintln!("set_torque(true) failed: {e}"),
                    },
                    Some(EdgeAction::ResumeDriving) => {
                        phase = Phase::Driving;
                    }
                    Some(EdgeAction::StopToHome) => {
                        // disable 结束 recurrent episode：策略 LSTM 清零，
                        // 低通锚点与 last_action 一并丢弃（Stopped→Driving
                        // 无斜坡，带着旧锚点回来就是一次踉跄）。
                        if let Some(s) = scheduler.as_mut() {
                            s.reset();
                        }
                        previous_targets = None;
                        last_action = [0.0; ACTION_LEN];
                        phase = Phase::Stopped;
                    }
                    None => {}
                }
                was_enabled = enabled;

                // 跌倒转移（只在 Driving/Limp 间；斜坡/抱持期间的跌倒
                // 不改变行为——斜坡回 home 本身就是对的恢复动作）。
                // M6：脚本动作在飞（busy：技能/ground_pick/起身中）时抑制
                // Driving→Limp——原版用 busy 门 FallPredictor（reference/robotd/
                // src/control.rs:318-322），我们门的是简化版直接转移（D32）。
                // fallen 报告不受影响，state 推送照真。
                let busy = scheduler.as_ref().is_some_and(|s| s.busy());
                match phase {
                    Phase::Driving if safety.fallen() && !busy => {
                        phase = Phase::Limp;
                    }
                    Phase::Limp if !safety.fallen() => {
                        // 起来了：从当前姿态斜坡回 home，策略锚点清零重来。
                        previous_targets = None;
                        last_action = [0.0; ACTION_LEN];
                        phase = Phase::RampUp {
                            start: sensors.positions,
                            tick: 0,
                        };
                    }
                    _ => {}
                }

                // 观测始终用当前 last_action 组装；Driving 成功推理后才更新
                // last_action。Driving 阶段会在 driving_tick 里用调度器重编码
                // 后的 command 重建观测；这里的版本供其他阶段的快照用。
                let observation = Observation::build(
                    &sensors.imu,
                    &sensors.positions,
                    &sensors.velocities,
                    &DEFAULT_POSITION,
                    &last_action,
                    &gated,
                );

                // 非 Driving 拍取到的技能边沿：丢弃并留日志（RPC 侧已按
                // enabled+driving 拒绝，这是兜底，对齐原版"policy 不在驱动
                // 时请求落不了地"）。
                if edges != 0 && !matches!(phase, Phase::Driving) {
                    eprintln!("scheduler: skill edges {edges:#010b} dropped: not driving");
                }

                let mut action_out = [0.0f32; ACTION_LEN];
                let mut obs_for_snapshot = observation;
                let mut skill_label: Option<&'static str> = None;
                let hold = sensors.positions;
                let mut wrote = false;
                // 斜坡完成的转移延后到 match 之后（match 借用着 phase）。
                let mut next_phase: Option<Phase> = None;
                match &mut phase {
                    Phase::Held { hold: latched } => {
                        let target = *latched.get_or_insert(sensors.positions);
                        wrote = safety
                            .apply(target, target, config.gain_running)
                            .is_ok();
                    }
                    Phase::RampUp { start, tick: t } => {
                        let target = ramp_target(start, *t);
                        wrote = safety.apply(target, hold, config.gain_running).is_ok();
                        *t += 1;
                        if *t > RAMP_TICKS {
                            // 注意 t==RAMP_TICKS 那一拍已把 home 原样写出，
                            // 再转移，避免末端少一拍造成"差一步没到"。
                            // 落点看 enabled 现值：斜坡途中的 enable 边沿
                            // 在这里汇入，不会丢（edge_transition 注释）。
                            next_phase = Some(ramp_done_phase(enabled));
                        }
                    }
                    Phase::Stopped => {
                        // 上电抱持 home：disable 不当拍斜坡，直接命令 home，
                        // 舵机按自己的速度走过去（原版 "Commanded directly,
                        // no ramp"）；torque 保持 on，gain 维持 running。
                        wrote = safety
                            .apply(DEFAULT_POSITION, hold, config.gain_running)
                            .is_ok();
                    }
                    Phase::Driving => {
                        match scheduler.as_mut() {
                            Some(s) => match driving_tick(
                                s,
                                &sensors,
                                &gated,
                                edges,
                                mouth,
                                previous_targets.as_ref(),
                                &last_action,
                            ) {
                                Ok(outcome) => {
                                    previous_targets = Some(outcome.targets);
                                    last_action = outcome.action;
                                    action_out = outcome.action;
                                    obs_for_snapshot = outcome.obs;
                                    skill_label = Some(outcome.label);
                                    wrote = safety
                                        .apply(outcome.targets, hold, config.gain_running)
                                        .is_ok();
                                }
                                Err(e) => {
                                    // 本拍不写、不更新 last_action；循环继续。
                                    eprintln!("policy infer failed: {e}");
                                }
                            },
                            // 进不了这里：scheduler 为 None 时永远停在 Held。
                            None => {
                                wrote = safety.apply(hold, hold, config.gain_running).is_ok();
                            }
                        }
                    }
                    Phase::Limp => {
                        // 软倒：策略不 step，目标跟随实测位置，低增益卸力。
                        wrote = safety.apply(hold, hold, config.gain_limp).is_ok();
                    }
                }
                if wrote {
                    stats.writes.fetch_add(1, Ordering::Relaxed);
                }
                if let Some(p) = next_phase {
                    phase = p;
                }
                stats
                    .driving
                    .store(matches!(phase, Phase::Driving), Ordering::Relaxed);

                let mut obs_arr = [0.0f32; OBS_LEN];
                obs_arr.copy_from_slice(obs_for_snapshot.as_slice());
                let _ = frame_tx.send(FrameSnapshot {
                    sensors: sensors.clone(),
                    obs: obs_arr,
                    action: action_out,
                    fallen: safety.fallen(),
                    enabled,
                    gain: safety.gain(),
                    torque: torque_on,
                    skill: skill_label,
                });

                tick += 1;
                stats.tick.store(tick, Ordering::Relaxed);
                stats
                    .last_tick_millis
                    .store(epoch.elapsed().as_millis() as u64, Ordering::Relaxed);
            }
        });
    }

    (stats, frame_rx)
}

/// 排序取 P99。deltas 会被原地排序（调用方统计完即丢弃，无副作用问题）。
fn p99(deltas_ms: &mut [f64]) -> f64 {
    if deltas_ms.is_empty() {
        return 0.0;
    }
    deltas_ms.sort_by(f64::total_cmp);
    let idx = ((deltas_ms.len() as f64) * 0.99).ceil() as usize;
    deltas_ms[idx.clamp(1, deltas_ms.len()) - 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::{SKILLS, WALK_ACTION_SCALE, request_bit};

    #[test]
    fn ramp_endpoints() {
        let start = [0.0; NUM_JOINTS];
        assert_eq!(ramp_target(&start, 0), start);
        assert_eq!(ramp_target(&start, RAMP_TICKS), DEFAULT_POSITION);
        assert_eq!(ramp_target(&start, RAMP_TICKS * 3), DEFAULT_POSITION);
    }

    #[test]
    fn ramp_midpoint_at_one_second() {
        let start = [0.0; NUM_JOINTS];
        let mid = ramp_target(&start, 50);
        for i in 0..NUM_JOINTS {
            let expected = (start[i] + DEFAULT_POSITION[i]) / 2.0;
            assert!(
                (mid[i] - expected).abs() < 1e-6,
                "joint {i}: {mid:?} vs expected midpoint {expected}"
            );
        }
    }

    #[test]
    fn apply_action_no_previous_is_home_plus_scaled_scatter() {
        let mut action = [0.0f32; ACTION_LEN];
        action[0] = 1.0; // left_hip_yaw
        action[8] = -0.5; // head_roll（策略槽 8 → 关节 8）
        action[9] = 2.0; // right_hip_yaw（槽 9 → 关节 10）

        let targets = apply_action(&action, None, WALK_ACTION_SCALE);
        let scattered = Observation::scatter_action(&action);

        for j in 0..NUM_JOINTS {
            let expected = DEFAULT_POSITION[j] + WALK_ACTION_SCALE * scattered[j];
            assert!(
                (targets[j] - expected).abs() < 1e-9,
                "joint {j}: got {} expected {expected}",
                targets[j]
            );
        }
        assert_eq!(targets[MOUTH_INDEX], DEFAULT_POSITION[MOUTH_INDEX]);
        assert_eq!(scattered[MOUTH_INDEX], 0.0);
    }

    #[test]
    fn apply_action_lowpass_head_legs_mouth_stays_home() {
        let action = [1.0f32; ACTION_LEN];
        let raw = apply_action(&action, None, WALK_ACTION_SCALE);
        let previous = [0.0f64; NUM_JOINTS]; // 人为锚点，方便验算系数
        let filtered = apply_action(&action, Some(&previous), WALK_ACTION_SCALE);

        for joint in HEAD_JOINTS {
            let expected = HEAD_LOWPASS * raw[joint];
            assert!(
                (filtered[joint] - expected).abs() < 1e-9,
                "head joint {joint}: {} vs {expected}",
                filtered[joint]
            );
        }
        for joint in 0..NUM_JOINTS {
            if HEAD_JOINTS.contains(&joint) || joint == MOUTH_INDEX {
                continue;
            }
            let expected = LEGS_LOWPASS * raw[joint];
            assert!(
                (filtered[joint] - expected).abs() < 1e-9,
                "leg joint {joint}: {} vs {expected}",
                filtered[joint]
            );
        }
        assert_eq!(filtered[MOUTH_INDEX], DEFAULT_POSITION[MOUTH_INDEX]);
    }

    /// 技能边沿"取一次清零"：ControlState 里是 mem::take 语义，
    /// 第二拍看到的是空掩码（edge 不是 level）。
    #[test]
    fn skill_edges_are_taken_once() {
        let ctl = shared_control();
        {
            let mut c = ctl.lock().unwrap();
            c.skill_edges |= crate::scheduler::EDGE_SIT_TOGGLE;
        }
        let first = std::mem::take(&mut ctl.lock().unwrap().skill_edges);
        let second = std::mem::take(&mut ctl.lock().unwrap().skill_edges);
        assert_eq!(first, crate::scheduler::EDGE_SIT_TOGGLE);
        assert_eq!(second, 0, "边沿取过一次就没了");
    }

    fn held() -> Phase {
        Phase::Held { hold: None }
    }

    fn ramp_up() -> Phase {
        Phase::RampUp {
            start: [0.0; NUM_JOINTS],
            tick: 0,
        }
    }

    #[test]
    fn driving_goes_straight_to_stopped_on_disable() {
        // 无斜坡、不卸 torque：StopToHome 的全部动作是 policy reset +
        // 切 Stopped（Stopped 臂当拍写 DEFAULT_POSITION）。
        assert!(matches!(
            edge_transition(&Phase::Driving, false, true, true),
            Some(EdgeAction::StopToHome)
        ));
    }

    #[test]
    fn stopped_resumes_driving_immediately_on_enable() {
        // 机器人已在 home 且上电：无斜坡无死窗（原版 driving = enabled && Ready）。
        assert!(matches!(
            edge_transition(&Phase::Stopped, true, false, true),
            Some(EdgeAction::ResumeDriving)
        ));
    }

    #[test]
    fn held_ramps_up_on_enable_only_with_policy() {
        assert!(matches!(
            edge_transition(&held(), true, false, true),
            Some(EdgeAction::PowerOnAndRamp)
        ));
        // 没策略的机器人 enable 无意义：留在 Held（D19），torque 保持 off。
        assert!(edge_transition(&held(), true, false, false).is_none());
    }

    #[test]
    fn edges_during_rampup_and_limp_are_deferred_not_dropped() {
        // RampUp/Limp 中的边沿返回 None——不是丢弃，是交给斜坡完成规则
        // （ramp_done_phase 按 enabled 现值落点）：斜坡终点就是 home，
        // Limp 恢复也经 RampUp 汇入同一条规则。
        assert!(edge_transition(&ramp_up(), false, true, true).is_none());
        assert!(edge_transition(&ramp_up(), true, false, true).is_none());
        assert!(edge_transition(&Phase::Limp, false, true, true).is_none());
        assert!(edge_transition(&Phase::Limp, true, false, true).is_none());
    }

    #[test]
    fn level_changes_nowhere_else_do_nothing() {
        // 非边沿（level 不变）与其他阶段的边沿都不动相位。
        assert!(edge_transition(&Phase::Driving, true, true, true).is_none());
        assert!(edge_transition(&Phase::Stopped, false, false, true).is_none());
        assert!(edge_transition(&Phase::Stopped, false, true, true).is_none());
        assert!(edge_transition(&held(), false, true, true).is_none());
    }

    #[test]
    fn ramp_completion_lands_by_current_enabled() {
        assert!(matches!(ramp_done_phase(true), Phase::Driving));
        assert!(matches!(ramp_done_phase(false), Phase::Stopped));
    }

    /// M6 切换连续性端到端断言（FakeIo 语义：sensors.positions = 上拍写出的
    /// targets，闭环直接跑 driving_tick——和生产循环同一条代码路径）。
    ///
    /// 场景：walk（带 twist 意图）→ sit → 坐姿 100 拍 → rise → walk 100 拍
    /// → ground_pick 全程 → kick_left 窗口 → 回 walk。全程逐拍记录写出目标，
    /// 两层断言：
    ///
    /// 1. **机制断言（切换拍）**：换网络的那一拍，写出目标必须精确等于
    ///    `apply_action(新网动作, Some(上拍锚点), 新槽 scale)`——即低通锚点
    ///    跨切换逐位保留（若切换把锚点清零，这一拍会写未滤波的 raw，公式
    ///    立刻不成立）。这是"切换无 blending 也不跳"的真正机制：
    ///    last_action 跨网共享回喂 + 锚点保留 + 换网即 reset LSTM
    ///    （reference/robotd/src/control.rs:211-214）。
    ///
    /// 2. **回归阈值（全程）**：任意相邻两拍、任一策略关节 |Δtarget| ≤ 5.0 rad。
    ///    实测依据：本场景（FakeIo 完美跟踪 + 速度恒 0，全程在策略训练分布
    ///    之外，动作大幅饱和）峰值 4.20 rad（起身中途 joint 1，非切换拍），
    ///    切换拍峰值 1.61 rad（ground_pick→walk，joint 4）；5.0 ≈ 1.2× 全程
    ///    峰值。低通 α=0.7 的推导上界 0.7×|Δraw| 在 OOD 下很松（raw 单拍可摆
    ///    6 rad），所以阈值按实测峰值留 20% 余量——它拦的是"切换机制坏了
    ///    导致的全局跳变"这类回归，不是证明数学上不可能跳。在分布内（sim）
    ///    该值小一到两个数量级。
    ///
    /// 嘴除外：嘴有意图直通覆写、无低通（原版同，reference/robotd/src/
    /// control.rs:618-620 低通跳过嘴，main.rs:3188-3194 意图直接覆写），
    /// 进 Driving 第一拍 mouth 从 home(0) 跳到 −5° 是原版行为。
    #[test]
    fn net_switches_never_jump_targets() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("policies");
        let ort = dir.join("../third_party/onnxruntime/lib/libonnxruntime.so");
        if !ort.exists() || SKILLS.iter().any(|s| !dir.join(s.file).exists()) {
            eprintln!("skip net_switches_never_jump_targets: policies or ORT missing");
            return;
        }
        if std::env::var_os("ORT_DYLIB_PATH").is_none() {
            // SAFETY: 测试进程内设置一次查找路径；ort 首次 API 调用前必须就绪。
            unsafe {
                std::env::set_var("ORT_DYLIB_PATH", &ort);
            }
        }
        let mut scheduler = Scheduler::load(&dir.join("velstand.onnx"), &dir)
            .expect("load scheduler");
        if scheduler.available_names().len() != 2 + SKILLS.len() {
            eprintln!("skip: not all M6 policies loaded");
            return;
        }

        // FakeIo 语义的传感回送：上一拍写出的 targets 就是这一拍读到的位置。
        let mut sensors = Sensors::default();
        sensors.positions = DEFAULT_POSITION;
        let mut previous: Option<[f64; NUM_JOINTS]> = None;
        let mut last_action = [0.0f32; ACTION_LEN];

        let client = Command {
            twist: [0.1, 0.0, 0.0],
            ..Command::default()
        };

        // (拍数, 本段开头要发的边沿)
        let script: &[(u64, u32)] = &[
            (100, 0),                                          // walk
            (1, crate::scheduler::EDGE_SIT_TOGGLE),            // → sit
            (100, 0),                                          // 坐姿
            (1, crate::scheduler::EDGE_SIT_TOGGLE),            // → rise
            (100, 0),                                          // rise 完成后回 walk
            (1, crate::scheduler::EDGE_GROUND_PICK),           // → ground_pick
            (160, 0),                                          // 跑满 140 拍回 walk
            (1, request_bit("kick_left").unwrap()),            // → kick
            (80, 0),                                           // 窗口结束回 walk
        ];

        let scale_of = |label: &str| -> f64 {
            match label {
                "walk" => crate::scheduler::WALK_ACTION_SCALE,
                "sit" | "rise" => crate::scheduler::SITSTAND_ACTION_SCALE,
                "ground_pick" => crate::scheduler::GROUND_PICK_ACTION_SCALE,
                _ => crate::scheduler::SKILL_ACTION_SCALE,
            }
        };

        let mut max_delta = 0.0f64;
        let mut max_where = String::new();
        let mut max_switch_delta = 0.0f64;
        let mut max_switch_where = String::new();
        let mut prev_label = "walk";
        let mut tick = 0u64;
        for (ticks, edges) in script {
            for k in 0..*ticks {
                let e = if k == 0 { *edges } else { 0 };
                let out = driving_tick(
                    &mut scheduler,
                    &sensors,
                    &client,
                    e,
                    0.0,
                    previous.as_ref(),
                    &last_action,
                )
                .expect("driving tick");
                if let Some(prev) = previous {
                    let switched_label = out.label != prev_label;
                    if switched_label {
                        // 机制断言：切换拍的目标 = 以「上拍写出值」为锚点的
                        // 低通结果。锚点若被切换清掉，这里写的是未滤波 raw，
                        // OOD 下单拍摆动数 rad，必然穿帮。
                        let expected =
                            apply_action(&out.action, Some(&prev), scale_of(out.label));
                        for j in 0..NUM_JOINTS {
                            if j == MOUTH_INDEX {
                                continue;
                            }
                            assert_eq!(
                                out.targets[j], expected[j],
                                "切换拍 {prev_label}->{} 关节 {j} 没用上上拍锚点",
                                out.label
                            );
                        }
                    }
                    for j in 0..NUM_JOINTS {
                        if j == MOUTH_INDEX {
                            continue; // 嘴有意图直通，无低通（见上）
                        }
                        let delta = (out.targets[j] - prev[j]).abs();
                        if delta > max_delta {
                            max_delta = delta;
                            max_where = format!(
                                "tick {tick} {prev_label}->{} joint {j}",
                                out.label
                            );
                        }
                        if switched_label && delta > max_switch_delta {
                            max_switch_delta = delta;
                            max_switch_where = format!(
                                "tick {tick} {prev_label}->{} joint {j}",
                                out.label
                            );
                        }
                        assert!(
                            delta <= 5.0,
                            "相邻两拍跳变 {delta:.3} rad @ tick {tick} \
                             ({prev_label} -> {}, joint {j})",
                            out.label
                        );
                    }
                }
                prev_label = out.label;
                sensors.positions = out.targets;
                previous = Some(out.targets);
                last_action = out.action;
                tick += 1;
            }
        }
        eprintln!("continuity: max |Δtarget| = {max_delta:.4} rad ({max_where})");
        eprintln!("continuity: max switch-tick |Δtarget| = {max_switch_delta:.4} rad ({max_switch_where})");
        assert!(max_delta > 0.01, "场景没跑起来？最大相邻差不该接近 0");
        assert!(max_switch_delta > 0.0, "场景里没有发生任何切换？");
    }
}

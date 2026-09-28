//! 50 Hz 控制任务：read → observe → gate → （策略）→ apply，并把最新快照
//! （Sensors + 观测 + 本拍动作 + 安全状态）通过 watch 频道发布给 RPC 层。
//!
//! M5 起所有写电机的路径都收进 Safety（唯一写句柄，见 safety.rs 模块
//! 文档），本循环只通过 Safety 的方法碰总线。状态机是原版
//! Bringup/LimpFall 三态机的简化版（简化已登记偏差簿 D23）：
//!
//!   Held ──enable──▶ RampUp(100拍) ──▶ Driving ◀──▶ Limp（跌倒⇄恢复）
//!     ▲                                    │
//!     └──── RampDown(100拍)+卸torque ◀──disable┘
//!
//! Held 绝不调 set_torque：进程启动不是移动机器人的理由——舵机 RAM 里的
//! torque 跨进程存活，被重启的 daemon 必须让站着的机器人继续站着（D9 收敛）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::watch;
use tokio::time::{Instant, MissedTickBehavior};

use crate::io::{RobotIo, Sensors};
use crate::model::{DEFAULT_POSITION, MOUTH_INDEX, NUM_JOINTS};
use crate::obs::{ACTION_LEN, Command, OBS_LEN, Observation};
use crate::policy::Policy;
use crate::safety::{Safety, SafetyConfig};

pub const TICK_PERIOD: Duration = Duration::from_millis(20);

/// 插值长度：2 秒 × 50 Hz = 100 拍。
pub const RAMP_TICKS: u64 = 100;

/// Walk 槽 action_scale（velstand 挂在 walk）。原型 alpha 默认值；不是 standing 的 1.0。
pub const ACTION_SCALE: f64 = 0.9;

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
    /// 最近一次 robot.drive 的时刻；deadman 据此算意图年龄。
    /// None = 从没被驾驶过。
    pub last_intent_at: Option<std::time::Instant>,
    /// robot.enable/robot.disable 写入；边沿检测在控制循环里做。
    pub enabled: bool,
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

/// `targets = home + 0.9 * scatter(action)`，再按头/腿做一阶低通。
/// `previous` 为 None 时（策略第一拍）不做低通。嘴始终等于 home。
pub fn apply_action(
    action: &[f32; ACTION_LEN],
    previous: Option<&[f64; NUM_JOINTS]>,
) -> [f64; NUM_JOINTS] {
    let offsets = Observation::scatter_action(action);
    let mut targets = [0.0; NUM_JOINTS];
    for j in 0..NUM_JOINTS {
        targets[j] = DEFAULT_POSITION[j] + ACTION_SCALE * offsets[j];
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
        }
    }
}

/// 主循环相位。原版是 Bringup/LimpFall 三态机加 FallPredictor，
/// 这里是偏差簿登记的简化版（D23）：fallen 判定直接触发软倒，
/// 斜坡是逐拍推进的相位而不是阻塞调用（D10，M8 裁决）。
enum Phase {
    /// 抱持：写启动时读到的姿态，绝不动 torque。hold 为 None 表示
    /// 还没读到过一帧（第一次 read 成功时锁存启动姿态）。
    Held { hold: Option<[f64; NUM_JOINTS]> },
    /// 从实测姿态线性斜坡到 home，完成后进 Driving。
    RampUp { start: [f64; NUM_JOINTS], tick: u64 },
    /// 策略闭环。
    Driving,
    /// 跌倒软倒：目标跟随实测位置（倒地过程中固定目标会累积误差=
    /// 电机顶着地板较劲；"软"的关键就是目标跟着身体走），低增益。
    Limp,
    /// 从实测姿态斜坡回 home，完成后卸 torque 回 Held。
    RampDown { start: [f64; NUM_JOINTS], tick: u64 },
}

/// 启动控制任务，返回计数器和最新快照的接收端。
/// `control` 由 RPC 层写（robot.drive/enable/disable）、循环每拍读。
/// `policy` 为 None 时（D19：加载失败不退出）永远停在 Held 抱持。
pub fn spawn(
    mut safety: Safety<Box<dyn RobotIo>>,
    mut policy: Option<Policy>,
    control: SharedControl,
) -> (Arc<Stats>, watch::Receiver<FrameSnapshot>) {
    let config = SafetyConfig::default();
    let stats = Arc::new(Stats::new(policy.is_some()));
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

                let (command, intent_age, enabled) = {
                    let ctl = control.lock().expect("control mutex poisoned");
                    (
                        ctl.command,
                        ctl.last_intent_at
                            .map(|t| t.elapsed())
                            .unwrap_or(Duration::MAX),
                        ctl.enabled,
                    )
                };
                // gate 每拍都调（armed 语义防日志噪音）；Driving 用 gated 命令组 obs。
                let (gated, _limit) = safety.gate(command, intent_age);

                // enable/disable 边沿检测。只在边沿动作，绝不在 Held 里
                // 每拍碰 torque。边沿立即生效（先于跌倒判定），否则
                // "Driving 中跌倒"与"同拍 disable"会互相覆盖。
                if enabled && !was_enabled {
                    // 没策略的机器人 enable 无意义：留在 Held 抱持（D19）。
                    if policy.is_some() && matches!(phase, Phase::Held { .. }) {
                        match safety.set_torque(true) {
                            Ok(()) => {
                                torque_on = true;
                                phase = Phase::RampUp {
                                    start: sensors.positions,
                                    tick: 0,
                                };
                            }
                            Err(e) => eprintln!("set_torque(true) failed: {e}"),
                        }
                    }
                } else if !enabled
                    && was_enabled
                    && !matches!(phase, Phase::Held { .. } | Phase::RampDown { .. })
                {
                    phase = Phase::RampDown {
                        start: sensors.positions,
                        tick: 0,
                    };
                }
                was_enabled = enabled;

                // 跌倒转移（只在 Driving/Limp 间；斜坡/抱持期间的跌倒
                // 不改变行为——斜坡回 home 本身就是对的恢复动作）。
                match phase {
                    Phase::Driving if safety.fallen() => {
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

                // 观测始终用当前 last_action 与 gated 命令组装；Driving 成功推理后
                // 才更新 last_action，所以快照里的 obs 是「本拍喂给策略的向量」。
                let observation = Observation::build(
                    &sensors.imu,
                    &sensors.positions,
                    &sensors.velocities,
                    &DEFAULT_POSITION,
                    &last_action,
                    &gated,
                );

                let mut action_out = [0.0f32; ACTION_LEN];
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
                            next_phase = Some(Phase::Driving);
                        }
                    }
                    Phase::RampDown { start, tick: t } => {
                        let target = ramp_target(start, *t);
                        wrote = safety.apply(target, hold, config.gain_running).is_ok();
                        *t += 1;
                        if *t > RAMP_TICKS {
                            match safety.set_torque(false) {
                                Ok(()) => torque_on = false,
                                Err(e) => eprintln!("set_torque(false) failed: {e}"),
                            }
                            next_phase = Some(Phase::Held {
                                hold: Some(DEFAULT_POSITION),
                            });
                        }
                    }
                    Phase::Driving => {
                        match policy.as_mut() {
                            Some(p) => match p.infer(&observation) {
                                Ok(action) => {
                                    let targets =
                                        apply_action(&action, previous_targets.as_ref());
                                    previous_targets = Some(targets);
                                    last_action = action;
                                    action_out = action;
                                    wrote = safety
                                        .apply(targets, hold, config.gain_running)
                                        .is_ok();
                                }
                                Err(e) => {
                                    // 本拍不写、不更新 last_action；循环继续。
                                    eprintln!("policy infer failed: {e}");
                                }
                            },
                            // 进不了这里：policy 为 None 时永远停在 Held。
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

                let mut obs_arr = [0.0f32; OBS_LEN];
                obs_arr.copy_from_slice(observation.as_slice());
                let _ = frame_tx.send(FrameSnapshot {
                    sensors: sensors.clone(),
                    obs: obs_arr,
                    action: action_out,
                    fallen: safety.fallen(),
                    enabled,
                    gain: safety.gain(),
                    torque: torque_on,
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

        let targets = apply_action(&action, None);
        let scattered = Observation::scatter_action(&action);

        for j in 0..NUM_JOINTS {
            let expected = DEFAULT_POSITION[j] + ACTION_SCALE * scattered[j];
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
        let raw = apply_action(&action, None);
        let previous = [0.0f64; NUM_JOINTS]; // 人为锚点，方便验算系数
        let filtered = apply_action(&action, Some(&previous));

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
}

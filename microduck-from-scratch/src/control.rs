//! 50 Hz 控制任务：read → 插值或策略推理 → write，并把最新快照
//! （Sensors + 观测 + 本拍动作）通过 watch 频道发布给 RPC 层。
//!
//! M1–M2：插值到 home 后保持。M3：插值满 RAMP_TICKS 后用 velstand
//! 闭环站立；FakeIo 完美跟踪，站立 = 停在 home 附近，且上一拍动作回灌观测。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::watch;
use tokio::time::{Instant, MissedTickBehavior};

use crate::io::{JointTargets, RobotIo, Sensors};
use crate::model::{DEFAULT_POSITION, MOUTH_INDEX, NUM_JOINTS};
use crate::obs::{ACTION_LEN, Command, OBS_LEN, Observation};
use crate::policy::Policy;

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
/// 不是"socket 活着"）。
#[derive(Default)]
pub struct Stats {
    pub tick: AtomicU64,
    pub reads: AtomicU64,
    pub writes: AtomicU64,
    pub skipped_reads: AtomicU64,
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
/// watch 里留着上一帧成功的快照，1 Hz 推送会重复它，而不是造一帧空数据。
#[derive(Debug, Clone)]
pub struct FrameSnapshot {
    pub sensors: Sensors,
    pub obs: [f32; OBS_LEN],
    /// 本拍刚写出的策略动作；插值期间或本拍未写出时为 0。
    pub action: [f32; ACTION_LEN],
}

impl Default for FrameSnapshot {
    fn default() -> Self {
        Self {
            sensors: Sensors::default(),
            obs: [0.0; OBS_LEN],
            action: [0.0; ACTION_LEN],
        }
    }
}

/// 启动控制任务，返回计数器和最新快照的接收端。
pub fn spawn(
    mut io: impl RobotIo + 'static,
    mut policy: Policy,
) -> (Arc<Stats>, watch::Receiver<FrameSnapshot>) {
    let stats = Arc::new(Stats::default());
    let (frame_tx, frame_rx) = watch::channel(FrameSnapshot::default());

    {
        let stats = stats.clone();
        tokio::spawn(async move {
            // interval_at 而非 interval：interval 的第一拍立即触发，
            // 第 0 帧会在初始姿态还没读到时就跑控制逻辑、把垃圾数据
            // 写上总线。推迟一个周期，让每一帧都走同一条
            // read → compute → write 路径。（已知坑，勿改回 interval。）
            let mut timer = tokio::time::interval_at(Instant::now() + TICK_PERIOD, TICK_PERIOD);
            // 原版用 Skip：积压时丢拍，避免 Burst 连发电机命令；Delay 实测掉到 43.1Hz。
            timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

            // 插值起点 = 第一次 read 成功时的姿态和拍号，而不是进程
            // 启动时刻：舵机电源未就绪时前若干拍 read 全失败，从启动
            // 时刻起算会让插值在总线恢复时已经走完、机器人瞬间跳变。
            let mut ramp_origin: Option<(u64, [f64; NUM_JOINTS])> = None;
            let mut tick: u64 = 0;
            // 插值期间保持 0；策略成功拍才更新，失败不改。
            let mut last_action = [0.0f32; ACTION_LEN];
            // 策略输出低通锚点；插值阶段不算，策略第一拍为 None。
            let mut previous_targets: Option<[f64; NUM_JOINTS]> = None;

            // 速率/抖动统计：每 5 秒一行，给 grep 用。
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
                    deltas_ms.clear();
                    window_ticks = 0;
                    window_start = Instant::now();
                }

                let sensors = match io.read() {
                    Ok(s) => s,
                    Err(_) => {
                        stats.skipped_reads.fetch_add(1, Ordering::Relaxed);
                        tick += 1;
                        stats.tick.store(tick, Ordering::Relaxed);
                        continue;
                    }
                };
                stats.reads.fetch_add(1, Ordering::Relaxed);

                let (origin_tick, start_pose) =
                    *ramp_origin.get_or_insert((tick, sensors.positions));
                let tick_in_ramp = tick - origin_tick;

                // 观测始终用当前 last_action 组装；成功推理后再更新 last_action，
                // 所以快照里的 obs 是「本拍喂给策略的向量」，不是写回后的下一拍。
                let observation = Observation::build(
                    &sensors.imu,
                    &sensors.positions,
                    &sensors.velocities,
                    &DEFAULT_POSITION,
                    &last_action,
                    &Command::default(),
                );

                let (target, action_out) = if tick_in_ramp < RAMP_TICKS {
                    // 插值阶段不跑推理；last_action 保持 0。
                    (ramp_target(&start_pose, tick_in_ramp), [0.0f32; ACTION_LEN])
                } else {
                    match policy.infer(&observation) {
                        Ok(action) => {
                            let targets = apply_action(&action, previous_targets.as_ref());
                            previous_targets = Some(targets);
                            last_action = action;
                            (targets, action)
                        }
                        Err(e) => {
                            // 本拍不写、不更新 last_action；循环继续。
                            eprintln!("policy infer failed: {e}");
                            let mut obs_arr = [0.0f32; OBS_LEN];
                            obs_arr.copy_from_slice(observation.as_slice());
                            let _ = frame_tx.send(FrameSnapshot {
                                sensors: sensors.clone(),
                                obs: obs_arr,
                                action: [0.0; ACTION_LEN],
                            });
                            tick += 1;
                            stats.tick.store(tick, Ordering::Relaxed);
                            continue;
                        }
                    }
                };

                let mut obs_arr = [0.0f32; OBS_LEN];
                obs_arr.copy_from_slice(observation.as_slice());
                let _ = frame_tx.send(FrameSnapshot {
                    sensors: sensors.clone(),
                    obs: obs_arr,
                    action: action_out,
                });

                if io.write(&JointTargets { positions: target }).is_ok() {
                    stats.writes.fetch_add(1, Ordering::Relaxed);
                }

                tick += 1;
                stats.tick.store(tick, Ordering::Relaxed);
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

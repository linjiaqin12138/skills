//! 50 Hz 控制任务：read → 计算本拍目标 → write，并把最新 Sensors
//! 通过 watch 频道发布给 RPC 层。
//!
//! M1 的全部策略就是"起立"：从启动时的当前姿态出发，2 秒线性插值到
//! DEFAULT_POSITION，之后保持。插值做成纯函数 `ramp_target`，
//! 是为了让"1 秒时在中点"能用普通单元测试断言，而不是去跑真时钟。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tokio::sync::watch;
use tokio::time::Instant;

use crate::io::{JointTargets, RobotIo, Sensors};
use crate::model::{DEFAULT_POSITION, NUM_JOINTS};

pub const TICK_PERIOD: Duration = Duration::from_millis(20);

/// 插值长度：2 秒 × 50 Hz = 100 拍。
pub const RAMP_TICKS: u64 = 100;

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

/// 启动控制任务，返回计数器和最新传感数据的接收端。
pub fn spawn(mut io: impl RobotIo + 'static) -> (Arc<Stats>, watch::Receiver<Sensors>) {
    let stats = Arc::new(Stats::default());
    let (sensors_tx, sensors_rx) = watch::channel(Sensors::default());

    {
        let stats = stats.clone();
        tokio::spawn(async move {
            // interval_at 而非 interval：interval 的第一拍立即触发，
            // 第 0 帧会在初始姿态还没读到时就跑控制逻辑、把垃圾数据
            // 写上总线。推迟一个周期，让每一帧都走同一条
            // read → compute → write 路径。（已知坑，勿改回 interval。）
            let mut timer = tokio::time::interval_at(Instant::now() + TICK_PERIOD, TICK_PERIOD);

            // 插值起点 = 第一次 read 成功时的姿态和拍号，而不是进程
            // 启动时刻：舵机电源未就绪时前若干拍 read 全失败，从启动
            // 时刻起算会让插值在总线恢复时已经走完、机器人瞬间跳变。
            let mut ramp_origin: Option<(u64, [f64; NUM_JOINTS])> = None;
            let mut tick: u64 = 0;

            // 速率/抖动统计：每 5 秒一行，给 grep 用。
            let mut window_start = Instant::now();
            let mut window_ticks: u64 = 0;
            let mut last_tick_at: Option<Instant> = None;
            let mut deltas_ms: Vec<f64> = Vec::with_capacity(256);

            loop {
                let now = timer.tick().await;
                if let Some(prev) = last_tick_at {
                    // 抖动 = 相邻拍间隔相对标称周期的偏差，不是间隔本身
                    // （间隔的 P99 恒等于 20ms，没有信息量）。
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
                        // 读失败（总线未就绪/瞬断）：跳过本拍，计数，
                        // 不 panic 不退出——机器人没电是可以恢复的状态，
                        // 进程退出不是。
                        stats.skipped_reads.fetch_add(1, Ordering::Relaxed);
                        tick += 1;
                        stats.tick.store(tick, Ordering::Relaxed);
                        continue;
                    }
                };
                stats.reads.fetch_add(1, Ordering::Relaxed);
                let _ = sensors_tx.send(sensors.clone());

                let (origin_tick, start_pose) =
                    *ramp_origin.get_or_insert((tick, sensors.positions));
                let target = ramp_target(&start_pose, tick - origin_tick);
                if io.write(&JointTargets { positions: target }).is_ok() {
                    stats.writes.fetch_add(1, Ordering::Relaxed);
                }

                tick += 1;
                stats.tick.store(tick, Ordering::Relaxed);
            }
        });
    }

    (stats, sensors_rx)
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
        // 保持：插值结束后钉在 home。
        assert_eq!(ramp_target(&start, RAMP_TICKS * 3), DEFAULT_POSITION);
    }

    #[test]
    fn ramp_midpoint_at_one_second() {
        let start = [0.0; NUM_JOINTS];
        let mid = ramp_target(&start, 50); // 1 秒 = 50 拍
        for i in 0..NUM_JOINTS {
            let expected = (start[i] + DEFAULT_POSITION[i]) / 2.0;
            assert!(
                (mid[i] - expected).abs() < 1e-6,
                "joint {i}: {mid:?} vs expected midpoint {expected}"
            );
        }
    }
}

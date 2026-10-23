//! 固定频率控制循环：50 Hz = 每 20ms 一拍，节拍本身是可测量的。
use tokio::time::{Duration, Instant, interval_at};

#[tokio::main]
async fn main() {
    let period = Duration::from_millis(20); // 50 Hz
    let start = Instant::now();
    let mut timer = interval_at(start + period, period);
    let mut ticks = 0u32;
    while start.elapsed() < Duration::from_millis(500) {
        timer.tick().await; // 真实项目里：读传感器 → 算目标 → 写执行器
        ticks += 1;
    }
    let hz = ticks as f64 / start.elapsed().as_secs_f64();
    println!("500ms 内跑了 {ticks} 拍，实测 {hz:.1} Hz");
}

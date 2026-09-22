//! watch：只保留最新值的广播。快生产者 + 慢消费者，中间值被丢弃。
use tokio::time::{Duration, interval, sleep};

#[tokio::main]
async fn main() {
    let (tx, rx) = tokio::sync::watch::channel(0u64);
    tokio::spawn(async move {
        let mut t = interval(Duration::from_millis(10)); // 快生产者
        for i in 1..=50 {
            t.tick().await;
            tx.send(i).unwrap();
        }
    });
    for _ in 0..3 {
        sleep(Duration::from_millis(180)).await; // 慢消费者
        println!("消费者只读到当时的最新值: {}", *rx.borrow());
    }
}

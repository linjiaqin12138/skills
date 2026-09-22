//! select!：同时等两个事件源，谁先就绪执行谁，输的分支被取消。
use tokio::time::{Duration, interval, sleep};

#[tokio::main]
async fn main() {
    let mut timer = interval(Duration::from_millis(100));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<&str>(1);
    tokio::spawn(async move {
        sleep(Duration::from_millis(250)).await;
        tx.send("客户端请求").await.unwrap();
    });
    for _ in 0..4 {
        tokio::select! {
            _ = timer.tick() => println!("分支A：定时器到点"),
            Some(msg) = rx.recv() => println!("分支B：收到{msg}"),
        }
    }
}

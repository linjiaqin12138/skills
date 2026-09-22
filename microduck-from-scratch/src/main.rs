//! miniduckd：M1 的守护进程。
//!
//! M0 的 JSON-RPC 骨架（hello / robot.health / robot.state 订阅）之上，
//! 现在有一个真的 50 Hz 控制任务在跑：FakeIo 假总线从全零姿态出发，
//! 2 秒线性插值到 home 姿态并保持（见 control.rs）。M0 的"tick 计数器"
//! 被这个控制任务接管——tick 的语义仍是"控制循环在跑"，只是现在
//! 循环里真的有 read → compute → write 了。

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use futures::{SinkExt, StreamExt};
use miniduck::io::{FakeIo, Sensors};
use miniduck::{API_VERSION, METHOD_NOT_FOUND, PARSE_ERROR, Request, ServerMessage, control};
use serde_json::json;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch;
use tokio_util::codec::{Framed, LinesCodec};

const SOCK_PATH: &str = "/tmp/miniduckd.sock";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let boot = Instant::now();

    // 故障注入开关（容错验收用）：MINIDUCK_FAKE_FAILING_READS=100 让
    // 假总线前 100 次 read 报错，模拟舵机电源未就绪。默认 0。
    let failing_reads = std::env::var("MINIDUCK_FAKE_FAILING_READS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let io = FakeIo::failing_reads(failing_reads);

    // 50 Hz 控制任务。stats 由循环自己记账，sensors_rx 是最新一帧
    // 传感数据的订阅口（watch：人人读到的是"最新值"，不是队列）。
    let (stats, sensors_rx) = control::spawn(io);

    // 上次异常退出残留的 socket 文件会让 bind 报 AddrInUse。先清再绑。
    let _ = std::fs::remove_file(SOCK_PATH);
    let listener = UnixListener::bind(SOCK_PATH)?;
    eprintln!("miniduckd listening on {SOCK_PATH}");

    loop {
        let (stream, _) = listener.accept().await?;
        // 每个连接一个任务：某条连接的客户端卡住不能拖死其他连接。
        tokio::spawn(serve(stream, stats.clone(), sensors_rx.clone(), boot));
    }
}

async fn serve(
    stream: UnixStream,
    stats: Arc<control::Stats>,
    sensors_rx: watch::Receiver<Sensors>,
    boot: Instant,
) {
    let mut framed = Framed::new(stream, LinesCodec::new());
    let mut subscribed = false;

    // 订阅推送频率仍为 1 Hz；对齐原版 50 Hz 的 robot.state 流留给
    // 客户端真的吃得下 50 Hz 的里程碑。
    let mut push = tokio::time::interval(Duration::from_secs(1));
    push.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        // select! 让"读请求"和"推通知"在一条连接上并存——
        // 这就是当初选 JSON-RPC/NDJSON 而非 HTTP 的那半个理由的实体化。
        tokio::select! {
            frame = framed.next() => {
                let line = match frame {
                    Some(Ok(line)) => line,
                    // 对端断开或帧损坏都是正常结局：任何对端都可能死。
                    _ => break,
                };
                let req: Request = match serde_json::from_str(&line) {
                    Ok(req) => req,
                    Err(e) => {
                        // 解析失败时 id 不可知，JSON-RPC 约定用 null；这里以 0 代之（M0 简化）。
                        let msg = ServerMessage::err(0, PARSE_ERROR, e.to_string());
                        if framed.send(serde_json::to_string(&msg).unwrap()).await.is_err() {
                            break;
                        }
                        continue;
                    }
                };

                let resp = match req.method.as_str() {
                    "hello" => ServerMessage::ok(
                        req.id,
                        json!({ "service": "miniduckd", "api_version": API_VERSION }),
                    ),
                    "robot.health" => ServerMessage::ok(
                        req.id,
                        json!({
                            "healthy": true,
                            "tick": stats.tick.load(Ordering::Relaxed),
                            "uptime_s": boot.elapsed().as_secs(),
                            "reads": stats.reads.load(Ordering::Relaxed),
                            "writes": stats.writes.load(Ordering::Relaxed),
                            "skipped_reads": stats.skipped_reads.load(Ordering::Relaxed),
                        }),
                    ),
                    "robot.state" => {
                        subscribed = true;
                        ServerMessage::ok(req.id, json!({ "subscribed": true }))
                    }
                    // 未知方法按名字拒绝——这是协议里唯一被允许的"拒绝"。
                    other => ServerMessage::err(
                        req.id,
                        METHOD_NOT_FOUND,
                        format!("method not found: {other}"),
                    ),
                };
                if framed.send(serde_json::to_string(&resp).unwrap()).await.is_err() {
                    break;
                }
            }
            _ = push.tick(), if subscribed => {
                // borrow() 只在取数这一瞬持有，随后立即释放——不能带着
                // watch 的读锁跨 await（ send 是异步的），否则控制任务
                // 每拍 publish 时都要等这条连接的网络。
                let positions = sensors_rx.borrow().positions;
                let note = ServerMessage::notify(
                    "robot.state",
                    json!({
                        "tick": stats.tick.load(Ordering::Relaxed),
                        "positions": positions,
                    }),
                );
                if framed.send(serde_json::to_string(&note).unwrap()).await.is_err() {
                    break;
                }
            }
        }
    }
}

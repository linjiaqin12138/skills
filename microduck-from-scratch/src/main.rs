//! miniduckd：M3 的守护进程。
//!
//! M0 的 JSON-RPC 骨架（hello / robot.health / robot.state 订阅）之上，
//! 50 Hz 控制任务：FakeIo 从全零插值到 home，满 100 拍后用 velstand.onnx
//! 闭环站立。robot.state 附带 IMU、61 维观测与 14 维动作（仍 1 Hz 推送）。

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use futures::{SinkExt, StreamExt};
use miniduck::io::FakeIo;
use miniduck::policy::Policy;
use miniduck::{API_VERSION, METHOD_NOT_FOUND, PARSE_ERROR, Request, ServerMessage, control};
use serde_json::json;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch;
use tokio_util::codec::{Framed, LinesCodec};

const SOCK_PATH: &str = "/tmp/miniduckd.sock";
const DEFAULT_POLICY: &str = "policies/velstand.onnx";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let boot = Instant::now();

    let policy_path = std::env::var("MINIDUCK_POLICY").unwrap_or_else(|_| DEFAULT_POLICY.into());
    let policy = Policy::load(&policy_path).map_err(|e| {
        eprintln!("failed to load policy {policy_path}: {e}");
        e
    })?;

    // 故障注入开关（容错验收用）：MINIDUCK_FAKE_FAILING_READS=100 让
    // 假总线前 100 次 read 报错，模拟舵机电源未就绪。默认 0。
    let failing_reads = std::env::var("MINIDUCK_FAKE_FAILING_READS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let io = FakeIo::failing_reads(failing_reads);

    // 50 Hz 控制任务。stats 由循环自己记账，frame_rx 是最新快照的订阅口。
    let (stats, frame_rx) = control::spawn(io, policy);

    // 上次异常退出残留的 socket 文件会让 bind 报 AddrInUse。先清再绑。
    let _ = std::fs::remove_file(SOCK_PATH);
    let listener = UnixListener::bind(SOCK_PATH)?;
    eprintln!("miniduckd listening on {SOCK_PATH}");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(serve(stream, stats.clone(), frame_rx.clone(), boot));
    }
}

async fn serve(
    stream: UnixStream,
    stats: Arc<control::Stats>,
    frame_rx: watch::Receiver<control::FrameSnapshot>,
    boot: Instant,
) {
    let mut framed = Framed::new(stream, LinesCodec::new());
    let mut subscribed = false;

    // 订阅推送频率仍为 1 Hz；对齐原版 50 Hz 的 robot.state 流留给
    // 客户端真的吃得下 50 Hz 的里程碑。
    let mut push = tokio::time::interval(Duration::from_secs(1));
    push.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            frame = framed.next() => {
                let line = match frame {
                    Some(Ok(line)) => line,
                    _ => break,
                };
                let req: Request = match serde_json::from_str(&line) {
                    Ok(req) => req,
                    Err(e) => {
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
                let frame = frame_rx.borrow().clone();
                let imu = &frame.sensors.imu;
                let note = ServerMessage::notify(
                    "robot.state",
                    json!({
                        "tick": stats.tick.load(Ordering::Relaxed),
                        "positions": frame.sensors.positions,
                        "imu": {
                            "gyro": imu.gyro,
                            "gravity": imu.gravity,
                            "quat": imu.quat,
                        },
                        "obs": frame.obs.as_slice(),
                        "action": frame.action.as_slice(),
                    }),
                );
                if framed.send(serde_json::to_string(&note).unwrap()).await.is_err() {
                    break;
                }
            }
        }
    }
}

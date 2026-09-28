//! mini-duckctl：M0 的客户端。每次连接先 hello 再发目标调用——
//! 握手不是可选项，是任何客户端进来要做的第一件事（原版同）。

use std::io::Write as _;

use futures::{SinkExt, StreamExt};
use miniduck::{Request, ServerMessage};
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LinesCodec};

const SOCK_PATH: &str = "/tmp/miniduckd.sock";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // M0 用手写参数解析；clap 留给功能面膨胀到值得它的里程碑。
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().cloned().unwrap_or_else(|| {
        eprintln!(
            "usage: mini-duckctl <health|state [--every N]|enable|disable|drive <vx> <vyaw> [--secs N]>"
        );
        std::process::exit(2);
    });

    let stream = UnixStream::connect(SOCK_PATH).await.unwrap_or_else(|e| {
        eprintln!("connect {SOCK_PATH}: {e}（守护进程在跑吗？）");
        std::process::exit(1);
    });
    let mut framed = Framed::new(stream, LinesCodec::new());

    hello(&mut framed).await?;

    match cmd.as_str() {
        "health" => {
            call(&mut framed, 2, "robot.health", serde_json::Value::Null).await?;
        }
        "enable" | "disable" => {
            call(
                &mut framed,
                2,
                &format!("robot.{cmd}"),
                serde_json::Value::Null,
            )
            .await?;
        }
        // state [--every N]：订阅 robot.state 通知流。50Hz 全打印会刷屏，
        // 默认每 50 帧打一行；调试验收要逐帧时给 --every 1。
        "state" | "subscribe" => {
            let every: u64 = match args.get(1).map(String::as_str) {
                Some("--every") => args.get(2).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                    eprintln!("--every wants a positive integer");
                    std::process::exit(2);
                }),
                None if cmd == "subscribe" => 1, // 旧名保持旧行为：逐帧打印
                None => 50,
                Some(other) => {
                    eprintln!("unknown state flag: {other}");
                    std::process::exit(2);
                }
            };
            call(&mut framed, 2, "robot.state", serde_json::Value::Null).await?;
            // 之后这条连接上只剩通知流，打到对端断开或 Ctrl-C。
            let mut n: u64 = 0;
            while let Some(frame) = framed.next().await {
                match frame {
                    Ok(line) => {
                        n += 1;
                        if n % every == 0 {
                            println!("{line}");
                        }
                    }
                    Err(e) => {
                        eprintln!("read: {e}");
                        std::process::exit(1);
                    }
                }
                let _ = std::io::stdout().flush();
            }
        }
        "drive" => {
            // drive <vx> <vyaw> [--secs N]：deadman 500ms 下单次意图只能
            // 驱动半秒，所以带 --secs 时每 100ms 重发一次意图——CLI 扮演
            // 手柄的角色，手柄就是持续发意图的。到时发零命令停车。
            // 不带 --secs 保持单次发送（之后由调用方负责停车）。
            let vx: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                eprintln!("usage: mini-duckctl drive <vx> <vyaw> [--secs N]");
                std::process::exit(2);
            });
            let vyaw: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                eprintln!("usage: mini-duckctl drive <vx> <vyaw> [--secs N]");
                std::process::exit(2);
            });
            let secs: Option<u64> = match args.get(3).map(String::as_str) {
                Some("--secs") => args.get(4).and_then(|s| s.parse().ok()),
                None => None,
                Some(other) => {
                    eprintln!("unknown drive flag: {other}");
                    std::process::exit(2);
                }
            };
            let mut id = 2;
            call(&mut framed, id, "robot.drive", serde_json::json!({"vx": vx, "vyaw": vyaw}))
                .await?;
            if let Some(secs) = secs {
                let deadline = tokio::time::Instant::now()
                    + std::time::Duration::from_secs(secs);
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    if tokio::time::Instant::now() >= deadline {
                        break;
                    }
                    id += 1;
                    // 重发是心跳不是新闻：静默收发，10 秒车程不该刷 100 行。
                    send(
                        &mut framed,
                        id,
                        "robot.drive",
                        serde_json::json!({"vx": vx, "vyaw": vyaw}),
                    )
                    .await?;
                    framed.next().await;
                }
                id += 1;
                call(
                    &mut framed,
                    id,
                    "robot.drive",
                    serde_json::json!({"vx": 0.0, "vyaw": 0.0}),
                )
                .await?;
            }
        }
        other => {
            eprintln!("unknown command: {other}");
            std::process::exit(2);
        }
    }
    Ok(())
}

type Conn = Framed<UnixStream, LinesCodec>;

async fn hello(framed: &mut Conn) -> std::io::Result<()> {
    send(framed, 1, "hello", serde_json::Value::Null).await?;
    let line = framed
        .next()
        .await
        .ok_or_else(|| std::io::Error::other("daemon closed before hello reply"))?
        .map_err(std::io::Error::other)?;
    // 版本差异只报告——协议设计原则见 lib.rs 的 API_VERSION 注释。
    let msg: ServerMessage = serde_json::from_str(&line)
        .map_err(|e| std::io::Error::other(format!("bad hello reply: {e}")))?;
    eprintln!("hello <- {line}");
    let _ = msg;
    Ok(())
}

async fn call(
    framed: &mut Conn,
    id: u64,
    method: &str,
    params: serde_json::Value,
) -> std::io::Result<()> {
    send(framed, id, method, params).await?;
    let line = framed
        .next()
        .await
        .ok_or_else(|| std::io::Error::other("daemon closed before reply"))?
        .map_err(std::io::Error::other)?;
    println!("{line}");
    Ok(())
}

async fn send(
    framed: &mut Conn,
    id: u64,
    method: &str,
    params: serde_json::Value,
) -> std::io::Result<()> {
    let req = Request {
        jsonrpc: miniduck::JSONRPC.into(),
        id,
        method: method.into(),
        params,
    };
    framed
        .send(serde_json::to_string(&req).unwrap())
        .await
        .map_err(std::io::Error::other)
}

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
    let cmd = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: mini-duckctl <health|subscribe>");
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
            call(&mut framed, 2, "robot.health").await?;
        }
        "subscribe" => {
            call(&mut framed, 2, "robot.state").await?;
            // 之后这条连接上只剩通知流，打到对端断开或 Ctrl-C。
            while let Some(frame) = framed.next().await {
                match frame {
                    Ok(line) => println!("{line}"),
                    Err(e) => {
                        eprintln!("read: {e}");
                        std::process::exit(1);
                    }
                }
                let _ = std::io::stdout().flush();
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
    send(framed, 1, "hello").await?;
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

async fn call(framed: &mut Conn, id: u64, method: &str) -> std::io::Result<()> {
    send(framed, id, method).await?;
    let line = framed
        .next()
        .await
        .ok_or_else(|| std::io::Error::other("daemon closed before reply"))?
        .map_err(std::io::Error::other)?;
    println!("{line}");
    Ok(())
}

async fn send(framed: &mut Conn, id: u64, method: &str) -> std::io::Result<()> {
    let req = Request {
        jsonrpc: miniduck::JSONRPC.into(),
        id,
        method: method.into(),
        params: serde_json::Value::Null,
    };
    framed
        .send(serde_json::to_string(&req).unwrap())
        .await
        .map_err(std::io::Error::other)
}

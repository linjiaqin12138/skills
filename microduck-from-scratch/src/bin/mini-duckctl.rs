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
            "usage: mini-duckctl <health|state [--every N]|enable|disable|move <vx> <vy> <vyaw> [--secs N]|do <skill>|skills|mouth <0..1>|head <np> <hp> <hy> <hr>|update <check|status|log [N]|apply <ver>|rollback>>"
        );
        std::process::exit(2);
    });

    // M7：update 子命令组连的是 mini-updaterd 的 socket，不是 miniduckd
    // 的——在建立 daemon 连接之前分流。
    if cmd == "update" {
        return update_cmd(&args[1..]).await;
    }

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
        "move" => {
            // move <vx> <vy> <vyaw> [--secs N]：deadman 500ms 下单次意图只能
            // 驱动半秒，所以带 --secs 时每 100ms 重发一次意图——CLI 扮演
            // 手柄的角色，手柄就是持续发意图的。到时发零命令停车。
            // 不带 --secs 保持单次发送（之后由调用方负责停车）。
            // M8 起发 robot.move（原版语义）；仍走请求式调用拿 accepted 回显。
            let vx: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                eprintln!("usage: mini-duckctl move <vx> <vy> <vyaw> [--secs N]");
                std::process::exit(2);
            });
            let vy: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                eprintln!("usage: mini-duckctl move <vx> <vy> <vyaw> [--secs N]");
                std::process::exit(2);
            });
            let vyaw: f64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                eprintln!("usage: mini-duckctl move <vx> <vy> <vyaw> [--secs N]");
                std::process::exit(2);
            });
            let secs: Option<u64> = match args.get(4).map(String::as_str) {
                Some("--secs") => args.get(5).and_then(|s| s.parse().ok()),
                None => None,
                Some(other) => {
                    eprintln!("unknown move flag: {other}");
                    std::process::exit(2);
                }
            };
            let mut id = 2;
            call(
                &mut framed,
                id,
                "robot.move",
                serde_json::json!({"vx": vx, "vy": vy, "vyaw": vyaw}),
            )
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
                        "robot.move",
                        serde_json::json!({"vx": vx, "vy": vy, "vyaw": vyaw}),
                    )
                    .await?;
                    framed.next().await;
                }
                id += 1;
                call(
                    &mut framed,
                    id,
                    "robot.move",
                    serde_json::json!({"vx": 0.0, "vy": 0.0, "vyaw": 0.0}),
                )
                .await?;
            }
        }
        // M6：技能请求。do <skill>（edge 语义，仲裁在控制循环）；skills 列名单。
        "do" => {
            let skill = args.get(1).cloned().unwrap_or_else(|| {
                eprintln!("usage: mini-duckctl do <skill>");
                std::process::exit(2);
            });
            call(&mut framed, 2, "robot.do", serde_json::json!({"skill": skill})).await?;
        }
        "skills" => {
            call(&mut framed, 2, "robot.skills", serde_json::Value::Null).await?;
        }
        // M6：嘴开度 0..1。
        "mouth" => {
            let position: f64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                eprintln!("usage: mini-duckctl mouth <0..1>");
                std::process::exit(2);
            });
            call(
                &mut framed,
                2,
                "robot.mouth",
                serde_json::json!({"position": position}),
            )
            .await?;
        }
        // M6：头姿，四个关节角（弧度）。
        "head" => {
            let parse = |i: usize| -> f64 {
                args.get(i).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                    eprintln!("usage: mini-duckctl head <neck_pitch> <head_pitch> <head_yaw> <head_roll>");
                    std::process::exit(2);
                })
            };
            call(
                &mut framed,
                2,
                "robot.head",
                serde_json::json!({
                    "neck_pitch": parse(1),
                    "head_pitch": parse(2),
                    "head_yaw": parse(3),
                    "head_roll": parse(4),
                }),
            )
            .await?;
        }
        other => {
            eprintln!("unknown command: {other}");
            std::process::exit(2);
        }
    }
    Ok(())
}

type Conn = Framed<UnixStream, LinesCodec>;

const UPDATER_SOCK: &str = "/tmp/mini-updaterd.sock";

/// M7：update <check|status|log [N]|apply <ver>|rollback>，连 mini-updaterd。
async fn update_cmd(args: &[String]) -> std::io::Result<()> {
    let sub = args.first().map(String::as_str).unwrap_or_else(|| {
        eprintln!("usage: mini-duckctl update <check|status|log [N]|apply <ver>|rollback>");
        std::process::exit(2);
    });
    let sock = std::env::var("MINIDUCK_UPDATER_SOCK").unwrap_or_else(|_| UPDATER_SOCK.into());
    let stream = UnixStream::connect(&sock).await.unwrap_or_else(|e| {
        eprintln!("connect {sock}: {e}（mini-updaterd 在跑吗？）");
        std::process::exit(1);
    });
    let mut framed = Framed::new(stream, LinesCodec::new());
    hello(&mut framed).await?;

    match sub {
        "check" => call(&mut framed, 2, "update.check", serde_json::Value::Null).await?,
        "status" => call(&mut framed, 2, "update.status", serde_json::Value::Null).await?,
        "log" => {
            let params = match args.get(1) {
                Some(n) => serde_json::json!({"n": n.parse::<u64>().unwrap_or_else(|_| {
                    eprintln!("usage: mini-duckctl update log [N]");
                    std::process::exit(2);
                })}),
                None => serde_json::Value::Null,
            };
            call(&mut framed, 2, "update.log", params).await?;
        }
        "apply" => {
            let version = args.get(1).cloned().unwrap_or_else(|| {
                eprintln!("usage: mini-duckctl update apply <version>");
                std::process::exit(2);
            });
            call(&mut framed, 2, "update.apply", serde_json::json!({"version": version})).await?;
        }
        "rollback" => call(&mut framed, 2, "update.rollback", serde_json::Value::Null).await?,
        other => {
            eprintln!("unknown update subcommand: {other}");
            std::process::exit(2);
        }
    }
    Ok(())
}

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
        id: Some(id),
        method: method.into(),
        params,
    };
    framed
        .send(serde_json::to_string(&req).unwrap())
        .await
        .map_err(std::io::Error::other)
}

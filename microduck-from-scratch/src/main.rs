//! miniduckd：M5 的守护进程。
//!
//! M0 的 JSON-RPC 骨架（hello / robot.health / robot.state 订阅）之上，
//! 50 Hz 控制任务经 Safety（唯一写句柄）驱动身体。
//! M5 新增：`robot.enable` / `robot.disable` 使能开关；`robot.health`
//! 报真实判定（D11）；`robot.state` 改为 50 Hz 逐帧推送（D14）；
//! 策略加载失败不再退出，抱持姿态报病（D19）；socket 文件 chmod 0660（D8）。

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use futures::{SinkExt, StreamExt};
use miniduck::control::{self, SharedControl};
use miniduck::io::{FakeIo, RobotIo, SimIo};
use miniduck::policy::Policy;
use miniduck::safety::{Safety, SafetyConfig};
use miniduck::{API_VERSION, METHOD_NOT_FOUND, PARSE_ERROR, Request, ServerMessage};
use serde_json::json;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch;
use tokio_util::codec::{Framed, LinesCodec};

const SOCK_PATH: &str = "/tmp/miniduckd.sock";
const DEFAULT_POLICY: &str = "policies/velstand.onnx";
const INVALID_PARAMS: i32 = -32602;

// health 阈值，全部来自原版 robotd-params：stall 25 拍 = 500ms、
// 最低 45Hz = 50Hz 的 90%、连续读错误 10 次。
const HEALTH_MAX_TICK_AGE_MS: u64 = 500;
const HEALTH_MIN_MILLIHZ: u64 = 45_000;
const HEALTH_MAX_CONSEC_READ_ERRORS: u64 = 10;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let boot = Instant::now();

    // 手写参数解析：--sim host:port 选 SimIo，否则 FakeIo。
    let mut sim_addr: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--sim" => {
                sim_addr = Some(args.next().unwrap_or_else(|| {
                    eprintln!("--sim wants host:port");
                    std::process::exit(2);
                }));
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }

    // D19：策略加载失败不再退出——Restart=always 下退出等于 crashloop，
    // 活着报病才能让 updater 回滚（原版 robotd 的理由）。policy=None 时
    // 控制循环永远停在 Held 抱持姿态，health 报 unhealthy。
    let policy_path = std::env::var("MINIDUCK_POLICY").unwrap_or_else(|_| DEFAULT_POLICY.into());
    let (policy, policy_error) = match Policy::load(&policy_path) {
        Ok(p) => (Some(p), None),
        Err(e) => {
            eprintln!("WARNING: failed to load policy {policy_path}: {e} — holding pose, reporting unhealthy");
            (None, Some(format!("policy unavailable: {e}")))
        }
    };
    let policy_error = Arc::new(policy_error);

    let io: Box<dyn RobotIo> = match &sim_addr {
        Some(addr) => {
            eprintln!("miniduckd driving MuJoCo body at {addr}");
            Box::new(SimIo::new(addr.clone()))
        }
        None => {
            // 故障注入开关（容错验收用）：MINIDUCK_FAKE_FAILING_READS=100 让
            // 假总线前 100 次 read 报错，模拟舵机电源未就绪。默认 0。
            let failing_reads = std::env::var("MINIDUCK_FAKE_FAILING_READS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            Box::new(FakeIo::failing_reads(failing_reads))
        }
    };

    let control = control::shared_control();

    // 50 Hz 控制任务。Safety 拥有唯一写句柄：从这里把 io 交出去之后，
    // 本进程再没有第二条碰电机的路径。stats 由循环自己记账。
    let safety = Safety::new(io, SafetyConfig::default());
    let (stats, frame_rx) = control::spawn(safety, policy, control.clone());

    // 上次异常退出残留的 socket 文件会让 bind 报 AddrInUse。先清再绑。
    let _ = std::fs::remove_file(SOCK_PATH);
    let listener = UnixListener::bind(SOCK_PATH)?;
    // D8：socket 文件 0660 就是 robotd 的全部鉴权——能打开这个文件的
    // 用户/组就能开车。对照结论：原版 robotd 没有 SO_PEERCRED，那是
    // configd/updaterd 的模式（M7 的事），这里不超前实现。
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(SOCK_PATH, std::fs::Permissions::from_mode(0o660))?;
    }
    eprintln!("miniduckd listening on {SOCK_PATH}");

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(serve(
            stream,
            stats.clone(),
            frame_rx.clone(),
            control.clone(),
            policy_error.clone(),
            boot,
        ));
    }
}

/// robot.health 的真实判定（D11 收敛）。暖机前（首个 5s 窗口未满，
/// achieved_millihz 还是 0）不按频率判——刚启动的循环频率必然低。
fn health(stats: &control::Stats, policy_error: &Option<String>, boot: Instant) -> serde_json::Value {
    let tick = stats.tick.load(Ordering::Relaxed);
    let consecutive_read_errors = stats.consecutive_read_errors.load(Ordering::Relaxed);
    let achieved_millihz = stats.achieved_millihz.load(Ordering::Relaxed);
    // saturating_sub：tick==0 时 last_tick_millis 为 0，启动 500ms 后
    // 循环若一拍没走，年龄超标、报病——正是要的效果。
    let tick_age_ms = stats
        .started
        .elapsed()
        .as_millis()
        .saturating_sub(stats.last_tick_millis.load(Ordering::Relaxed) as u128)
        as u64;

    let mut reasons: Vec<String> = Vec::new();
    if !stats.policy_ok.load(Ordering::Relaxed) {
        reasons.push(
            policy_error
                .clone()
                .unwrap_or_else(|| "policy unavailable".into()),
        );
    }
    if consecutive_read_errors >= HEALTH_MAX_CONSEC_READ_ERRORS {
        reasons.push(format!("consecutive read errors: {consecutive_read_errors}"));
    }
    if tick_age_ms > HEALTH_MAX_TICK_AGE_MS {
        reasons.push(format!("control loop stalled: last tick {tick_age_ms}ms ago"));
    }
    if achieved_millihz > 0 && achieved_millihz < HEALTH_MIN_MILLIHZ {
        reasons.push(format!(
            "tick rate too low: {:.1}Hz",
            achieved_millihz as f64 / 1000.0
        ));
    }

    json!({
        "healthy": reasons.is_empty(),
        "reason": reasons,
        "tick": tick,
        "uptime_s": boot.elapsed().as_secs(),
        "reads": stats.reads.load(Ordering::Relaxed),
        "writes": stats.writes.load(Ordering::Relaxed),
        "skipped_reads": stats.skipped_reads.load(Ordering::Relaxed),
        "consecutive_read_errors": consecutive_read_errors,
        "achieved_hz": achieved_millihz as f64 / 1000.0,
    })
}

async fn serve(
    stream: UnixStream,
    stats: Arc<control::Stats>,
    mut frame_rx: watch::Receiver<control::FrameSnapshot>,
    control: SharedControl,
    policy_error: Arc<Option<String>>,
    boot: Instant,
) {
    let mut framed = Framed::new(stream, LinesCodec::new());
    let mut subscribed = false;

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
                        health(&stats, &policy_error, boot),
                    ),
                    "robot.state" => {
                        subscribed = true;
                        ServerMessage::ok(req.id, json!({ "subscribed": true }))
                    }
                    // M5 使能开关（收敛 D11/D19 的配套）：无参数，写共享
                    // ControlState.enabled，边沿检测与斜坡在控制循环里做。
                    "robot.enable" => {
                        control.lock().expect("control mutex poisoned").enabled = true;
                        ServerMessage::ok(req.id, json!({ "enabled": true }))
                    }
                    "robot.disable" => {
                        control.lock().expect("control mutex poisoned").enabled = false;
                        ServerMessage::ok(req.id, json!({ "enabled": false }))
                    }
                    // 速度命令进共享 ControlState 并刷新意图时刻（deadman
                    // 依据）。vy 恒 0（侧向未开放）。
                    "robot.drive" => {
                        let vx = req.params.get("vx").and_then(|v| v.as_f64());
                        let vyaw = req.params.get("vyaw").and_then(|v| v.as_f64());
                        match (vx, vyaw) {
                            (Some(vx), Some(vyaw)) if vx.is_finite() && vyaw.is_finite() => {
                                let mut ctl = control.lock().expect("control mutex poisoned");
                                ctl.command.twist = [vx, 0.0, vyaw];
                                ctl.last_intent_at = Some(Instant::now());
                                ServerMessage::ok(
                                    req.id,
                                    json!({ "driving": true, "vx": vx, "vyaw": vyaw }),
                                )
                            }
                            _ => ServerMessage::err(
                                req.id,
                                INVALID_PARAMS,
                                "robot.drive wants finite numbers {vx, vyaw}",
                            ),
                        }
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
            // D14：robot.state 从 1Hz interval 改为帧驱动——控制循环每拍
            // 发一帧快照，这里 changed() 一醒就推，50Hz 直通。latest-wins：
            // 推送慢于 50Hz 时 watch 只保留最新帧，不积压（逐订阅者降频的
            // hz 参数是禁止提前实现项 D25）。
            changed = frame_rx.changed(), if subscribed => {
                if changed.is_err() {
                    break; // 控制任务没了，这条连接也没有存在意义
                }
                let frame = frame_rx.borrow_and_update().clone();
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
                        "fallen": frame.fallen,
                        "enabled": frame.enabled,
                        "gain": frame.gain,
                        "torque": frame.torque,
                    }),
                );
                if framed.send(serde_json::to_string(&note).unwrap()).await.is_err() {
                    break;
                }
            }
        }
    }
}

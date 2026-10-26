use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command as ProcessCommand, Stdio};
use std::time::{Duration, Instant as WallInstant};

use miniduck::control;
use miniduck::io::{RobotIo, SimIo};
use miniduck::obs::Command;
use miniduck::safety::{Safety, SafetyConfig};
use miniduck::scheduler::Scheduler;

#[derive(Clone, Copy)]
enum Motion {
    Forward,
    Turn,
    TurnRate(f64),
    Combined(f64),
}

impl Motion {
    fn name(self) -> String {
        match self {
            Self::Forward => "forward".into(),
            Self::Turn => "turn".into(),
            Self::TurnRate(rate) => format!("turn-{rate}"),
            Self::Combined(rate) => format!("combined-vx0.8-vyaw{rate}"),
        }
    }

    fn twist(self) -> [f64; 3] {
        match self {
            Self::Forward => [0.8, 0.0, 0.0],
            Self::Turn => [0.0, 0.0, 2.0],
            Self::TurnRate(rate) => [0.0, 0.0, rate],
            Self::Combined(rate) => [0.8, 0.0, rate],
        }
    }

    fn seconds(self) -> u64 {
        match self {
            Self::Forward => 10,
            Self::Turn | Self::TurnRate(_) | Self::Combined(_) => 6,
        }
    }
}

struct SimProcess {
    child: Child,
    link: BufReader<TcpStream>,
    log_path: PathBuf,
}

impl SimProcess {
    fn start(motion: Motion, repetition: usize) -> Self {
        let reservation = TcpListener::bind("127.0.0.1:0").expect("reserve sim port");
        let port = reservation.local_addr().unwrap().port();
        drop(reservation);

        let log_path = std::env::temp_dir().join(format!(
            "miniduck-odometry-{}-{}-{repetition}.log",
            std::process::id(),
            motion.name(),
        ));
        let stderr = File::create(&log_path).expect("create sim log");
        let python =
            std::env::var("SIM_PYTHON").unwrap_or_else(|_| "/opt/sim-venv/bin/python".into());
        let mut child = ProcessCommand::new(python)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .args(["sim/duck_body.py", "--port", &port.to_string()])
            .env("MUJOCO_GL", "disable")
            .stdout(Stdio::null())
            .stderr(stderr)
            .spawn()
            .expect("spawn handmade BAM sim");
        let deadline = WallInstant::now() + Duration::from_secs(30);
        let link = loop {
            if let Some(status) = child.try_wait().expect("poll sim") {
                panic!("sim exited with {status}; log {}", log_path.display());
            }
            if WallInstant::now() >= deadline {
                panic!("sim port did not open; log {}", log_path.display());
            }
            match connect(port) {
                Ok(link) => break link,
                Err(_) => std::thread::sleep(Duration::from_millis(100)),
            }
        };
        let mut process = Self {
            child,
            link,
            log_path,
        };

        loop {
            match process.call(serde_json::json!({"op": "body"})) {
                Ok(_) => break,
                Err(error) if WallInstant::now() < deadline => {
                    if let Some(status) = process.child.try_wait().expect("poll sim") {
                        panic!(
                            "sim exited with {status}; log {}",
                            process.log_path.display()
                        );
                    }
                    let _ = error;
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(error) => panic!(
                    "sim did not start: {error}; log {}",
                    process.log_path.display()
                ),
            }
        }
        process
    }

    fn call(&mut self, request: serde_json::Value) -> Result<serde_json::Value, String> {
        let stream = self.link.get_mut();
        stream
            .write_all(format!("{request}\n").as_bytes())
            .map_err(|error| error.to_string())?;
        let mut response = String::new();
        if self
            .link
            .read_line(&mut response)
            .map_err(|error| error.to_string())?
            == 0
        {
            return Err("sim closed connection".into());
        }
        let value: serde_json::Value =
            serde_json::from_str(&response).map_err(|error| error.to_string())?;
        if let Some(error) = value.get("error") {
            return Err(error.to_string());
        }
        Ok(value)
    }

    fn body(&mut self) -> [f64; 3] {
        let response = self
            .call(serde_json::json!({"op": "body"}))
            .expect("body ground truth");
        serde_json::from_value(response["body_pos"].clone()).expect("body_pos array")
    }
}

impl Drop for SimProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn connect(port: u16) -> std::io::Result<BufReader<TcpStream>> {
    let stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_nodelay(true)?;
    stream.set_read_timeout(Some(Duration::from_millis(250)))?;
    stream.set_write_timeout(Some(Duration::from_millis(250)))?;
    Ok(BufReader::new(stream))
}

fn yaw(quat: [f64; 4]) -> f64 {
    let norm = quat.iter().map(|value| value * value).sum::<f64>().sqrt();
    let [w, x, y, z] = quat.map(|value| value / norm);
    (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (y * y + z * z))
}

fn wrapped(angle: f64) -> f64 {
    (angle + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}

#[derive(Debug)]
struct CaseResult {
    name: String,
    true_delta: [f64; 3],
    estimate: [f64; 3],
    endpoint_error: f64,
    max_jump: f64,
    max_yaw_error: f64,
    anchor_switches: u64,
    actual_yaw: f64,
    fallen: bool,
    upright: bool,
}

fn run_case(motion: Motion, repetition: usize, manifest: &Path) -> CaseResult {
    let mut sim = SimProcess::start(motion, repetition);
    let address = sim.link.get_ref().peer_addr().unwrap().to_string();
    let mut recorder = std::env::var_os("ODOMETRY_RECORD_DIR").map(|directory| {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).expect("create recording directory");
        let path = directory.join(format!("{}-{repetition}.jsonl", motion.name()));
        BufWriter::new(File::create(path).expect("create odometry stream recording"))
    });
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("fresh runtime per case");

    let (
        initial_body,
        estimate_delta,
        actual_yaw,
        max_jump,
        max_yaw_error,
        switches,
        fallen,
        upright,
    ) = runtime.block_on(async {
        let scheduler = Scheduler::load(
            &manifest.join("policies/velstand.onnx"),
            &manifest.join("policies"),
        )
        .expect("load current walk scheduler");
        let io: Box<dyn RobotIo> = Box::new(SimIo::new(address));
        let mut safety = Safety::new(io, SafetyConfig::default());
        safety.set_torque(true).expect("sim torque on");
        let shared = control::shared_control();
        shared.lock().unwrap().enabled = true;
        let (_stats, mut frames) =
            control::spawn(safety, Some(scheduler), shared.clone(), true, false);
        let intent = tokio::spawn({
            let shared = shared.clone();
            async move {
                let mut timer = tokio::time::interval(Duration::from_millis(50));
                loop {
                    timer.tick().await;
                    shared.lock().unwrap().last_intent_at = Some(std::time::Instant::now());
                }
            }
        });

        let settle_deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        let mut baseline = None;
        while tokio::time::timeout_at(settle_deadline, frames.changed())
            .await
            .is_ok()
        {
            baseline = Some(frames.borrow_and_update().clone());
        }
        let baseline = baseline.expect("control produced settling frames");
        let initial_body = sim.body();
        {
            let mut state = shared.lock().unwrap();
            state.command = Command {
                twist: motion.twist(),
                ..Command::default()
            };
            state.last_intent_at = Some(std::time::Instant::now());
        }

        let deadline = tokio::time::Instant::now() + Duration::from_secs(motion.seconds());
        let start_yaw = yaw(baseline.sensors.imu.quat);
        let start_position = baseline.odometry.position;
        let mut previous = start_position;
        let mut last = baseline;
        let mut max_jump = 0.0f64;
        let mut max_yaw_error = 0.0f64;
        let mut actual_yaw = 0.0;
        let mut switches = 0;
        while tokio::time::timeout_at(deadline, frames.changed())
            .await
            .is_ok()
        {
            let frame = frames.borrow_and_update().clone();
            if let Some(recorder) = &mut recorder {
                writeln!(
                    recorder,
                    "{}",
                    serde_json::json!({
                        "positions": frame.sensors.positions,
                        "quat": frame.sensors.imu.quat,
                    })
                )
                .expect("record odometry sample");
            }
            let true_yaw = yaw(frame.sensors.imu.quat);
            actual_yaw = wrapped(true_yaw - start_yaw);
            max_yaw_error = max_yaw_error.max(wrapped(frame.odometry.yaw - true_yaw).abs());
            max_jump = max_jump.max(
                (0..3)
                    .map(|axis| (frame.odometry.position[axis] - previous[axis]).powi(2))
                    .sum::<f64>()
                    .sqrt(),
            );
            previous = frame.odometry.position;
            switches = frame.odometry.anchor_switches;
            last = frame;
        }
        if let Some(recorder) = &mut recorder {
            recorder.flush().expect("flush odometry recording");
        }
        intent.abort();
        let upright =
            last.sensors.imu.gravity[2] < -0.85 && last.skill == Some("walk") && !last.fallen;
        let estimate_delta = [
            last.odometry.position[0] - start_position[0],
            last.odometry.position[1] - start_position[1],
            last.odometry.position[2] - start_position[2],
        ];
        (
            initial_body,
            estimate_delta,
            actual_yaw,
            max_jump,
            max_yaw_error,
            switches,
            last.fallen,
            upright,
        )
    });
    drop(runtime);

    let final_body = sim.body();
    let true_delta = [
        final_body[0] - initial_body[0],
        final_body[1] - initial_body[1],
        final_body[2] - initial_body[2],
    ];
    let endpoint_error = ((estimate_delta[0] - true_delta[0]).powi(2)
        + (estimate_delta[1] - true_delta[1]).powi(2))
    .sqrt();
    CaseResult {
        name: format!("{}-{repetition}", motion.name()),
        true_delta,
        estimate: estimate_delta,
        endpoint_error,
        max_jump,
        max_yaw_error,
        anchor_switches: switches,
        actual_yaw,
        fallen,
        upright: upright && final_body[2] > 0.08,
    }
}

#[test]
#[ignore = "spawns six handmade BAM sims; run by accept/m09c-b02.sh"]
fn three_fresh_walks_and_three_fresh_turns_meet_distribution_gates() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let probe = std::env::var("ODOMETRY_LIVE_PROBE").ok();
    let cases = match probe.as_deref() {
        Some("forward-one") => vec![(Motion::Forward, 1)],
        Some("turn-probes") => [
            Motion::TurnRate(1.5),
            Motion::TurnRate(2.0),
            Motion::Combined(0.8),
            Motion::Combined(1.5),
        ]
        .into_iter()
        .flat_map(|motion| [(motion, 1), (motion, 2)])
        .collect(),
        None => vec![
            (Motion::Forward, 1),
            (Motion::Forward, 2),
            (Motion::Forward, 3),
            (Motion::Turn, 1),
            (Motion::Turn, 2),
            (Motion::Turn, 3),
        ],
        Some(other) => panic!("unknown ODOMETRY_LIVE_PROBE={other}"),
    };
    let results: Vec<_> = cases
        .into_iter()
        .map(|(motion, repetition)| run_case(motion, repetition, &manifest))
        .collect();

    for result in &results {
        println!(
            "{} true_delta=[{:.6},{:.6},{:.6}] estimate=[{:.6},{:.6},{:.6}] \
             endpoint_xy={:.6} max_tick_jump={:.6} max_yaw_error={:.9} \
             anchor_switches={} actual_yaw={:.6} fallen={} upright={}",
            result.name,
            result.true_delta[0],
            result.true_delta[1],
            result.true_delta[2],
            result.estimate[0],
            result.estimate[1],
            result.estimate[2],
            result.endpoint_error,
            result.max_jump,
            result.max_yaw_error,
            result.anchor_switches,
            result.actual_yaw,
            result.fallen,
            result.upright,
        );
    }
    if probe.is_some() {
        return;
    }
    let mut errors: Vec<_> = results.iter().map(|result| result.endpoint_error).collect();
    errors.sort_by(f64::total_cmp);
    let median = (errors[2] + errors[3]) / 2.0;
    let max_endpoint = errors[5];
    let max_jump = results
        .iter()
        .map(|result| result.max_jump)
        .fold(0.0, f64::max);
    let max_yaw_error = results
        .iter()
        .map(|result| result.max_yaw_error)
        .fold(0.0, f64::max);
    println!(
        "summary endpoint_samples={errors:?} median={median:.6} max_endpoint={max_endpoint:.6} \
         max_tick_jump={max_jump:.6} max_yaw_error={max_yaw_error:.9}"
    );

    for result in &results {
        if result.name.starts_with("forward") {
            assert!(
                result.true_delta[0] >= 2.0,
                "{}: true dx {}",
                result.name,
                result.true_delta[0]
            );
            assert!(
                result.upright && !result.fallen,
                "{}: not upright/fallen",
                result.name
            );
        } else {
            assert!(
                result.actual_yaw.abs() >= 0.5,
                "{}: actual yaw {}",
                result.name,
                result.actual_yaw
            );
        }
        assert!(
            result.endpoint_error <= 0.12,
            "{}: endpoint {}",
            result.name,
            result.endpoint_error
        );
        assert!(
            result.max_jump < 0.02,
            "{}: tick jump {}",
            result.name,
            result.max_jump
        );
        assert!(
            result.max_yaw_error < 1e-6,
            "{}: yaw error {}",
            result.name,
            result.max_yaw_error
        );
    }
    assert!(median <= 0.07, "median endpoint {median}");
}

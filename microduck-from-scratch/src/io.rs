//! 舵机总线抽象：RobotIo trait + FakeIo + SimIo。
//!
//! trait 存在的理由（原版文档 §2.4）：真实总线和假总线共用一个接口，
//! 测试和笔记本开发全程跑 FakeIo，`cargo test` 不需要硬件。M5 收敛 D12
//! 的安全层部分：补 set_gain/set_torque/imu_ready（跌倒卸力和"滤波未收敛
//! 不许投票"都要用到）。reboot/slow_sensors/imu_stale 等仍是 M8 的事
//! （偏差簿 D12 残余），现在写上只是没人调用的死代码。

use crate::model::NUM_JOINTS;
use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// 总线错误。M1 只需要"读/写失败了"这一个事实（控制循环据此跳过本拍），
/// 不分类——分类等到有调用点真的按类别分支时再加。
#[derive(Debug, Clone)]
pub struct IoError(pub String);

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for IoError {}

pub type Result<T> = std::result::Result<T, IoError>;

/// IMU 数据。M1 只当占位：FakeIo 恒定返回直立姿态，不做滤波。
#[derive(Debug, Clone, Copy)]
pub struct ImuData {
    pub gyro: [f64; 3],
    pub gravity: [f64; 3],
    /// 四元数，wxyz 序。
    pub quat: [f64; 4],
}

impl Default for ImuData {
    /// 直立：重力朝下（机体系 z 向下为正），单位四元数。
    fn default() -> Self {
        ImuData {
            gyro: [0.0; 3],
            gravity: [0.0, 0.0, -1.0],
            quat: [1.0, 0.0, 0.0, 0.0],
        }
    }
}

/// 一次总线事务读到的全部传感数据。关节和 IMU 放同一个结构体，
/// 因为硬件上它们就是一次 sync_read 一起回来的；拆开会让"这一帧的
/// 姿态和这一帧的关节角是不是同一时刻"变成调用方的责任。
#[derive(Debug, Clone, Default)]
pub struct Sensors {
    pub positions: [f64; NUM_JOINTS],
    pub velocities: [f64; NUM_JOINTS],
    pub currents_ma: [f64; NUM_JOINTS],
    pub imu: ImuData,
}

/// 纯位置控制的目标。
#[derive(Debug, Clone)]
pub struct JointTargets {
    pub positions: [f64; NUM_JOINTS],
}

pub trait RobotIo: Send {
    fn read(&mut self) -> Result<Sensors>;
    fn write(&mut self, targets: &JointTargets) -> Result<()>;
    /// 整组舵机的位置环 kp（0..=200，XL330 增益寄存器口径）。
    fn set_gain(&mut self, kp: u16) -> Result<()>;
    /// 舵机出力开关。torque off 时 goal 寄存器照写只是不出力。
    fn set_torque(&mut self, on: bool) -> Result<()>;
    /// 姿态滤波是否已收敛。默认 true：FakeIo/SimIo 的 IMU 没有滤波器，
    /// 第一帧就是收敛值；真总线的 SFLP 滤波器需要几秒样本才有意义。
    fn imu_ready(&self) -> bool {
        true
    }
}

/// Box 转发：main 按 --sim 在 FakeIo/SimIo 间二选一，需要 trait object。
impl RobotIo for Box<dyn RobotIo> {
    fn read(&mut self) -> Result<Sensors> {
        (**self).read()
    }
    fn write(&mut self, targets: &JointTargets) -> Result<()> {
        (**self).write(targets)
    }
    fn set_gain(&mut self, kp: u16) -> Result<()> {
        (**self).set_gain(kp)
    }
    fn set_torque(&mut self, on: bool) -> Result<()> {
        (**self).set_torque(on)
    }
    fn imu_ready(&self) -> bool {
        (**self).imu_ready()
    }
}

/// 假舵机总线：完美跟踪——write 之后 read 原样返回写入的位置，
/// 相当于"舵机瞬间完美跟随"。故意不做一阶惯性模型：M1 没有任何
/// 逻辑依赖跟踪延迟，加了只会让测试多一个要调的参数。
///
/// `failing_reads(n)` 模拟舵机电源未就绪：前 n 次 read 报错，之后正常。
pub struct FakeIo {
    position: [f64; NUM_JOINTS],
    failing_reads_left: u64,
    /// 计数器公开，测试直接断言；控制循环自己的计数在 Stats 里。
    pub reads: u64,
    pub writes: u64,
    /// M5：记录最近一次 set_gain/set_torque 及 gain 写总线次数，
    /// 测试据此断言 Safety 的增益缓存（值不变不重写）。
    pub gain: Option<u16>,
    pub torque: Option<bool>,
    pub gain_writes: u64,
    /// 假总线没有姿态滤波器，默认即收敛；测试可翻成 false 模拟
    /// "SFLP 未收敛"（原版血泪回归的场景）。
    pub imu_ready: bool,
}

impl FakeIo {
    /// 全零姿态（"躺平"）——M1 控制循环的起点。
    pub fn new() -> Self {
        Self::failing_reads(0)
    }

    pub fn failing_reads(n: u64) -> Self {
        FakeIo {
            position: [0.0; NUM_JOINTS],
            failing_reads_left: n,
            reads: 0,
            writes: 0,
            gain: None,
            torque: None,
            gain_writes: 0,
            imu_ready: true,
        }
    }
}

impl Default for FakeIo {
    fn default() -> Self {
        Self::new()
    }
}

// ---- SimIo：MuJoCo 仿真体，TCP + NDJSON（M4）----
//
// 第三个 RobotIo 实现（FakeIo、真实总线之外）。协议自定、形状对齐原版
// duck-control/src/sim.rs（偏差 D4，M8 对齐帧格式）：op 标签帧，hello 握手
// 带 protocol 版本号与关节数，read/write 一问一答。
//
// 断线语义：任何错误都丢连接、返回 Err，由控制循环下一拍重连——控制循环
// 本身就是重试定时器，不需要后台重连线程。

/// 仿真侧协议版本；与 sim/duck_body.py 的 PROTOCOL 一致，不一致就报错（报两个号）。
pub const SIM_PROTOCOL: u32 = 1;

/// 单次请求等应答的上限。比一拍（20ms）宽得多——接触密集的步进可以偶尔迟到；
/// 又短到不会让卡死的仿真体拖死控制循环。
const SIM_TIMEOUT: Duration = Duration::from_millis(200);

pub struct SimIo {
    addr: String,
    link: Option<(TcpStream, BufReader<TcpStream>)>,
}

impl SimIo {
    /// 只记地址，不连接：仿真体没起时守护进程也必须能起来，与总线没电同理。
    pub fn new(addr: impl Into<String>) -> Self {
        SimIo {
            addr: addr.into(),
            link: None,
        }
    }

    fn connect(&mut self) -> Result<&mut (TcpStream, BufReader<TcpStream>)> {
        if self.link.is_none() {
            let address = self
                .addr
                .to_socket_addrs()
                .map_err(|e| IoError(format!("resolve {}: {e}", self.addr)))?
                .next()
                .ok_or_else(|| IoError(format!("{} resolved to no address", self.addr)))?;
            let stream = TcpStream::connect_timeout(&address, SIM_TIMEOUT)
                .map_err(|e| IoError(format!("connect {}: {e}", self.addr)))?;
            // Nagle 会把小包攒最多 ~40ms——两拍——每拍都变成超时事故，必须关。
            let _ = stream.set_nodelay(true);
            let _ = stream.set_read_timeout(Some(SIM_TIMEOUT));
            let _ = stream.set_write_timeout(Some(SIM_TIMEOUT));
            let reader = BufReader::new(
                stream
                    .try_clone()
                    .map_err(|e| IoError(format!("clone stream: {e}")))?,
            );
            self.link = Some((stream, reader));

            let hello: serde_json::Value = self.call(&serde_json::json!({
                "op": "hello",
                "protocol": SIM_PROTOCOL,
                "joints": NUM_JOINTS,
            }))?;
            let protocol = hello.get("protocol").and_then(|v| v.as_u64()).unwrap_or(0);
            if protocol != SIM_PROTOCOL as u64 {
                self.link = None;
                return Err(IoError(format!(
                    "simulator speaks protocol {protocol}, daemon speaks {SIM_PROTOCOL}"
                )));
            }
        }
        Ok(self.link.as_mut().expect("just connected"))
    }

    /// 一问一答；任何环节失败都丢连接（行协议无法重新同步帧边界，只能重来）。
    fn call(&mut self, request: &serde_json::Value) -> Result<serde_json::Value> {
        let result = (|| {
            let (writer, reader) = self.connect()?;
            let mut line = serde_json::to_string(request)
                .map_err(|e| IoError(e.to_string()))?;
            line.push('\n');
            writer
                .write_all(line.as_bytes())
                .map_err(|e| IoError(format!("send: {e}")))?;
            let mut answer = String::new();
            let n = reader
                .read_line(&mut answer)
                .map_err(|e| IoError(format!("recv: {e}")))?;
            if n == 0 {
                return Err(IoError("simulator closed the connection".into()));
            }
            let value: serde_json::Value =
                serde_json::from_str(&answer).map_err(|e| IoError(format!("bad frame: {e}")))?;
            if let Some(err) = value.get("error").and_then(|e| e.as_str()) {
                return Err(IoError(format!("simulator refused: {err}")));
            }
            Ok(value)
        })();
        if result.is_err() {
            self.link = None;
        }
        result
    }
}

impl RobotIo for SimIo {
    fn read(&mut self) -> Result<Sensors> {
        let frame = self.call(&serde_json::json!({"op": "read"}))?;
        let arr = |key: &str, len: usize| -> Result<Vec<f64>> {
            let v: Vec<f64> = serde_json::from_value(
                frame
                    .get(key)
                    .cloned()
                    .ok_or_else(|| IoError(format!("frame missing {key}")))?,
            )
            .map_err(|e| IoError(format!("frame {key}: {e}")))?;
            if v.len() != len {
                return Err(IoError(format!("frame {key}: {} != {len} numbers", v.len())));
            }
            Ok(v)
        };
        let mut sensors = Sensors::default();
        sensors.positions.copy_from_slice(&arr("positions", NUM_JOINTS)?);
        sensors.velocities.copy_from_slice(&arr("velocities", NUM_JOINTS)?);
        let imu = frame
            .get("imu")
            .ok_or_else(|| IoError("frame missing imu".into()))?;
        // 定长数组一律严格校验长度：长度不符是协议漂移，必须报错而不是
        // 静默补零/截断（D22 收敛：quat 此前 resize(4,0) 宽容解析，
        // copy3 此前长度不符会 panic——两处都与 positions 的严格校验不一致）。
        let copy3 = |key: &str, dst: &mut [f64; 3]| -> Result<()> {
            let v: Vec<f64> = serde_json::from_value(
                imu.get(key)
                    .cloned()
                    .ok_or_else(|| IoError(format!("frame imu missing {key}")))?,
            )
            .map_err(|e| IoError(format!("frame imu {key}: {e}")))?;
            if v.len() != 3 {
                return Err(IoError(format!("frame imu {key}: {} != 3 numbers", v.len())));
            }
            dst.copy_from_slice(&v);
            Ok(())
        };
        copy3("gyro", &mut sensors.imu.gyro)?;
        copy3("gravity", &mut sensors.imu.gravity)?;
        let quat: Vec<f64> = serde_json::from_value(
            imu.get("quat")
                .cloned()
                .ok_or_else(|| IoError("frame imu missing quat".into()))?,
        )
        .map_err(|e| IoError(format!("frame imu quat: {e}")))?;
        if quat.len() != 4 {
            return Err(IoError(format!(
                "frame imu quat: {} != 4 numbers",
                quat.len()
            )));
        }
        sensors.imu.quat.copy_from_slice(&quat);
        Ok(sensors)
    }

    fn write(&mut self, targets: &JointTargets) -> Result<()> {
        self.call(&serde_json::json!({
            "op": "write",
            "targets": targets.positions,
        }))?;
        Ok(())
    }

    fn set_gain(&mut self, kp: u16) -> Result<()> {
        self.call(&serde_json::json!({
            "op": "set_gain",
            "gain": kp,
        }))?;
        Ok(())
    }

    fn set_torque(&mut self, on: bool) -> Result<()> {
        self.call(&serde_json::json!({
            "op": "set_torque",
            "on": on,
        }))?;
        Ok(())
    }
    // imu_ready 用默认 true：仿真 IMU 是直接解算的，没有滤波收敛过程。
}

impl RobotIo for FakeIo {
    fn read(&mut self) -> Result<Sensors> {
        self.reads += 1;
        if self.failing_reads_left > 0 {
            self.failing_reads_left -= 1;
            return Err(IoError("servo bus not powered yet".into()));
        }
        Ok(Sensors {
            positions: self.position,
            ..Sensors::default()
        })
    }

    fn write(&mut self, targets: &JointTargets) -> Result<()> {
        self.writes += 1;
        self.position = targets.positions;
        Ok(())
    }

    fn set_gain(&mut self, kp: u16) -> Result<()> {
        self.gain_writes += 1;
        self.gain = Some(kp);
        Ok(())
    }

    fn set_torque(&mut self, on: bool) -> Result<()> {
        self.torque = Some(on);
        Ok(())
    }

    fn imu_ready(&self) -> bool {
        self.imu_ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_io_tracks_writes_perfectly() {
        let mut io = FakeIo::new();
        let mut positions = [0.0; NUM_JOINTS];
        positions[2] = -0.4579;
        positions[12] = 0.4579;
        io.write(&JointTargets { positions }).unwrap();
        let sensors = io.read().unwrap();
        assert_eq!(sensors.positions, positions);
        assert_eq!(io.reads, 1);
        assert_eq!(io.writes, 1);
    }

    #[test]
    fn fake_io_failing_reads_recover_after_n() {
        let mut io = FakeIo::failing_reads(3);
        for _ in 0..3 {
            assert!(io.read().is_err());
        }
        assert!(io.read().is_ok());
        assert_eq!(io.reads, 4);
    }

    /// 脚本化假仿真体：每条连接按剧本应答，一行请求换一行应答。
    fn scripted_sim(
        scripts: Vec<Vec<&'static str>>,
    ) -> (String, std::thread::JoinHandle<Vec<String>>) {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap().to_string();
        let handle = std::thread::spawn(move || {
            let mut heard = Vec::new();
            for script in scripts {
                let (stream, _) = listener.accept().unwrap();
                let mut out = stream.try_clone().unwrap();
                let mut lines = BufReader::new(stream);
                for reply in script {
                    let mut line = String::new();
                    if lines.read_line(&mut line).unwrap() == 0 {
                        break;
                    }
                    heard.push(line.trim().to_string());
                    out.write_all(reply.as_bytes()).unwrap();
                    out.write_all(b"\n").unwrap();
                }
            }
            heard
        });
        (addr, handle)
    }

    const SIM_HELLO: &str = r#"{"protocol":1,"joints":15}"#;
    const SIM_SENSORS: &str = concat!(
        r#"{"positions":[0.1,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"#,
        r#""velocities":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0.5],"#,
        r#""imu":{"gyro":[0,0,0],"gravity":[0,0,-1],"quat":[1,0,0,0]},"#,
        r#""body_pos":[0,0,0.1],"sim_time":0.02}"#
    );

    #[test]
    fn sim_io_read_carries_sensors() {
        let (addr, _sim) = scripted_sim(vec![vec![SIM_HELLO, SIM_SENSORS]]);
        let mut io = SimIo::new(addr);
        let sensors = io.read().unwrap();
        assert_eq!(sensors.positions[0], 0.1);
        assert_eq!(sensors.velocities[NUM_JOINTS - 1], 0.5);
        assert_eq!(sensors.imu.gravity, [0.0, 0.0, -1.0]);
    }

    #[test]
    fn sim_io_reconnects_after_disconnect() {
        // 仿真体改模型会重启：第一次连接中途挂断，下一拍必须无人工干预重连。
        let (addr, sim) = scripted_sim(vec![vec![SIM_HELLO], vec![SIM_HELLO, SIM_SENSORS]]);
        let mut io = SimIo::new(addr);
        assert!(io.read().is_err());
        let sensors = io.read().unwrap();
        assert_eq!(sensors.positions[0], 0.1);
        let heard = sim.join().unwrap();
        assert_eq!(heard.iter().filter(|l| l.contains("hello")).count(), 2);
    }

    #[test]
    fn sim_io_refusal_is_an_error() {
        let (addr, _sim) = scripted_sim(vec![vec![
            SIM_HELLO,
            r#"{"error":"targets must be 15 numbers"}"#,
        ]]);
        let mut io = SimIo::new(addr);
        let err = io
            .write(&JointTargets {
                positions: [0.0; NUM_JOINTS],
            })
            .unwrap_err();
        assert!(err.to_string().contains("15 numbers"), "{err}");
    }

    #[test]
    fn sim_io_quat_must_have_exactly_four_numbers() {
        // D22：quat 与 positions/gyro 同一严格标准，长度≠4 即 Err。
        let bad = concat!(
            r#"{"positions":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"#,
            r#""velocities":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"#,
            r#""imu":{"gyro":[0,0,0],"gravity":[0,0,-1],"quat":[1,0,0]}}"#,
        );
        let (addr, _sim) = scripted_sim(vec![vec![SIM_HELLO, bad]]);
        let mut io = SimIo::new(addr);
        let err = io.read().unwrap_err();
        assert!(err.to_string().contains("quat"), "{err}");

        // 长度=4 正常通过（对照组，防止校验把合法帧也拒了）。
        let (addr, _sim) = scripted_sim(vec![vec![SIM_HELLO, SIM_SENSORS]]);
        let mut io = SimIo::new(addr);
        assert_eq!(io.read().unwrap().imu.quat, [1.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn sim_io_set_gain_and_torque_frames() {
        let (addr, sim) = scripted_sim(vec![vec![SIM_HELLO, "{}", "{}"]]);
        let mut io = SimIo::new(addr);
        io.set_gain(50).unwrap();
        io.set_torque(false).unwrap();
        let heard = sim.join().unwrap();
        assert!(heard[1].contains(r#""op":"set_gain""#), "{}", heard[1]);
        assert!(heard[1].contains(r#""gain":50"#), "{}", heard[1]);
        assert!(heard[2].contains(r#""op":"set_torque""#), "{}", heard[2]);
        assert!(heard[2].contains(r#""on":false"#), "{}", heard[2]);
    }

    #[test]
    fn fake_io_remembers_gain_and_torque() {
        let mut io = FakeIo::new();
        assert_eq!(io.gain, None);
        assert_eq!(io.torque, None);
        io.set_gain(200).unwrap();
        io.set_torque(true).unwrap();
        assert_eq!(io.gain, Some(200));
        assert_eq!(io.torque, Some(true));
        assert_eq!(io.gain_writes, 1);
        assert!(io.imu_ready());
    }
}

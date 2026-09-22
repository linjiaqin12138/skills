//! 舵机总线抽象：RobotIo trait + FakeIo。
//!
//! trait 存在的理由（原版文档 §2.4）：真实总线和假总线共用一个接口，
//! 测试和笔记本开发全程跑 FakeIo，`cargo test` 不需要硬件。M1 只定义
//! read/write 两个方法——set_gain/set_torque/reboot/slow_sensors 是
//! M5 的事，现在写上只是没人调用的死代码（本项目规则：当前里程碑够用即可）。

use crate::model::NUM_JOINTS;
use std::fmt;

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
        }
    }
}

impl Default for FakeIo {
    fn default() -> Self {
        Self::new()
    }
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
}

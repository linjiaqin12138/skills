//! 策略观测向量：61 维扁平 f32。
//!
//! 下标契约错了不会 panic——策略照样跑，机器人以"像调参问题"的方式摔倒。
//! 所以布局用命名偏移常量钉死，并用单测覆盖每个块边界；本模块刻意做成
//! 纯函数，让错误在 `cargo test` 里暴露，而不是在真机上表现为走路不稳。

use crate::io::ImuData;
use crate::model::{MOUTH_INDEX, NUM_JOINTS};

pub const OBS_LEN: usize = 61;
pub const ACTION_LEN: usize = 14;
pub const OBS_JOINTS: usize = NUM_JOINTS - 1;
pub const COMMAND_LEN: usize = 13;

/// 块起点：写成常量而不是每次手算，是为了让"改一处偏移"立刻变成编译期
/// 可见的符号漂移，而不是在 fill 循环里静默写错相邻块。
const OFF_GYRO: usize = 0;
const OFF_GRAVITY: usize = 3;
const OFF_POS: usize = 6;
const OFF_VEL: usize = 20;
const OFF_LAST_ACTION: usize = 34;
const OFF_TWIST: usize = 48;
const OFF_HEAD: usize = 51;
const OFF_BODY_XY: usize = 55;
const OFF_BODY_Z: usize = 57;
const OFF_BODY_ROLL: usize = 58;
const OFF_BODY_PITCH: usize = 59;
const OFF_BODY_YAW: usize = 60;

/// 客户端意图，物理单位、躯干系。转成命令块只发生在 `Observation::build`，
/// 避免别处各自拼 13 维时把 body 顺序或 unbound 轴写错。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Command {
    pub twist: [f64; 3],
    pub head: [f64; 4],
    pub body: BodyPose,
}

/// 站姿偏移：训练里只绑了 z/roll/pitch。x/y/yaw 在向量里恒写 0（见 build），
/// 这里不暴露它们，免得调用方以为传进去会生效。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BodyPose {
    pub z: f64,
    pub roll: f64,
    pub pitch: f64,
}

/// 策略槽 → 总线关节。读写共用这一处：读观测跳过嘴、写动作也跳过嘴，
/// 两套映射会让嘴之后的关节全体错开一格且毫无报错。
#[inline]
const fn joint_of(slot: usize) -> usize {
    if slot < MOUTH_INDEX {
        slot
    } else {
        slot + 1
    }
}

fn policy_joints(values: &[f64; NUM_JOINTS]) -> [f64; OBS_JOINTS] {
    std::array::from_fn(|slot| values[joint_of(slot)])
}

#[derive(Debug, Clone, Copy)]
pub struct Observation {
    data: [f32; OBS_LEN],
}

impl Observation {
    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    /// 组装观测。位置进向量前减 home——绝对角是训练分布里没见过的常数偏移，
    /// 策略会把整条腿当成"已经偏了很远"来纠正。
    pub fn build(
        imu: &ImuData,
        joint_positions: &[f64; NUM_JOINTS],
        joint_velocities: &[f64; NUM_JOINTS],
        home_pose: &[f64; NUM_JOINTS],
        last_action: &[f32; ACTION_LEN],
        command: &Command,
    ) -> Self {
        let mut data = [0.0f32; OBS_LEN];

        for i in 0..3 {
            data[OFF_GYRO + i] = imu.gyro[i] as f32;
            data[OFF_GRAVITY + i] = imu.gravity[i] as f32;
        }

        let angles = policy_joints(joint_positions);
        let home = policy_joints(home_pose);
        for i in 0..OBS_JOINTS {
            data[OFF_POS + i] = (angles[i] - home[i]) as f32;
        }

        let vels = policy_joints(joint_velocities);
        for i in 0..OBS_JOINTS {
            data[OFF_VEL + i] = vels[i] as f32;
        }

        // last_action 已是策略原始 f32，直接拷——再 as 一次会引入无意义的舍入路径。
        data[OFF_LAST_ACTION..OFF_LAST_ACTION + ACTION_LEN].copy_from_slice(last_action);

        for i in 0..3 {
            data[OFF_TWIST + i] = command.twist[i] as f32;
        }
        for i in 0..4 {
            data[OFF_HEAD + i] = command.head[i] as f32;
        }

        // body x/y/yaw：训练未绑定。调用方传什么都写 0，否则策略看到从未见过的通道。
        data[OFF_BODY_XY] = 0.0;
        data[OFF_BODY_XY + 1] = 0.0;
        data[OFF_BODY_Z] = command.body.z as f32;
        data[OFF_BODY_ROLL] = command.body.roll as f32;
        data[OFF_BODY_PITCH] = command.body.pitch as f32;
        data[OFF_BODY_YAW] = 0.0;

        Self { data }
    }

    /// 14 维动作铺回 15 关节。嘴不在策略输出里，必须保持 0；若写成顺序填满，
    /// 右腿全体会吃到左腿多出来的那一维。
    pub fn scatter_action(action: &[f32; ACTION_LEN]) -> [f64; NUM_JOINTS] {
        let mut out = [0.0f64; NUM_JOINTS];
        for (slot, value) in action.iter().enumerate() {
            out[joint_of(slot)] = *value as f64;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DEFAULT_POSITION;

    fn imu() -> ImuData {
        ImuData {
            gyro: [1.0, 2.0, 3.0],
            gravity: [4.0, 5.0, 6.0],
            quat: [1.0, 0.0, 0.0, 0.0],
        }
    }

    fn command() -> Command {
        Command {
            twist: [0.1, 0.2, 0.3],
            head: [0.4, 0.5, 0.6, 0.7],
            body: BodyPose {
                z: 0.8,
                roll: 0.9,
                pitch: 1.0,
            },
        }
    }

    fn build_with(positions: [f64; NUM_JOINTS], last_action: [f32; ACTION_LEN]) -> Observation {
        Observation::build(
            &imu(),
            &positions,
            &[0.0; NUM_JOINTS],
            &DEFAULT_POSITION,
            &last_action,
            &command(),
        )
    }

    #[test]
    fn layout_widths_sum_to_obs_len() {
        assert_eq!(3 + 3 + OBS_JOINTS * 3 + COMMAND_LEN, OBS_LEN);
        assert_eq!(OBS_JOINTS, ACTION_LEN);
        assert_eq!(ACTION_LEN, 14);
    }

    #[test]
    fn every_block_lands_at_its_documented_offset() {
        let mut positions = DEFAULT_POSITION;
        positions[0] += 0.25;
        let mut last_action = [0.0f32; ACTION_LEN];
        last_action[0] = -0.5;
        last_action[ACTION_LEN - 1] = 0.75;

        let obs = build_with(positions, last_action);
        let d = obs.as_slice();

        assert_eq!(&d[0..3], &[1.0, 2.0, 3.0]);
        assert_eq!(&d[3..6], &[4.0, 5.0, 6.0]);
        assert!((d[6] - 0.25).abs() < 1e-5);
        for (i, &v) in d[7..20].iter().enumerate() {
            assert!(v.abs() < 1e-5, "pos slot {} should be ~0, got {v}", i + 1);
        }
        assert_eq!(d[20], 0.0);
        assert_eq!(d[34], -0.5);
        assert_eq!(d[47], 0.75);
        assert_eq!(&d[48..51], &[0.1, 0.2, 0.3]);
        assert_eq!(&d[51..55], &[0.4, 0.5, 0.6, 0.7]);
        assert_eq!(d[55], 0.0);
        assert_eq!(d[56], 0.0);
        assert!((d[57] - 0.8).abs() < 1e-5);
        assert!((d[58] - 0.9).abs() < 1e-5);
        assert!((d[59] - 1.0).abs() < 1e-5);
        assert_eq!(d[60], 0.0);
    }

    #[test]
    fn at_home_position_block_is_near_zero() {
        let obs = build_with(DEFAULT_POSITION, [0.0; ACTION_LEN]);
        for (i, &v) in obs.as_slice()[6..20].iter().enumerate() {
            assert!(v.abs() < 1e-5, "joint {i} at home should be ~0, got {v}");
        }
    }

    #[test]
    fn mouth_is_excluded_from_observation() {
        let mut positions = DEFAULT_POSITION;
        positions[MOUTH_INDEX] += 1.0;
        let obs = build_with(positions, [0.0; ACTION_LEN]);
        for (i, &v) in obs.as_slice()[6..20].iter().enumerate() {
            assert!(
                v.abs() < 1e-5,
                "moving mouth changed obs pos slot {i} to {v}"
            );
        }
    }

    #[test]
    fn scatter_action_skips_mouth() {
        let mut action = [0.0f32; ACTION_LEN];
        for (i, a) in action.iter_mut().enumerate() {
            *a = (i + 1) as f32;
        }
        let scattered = Observation::scatter_action(&action);
        assert_eq!(scattered[MOUTH_INDEX], 0.0);
        assert_eq!(scattered[0], 1.0);
        assert_eq!(scattered[8], 9.0);
        assert_eq!(scattered[10], 10.0);
        assert_eq!(scattered[14], 14.0);
    }
}

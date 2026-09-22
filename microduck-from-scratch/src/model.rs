//! 身体模型：15 关节常量表。
//!
//! 关节顺序就是协议里 positions/targets 数组的位置序，所以这张表是
//! 全项目唯一允许的来源——原版把 JOINT_NAMES 放在 IPC 协议 crate 里、
//! 再用 const 断言锁死控制侧和协议侧不漂移；我们目前只有一份表，
//! 等协议类型多到值得拆模块时再上同样的锁。

/// 关节数。数组长度都以此为准，测试里再断言一遍——表是手抄的，
/// 抄错一位不会编译错，只会把左腿的命令写到右腿上。
pub const NUM_JOINTS: usize = 15;

/// 关节名，索引即协议位置序。
pub const JOINT_NAMES: [&str; NUM_JOINTS] = [
    "left_hip_yaw",   // 0
    "left_hip_roll",  // 1
    "left_hip_pitch", // 2
    "left_knee",      // 3
    "left_ankle",     // 4
    "neck_pitch",     // 5
    "head_pitch",     // 6
    "head_yaw",       // 7
    "head_roll",      // 8
    "mouth",          // 9
    "right_hip_yaw",  // 10
    "right_hip_roll", // 11
    "right_hip_pitch",// 12
    "right_knee",     // 13
    "right_ankle",    // 14
];

/// 嘴不参与运动策略，M1 也不动它。单列出来是因为 14/15 的歧义
/// （"策略维数 14"vs"总线维数 15"）在原版文档里是专门解释过的坑。
pub const MOUTH_INDEX: usize = 9;

/// home 姿态（弧度）：直立待命位。左右腿镜像（roll/pitch/knee/ankle
/// 等大反号），躯干前倾让质心落在踝关节轴正上方——双足静止站立的
/// 稳定性就押在这几百度上，所以它是常量而不是配置。
pub const DEFAULT_POSITION: [f64; NUM_JOINTS] = [
    0.0, -0.0873, -0.4579, -0.0049, 0.4530, // 左腿
    0.3491, 0.3491, 0.0, 0.0, 0.0,          // 颈/头/嘴
    0.0, 0.0873, 0.4579, 0.0049, -0.4530,   // 右腿（镜像）
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_lengths_are_consistent() {
        assert_eq!(JOINT_NAMES.len(), NUM_JOINTS);
        assert_eq!(DEFAULT_POSITION.len(), NUM_JOINTS);
    }

    #[test]
    fn home_pose_legs_are_mirrored() {
        let p = &DEFAULT_POSITION;
        // 左腿索引 0..5，右腿 10..15，镜像关节等大反号。
        for (l, r) in [(0, 10), (1, 11), (2, 12), (3, 13), (4, 14)] {
            assert!(
                (p[l] + p[r]).abs() < 1e-9,
                "{} 与 {} 不镜像：{} vs {}",
                JOINT_NAMES[l],
                JOINT_NAMES[r],
                p[l],
                p[r]
            );
        }
    }
}

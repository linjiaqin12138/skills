//! Alpha 双足局部里程计：固定腿链 FK、IMU 姿态与 4×4 鞋底锚点。
//! 只覆盖控制循环所需的两个 foot site；通用模型与其他运动学查询属于 D55。

use crate::model::NUM_JOINTS;

const LEFT: usize = 0;
const RIGHT: usize = 1;
const SWITCH_WORLD_Z: f64 = 0.010;
const SWITCH_CONFIRM_TICKS: u32 = 2;

const ALPHA16: [[[f64; 3]; 16]; 2] = [
    [
        [-0.018300, -0.019510, 0.005894],
        [-0.001500, -0.019659, 0.001516],
        [0.015500, -0.019659, 0.002082],
        [0.031699, -0.019061, 0.005531],
        [-0.018500, -0.006960, 0.002891],
        [-0.001500, -0.006960, -0.000607],
        [0.015500, -0.006960, -0.000573],
        [0.032500, -0.006960, 0.002863],
        [-0.018500, 0.005740, 0.003709],
        [-0.001500, 0.005740, 0.000500],
        [0.015500, 0.005740, 0.000474],
        [0.032500, 0.005740, 0.003683],
        [-0.018300, 0.018290, 0.007998],
        [-0.001500, 0.018439, 0.003859],
        [0.015500, 0.018439, 0.004318],
        [0.031699, 0.017841, 0.007574],
    ],
    [
        [-0.018300, -0.018290, 0.007998],
        [-0.001500, -0.018439, 0.003859],
        [0.015500, -0.018439, 0.004320],
        [0.031699, -0.017841, 0.007574],
        [-0.018500, -0.005740, 0.003712],
        [-0.001500, -0.005740, 0.000500],
        [0.015500, -0.005740, 0.000474],
        [0.032500, -0.005740, 0.003681],
        [-0.018500, 0.006960, 0.002893],
        [-0.001500, 0.006960, -0.000607],
        [0.015500, 0.006960, -0.000573],
        [0.032500, 0.006960, 0.002863],
        [-0.018300, 0.019510, 0.005894],
        [-0.001500, 0.019659, 0.001516],
        [0.015500, 0.019659, 0.002080],
        [0.031699, 0.019061, 0.005531],
    ],
];

#[derive(Debug, Clone, Copy, PartialEq)]
struct Quat {
    w: f64,
    x: f64,
    y: f64,
    z: f64,
}

impl Quat {
    const IDENTITY: Self = Self {
        w: 1.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    const fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        Self { w, x, y, z }
    }

    fn from_wxyz(wxyz: [f64; 4]) -> Self {
        Self::new(wxyz[0], wxyz[1], wxyz[2], wxyz[3])
    }

    fn normalized(self) -> Self {
        let norm = (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if norm < 1e-12 {
            return Self::IDENTITY;
        }
        Self::new(self.w / norm, self.x / norm, self.y / norm, self.z / norm)
    }

    fn about_z(angle: f64) -> Self {
        let (sin, cos) = (angle / 2.0).sin_cos();
        Self::new(cos, 0.0, 0.0, sin)
    }

    fn multiply(self, rhs: Self) -> Self {
        Self::new(
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        )
    }

    fn rotate(self, vector: [f64; 3]) -> [f64; 3] {
        let [x, y, z] = vector;
        let tx = 2.0 * (self.y * z - self.z * y);
        let ty = 2.0 * (self.z * x - self.x * z);
        let tz = 2.0 * (self.x * y - self.y * x);
        [
            x + self.w * tx + self.y * tz - self.z * ty,
            y + self.w * ty + self.z * tx - self.x * tz,
            z + self.w * tz + self.x * ty - self.y * tx,
        ]
    }

    fn yaw(self) -> f64 {
        let sin_yaw = 2.0 * (self.w * self.z + self.x * self.y);
        let cos_yaw = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        sin_yaw.atan2(cos_yaw)
    }

    #[cfg(test)]
    fn wxyz(self) -> [f64; 4] {
        [self.w, self.x, self.y, self.z]
    }
}

#[derive(Debug, Clone, Copy)]
struct Pose {
    position: [f64; 3],
    rotation: Quat,
}

impl Pose {
    const IDENTITY: Self = Self {
        position: [0.0; 3],
        rotation: Quat::IDENTITY,
    };

    const fn new(position: [f64; 3], rotation: Quat) -> Self {
        Self { position, rotation }
    }

    fn point(self, local: [f64; 3]) -> [f64; 3] {
        let rotated = self.rotation.rotate(local);
        [
            self.position[0] + rotated[0],
            self.position[1] + rotated[1],
            self.position[2] + rotated[2],
        ]
    }

    fn then(self, child: Self) -> Self {
        Self {
            position: self.point(child.position),
            rotation: self.rotation.multiply(child.rotation),
        }
    }

    fn hinge_z(self, angle: f64) -> Self {
        Self {
            rotation: self.rotation.multiply(Quat::about_z(angle)),
            ..self
        }
    }
}

struct LegLink {
    rest: Pose,
    joint_offset: usize,
}

const LEFT_LEG: [LegLink; 5] = [
    LegLink {
        rest: Pose::new(
            [0.006, 0.0175, -0.005],
            Quat::new(0.0, -0.707107, -0.707107, 0.0),
        ),
        joint_offset: 0,
    },
    LegLink {
        rest: Pose::new(
            [0.0, 0.0165, 0.0125],
            Quat::new(0.707107, -0.707107, 0.0, 0.0),
        ),
        joint_offset: 1,
    },
    LegLink {
        rest: Pose::new([0.025, 0.0, -0.0185], Quat::new(0.5, -0.5, 0.5, -0.5)),
        joint_offset: 2,
    },
    LegLink {
        rest: Pose::new(
            [0.022, 0.0357771, -0.004],
            Quat::new(0.0, 0.707107, 0.707107, 0.0),
        ),
        joint_offset: 3,
    },
    LegLink {
        rest: Pose::new([0.0, 0.042, -0.026], Quat::new(0.0, 1.0, 0.0, 0.0)),
        joint_offset: 4,
    },
];

const RIGHT_LEG: [LegLink; 5] = [
    LegLink {
        rest: Pose::new(
            [0.006, -0.0175, -0.005],
            Quat::new(0.0, -0.707107, -0.707107, 0.0),
        ),
        joint_offset: 10,
    },
    LegLink {
        rest: Pose::new(
            [0.0, 0.0165, 0.0125],
            Quat::new(0.0, 0.0, 0.707107, 0.707107),
        ),
        joint_offset: 11,
    },
    LegLink {
        rest: Pose::new([0.025, 0.0, -0.0185], Quat::new(0.5, -0.5, 0.5, -0.5)),
        joint_offset: 12,
    },
    LegLink {
        rest: Pose::new([-0.022, 0.0357771, -0.004], Quat::new(0.0, 1.0, 0.0, 0.0)),
        joint_offset: 13,
    },
    LegLink {
        rest: Pose::new(
            [-0.042, 0.0, -0.026],
            Quat::new(0.0, -0.707107, 0.707107, 0.0),
        ),
        joint_offset: 14,
    },
];

const FOOT_SITES: [Pose; 2] = [
    Pose::new(
        [0.0, -0.0237879, -0.0140852],
        Quat::new(0.0, 0.0, 0.707107, 0.707107),
    ),
    Pose::new(
        [0.0, 0.0237879, -0.0140852],
        Quat::new(0.0, 0.0, 0.707107, -0.707107),
    ),
];

fn foot_pose(joints: &[f64; NUM_JOINTS], foot: usize) -> Pose {
    let chain = if foot == LEFT { &LEFT_LEG } else { &RIGHT_LEG };
    let mut pose = Pose::IDENTITY;
    for link in chain {
        let rest = Pose {
            rotation: link.rest.rotation.normalized(),
            ..link.rest
        };
        pose = pose.then(rest).hinge_z(joints[link.joint_offset]);
    }
    let site = Pose {
        rotation: FOOT_SITES[foot].rotation.normalized(),
        ..FOOT_SITES[foot]
    };
    pose.then(site)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OdometrySnapshot {
    pub position: [f64; 3],
    pub yaw: f64,
    pub anchor_xy: [f64; 2],
    pub anchor_foot: usize,
    pub anchor_switches: u64,
    pub pending_foot: Option<usize>,
    pub pending_ticks: u32,
}

impl Default for OdometrySnapshot {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            yaw: 0.0,
            anchor_xy: [0.0; 2],
            anchor_foot: LEFT,
            anchor_switches: 0,
            pending_foot: None,
            pending_ticks: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Pending {
    foot: usize,
    local: [f64; 3],
    world_xy: [f64; 2],
    ticks: u32,
}

pub struct Odometry {
    anchor_foot: usize,
    anchor_local: [f64; 3],
    anchor_xy: [f64; 2],
    position: [f64; 3],
    yaw: f64,
    pending: Option<Pending>,
    anchor_switches: u64,
    needs_init: bool,
}

impl Odometry {
    pub fn alpha() -> Self {
        Self {
            anchor_foot: LEFT,
            anchor_local: [0.0; 3],
            anchor_xy: [0.0; 2],
            position: [0.0; 3],
            yaw: 0.0,
            pending: None,
            anchor_switches: 0,
            needs_init: true,
        }
    }

    pub fn update(&mut self, joints: &[f64; NUM_JOINTS], quat_wxyz: [f64; 4]) {
        let rotation = Quat::from_wxyz(quat_wxyz).normalized();
        let feet = [foot_pose(joints, LEFT), foot_pose(joints, RIGHT)];

        if self.needs_init {
            let left_origin = rotation.rotate(feet[LEFT].position);
            self.anchor_xy = [left_origin[0], left_origin[1]];
            self.needs_init = false;
        }

        self.reproject(rotation, &feet);
        let candidate = self.lowest_candidate(rotation, &feet);
        self.update_pending(candidate, rotation, &feet);
        self.yaw = rotation.yaw();
    }

    fn update_pending(&mut self, candidate: Option<Pending>, rotation: Quat, feet: &[Pose; 2]) {
        match candidate {
            None => self.pending = None,
            Some(candidate) => {
                if self
                    .pending
                    .as_ref()
                    .is_some_and(|pending| pending.foot == candidate.foot)
                {
                    if let Some(pending) = &mut self.pending {
                        pending.ticks += 1;
                    }
                } else {
                    self.pending = Some(candidate);
                }

                if self
                    .pending
                    .is_some_and(|pending| pending.ticks >= SWITCH_CONFIRM_TICKS)
                {
                    let confirmed = self.pending.take().expect("confirmed candidate exists");
                    self.anchor_foot = confirmed.foot;
                    self.anchor_local = confirmed.local;
                    self.anchor_xy = confirmed.world_xy;
                    self.anchor_switches += 1;
                    self.reproject(rotation, feet);
                }
            }
        }
    }

    fn reproject(&mut self, rotation: Quat, feet: &[Pose; 2]) {
        let contact_in_trunk = feet[self.anchor_foot].point(self.anchor_local);
        let contact = rotation.rotate(contact_in_trunk);
        self.position = [
            self.anchor_xy[0] - contact[0],
            self.anchor_xy[1] - contact[1],
            -contact[2],
        ];
    }

    fn lowest_candidate(&self, rotation: Quat, feet: &[Pose; 2]) -> Option<Pending> {
        let mut lowest_z = SWITCH_WORLD_Z;
        let mut candidate = None;
        for foot in [LEFT, RIGHT] {
            for local in ALPHA16[foot] {
                let contact = rotation.rotate(feet[foot].point(local));
                let world = [
                    self.position[0] + contact[0],
                    self.position[1] + contact[1],
                    self.position[2] + contact[2],
                ];
                if world[2] < lowest_z {
                    lowest_z = world[2];
                    candidate = Some(Pending {
                        foot,
                        local,
                        world_xy: [world[0], world[1]],
                        ticks: 1,
                    });
                }
            }
        }
        candidate
    }

    pub fn snapshot(&self) -> OdometrySnapshot {
        OdometrySnapshot {
            position: self.position,
            yaw: self.yaw,
            anchor_xy: self.anchor_xy,
            anchor_foot: self.anchor_foot,
            anchor_switches: self.anchor_switches,
            pending_foot: self.pending.map(|pending| pending.foot),
            pending_ticks: self.pending.map_or(0, |pending| pending.ticks),
        }
    }
}

impl Default for Odometry {
    fn default() -> Self {
        Self::alpha()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yaw_quat(yaw: f64) -> [f64; 4] {
        [(yaw / 2.0).cos(), 0.0, 0.0, (yaw / 2.0).sin()]
    }

    fn roll_quat(roll: f64) -> [f64; 4] {
        [(roll / 2.0).cos(), (roll / 2.0).sin(), 0.0, 0.0]
    }

    fn assert_vector(actual: [f64; 3], expected: [f64; 3], tolerance: f64) {
        for axis in 0..3 {
            assert!(
                (actual[axis] - expected[axis]).abs() < tolerance,
                "axis {axis}: actual {actual:?}, expected {expected:?}"
            );
        }
    }

    #[test]
    fn alpha16_has_sixteen_bounded_mirrored_points_per_foot() {
        assert_eq!(ALPHA16[LEFT].len(), 16);
        assert_eq!(ALPHA16[RIGHT].len(), 16);
        for points in ALPHA16 {
            for point in points {
                assert!((-0.021..=0.035).contains(&point[0]));
                assert!((-0.022..=0.022).contains(&point[1]));
                assert!((-0.002..=0.012).contains(&point[2]));
            }
        }
        for right in ALPHA16[RIGHT] {
            assert!(
                ALPHA16[LEFT].iter().any(|left| {
                    (right[0] - left[0]).abs() < 3e-4
                        && (right[1] + left[1]).abs() < 3e-4
                        && (right[2] - left[2]).abs() < 3e-4
                }),
                "no mirror for {right:?}"
            );
        }
    }

    #[test]
    fn fixed_leg_fk_matches_upstream_mujoco_fixtures() {
        let cases = [
            (
                [
                    0.48841914856958574,
                    -0.2160047578882288,
                    0.8827903082279525,
                    -1.0593515089821106,
                    0.8341777527314465,
                    -0.2608837610885546,
                    -0.2799168207394651,
                    0.03853788898969057,
                    -0.22723482104100412,
                    0.0,
                    -0.0998030129691927,
                    -0.08252700944989971,
                    -1.3863963789463658,
                    -0.1863673961680259,
                    -1.348023663881893,
                ],
                [
                    -0.05267356796541412,
                    0.09749247571622972,
                    0.025384670871651166,
                ],
                [
                    0.14957745053037239,
                    0.2173859701317229,
                    0.9533644624021315,
                    -0.14651323527728757,
                ],
                [
                    -0.07325073347470062,
                    -0.05545018790883256,
                    0.0207301581250213,
                ],
                [
                    0.293788019309182,
                    -0.05970807477755525,
                    0.9536809020848409,
                    -0.02482503807426184,
                ],
            ),
            (
                [
                    0.19184437964246276,
                    0.17703394067455125,
                    -0.07944191354406027,
                    -1.1046998935310839,
                    -1.537364743699262,
                    -0.5760663170899201,
                    -0.11942608302330915,
                    -2.8592542005156916,
                    0.2120394465562941,
                    0.0,
                    0.32628821378710715,
                    -0.26902530514010864,
                    1.1819168641403959,
                    0.7211221197893845,
                    0.23782176797891497,
                ],
                [
                    -0.04462419405020574,
                    0.07241233062444753,
                    -0.07795046296211391,
                ],
                [
                    0.9570355959509607,
                    0.06096465280176598,
                    -0.2593039603813678,
                    -0.11457589328513612,
                ],
                [
                    0.024263605355733804,
                    -0.08447528158379942,
                    -0.10253400272516706,
                ],
                [
                    0.9262129081262941,
                    -0.17942162143751972,
                    -0.3141863503245751,
                    -0.10594558910444084,
                ],
            ),
        ];

        for (joints, left_position, left_rotation, right_position, right_rotation) in cases {
            for (foot, expected_position, expected_rotation) in [
                (LEFT, left_position, left_rotation),
                (RIGHT, right_position, right_rotation),
            ] {
                let actual = foot_pose(&joints, foot);
                assert_vector(actual.position, expected_position, 1e-6);
                let actual_rotation = actual.rotation.wxyz();
                for axis in 0..4 {
                    let error = (actual_rotation[axis] - expected_rotation[axis])
                        .abs()
                        .min((actual_rotation[axis] + expected_rotation[axis]).abs());
                    assert!(error < 1e-6, "foot {foot}, quaternion axis {axis}: {error}");
                }
            }
        }
    }

    #[test]
    fn every_reference_tick_matches_upstream_bitwise() {
        let fixture = include_str!("../tests/fixtures/odometry_reference_400.csv");
        let mut odometry = Odometry::alpha();
        let mut joints = [0.0; NUM_JOINTS];
        let mut previous_foot = None;
        let mut transitions = 0;
        let mut checked = 0;

        for (tick, line) in fixture.lines().skip(1).enumerate() {
            let fields: Vec<_> = line.split(',').collect();
            assert_eq!(fields.len(), 8, "tick {tick}: malformed fixture row");
            assert_eq!(fields[0].parse::<usize>().unwrap(), tick);
            let number = |field: usize| fields[field].parse::<f64>().unwrap();
            let expected_position = [number(1), number(2), number(3)];
            let expected_anchor = [number(6), number(7)];

            let t = tick as f64;
            joints[1] = 0.15 * (0.05 * t).sin();
            joints[2] = 0.30 * (0.07 * t).sin();
            joints[3] = 0.20 * (0.07 * t).cos();
            joints[12] = 0.30 * (0.07 * t).cos();
            joints[13] = 0.20 * (0.07 * t).sin();
            odometry.update(&joints, roll_quat(0.10 * (0.05 * t).sin()));
            let actual = odometry.snapshot();

            for axis in 0..3 {
                assert_eq!(
                    actual.position[axis].to_bits(),
                    expected_position[axis].to_bits(),
                    "tick {tick}: position axis {axis}"
                );
            }
            assert_eq!(actual.yaw.to_bits(), number(4).to_bits(), "tick {tick}: yaw");
            assert_eq!(actual.anchor_foot, number(5) as usize, "tick {tick}: anchor foot");
            for axis in 0..2 {
                assert_eq!(
                    actual.anchor_xy[axis].to_bits(),
                    expected_anchor[axis].to_bits(),
                    "tick {tick}: anchor XY axis {axis}"
                );
            }
            if previous_foot.is_some_and(|foot| foot != actual.anchor_foot) {
                transitions += 1;
            }
            previous_foot = Some(actual.anchor_foot);
            checked += 1;
        }
        assert_eq!(checked, 400);
        assert_eq!(transitions, 9);
    }

    #[test]
    fn standing_still_stays_at_origin_for_two_hundred_ticks() {
        let mut odometry = Odometry::alpha();
        for _ in 0..200 {
            odometry.update(&[0.0; NUM_JOINTS], [1.0, 0.0, 0.0, 0.0]);
        }
        let state = odometry.snapshot();
        assert!(state.position[0].abs() < 1e-9 && state.position[1].abs() < 1e-9);
        assert!(state.position[2] > 0.02);
        assert_eq!(state.yaw, 0.0);
    }

    #[test]
    fn yaw_comes_from_normalized_imu_quaternion() {
        let mut odometry = Odometry::alpha();
        let mut quat = yaw_quat(0.3);
        for value in &mut quat {
            *value *= 3.0;
        }
        odometry.update(&[0.0; NUM_JOINTS], quat);
        assert!((odometry.snapshot().yaw - 0.3).abs() < 1e-12);
    }

    #[test]
    fn mouth_angle_does_not_change_estimate() {
        let mut quiet = Odometry::alpha();
        let mut mouth_at_42 = Odometry::alpha();
        let mut joints = [0.0; NUM_JOINTS];
        for _ in 0..10 {
            quiet.update(&joints, [1.0, 0.0, 0.0, 0.0]);
            joints[9] = 42.0;
            mouth_at_42.update(&joints, [1.0, 0.0, 0.0, 0.0]);
        }
        assert_eq!(quiet.snapshot(), mouth_at_42.snapshot());
    }

    #[test]
    fn one_tick_candidate_does_not_switch_and_two_ticks_confirm() {
        let mut odometry = Odometry::alpha();
        for _ in 0..10 {
            odometry.update(&[0.0; NUM_JOINTS], [1.0, 0.0, 0.0, 0.0]);
        }
        let initial = odometry.snapshot().anchor_foot;
        let roll = if initial == LEFT { 0.35 } else { -0.35 };

        odometry.update(&[0.0; NUM_JOINTS], roll_quat(roll));
        assert_eq!(odometry.snapshot().anchor_foot, initial);
        odometry.update(&[0.0; NUM_JOINTS], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(odometry.snapshot().anchor_foot, initial);

        odometry.update(&[0.0; NUM_JOINTS], roll_quat(roll));
        assert_eq!(odometry.snapshot().anchor_foot, initial);
        odometry.update(&[0.0; NUM_JOINTS], roll_quat(roll));
        assert_ne!(odometry.snapshot().anchor_foot, initial);
    }

    #[test]
    fn a_different_foot_restarts_confirmation() {
        let mut odometry = Odometry::alpha();
        odometry.update(&[0.0; NUM_JOINTS], [1.0, 0.0, 0.0, 0.0]);
        odometry.update(&[0.0; NUM_JOINTS], roll_quat(0.35));
        assert_eq!(odometry.snapshot().pending_ticks, 1);
        odometry.update(&[0.0; NUM_JOINTS], roll_quat(-0.35));
        assert_eq!(odometry.snapshot().pending_ticks, 1);
        assert_eq!(odometry.snapshot().pending_foot, Some(LEFT));
        odometry.update(&[0.0; NUM_JOINTS], roll_quat(-0.35));
        assert_eq!(odometry.snapshot().pending_ticks, 0);
    }

    #[test]
    fn no_candidate_clears_pending() {
        let mut odometry = Odometry::alpha();
        odometry.update(&[0.0; NUM_JOINTS], [1.0, 0.0, 0.0, 0.0]);
        odometry.update(&[0.0; NUM_JOINTS], roll_quat(0.35));
        assert_eq!(odometry.snapshot().pending_ticks, 1);
        odometry.update_pending(None, Quat::IDENTITY, &[Pose::IDENTITY; 2]);
        assert_eq!(odometry.snapshot().pending_foot, None);
        assert_eq!(odometry.snapshot().pending_ticks, 0);
    }

    #[test]
    fn stance_sweep_moves_trunk_without_twenty_mm_jumps() {
        let mut odometry = Odometry::alpha();
        let mut joints = [0.0; NUM_JOINTS];
        for _ in 0..10 {
            odometry.update(&joints, [1.0, 0.0, 0.0, 0.0]);
        }
        let start = odometry.snapshot().position;
        let mut previous = start;
        for tick in 1..=50 {
            let angle = 0.2 * tick as f64 / 50.0;
            joints[2] = angle;
            joints[12] = angle;
            odometry.update(&joints, [1.0, 0.0, 0.0, 0.0]);
            let current = odometry.snapshot().position;
            let jump = (0..3)
                .map(|axis| (current[axis] - previous[axis]).powi(2))
                .sum::<f64>()
                .sqrt();
            assert!(jump < 0.02, "tick {tick}: {jump}");
            previous = current;
        }
        let travel = ((previous[0] - start[0]).powi(2) + (previous[1] - start[1]).powi(2)).sqrt();
        assert!(travel > 0.005, "travel {travel}");
    }
}

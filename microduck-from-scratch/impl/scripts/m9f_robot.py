from __future__ import annotations

from pathlib import Path

import mujoco
from bam.mjlab import BamActuatorCfg
from mjlab.entity import EntityArticulationInfoCfg, EntityCfg
from mjlab.utils.spec_config import CollisionCfg

HANDMADE_XML = Path(__file__).resolve().parents[1] / "sim" / "assets" / "duck_handmade.xml"

FOOT_GEOM_NAMES = (
    "left_foot_collision",
    "left_foot_collision_part2",
    "right_foot_collision",
    "right_foot_collision_part2",
)
FOOT_GEOM_PATTERN = r"^(left|right)_foot_collision(_part2)?$"


def get_handmade_spec() -> mujoco.MjSpec:
    return mujoco.MjSpec.from_file(str(HANDMADE_XML))


HANDMADE_HOME = EntityCfg.InitialStateCfg(
    joint_pos={
        r".*hip_yaw.*": 0.0,
        r".*left_hip_roll.*": -0.0873,
        r".*right_hip_roll.*": 0.0873,
        r".*left_hip_pitch.*": -0.4579,
        r".*right_hip_pitch.*": 0.4579,
        r".*left_knee.*": -0.0049,
        r".*right_knee.*": 0.0049,
        r".*left_ankle.*": 0.4530,
        r".*right_ankle.*": -0.4530,
        r".*neck_pitch.*": 0.3491,
        r".*head_pitch.*": 0.3491,
        r".*head_yaw.*": 0.0,
        r".*head_roll.*": 0.0,
    },
    joint_vel={".*": 0.0},
)

HANDMADE_COLLISION = CollisionCfg(
    geom_names_expr=[FOOT_GEOM_PATTERN, ".*_collision"],
    condim={FOOT_GEOM_PATTERN: 3, ".*_collision": 1},
    priority={FOOT_GEOM_PATTERN: 1},
    friction={FOOT_GEOM_PATTERN: (1.0,)},
)

HANDMADE_BAM = BamActuatorCfg(
    motor_name="xl330",
    model="m6",
    target_names_expr=(r"^(?!passive_).*",),
    kp_fw=200.0,
    vin_range=(6.5, 8.2),
    vin_drop_gain_range=(0.0, 0.2),
    vin_min=6.0,
    delay_min_lag=3,
    delay_max_lag=6,
)

HANDMADE_ROBOT_CFG = EntityCfg(
    spec_fn=get_handmade_spec,
    init_state=HANDMADE_HOME,
    collisions=(HANDMADE_COLLISION,),
    articulation=EntityArticulationInfoCfg(
        actuators=(HANDMADE_BAM,),
        soft_joint_pos_limit_factor=0.9,
    ),
)

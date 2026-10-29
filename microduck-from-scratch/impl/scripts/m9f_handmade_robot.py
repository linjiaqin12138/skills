"""Handmade MicroDuck EntityCfg for M9f Velocity training.

Mirrors MICRODUCK_WALK_ROBOT_CFG (HOME_FRAME / BAM actuators / FULL_COLLISION)
but loads impl/sim/assets/duck_handmade.xml instead of official robot_walk.xml.
"""

from __future__ import annotations

from pathlib import Path

import mujoco
from mjlab.entity import EntityArticulationInfoCfg, EntityCfg
from mjlab_microduck.robot.microduck_constants import (
    FULL_COLLISION,
    HOME_FRAME,
    actuators,
)

# Absolute path so MjSpec resolves meshdir=assets next to this XML regardless of cwd.
HANDMADE_XML: Path = (
    Path(__file__).resolve().parents[1] / "sim" / "assets" / "duck_handmade.xml"
)


def get_handmade_spec() -> mujoco.MjSpec:
    return mujoco.MjSpec.from_file(str(HANDMADE_XML))


HANDMADE_ROBOT_CFG = EntityCfg(
    spec_fn=get_handmade_spec,
    init_state=HOME_FRAME,
    collisions=(FULL_COLLISION,),
    articulation=EntityArticulationInfoCfg(
        actuators=(actuators,),
        soft_joint_pos_limit_factor=0.9,
    ),
)

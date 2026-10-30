from __future__ import annotations

import math
from copy import deepcopy

from mjlab.envs import ManagerBasedRlEnvCfg
from mjlab.envs.mdp import dr
from mjlab.envs.mdp.actions import JointPositionActionCfg
from mjlab.managers import (
    CurriculumTermCfg,
    EventTermCfg,
    ObservationTermCfg,
    RewardTermCfg,
    TerminationTermCfg,
)
from mjlab.managers.scene_entity_config import SceneEntityCfg
from mjlab.rl import RslRlModelCfg, RslRlOnPolicyRunnerCfg, RslRlPpoAlgorithmCfg
from mjlab.sensor import (
    ContactMatch,
    ContactSensorCfg,
    ObjRef,
    RingPatternCfg,
    TerrainHeightSensorCfg,
)
from mjlab.tasks.velocity import mdp
from mjlab.tasks.velocity.mdp import UniformVelocityCommandCfg
from mjlab.tasks.velocity.velocity_env_cfg import make_velocity_env_cfg
from mjlab.terrains import TerrainEntityCfg
from mjlab.utils.noise import UniformNoiseCfg as Unoise

import m9f_mdp
from m9f_robot import FOOT_GEOM_NAMES, FOOT_GEOM_PATTERN, HANDMADE_ROBOT_CFG

ACTOR_OBS_DIM = 61
ACTION_DIM = 14
SERVO_JOINT_PATTERN = r"^(?!passive_).*"
SITE_NAMES = ("left_foot", "right_foot")


def make_handmade_rl_cfg() -> RslRlOnPolicyRunnerCfg:
    return RslRlOnPolicyRunnerCfg(
        actor=RslRlModelCfg(
            hidden_dims=(512, 256, 128),
            activation="elu",
            obs_normalization=True,
            distribution_cfg={
                "class_name": "GaussianDistribution",
                "init_std": 1.0,
                "std_type": "scalar",
            },
        ),
        critic=RslRlModelCfg(
            hidden_dims=(512, 256, 128),
            activation="elu",
            obs_normalization=True,
        ),
        algorithm=RslRlPpoAlgorithmCfg(
            value_loss_coef=1.0,
            use_clipped_value_loss=True,
            clip_param=0.2,
            entropy_coef=0.01,
            num_learning_epochs=5,
            num_mini_batches=4,
            learning_rate=1.0e-3,
            schedule="adaptive",
            gamma=0.99,
            lam=0.95,
            desired_kl=0.01,
            max_grad_norm=1.0,
        ),
        logger="tensorboard",
        upload_model=False,
        experiment_name="m9f_handmade_velocity",
        run_name="own_task",
        save_interval=50,
        num_steps_per_env=24,
        max_iterations=1000,
    )


def make_handmade_velocity_env_cfg(play: bool = False) -> ManagerBasedRlEnvCfg:
    feet_contact = ContactSensorCfg(
        name="feet_ground_contact",
        primary=ContactMatch(mode="geom", pattern=FOOT_GEOM_PATTERN, entity="robot"),
        secondary=ContactMatch(mode="body", pattern="terrain"),
        fields=("found", "force"),
        reduce="netforce",
        num_slots=1,
        track_air_time=True,
    )
    self_contact = ContactSensorCfg(
        name="self_collision",
        primary=ContactMatch(mode="subtree", pattern="trunk_base", entity="robot"),
        secondary=ContactMatch(mode="subtree", pattern="trunk_base", entity="robot"),
        fields=("found",),
        reduce="none",
        num_slots=1,
    )
    foot_height = TerrainHeightSensorCfg(
        name="foot_height_scan",
        frame=tuple(ObjRef(type="site", name=name, entity="robot") for name in SITE_NAMES),
        pattern=RingPatternCfg.single_ring(radius=0.04, num_samples=2),
        ray_alignment="yaw",
        max_distance=1.0,
        exclude_parent_body=True,
        include_geom_groups=(0,),
        debug_vis=False,
    )

    cfg = make_velocity_env_cfg()
    cfg.scene.entities = {"robot": HANDMADE_ROBOT_CFG}
    cfg.scene.sensors = (feet_contact, self_contact, foot_height)
    cfg.scene.terrain = TerrainEntityCfg(terrain_type="plane")
    cfg.viewer.body_name = "trunk_base"
    cfg.viewer.distance = 1.2

    action = cfg.actions["joint_pos"]
    if not isinstance(action, JointPositionActionCfg):
        raise TypeError("joint_pos must remain a position action")
    action.actuator_names = (SERVO_JOINT_PATTERN,)
    action.scale = 1.0
    action.use_default_offset = True

    actor = cfg.observations["actor"]
    critic = cfg.observations["critic"]
    del actor.terms["base_lin_vel"]
    del actor.terms["height_scan"]
    del critic.terms["height_scan"]
    critic.terms.pop("foot_contact_forces", None)

    servo_assets = SceneEntityCfg("robot", joint_names=(SERVO_JOINT_PATTERN,))
    for group in (actor, critic):
        for name in ("joint_pos", "joint_vel"):
            group.terms[name] = deepcopy(group.terms[name])
            group.terms[name].params["asset_cfg"] = deepcopy(servo_assets)
        group.terms["head_command"] = ObservationTermCfg(
            func=m9f_mdp.zero_observation,
            params={"width": 4},
        )
        group.terms["body_command"] = ObservationTermCfg(
            func=m9f_mdp.zero_observation,
            params={"width": 6},
        )

    actor.terms["base_ang_vel"] = deepcopy(actor.terms["base_ang_vel"])
    actor.terms["projected_gravity"] = deepcopy(actor.terms["projected_gravity"])
    actor.terms["joint_vel"] = deepcopy(actor.terms["joint_vel"])
    for name in ("base_ang_vel", "projected_gravity"):
        actor.terms[name].delay_min_lag = 0
        actor.terms[name].delay_max_lag = 1
        actor.terms[name].delay_update_period = 64
    actor.terms["joint_vel"].delay_min_lag = 1
    actor.terms["joint_vel"].delay_max_lag = 1
    actor.terms["joint_vel"].delay_update_period = 0

    actor.terms["base_ang_vel"].noise = Unoise(n_min=-0.03, n_max=0.03)
    actor.terms["projected_gravity"].noise = Unoise(n_min=-0.01, n_max=0.01)
    actor.terms["joint_pos"].noise = Unoise(n_min=-0.001, n_max=0.001)
    actor.terms["joint_vel"].noise = Unoise(n_min=-0.25, n_max=0.25)
    actor.terms["joint_pos"].params["biased"] = True
    critic.terms["joint_pos"].params["biased"] = False
    critic.terms["foot_air_time"].func = m9f_mdp.foot_air_time
    critic.terms["foot_contact"].func = m9f_mdp.foot_contact

    command = deepcopy(cfg.commands["twist"])
    if not isinstance(command, UniformVelocityCommandCfg):
        raise TypeError("twist must remain a velocity command")
    command.heading_command = False
    command.ranges.heading = None
    command.rel_heading_envs = 0.0
    command.rel_world_envs = 0.0
    command.rel_forward_envs = 0.0
    command.rel_standing_envs = 0.02
    command.resampling_time_range = (3.0, 8.0)
    command.ranges.lin_vel_x = (0.2, 0.4)
    command.ranges.lin_vel_y = (0.0, 0.0)
    command.ranges.ang_vel_z = (0.0, 0.0)
    cfg.commands = {"twist": command}

    cfg.rewards["track_linear_velocity"] = RewardTermCfg(
        func=m9f_mdp.track_forward_velocity,
        weight=4.0,
        params={
            "command_name": "twist",
            "std": math.sqrt(0.1),
            "min_height": 0.085,
            "full_height": 0.115,
        },
    )
    cfg.rewards["track_angular_velocity"].weight = 0.0
    cfg.rewards["track_angular_velocity"].params["std"] = math.sqrt(0.5)
    cfg.rewards["forward_velocity"] = RewardTermCfg(
        func=m9f_mdp.forward_velocity,
        weight=2.0,
        params={"command_name": "twist", "min_height": 0.085, "full_height": 0.115},
    )
    cfg.rewards["upright"].weight = 2.0
    cfg.rewards["upright"].params["std"] = math.sqrt(0.05)
    cfg.rewards["upright"].params["asset_cfg"].body_names = ("trunk_base",)

    standing_std = {
        r".*hip_yaw.*": 0.1,
        r".*hip_roll.*": 0.05,
        r".*hip_pitch.*": 0.15,
        r".*knee.*": 0.15,
        r".*ankle.*": 0.1,
        r".*neck.*|.*head.*": 0.2,
    }
    walking_std = {
        r".*hip_yaw.*": 0.3,
        r".*hip_roll.*": 0.05,
        r".*hip_pitch.*": 0.4,
        r".*knee.*": 0.4,
        r".*ankle.*": 0.25,
        r".*neck.*|.*head.*": 0.3,
    }
    cfg.rewards["pose"].params.update(
        asset_cfg=servo_assets,
        std_standing=standing_std,
        std_walking=walking_std,
        std_running=walking_std,
        walking_threshold=0.01,
    )
    cfg.rewards["pose"].weight = 1.0
    cfg.rewards["air_time"] = RewardTermCfg(
        func=m9f_mdp.feet_air_time,
        weight=3.0,
        params={
            "sensor_name": feet_contact.name,
            "command_name": "twist",
            "threshold_min": 0.125,
            "threshold_max": 0.3,
            "command_threshold": 0.01,
            "min_height": 0.085,
            "full_height": 0.115,
        },
    )
    cfg.rewards["foot_clearance"].params.update(
        target_height=0.02,
        command_threshold=0.01,
        asset_cfg=SceneEntityCfg("robot", site_names=SITE_NAMES),
    )
    cfg.rewards.pop("foot_swing_height", None)
    cfg.rewards["foot_slip"] = RewardTermCfg(
        func=m9f_mdp.feet_slip,
        weight=-0.1,
        params={
            "sensor_name": feet_contact.name,
            "command_name": "twist",
            "command_threshold": 0.01,
            "asset_cfg": SceneEntityCfg("robot", site_names=SITE_NAMES),
        },
    )
    cfg.rewards["self_collisions"] = RewardTermCfg(
        func=mdp.self_collision_cost,
        weight=-1.0,
        params={"sensor_name": self_contact.name},
    )
    cfg.rewards["body_ang_vel"].weight = -0.05
    cfg.rewards["body_ang_vel"].params["asset_cfg"].body_names = ("trunk_base",)
    cfg.rewards["angular_momentum"].weight = -0.02
    cfg.rewards["action_rate_l2"].weight = -0.1
    cfg.rewards.pop("soft_landing", None)

    cfg.events["expand_bam_fields"] = EventTermCfg(
        func=m9f_mdp.expand_bam_fields,
        mode="startup",
    )
    cfg.events["foot_friction"].params["asset_cfg"].geom_names = FOOT_GEOM_NAMES
    cfg.events["foot_friction"].params["ranges"] = (0.7, 1.3)
    cfg.events["base_com"].params["asset_cfg"].body_names = ("trunk_base",)
    cfg.events["base_com"].params["ranges"] = {
        0: (-0.003, 0.003),
        1: (-0.003, 0.003),
        2: (-0.003, 0.003),
    }
    cfg.events["reset_base"].params["pose_range"]["z"] = (0.12, 0.13)
    cfg.events["reset_robot_joints"].params["asset_cfg"].joint_names = (SERVO_JOINT_PATTERN,)
    cfg.events.pop("push_robot", None)
    cfg.events["randomize_armature"] = EventTermCfg(
        func=dr.joint_armature,
        mode="reset",
        params={
            "asset_cfg": SceneEntityCfg("robot", joint_names=(SERVO_JOINT_PATTERN,)),
            "operation": "scale",
            "ranges": (0.9, 1.1),
        },
    )

    cfg.terminations["nan_state"] = TerminationTermCfg(
        func=m9f_mdp.robot_state_is_nan,
        time_out=False,
        params={"sensor_names": (feet_contact.name,)},
    )
    cfg.curriculum.clear()
    cfg.curriculum["action_rate_weight"] = CurriculumTermCfg(
        func=m9f_mdp.reward_weight,
        params={
            "reward_name": "action_rate_l2",
            "weight_stages": [
                {"step": 0, "weight": -0.1},
                {"step": 500 * 24, "weight": -0.2},
                {"step": 750 * 24, "weight": -0.4},
                {"step": 1000 * 24, "weight": -0.6},
            ],
        },
    )
    cfg.curriculum["standing_envs"] = CurriculumTermCfg(
        func=m9f_mdp.standing_envs,
        params={
            "command_name": "twist",
            "stages": [
                {"step": 0, "fraction": 0.02},
                {"step": 500 * 24, "fraction": 0.05},
                {"step": 750 * 24, "fraction": 0.1},
                {"step": 1000 * 24, "fraction": 0.15},
            ],
        },
    )

    if play:
        for name in (
            "push_robot",
            "foot_friction",
            "base_com",
            "encoder_bias",
            "randomize_armature",
        ):
            cfg.events.pop(name, None)
        cfg.curriculum.clear()
        actor.enable_corruption = False
        actor.terms["joint_pos"].params["biased"] = False
        command.rel_standing_envs = 0.0
        command.rel_forward_envs = 0.0
        command.resampling_time_range = (999.0, 1000.0)
        command.debug_vis = False
        cfg.episode_length_s = 30.0

    return cfg

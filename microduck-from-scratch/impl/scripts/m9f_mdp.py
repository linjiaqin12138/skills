from __future__ import annotations

import torch
from mjlab.managers.event_manager import requires_model_fields
from mjlab.managers.scene_entity_config import SceneEntityCfg


@requires_model_fields("dof_frictionloss", "dof_damping")
def expand_bam_fields(env, env_ids) -> None:
    del env, env_ids


def _per_foot(values: torch.Tensor) -> torch.Tensor:
    if values.ndim != 2 or values.shape[1] != 4:
        raise ValueError(f"expected [batch, 4] foot-layer values, got {tuple(values.shape)}")
    return values.reshape(values.shape[0], 2, 2)


def zero_observation(env, width: int) -> torch.Tensor:
    return torch.zeros(env.num_envs, width, device=env.device)


def foot_air_time(env, sensor_name: str) -> torch.Tensor:
    sensor = env.scene[sensor_name]
    values = sensor.data.current_air_time
    if values is None:
        raise ValueError(f"{sensor_name} does not track air time")
    return _per_foot(values).max(dim=2).values


def foot_contact(env, sensor_name: str) -> torch.Tensor:
    sensor = env.scene[sensor_name]
    values = sensor.data.found
    if values is None:
        raise ValueError(f"{sensor_name} does not report contact")
    return _per_foot(values).any(dim=2).float()


def _height_gate(env, min_height: float, full_height: float) -> torch.Tensor:
    root_z = env.scene["robot"].data.root_link_pos_w[:, 2]
    return ((root_z - min_height) / (full_height - min_height)).clamp(0.0, 1.0)


def track_forward_velocity(
    env,
    command_name: str,
    std: float,
    min_height: float,
    full_height: float,
) -> torch.Tensor:
    command = env.command_manager.get_command(command_name)
    actual = env.scene["robot"].data.root_link_lin_vel_b[:, 0]
    tracking = torch.exp(-((command[:, 0] - actual) / std).square())
    return tracking * _height_gate(env, min_height, full_height)


def forward_velocity(
    env,
    command_name: str,
    min_height: float,
    full_height: float,
) -> torch.Tensor:
    command = env.command_manager.get_command(command_name)
    target = command[:, 0].clamp(min=0.05)
    actual = env.scene["robot"].data.root_link_lin_vel_b[:, 0]
    progress = (actual / target).clamp(min=0.0, max=1.0)
    return progress * _height_gate(env, min_height, full_height)


def feet_air_time(
    env,
    sensor_name: str,
    command_name: str,
    threshold_min: float,
    threshold_max: float,
    command_threshold: float,
    min_height: float,
    full_height: float,
) -> torch.Tensor:
    air_time = foot_air_time(env, sensor_name)
    in_range = (air_time > threshold_min) & (air_time < threshold_max)
    reward = in_range.float().sum(dim=1)
    command = env.command_manager.get_command(command_name)
    active = torch.norm(command[:, :2], dim=1) + command[:, 2].abs()
    reward = reward * (active > command_threshold).float()
    reward = reward * _height_gate(env, min_height, full_height)
    airborne = air_time > 0
    env.extras["log"]["Metrics/air_time_mean"] = (
        (air_time * airborne).sum() / airborne.sum().clamp(min=1)
    )
    return reward


def feet_slip(
    env,
    sensor_name: str,
    command_name: str,
    command_threshold: float,
    asset_cfg: SceneEntityCfg,
) -> torch.Tensor:
    asset = env.scene[asset_cfg.name]
    contact = foot_contact(env, sensor_name)
    foot_vel = asset.data.site_lin_vel_w[:, asset_cfg.site_ids, :2]
    speed_sq = foot_vel.square().sum(dim=-1)
    command = env.command_manager.get_command(command_name)
    active = torch.norm(command[:, :2], dim=1) + command[:, 2].abs()
    cost = (speed_sq * contact).sum(dim=1) * (active > command_threshold).float()
    env.extras["log"]["Metrics/slip_velocity_mean"] = (
        (speed_sq.sqrt() * contact).sum() / contact.sum().clamp(min=1)
    )
    return cost


def reward_weight(env, env_ids, reward_name: str, weight_stages: list[dict]) -> torch.Tensor:
    del env_ids
    term = env.reward_manager.get_term_cfg(reward_name)
    for stage in weight_stages:
        if env.common_step_counter >= stage["step"]:
            term.weight = stage["weight"]
    return torch.tensor([term.weight], device=env.device)


def standing_envs(env, env_ids, command_name: str, stages: list[dict]) -> torch.Tensor:
    del env_ids
    term = env.command_manager.get_term_cfg(command_name)
    for stage in stages:
        if env.common_step_counter >= stage["step"]:
            term.rel_standing_envs = stage["fraction"]
    return torch.tensor([term.rel_standing_envs], device=env.device)


def robot_state_is_nan(env, sensor_names: tuple[str, ...]) -> torch.Tensor:
    robot = env.scene["robot"]
    fields = (
        robot.data.root_link_pos_w,
        robot.data.root_link_quat_w,
        robot.data.root_link_lin_vel_w,
        robot.data.root_link_ang_vel_w,
        robot.data.joint_pos,
        robot.data.joint_vel,
    )
    invalid = torch.zeros(env.num_envs, dtype=torch.bool, device=env.device)
    for value in fields:
        invalid |= ~torch.isfinite(value).reshape(env.num_envs, -1).all(dim=1)
    for name in sensor_names:
        data = env.scene[name].data
        for value in (data.found, data.force, data.current_air_time):
            if value is not None:
                invalid |= ~torch.isfinite(value).reshape(env.num_envs, -1).all(dim=1)
    return invalid

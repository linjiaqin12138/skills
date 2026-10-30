#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
from dataclasses import asdict
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Evaluate a checkpoint in the own handmade environment")
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--num-envs", type=int, default=16)
    parser.add_argument("--seconds", type=float, default=5.0)
    parser.add_argument("--vx", type=float, default=0.3)
    return parser.parse_args()


def _set_forward_command(env, vx: float) -> None:
    term = env.command_manager.get_term("twist")
    for name in ("is_standing_env", "is_heading_env", "is_world_env", "is_forward_env"):
        getattr(term, name)[:] = False
    command = env.command_manager.get_command("twist")
    command[:, 0] = vx
    command[:, 1:] = 0.0


def main() -> None:
    args = parse_args()
    import torch
    from mjlab.envs import ManagerBasedRlEnv
    from mjlab.rl import RslRlVecEnvWrapper
    from mjlab.tasks.velocity.rl import VelocityOnPolicyRunner
    from mjlab.utils.torch import configure_torch_backends

    import m9f_mdp
    from m9f_env_cfg import make_handmade_rl_cfg, make_handmade_velocity_env_cfg
    from m9f_robot import HANDMADE_XML

    device = "cuda:0" if torch.cuda.is_available() else "cpu"
    if device == "cpu":
        os.environ.setdefault("CUDA_VISIBLE_DEVICES", "")
    configure_torch_backends()

    cfg = make_handmade_velocity_env_cfg(play=True)
    cfg.scene.num_envs = args.num_envs
    cfg.terminations = {}
    env = ManagerBasedRlEnv(cfg=cfg, device=device)
    wrapped = RslRlVecEnvWrapper(env, clip_actions=None)
    agent = make_handmade_rl_cfg()

    try:
        runner = VelocityOnPolicyRunner(
            wrapped,
            asdict(agent),
            str(args.output.expanduser().resolve().parent),
            device,
        )
        checkpoint = args.checkpoint.expanduser().resolve()
        runner.load(str(checkpoint), load_cfg={"actor": True}, strict=True, map_location=device)
        policy = runner.get_inference_policy(device=device)

        _set_forward_command(env, args.vx)
        obs = wrapped.get_observations()
        start_xy = env.scene["robot"].data.root_link_pos_w[:, :2].clone()
        root_z = []
        actual_vx = []
        air_time = []
        contact = []
        steps = round(args.seconds / env.step_dt)
        for _ in range(steps):
            _set_forward_command(env, args.vx)
            with torch.no_grad():
                actions = policy(obs)
            obs, _, _, _ = wrapped.step(actions)
            robot = env.scene["robot"]
            root_z.append(robot.data.root_link_pos_w[:, 2].clone())
            actual_vx.append(robot.data.root_link_lin_vel_b[:, 0].clone())
            air_time.append(m9f_mdp.foot_air_time(env, "feet_ground_contact"))
            contact.append(m9f_mdp.foot_contact(env, "feet_ground_contact"))

        z = torch.stack(root_z)
        displacement = torch.linalg.vector_norm(
            env.scene["robot"].data.root_link_pos_w[:, :2] - start_xy,
            dim=1,
        )
        result = {
            "checkpoint": str(checkpoint),
            "robot_xml": str(HANDMADE_XML),
            "num_envs": args.num_envs,
            "seconds": args.seconds,
            "steps": steps,
            "command_vx": args.vx,
            "root_z_mean": z.mean().item(),
            "root_z_min": z.min().item(),
            "root_z_max": z.max().item(),
            "low_height_fraction": (z < 0.09).float().mean().item(),
            "displacement_mean": displacement.mean().item(),
            "displacement_min": displacement.min().item(),
            "displacement_max": displacement.max().item(),
            "actual_vx_mean": torch.stack(actual_vx).mean().item(),
            "air_time_mean": torch.stack(air_time).mean().item(),
            "contact_fraction": torch.stack(contact).mean().item(),
        }
        output = args.output.expanduser().resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps(result, indent=2), flush=True)
        print("evaluation_ok=1", flush=True)
    finally:
        wrapped.close()


if __name__ == "__main__":
    main()

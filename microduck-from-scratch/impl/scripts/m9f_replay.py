#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import shutil
from dataclasses import asdict
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Replay a checkpoint in the own handmade environment")
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--video-length", type=int, default=250)
    parser.add_argument("--video-width", type=int, default=1280)
    parser.add_argument("--video-height", type=int, default=720)
    parser.add_argument("--vx", type=float, default=0.3)
    parser.add_argument("--no-terminations", action="store_true")
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
    from mjlab.utils.wrappers import VideoRecorder

    from m9f_env_cfg import make_handmade_rl_cfg, make_handmade_velocity_env_cfg
    from m9f_robot import HANDMADE_XML

    device = "cuda:0" if torch.cuda.is_available() else "cpu"
    if device == "cpu":
        os.environ.setdefault("CUDA_VISIBLE_DEVICES", "")
    configure_torch_backends()

    cfg = make_handmade_velocity_env_cfg(play=True)
    cfg.scene.num_envs = 1
    cfg.viewer.width = args.video_width
    cfg.viewer.height = args.video_height
    cfg.viewer.max_extra_envs = 0
    if args.no_terminations:
        cfg.terminations = {}

    checkpoint = args.checkpoint.expanduser().resolve()
    out_dir = args.out_dir.expanduser().resolve()
    work_dir = out_dir / checkpoint.stem
    work_dir.mkdir(parents=True, exist_ok=True)
    env = ManagerBasedRlEnv(cfg=cfg, device=device, render_mode="rgb_array")
    recorded = VideoRecorder(
        env,
        video_folder=work_dir,
        step_trigger=lambda step: step == 0,
        video_length=args.video_length,
        disable_logger=True,
    )
    wrapped = RslRlVecEnvWrapper(recorded, clip_actions=None)

    try:
        runner = VelocityOnPolicyRunner(
            wrapped,
            asdict(make_handmade_rl_cfg()),
            str(out_dir),
            device,
        )
        runner.load(str(checkpoint), load_cfg={"actor": True}, strict=True, map_location=device)
        policy = runner.get_inference_policy(device=device)
        _set_forward_command(env, args.vx)
        obs = wrapped.get_observations()
        for _ in range(args.video_length):
            _set_forward_command(env, args.vx)
            with torch.no_grad():
                actions = policy(obs)
            obs, _, _, _ = wrapped.step(actions)
    finally:
        wrapped.close()

    videos = sorted(work_dir.rglob("*.mp4"))
    if not videos:
        raise RuntimeError(f"no MP4 written under {work_dir}")
    final = out_dir / f"{checkpoint.stem}.mp4"
    shutil.copy2(videos[0], final)
    print(f"robot_xml={HANDMADE_XML}", flush=True)
    print(f"video={final}", flush=True)
    print("replay_ok=1", flush=True)


if __name__ == "__main__":
    main()

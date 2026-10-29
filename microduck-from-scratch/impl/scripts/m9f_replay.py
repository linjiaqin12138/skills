#!/usr/bin/env python3
"""M9f: headless video replay of handmade Velocity checkpoints.

Loads duck_handmade + a list of model_*.pt, steps each for --video-length,
writes mp4 under <log-dir>/videos/replay/. Does not use official `play` CLI
(that path binds robot_walk).

Usage (inside microduck_rl uv env):
  uv run python /path/to/m9f_replay.py --log-dir /path/to/train_run
  uv run python /path/to/m9f_replay.py --checkpoint a.pt --checkpoint b.pt
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import asdict, replace
from pathlib import Path


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    p = argparse.ArgumentParser(description="m9f_replay: handmade checkpoint videos")
    p.add_argument(
        "--log-dir",
        type=str,
        default=None,
        help="Train log dir containing model_*.pt (replays all, sorted by step)",
    )
    p.add_argument(
        "--checkpoint",
        action="append",
        default=[],
        help="Explicit .pt path (repeatable). Overrides --log-dir glob if set.",
    )
    p.add_argument("--num-envs", type=int, default=4, help="Parallel envs for replay")
    p.add_argument(
        "--video-length",
        type=int,
        default=300,  # 300 steps × 0.02s = 6s wall sim time
        help="Env steps to record per checkpoint",
    )
    p.add_argument(
        "--every",
        type=int,
        default=1,
        help="Keep every Nth checkpoint when using --log-dir (default: 1 = all)",
    )
    p.add_argument(
        "--out-dir",
        type=str,
        default=None,
        help="Video folder (default: <log-dir>/videos/replay or ./videos/replay)",
    )
    p.add_argument(
        "--no-terminations",
        action="store_true",
        help="Disable terminations so fallen robots keep rolling for the clip",
    )
    return p.parse_args(argv)


def _ckpt_step(path: Path) -> int:
    m = re.search(r"model_(\d+)\.pt$", path.name)
    return int(m.group(1)) if m else -1


def _collect_checkpoints(args: argparse.Namespace) -> list[Path]:
    if args.checkpoint:
        paths = [Path(c).expanduser().resolve() for c in args.checkpoint]
    elif args.log_dir:
        log_dir = Path(args.log_dir).expanduser().resolve()
        paths = sorted(log_dir.glob("model_*.pt"), key=_ckpt_step)
        if args.every > 1:
            paths = paths[:: args.every]
    else:
        raise SystemExit("need --log-dir or --checkpoint")
    missing = [p for p in paths if not p.is_file()]
    if missing:
        raise SystemExit(f"missing checkpoints: {missing}")
    if not paths:
        raise SystemExit("no checkpoints found")
    return paths


def _replay_one(
    *,
    ckpt: Path,
    out_dir: Path,
    num_envs: int,
    video_length: int,
    no_terminations: bool,
    device: str,
) -> Path:
    from mjlab.envs import ManagerBasedRlEnv
    from mjlab.rl import RslRlVecEnvWrapper
    from mjlab.utils.torch import configure_torch_backends
    from mjlab.utils.wrappers import VideoRecorder
    from mjlab_microduck.tasks import MicroduckOnPolicyRunner
    from mjlab_microduck.tasks.microduck_velocity_env_cfg import (
        MicroduckRlCfg,
        make_microduck_velocity_env_cfg,
    )

    from m9f_handmade_robot import HANDMADE_ROBOT_CFG, HANDMADE_XML

    configure_torch_backends()

    env_cfg = make_microduck_velocity_env_cfg(play=True)
    env_cfg.scene.entities = {"robot": HANDMADE_ROBOT_CFG}
    env_cfg.scene.num_envs = num_envs
    if no_terminations:
        env_cfg.terminations = {}

    agent = replace(MicroduckRlCfg, experiment_name="m9f_handmade_replay", run_name="replay")

    out_dir.mkdir(parents=True, exist_ok=True)
    # One folder per ckpt so VideoRecorder's default naming stays unique.
    ckpt_video_dir = out_dir / ckpt.stem
    ckpt_video_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print(f"m9f_replay | handmade | {ckpt.name}")
    print(f"  xml     : {HANDMADE_XML}")
    print(f"  device  : {device}")
    print(f"  steps   : {video_length}")
    print(f"  out     : {ckpt_video_dir}")
    print("=" * 60, flush=True)

    env = ManagerBasedRlEnv(cfg=env_cfg, device=device, render_mode="rgb_array")
    env = VideoRecorder(
        env,
        video_folder=ckpt_video_dir,
        step_trigger=lambda step: step == 0,
        video_length=video_length,
        disable_logger=True,
    )
    env = RslRlVecEnvWrapper(env, clip_actions=agent.clip_actions)

    # Runner needs a log dir string; use ckpt parent (unused for learn).
    runner = MicroduckOnPolicyRunner(env, asdict(agent), str(ckpt.parent), device)
    runner.load(str(ckpt), load_cfg={"actor": True}, strict=True, map_location=device)
    policy = runner.get_inference_policy(device=device)

    obs = env.get_observations()
    for _ in range(video_length):
        actions = policy(obs)
        obs, _, _, _ = env.step(actions)

    env.close()

    import shutil

    videos = sorted(ckpt_video_dir.rglob("*.mp4"))
    if not videos:
        raise RuntimeError(f"no mp4 written under {ckpt_video_dir}")
    final = out_dir / f"{ckpt.stem}.mp4"
    shutil.copy2(videos[0], final)
    print(f"wrote {final}", flush=True)
    return final


def main(argv: list[str] | None = None) -> None:
    args = parse_args(argv)
    script_dir = Path(__file__).resolve().parent
    if str(script_dir) not in sys.path:
        sys.path.insert(0, str(script_dir))

    import torch

    if torch.cuda.is_available():
        device = "cuda:0"
    else:
        os.environ.setdefault("CUDA_VISIBLE_DEVICES", "")
        device = "cpu"

    ckpts = _collect_checkpoints(args)
    if args.out_dir:
        out_dir = Path(args.out_dir).expanduser().resolve()
    elif args.log_dir:
        out_dir = Path(args.log_dir).expanduser().resolve() / "videos" / "replay"
    else:
        out_dir = Path("videos/replay").resolve()

    print(f"m9f_replay: {len(ckpts)} checkpoint(s) → {out_dir}", flush=True)
    for ckpt in ckpts:
        _replay_one(
            ckpt=ckpt,
            out_dir=out_dir,
            num_envs=args.num_envs,
            video_length=args.video_length,
            no_terminations=args.no_terminations,
            device=device,
        )
    print("replay_ok=1", flush=True)


if __name__ == "__main__":
    main()

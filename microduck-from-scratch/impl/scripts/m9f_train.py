#!/usr/bin/env python3
"""M9f Bite 1: own Velocity train entry on duck_handmade.xml.

Assembles mjlab env + rsl_rl runner via public APIs. Does not wrap the official
train console script (no uv-run-train, no train_cli subprocess).
"""

from __future__ import annotations

import argparse
import os
import sys
from dataclasses import asdict, replace
from datetime import datetime
from pathlib import Path


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="m9f_train: Velocity PPO with handmade MicroDuck body",
    )
    p.add_argument(
        "--num-envs",
        type=int,
        default=64,  # 官方冒烟规模（AGENTS.md: 64 envs × 5 iters）
        help="Parallel envs (default: 64, smoke scale)",
    )
    p.add_argument(
        "--max-iterations",
        type=int,
        default=5,  # 官方冒烟规模；长训由调用方加大
        help="PPO learning iterations (default: 5, smoke scale)",
    )
    p.add_argument(
        "--log-dir",
        type=str,
        default=None,
        help="Run log directory (default: logs/rsl_rl/<experiment>/<timestamp>)",
    )
    p.add_argument(
        "--smoke",
        action="store_true",
        help="Print smoke_ok=1 at end (also auto when max_iterations<=5)",
    )
    p.add_argument(
        "--seed",
        type=int,
        default=None,
        help="Override agent/env seed (default: MicroduckRlCfg.seed)",
    )
    p.add_argument(
        "--save-interval",
        type=int,
        default=50,  # 官方 Velocity 默认 250；中间版本回放需要更密
        help="Checkpoint every N learning iterations (default: 50)",
    )
    return p.parse_args(argv)


def _print_banner(
    *,
    handmade_xml: Path,
    num_envs: int,
    max_iterations: int,
    save_interval: int,
    device: str,
    log_dir: Path,
) -> None:
    print("=" * 60)
    print("m9f_train  |  handmade Velocity (duck_handmade.xml)")
    print("=" * 60)
    print(f"  robot xml     : {handmade_xml}")
    print(f"  num_envs      : {num_envs}")
    print(f"  max_iterations: {max_iterations}")
    print(f"  save_interval : {save_interval}")
    print(f"  device        : {device}")
    print(f"  log_dir       : {log_dir}")
    print("=" * 60, flush=True)


def run_train(args: argparse.Namespace) -> None:
    # Script dir on path so sibling m9f_handmade_robot imports when launched as a file.
    script_dir = Path(__file__).resolve().parent
    if str(script_dir) not in sys.path:
        sys.path.insert(0, str(script_dir))

    import torch
    from mjlab.envs import ManagerBasedRlEnv
    from mjlab.rl import RslRlVecEnvWrapper
    from mjlab.utils.os import dump_yaml
    from mjlab.utils.torch import configure_torch_backends
    from mjlab_microduck.tasks import MicroduckOnPolicyRunner
    from mjlab_microduck.tasks.microduck_velocity_env_cfg import (
        MicroduckRlCfg,
        make_microduck_velocity_env_cfg,
    )

    from m9f_handmade_robot import HANDMADE_ROBOT_CFG, HANDMADE_XML

    if not HANDMADE_XML.is_file():
        raise FileNotFoundError(f"handmade XML missing: {HANDMADE_XML}")

    # Prefer CUDA when present; CPU path matches mjlab train when CUDA_VISIBLE_DEVICES="".
    if torch.cuda.is_available():
        device = "cuda:0"
    else:
        os.environ.setdefault("CUDA_VISIBLE_DEVICES", "")
        device = "cpu"

    configure_torch_backends()

    env_cfg = make_microduck_velocity_env_cfg()
    env_cfg.scene.entities = {"robot": HANDMADE_ROBOT_CFG}
    env_cfg.scene.num_envs = args.num_envs

    if args.save_interval < 1:
        raise SystemExit("--save-interval must be >= 1")

    # MicroduckRlCfg is a module-level instance — replace() so we don't mutate shared state.
    agent = replace(
        MicroduckRlCfg,
        max_iterations=args.max_iterations,
        save_interval=args.save_interval,
        experiment_name="m9f_handmade_velocity",
        run_name="handmade",
    )
    if args.seed is not None:
        agent = replace(agent, seed=args.seed)
    env_cfg.seed = agent.seed

    if args.log_dir:
        log_dir = Path(args.log_dir).expanduser().resolve()
    else:
        stamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
        log_dir = (Path("logs") / "rsl_rl" / agent.experiment_name / stamp).resolve()
    log_dir.mkdir(parents=True, exist_ok=True)

    _print_banner(
        handmade_xml=HANDMADE_XML,
        num_envs=args.num_envs,
        max_iterations=args.max_iterations,
        save_interval=args.save_interval,
        device=device,
        log_dir=log_dir,
    )

    env = ManagerBasedRlEnv(cfg=env_cfg, device=device)
    env = RslRlVecEnvWrapper(env, clip_actions=agent.clip_actions)

    agent_cfg = asdict(agent)
    # Dump before runner ctor: runner may inject non-serializable objects into agent_cfg.
    dump_yaml(log_dir / "params" / "env.yaml", asdict(env_cfg))
    dump_yaml(log_dir / "params" / "agent.yaml", agent_cfg)

    runner = MicroduckOnPolicyRunner(env, agent_cfg, str(log_dir), device)
    runner.add_git_repo_to_log(__file__)
    runner.learn(
        num_learning_iterations=agent.max_iterations,
        init_at_random_ep_len=True,
    )
    env.close()

    is_smoke = args.smoke or args.max_iterations <= 5
    if is_smoke:
        print("smoke_ok=1", flush=True)
    else:
        print("train_finished=1", flush=True)


def main(argv: list[str] | None = None) -> None:
    args = parse_args(argv)
    run_train(args)


if __name__ == "__main__":
    main()

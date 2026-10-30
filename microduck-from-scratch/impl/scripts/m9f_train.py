#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
from dataclasses import asdict, replace
from datetime import datetime
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Train the own handmade MicroDuck velocity task")
    parser.add_argument("--num-envs", type=int, default=64)
    parser.add_argument("--max-iterations", type=int, default=5)
    parser.add_argument("--save-interval", type=int, default=50)
    parser.add_argument("--seed", type=int, default=None)
    parser.add_argument("--log-dir", type=Path, default=None)
    parser.add_argument("--resume-from", type=Path, default=None)
    parser.add_argument("--smoke", action="store_true")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.num_envs < 1 or args.max_iterations < 1 or args.save_interval < 1:
        raise SystemExit("num-envs, max-iterations and save-interval must be positive")

    import torch
    from mjlab.envs import ManagerBasedRlEnv
    from mjlab.rl import RslRlVecEnvWrapper
    from mjlab.tasks.velocity.rl import VelocityOnPolicyRunner
    from mjlab.utils.os import dump_yaml
    from mjlab.utils.torch import configure_torch_backends

    from m9f_env_cfg import make_handmade_rl_cfg, make_handmade_velocity_env_cfg
    from m9f_robot import HANDMADE_XML

    if not HANDMADE_XML.is_file():
        raise FileNotFoundError(HANDMADE_XML)
    device = "cuda:0" if torch.cuda.is_available() else "cpu"
    if device == "cpu":
        os.environ.setdefault("CUDA_VISIBLE_DEVICES", "")
    configure_torch_backends()

    env_cfg = make_handmade_velocity_env_cfg()
    env_cfg.scene.num_envs = args.num_envs
    agent = replace(
        make_handmade_rl_cfg(),
        max_iterations=args.max_iterations,
        save_interval=args.save_interval,
    )
    if args.seed is not None:
        agent = replace(agent, seed=args.seed)
    env_cfg.seed = agent.seed

    if args.log_dir is None:
        stamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
        log_dir = Path("logs") / agent.experiment_name / stamp
    else:
        log_dir = args.log_dir
    log_dir = log_dir.expanduser().resolve()
    log_dir.mkdir(parents=True, exist_ok=True)

    print("m9f_train | own handmade Velocity", flush=True)
    print(f"xml={HANDMADE_XML}", flush=True)
    print(f"num_envs={args.num_envs} max_iterations={args.max_iterations}", flush=True)
    print(f"device={device} log_dir={log_dir}", flush=True)

    env = ManagerBasedRlEnv(cfg=env_cfg, device=device)
    wrapped = RslRlVecEnvWrapper(env, clip_actions=agent.clip_actions)
    dump_yaml(log_dir / "params" / "env.yaml", asdict(env_cfg))
    dump_yaml(log_dir / "params" / "agent.yaml", asdict(agent))

    try:
        runner = VelocityOnPolicyRunner(wrapped, asdict(agent), str(log_dir), device)
        iterations = args.max_iterations
        if args.resume_from is not None:
            checkpoint = args.resume_from.expanduser().resolve()
            runner.load(str(checkpoint), strict=True, map_location=device)
            current = int(getattr(runner, "current_learning_iteration", 0))
            iterations = args.max_iterations - current
            if iterations < 1:
                raise ValueError(f"checkpoint is already at iteration {current}")
            print(f"resumed_from={checkpoint} remaining_iterations={iterations}", flush=True)
        runner.learn(num_learning_iterations=iterations, init_at_random_ep_len=True)
    finally:
        wrapped.close()

    if args.smoke or args.max_iterations <= 5:
        print("smoke_ok=1", flush=True)
    else:
        print("train_finished=1", flush=True)


if __name__ == "__main__":
    main()

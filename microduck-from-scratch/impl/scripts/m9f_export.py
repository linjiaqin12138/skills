#!/usr/bin/env python3
"""Export an own handmade Velocity checkpoint to ONNX (normalizer baked via as_onnx)."""

from __future__ import annotations

import argparse
import json
import os
from dataclasses import asdict
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Export own handmade Velocity policy to ONNX")
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="Destination .onnx path (parent dirs created)",
    )
    parser.add_argument("--num-envs", type=int, default=1)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    import torch
    from mjlab.envs import ManagerBasedRlEnv
    from mjlab.rl import RslRlVecEnvWrapper
    from mjlab.tasks.velocity.rl import VelocityOnPolicyRunner
    from mjlab.utils.torch import configure_torch_backends

    from m9f_env_cfg import make_handmade_rl_cfg, make_handmade_velocity_env_cfg
    from m9f_robot import HANDMADE_XML

    if not HANDMADE_XML.is_file():
        raise FileNotFoundError(HANDMADE_XML)

    checkpoint = args.checkpoint.expanduser().resolve()
    if not checkpoint.is_file():
        raise FileNotFoundError(checkpoint)
    output = args.output.expanduser().resolve()
    output.parent.mkdir(parents=True, exist_ok=True)

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
            str(output.parent),
            device,
        )
        runner.load(str(checkpoint), load_cfg={"actor": True}, strict=True, map_location=device)
        runner.export_policy_to_onnx(str(output.parent), filename=output.name, verbose=False)
        if not output.is_file():
            raise FileNotFoundError(f"export wrote nothing at {output}")

        # Shape probe: actor obs 61 → action 14 (daemon / ONNX Runtime contract).
        import onnxruntime as ort

        session = ort.InferenceSession(str(output), providers=["CPUExecutionProvider"])
        inputs = session.get_inputs()
        outputs = session.get_outputs()
        if len(inputs) != 1 or len(outputs) != 1:
            raise RuntimeError(
                f"expected 1-in/1-out feedforward, got in={len(inputs)} out={len(outputs)}"
            )
        in_shape = list(inputs[0].shape)
        out_shape = list(outputs[0].shape)
        # dynamic batch → treat last dim
        in_dim = int(in_shape[-1])
        out_dim = int(out_shape[-1])
        if in_dim != 61 or out_dim != 14:
            raise RuntimeError(f"bad ONNX dims in={in_shape} out={out_shape}; want 61→14")

        meta = {
            "checkpoint": str(checkpoint),
            "onnx": str(output),
            "robot_xml": str(HANDMADE_XML),
            "input_name": inputs[0].name,
            "output_name": outputs[0].name,
            "input_shape": in_shape,
            "output_shape": out_shape,
            "obs_dim": in_dim,
            "action_dim": out_dim,
        }
        print(json.dumps(meta, indent=2), flush=True)
        print(f"export_ok=1 onnx={output}", flush=True)
    finally:
        wrapped.close()


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Headless stand/walk eval for M9d Bite 2 (cloud, no display).

Same physics path as official scripts/infer_policy.py (BAM + ONNX + PolicyInference),
fixed horizon — stubs mujoco.viewer. Writes JSON for accept/m09d-b02.sh.

From microduck_rl root on the GPU box:

  MUJOCO_GL=egl uv run python /path/to/m9d_infer_headless.py \\
    --walking /path/to/policy.onnx --seconds 10 --lin-vel-x 0.3 \\
    --out /path/to/walk.json
"""
from __future__ import annotations

import argparse
import json
import sys
import time
import types
from pathlib import Path


def _stub_viewer() -> None:
    viewer = types.ModuleType("mujoco.viewer")

    class _Fake:
        def __init__(self, *a, **k):
            self.user_scn = None
            self._running = True

        def is_running(self):
            return self._running

        def sync(self):
            pass

        def __enter__(self):
            return self

        def __exit__(self, *a):
            self._running = False

    viewer.launch_passive = lambda *a, **k: _Fake()
    sys.modules["mujoco.viewer"] = viewer


def _load_infer_policy(repo: Path):
    _stub_viewer()
    import importlib.util

    path = repo / "scripts" / "infer_policy.py"
    spec = importlib.util.spec_from_file_location("infer_policy", path)
    mod = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(mod)
    return mod


def run_trial(
    ip,
    *,
    onnx: Path,
    seconds: float,
    lin_vel_x: float,
    lin_vel_y: float,
    ang_vel_z: float,
    no_bam: bool,
    vin: float,
    vin_drop_gain: float,
    kp_fw: float,
    new_cmd_obs: bool,
) -> dict:
    import mujoco

    xml_path = ip.MICRODUCK_XML
    bam_ctrl = None
    if not no_bam:
        bam_model = ip.load_bam_model(kp_fw, vin, None)
        vin_drop = vin_drop_gain if vin_drop_gain > 0 else None
        model, data, bam_ctrl, _names = ip.load_mujoco_with_bam(
            xml_path, bam_model, timestep=0.005, vin_drop_gain=vin_drop, vin_min=ip.BAM_VIN_MIN
        )
    else:
        model = mujoco.MjModel.from_xml_path(xml_path)
        model.opt.timestep = 0.005
        data = mujoco.MjData(model)

    policy = ip.PolicyInference(
        model,
        data,
        walking_onnx_path=str(onnx),
        bam_ctrl=bam_ctrl,
        use_projected_gravity=True,
        new_cmd_obs=new_cmd_obs,
    )

    freejoint_id = mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_JOINT, "trunk_base_freejoint")
    qpos_adr = int(model.jnt_qposadr[freejoint_id])
    data.qpos[qpos_adr + 0] = 0.0
    data.qpos[qpos_adr + 1] = 0.0
    data.qpos[qpos_adr + 2] = 0.125
    data.qpos[qpos_adr + 3 : qpos_adr + 7] = [1, 0, 0, 0]
    for i, qpos_idx in enumerate(policy.joint_qpos_indices):
        data.qpos[qpos_idx] = policy.default_pose[i]
    if bam_ctrl is not None:
        bam_ctrl.reset(data.qpos)
    policy.set_position_targets(policy.default_pose)
    mujoco.mj_forward(model, data)

    policy.set_vel_cmd(lin_vel_x, lin_vel_y, ang_vel_z)

    obs = policy.get_observations()
    obs_dim = int(obs.shape[-1]) if hasattr(obs, "shape") else len(obs)

    x0 = float(data.qpos[qpos_adr + 0])
    y0 = float(data.qpos[qpos_adr + 1])
    z0 = float(data.qpos[qpos_adr + 2])

    control_dt = 0.02
    decimation = max(1, int(round(control_dt / model.opt.timestep)))
    n_ctrl = int(round(seconds / control_dt))

    zs = []
    t0 = time.time()
    for _ in range(n_ctrl):
        action = policy.infer()
        policy.apply_action(action)
        for _ in range(decimation):
            if bam_ctrl is not None:
                bam_ctrl.update()
            mujoco.mj_step(model, data)
        zs.append(float(data.qpos[qpos_adr + 2]))

    x1 = float(data.qpos[qpos_adr + 0])
    y1 = float(data.qpos[qpos_adr + 1])
    z1 = float(data.qpos[qpos_adr + 2])
    dx = x1 - x0
    dy = y1 - y0
    min_z = float(min(zs)) if zs else z1
    mean_z = float(sum(zs) / len(zs)) if zs else z1
    # Standing MicroDuck trunk ~0.12–0.13 m; near-floor ≈ fallen
    fell = min_z < 0.08

    return {
        "onnx": str(onnx),
        "seconds": seconds,
        "cmd": {"vx": lin_vel_x, "vy": lin_vel_y, "wz": ang_vel_z},
        "no_bam": no_bam,
        "new_cmd_obs": new_cmd_obs,
        "obs_dim": obs_dim,
        "start_xyz": [x0, y0, z0],
        "end_xyz": [x1, y1, z1],
        "dx": dx,
        "dy": dy,
        "forward_m": dx,
        "min_trunk_z": min_z,
        "mean_trunk_z": mean_z,
        "fell": fell,
        "wall_s": time.time() - t0,
        "ctrl_steps": n_ctrl,
    }


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", type=Path, default=Path("."), help="microduck_rl root")
    ap.add_argument("--walking", type=Path, required=True)
    ap.add_argument("--seconds", type=float, default=10.0)
    ap.add_argument("--lin-vel-x", type=float, default=0.0)
    ap.add_argument("--lin-vel-y", type=float, default=0.0)
    ap.add_argument("--ang-vel-z", type=float, default=0.0)
    ap.add_argument("--no-bam", action="store_true")
    ap.add_argument("--vin", type=float, default=7.4)
    ap.add_argument("--vin-drop-gain", type=float, default=0.1)
    ap.add_argument("--kp-fw", type=float, default=200.0)
    ap.add_argument(
        "--legacy-cmd-obs",
        action="store_true",
        help="51D legacy command layout (Velocity 61D needs this OFF)",
    )
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()

    repo = args.repo.resolve()
    ip = _load_infer_policy(repo)
    summary = run_trial(
        ip,
        onnx=args.walking.resolve(),
        seconds=args.seconds,
        lin_vel_x=args.lin_vel_x,
        lin_vel_y=args.lin_vel_y,
        ang_vel_z=args.ang_vel_z,
        no_bam=args.no_bam,
        vin=args.vin,
        vin_drop_gain=args.vin_drop_gain,
        kp_fw=args.kp_fw,
        new_cmd_obs=not args.legacy_cmd_obs,
    )
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

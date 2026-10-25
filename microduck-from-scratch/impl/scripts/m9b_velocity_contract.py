#!/usr/bin/env python3
"""M9b Bite 1：Velocity 配方 obs/action/命令采样契约 ↔ impl/src/obs.rs 静态对账。

不抄官方源码进 impl/；只解析 reference（或 MICRODUCK_RL_ROOT）里的 cfg，
对照我们运行时的 61D 布局常量。不重教 PPO。

用法（cwd 可为 impl/ 或仓库根）：
    MICRODUCK_RL_ROOT=... python impl/scripts/m9b_velocity_contract.py
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

# --- 预期契约（来源：官方 microduck_velocity_env_cfg + 我们 obs.rs 布局）---
EXPECTED_LIN_VEL_X = (-0.4, 0.4)
EXPECTED_LIN_VEL_Y = (-0.3, 0.3)
EXPECTED_ANG_VEL_Z = (-1.0, 1.0)
EXPECTED_TURN_IN_PLACE = 0.15
EXPECTED_ACTION_SCALE = 1.0

EXPECTED_OBS_LEN = 61
EXPECTED_ACTION_LEN = 14
EXPECTED_OFFSETS = {
    "OFF_GYRO": 0,
    "OFF_GRAVITY": 3,
    "OFF_POS": 6,
    "OFF_VEL": 20,
    "OFF_LAST_ACTION": 34,
    "OFF_TWIST": 48,
    "OFF_HEAD": 51,
}

# 61D 块地图：名字对齐训练侧语义，起点对齐 obs.rs 偏移常量。
# 布局：[base_ang_vel(3), projected_gravity(3), joint_pos_rel(14), joint_vel(14),
#        last_action(14)] + [twist(3), head_pose(4), body_pose(6)]
OBS_BLOCKS = (
    ("gyro / base_ang_vel", 0, 3),
    ("gravity / projected_gravity", 3, 3),
    ("pos / joint_pos_rel", 6, 14),
    ("vel / joint_vel", 20, 14),
    ("last_action", 34, 14),
    ("twist", 48, 3),
    ("head / head_pose", 51, 4),
    ("body / body_pose", 55, 6),
)


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def script_paths() -> tuple[Path, Path]:
    """用脚本自身定位 impl/，不依赖 cwd。"""
    scripts_dir = Path(__file__).resolve().parent
    impl_dir = scripts_dir.parent
    obs_rs = impl_dir / "src" / "obs.rs"
    return impl_dir, obs_rs


def rl_cfg_path() -> Path:
    root = os.environ.get("MICRODUCK_RL_ROOT")
    if not root:
        fail("环境变量 MICRODUCK_RL_ROOT 未设置（应指向 microduck_rl 根）")
    cfg = (
        Path(root)
        / "src"
        / "mjlab_microduck"
        / "tasks"
        / "microduck_velocity_env_cfg.py"
    )
    if not cfg.is_file():
        fail(f"找不到官方 velocity cfg：{cfg}")
    return cfg


def parse_float_tuple(text: str, name: str) -> tuple[float, float]:
    m = re.search(
        rf"{re.escape(name)}\s*=\s*\(\s*([+-]?\d+(?:\.\d+)?)\s*,\s*([+-]?\d+(?:\.\d+)?)\s*\)",
        text,
    )
    if not m:
        fail(f"官方 cfg 未解析到赋值：{name} = (lo, hi)")
    return float(m.group(1)), float(m.group(2))


def parse_float_assign(text: str, name: str) -> float:
    m = re.search(rf"{re.escape(name)}\s*=\s*([+-]?\d+(?:\.\d+)?)", text)
    if not m:
        fail(f"官方 cfg 未解析到赋值：{name} = <float>")
    return float(m.group(1))


def parse_action_scale(text: str) -> float:
    # 官方写法：joint_pos_action.scale = 1.0（变量名可能变，故兼认 *.scale =）
    m = re.search(
        r"(?:joint_pos_action|\w+)\.scale\s*=\s*([+-]?\d+(?:\.\d+)?)",
        text,
    )
    if not m:
        fail("官方 cfg 未解析到 joint_pos_action.scale = <float>（或等价 *.scale 赋值）")
    return float(m.group(1))


def check_tuple(actual: tuple[float, float], expected: tuple[float, float], label: str) -> None:
    if actual != expected:
        fail(f"{label}: 得到 {actual}，预期 {expected}")


def check_float(actual: float, expected: float, label: str) -> None:
    if actual != expected:
        fail(f"{label}: 得到 {actual}，预期 {expected}")


def parse_usize_const(text: str, name: str) -> int:
    # 兼容 pub const / const
    m = re.search(
        rf"(?:pub\s+)?const\s+{re.escape(name)}\s*:\s*usize\s*=\s*(\d+)\s*;",
        text,
    )
    if not m:
        fail(f"obs.rs 未找到常量：{name}: usize = <int>")
    return int(m.group(1))


def check_official_cfg(cfg_text: str) -> None:
    lin_x = parse_float_tuple(cfg_text, "command.ranges.lin_vel_x")
    lin_y = parse_float_tuple(cfg_text, "command.ranges.lin_vel_y")
    ang_z = parse_float_tuple(cfg_text, "command.ranges.ang_vel_z")
    tip = parse_float_assign(cfg_text, "TURN_IN_PLACE_FRACTION")
    scale = parse_action_scale(cfg_text)

    check_tuple(lin_x, EXPECTED_LIN_VEL_X, "command.ranges.lin_vel_x")
    check_tuple(lin_y, EXPECTED_LIN_VEL_Y, "command.ranges.lin_vel_y")
    check_tuple(ang_z, EXPECTED_ANG_VEL_Z, "command.ranges.ang_vel_z")
    check_float(tip, EXPECTED_TURN_IN_PLACE, "TURN_IN_PLACE_FRACTION")
    check_float(scale, EXPECTED_ACTION_SCALE, "joint_pos_action.scale")

    print("official velocity cfg OK:")
    print(f"  lin_vel_x = {lin_x}")
    print(f"  lin_vel_y = {lin_y}")
    print(f"  ang_vel_z = {ang_z}")
    print(f"  TURN_IN_PLACE_FRACTION = {tip}")
    print(f"  joint_pos_action.scale = {scale}")


def check_obs_rs(obs_text: str) -> None:
    obs_len = parse_usize_const(obs_text, "OBS_LEN")
    action_len = parse_usize_const(obs_text, "ACTION_LEN")
    if obs_len != EXPECTED_OBS_LEN:
        fail(f"OBS_LEN: 得到 {obs_len}，预期 {EXPECTED_OBS_LEN}")
    if action_len != EXPECTED_ACTION_LEN:
        fail(f"ACTION_LEN: 得到 {action_len}，预期 {EXPECTED_ACTION_LEN}")

    for name, expected in EXPECTED_OFFSETS.items():
        actual = parse_usize_const(obs_text, name)
        if actual != expected:
            fail(f"{name}: 得到 {actual}，预期 {expected}")

    print("impl/src/obs.rs OK:")
    print(f"  OBS_LEN = {obs_len}")
    print(f"  ACTION_LEN = {action_len}")
    for name, expected in EXPECTED_OFFSETS.items():
        print(f"  {name} = {expected}")


def print_block_map() -> None:
    width_sum = sum(w for _, _, w in OBS_BLOCKS)
    print("")
    print("61D obs block map (training-aligned layout):")
    print(f"  {'name':<32} {'start':>5} {'width':>5}")
    print(f"  {'-' * 32} {'-' * 5} {'-' * 5}")
    for name, start, width in OBS_BLOCKS:
        print(f"  {name:<32} {start:>5} {width:>5}")
    print(f"  {'TOTAL':<32} {'':>5} {width_sum:>5}")
    if width_sum != EXPECTED_OBS_LEN:
        fail(f"块宽度和 = {width_sum}，预期 {EXPECTED_OBS_LEN}")

    # 起点必须与 OBS_BLOCKS / OFF_* 一致（防表格手改漂移）
    expected_starts = {
        "gyro / base_ang_vel": EXPECTED_OFFSETS["OFF_GYRO"],
        "gravity / projected_gravity": EXPECTED_OFFSETS["OFF_GRAVITY"],
        "pos / joint_pos_rel": EXPECTED_OFFSETS["OFF_POS"],
        "vel / joint_vel": EXPECTED_OFFSETS["OFF_VEL"],
        "last_action": EXPECTED_OFFSETS["OFF_LAST_ACTION"],
        "twist": EXPECTED_OFFSETS["OFF_TWIST"],
        "head / head_pose": EXPECTED_OFFSETS["OFF_HEAD"],
        "body / body_pose": EXPECTED_OFFSETS["OFF_HEAD"] + 4,  # head 宽 4 → body@55
    }
    for name, start, width in OBS_BLOCKS:
        if start != expected_starts[name]:
            fail(f"块地图起点漂移：{name} start={start}，预期 {expected_starts[name]}")
        # 相邻块必须首尾相接
    for i in range(len(OBS_BLOCKS) - 1):
        name, start, width = OBS_BLOCKS[i]
        next_start = OBS_BLOCKS[i + 1][1]
        if start + width != next_start:
            fail(
                f"块不连续：{name} 结束于 {start + width}，"
                f"下一块起点 {next_start}"
            )


def main() -> None:
    _, obs_rs = script_paths()
    if not obs_rs.is_file():
        fail(f"找不到 obs.rs：{obs_rs}")

    cfg_path = rl_cfg_path()
    cfg_text = cfg_path.read_text(encoding="utf-8")
    obs_text = obs_rs.read_text(encoding="utf-8")

    check_official_cfg(cfg_text)
    check_obs_rs(obs_text)
    print_block_map()
    print("")
    print("m9b_velocity_contract: PASS")


if __name__ == "__main__":
    main()

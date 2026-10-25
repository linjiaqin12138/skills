#!/usr/bin/env python3
"""M9b Bite 2：从官方 Velocity cfg 抽出奖励权重与课程阶梯，并用离散行为袋打分。

不抄官方源进 impl/；只 regex 解析 reference（或 MICRODUCK_RL_ROOT）里的 cfg。
场景是教学代理（分项得分字典），不是物理仿真。

用法（由 accept/m09b-b02.sh 设置 MICRODUCK_RL_ROOT）：
    MICRODUCK_RL_ROOT=... python impl/scripts/m9b_reward_curriculum_map.py
"""

from __future__ import annotations

import math
import os
import re
import sys
from pathlib import Path

TOL = 1e-9

# 断言期望（来源：microduck_velocity_env_cfg.py 奖励段 + action_rate/standing 课程）
EXPECTED_WEIGHTS = {
    "track_linear_velocity": 2.0,
    "track_angular_velocity": 2.0,
    "upright": 2.0,
    "air_time": 3.0,
    "foot_slip": -0.1,
    "action_rate_l2": -0.1,  # cfg 直接赋值初值，非课程最终 -1.0
    "body_ang_vel": -0.05,
    "self_collisions": -1.0,
}
EXPECTED_TRACK_LIN_STD_ARG = 0.1  # params["std"] = math.sqrt(0.1)

# 离散行为袋：键对齐奖励名。
# 正奖励项：score 高更好；负权重 cost 项：score≥0 表示违规量（mjlab cost 习惯）。
# 非物理仿真——只用于演示「改权重谁赢」。
BEHAVIORS: dict[str, dict[str, float]] = {
    "walk": {
        # 跟踪好、直立好、少滑、少抖动（总分高度依赖 upright）
        "track_linear_velocity": 0.70,
        "track_angular_velocity": 0.55,
        "upright": 0.98,
        "air_time": 0.35,
        "foot_slip": 0.05,
        "action_rate_l2": 0.10,
        "body_ang_vel": 0.06,
        "self_collisions": 0.0,
    },
    "roll": {
        # 前进分更高但直立差、身体乱转（滚着蹭位移）
        "track_linear_velocity": 0.98,
        "track_angular_velocity": 0.15,
        "upright": 0.04,
        "air_time": 0.05,
        "foot_slip": 0.90,
        "action_rate_l2": 0.80,
        "body_ang_vel": 2.80,
        "self_collisions": 0.20,
    },
}


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def nearly_eq(a: float, b: float, tol: float = TOL) -> bool:
    return abs(a - b) <= tol


def assert_near(actual: float, expected: float, label: str) -> None:
    if not nearly_eq(actual, expected):
        fail(f"{label}: 得到 {actual!r}，预期 {expected!r}（容差 {TOL}）")


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


def parse_num_steps_per_env(text: str) -> int:
    m = re.search(r"^NUM_STEPS_PER_ENV\s*=\s*(\d+)\s*$", text, re.MULTILINE)
    if not m:
        fail("官方 cfg 未解析到 NUM_STEPS_PER_ENV = <int>")
    return int(m.group(1))


def eval_step_expr(expr: str, num_steps: int) -> int:
    """解析课程 step：纯数字，或 `N * NUM_STEPS_PER_ENV` / `N * 24`。"""
    s = expr.strip().replace(" ", "")
    s = s.replace("NUM_STEPS_PER_ENV", str(num_steps))
    if not re.fullmatch(r"\d+(?:\*\d+)*", s):
        fail(f"无法安全求值 step 表达式：{expr!r}")
    # 仅允许整数乘法链（代理假设：官方只用 N * NUM_STEPS_PER_ENV 或 N * 24）
    parts = [int(p) for p in s.split("*")]
    out = 1
    for p in parts:
        out *= p
    return out


def parse_reward_weight(text: str, name: str) -> float:
    """取 cfg.rewards[\"name\"].weight = X，或 RewardTermCfg(..., weight=X, ...)。"""
    # 直接赋值（多数项）；用 search 取首次——action_rate_l2 初值在课程之前
    m = re.search(
        rf'cfg\.rewards\["{re.escape(name)}"\]\.weight\s*=\s*([+-]?\d+(?:\.\d+)?)',
        text,
    )
    if m:
        return float(m.group(1))

    # RewardTermCfg 构造（如 self_collisions）
    block = re.search(
        rf'cfg\.rewards\["{re.escape(name)}"\]\s*=\s*RewardTermCfg\s*\((.*?)\)',
        text,
        re.DOTALL,
    )
    if block:
        wm = re.search(r"weight\s*=\s*([+-]?\d+(?:\.\d+)?)", block.group(1))
        if wm:
            return float(wm.group(1))

    fail(f'官方 cfg 未解析到奖励权重：{name}')


def parse_track_lin_std(text: str) -> float:
    m = re.search(
        r'cfg\.rewards\["track_linear_velocity"\]\.params\["std"\]\s*=\s*'
        r"math\.sqrt\(([^)]+)\)",
        text,
    )
    if not m:
        fail('官方 cfg 未解析到 track_linear_velocity.params["std"] = math.sqrt(...)')
    arg = float(m.group(1).strip())
    return math.sqrt(arg)


def parse_weight_stages(text: str, curriculum_key: str, num_steps: int) -> list[tuple[int, float]]:
    """抽出 curriculum[key] 的 weight_stages：[(step, weight), ...]。"""
    m = re.search(
        rf'cfg\.curriculum\["{re.escape(curriculum_key)}"\]\s*=\s*CurriculumTermCfg\s*\((.*?)\)\s*',
        text,
        re.DOTALL,
    )
    if not m:
        fail(f'官方 cfg 未解析到 curriculum["{curriculum_key}"]')
    block = m.group(1)
    stages_m = re.search(r'"weight_stages"\s*:\s*\[(.*?)\]', block, re.DOTALL)
    if not stages_m:
        fail(f'curriculum["{curriculum_key}"] 无 weight_stages')
    stages: list[tuple[int, float]] = []
    for sm in re.finditer(
        r'\{\s*"step"\s*:\s*([^,]+?)\s*,\s*"weight"\s*:\s*([+-]?\d+(?:\.\d+)?)\s*\}',
        stages_m.group(1),
    ):
        step = eval_step_expr(sm.group(1), num_steps)
        weight = float(sm.group(2))
        stages.append((step, weight))
    if not stages:
        fail(f'curriculum["{curriculum_key}"] weight_stages 为空')
    return stages


def parse_standing_stages(text: str, num_steps: int) -> list[tuple[int, float]]:
    m = re.search(
        r'cfg\.curriculum\["standing_envs"\]\s*=\s*CurriculumTermCfg\s*\((.*?)\)\s*',
        text,
        re.DOTALL,
    )
    if not m:
        fail('官方 cfg 未解析到 curriculum["standing_envs"]')
    block = m.group(1)
    stages_m = re.search(r'"standing_stages"\s*:\s*\[(.*?)\]', block, re.DOTALL)
    if not stages_m:
        fail('curriculum["standing_envs"] 无 standing_stages')
    stages: list[tuple[int, float]] = []
    for sm in re.finditer(
        r'\{\s*"step"\s*:\s*([^,]+?)\s*,\s*"rel_standing_envs"\s*:\s*([+-]?\d+(?:\.\d+)?)\s*\}',
        stages_m.group(1),
    ):
        step = eval_step_expr(sm.group(1), num_steps)
        rel = float(sm.group(2))
        stages.append((step, rel))
    if not stages:
        fail("standing_stages 为空")
    return stages


def stage_at(stages: list[tuple[int, float]], step: int) -> float:
    """步进函数：取 step <= 目标 的最后一档（与 mdp.reward_weight 习惯一致）。"""
    chosen = None
    for s, v in stages:
        if s <= step:
            chosen = v
        else:
            break
    if chosen is None:
        fail(f"课程无 step≤{step} 的档位：{stages}")
    return chosen


def score_total(weights: dict[str, float], feats: dict[str, float]) -> float:
    return sum(float(weights[k]) * float(feats[k]) for k in weights)


def check_A_rewards(text: str) -> dict[str, float]:
    print("=== A. 奖励权重（从官方 cfg 抽出）===")
    weights: dict[str, float] = {}
    for name, expected in EXPECTED_WEIGHTS.items():
        actual = parse_reward_weight(text, name)
        assert_near(actual, expected, f"rewards[{name!r}].weight")
        weights[name] = actual
        print(f"  {name}.weight = {actual}  OK")

    std = parse_track_lin_std(text)
    assert_near(std, math.sqrt(EXPECTED_TRACK_LIN_STD_ARG), "track_linear_velocity.params.std")
    print(f"  track_linear_velocity.params.std = {std}  (== sqrt(0.1))  OK")
    return weights


def check_B_curriculum(text: str, num_steps: int) -> None:
    print("")
    print("=== B. 课程阶梯 ===")
    print(f"  NUM_STEPS_PER_ENV = {num_steps}")

    ar_stages = parse_weight_stages(text, "action_rate_weight", num_steps)
    w0 = stage_at(ar_stages, 0)
    w_end = stage_at(ar_stages, 1500 * num_steps)
    assert_near(w0, -0.1, "action_rate_weight @ step 0")
    assert_near(w_end, -1.0, f"action_rate_weight @ step 1500*{num_steps}")
    print(f"  action_rate_weight: step 0 → {w0}, step 1500*{num_steps}={1500 * num_steps} → {w_end}  OK")
    # 脚注：smoothness 课后再加码（AGENTS regularizer-timing）
    print("    [AGENTS: regularizer-timing] action_rate 从弱到强，先发现步态再收抖动")

    st_stages = parse_standing_stages(text, num_steps)
    r0 = stage_at(st_stages, 0)
    r_end = stage_at(st_stages, 2000 * num_steps)
    assert_near(r0, 0.02, "standing_envs @ step 0")
    assert_near(r_end, 0.25, f"standing_envs @ step 2000*{num_steps}")
    print(f"  standing_envs: step 0 → {r0}, step 2000*{num_steps}={2000 * num_steps} → {r_end}  OK")


def check_C_scenarios(weights: dict[str, float]) -> None:
    print("")
    print("=== C. 投机场景（离散行为袋，非物理仿真）===")
    print("  约定：总分 = Σ weight_i * score_i；负权重项 score≥0 = 违规量")
    print("  [AGENTS: sign-convention] mjlab cost≥0 → 负权重；勿双重取负")

    for name, feats in BEHAVIORS.items():
        total = score_total(weights, feats)
        parts = {k: round(weights[k] * feats[k], 4) for k in weights}
        print(f"  {name:4s}  总分={total:8.4f}  分项贡献={parts}")

    walk_t = score_total(weights, BEHAVIORS["walk"])
    roll_t = score_total(weights, BEHAVIORS["roll"])
    if not (walk_t > roll_t):
        fail(f"默认权重下期望 walk > roll，得到 walk={walk_t}, roll={roll_t}")
    print(f"  断言 walk ({walk_t:.4f}) > roll ({roll_t:.4f})  OK")
    print("    [AGENTS: letter-of-reward] 默认配方惩罚翻滚蹭位移；优化奖励字面才像在走")

    # 关掉 upright，放大 track_linear → 滚着蹭位移反而更优
    hacked = dict(weights)
    hacked["upright"] = 0.0
    hacked["track_linear_velocity"] = weights["track_linear_velocity"] * 8.0  # 2.0 → 16.0
    walk_h = score_total(hacked, BEHAVIORS["walk"])
    roll_h = score_total(hacked, BEHAVIORS["roll"])
    print(
        f"  黑客权重：upright=0, track_linear={hacked['track_linear_velocity']} "
        f"→ walk={walk_h:.4f}, roll={roll_h:.4f}"
    )
    if not (roll_h > walk_h):
        fail(f"黑客权重下期望 roll > walk，得到 walk={walk_h}, roll={roll_h}")
    print(f"  断言 roll ({roll_h:.4f}) > walk ({walk_h:.4f})  OK")
    print("    [AGENTS: letter-of-reward] 去掉直立约束并放大前进项 → 投机行为反超")


def check_D_std_clue(std: float) -> None:
    print("")
    print("=== D. 「0.3 不走」线索 ===")
    approx = 0.316
    assert_near(std, math.sqrt(0.1), "track_lin std")
    print(f"  track_lin std ≈ {std:.6f}（≈{approx}，√0.1）")
    print(
        "  小指令误差代价近零：|err|≪std 时高斯跟踪几乎不掉分 → "
        "「命令 0.3 看起来不走」的配方侧线索"
    )


def main() -> None:
    cfg_path = rl_cfg_path()
    text = cfg_path.read_text(encoding="utf-8")
    num_steps = parse_num_steps_per_env(text)

    weights = check_A_rewards(text)
    check_B_curriculum(text, num_steps)
    check_C_scenarios(weights)

    std = parse_track_lin_std(text)
    check_D_std_clue(std)

    print("")
    print("PASS 摘要:")
    print("  A 奖励权重 8 项 + track_lin std=√0.1")
    print("  B action_rate_weight 0→-0.1 / 1500*24→-1.0；standing 0→0.02 / 2000*24→0.25")
    print("  C 默认 walk>roll；upright=0+放大 track_lin 后 roll>walk")
    print("  D track_lin std≈0.316 → 小误差近零代价")
    print("m9b_reward_curriculum_map: PASS")


if __name__ == "__main__":
    main()

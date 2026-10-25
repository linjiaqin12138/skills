#!/usr/bin/env python3
"""按 weights.json 给 behaviors.json 里每种行为打总分。"""

from __future__ import annotations

import json
from pathlib import Path

HERE = Path(__file__).resolve().parent


def load(name: str) -> dict:
    with (HERE / name).open(encoding="utf-8") as f:
        return json.load(f)


def score(weights: dict, feats: dict) -> float:
    return sum(float(weights[k]) * float(feats[k]) for k in weights)


def main() -> None:
    weights = load("weights.json")
    behaviors = load("behaviors.json")
    keys = ("track_lin", "upright", "action_rate")
    print("权重:", {k: weights[k] for k in keys})
    print()
    rows = []
    for name, feats in behaviors.items():
        total = score(weights, feats)
        parts = {k: round(float(weights[k]) * float(feats[k]), 4) for k in keys}
        rows.append((total, name, parts))
        print(f"{name:8s}  总分={total:7.3f}  分项={parts}")
    rows.sort(reverse=True)
    print()
    print("最高分行为:", rows[0][1], f"(总分 {rows[0][0]:.3f})")


if __name__ == "__main__":
    main()

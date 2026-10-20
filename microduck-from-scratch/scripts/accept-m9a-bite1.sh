#!/usr/bin/env bash
# M9a Bite 1 验收：自建 MJCF（零 STL 基本几何体）结构/惯量/接触物理。
#
# 用法（宿主机）：
#   bash scripts/accept-m9a-bite1.sh
#   SIM_PYTHON=/opt/sim-venv/bin/python bash scripts/accept-m9a-bite1.sh   # 容器内
#
# 断言组见 sim/accept_m9a_bite1.py docstring（A 结构 / B 惯量 / C 贴地 / D 穿透）。
set -u
cd "$(dirname "$0")/.."

SIM_PYTHON="${SIM_PYTHON:-$HOME/microduck-verify/.venv/bin/python}"

"$SIM_PYTHON" sim/accept_m9a_bite1.py

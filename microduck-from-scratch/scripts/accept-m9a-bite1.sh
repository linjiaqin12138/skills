#!/usr/bin/env bash
# M9a Bite 1 验收：自建 MJCF（碰撞盒+惯量自建，视觉挂官方 STL）结构/惯量/接触物理。
#
# 用法（无人值守验收在容器内跑，宿主机只在没容器时兜底）：
#   docker compose exec rust bash scripts/accept-m9a-bite1.sh
#   SIM_PYTHON=/path/to/python bash scripts/accept-m9a-bite1.sh   # 显式指定解释器
#
# 断言组见 sim/accept_m9a_bite1.py docstring（A 结构 / B 惯量 / C 贴地 / D 穿透）。
set -u
cd "$(dirname "$0")/.."

if [ -z "${SIM_PYTHON:-}" ]; then
  if [ -x /opt/sim-venv/bin/python ]; then          # 容器内
    SIM_PYTHON=/opt/sim-venv/bin/python
  elif [ -x "$PWD/.venv/bin/python" ]; then          # 宿主机兜底（仅供需要 viewer 之外的临时排查）
    SIM_PYTHON="$PWD/.venv/bin/python"
  else
    echo "找不到可用的 python：容器内应带 /opt/sim-venv；宿主机兜底环境见 docs/dev-container.md「宿主机 viewer 环境」" >&2
    exit 1
  fi
fi

"$SIM_PYTHON" sim/accept_m9a_bite1.py

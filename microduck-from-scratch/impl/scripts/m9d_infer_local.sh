#!/usr/bin/env bash
# M9d：本机开官方 viewer 看云上 export 的策略（有显示器时用这条，不用 headless）。
#
# 前置：
#   1. scp 云机 eval/policy_export.onnx → tutor/artifacts/m09d/policy_export.onnx
#   2. vendor/microduck_rl 已 clone 且 uv sync 过
#
# 用法（仓库根）：
#   bash impl/scripts/m9d_infer_local.sh              # 默认行走，键盘控速
#   bash impl/scripts/m9d_infer_local.sh --no-bam     # XML PD 对照
#   LIN_VEL_X=0 bash impl/scripts/m9d_infer_local.sh  # 偏站立（仍可键盘改）
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
RL="${MICRODUCK_RL:-$REPO/vendor/microduck_rl}"
ONNX="${POLICY_ONNX:-$REPO/tutor/artifacts/m09d/policy_export.onnx}"

[ -d "$RL" ] || { echo "缺 $RL —— 先 clone microduck_rl" >&2; exit 1; }
[ -f "$ONNX" ] || { echo "缺 $ONNX —— 先从云机 scp policy_export.onnx" >&2; exit 1; }
[ -n "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ] || {
  echo "无 DISPLAY/WAYLAND_DISPLAY，本机 viewer 开不了；云上请用 m9d_infer_headless.py" >&2
  exit 1
}

export PATH="${HOME}/.local/bin:${PATH}"
export MUJOCO_GL="${MUJOCO_GL:-glfw}"

cd "$RL"
exec uv run scripts/infer_policy.py --walking "$ONNX" --new-cmd-obs "$@"

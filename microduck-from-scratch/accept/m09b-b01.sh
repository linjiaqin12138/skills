#!/usr/bin/env bash
# M9b Bite 1：Velocity 配方地图 — scene/obs/action 契约对账。
#
# 在仓库根执行：bash accept/m09b-b01.sh
# 容器用法：docker exec miniduck-rust bash /work/accept/m09b-b01.sh
#   （工作目录挂载为 /work = microduck-from-scratch）
#
# 预期：
#   A. cargo test obs:: 布局相关单测全绿（OBS_LEN=61、嘴排除、块边界）。
#   B. m9b_velocity_contract.py 从官方 velocity cfg 解析出：
#        twist lin_vel_x=(-0.4,0.4) / lin_vel_y=(-0.3,0.3) / ang_vel_z=(-1.0,1.0)；
#        TURN_IN_PLACE_FRACTION=0.15；action scale=1.0；
#      并与 impl/src/obs.rs 的 OBS_LEN/ACTION_LEN/块偏移契约一致。
#   C. 脚本打印 61D 块地图（gyro/gravity/pos/vel/last_action/twist/head/body），
#      宽度和为 61；与 recon/README 训练侧描述一致。
#   D. （可选）若 MICRODUCK_RL_VENV 或 RL 根下 .venv 可用，跑 CPU pytest 子集
#      test_velstand_cfg 中不 step env 的用例；不可用则跳过并注明，不算失败。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO/impl"

fail() { echo "FAIL: $1" >&2; exit 1; }

PYTHON="${SIM_PYTHON:-python3}"
if [ -x /opt/sim-venv/bin/python ]; then
  PYTHON=/opt/sim-venv/bin/python
fi

echo "== A. obs 布局单测 =="
cargo test obs:: -- --nocapture

echo "== B/C. Velocity 契约对账 =="
RL_ROOT="${MICRODUCK_RL_ROOT:-$REPO/reference/microduck_rl}"
[ -d "$RL_ROOT/src/mjlab_microduck" ] \
  || fail "缺官方 RL 树：$RL_ROOT（设 MICRODUCK_RL_ROOT 或放好 reference/microduck_rl）"

MICRODUCK_RL_ROOT="$RL_ROOT" "$PYTHON" "$REPO/impl/scripts/m9b_velocity_contract.py"

echo "== D. 可选 CPU pytest =="
VENV=""
if [ -n "${MICRODUCK_RL_VENV:-}" ] && [ -x "${MICRODUCK_RL_VENV}/bin/python" ]; then
  VENV="$MICRODUCK_RL_VENV"
elif [ -x "$RL_ROOT/.venv/bin/python" ]; then
  VENV="$RL_ROOT/.venv"
fi
if [ -n "$VENV" ]; then
  (cd "$RL_ROOT" && "$VENV/bin/python" -m pytest tests/test_velstand_cfg.py -q --tb=line) \
    || fail "可选 pytest 已启用但失败"
  echo "optional pytest PASS"
else
  echo "SKIP optional pytest（无 MICRODUCK_RL_VENV / $RL_ROOT/.venv）"
fi

echo "M9b Bite 1 验收 PASS：obs 布局 + Velocity 契约对账全绿"

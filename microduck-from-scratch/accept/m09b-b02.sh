#!/usr/bin/env bash
# M9b Bite 2：Velocity 奖励权重 + 课程阶梯 + 投机场景。
#
# 在仓库根执行：bash accept/m09b-b02.sh
# 容器：docker exec -w /work miniduck-rust bash /work/accept/m09b-b02.sh
#
# 预期：
#   A. 从官方 velocity cfg 抽出并断言关键奖励权重（至少）：
#        track_linear_velocity=2.0, std=sqrt(0.1)；
#        track_angular_velocity=2.0；upright=2.0；
#        air_time=3.0；foot_slip=-0.1；action_rate_l2 初值=-0.1；
#        body_ang_vel=-0.05；self_collisions=-1.0。
#   B. 课程阶梯：action_rate_l2 在 step 0 → -0.1，在 1500*24 → -1.0；
#      standing_envs 在 step 0 → 0.02，在 2000*24 → 0.25。
#   C. 投机场景（离散行为袋，非物理仿真）在抽出权重下：
#        1) 非零前进命令时，平稳跟踪 walk 总分高于「滚着蹭位移」roll；
#        2) 若把 upright 权重改成 0 且加大前进项，roll 反超 walk（呼应先修）；
#        3) 打印每条场景对应的 AGENTS 守则标签（letter-of-reward / regularizer-timing 等）。
#   D. 脚本退出码 0，并打印「0.3 不走」线索：track_lin std≈0.316 与小指令误差代价近零。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO/impl"

fail() { echo "FAIL: $1" >&2; exit 1; }

PYTHON="${SIM_PYTHON:-python3}"
[ -x /opt/sim-venv/bin/python ] && PYTHON=/opt/sim-venv/bin/python

RL_ROOT="${MICRODUCK_RL_ROOT:-$REPO/reference/microduck_rl}"
[ -d "$RL_ROOT/src/mjlab_microduck" ] \
  || fail "缺官方 RL 树：$RL_ROOT"

MICRODUCK_RL_ROOT="$RL_ROOT" "$PYTHON" "$REPO/impl/scripts/m9b_reward_curriculum_map.py"

echo "M9b Bite 2 验收 PASS：奖励表 / 课程阶梯 / 投机场景全绿"

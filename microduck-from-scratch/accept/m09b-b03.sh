#!/usr/bin/env bash
# M9b Bite 3：train / export / manifest 操作判断（无 GPU 真训）。
#
# 在仓库根：bash accept/m09b-b03.sh
# 容器：docker exec -w /work miniduck-rust bash /work/accept/m09b-b03.sh
#
# 预期：
#   A. m9b_ops_quiz.py 全部判断题通过（resume≠warm_start、冒烟 64×5、
#      export 烘 normalizer、manifest [1,61]→[1,14]、schema 2 等；题干在脚本内）。
#   B. 官方 RL 树存在可指认的入口文件：
#        train_cli / export.py / publish/manifest.py（或 scripts/export.py）。
#   C. 不要求 uv sync、不 step env、不跑 GPU。
#   D. 退出码 0。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO/impl"

fail() { echo "FAIL: $1" >&2; exit 1; }

PYTHON="${SIM_PYTHON:-python3}"
[ -x /opt/sim-venv/bin/python ] && PYTHON=/opt/sim-venv/bin/python

RL_ROOT="${MICRODUCK_RL_ROOT:-$REPO/reference/microduck_rl}"
[ -d "$RL_ROOT/src/mjlab_microduck" ] || fail "缺 $RL_ROOT"

MICRODUCK_RL_ROOT="$RL_ROOT" "$PYTHON" "$REPO/impl/scripts/m9b_ops_quiz.py"

echo "M9b Bite 3 验收 PASS：操作判断题 + 入口文件存在性"

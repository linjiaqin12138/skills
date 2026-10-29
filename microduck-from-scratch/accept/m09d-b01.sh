#!/usr/bin/env bash
# M9d Bite 1：云 4090 环境 + 官方冒烟证据门。
#
# 在仓库根：bash accept/m09d-b01.sh
#
# 预期：
#   A. decisions/0008-m9d-cloud-4090.md 存在且采用「4090 + uv 锁版」。
#   B. impl/scripts/m9d_cloud_smoke.sh 存在且可执行意图清晰（64×5）。
#   C. tutor/runs/m09d-b01.txt 含实测证据：RTX 4090、torch 2.9.x、SMOKE PASS / smoke_ok=1。
#   D. 本脚本不 SSH、不烧 GPU；只核本地交付物与跑证。
#   E. 退出码 0。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

fail() { echo "FAIL: $1" >&2; exit 1; }

DEC="$REPO/decisions/0008-m9d-cloud-4090.md"
[ -f "$DEC" ] || fail "缺 $DEC"
grep -q 'RTX 4090' "$DEC" || fail "0008 未写明 4090"
grep -q 'torch==2.9.1' "$DEC" || fail "0008 未钉 torch==2.9.1"
grep -q '备选 1' "$DEC" || fail "0008 未采用备选 1"

SMOKE="$REPO/impl/scripts/m9d_cloud_smoke.sh"
[ -f "$SMOKE" ] || fail "缺 $SMOKE"
grep -q 'num-envs 64' "$SMOKE" || fail "冒烟脚本未钉 64 envs"
grep -q 'max_iterations 5' "$SMOKE" || fail "冒烟脚本未钉 5 iter"
grep -q 'uv sync' "$SMOKE" || fail "冒烟脚本未走 uv sync"

RUN="$REPO/tutor/runs/m09d-b01.txt"
[ -f "$RUN" ] || fail "缺跑证 $RUN（先在云机跑 m9d_cloud_smoke.sh 并把输出写入该文件）"
grep -Eiq 'RTX 4090|GeForce RTX 4090' "$RUN" || fail "跑证未出现 RTX 4090"
grep -Eq 'torch 2\.9' "$RUN" || fail "跑证未出现 torch 2.9.x（勿用镜像 2.1.3）"
grep -Eq 'SMOKE PASS|smoke_ok=1' "$RUN" || fail "跑证无 SMOKE PASS / smoke_ok=1"

echo "M9d Bite 1 验收 PASS：决定 + 冒烟脚本 + 4090/torch2.9 跑证"

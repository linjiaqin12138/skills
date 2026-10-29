#!/usr/bin/env bash
# M9f Bite 1：自写训练入口 + handmade 身体；云冒烟 64×5。
#
# 在仓库根：bash accept/m09f-b01.sh
#
# 预期：
#   A. decisions/0011 存在且指向 m09f-b01。
#   B. impl/scripts/m9f_train.py 存在；源码不出现「uv run train」或
#      「mjlab_microduck.train_cli」作为主训练路径（允许注释里对比提及）。
#   C. 源码显式引用 duck_handmade.xml（或等价 HANDMADE 路径常量）。
#   D. tutor/runs/m09f-b01.txt 含云冒烟证据：
#        - 出现 m9f_train / handmade
#        - num-envs 或 64 与 max_iterations 或 5（冒烟规模）
#        - 退出成功线索（smoke_ok / finished / max_iterations）
#   E. 退出码 0。
#
# 本机无 GPU：脚本只做 A–C 静态门；缺跑证时 FAIL 并提示云上命令。
# 云上跑冒烟（在已 uv sync 的 microduck_rl 工作树）：
#   uv run python /path/to/impl/scripts/m9f_train.py --num-envs 64 --max-iterations 5
#   把终端输出追加进 tutor/runs/m09f-b01.txt，并写 smoke_ok=1
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

fail() { echo "FAIL: $1" >&2; exit 1; }

[ -f "$REPO/decisions/0011-m9f-handmade-own-train.md" ] || fail "缺 decisions/0011"
grep -q 'm09f-b01' "$REPO/decisions/0011-m9f-handmade-own-train.md" || fail "0011 未挂 m09f-b01"

TRAIN="$REPO/impl/scripts/m9f_train.py"
[ -f "$TRAIN" ] || fail "缺 $TRAIN"

# B：禁止把官方 train CLI 当主路径（允许注释；用简单启发式扫非注释行）
python3 - "$TRAIN" <<'PY' || fail "训练脚本仍走官方 train CLI"
from pathlib import Path
import sys
text = Path(sys.argv[1]).read_text()
code_lines = []
for line in text.splitlines():
    s = line.strip()
    if s.startswith("#"):
        continue
    if "#" in line:
        line = line[: line.index("#")]
    code_lines.append(line)
code = "\n".join(code_lines)
forbidden = (
    "uv run train",
    "mjlab_microduck.train_cli",
    "from mjlab_microduck.train_cli",
    "import mjlab_microduck.train_cli",
)
for f in forbidden:
    if f in code:
        raise SystemExit(f"forbidden in non-comment code: {f}")
print("cli_gate_ok")
PY

# C：handmade 路径
grep -q 'duck_handmade' "$TRAIN" || fail "m9f_train.py 未引用 duck_handmade"

[ -f "$REPO/impl/sim/assets/duck_handmade.xml" ] || fail "缺 duck_handmade.xml"

RUN="$REPO/tutor/runs/m09f-b01.txt"
if [ ! -f "$RUN" ]; then
  fail "缺跑证 $RUN —— 云上执行: uv run python impl/scripts/m9f_train.py --num-envs 64 --max-iterations 5 ，输出写入该文件并含 smoke_ok=1"
fi
grep -Eqi 'm9f_train|handmade' "$RUN" || fail "跑证未提 m9f_train/handmade"
grep -Eqi '64|num.envs' "$RUN" || fail "跑证无 64/num_envs 线索"
grep -Eqi '5|max.iter' "$RUN" || fail "跑证无 5/max_iterations 线索"
grep -Eqi 'smoke_ok=1|training complete|Learning iteration 5|max_iterations' "$RUN" \
  || fail "跑证无冒烟成功线索"

echo "M9f Bite 1 验收 PASS：自写入口 + handmade + 云冒烟证据"

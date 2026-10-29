#!/usr/bin/env bash
# M9d Bite 2：官方 export + headless infer 站/走雏形。
#
# 在仓库根：bash accept/m09d-b02.sh
#
# 预期：
#   A. decisions/0009 与 impl/scripts/m9d_infer_headless.py 存在。
#   B. tutor/runs/m09d-b02.txt 含 stand/walk BAM 结果：
#        stand: fell=false 且 min_trunk_z>=0.10
#        walk vx=0.3 BAM: fell=false 且 forward_m>=0.30
#   C. obs_dim=61；有 policy_export / Written onnx 线索。
#   D. 退出码 0。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

fail() { echo "FAIL: $1" >&2; exit 1; }

[ -f "$REPO/decisions/0009-m9d-infer-headless.md" ] || fail "缺 0009"
[ -f "$REPO/impl/scripts/m9d_infer_headless.py" ] || fail "缺 m9d_infer_headless.py"

RUN="$REPO/tutor/runs/m09d-b02.txt"
[ -f "$RUN" ] || fail "缺跑证 $RUN"
grep -q '"obs_dim": 61' "$RUN" || fail "跑证无 obs_dim=61"
grep -Eqi 'policy_export\.onnx|Written' "$RUN" || fail "跑证无 export onnx 线索"

python3 - "$RUN" <<'PY'
import json, sys
from pathlib import Path
text = Path(sys.argv[1]).read_text()
objs = []
i = 0
while True:
    j = text.find("{", i)
    if j < 0:
        break
    depth = 0
    for k in range(j, len(text)):
        if text[k] == "{":
            depth += 1
        elif text[k] == "}":
            depth -= 1
            if depth == 0:
                chunk = text[j : k + 1]
                if '"fell"' in chunk and '"cmd"' in chunk:
                    try:
                        objs.append(json.loads(chunk))
                    except json.JSONDecodeError:
                        pass
                i = k + 1
                break
    else:
        break

stands = [o for o in objs if abs(float(o["cmd"]["vx"])) < 1e-9 and not o.get("no_bam")]
walks = [o for o in objs if abs(float(o["cmd"]["vx"]) - 0.3) < 1e-9 and not o.get("no_bam")]
if not stands:
    raise SystemExit("FAIL: 无 stand BAM JSON")
if not walks:
    raise SystemExit("FAIL: 无 walk BAM vx=0.3 JSON")
s, w = stands[0], walks[0]
if s.get("fell"):
    raise SystemExit("FAIL: stand fell")
if float(s.get("min_trunk_z", 0)) < 0.10:
    raise SystemExit(f"FAIL: stand min_trunk_z={s.get('min_trunk_z')}")
if w.get("fell"):
    raise SystemExit("FAIL: walk fell")
if float(w.get("forward_m", 0)) < 0.30:
    raise SystemExit(f"FAIL: walk forward_m={w.get('forward_m')} < 0.30")
print("gates_ok", "stand_z", s["min_trunk_z"], "walk_dx", w["forward_m"])
PY

echo "M9d Bite 2 验收 PASS：export + headless 站/走雏形"

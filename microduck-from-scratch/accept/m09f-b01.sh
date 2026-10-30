#!/usr/bin/env bash
# M9f Bite 1：自有 handmade Velocity 环境 + GPU 从头训练。
#
# 在仓库根：
#   bash accept/m09f-b01.sh --static-only   # 本地静态门
#   bash accept/m09f-b01.sh                 # 完整门（需 GPU 跑证与最终评估）
#
# 预期：
#   A. 新决定 0012 存在且挂 m09f-b01；六个 m9f 模块齐全且语法有效。
#   B. 主路径无 mjlab_microduck import、无 uv run train；训练/评估/回放共用自有 env。
#   C. handmade XML、61D obs、14D action、scale=1.0 与四脚底 geom 配置显式存在。
#   D. 完整门要求 tutor/runs/m09f-b01.txt 含 baseline_ok=1、smoke_ok=1、
#      short_train_ok=1、train_finished=1。
#   E. tutor/artifacts/m09f/evaluation.json：5 秒前进评估，root_z_mean 在
#      [0.105, 0.135]，displacement_mean >= 0.5m，air_time_mean <= 1.5；HD 视频存在。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

fail() { echo "FAIL: $1" >&2; exit 1; }

[ -f decisions/0012-m9f-own-velocity-task.md ] || fail "缺 decisions/0012"
grep -q 'm09f-b01' decisions/0012-m9f-own-velocity-task.md || fail "0012 未挂 m09f-b01"

python3 - "$REPO" <<'PY'
import ast
from pathlib import Path
import sys

repo = Path(sys.argv[1])
scripts = repo / "impl" / "scripts"
names = (
    "m9f_robot.py",
    "m9f_env_cfg.py",
    "m9f_mdp.py",
    "m9f_train.py",
    "m9f_evaluate.py",
    "m9f_replay.py",
)
texts = {}
for name in names:
    path = scripts / name
    if not path.is_file():
        raise SystemExit(f"missing {path}")
    text = path.read_text()
    texts[name] = text
    tree = ast.parse(text, filename=str(path))
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imported = [alias.name for alias in node.names]
        elif isinstance(node, ast.ImportFrom):
            imported = [node.module or ""]
        else:
            continue
        if any(module == "mjlab_microduck" or module.startswith("mjlab_microduck.") for module in imported):
            raise SystemExit(f"official MicroDuck import in {name}:{node.lineno}")
    if "uv run train" in text:
        raise SystemExit(f"official train CLI in {name}")

robot = texts["m9f_robot.py"]
for token in ("duck_handmade", "left_foot_collision", "right_foot_collision", "_part2", "BamActuatorCfg"):
    if token not in robot:
        raise SystemExit(f"robot config missing {token}")

env = texts["m9f_env_cfg.py"]
for token in ("61", "14", "scale", "feet_ground_contact", "left_foot", "right_foot"):
    if token not in env:
        raise SystemExit(f"env config missing {token}")

for name in ("m9f_train.py", "m9f_evaluate.py", "m9f_replay.py"):
    if "m9f_env_cfg" not in texts[name] or "m9f_robot" not in texts[name]:
        raise SystemExit(f"{name} does not use the shared own env/robot modules")

print("own_source_gate_ok")
print("handmade_contact_gate_ok")
print("python_syntax_gate_ok")
PY

[ -f impl/sim/assets/duck_handmade.xml ] || fail "缺 handmade XML"

if [ "${1:-}" = "--static-only" ]; then
  echo "M9f Bite 1 静态验收 PASS"
  exit 0
fi

RUN=tutor/runs/m09f-b01.txt
[ -f "$RUN" ] || fail "缺 GPU 跑证 $RUN"
for marker in baseline_ok=1 smoke_ok=1 short_train_ok=1 train_finished=1; do
  grep -q "$marker" "$RUN" || fail "跑证缺 $marker"
done

python3 - "$REPO" <<'PY'
import json
import math
from pathlib import Path
import sys

repo = Path(sys.argv[1])
path = repo / "tutor" / "artifacts" / "m09f" / "evaluation.json"
if not path.is_file():
    raise SystemExit(f"missing {path}")
data = json.loads(path.read_text())
for key in ("root_z_mean", "displacement_mean", "air_time_mean"):
    value = float(data[key])
    if not math.isfinite(value):
        raise SystemExit(f"non-finite {key}")
if not 0.105 <= float(data["root_z_mean"]) <= 0.135:
    raise SystemExit(f"root_z_mean outside gate: {data['root_z_mean']}")
if float(data["displacement_mean"]) < 0.5:
    raise SystemExit(f"displacement_mean below gate: {data['displacement_mean']}")
if float(data["air_time_mean"]) > 1.5:
    raise SystemExit(f"air_time_mean above gate: {data['air_time_mean']}")
videos = list((repo / "tutor" / "artifacts" / "m09f" / "videos").glob("*.mp4"))
if not videos:
    raise SystemExit("missing HD replay mp4")
print("gait_check_ok=1")
PY

echo "M9f Bite 1 验收 PASS：自有环境 + GPU 训练 + 定量步态"

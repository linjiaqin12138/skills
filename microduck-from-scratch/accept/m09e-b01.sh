#!/usr/bin/env bash
# M9e Bite 1：自训 ONNX 进 miniduckd + handmade BAM 闭环（站 + vx=0.3 走雏形）。
#
# 在仓库根：bash accept/m09e-b01.sh
# 容器：docker compose -f impl/docker-compose.yml exec rust bash /work/accept/m09e-b01.sh
#
# 预期：
#   A. decisions/0010 存在；tutor/artifacts/m09d/policy_export.onnx 存在。
#   B. MINIDUCK_POLICY 指向该 ONNX，MINIDUCK_WALK_ACTION_SCALE=1.0；
#      默认 BAM 场景仍是 scene_handmade.xml。
#   C. 站立：enable 后 z>0.08、skill=walk、fallen=false。
#   D. vx=0.3 ×10s：未倒（fallen=false），躯干前移 forward_m >= 0.30
#      （与 M9d headless 雏形同门；对照公开 velstand 在 0.3 几乎不走）。
#   E. 单测或实现能证明 walk scale 缺省 0.9、环境变量可改为 1.0。
#   F. 退出码 0。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO/impl"

SIM_PYTHON="${SIM_PYTHON:-/opt/sim-venv/bin/python}"
SIM_PORT=7810
SIM_ADDR=127.0.0.1:$SIM_PORT
SIM_PID=""
DAEMON_PID=""
ONNX="$REPO/tutor/artifacts/m09d/policy_export.onnx"

cleanup() {
    if [ -n "${DAEMON_PID:-}" ]; then kill "$DAEMON_PID" 2>/dev/null || true; wait "$DAEMON_PID" 2>/dev/null || true; fi
    if [ -n "${SIM_PID:-}" ]; then kill "$SIM_PID" 2>/dev/null || true; wait "$SIM_PID" 2>/dev/null || true; fi
    DAEMON_PID=""
    SIM_PID=""
}
trap cleanup EXIT

fail() { echo "FAIL: $1" >&2; exit 1; }

[ -f "$REPO/decisions/0010-m9e-walk-action-scale.md" ] || fail "缺 decisions/0010"
[ -f "$ONNX" ] || fail "缺 $ONNX（先从云机 scp policy_export.onnx）"
[ -x "$SIM_PYTHON" ] || fail "缺 $SIM_PYTHON；验收须在带 /opt/sim-venv 的容器内运行"
[ -f third_party/onnxruntime/lib/libonnxruntime.so ] || fail "缺 ONNX Runtime；先 bash scripts/fetch-m3.sh"

# E：scale 旋钮（实现侧单测；这里只要求二进制能起来且环境被读——详细断言在 cargo test）
cargo test --lib walk_action_scale -- --nocapture
cargo build --bins

rm -f /tmp/miniduckd.sock /tmp/m9e-b01-state.json
env MUJOCO_GL=disable "$SIM_PYTHON" sim/duck_body.py --port "$SIM_PORT" \
    >/tmp/m9e-b01-sim.out 2>/tmp/m9e-b01-sim.log &
SIM_PID=$!
sleep 2
kill -0 "$SIM_PID" 2>/dev/null || fail "sim 起不来：/tmp/m9e-b01-sim.log"
grep -q 'scene=.*scene_handmade.xml' /tmp/m9e-b01-sim.log \
    || fail "默认 BAM 场景不是 scene_handmade.xml"

env MINIDUCK_POLICY="$ONNX" MINIDUCK_WALK_ACTION_SCALE=1.0 \
    ./target/debug/miniduckd --sim "$SIM_ADDR" \
    >/tmp/m9e-b01-daemon.out 2>/tmp/m9e-b01-daemon.log &
DAEMON_PID=$!
sleep 2
kill -0 "$DAEMON_PID" 2>/dev/null || fail "daemon 起不来：/tmp/m9e-b01-daemon.log"
grep -qiE 'failed to load policy|policy unavailable' /tmp/m9e-b01-daemon.log \
    && fail "策略加载失败：/tmp/m9e-b01-daemon.log"

./target/debug/mini-duckctl enable >/dev/null

body() {
    SIM_PORT="$SIM_PORT" "$SIM_PYTHON" - <<'PY'
import json, os, socket
s = socket.create_connection(("127.0.0.1", int(os.environ["SIM_PORT"])), timeout=5)
f = s.makefile("rw")
f.write(json.dumps({"op": "body"}) + "\n"); f.flush()
r = json.loads(f.readline())
p = r["body_pos"]
print(p[0], p[1], p[2], r["sim_time"])
PY
}

state_snapshot() {
    timeout 3 ./target/debug/mini-duckctl state --every 1 2>/dev/null \
        | grep '"method":"robot.state"' | head -1 > /tmp/m9e-b01-state.json || true
    [ -s /tmp/m9e-b01-state.json ] || { echo "? ?"; return; }
    "$SIM_PYTHON" -c '
import json
p=json.load(open("/tmp/m9e-b01-state.json"))["params"]
print(p.get("skill"), p.get("fallen"))
' 2>/dev/null || echo "? ?"
}

wait_stand() {
    local z="0" info="? ?" i
    for i in $(seq 1 30); do
        z=$(body | awk '{print $3}')
        info=$(state_snapshot)
        if awk -v z="$z" 'BEGIN { exit !(z > 0.08) }' \
            && [ "$info" = "walk False" ]; then
            echo "$z"
            return
        fi
        sleep 0.5
    done
    fail "未站稳：z=$z skill/fallen=$info"
}

STAND_Z=$(wait_stand)
echo "stand_ok z=$STAND_Z"

X0=$(body | awk '{print $1}')
./target/debug/mini-duckctl move 0.3 0 0 --secs 10 >/dev/null || fail "move"
X1=$(body | awk '{print $1}')
Z1=$(body | awk '{print $3}')
INFO=$(state_snapshot)
FORWARD=$("$SIM_PYTHON" -c "print(float('$X1')-float('$X0'))")

echo "walk vx=0.3: x0=$X0 x1=$X1 forward=$FORWARD z=$Z1 skill/fallen=$INFO"

awk -v f="$FORWARD" 'BEGIN { exit !(f >= 0.30) }' \
    || fail "forward_m=$FORWARD < 0.30（自训 0.3 雏形未过）"
awk -v z="$Z1" 'BEGIN { exit !(z > 0.08) }' || fail "走后 z=$Z1 过低"
[ "$INFO" = "walk False" ] || fail "走后 skill/fallen=$INFO（期望 walk False）"

echo "M9e Bite 1 验收 PASS：自训 ONNX 闭环站 + vx=0.3 走雏形 forward=$FORWARD"

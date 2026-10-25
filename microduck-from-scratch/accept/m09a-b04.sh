#!/usr/bin/env bash
# M9a Bite 4：D4 sim 协议面对齐官方 body_server / duck-control sim.rs。
#
# 在仓库根执行：bash accept/m09a-b04.sh
# 容器用法：docker compose -p microduck-from-scratch -f impl/docker-compose.yml exec rust bash /work/accept/m09a-b04.sh
#
# 预期：
#   A. hello：protocol=1 握手成功；错误 protocol 被拒绝。
#   B. 线上 op 名是 gain（字段 kp）与 torque（字段 on）；旧名 set_gain/set_torque 必须失败。
#   C. write/read 走 15 槽总线，嘴在 index 9；直立时 imu.gravity ≈ [0,0,-1]（trunk 系投影）。
#   D. 教学附加 op push/body 仍可用（不在官方协议内，但本仓验收依赖）。
#   E. cargo test 全绿（含 SimIo 帧断言改到 gain/torque）。
#   F. 冒烟闭环：BAM 默认 scene_handmade 起 sim+daemon，enable 后 2s 内 skill=walk 且 fallen=false。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO/impl"

SIM_PYTHON="${SIM_PYTHON:-/opt/sim-venv/bin/python}"
SIM_PORT=7810
SIM_ADDR=127.0.0.1:$SIM_PORT
SIM_PID=""
DAEMON_PID=""

cleanup() {
    if [ -n "${DAEMON_PID:-}" ]; then kill "$DAEMON_PID" 2>/dev/null || true; wait "$DAEMON_PID" 2>/dev/null || true; fi
    if [ -n "${SIM_PID:-}" ]; then kill "$SIM_PID" 2>/dev/null || true; wait "$SIM_PID" 2>/dev/null || true; fi
    DAEMON_PID=""
    SIM_PID=""
}
trap cleanup EXIT

fail() { echo "FAIL: $1" >&2; exit 1; }

[ -x "$SIM_PYTHON" ] || fail "缺 $SIM_PYTHON；验收须在带 /opt/sim-venv 的容器内运行"
[ -f policies/velstand.onnx ] || fail "缺 policies/velstand.onnx；先运行 bash scripts/fetch-m3.sh"
[ -f third_party/onnxruntime/lib/libonnxruntime.so ] || fail "缺 ONNX Runtime；先运行 bash scripts/fetch-m3.sh"

cargo build --bins
cargo test

echo "== A/B/C/D. 线协议断言 =="
cleanup
rm -f /tmp/miniduckd.sock
env MUJOCO_GL=disable "$SIM_PYTHON" sim/duck_body.py --port "$SIM_PORT" \
    >/tmp/m09a-b04-sim.out 2>/tmp/m09a-b04-sim.log &
SIM_PID=$!
sleep 2
kill -0 "$SIM_PID" 2>/dev/null || fail "sim 起不来：/tmp/m09a-b04-sim.log"

SIM_PORT="$SIM_PORT" "$SIM_PYTHON" - <<'PY'
import json, os, socket, sys

port = int(os.environ["SIM_PORT"])

def call(req):
    s = socket.create_connection(("127.0.0.1", port), timeout=5)
    f = s.makefile("rw")
    f.write(json.dumps(req) + "\n"); f.flush()
    line = f.readline()
    if not line:
        raise SystemExit("empty reply for " + json.dumps(req))
    return json.loads(line)

# A. PROTOCOL=1
hello = call({"op": "hello", "protocol": 1, "joints": 15})
assert hello.get("protocol") == 1, hello
bad = call({"op": "hello", "protocol": 99, "joints": 15})
assert "error" in bad, bad

# B. 官方 op 名；旧名必须失败
ok_gain = call({"op": "gain", "kp": 50})
assert "error" not in ok_gain, ok_gain
ok_torque = call({"op": "torque", "on": True})
assert "error" not in ok_torque, ok_torque
old_g = call({"op": "set_gain", "gain": 50})
assert "error" in old_g, old_g
old_t = call({"op": "set_torque", "on": False})
assert "error" in old_t, old_t

# C. 15 槽 + 嘴位 + trunk 重力
targets = [0.0] * 15
targets[9] = -0.087  # mouth
w = call({"op": "write", "targets": targets})
assert "error" not in w, w
r = call({"op": "read"})
assert "error" not in r, r
assert len(r["positions"]) == 15, len(r["positions"])
assert len(r["velocities"]) == 15, len(r["velocities"])
# 嘴不是执行器：读回应保持总线槽位长度，嘴值由服务端插入（常见为 0）
assert abs(r["positions"][9]) < 1.0, r["positions"][9]
g = r["imu"]["gravity"]
assert abs(g[0]) < 0.15 and abs(g[1]) < 0.15 and g[2] < -0.85, g

# D. 教学附加帧
body = call({"op": "body"})
assert "body_pos" in body and "sim_time" in body, body
pushed = call({"op": "push", "vx": 0.0, "vy": 0.0})
assert "error" not in pushed, pushed
print("protocol asserts PASS")
PY

echo "== F. 冒烟闭环 =="
cleanup
rm -f /tmp/miniduckd.sock /tmp/m09a-b04-state.json
env MUJOCO_GL=disable "$SIM_PYTHON" sim/duck_body.py --port "$SIM_PORT" \
    >/tmp/m09a-b04-sim2.out 2>/tmp/m09a-b04-sim2.log &
SIM_PID=$!
sleep 2
kill -0 "$SIM_PID" 2>/dev/null || fail "sim 起不来：/tmp/m09a-b04-sim2.log"
grep -q 'scene=.*scene_handmade.xml' /tmp/m09a-b04-sim2.log \
    || fail "默认 BAM 场景不是 scene_handmade.xml"

./target/debug/miniduckd --sim "$SIM_ADDR" \
    >/tmp/m09a-b04-daemon.out 2>/tmp/m09a-b04-daemon.log &
DAEMON_PID=$!
for _ in $(seq 1 50); do
    [ -S /tmp/miniduckd.sock ] && break
    sleep 0.1
done
[ -S /tmp/miniduckd.sock ] || fail "daemon socket 未出现：/tmp/m09a-b04-daemon.log"

./target/debug/mini-duckctl enable on >/tmp/m09a-b04-enable.out 2>&1 || fail "enable 失败"
sleep 2
timeout 3 ./target/debug/mini-duckctl state --every 1 2>/dev/null \
    | grep '"method":"robot.state"' | head -1 > /tmp/m09a-b04-state.json || true
[ -s /tmp/m09a-b04-state.json ] || fail "读不到 robot.state"
"$SIM_PYTHON" - <<'PY'
import json
p = json.load(open("/tmp/m09a-b04-state.json"))["params"]
assert p.get("skill") == "walk", p
assert p.get("fallen") is False, p
print("smoke PASS", p.get("skill"), "fallen=", p.get("fallen"))
PY

echo "M9a Bite 4 验收 PASS：D4 协议面 gain/torque + PROTOCOL/15 槽/重力 + 冒烟闭环全绿"

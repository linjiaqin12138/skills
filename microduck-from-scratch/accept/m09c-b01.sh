#!/usr/bin/env bash
# M9c Bite 1：FallPredictor + Limp/Posing 跌倒反射。
#
# 在仓库根执行：bash accept/m09c-b01.sh
# 容器用法：docker compose -f impl/docker-compose.yml exec rust bash /work/accept/m09c-b01.sh
#
# 预期：
#   A. 全部 Rust 单测通过；新增覆盖至少包括：解析重力 z 导数、落脚/静态倾斜/慢倾不触发、
#      快速倾倒第 3 拍只触发一次、恢复与 reset 重新武装、落地静止去抖、盲拍不算静止、
#      Posing 0/中点/终点、busy/sitting/非新鲜样本清预测去抖、disable 中断反射。
#   B. accept/m08-b01.sh 全回归通过；现有 robot.do、homed、safeToRestart 行为不回退。
#   C. 默认 MINIDUCK_LIMP_FALL 未设时与上游一致为 OFF：sim push 后 fallen 仍如实报告，
#      但不出现 gain=50/160 或 limp_fall/limp_pose，策略不被反射抢占。
#   D. MINIDUCK_LIMP_FALL=1 时做 2 次全新 sim/daemon 闭环；每次 push 后按序观察：
#      limp_fall+gain50 → limp_pose+gain160 → walk+gain200；Limp 不超过 1500ms，
#      Posing 为 600ms；最后恢复策略接管。不要求 velstand 从躺平物理站起。
#   E. 两次样本都打印阶段时序；不把单次成功当可靠性证据。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO/impl"

SIM_PYTHON="${SIM_PYTHON:-/opt/sim-venv/bin/python}"
SIM_PORT=7811
SIM_ADDR=127.0.0.1:$SIM_PORT
SIM_PID=""
DAEMON_PID=""

cleanup() {
    if [ -n "$DAEMON_PID" ]; then kill "$DAEMON_PID" 2>/dev/null || true; wait "$DAEMON_PID" 2>/dev/null || true; fi
    if [ -n "$SIM_PID" ]; then kill "$SIM_PID" 2>/dev/null || true; wait "$SIM_PID" 2>/dev/null || true; fi
    DAEMON_PID=""
    SIM_PID=""
}
trap cleanup EXIT
fail() { echo "FAIL: $1" >&2; exit 1; }

[ -x "$SIM_PYTHON" ] || fail "缺 $SIM_PYTHON；验收须在带 /opt/sim-venv 的容器内运行"
[ -f policies/velstand.onnx ] || fail "缺 policies/velstand.onnx；先运行 bash scripts/fetch-m3.sh"
[ -f third_party/onnxruntime/lib/libonnxruntime.so ] || fail "缺 ONNX Runtime；先运行 bash scripts/fetch-m3.sh"

echo "===== A. Rust 单测 ====="
cargo test

echo "===== B. M8 接口与状态机回归 ====="
bash "$REPO/accept/m08-b01.sh"
cleanup
sleep 1

start_case() {
    local name="$1" limp="$2"
    cleanup
    rm -f /tmp/miniduckd.sock
    env MUJOCO_GL=disable "$SIM_PYTHON" sim/duck_body.py --port "$SIM_PORT" \
        >"/tmp/m9c-${name}-sim.out" 2>"/tmp/m9c-${name}-sim.log" &
    SIM_PID=$!
    sleep 2
    kill -0 "$SIM_PID" 2>/dev/null || fail "$name sim 起不来：/tmp/m9c-${name}-sim.log"
    if [ "$limp" = 1 ]; then
        MINIDUCK_LIMP_FALL=1 ./target/debug/miniduckd --sim "$SIM_ADDR" \
            >"/tmp/m9c-${name}-daemon.out" 2>"/tmp/m9c-${name}-daemon.log" &
    else
        ./target/debug/miniduckd --sim "$SIM_ADDR" \
            >"/tmp/m9c-${name}-daemon.out" 2>"/tmp/m9c-${name}-daemon.log" &
    fi
    DAEMON_PID=$!
    sleep 2
    kill -0 "$DAEMON_PID" 2>/dev/null || fail "$name daemon 起不来：/tmp/m9c-${name}-daemon.log"
    ./target/debug/mini-duckctl enable >/dev/null
    sleep 1
}

run_case() {
    local name="$1" expected="$2"
    start_case "$name" "$expected"
    CASE_NAME="$name" EXPECT_LIMP="$expected" SIM_PORT="$SIM_PORT" "$SIM_PYTHON" - <<'PY'
import json, os, socket, time

name = os.environ["CASE_NAME"]
expected = os.environ["EXPECT_LIMP"] == "1"
port = int(os.environ["SIM_PORT"])

def tcp(obj):
    s = socket.create_connection(("127.0.0.1", port), timeout=5)
    f = s.makefile("rw")
    f.write(json.dumps(obj) + "\n"); f.flush()
    reply = json.loads(f.readline())
    s.close()
    return reply

s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.settimeout(6)
s.connect("/tmp/miniduckd.sock")
f = s.makefile("rw")

def send(obj):
    f.write(json.dumps(obj) + "\n"); f.flush()

def response(rid):
    while True:
        msg = json.loads(f.readline())
        if msg.get("id") == rid:
            assert "result" in msg, msg
            return msg["result"]

send({"jsonrpc":"2.0","id":1,"method":"hello"}); response(1)
send({"jsonrpc":"2.0","id":2,"method":"robot.subscribe","params":{}}); response(2)
tcp({"op":"push","vx":4.0,"vy":0.0})
start = time.monotonic()
frames = []
while time.monotonic() - start < 4.5:
    msg = json.loads(f.readline())
    assert msg.get("method") == "robot.state", msg
    p = msg["params"]
    frames.append((time.monotonic()-start, p.get("skill"), p.get("gain"), p["fallen"]))
s.close()

assert any(f[3] for f in frames), f"{name}: push 后从未报告 fallen"
sequence = []
for t, skill, gain, fallen in frames:
    item = (skill, gain)
    if not sequence or sequence[-1][1] != item:
        sequence.append((t, item))
print(f"{name}: frames={len(frames)} transitions=" + ", ".join(
    f"{t:.2f}s:{skill}/g{gain}" for t, (skill, gain) in sequence))

if not expected:
    assert not any(g in (50,160) or sk in ("limp_fall","limp_pose") for _,sk,g,_ in frames), \
        f"{name}: 默认 OFF 却进入跌倒反射"
    assert all(sk == "walk" for _,sk,_,_ in frames), f"{name}: OFF 时策略被抢占"
    print(f"{name}: 默认 OFF，fallen 只报告不门控  OK")
else:
    wanted = [("limp_fall",50),("limp_pose",160),("walk",200)]
    positions = []
    for target in wanted:
        for i, (_, pair) in enumerate(sequence):
            if pair == target and (not positions or i > positions[-1]):
                positions.append(i)
                break
        else:
            raise AssertionError(f"{name}: 缺阶段或顺序错误 {target}: {sequence}")
    limp_start = sequence[positions[0]][0]
    pose_start = sequence[positions[1]][0]
    walk_start = sequence[positions[2]][0]
    limp_ms = (pose_start-limp_start)*1000
    pose_ms = (walk_start-pose_start)*1000
    assert limp_ms <= 1650, f"{name}: Limp 超 1500ms 上限: {limp_ms:.0f}ms"
    assert 450 <= pose_ms <= 850, f"{name}: Posing 应约 600ms: {pose_ms:.0f}ms"
    print(f"{name}: Limp={limp_ms:.0f}ms Posing={pose_ms:.0f}ms，50→160→200 顺序恢复  OK")
PY
}

echo "===== C. 默认 OFF ====="
run_case off 0
echo "===== D/E. 显式 ON，两次独立闭环 ====="
run_case on1 1
run_case on2 1

echo "M9c Bite 1 验收全部通过"

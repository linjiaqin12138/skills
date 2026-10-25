#!/usr/bin/env bash
# M9a Bite 3：BAM 参数对账 + 自建 MJCF 闭环站立与行走。
#
# 在仓库根执行：bash accept/m09a-b03.sh
# 容器用法：docker compose -p microduck-from-scratch -f impl/docker-compose.yml exec rust bash /work/accept/m09a-b03.sh
#
# 预期：
#   A. motor/model/kp/电压范围/压降范围/vin_min/max_current/timestep 与官方训练及 CPU 回放对账；
#      每只脚的碰撞鞋底拆成至少 2 个 box，滑动摩擦回到官方 CPU 值 1.0。
#   B. kp/vin/vin_drop_gain 可显式覆盖；delay=3 时第 4 拍才执行第 1 拍目标；默认 delay=0；
#      延迟实验仍走 15 槽总线输入，不扩大线上契约。
#   C. 默认 BAM 场景必须是 scene_handmade.xml；3 次全新进程闭环均站立，
#      vx=0.8 ×10s 每次前移 >=2.0m、走后 z>0.08 且 skill=walk/fallen=false。
#      打印 3 个样本、最小值与中位数；不把单次成功当分布级证据。
#   D. 当前细分鞋底 handmade 做 2 次 sit_toggle 全循环：坐姿 z∈[0.04,0.10]、
#      skill=sit、fallen=false；起身后 z>0.08 且回 walk/fallen=false。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO/impl"

SIM_PYTHON="${SIM_PYTHON:-/opt/sim-venv/bin/python}"
SIM_PORT=7809
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

cargo build --bins
"$SIM_PYTHON" "$REPO/accept/m09a_bite3.py"

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
        | grep '"method":"robot.state"' | head -1 > /tmp/m9a-b03-state.json || true
    [ -s /tmp/m9a-b03-state.json ] || { echo "? ?"; return; }
    "$SIM_PYTHON" -c '
import json
p=json.load(open("/tmp/m9a-b03-state.json"))["params"]
print(p.get("skill"), p.get("fallen"))
' 2>/dev/null || echo "? ?"
}

fresh_closed_loop() {
    local run="$1"
    cleanup
    rm -f /tmp/miniduckd.sock /tmp/m9a-b03-state.json
    env MUJOCO_GL=disable "$SIM_PYTHON" sim/duck_body.py --port "$SIM_PORT" \
        >"/tmp/m9a-b03-sim-${run}.out" 2>"/tmp/m9a-b03-sim-${run}.log" &
    SIM_PID=$!
    sleep 2
    kill -0 "$SIM_PID" 2>/dev/null || fail "sim 起不来：/tmp/m9a-b03-sim-${run}.log"
    grep -q 'scene=.*scene_handmade.xml' "/tmp/m9a-b03-sim-${run}.log" \
        || fail "默认 BAM 场景不是 scene_handmade.xml"
    ./target/debug/miniduckd --sim "$SIM_ADDR" \
        >"/tmp/m9a-b03-daemon-${run}.out" 2>"/tmp/m9a-b03-daemon-${run}.log" &
    DAEMON_PID=$!
    sleep 2
    kill -0 "$DAEMON_PID" 2>/dev/null || fail "daemon 起不来：/tmp/m9a-b03-daemon-${run}.log"
    ./target/debug/mini-duckctl enable >/dev/null
}

wait_stand() {
    local z="0" info="? ?" i
    for i in $(seq 1 24); do
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

echo "===== M9a Bite 3 自建模型闭环：3 次独立运行 ====="
DISTANCES=()
for run in 1 2 3; do
    echo "-- run $run --"
    fresh_closed_loop "$run"
    STAND_Z=$(wait_stand)
    echo "站立：PASS z=$STAND_Z"
    BODY0=$(body)
    X0=$(echo "$BODY0" | awk '{print $1}')
    ./target/debug/mini-duckctl move 0.8 0 0 --secs 10 >/dev/null
    BODY1=$(body)
    X1=$(echo "$BODY1" | awk '{print $1}')
    Z1=$(echo "$BODY1" | awk '{print $3}')
    INFO=$(state_snapshot)
    DIST=$(awk -v a="$X0" -v b="$X1" 'BEGIN { printf "%.6f", b-a }')
    echo "行走样本 $run：dx=$DIST m, z=$Z1, skill/fallen=$INFO"
    awk -v d="$DIST" 'BEGIN { exit !(d >= 2.0) }' || fail "run $run 位移不足：$DIST < 2.0"
    awk -v z="$Z1" 'BEGIN { exit !(z > 0.08) }' || fail "run $run 走后摔倒：z=$Z1"
    [ "$INFO" = "walk False" ] || fail "run $run 走后状态不对：$INFO"
    DISTANCES+=("$DIST")
done

SUMMARY=$(printf '%s\n' "${DISTANCES[@]}" | "$SIM_PYTHON" -c '
import statistics, sys
xs=[float(x) for x in sys.stdin]
print(f"samples={xs}; min={min(xs):.3f}; median={statistics.median(xs):.3f}")
')
echo "$SUMMARY"

echo "== handmade sit_toggle：2 次全新进程，坐稳 + 起身 =="
for run in 1 2; do
    echo "-- sit run $run --"
    fresh_closed_loop "sit-$run"
    wait_stand >/dev/null
    ./target/debug/mini-duckctl do sit_toggle >/dev/null || fail "sit run $run：do sit"
    sleep 2.5
    SIT_Z=$(body | awk '{print $3}')
    SIT_INFO=$(state_snapshot)
    awk -v z="$SIT_Z" 'BEGIN { exit !(z >= 0.04 && z <= 0.10) }' \
        || fail "sit run $run 坐姿高度不对：z=$SIT_Z"
    [ "$SIT_INFO" = "sit False" ] || fail "sit run $run 坐姿状态不对：$SIT_INFO"
    ./target/debug/mini-duckctl do sit_toggle >/dev/null || fail "sit run $run：起身"
    sleep 3
    UP_Z=$(body | awk '{print $3}')
    UP_INFO=$(state_snapshot)
    awk -v z="$UP_Z" 'BEGIN { exit !(z > 0.08) }' \
        || fail "sit run $run 起身后摔倒：z=$UP_Z"
    [ "$UP_INFO" = "walk False" ] || fail "sit run $run 起身状态不对：$UP_INFO"
    echo "sit 样本 $run：坐姿 z=$SIT_Z，起身 z=$UP_Z，skill/fallen=$UP_INFO"
done

echo "M9a Bite 3 验收 PASS：BAM 对账 / 延迟队列 / 3 次站立行走 / 2 次 sit 全循环全绿"

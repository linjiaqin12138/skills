#!/usr/bin/env bash
# M4 验收：sim 行走。一键可重跑。
#
# 用法（宿主机）：docker compose exec rust bash scripts/accept-m4.sh
# 全程在容器内跑：sim（MuJoCo）、miniduckd --sim、mini-duckctl 都只见
# 127.0.0.1，不需要跨容器网络。容器里已有 ORT_DYLIB_PATH。
set -u
cd /work

SIM_ADDR=127.0.0.1:7801
SIM_PID=""
DAEMON_PID=""

cleanup() {
    [ -n "$DAEMON_PID" ] && kill "$DAEMON_PID" 2>/dev/null
    [ -n "$SIM_PID" ] && kill "$SIM_PID" 2>/dev/null
    # 残留兜底（[.] 防止 pkill 匹配到本脚本自己的命令行）
    pkill -f "duck_body[.]py" 2>/dev/null
    pkill -x miniduckd 2>/dev/null
}
trap cleanup EXIT
cleanup
sleep 1
rm -f /tmp/miniduckd.sock

echo "== 1. 启动 MuJoCo 仿真体 =="
python3 -c "import mujoco" 2>/dev/null || {
    echo "FAIL: 容器里缺 mujoco。装：python3 -m pip install --user --break-system-packages mujoco"
    echo "（容器若无 pip：curl get-pip.py 后 python3 get-pip.py --user --break-system-packages）"
    exit 1
}
python3 sim/duck_body.py --port 7801 2>/tmp/m4-sim.log &
SIM_PID=$!
sleep 3

echo "== 2. 启动 miniduckd --sim $SIM_ADDR =="
./target/debug/miniduckd --sim "$SIM_ADDR" 2>/tmp/m4-daemon.log &
DAEMON_PID=$!
sleep 2

echo "== 3. health（控制循环应已在跑）=="
./target/debug/mini-duckctl health || { echo "FAIL: health"; exit 1; }

echo "== 3b. enable（M5 起启动抱持不动，必须显式使能）=="
./target/debug/mini-duckctl enable || { echo "FAIL: enable"; exit 1; }

echo "== 4. 站稳（3 秒，斜坡 2s + 策略闭环 1s）=="
sleep 3
BODY0=$(python3 -c "
import json, socket
s = socket.create_connection(('127.0.0.1', 7801), timeout=5)
f = s.makefile('rw')
f.write(json.dumps({'op': 'body'}) + '\n'); f.flush()
r = json.loads(f.readline())
print(r['body_pos'][0], r['body_pos'][1], r['body_pos'][2], r['sim_time'])")
echo "站稳后 body(x y z t): $BODY0"
Z0=$(echo "$BODY0" | awk '{print $3}')
awk -v z="$Z0" 'BEGIN { if (z < 0.08) { print "FAIL: 鸭子倒了（躯干 z=" z "）"; exit 1 } }' || exit 1

echo "== 5. move vx=0.15 持续 10 秒 =="
X0=$(echo "$BODY0" | awk '{print $1}')
./target/debug/mini-duckctl move 0.15 0 0 --secs 10 || { echo "FAIL: move"; exit 1; }

BODY1=$(python3 -c "
import json, socket
s = socket.create_connection(('127.0.0.1', 7801), timeout=5)
f = s.makefile('rw')
f.write(json.dumps({'op': 'body'}) + '\n'); f.flush()
r = json.loads(f.readline())
print(r['body_pos'][0], r['body_pos'][1], r['body_pos'][2])")
echo "move 后 body(x y z): $BODY1"
X1=$(echo "$BODY1" | awk '{print $1}')
DIST=$(awk -v a="$X0" -v b="$X1" 'BEGIN { printf "%.3f", b - a }')
echo "== 结果：10 秒前进位移 = ${DIST} m（门槛 0.5m）=="
awk -v d="$DIST" 'BEGIN { exit !(d + 0 >= 0.5) }' && echo "PASS" || { echo "FAIL: 位移不足"; exit 1; }

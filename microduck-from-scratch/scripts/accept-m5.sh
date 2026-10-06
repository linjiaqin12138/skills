#!/usr/bin/env bash
# M5 验收：安全层。一键可重跑。
#
# 用法（宿主机）：docker compose exec rust bash scripts/accept-m5.sh
# 全程在容器内跑。与 accept-m4.sh 错开端口（7803），socket 同一文件
# （同一时刻只跑一个 daemon，阶段间杀掉再起）。
#
# 六项断言：
#   1. 启动不动 + enable 后才动
#   2. deadman：意图失联 500ms 后 twist 清零；--secs 重发期间保持
#   3. sim 跌倒卸力：push → fallen=true + gain=50 + 瘫在地上
#   4. health 真话：杀 sim → unhealthy；重启 → 恢复
#   5. robot.state 50Hz：2 秒 ≥90 帧通知
#   6. 行走回归：move 10s 位移 ≥0.5m
set -u
cd /work

SIM_PORT=${MINIDUCK_SIM_PORT:-7803}
SIM_ADDR=127.0.0.1:$SIM_PORT
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

# 抓 robot.state 通知：$1=秒数，stdout 输出通知行（跳过订阅 ack）。
state_frames() {
    timeout "$1" ./target/debug/mini-duckctl state --every 1 2>/dev/null | tail -n +2
}

# 抓一帧。
state_frame() {
    state_frames 1 | head -1
}

# 查躯干世界系位置。
body() {
    python3 -c "
import json, socket
s = socket.create_connection(('127.0.0.1', $SIM_PORT), timeout=5)
f = s.makefile('rw')
f.write(json.dumps({'op': 'body'}) + '\n'); f.flush()
r = json.loads(f.readline())
print(r['body_pos'][0], r['body_pos'][1], r['body_pos'][2], r['sim_time'])"
}

fail() { echo "FAIL: $1"; exit 1; }

echo "===== 阶段 A：FakeIo（无 sim）====="

echo "== A1. 启动不动 =="
./target/debug/miniduckd 2>/tmp/m5-daemon-a.log &
DAEMON_PID=$!
sleep 1
F=$(state_frame)
[ -n "$F" ] || fail "没收到 robot.state 帧"
echo "$F" | python3 -c "
import json, sys
p = json.loads(sys.stdin.read())['params']
pos = p['positions']
assert all(abs(v) < 1e-12 for v in pos), f'启动后应全 0（启动姿态），实际 {pos}'
assert p['enabled'] is False, f'启动后 enabled 应为 false，实际 {p[\"enabled\"]}'
assert p['torque'] is False, '启动后本进程不应命令过 torque'
print('启动姿态 positions 全 0，enabled=false，torque=false  OK')"

echo "== A1b. enable → 斜坡到 home → 站稳 =="
./target/debug/mini-duckctl enable >/dev/null || fail "enable"
sleep 3
F=$(state_frame)
echo "$F" | python3 -c "
import json, sys
p = json.loads(sys.stdin.read())['params']
home = [0.0, -0.0873, -0.4579, -0.0049, 0.4530, 0.3491, 0.3491, 0.0, 0.0, 0.0,
        0.0, 0.0873, 0.4579, 0.0049, -0.4530]
diff = max(abs(a - b) for a, b in zip(p['positions'], home))
assert p['enabled'] is True, 'enable 后 enabled 应为 true'
assert p['gain'] == 200, f'Driving 增益应为 200，实际 {p[\"gain\"]}'
# Driving 阶段目标 = home + 0.9×策略动作，不会逐位等于 home；
# 这里断言「已到达 home 附近被策略闭环」，精确相等留给 disable 后的 Held。
# 阈值 0.4：FakeIo 上实测漂移 0.13~0.18 rad（速度恒 0 的假总线本来
# 就不是策略的训练分布），留一倍余量。
assert diff < 0.4, f'enable 3s 后应贴近 home，最大偏差 {diff}'
print(f'enable 后 enabled=true, gain=200, 距 home 最大偏差 {diff:.4f} rad  OK')"
./target/debug/mini-duckctl health | tee /dev/stderr | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['healthy'] is True, f'应为 healthy，实际 {r}'
print('health healthy=true  OK')" || fail "health"

echo "== A1c. disable → 斜坡回 home → 卸 torque，位置精确等于 home =="
./target/debug/mini-duckctl disable >/dev/null || fail "disable"
sleep 3
F=$(state_frame)
echo "$F" | python3 -c "
import json, sys
p = json.loads(sys.stdin.read())['params']
home = [0.0, -0.0873, -0.4579, -0.0049, 0.4530, 0.3491, 0.3491, 0.0, 0.0, 0.0,
        0.0, 0.0873, 0.4579, 0.0049, -0.4530]
diff = max(abs(a - b) for a, b in zip(p['positions'], home))
assert diff < 1e-6, f'disable 后 Held(home) 应逐位等于 home，最大偏差 {diff}'
assert p['enabled'] is False and p['torque'] is False, \
    f'disable 后 enabled/torque 应为 false，实际 {p[\"enabled\"]}/{p[\"torque\"]}'
print(f'disable 后回到 Held：|positions-home|max={diff:.2e}，enabled=false，torque=false  OK')"

echo "== A2. deadman =="
./target/debug/mini-duckctl enable >/dev/null || fail "enable"
sleep 3
./target/debug/mini-duckctl move 0.15 0 0 >/dev/null || fail "move"
state_frames 1 | python3 -c "
import json, sys
seen = [json.loads(l)['params'] for l in sys.stdin if l.strip()]
assert seen, '没抓到帧'
hit = [f for f in seen if abs(f['obs'][48] - 0.15) < 1e-6]
assert hit, f'单次意图后 obs twist vx 应为 0.15，抓到 {[f[\"obs\"][48] for f in seen]}'
print(f'单次意图后 obs twist vx={hit[0][\"obs\"][48]}  OK')" || fail "deadman 新鲜意图"
sleep 1
F=$(state_frame)
echo "$F" | python3 -c "
import json, sys
p = json.loads(sys.stdin.read())['params']
twist = p['obs'][48:51]
assert all(abs(v) < 1e-12 for v in twist), f'失联 1s 后 twist 应清零，实际 {twist}'
print('失联 1s 后 obs twist=[0,0,0]（deadman 生效）  OK')"
./target/debug/mini-duckctl move 0.15 0 0 --secs 2 >/dev/null &
DRIVE_PID=$!
sleep 1.5
F=$(state_frame)
echo "$F" | python3 -c "
import json, sys
p = json.loads(sys.stdin.read())['params']
assert abs(p['obs'][48] - 0.15) < 1e-6, \
    f'--secs 重发期间 twist 应保持 0.15，实际 {p[\"obs\"][48]}'
print('--secs 2 期间（1.5s 处采样）twist vx 保持 0.15  OK')"
wait $DRIVE_PID

kill $DAEMON_PID 2>/dev/null; DAEMON_PID=""
sleep 1
rm -f /tmp/miniduckd.sock

echo "===== 阶段 B：MuJoCo sim ====="

echo "== B0. 起 sim + daemon --sim =="
python3 -c "import mujoco" 2>/dev/null || fail "容器里缺 mujoco"
python3 sim/duck_body.py --port $SIM_PORT 2>/tmp/m5-sim.log &
SIM_PID=$!
sleep 3
./target/debug/miniduckd --sim "$SIM_ADDR" 2>/tmp/m5-daemon-b.log &
DAEMON_PID=$!
sleep 2
./target/debug/mini-duckctl enable >/dev/null || fail "enable"

# 等站稳（斜坡 2s + 策略闭环），最多 8 秒。
for i in $(seq 1 16); do
    Z=$(body | awk '{print $3}')
    awk -v z="$Z" 'BEGIN { exit !(z > 0.08) }' && break
    [ "$i" = 16 ] && fail "鸭子没站起来（z=$Z）"
    sleep 0.5
done
echo "站稳：body z=$Z"

echo "== B3. push 推倒 → fallen + 卸力（gain 50）=="
python3 -c "
import json, socket
s = socket.create_connection(('127.0.0.1', $SIM_PORT), timeout=5)
f = s.makefile('rw')
f.write(json.dumps({'op': 'push', 'vx': 1.5, 'vy': 0.0}) + '\n'); f.flush()
print('push 应答:', f.readline().strip())"
sleep 0.5
state_frames 3 | python3 -c "
import json, sys
frames = [json.loads(l)['params'] for l in sys.stdin if l.strip()]
assert frames, '没抓到帧'
hit = [f for f in frames if f['fallen'] and f['gain'] == 50]
assert hit, 'push 后 3.5s 内未见到 fallen=true 且 gain=50 的帧'
print(f'捕获 {len(frames)} 帧，首帧 fallen+gain=50 出现在第 {frames.index(hit[0]) + 1} 帧  OK')" \
    || fail "跌倒卸力"
sleep 1
Z=$(body | awk '{print $3}')
awk -v z="$Z" 'BEGIN { exit !(z < 0.08) }' \
    && echo "瘫在地上：body z=$Z（站立约 0.12）  OK" \
    || fail "鸭子没瘫下去（z=$Z）"

echo "== B4. health 真话：杀 sim =="
kill $SIM_PID 2>/dev/null; SIM_PID=""
for i in $(seq 1 6); do
    H=$(./target/debug/mini-duckctl health 2>/dev/null | head -1)
    [ -n "$H" ] && echo "$H" | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
exit(0 if r['healthy'] is False else 1)" && break
    [ "$i" = 6 ] && fail "杀 sim 3s 后 health 仍报 healthy: $H"
    sleep 0.5
done
echo "杀 sim 后 health: $H" | head -c 400; echo
echo "$H" | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['consecutive_read_errors'] >= 10, r
print('healthy=false，原因:', r['reason'])"

echo "== B4b. 重启 sim → 重连恢复 =="
python3 sim/duck_body.py --port $SIM_PORT 2>/tmp/m5-sim2.log &
SIM_PID=$!
for i in $(seq 1 20); do
    H=$(./target/debug/mini-duckctl health 2>/dev/null | head -1)
    [ -n "$H" ] && echo "$H" | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
exit(0 if r['healthy'] is True else 1)" && break
    [ "$i" = 20 ] && fail "重启 sim 10s 后 health 未恢复: $H"
    sleep 0.5
done
echo "重连后 healthy=true  OK"

echo "== B5. robot.state 50Hz =="
N=$(state_frames 2 | wc -l)
echo "2 秒收到 $N 帧通知"
[ "$N" -ge 90 ] || fail "50Hz 推送不足：2s 只收到 $N 帧（≥90）"

echo "== B6. 行走回归 =="
# 重连后鸭子从 STAND 姿态斜坡回 home 再闭环，先等站稳。
for i in $(seq 1 16); do
    Z=$(body | awk '{print $3}')
    awk -v z="$Z" 'BEGIN { exit !(z > 0.08) }' && break
    [ "$i" = 16 ] && fail "重连后鸭子没站起来（z=$Z）"
    sleep 0.5
done
BODY0=$(body)
X0=$(echo "$BODY0" | awk '{print $1}')
echo "起步 body: $BODY0"
./target/debug/mini-duckctl move 0.15 0 0 --secs 10 >/dev/null || fail "move"
BODY1=$(body)
X1=$(echo "$BODY1" | awk '{print $1}')
DIST=$(awk -v a="$X0" -v b="$X1" 'BEGIN { printf "%.3f", b - a }')
echo "10 秒前进位移 = ${DIST} m（门槛 0.5m）"
awk -v d="$DIST" 'BEGIN { exit !(d + 0 >= 0.5) }' || fail "位移不足"

echo
echo "M5 验收全部通过：启动不动/deadman/跌倒卸力/health 真话/50Hz state/行走回归"

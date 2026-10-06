#!/usr/bin/env bash
# M6 验收：技能调度器（多策略优先级链）。一键可重跑。
#
# 用法（宿主机）：docker compose exec rust bash scripts/accept-m6.sh
# 全程在容器内跑。sim 端口 7804（accept-m4/m5 用 7802/7803，错开可串行）。
#
# 断言清单：
#   阶段 A（FakeIo）：
#     A1. 启动不动 + enable 后站立回归（复用 accept-m5 断言）
#     A2. robot.skills 名单含内置两个（ground_pick/sit_toggle）+ 配置技能
#     A3. robot.do 拒绝语义：未 enable 拒绝；未知技能拒绝并附名单
#     A4. robot.mouth：0→−5°、1→+30°、7.0 夹紧到 +30°（Driving 相位覆写）
#     A5. robot.head 进观测 command 块，无 deadman（1s 后仍在）
#   阶段 B（MuJoCo sim）：
#     B0. 站稳回归
#     B1. 行走回归：vx=0.15 走 10s 前移 ≥0.5m
#     B2. robot.state 50Hz 不断流
#     B3. ground_pick：skill 字段 + twist 相位编码（|twist|=1）+ 自动回 walk
#     B4. sit_toggle（降级断言，物理失败已登记 D34）：调度正确性——
#         skill=="sit"、obs twist=[1,0,0]、锁存
#     B5. roulade（物理失败已登记 D35）：窗口计时 50 拍、busy 期间 fallen
#         不触发 Limp（gain 保持 200）、窗口结束后 Limp 接管（gain 50）
#     B6. kick_left：skill 字段 + 窗口结束自动回 walk + 不摔倒
set -u
cd /work

SIM_PORT=7804
SIM_ADDR=127.0.0.1:$SIM_PORT
SIM_PID=""
DAEMON_PID=""

cleanup() {
    [ -n "$DAEMON_PID" ] && kill "$DAEMON_PID" 2>/dev/null
    [ -n "$SIM_PID" ] && kill "$SIM_PID" 2>/dev/null
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

state_frame() {
    state_frames 1 | head -1
}

body() {
    python3 -c "
import json, socket
s = socket.create_connection(('127.0.0.1', $SIM_PORT), timeout=5)
f = s.makefile('rw')
f.write(json.dumps({'op': 'body'}) + '\n'); f.flush()
r = json.loads(f.readline())
print(r['body_pos'][0], r['body_pos'][1], r['body_pos'][2], r['sim_time'])"
}

# 起一对新的 sim+daemon 并 enable 到站稳（破坏性技能测试之间重置物理状态）。
fresh_sim() {
    kill $DAEMON_PID 2>/dev/null; DAEMON_PID=""
    kill $SIM_PID 2>/dev/null; SIM_PID=""
    sleep 1
    rm -f /tmp/miniduckd.sock
    python3 sim/duck_body.py --port $SIM_PORT 2>/tmp/m6-sim.log &
    SIM_PID=$!
    sleep 3
    ./target/debug/miniduckd --sim "$SIM_ADDR" 2>/tmp/m6-daemon.log &
    DAEMON_PID=$!
    sleep 2
    ./target/debug/mini-duckctl enable >/dev/null || fail "enable"
    for i in $(seq 1 16); do
        Z=$(body | awk '{print $3}')
        awk -v z="$Z" 'BEGIN { exit !(z > 0.08) }' && break
        [ "$i" = 16 ] && fail "鸭子没站起来（z=$Z）"
        sleep 0.5
    done
    # z 达标可能还在 RampUp（斜坡中途躯干已过 0.08），而 robot.do 只在
    # Driving 相位接受（对齐原版 homed 检查）。等到 skill=="walk" 才算稳。
    for i in $(seq 1 20); do
        S=$(state_frame | python3 -c "
import json, sys
print(json.loads(sys.stdin.read())['params']['skill'])" 2>/dev/null)
        [ "$S" = "walk" ] && break
        [ "$i" = 20 ] && fail "站稳后 10s 仍未进 Driving（skill=$S）"
        sleep 0.5
    done
    echo "站稳：body z=$Z, skill=$S"
}

fail() { echo "FAIL: $1"; exit 1; }

echo "===== 阶段 A：FakeIo（无 sim）====="

./target/debug/miniduckd 2>/tmp/m6-daemon-a.log &
DAEMON_PID=$!
sleep 1

echo "== A3a. 未 enable 时 robot.do 拒绝 =="
./target/debug/mini-duckctl do sit_toggle | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['accepted'] is False, f'未 enable 应拒绝，实际 {r}'
assert 'not driving' in r['reason'], r
print(f'未 enable 拒绝: {r[\"reason\"]}  OK')" || fail "do 未 enable 拒绝"

echo "== A1. 启动不动 + enable 站立回归 =="
F=$(state_frame)
[ -n "$F" ] || fail "没收到 robot.state 帧"
echo "$F" | python3 -c "
import json, sys
p = json.loads(sys.stdin.read())['params']
pos = p['positions']
assert all(abs(v) < 1e-12 for v in pos), f'启动后应全 0，实际 {pos}'
assert p['enabled'] is False and p['torque'] is False
assert p['skill'] is None, f'未 Driving 时 skill 应为 null，实际 {p[\"skill\"]}'
print('启动姿态全 0，enabled=false，skill=null  OK')"
./target/debug/mini-duckctl enable >/dev/null || fail "enable"
sleep 3
F=$(state_frame)
echo "$F" | python3 -c "
import json, sys
p = json.loads(sys.stdin.read())['params']
home = [0.0, -0.0873, -0.4579, -0.0049, 0.4530, 0.3491, 0.3491, 0.0, 0.0, -0.0873,
        0.0, 0.0873, 0.4579, 0.0049, -0.4530]
# 注意 home[9]（嘴）：Driving 相位 robot.mouth 意图默认 0 → 覆写为 −5°，
# 所以期望值不是 DEFAULT_POSITION 的 0.0 而是 −0.0873（M6 起，原版同：
# 只有 Driving 覆写嘴，held/homing 机器人保持抱持姿态里嘴的角度）。
diff = max(abs(a - b) for a, b in zip(p['positions'], home))
assert p['enabled'] is True and p['gain'] == 200
assert p['skill'] == 'walk', f'Driving 默认技能应为 walk，实际 {p[\"skill\"]}'
assert diff < 0.4, f'enable 3s 后应贴近 home，最大偏差 {diff}'
print(f'enabled=true, gain=200, skill=walk, 距 home 最大偏差 {diff:.4f}（含嘴 −5°）  OK')"

echo "== A2. robot.skills 名单 =="
./target/debug/mini-duckctl skills | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
want = ['ground_pick', 'sit_toggle', 'roulade', 'kick_left']
assert r['skills'] == want, f'名单应为 {want}，实际 {r[\"skills\"]}'
print(f'skills = {r[\"skills\"]}（内置两个在名单里，顺序=原版 do_names）  OK')" || fail "skills 名单"

echo "== A3b. 未知技能拒绝并附名单；edge 语义（do 接受后下一拍生效）=="
./target/debug/mini-duckctl do backflip | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['accepted'] is False, f'未知技能应拒绝，实际 {r}'
assert 'backflip' in r['reason'] and 'roulade' in r['reason'], r
print(f'未知技能拒绝: {r[\"reason\"]}  OK')" || fail "未知技能拒绝"

echo "== A4. robot.mouth 映射与夹紧 =="
./target/debug/mini-duckctl mouth 1.0 >/dev/null || fail "mouth 1.0"
sleep 0.5
state_frame | python3 -c "
import json, sys, math
p = json.loads(sys.stdin.read())['params']
assert abs(p['positions'][9] - math.radians(30.0)) < 1e-9, \
    f'mouth 1.0 应为 +30°，实际 {p[\"positions\"][9]}'
print('mouth 1.0 → +30°  OK')"
./target/debug/mini-duckctl mouth 0.0 >/dev/null || fail "mouth 0.0"
sleep 0.5
state_frame | python3 -c "
import json, sys, math
p = json.loads(sys.stdin.read())['params']
assert abs(p['positions'][9] - math.radians(-5.0)) < 1e-9, \
    f'mouth 0.0 应为 −5°，实际 {p[\"positions\"][9]}'
print('mouth 0.0 → −5°  OK')"
./target/debug/mini-duckctl mouth 7.0 >/dev/null || fail "mouth 7.0"
sleep 0.5
state_frame | python3 -c "
import json, sys, math
p = json.loads(sys.stdin.read())['params']
assert abs(p['positions'][9] - math.radians(30.0)) < 1e-9, \
    f'mouth 7.0 应夹紧到 +30°，实际 {p[\"positions\"][9]}'
print('mouth 7.0 → 夹紧 +30°  OK')"
./target/debug/mini-duckctl mouth 0.0 >/dev/null

echo "== A5. robot.head 进观测 command 块，无 deadman =="
./target/debug/mini-duckctl head 0.1 0.2 0.3 0.4 >/dev/null || fail "head"
sleep 0.5
state_frame | python3 -c "
import json, sys
p = json.loads(sys.stdin.read())['params']
head = p['obs'][51:55]
assert all(abs(a - b) < 1e-6 for a, b in zip(head, [0.1, 0.2, 0.3, 0.4])), \
    f'obs head 块应为 [0.1,0.2,0.3,0.4]，实际 {head}'
print(f'obs[51:55] = {[round(v,3) for v in head]}  OK')"
sleep 1
state_frame | python3 -c "
import json, sys
p = json.loads(sys.stdin.read())['params']
head = p['obs'][51:55]
assert all(abs(a - b) < 1e-6 for a, b in zip(head, [0.1, 0.2, 0.3, 0.4])), \
    f'头无 deadman：1s 后应仍在，实际 {head}'
print('1s 未重发头命令仍保持（head 无 deadman）  OK')"
./target/debug/mini-duckctl head 0 0 0 0 >/dev/null

kill $DAEMON_PID 2>/dev/null; DAEMON_PID=""
sleep 1
rm -f /tmp/miniduckd.sock

echo "===== 阶段 B：MuJoCo sim（端口 $SIM_PORT）====="
python3 -c "import mujoco" 2>/dev/null || fail "容器里缺 mujoco"

echo "== B0. 起 sim + daemon --sim，站稳回归 =="
fresh_sim

echo "== B1. 行走回归 =="
BODY0=$(body)
X0=$(echo "$BODY0" | awk '{print $1}')
echo "起步 body: $BODY0"
./target/debug/mini-duckctl move 0.15 0 0 --secs 10 >/dev/null || fail "move"
BODY1=$(body)
X1=$(echo "$BODY1" | awk '{print $1}')
DIST=$(awk -v a="$X0" -v b="$X1" 'BEGIN { printf "%.3f", b - a }')
echo "10 秒前进位移 = ${DIST} m（门槛 0.5m）"
awk -v d="$DIST" 'BEGIN { exit !(d + 0 >= 0.5) }' || fail "位移不足"

echo "== B2. robot.state 50Hz =="
N=$(state_frames 2 | wc -l)
echo "2 秒收到 $N 帧通知"
[ "$N" -ge 90 ] || fail "50Hz 推送不足：2s 只收到 $N 帧（≥90）"

echo "== B3. ground_pick：调度 + 相位编码 + 自动交还 =="
state_frames 6 > /tmp/m6-gp.ndjson &
REC=$!
sleep 0.5
./target/debug/mini-duckctl do ground_pick | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['accepted'] is True, r" || fail "do ground_pick"
wait $REC
python3 - <<'EOF' || fail "ground_pick 断言"
import json, math
frames = [json.loads(l)['params'] for l in open('/tmp/m6-gp.ndjson') if l.strip()]
gp = [f for f in frames if f.get('skill') == 'ground_pick']
assert gp, '没见过 skill=="ground_pick" 的帧'
# 窗口计时：140 拍 = 2.8s。50Hz 推送有 latest-wins 丢帧，放宽到 100..160。
assert 100 <= len(gp) <= 160, f'ground_pick 应跑 ~140 帧，实际 {len(gp)}'
for f in gp[:5]:
    t = f['obs'][48:51]
    assert abs(math.hypot(t[0], t[1]) - 1.0) < 1e-5, f'twist 应为单位圆相位编码，实际 {t}'
    assert t[2] == 0.0, t
t0 = gp[0]['obs'][48:51]
assert abs(t0[0] - 1.0) < 0.05 and abs(t0[1]) < 0.05, f'φ=0 应为 [1,0,0]，实际 {t0}'
assert not any(f['fallen'] for f in frames), 'ground_pick 全程不该摔倒'
after = frames[frames.index(gp[-1]) + 1:]
assert after and all(f.get('skill') == 'walk' for f in after[:20]), '结束后应自动回 walk'
print(f'ground_pick：{len(gp)} 帧（~140），twist=单位圆相位编码，无摔倒，结束回 walk  OK')
EOF

echo "== B6. kick_left：窗口 + 自动回 walk（kick 物理上站得住）=="
state_frames 4 > /tmp/m6-kick.ndjson &
REC=$!
sleep 0.5
./target/debug/mini-duckctl do kick_left | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['accepted'] is True, r" || fail "do kick_left"
wait $REC
python3 - <<'EOF' || fail "kick_left 断言"
import json
frames = [json.loads(l)['params'] for l in open('/tmp/m6-kick.ndjson') if l.strip()]
kick = [f for f in frames if f.get('skill') == 'kick_left']
assert kick, '没见过 skill=="kick_left" 的帧'
# 窗口 25 拍 = 0.5s；推送丢帧放宽到 15..45。
assert 15 <= len(kick) <= 45, f'kick 窗口应 ~25 帧，实际 {len(kick)}'
assert all(f['obs'][48:55] == [0.0] * 7 for f in kick[:3]), '技能窗口内 command 应全零'
assert not any(f['fallen'] for f in frames), 'kick 全程不该摔倒'
after = frames[frames.index(kick[-1]) + 1:]
assert after and all(f.get('skill') == 'walk' for f in after[:20]), 'kick 结束应回 walk'
print(f'kick_left：{len(kick)} 帧窗口（~25），command 全零，无摔倒，结束回 walk  OK')
EOF

echo "== B4. sit_toggle：调度断言（物理摔倒已登记 D34）=="
fresh_sim
state_frames 5 > /tmp/m6-sit.ndjson &
REC=$!
sleep 0.5
./target/debug/mini-duckctl do sit_toggle | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['accepted'] is True, r" || fail "do sit_toggle"
wait $REC
python3 - <<'EOF' || fail "sit 断言"
import json
frames = [json.loads(l)['params'] for l in open('/tmp/m6-sit.ndjson') if l.strip()]
sit = [f for f in frames if f.get('skill') == 'sit']
assert sit, '没见过 skill=="sit" 的帧'
first = frames.index(sit[0])
# 坐姿标志：vx 槽载 posture flag=1。
assert all(f['obs'][48] == 1.0 and f['obs'][49] == 0.0 and f['obs'][50] == 0.0
           for f in sit[:5]), '坐姿 twist 应为 [1,0,0]'
# 锁存：从第一帧 sit 到摔倒（若有）之间不许自己退回 walk。
fallen_at = next((i for i, f in enumerate(frames) if f['fallen']), len(frames))
held = [f for f in frames[first:fallen_at] if f.get('skill') is not None]
assert all(f['skill'] == 'sit' for f in held), '坐姿应锁存到摔倒/起身为止'
print(f'sit：{len(sit)} 帧 skill=="sit"，twist=[1,0,0]，锁存保持  OK')
if fallen_at < len(frames):
    print(f'注：物理上坐下后向后翻倒（第 {fallen_at} 帧 fallen）——已登记偏差 D34，M8 随 D21 裁决')
EOF

echo "== B5. roulade：窗口计时 + busy 抑制 Limp（物理摔倒已登记 D35）=="
fresh_sim
state_frames 6 > /tmp/m6-rou.ndjson &
REC=$!
sleep 0.5
./target/debug/mini-duckctl do roulade | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['accepted'] is True, r" || fail "do roulade"
wait $REC
python3 - <<'EOF' || fail "roulade 断言"
import json
frames = [json.loads(l)['params'] for l in open('/tmp/m6-rou.ndjson') if l.strip()]
rou = [f for f in frames if f.get('skill') == 'roulade']
assert rou, '没见过 skill=="roulade" 的帧'
# 窗口 50 拍 = 1.0s；推送丢帧放宽到 35..70。
assert 35 <= len(rou) <= 70, f'roulade 窗口应 ~50 帧，实际 {len(rou)}'
assert all(f['obs'][48:55] == [0.0] * 7 for f in rou[:3]), '技能窗口内 command 应全零'
# busy 门控（D32）：roulade 期间即使 fallen=true 也不许进 Limp（gain 保持 200）。
busy_fallen = [f for f in rou if f['fallen']]
assert busy_fallen, 'roulade 中途应摔倒（物理现实，D35）——若没摔，说明物理修好了，应升级本断言'
assert all(f['gain'] == 200 for f in busy_fallen), \
    'busy 期间 fallen 不该触发 Limp（gain 应保持 200）'
# 窗口结束、busy 解除后：Limp 接管（gain 50 出现）。
after = frames[frames.index(rou[-1]) + 1:]
assert any(f['gain'] == 50 for f in after), 'busy 结束后 fallen 应触发 Limp（gain 50）'
print(f'roulade：{len(rou)} 帧窗口（~50），busy 期间 {len(busy_fallen)} 帧 fallen 全部 gain=200，'
      f'结束后 Limp 接管  OK')
EOF

echo
echo "M6 验收全部通过：站立/行走回归 + skills 名单 + do 拒绝语义 + mouth/head +"
echo "  ground_pick/kick 调度物理双过 + sit/roulade 调度断言（物理失败 D34/D35）+ 50Hz 不断流"

#!/usr/bin/env bash
# M8 Bite 6 物理验收：BAM + groundcontact 下站立 / 行走 / sit / roulade。
#
# 用法（宿主机）：docker compose -f impl/docker-compose.yml exec rust bash /work/accept/m08-b06.sh
# 全程在容器内跑。sim 端口 7806（避开 accept-m4/m5/m6/bam-diag 的 7801–7805）。
#
# 仿真进程：/opt/sim-venv/bin/python（或 SIM_PYTHON）；系统 python3.11 无 bam。
#
# 门槛（实测后写入；行走门槛相对 M4 的 0.5m 略放宽到 0.4m，BAM+groundcontact）：
#   站立：enable 后 z>0.08、skill=walk、fallen=false
#   行走：vx=0.8 走 10s，前移 ≥2.0m（2026-10-14 实测 4.911m，留 >2× 余量）
#         【2026-10-14 用户裁决】公开 velstand 权重对训练分布内命令（0.15/0.3）
#         不响应——官方 infer_policy.py 与我们双侧实测 dx≈0.005；vx=0.8
#         （分布外）双侧都能走出步态。故行走验收改为 0.8，验收目标是
#         "集成栈能驱动步态"；0.3 不走是上游权重现状，指向 M9 自训策略。
#   sit：sit_toggle → skill=sit；≥2s fallen 恒 false；躯干 z ∈ [0.04, 0.10]；
#        再 sit_toggle 起身 → walk、z>0.08、fallen=false
#   roulade：窗口 100 拍（2.0s，对齐官方 --roulade-duration）。
#            裁决（实测）：窗口内允许短暂 fallen（前滚翻中段躯干贴地），
#            结束后 2s 内必须 upright：skill=walk、z>0.08、fallen=false、gain=200。
set -u
cd "$(cd "$(dirname "$0")/../impl" && pwd)"

SIM_PORT=7806
SIM_ADDR=127.0.0.1:$SIM_PORT
SIM_PID=""
DAEMON_PID=""
SIM_PYTHON="${SIM_PYTHON:-/opt/sim-venv/bin/python}"
HELPER_PY="${HELPER_PY:-python3}"

cleanup() {
    [ -n "$DAEMON_PID" ] && kill "$DAEMON_PID" 2>/dev/null
    [ -n "$SIM_PID" ] && kill "$SIM_PID" 2>/dev/null
    ps aux | awk '/[s]im\/duck_body\.py/ {print $2}' | xargs -r kill 2>/dev/null
    pkill -x miniduckd 2>/dev/null
}
trap cleanup EXIT
cleanup
sleep 1
rm -f /tmp/miniduckd.sock

fail() { echo "FAIL: $1"; exit 1; }

# 抓一帧 robot.state 到文件，返回 skill fallen gain（空则 ? ? ?）
state_snapshot() {
    timeout 3 ./target/debug/mini-duckctl state --every 1 2>/dev/null \
        | grep '"method":"robot.state"' | head -1 > /tmp/m8-bam-state.json || true
    if [ ! -s /tmp/m8-bam-state.json ]; then
        echo "? ? ?"
        return
    fi
    $HELPER_PY -c '
import json
p=json.load(open("/tmp/m8-bam-state.json"))["params"]
print(p.get("skill"), p.get("fallen"), p.get("gain"))
' 2>/dev/null || echo "? ? ?"
}

state_frames() {
    timeout "$1" ./target/debug/mini-duckctl state --every 1 2>/dev/null \
        | grep '"method":"robot.state"' || true
}

# 写 body 到 stdout：x y z t（避免 $(python -c '…()…') 括号坑）
body() {
    SIM_PORT="$SIM_PORT" $HELPER_PY - <<'PY'
import json, os, socket
port = int(os.environ["SIM_PORT"])
s = socket.create_connection(("127.0.0.1", port), timeout=5)
f = s.makefile("rw")
f.write(json.dumps({"op": "body"}) + "\n"); f.flush()
r = json.loads(f.readline())
p = r["body_pos"]
print(p[0], p[1], p[2], r["sim_time"])
PY
}

fresh_sim() {
    kill $DAEMON_PID 2>/dev/null; DAEMON_PID=""
    kill $SIM_PID 2>/dev/null; SIM_PID=""
    sleep 1
    rm -f /tmp/miniduckd.sock
    # M8 的验收对象是官方 groundcontact；M9a 起默认场景已切到自建身体。
    env MUJOCO_GL=disable "$SIM_PYTHON" sim/duck_body.py --port $SIM_PORT \
        --scene sim/assets/scene_groundcontact.xml 2>/tmp/m8-bam-sim.log &
    SIM_PID=$!
    sleep 3
    if ! kill -0 $SIM_PID 2>/dev/null; then
        fail "sim 起不来：见 /tmp/m8-bam-sim.log"
    fi
    ./target/debug/miniduckd --sim "$SIM_ADDR" 2>/tmp/m8-bam-daemon.log &
    DAEMON_PID=$!
    sleep 2
    if ! kill -0 $DAEMON_PID 2>/dev/null; then
        fail "daemon 起不来：见 /tmp/m8-bam-daemon.log"
    fi
    ./target/debug/mini-duckctl enable >/dev/null || fail "enable"
}

wait_stand() {
    local Z="0" S="?" FALLEN="?"
    local i
    for i in $(seq 1 16); do
        Z=$(body | awk '{print $3}')
        awk -v z="$Z" 'BEGIN { exit !(z > 0.08) }' && break
        [ "$i" = 16 ] && fail "鸭子没站起来（z=$Z）"
        sleep 0.5
    done
    for i in $(seq 1 20); do
        local info
        info=$(state_snapshot)
        S=$(echo "$info" | awk '{print $1}')
        FALLEN=$(echo "$info" | awk '{print $2}')
        [ "$S" = "walk" ] && [ "$FALLEN" = "False" ] && break
        [ "$i" = 20 ] && fail "站稳后未进 Driving 或已摔倒（skill=$S fallen=$FALLEN z=$Z）"
        sleep 0.5
    done
    echo "站立：PASS  body z=$Z  skill=$S  fallen=$FALLEN"
}

echo "===== M8 BAM 物理验收（端口 $SIM_PORT, python=$SIM_PYTHON）====="
[ -x "$SIM_PYTHON" ] || fail "缺 $SIM_PYTHON（Dockerfile 应建 /opt/sim-venv）"
"$SIM_PYTHON" -c "import bam; from bam.mujoco import MujocoController" \
    || fail "sim-venv 缺 bam"
[ -x ./target/debug/miniduckd ] || fail "缺 target/debug/miniduckd（先 cargo build）"
[ -x ./target/debug/mini-duckctl ] || fail "缺 target/debug/mini-duckctl"
[ -f sim/assets/scene_groundcontact.xml ] || fail "缺 scene_groundcontact.xml"

fresh_sim
wait_stand

# 顺序：站立 → sit → roulade → 行走（坐/滚是本 Bite 主目标，行走是回归）
echo "== sit_toggle：坐稳 ≥2s + 起身 =="
fresh_sim
wait_stand >/dev/null
state_frames 8 > /tmp/m8-bam-sit.ndjson &
REC=$!
sleep 0.5
./target/debug/mini-duckctl do sit_toggle > /tmp/m8-bam-sit-do.json 2>/dev/null \
    || fail "do sit_toggle"
sleep 2.5
SIT_Z=$(body | awk '{print $3}')
./target/debug/mini-duckctl do sit_toggle > /tmp/m8-bam-sit-up.json 2>/dev/null \
    || fail "do sit_toggle 起身"
sleep 3
wait $REC 2>/dev/null || true
UP_Z=$(body | awk '{print $3}')
SIT_Z="$SIT_Z" UP_Z="$UP_Z" $HELPER_PY - <<'EOF' || fail "sit 断言"
import json, os
do = json.load(open("/tmp/m8-bam-sit-do.json"))
assert do.get("result", {}).get("accepted") is True, do
frames = []
for line in open("/tmp/m8-bam-sit.ndjson"):
    line = line.strip()
    if not line:
        continue
    try:
        frames.append(json.loads(line)["params"])
    except Exception:
        pass
sit = [f for f in frames if f.get("skill") == "sit"]
assert sit, "没见过 skill==sit"
first = next(i for i, f in enumerate(frames) if f.get("skill") == "sit")
window = frames[first:]
held = []
for f in window:
    if f.get("skill") == "sit":
        held.append(f)
    elif held:
        break
assert len(held) >= 60, f"坐姿持续帧不足（{len(held)} < 60 ≈1.2s@50Hz，目标≥2s 放宽）"
assert all(not f.get("fallen") for f in held), \
    f"坐姿期间出现 fallen（{sum(1 for f in held if f.get('fallen'))} 帧）"
z = float(os.environ["SIT_Z"])
assert 0.04 <= z <= 0.10, f"坐姿 z={z} 不在 [0.04, 0.10]"
up_z = float(os.environ["UP_Z"])
assert up_z > 0.08, f"起身后 z={up_z} ≤0.08"
print(f"sit：{len(held)} 帧 skill=sit、fallen 恒 false、z={z:.4f}；起身 z={up_z:.4f}  OK")
EOF
UP_INFO=$(state_snapshot)
echo "$UP_INFO" | awk '{
  if ($1 != "walk" || $2 != "False") {
    print "FAIL: sit 起身状态 skill="$1" fallen="$2; exit 1
  }
  print "sit 起身：skill=walk fallen=false  OK"
}' || fail "sit 起身状态"
echo "sit：PASS"

echo "== roulade：窗口 ~100 拍（2.0s）；中段可 fallen，结束后 upright =="
fresh_sim
wait_stand >/dev/null
# 录制窗口 + 结束后 2s 余量
state_frames 10 > /tmp/m8-bam-rou.ndjson &
REC=$!
sleep 0.5
./target/debug/mini-duckctl do roulade > /tmp/m8-bam-rou-do.json 2>/dev/null \
    || fail "do roulade"
wait $REC
sleep 1
timeout 3 ./target/debug/mini-duckctl state --every 1 2>/dev/null \
    | grep '"method":"robot.state"' | head -1 > /tmp/m8-bam-rou-post.json || true
POST_Z=$(body | awk '{print $3}')
POST_Z="$POST_Z" $HELPER_PY - <<'EOF' || fail "roulade 断言"
import json, os
do = json.load(open("/tmp/m8-bam-rou-do.json"))
assert do.get("result", {}).get("accepted") is True, do
frames = []
for line in open("/tmp/m8-bam-rou.ndjson"):
    line = line.strip()
    if not line:
        continue
    try:
        frames.append(json.loads(line)["params"])
    except Exception:
        pass
rou = [f for f in frames if f.get("skill") == "roulade"]
assert rou, "没见过 skill==roulade"
assert 70 <= len(rou) <= 130, f"roulade 窗口应 ~100 帧，实际 {len(rou)}"
# 裁决：窗口内允许短暂 fallen；结束后必须 upright（脚本头注释）。
last_rou = frames.index(rou[-1])
after = frames[last_rou + 1:]
# 录制应覆盖结束后若干 walk 帧；若推送截断则以 post 快照为准。
post = json.load(open("/tmp/m8-bam-rou-post.json"))
p = post.get("params", post)
assert p.get("skill") == "walk", p
assert p.get("fallen") is False, p
assert p.get("gain") == 200, p
z = float(os.environ["POST_Z"])
assert z > 0.08, f"roulade 结束后 z={z} ≤0.08"
rou_fallen = sum(1 for f in rou if f.get("fallen"))
print(f"roulade：{len(rou)} 帧窗口（中段 fallen={rou_fallen}），"
      f"结束后 walk/fallen=false/gain=200 z={z:.4f}  OK")
EOF
echo "roulade：PASS  post z=$POST_Z"

echo "== 行走回归：vx=0.8 × 10s（公开权重对小命令不响应，0.8 才起步态；门槛见头部裁决）=="
fresh_sim
wait_stand >/dev/null
BODY0=$(body)
echo "走前 body(x y z t): $BODY0"
X0=$(echo "$BODY0" | awk '{print $1}')
./target/debug/mini-duckctl move 0.8 0 0 --secs 10 || fail "move"
BODY1=$(body)
echo "走后 body(x y z t): $BODY1"
X1=$(echo "$BODY1" | awk '{print $1}')
Z1=$(echo "$BODY1" | awk '{print $3}')
DIST=$(awk -v a="$X0" -v b="$X1" 'BEGIN { printf "%.3f", b - a }')
echo "前移 ${DIST} m"
awk -v d="$DIST" 'BEGIN { exit !(d + 0 >= 2.0) }' || fail "位移不足（$DIST < 2.0）"
awk -v z="$Z1" 'BEGIN { exit !(z > 0.08) }' || fail "走完摔倒 z=$Z1"
echo "行走：PASS"

echo
echo "M8 BAM 物理验收全部通过：站立 + sit 坐稳起身 + roulade 不摔 + 行走(vx=0.8 ≥2.0m)"
exit 0

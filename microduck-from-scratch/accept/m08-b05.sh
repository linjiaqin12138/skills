#!/usr/bin/env bash
# M8 Bite 5 诊断：对照教程增益 vs 官方 no-bam 增益，验证 sit/roulade
# 摔倒归因是否仍指向 BAM 动力学缺失（D21/D34/D35）。不是验收门槛。
#
# 用法（宿主机）：docker compose exec rust bash scripts/accept-m8-bam-diag.sh
# 全程在容器内跑。sim 端口 7805（避开 accept-m4/m5/m6 的 7802/7803/7804）。
#
# 重要（Bite 6 后）：duck_body.py 默认开 BAM。本诊断测的是 XML PD 增益对照，
# 必须显式 --no-bam（否则 A/B 组增益 env 不生效，语义被 BAM 盖掉）。
#
# 两组增益（经 duck_body.py 读 MINIDUCK_SIM_KP / MINIDUCK_SIM_KV，仅 --no-bam）：
#   A：默认 kp=8 kv=0.25（M4 扫参站稳）
#   B：kp=0.55 kv=0（microduck_rl --no-bam XML chosen_actuator 原值）
#
# 整场 exit 0：脚本跑完并打出对照表即成功；B 组站立失败是合法证据，
# 不因此 exit 1。仅 daemon/sim 起不来才失败。
set -u
cd /work

SIM_PORT=7805
SIM_ADDR=127.0.0.1:$SIM_PORT
SIM_PID=""
DAEMON_PID=""
# 诊断可走系统 python3（--no-bam，无需 bam）；BAM 验收用 accept-m8-bam.sh。
SIM_PYTHON="${SIM_PYTHON:-python3}"
# 结果行：label|gain|stand|sit_skill|sit_fall|rou_fallen|note
RESULTS=()

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

fail_hard() { echo "FAIL: $1"; exit 1; }

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

# $1=标签 $2=可选 env 赋值串（空=默认增益）
fresh_sim() {
    local label="$1"
    local env_prefix="${2:-}"
    kill $DAEMON_PID 2>/dev/null; DAEMON_PID=""
    kill $SIM_PID 2>/dev/null; SIM_PID=""
    sleep 1
    rm -f /tmp/miniduckd.sock
    # 本诊断不需要画面；容器若缺 libOSMesa，默认 osmesa 后端会在 import 时报错。
    # MUJOCO_GL=disable 只跑物理（与 duck_body.py 的 setdefault 兼容：已有则不覆盖）。
    # 必须 --no-bam：Bite 6 后默认 BAM，否则 MINIDUCK_SIM_KP/KV 对照失效。
    # shellcheck disable=SC2086
    env MUJOCO_GL=disable $env_prefix "$SIM_PYTHON" sim/duck_body.py --no-bam \
        --port $SIM_PORT 2>/tmp/m8-bam-diag-sim.log &
    SIM_PID=$!
    sleep 3
    if ! kill -0 $SIM_PID 2>/dev/null; then
        fail_hard "sim 起不来（$label）：见 /tmp/m8-bam-diag-sim.log"
    fi
    ./target/debug/miniduckd --sim "$SIM_ADDR" 2>/tmp/m8-bam-diag-daemon.log &
    DAEMON_PID=$!
    sleep 2
    if ! kill -0 $DAEMON_PID 2>/dev/null; then
        fail_hard "daemon 起不来（$label）：见 /tmp/m8-bam-diag-daemon.log"
    fi
    ./target/debug/mini-duckctl enable >/dev/null || fail_hard "enable 失败（$label）"
}

wait_stand_attempt() {
    # 最多约 5s 看 z；再最多约 5s 看 skill=walk。不 fail。
    local Z="0" S="?"
    for i in $(seq 1 10); do
        Z=$(body | awk '{print $3}')
        awk -v z="$Z" 'BEGIN { exit !(z > 0.08) }' && break
        sleep 0.5
    done
    if awk -v z="$Z" 'BEGIN { exit !(z > 0.08) }'; then
        for i in $(seq 1 10); do
            S=$(state_frame | python3 -c "
import json, sys
print(json.loads(sys.stdin.read())['params']['skill'])" 2>/dev/null || echo "?")
            [ "$S" = "walk" ] && break
            sleep 0.5
        done
    fi
    echo "$Z $S"
}

# 跑一组诊断：$1=标签 $2=增益描述 $3=env 前缀
run_group() {
    local label="$1"
    local gain_desc="$2"
    local env_prefix="${3:-}"
    local stand="FAIL"
    local sit_skill="no"
    local sit_fall="none"
    local rou_fallen=0
    local note=""

    echo
    echo "===== 组 $label：$gain_desc ====="
    fresh_sim "$label" "$env_prefix"

    local stand_out Z S
    stand_out=$(wait_stand_attempt)
    Z=$(echo "$stand_out" | awk '{print $1}')
    S=$(echo "$stand_out" | awk '{print $2}')
    if awk -v z="$Z" 'BEGIN { exit !(z > 0.08) }'; then
        if [ "$S" = "walk" ]; then
            stand="PASS"
            echo "站立：$stand  body z=$Z  skill=$S"
        else
            stand="NOTE"
            note="z>0.08 但未进 Driving（skill=$S）"
            echo "站立：$stand  body z=$Z  skill=$S  ($note)"
        fi
    else
        stand="FAIL"
        note="站立失败 z=$Z（no-bam 增益下合法证据）"
        echo "站立：$stand  body z=$Z  ($note)"
    fi

    echo "-- sit_toggle --"
    state_frames 5 > /tmp/m8-bam-diag-sit.ndjson &
    local REC=$!
    sleep 0.5
    ./target/debug/mini-duckctl do sit_toggle > /tmp/m8-bam-diag-sit-do.json 2>/dev/null || true
    wait $REC
    # 补一次 body z，便于坐姿摔倒时对照躯干高度
    local SIT_Z
    SIT_Z=$(body | awk '{print $3}' 2>/dev/null || echo "?")
    local sit_info
    sit_info=$(SIT_Z="$SIT_Z" python3 - <<'EOF'
import json, os
accepted = None
try:
    do = json.load(open("/tmp/m8-bam-diag-sit-do.json"))
    accepted = do.get("result", {}).get("accepted")
except Exception:
    pass
frames = []
for line in open("/tmp/m8-bam-diag-sit.ndjson"):
    line = line.strip()
    if not line:
        continue
    try:
        frames.append(json.loads(line)["params"])
    except Exception:
        pass
sit = [f for f in frames if f.get("skill") == "sit"]
skill_ok = bool(sit)
fallen_at = next((i for i, f in enumerate(frames) if f.get("fallen")), None)
if fallen_at is not None:
    f = frames[fallen_at]
    gz = f.get("imu", {}).get("gravity", [None, None, None])[2]
    fall_s = f"frame={fallen_at} z≈{os.environ.get('SIT_Z','?')} gravity_z={gz}"
else:
    fall_s = "none"
print(f"{'yes' if skill_ok else 'no'}|{fall_s}|accepted={accepted}")
EOF
)
    sit_skill=$(echo "$sit_info" | cut -d'|' -f1)
    sit_fall=$(echo "$sit_info" | cut -d'|' -f2)
    echo "sit：skill=sit? $sit_skill；摔倒：$sit_fall；$(echo "$sit_info" | cut -d'|' -f3)"

    echo "-- roulade --"
    # sit 后可能已倒；重新起 sim 再测 roulade，避免污染
    fresh_sim "$label-rou" "$env_prefix"
    wait_stand_attempt >/dev/null

    state_frames 6 > /tmp/m8-bam-diag-rou.ndjson &
    REC=$!
    sleep 0.5
    ./target/debug/mini-duckctl do roulade > /tmp/m8-bam-diag-rou-do.json 2>/dev/null || true
    wait $REC
    local rou_info
    rou_info=$(python3 - <<'EOF'
import json
accepted = None
try:
    do = json.load(open("/tmp/m8-bam-diag-rou-do.json"))
    accepted = do.get("result", {}).get("accepted")
except Exception:
    pass
frames = []
for line in open("/tmp/m8-bam-diag-rou.ndjson"):
    line = line.strip()
    if not line:
        continue
    try:
        frames.append(json.loads(line)["params"])
    except Exception:
        pass
rou = [f for f in frames if f.get("skill") == "roulade"]
fallen_in_win = sum(1 for f in rou if f.get("fallen"))
any_fallen = any(f.get("fallen") for f in frames)
print(f"{fallen_in_win}|win_frames={len(rou)} any_fallen={any_fallen} accepted={accepted}")
EOF
)
    rou_fallen=$(echo "$rou_info" | cut -d'|' -f1)
    echo "roulade：窗口内 fallen 帧=$rou_fallen；$(echo "$rou_info" | cut -d'|' -f2)"

    RESULTS+=("$label|$gain_desc|$stand|$sit_skill|$sit_fall|$rou_fallen|$note")
}

python3 -c "import mujoco" 2>/dev/null || fail_hard "容器里缺 mujoco"
[ -x ./target/debug/miniduckd ] || fail_hard "缺 target/debug/miniduckd（先 cargo build）"
[ -x ./target/debug/mini-duckctl ] || fail_hard "缺 target/debug/mini-duckctl"

echo "M8 BAM 诊断：对照 kp=8/kv=0.25 vs kp=0.55/kv=0（端口 $SIM_PORT）"

run_group "A" "kp=8 kv=0.25（教程默认）" ""
run_group "B" "kp=0.55 kv=0（no-bam XML）" "MINIDUCK_SIM_KP=0.55 MINIDUCK_SIM_KV=0"

echo
echo "========== 对照表 =========="
printf "%-4s %-28s %-7s %-10s %-42s %-12s %s\n" \
    "组" "增益" "站立" "sit技能" "sit摔倒(时刻/z/gravity_z)" "rou fallen" "备注"
printf "%-4s %-28s %-7s %-10s %-42s %-12s %s\n" \
    "----" "----------------------------" "-------" "----------" "------------------------------------------" "------------" "----"
for row in "${RESULTS[@]}"; do
    IFS='|' read -r lab gain stand sit_sk sit_fa rou note <<<"$row"
    printf "%-4s %-28s %-7s %-10s %-42s %-12s %s\n" \
        "$lab" "$gain" "$stand" "$sit_sk" "$sit_fa" "$rou" "$note"
done
echo "============================"
echo
echo "结论提示：若 A 能站稳但 sit/roulade 仍摔、B 站立也失败，则摔倒归因更指向"
echo "BAM 动力学缺失（D21/D34/D35），而非「教程增益过大」；真正移植 BAM 是下一步。"
echo "本脚本为诊断，整场 exit 0（除非 sim/daemon 起不来）。"
exit 0

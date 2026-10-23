#!/usr/bin/env bash
# M8-C1 updater 验收：apply 预检 robot.safeToRestart / robot.modelApi。
#
# 用法（宿主机）：docker compose -f impl/docker-compose.yml exec rust bash /work/accept/m08-b04.sh
set -u
cd "$(cd "$(dirname "$0")/../impl" && pwd)"

ROOT=/tmp/m8-updater-demo
UPDSOCK=/tmp/mini-updaterd.sock
DUCKSOCK=/tmp/miniduckd.sock
export MINIDUCK_UPDATER_ROOT=$ROOT
export MINIDUCK_UPDATER_GATE_MS=4000
export MINIDUCK_POLICY=$PWD/policies/velstand.onnx
export MINIDUCK_UPDATER_SOCK=$UPDSOCK

PASS=0
FAILED=0
UPD_PID=""
MOVE_PID=""

ok()   { echo "PASS: $1"; PASS=$((PASS+1)); }
fail() { echo "FAIL: $1"; FAILED=$((FAILED+1)); }
fatal(){ echo "FATAL: $1"; summary; exit 1; }
summary() {
    echo
    echo "===== M8-C1 updater 汇总：PASS=$PASS FAIL=$FAILED ====="
}

cleanup_procs() {
    [ -n "$MOVE_PID" ] && kill -9 "$MOVE_PID" 2>/dev/null
    [ -n "$UPD_PID" ] && kill -9 "$UPD_PID" 2>/dev/null
    pkill -9 -f "target/debug/mini-updaterd" 2>/dev/null
    pkill -9 -f "bin/miniduckd" 2>/dev/null
    pkill -9 -x miniduckd 2>/dev/null
    pkill -9 -f miniduckd-real 2>/dev/null
}
cleanup() { cleanup_procs; rm -rf "$ROOT"; rm -f "$UPDSOCK" "$DUCKSOCK"; }
trap cleanup EXIT
cleanup
sleep 0.5

echo "===== 0. 构建 ====="
cargo build 2>&1 | tail -1

# ---- 造 release ----
make_release() {
    local ver=$1 kind=$2
    local stage=$ROOT/.mk-$ver
    rm -rf "$stage"; mkdir -p "$stage/bin" "$ROOT/source"
    if [ "$kind" = real ]; then
        cp target/debug/miniduckd "$stage/bin/miniduckd"
    else
        cp target/debug/miniduckd "$stage/bin/miniduckd-real"
        cat > "$stage/bin/miniduckd" <<'EOF'
#!/bin/sh
DIR=$(dirname "$0")
export MINIDUCK_POLICY=/nonexistent.onnx
exec "$DIR/miniduckd-real"
EOF
        chmod +x "$stage/bin/miniduckd"
    fi
    tar -C "$stage" -cf "$ROOT/source/$ver.tar" .
    local sha
    sha=$(sha256sum "$ROOT/source/$ver.tar" | awk '{print $1}')
    printf '{"version":"%s","artifact":"%s.tar","sha256":"%s"}\n' "$ver" "$ver" "$sha" \
        > "$ROOT/source/$ver.manifest.json"
    rm -rf "$stage"
}

make_release 1.0.0 real
make_release 2.0.0 real

# bootstrap「出厂镜像」
mkdir -p "$ROOT/install/releases/1.0.0" "$ROOT/state"
tar -xf "$ROOT/source/1.0.0.tar" -C "$ROOT/install/releases/1.0.0"
ln -s releases/1.0.0 "$ROOT/install/current"
echo "release 1.0.0/2.0.0 就绪，出厂 current=1.0.0"

start_updaterd() {
    rm -f "$UPDSOCK"
    ./target/debug/mini-updaterd 2>>/tmp/m8-updaterd.log &
    UPD_PID=$!
    disown
    for _ in $(seq 1 80); do [ -S "$UPDSOCK" ] && return 0; sleep 0.25; done
    fatal "updaterd 没起来（/tmp/m8-updaterd.log）"
}

upd() { ./target/debug/mini-duckctl update "$@"; }

wait_health() {
    for i in $(seq 1 "$1"); do
        H=$(./target/debug/mini-duckctl health 2>/dev/null | python3 -c "
import json, sys
try: print(json.loads(sys.stdin.read())['result']['healthy'])
except Exception: pass" 2>/dev/null)
        [ "$H" = "True" ] && return 0
        sleep 1
    done
    return 1
}

wait_driving() {
    timeout 10 ./target/debug/mini-duckctl subscribe --every 1 2>/dev/null | python3 -c "
import json, sys
for line in sys.stdin:
    try:
        p = json.loads(line).get('params', {})
        if p.get('skill') == 'walk':
            sys.exit(0)
    except Exception:
        pass
sys.exit(1)
" 2>/dev/null
}

current_target() { readlink "$ROOT/install/current"; }

echo
echo "===== 场景：走路时 apply 被 safeToRestart 拒绝 ====="
start_updaterd
wait_health 15 || fatal "出厂 daemon 没健康"

# 让机器人进入 Driving（skill=walk）。
./target/debug/mini-duckctl enable on >/dev/null || fatal "enable on 失败"
wait_driving || fatal "没进入 Driving"
ok "robot 进入 Driving"

# 后台持续发 move 意图，让 robot 处于 walking 状态。
./target/debug/mini-duckctl move 0.1 0 0 --secs 30 >/tmp/m8-move.log 2>&1 &
MOVE_PID=$!
disown
sleep 0.3

# 此时 safeToRestart 应为 false。
SAFE=$(python3 - <<'PY'
import json, socket
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.settimeout(3)
s.connect("/tmp/miniduckd.sock")
f = s.makefile("rw")
f.write(json.dumps({"jsonrpc":"2.0","id":1,"method":"hello"})+"\n"); f.flush()
f.readline()
f.write(json.dumps({"jsonrpc":"2.0","id":2,"method":"robot.safeToRestart","params":{}})+"\n"); f.flush()
print(json.loads(f.readline())["result"]["safe"])
PY
)
[ "$SAFE" = "False" ] && ok "走路时 safeToRestart=false" || fail "走路时 safeToRestart=$SAFE"

# apply 2.0.0 应因 not safe to restart 被拒绝，且 current 不动。
upd apply 2.0.0 | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())
err = r.get('error') or {}
assert err.get('code') == -32003, r
assert 'not safe to restart' in err.get('message', ''), r
print(f\"拒绝: {err['message'][:80]}\")
" && ok "apply 被 -32003 拒绝" || fail "apply 未被拒绝"
[ "$(current_target)" = "releases/1.0.0" ] && ok "拒绝后 current 保持 1.0.0" || fail "current=$(current_target)"

# 停止 move 意图，等 deadman 让机器人停下来。
kill -9 "$MOVE_PID" 2>/dev/null; MOVE_PID=""
./target/debug/mini-duckctl move 0 0 0 >/dev/null
sleep 0.7

# 再次 apply 应成功 committed。
upd apply 2.0.0 | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['outcome'] == 'committed' and r['to'] == '2.0.0', r
print(f\"committed {r['from']} -> {r['to']}\")
" && ok "静止后 apply 2.0.0 成功" || fail "静止后 apply 失败"
[ "$(current_target)" = "releases/2.0.0" ] && ok "current->2.0.0" || fail "current=$(current_target)"
wait_health 15 || fatal "升级后 daemon 不健康"

echo
echo "===== 场景：daemon 不可达时 apply 不被 model_api 拒（D65）====="
# 原版语义（upstream/updater/src/policy.rs:1026-1039）：model_api 未知
# （None）= 兼容，只在已知版本不匹配时才拒。杀掉 daemon 让两个预检都
# 拿到不可达/None，apply 1.0.0 应照常走完：swap + respawn + 健康门 → committed。
pkill -9 -x miniduckd; sleep 0.5
upd apply 1.0.0 | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())
err = r.get('error') or {}
assert 'model api' not in err.get('message', ''), f'仍被 model_api 拒: {r}'
res = r.get('result') or {}
assert res.get('outcome') == 'committed' and res.get('to') == '1.0.0', r
print(f\"daemon 不可达也 committed: {res['from']} -> {res['to']}\")
" && ok "daemon 不可达时 apply 1.0.0 不被 model_api 拒且 committed" \
  || fail "daemon 不可达时 apply 被拒或失败"
[ "$(current_target)" = "releases/1.0.0" ] && ok "current->1.0.0" || fail "current=$(current_target)"

echo
summary
[ "$FAILED" = 0 ]

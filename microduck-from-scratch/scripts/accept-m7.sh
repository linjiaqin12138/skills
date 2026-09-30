#!/usr/bin/env bash
# M7 验收：mini-updaterd（迷你 updater）。一键可重跑。
#
# 用法（宿主机）：docker compose exec rust bash scripts/accept-m7.sh
# 全程在容器内跑，MINIDUCK_UPDATER_ROOT=/tmp/m7-demo 全程隔离。
#
# 断言清单：
#   场景 A 开心路径：出厂 1.0.0 → updaterd 自起 daemon → check 列出 2.0.0 →
#     apply 2.0.0 committed → status idle/current=2.0.0 → log Committed →
#     daemon healthy；手动 rollback 回 1.0.0 再 apply 回来
#   场景 B 投毒回滚：apply 3.0.0-bad（wrapper 把 MINIDUCK_POLICY 指向不存在，
#     靠 D19 报病，不改 miniduckd 一行）→ 门否决 → 自动回滚 2.0.0 →
#     daemon 恢复 healthy → log RolledBack
#   场景 C 篡改拒绝：tar 追加一个字节 → sha256 不符 → apply 被拒 →
#     current 没动、releases/ 无残留、staging 清空、log Rejected
#   场景 D 并发：apply 进行中再发 apply → 第二个 Busy
#   场景 E 崩溃恢复：E1 EXIT_AFTER_SWAP=1 apply 健康 2.0.1，updaterd 死后
#     daemon 仍在跑 → 重启 → pending 驱动重跑门 → confirm；E2 同注入 apply
#     投毒版 → 重启后裁决回滚 previous
set -u
cd /work

ROOT=/tmp/m7-demo
UPDSOCK=/tmp/mini-updaterd.sock
DUCKSOCK=/tmp/miniduckd.sock
export MINIDUCK_UPDATER_ROOT=$ROOT
export MINIDUCK_UPDATER_GATE_MS=4000          # 测试预算（生产默认 30s）
export MINIDUCK_POLICY=/work/policies/velstand.onnx  # 让 spawn 的 daemon 健康

PASS=0
FAILED=0
UPD_PID=""

ok()   { echo "PASS: $1"; PASS=$((PASS+1)); }
fail() { echo "FAIL: $1"; FAILED=$((FAILED+1)); }
fatal(){ echo "FATAL: $1"; summary; exit 1; }
summary() {
    echo
    echo "===== M7 汇总：PASS=$PASS FAIL=$FAILED ====="
}

cleanup_procs() {
    [ -n "$UPD_PID" ] && kill -9 "$UPD_PID" 2>/dev/null
    pkill -f "target/debug/mini-updaterd" 2>/dev/null
    pkill -f "bin/miniduckd" 2>/dev/null
    pkill -x miniduckd 2>/dev/null
    pkill -f miniduckd-real 2>/dev/null
}
cleanup() { cleanup_procs; rm -rf "$ROOT"; rm -f "$UPDSOCK" "$DUCKSOCK"; }
trap cleanup EXIT
cleanup
sleep 0.5

echo "===== 0. 构建 + 单测 ====="
cargo build 2>&1 | tail -1
cargo test 2>&1 | tee /tmp/m7-test0.log | grep -E "^test result" | head -1
grep -q "test result: ok" /tmp/m7-test0.log || fatal "cargo test 不绿"
grep -q "0 failed" /tmp/m7-test0.log || fatal "cargo test 有失败"

# ---- 造 release ----
# $1=版本 $2=real|bad。tar 里放 bin/miniduckd（真二进制副本或投毒 wrapper）。
make_release() {
    local ver=$1 kind=$2
    local stage=$ROOT/.mk-$ver
    rm -rf "$stage"; mkdir -p "$stage/bin" "$ROOT/source"
    if [ "$kind" = real ]; then
        cp target/debug/miniduckd "$stage/bin/miniduckd"
    else
        # 投毒版：wrapper 指向不存在的策略（D19：加载失败不退出、报病）。
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
make_release 2.0.1 real
make_release 3.0.0-bad bad

# bootstrap「出厂镜像」：直接解包 + 建 symlink（模拟工厂装机，无 log 行）。
mkdir -p "$ROOT/install/releases/1.0.0" "$ROOT/state"
tar -xf "$ROOT/source/1.0.0.tar" -C "$ROOT/install/releases/1.0.0"
ln -s releases/1.0.0 "$ROOT/install/current"
echo "release 1.0.0/2.0.0/2.0.1/3.0.0-bad 就绪，出厂 current=1.0.0"

# ---- 助手 ----
# INJECT=1 前缀启动带 EXIT_AFTER_SWAP 注入的 updaterd。
start_updaterd() {
    # 清掉上次 updaterd 残留的 socket 文件，否则下面的等待循环会被
    # 旧文件骗到（新 updaterd 还没 bind 就放行，客户端 ECONNREFUSED）。
    rm -f "$UPDSOCK"
    MINIDUCK_UPDATER_EXIT_AFTER_SWAP="${INJECT:-0}" ./target/debug/mini-updaterd \
        2>>/tmp/m7-updaterd.log &
    UPD_PID=$!
    # 有 pending 时恢复（含最多 ~2×gate 预算）跑完才 bind，给足 20s。
    for _ in $(seq 1 80); do [ -S "$UPDSOCK" ] && return 0; sleep 0.25; done
    fatal "updaterd 没起来（/tmp/m7-updaterd.log）"
}

upd() { ./target/debug/mini-duckctl update "$@" 2>/dev/null; }

# 等 miniduckd 健康（$1=秒数上限）；stdout 空=超时未健康。
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

current_target() { readlink "$ROOT/install/current"; }

echo
echo "===== 场景 A：开心路径 ====="
start_updaterd

echo "== A1. updaterd 自起 daemon，mini-duckctl state 通 =="
wait_health 15 || fatal "A1: 出厂 daemon 没健康"
F=$(timeout 3 ./target/debug/mini-duckctl state 2>/dev/null | head -1)
if [ -n "$F" ]; then ok "A1 daemon 自起且 state 有帧"; else fail "A1 state 无帧"; fi

echo "== A2. update.check 列出可用版本与 current =="
upd check | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert '2.0.0' in r['available'], r
assert '3.0.0-bad' in r['available'], r
assert r['current'] == '1.0.0', r
print(f'available={r[\"available\"]} current={r[\"current\"]}')" && ok "A2 check" || fail "A2 check"

echo "== A3. apply 2.0.0 → committed =="
upd apply 2.0.0 | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['outcome'] == 'committed' and r['to'] == '2.0.0', r
print(f'committed {r[\"from\"]} -> {r[\"to\"]}')" && ok "A3 apply 2.0.0" || fail "A3 apply 2.0.0"
[ "$(current_target)" = "releases/2.0.0" ] && ok "A3 current->2.0.0" || fail "A3 current=$(current_target)"
[ ! -f "$ROOT/state/pending.json" ] && ok "A3 pending 已清" || fail "A3 pending 残留"

echo "== A4. status / log / 新 daemon healthy =="
upd status | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['phase'] == 'idle', r
assert r['current'] == '2.0.0', r
assert r['previous'] == '1.0.0', r
assert r['pending'] is None, r" && ok "A4 status idle/current=2.0.0/previous=1.0.0" || fail "A4 status"
upd log | python3 -c "
import json, sys
e = json.loads(sys.stdin.read())['result']['entries']
last = e[-1]
assert last['outcome'] == 'committed' and last['to'] == '2.0.0' and last['from'] == '1.0.0', last
assert last.get('peer_uid') is not None, last
print(f'log 末行: {last[\"outcome\"]} {last[\"from\"]}->{last[\"to\"]} peer_uid={last[\"peer_uid\"]}')" \
    && ok "A4 log Committed（带 peer_uid）" || fail "A4 log"
wait_health 15 && ok "A4 新 daemon healthy" || fail "A4 新 daemon 不健康"

echo "== A5. 手动 rollback 回 1.0.0，再 apply 回 2.0.0（给 B 铺垫）=="
upd rollback | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['outcome'] == 'rolled_back' and r['to'] == '1.0.0', r" && ok "A5 rollback RPC" || fail "A5 rollback"
[ "$(current_target)" = "releases/1.0.0" ] && ok "A5 current->1.0.0" || fail "A5 current=$(current_target)"
upd apply 2.0.0 >/dev/null && [ "$(current_target)" = "releases/2.0.0" ] \
    && ok "A5 重回 2.0.0" || fail "A5 重回 2.0.0"
wait_health 15 || fatal "A5 回 2.0.0 后 daemon 不健康"

echo
echo "===== 场景 B：投毒回滚 ====="
upd apply 3.0.0-bad | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())['result']
assert r['outcome'] == 'rolled_back' and r['to'] == '3.0.0-bad', r
print(f'门否决: {r[\"reason\"][:80]}')" && ok "B1 门否决自动回滚" || fail "B1 apply 3.0.0-bad"
[ "$(current_target)" = "releases/2.0.0" ] && ok "B2 current 回到 2.0.0" || fail "B2 current=$(current_target)"
# 回归断言：门否决回滚后 pending 必须清（曾经漏清，重启会重复裁决已结束的 trial）。
[ ! -f "$ROOT/state/pending.json" ] && ok "B2b pending 已清" || fail "B2b pending 残留"
wait_health 15 && ok "B3 daemon 恢复 healthy" || fail "B3 daemon 不健康"
upd log | python3 -c "
import json, sys
e = json.loads(sys.stdin.read())['result']['entries']
rb = [x for x in e if x['outcome'] == 'rolled_back' and x['to'] == '3.0.0-bad' and x['from'] == '2.0.0']
assert rb, e
print(f'RolledBack 行: {rb[-1][\"reason\"][:80]}')" && ok "B4 log RolledBack(2.0.0→3.0.0-bad)" || fail "B4 log"

echo
echo "===== 场景 C：篡改拒绝 ====="
# B 已经把 3.0.0-bad 装进 releases/（装是成功的，是门否决了它）。C 断
# 言的是"篡改的 apply 不创建版本目录"，所以先手动卸掉它模拟干净局面。
rm -rf "$ROOT/install/releases/3.0.0-bad"
printf 'x' >> "$ROOT/source/3.0.0-bad.tar"   # 改一个字节，sha256 不符
upd apply 3.0.0-bad | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())
err = r.get('error') or {}
assert err.get('code') == -32003 and 'sha256 mismatch' in err.get('message', ''), r
print(f'拒绝: {err[\"message\"][:70]}')" && ok "C1 sha256 不符被拒" || fail "C1 未被拒"
[ "$(current_target)" = "releases/2.0.0" ] && ok "C2 current 没动" || fail "C2 current=$(current_target)"
[ ! -e "$ROOT/install/releases/3.0.0-bad" ] && ok "C3 releases/ 无残留" || fail "C3 releases 有残留"
if ls "$ROOT/install"/.staging-* >/dev/null 2>&1; then fail "C4 staging 残留"; else ok "C4 staging 已清"; fi
upd log | python3 -c "
import json, sys
e = json.loads(sys.stdin.read())['result']['entries']
assert e[-1]['outcome'] == 'rejected' and e[-1]['to'] == '3.0.0-bad', e[-1]" \
    && ok "C5 log Rejected" || fail "C5 log"
make_release 3.0.0-bad bad   # 修好，E2 还要用

echo
echo "===== 场景 D：并发单飞 ====="
upd apply 2.0.0 > /tmp/m7-d-first.json 2>&1 &
BG=$!
sleep 1   # 第一个 apply 已进入持锁段（verify/extract/swap/gate 任一）
upd apply 2.0.1 | python3 -c "
import json, sys
r = json.loads(sys.stdin.read())
err = r.get('error') or {}
assert err.get('code') == -32001 and 'Busy' in err.get('message', ''), r
print(f'第二个 apply 被拒: {err[\"message\"]}')" && ok "D1 并发 apply 拿 Busy" || fail "D1 没有 Busy"
wait $BG
python3 -c "
import json
r = json.loads(open('/tmp/m7-d-first.json').read())['result']
assert r['outcome'] == 'committed', r" && ok "D2 第一个 apply 正常完成" || fail "D2 第一个 apply"
wait_health 15 || fatal "D 后 daemon 不健康"

echo
echo "===== 场景 E：崩溃恢复 ====="
echo "== E0. kill -9 updaterd 不杀 daemon =="
kill -9 $UPD_PID; wait $UPD_PID 2>/dev/null; UPD_PID=""
sleep 0.5
wait_health 5 && ok "E0 kill -9 updaterd 后 daemon 仍在跑" || fail "E0 daemon 被带走了"

echo "== E1. EXIT_AFTER_SWAP apply 健康 2.0.1 → 重启 confirm =="
INJECT=1 start_updaterd
# apply 会因 updaterd exit(1) 拿不到响应——客户端 EOF 是预期行为。
upd apply 2.0.1 >/dev/null 2>&1
sleep 1
if kill -0 $UPD_PID 2>/dev/null; then fail "E1 updaterd 没按注入死掉"; else ok "E1 updaterd swap 后 exit(1)"; fi
UPD_PID=""
wait_health 10 && ok "E1 新 daemon(2.0.1) 仍在跑" || fail "E1 daemon 没在跑"
python3 -c "
import json
p = json.load(open('$ROOT/state/pending.json'))
assert p['version'] == '2.0.1' and p['previous'] == '2.0.0' and p['boots'] == 0, p
print(f'pending: {p}')" && ok "E1 pending 已 arm(boots=0)" || fail "E1 pending"
[ "$(current_target)" = "releases/2.0.1" ] && ok "E1 current->2.0.1" || fail "E1 current=$(current_target)"

INJECT= start_updaterd   # 重启无注入：pending 驱动重跑门 → confirm
[ ! -f "$ROOT/state/pending.json" ] && ok "E1 重启后 pending 已 confirm" || fail "E1 pending 未清"
[ "$(current_target)" = "releases/2.0.1" ] && ok "E1 current 保持 2.0.1" || fail "E1 current=$(current_target)"
upd log | python3 -c "
import json, sys
e = json.loads(sys.stdin.read())['result']['entries']
last = e[-1]
assert last['outcome'] == 'committed' and last['to'] == '2.0.1', last
assert 'recovered' in (last.get('reason') or ''), last
print(f'恢复记账: {last[\"reason\"][:60]}')" && ok "E1 log 恢复后 Committed" || fail "E1 log"
wait_health 15 || fatal "E1 重启后 daemon 不健康"

echo "== E2. EXIT_AFTER_SWAP apply 投毒版 → 重启裁决回滚 =="
kill -9 $UPD_PID; wait $UPD_PID 2>/dev/null; UPD_PID=""
sleep 0.5
INJECT=1 start_updaterd
upd apply 3.0.0-bad >/dev/null 2>&1
sleep 2
kill -0 $UPD_PID 2>/dev/null && fail "E2 updaterd 没按注入死掉" || ok "E2 updaterd swap 后 exit(1)"
UPD_PID=""
# 投毒 daemon 活着但报病（D19 行为，证明换过去的真是坏版）。
H=$(./target/debug/mini-duckctl health 2>/dev/null | python3 -c "
import json, sys
print(json.loads(sys.stdin.read())['result']['healthy'])" 2>/dev/null)
[ "$H" = "False" ] && ok "E2 投毒 daemon 报病（D19）" || fail "E2 投毒 daemon health=$H"
python3 -c "
import json
p = json.load(open('$ROOT/state/pending.json'))
assert p['version'] == '3.0.0-bad' and p['previous'] == '2.0.1', p" && ok "E2 pending(3.0.0-bad)" || fail "E2 pending"

INJECT= start_updaterd   # 重启：门否决投毒版 → 回滚 2.0.1（bind 前跑完）
[ ! -f "$ROOT/state/pending.json" ] && ok "E2 重启后 pending 已清" || fail "E2 pending 未清"
[ "$(current_target)" = "releases/2.0.1" ] && ok "E2 current 回滚到 2.0.1" || fail "E2 current=$(current_target)"
upd log | python3 -c "
import json, sys
e = json.loads(sys.stdin.read())['result']['entries']
rb = [x for x in e if x['outcome'] == 'rolled_back' and x['to'] == '3.0.0-bad' and x['from'] == '2.0.1']
assert rb, e
print(f'恢复回滚: {rb[-1][\"reason\"][:70]}')" && ok "E2 log RolledBack(2.0.1→3.0.0-bad)" || fail "E2 log"
wait_health 15 && ok "E2 回滚后 daemon healthy" || fail "E2 daemon 不健康"

echo
echo "===== 收尾：cargo test 复核 ====="
cargo test 2>&1 | grep -E "^test result" | head -1
cargo test 2>&1 | grep -q "0 failed" && ok "cargo test 全绿" || fail "cargo test"

summary
[ "$FAILED" = 0 ]

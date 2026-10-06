#!/usr/bin/env bash
# M8 验收：接口对齐。一键可重跑。
#
# 用法（宿主机）：docker compose exec rust bash scripts/accept-m8.sh
# 全程在容器内跑。FakeIo（miniduckd 无 --sim 即 FakeIo，不需要仿真），
# socket 同一文件（同一时刻只跑一个 daemon）。
#
# 本 Bite 断言（robot.drive → 原版语义 robot.move）：
#   a. 无 id 通知 robot.move → state 推送 obs twist == [vx, vy, vyaw]
#   b. 通知不产生任何响应行（推送帧除外）
#   c. 带 id → 响应 accepted:true + 三速度回显
#   d. 带 id 缺省 vy → 按 0 接受
#   e. 未知字段：带 id → INVALID_PARAMS；无 id → 静默且 twist 不被改动
#   f. 非有限值 1e999 → serde_json 解析阶段拒（PARSE_ERROR，id null）
#   g. deadman 回归：停发 ≥600ms 后 obs twist 归零
#   h. robot.drive → METHOD_NOT_FOUND
set -u
cd /work

DAEMON_PID=""

cleanup() {
    [ -n "$DAEMON_PID" ] && kill "$DAEMON_PID" 2>/dev/null
    pkill -x miniduckd 2>/dev/null
    rm -f /tmp/miniduckd.sock
}
trap cleanup EXIT
cleanup
sleep 1

./target/debug/miniduckd 2>/tmp/m8-daemon.log &
DAEMON_PID=$!
sleep 1
[ -S /tmp/miniduckd.sock ] || { echo "FAIL: daemon 没起来（/tmp/m8-daemon.log）"; exit 1; }

# 全部断言在一条连接上按时间线走完（notification 语义要看帧间关系，
# 分段重连反而说不清）。发帧用 python3 裸 socket——mini-duckctl 只会发
# 请求式帧，发不了无 id 通知（与 accept-m5.sh 的 python 助手同款做法）。
python3 - <<'PY'
import json, socket, time

s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.settimeout(5)
s.connect("/tmp/miniduckd.sock")
f = s.makefile("rw")

def send(obj):
    f.write(json.dumps(obj) + "\n"); f.flush()

def readline():
    line = f.readline()
    assert line, "daemon 断了连接"
    return json.loads(line)

def read_until_response(rid, budget=5.0):
    # 读帧直到拿到 id==rid 的响应；途中只许是 robot.state 推送。
    deadline = time.time() + budget
    while time.time() < deadline:
        msg = readline()
        if msg.get("id") == rid and ("result" in msg or "error" in msg):
            return msg
        assert msg.get("method") == "robot.state", f"等响应时混进意外帧: {msg}"
    raise AssertionError(f"等 id={rid} 响应超时")

def read_pushes(secs):
    # 收 secs 秒的帧，断言全部是推送（无响应行），返回帧列表。
    deadline = time.time() + secs
    frames = []
    while time.time() < deadline:
        msg = readline()
        assert msg.get("method") == "robot.state", f"出现意外响应行: {msg}"
        frames.append(msg["params"])
    return frames

def twist_of(params):
    # obs twist 三维的偏移 OFF_TWIST=48（src/obs.rs）。
    return params["obs"][48:51]

def close(a, b, tol=1e-6):
    # obs 里是 f32 落盘的值，和 f64 字面量有 ~1e-8 差，容差 1e-6。
    return all(abs(x - y) < tol for x, y in zip(a, b))

# 入场第一件事：hello + 订阅 robot.state（协议规定，lib.rs 注释）。
send({"jsonrpc": "2.0", "id": 1, "method": "hello"})
r = read_until_response(1)
assert r["result"]["service"] == "miniduckd", r
send({"jsonrpc": "2.0", "id": 2, "method": "robot.state"})
r = read_until_response(2)
assert r["result"]["subscribed"] is True, r
print("握手 + 订阅  OK")

# ── 断言 a + b：无 id 通知静默生效 ──
send({"jsonrpc": "2.0", "method": "robot.move",
      "params": {"vx": 0.1, "vy": 0.2, "vyaw": -0.3}})
deadline = time.time() + 1.0
hit = None
while time.time() < deadline and hit is None:
    msg = readline()
    # 断言 b：通知不应换来任何响应行，这条连接上只许有推送。
    assert msg.get("method") == "robot.state", f"通知产生了响应行: {msg}"
    t = twist_of(msg["params"])
    if close(t, [0.1, 0.2, -0.3]):
        hit = t
assert hit, "1s 内未见 twist=[0.1, 0.2, -0.3] 的推送"
print(f"a. 通知生效 twist={hit}  OK")
print("b. 通知全程无响应行  OK")

# ── 断言 c：带 id → accepted + 回显 ──
send({"jsonrpc": "2.0", "id": 3, "method": "robot.move",
      "params": {"vx": 0.05, "vy": -0.05, "vyaw": 0.1}})
r = read_until_response(3)
res = r.get("result", {})
assert res.get("accepted") is True, r
assert close([res["vx"], res["vy"], res["vyaw"]], [0.05, -0.05, 0.1]), r
print(f"c. 请求式 accepted 回显 {[res['vx'], res['vy'], res['vyaw']]}  OK")

# ── 断言 d：缺省 vy 按 0 ──
send({"jsonrpc": "2.0", "id": 4, "method": "robot.move",
      "params": {"vx": 0.1, "vyaw": 0.2}})
r = read_until_response(4)
res = r.get("result", {})
assert res.get("accepted") is True and res.get("vy") == 0.0, r
print(f"d. 缺省 vy 按 0 接受 {res}  OK")

# ── 断言 e1：带 id 未知字段 → INVALID_PARAMS ──
send({"jsonrpc": "2.0", "id": 5, "method": "robot.move",
      "params": {"vx": 0.1, "vz": 9}})
r = read_until_response(5)
err = r.get("error") or {}
assert err.get("code") == -32602, f"未知字段应回 INVALID_PARAMS，实际 {r}"
print(f"e1. 未知字段 INVALID_PARAMS: {err['message'][:60]}  OK")

# ── 断言 e2：无 id 同参数 → 静默，twist 不被改动 ──
# 先发一帧合法通知刷新意图与 deadman 钟，确认它落了，再发非法通知，
# 随后 300ms（< deadman 500ms）内 twist 必须恒为合法帧的值。
send({"jsonrpc": "2.0", "method": "robot.move",
      "params": {"vx": 0.3, "vy": 0.1, "vyaw": -0.2}})
deadline = time.time() + 1.0
while True:
    msg = readline()
    assert msg.get("method") == "robot.state", msg
    if close(twist_of(msg["params"]), [0.3, 0.1, -0.2]):
        break
    assert time.time() < deadline, "合法通知没生效"
send({"jsonrpc": "2.0", "method": "robot.move",
      "params": {"vx": 0.1, "vz": 9}})
frames = read_pushes(0.3)
assert frames, "没收到推送"
bad = [twist_of(p) for p in frames if not close(twist_of(p), [0.3, 0.1, -0.2])]
assert not bad, f"非法通知改动了 twist: {bad[:3]}"
print(f"e2. 非法通知静默丢弃，{len(frames)} 帧 twist 保持 [0.3, 0.1, -0.2]  OK")

# ── 断言 f：1e999 在 JSON 解析阶段就被拒（PARSE_ERROR，id null）──
# serde_json 对超出 f64 的字面量直接报 "number out of range"（单测
# move_params_json_overflow_rejected_at_parse 锁定该行为）——JSON 线路
# 根本造不出 inf/NaN 的 params，daemon 的有限值检查是深度防御。
# 所以这帧死在整个 Request 解析上：PARSE_ERROR 且按规范 id 为 null。
# 注意要手写帧字面量：python 的 json.dumps 会把 inf 印成裸 Infinity
# （那不是 JSON），测的就成了词法错误而非数值溢出。
f.write('{"jsonrpc":"2.0","id":6,"method":"robot.move",'
        '"params":{"vx":1e999,"vy":0,"vyaw":0}}\n'); f.flush()
deadline = time.time() + 2.0
while True:
    msg = readline()
    if "error" in msg:
        break
    assert time.time() < deadline, "没等到 PARSE_ERROR"
assert msg["id"] is None and msg["error"]["code"] == -32700, \
    f"1e999 应回 PARSE_ERROR + id null，实际 {msg}"
assert "out of range" in msg["error"]["message"], msg
print(f"f. 1e999 → PARSE_ERROR id=null: {msg['error']['message'][:50]}  OK")

# ── 断言 g：deadman 回归，停发 ≥600ms 后 twist 归零 ──
time.sleep(0.7)
frames = read_pushes(0.1)
assert frames, "没收到推送"
t = twist_of(frames[-1])
assert close(t, [0.0, 0.0, 0.0], tol=1e-12), f"停发 0.7s 后 twist 应归零，实际 {t}"
print("g. 停发 0.7s 后 obs twist=[0,0,0]（deadman 生效）  OK")

# ── 断言 h：robot.drive 已死 ──
send({"jsonrpc": "2.0", "id": 7, "method": "robot.drive",
      "params": {"vx": 0.1, "vyaw": 0}})
r = read_until_response(7)
err = r.get("error") or {}
assert err.get("code") == -32601, f"robot.drive 应回 METHOD_NOT_FOUND，实际 {r}"
print(f"h. robot.drive → METHOD_NOT_FOUND: {err['message']}  OK")

print()
print("M8（本 Bite）断言 a–h 全部通过")
PY
RC=$?

echo
if [ "$RC" = 0 ]; then
    echo "M8 验收（robot.move 对齐）全部通过"
else
    echo "FAIL: 断言脚本退出码 $RC"
fi
exit "$RC"

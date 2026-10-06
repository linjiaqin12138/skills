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
# Bite 2 断言（robot.enable {on, toggle} + notification 统一入口）：
#   i. robot.enable {on:true} 带 id → accepted + reason 文案，推送 enabled=true
#   j. 无 id robot.head → 静默生效（obs[51:55] 变为目标值）
#   k. 无 id robot.mouth → 静默生效（positions[9] 朝 +30° 移动，需 Driving）
#   l. 无 id 非意图方法（robot.health）→ 静默丢弃
#   m. 带 id robot.head/robot.mouth → 请求式回显不变
#   n. robot.enable toggle:true → 翻转（开→关），推送 enabled=false；
#      且不卸 torque、当拍直接回 home（无斜坡，Stopped 上电抱持）
#   o. {on:true, toggle:true} → toggle 优先（忽略 on），关→开；
#      且 Stopped→Driving 无斜坡（1s 内 skill=walk）
#   p. enable 未知字段 → INVALID_PARAMS；robot.disable → METHOD_NOT_FOUND
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

def wait_push(pred, what, budget=2.0):
    # 读推送直到 pred(params) 为真；途中只许是 robot.state 推送。
    deadline = time.time() + budget
    while time.time() < deadline:
        msg = readline()
        assert msg.get("method") == "robot.state", f"等推送时混进意外帧: {msg}"
        if pred(msg["params"]):
            return msg["params"]
    raise AssertionError(f"{budget}s 内未等到：{what}")

# ── 断言 i：robot.enable {on:true} → accepted + reason，推送 enabled=true ──
send({"jsonrpc": "2.0", "id": 8, "method": "robot.enable", "params": {"on": True}})
r = read_until_response(8)
res = r.get("result", {})
assert res.get("accepted") is True, r
assert res.get("reason") == "enabled — driving", f"reason 应逐字对齐原版，实际 {res}"
wait_push(lambda p: p["enabled"] is True, "推送 enabled=true")
print(f"i. enable on → {res['reason']!r}，推送 enabled=true  OK")

# mouth 只在 Driving 阶段生效（D31），先等斜坡走完。可观测量选 skill 字段：
# driving_tick 跑起来推送里才有 "walk"（Held/RampUp/Stopped 都是 null）——
# gain 判不出（RampUp/Stopped 也用 running 增益 200）。RampUp 100 拍 = 2s。
wait_push(lambda p: p.get("skill") == "walk", "进入 Driving（skill=walk）", budget=8.0)
print("  已进入 Driving（skill=walk）")

# ── 断言 j：无 id robot.head → 静默生效（obs[51:55] 变为目标值）──
HEAD_TARGET = [0.1, -0.1, 0.2, 0.05]
send({"jsonrpc": "2.0", "method": "robot.head",
      "params": {"neck_pitch": 0.1, "head_pitch": -0.1,
                 "head_yaw": 0.2, "head_roll": 0.05}})
# obs head 块偏移 OFF_HEAD=51（src/obs.rs），直通 command.head 无低通。
deadline = time.time() + 1.0
hit = None
while time.time() < deadline and hit is None:
    msg = readline()
    assert msg.get("method") == "robot.state", f"通知产生了响应行: {msg}"
    h = msg["params"]["obs"][51:55]
    if close(h, HEAD_TARGET):
        hit = h
assert hit, f"1s 内 obs head 未变成 {HEAD_TARGET}"
print(f"j. 无 id robot.head 静默生效 obs[51:55]={hit}，全程无响应行  OK")

# ── 断言 k：无 id robot.mouth {position:1.0} → 静默生效 ──
# MOUTH_INDEX=9（src/model.rs）；0..1 → −5°..+30°，position=1 目标 ≈0.524rad。
# FakeIo.write 是精确回写（io.rs:340-344，自述故意不做一阶惯性模型），
# 2s 预算和 0.2rad 阈值因此很富余（实测精确到 0.524）。
send({"jsonrpc": "2.0", "method": "robot.mouth", "params": {"position": 1.0}})
deadline = time.time() + 2.0
peak = None
while time.time() < deadline:
    msg = readline()
    assert msg.get("method") == "robot.state", f"通知产生了响应行: {msg}"
    mouth = msg["params"]["positions"][9]
    peak = mouth if peak is None else max(peak, mouth)
    if mouth > 0.2:
        break
assert peak is not None and peak > 0.2, f"2s 内嘴关节未到 0.2rad（峰值 {peak}）"
print(f"k. 无 id robot.mouth 静默生效 positions[9]={peak:.3f}rad（目标 0.524）  OK")

# ── 断言 l：无 id 非意图方法（robot.health）→ 静默丢弃 ──
send({"jsonrpc": "2.0", "method": "robot.health"})
frames = read_pushes(0.5)
assert frames, "没收到推送"
print(f"l. 无 id robot.health 静默，{len(frames)} 帧内只有 robot.state 推送  OK")

# ── 断言 m：带 id robot.head / robot.mouth → 请求式回显不变 ──
send({"jsonrpc": "2.0", "id": 9, "method": "robot.head",
      "params": {"neck_pitch": 0.0, "head_pitch": 0.0,
                 "head_yaw": 0.0, "head_roll": 0.0}})
r = read_until_response(9)
res = r.get("result", {})
assert close(res.get("head", []), [0.0, 0.0, 0.0, 0.0]), f"head 应回显，实际 {r}"
send({"jsonrpc": "2.0", "id": 10, "method": "robot.mouth", "params": {"position": 0.5}})
r = read_until_response(10)
res = r.get("result", {})
assert abs(res.get("mouth", -1) - 0.5) < 1e-12, f"mouth 应回显，实际 {r}"
print(f"m. 带 id head/mouth 请求式回显不变（mouth 回显 {res['mouth']}）  OK")

# ── 断言 n：robot.enable toggle:true → 翻转（当前开→关）──
# on 是必填字段（toggle 不免除 on，对照原版 EnableParams），带上 on 但被忽略。
send({"jsonrpc": "2.0", "id": 11, "method": "robot.enable",
      "params": {"on": False, "toggle": True}})
r = read_until_response(11)
res = r.get("result", {})
assert res.get("accepted") is True, r
assert res.get("reason") == "disabled — returning to the home pose", \
    f"toggle 关 reason 应逐字对齐原版，实际 {res}"
wait_push(lambda p: p["enabled"] is False, "推送 enabled=false")
print(f"n. toggle 翻转 开→关 → {res['reason']!r}，推送 enabled=false  OK")

# ── 断言 n2：disable 的原版语义——不卸 torque、当拍直接回 home（无斜坡）──
# 原版 was_driving && !driving && !enabled 边沿只做 policy reset + 直接命令
# home（"Commanded directly, no ramp"）；卸 torque 是 robot.relax 的活
# （enable 管策略、init/relax 管电源）。FakeIo 精确回写：Stopped 当拍写
# home，下一拍读到的 positions 就逐位等于 home。
HOME = [0.0, -0.0873, -0.4579, -0.0049, 0.4530, 0.3491, 0.3491, 0.0, 0.0, 0.0,
        0.0, 0.0873, 0.4579, 0.0049, -0.4530]
wait_push(
    lambda p: max(abs(a - b) for a, b in zip(p["positions"], HOME)) < 1e-9,
    "positions 逐位等于 home（Stopped 抱持）",
    budget=1.0,
)
frames = read_pushes(0.5)
off = [p for p in frames if p["torque"] is not True]
assert not off, f"disable 后 torque 应保持 on，出现 torque=false 帧: {off[:1]}"
print(f"n2. disable 后 positions 逐位等于 home，{len(frames)} 帧 torque 全为 true  OK")

# ── 断言 o：{on:true, toggle:true} → toggle 优先（忽略 on），当前关→开 ──
send({"jsonrpc": "2.0", "id": 12, "method": "robot.enable",
      "params": {"on": True, "toggle": True}})
r = read_until_response(12)
res = r.get("result", {})
assert res.get("accepted") is True, r
assert res.get("reason") == "enabled — driving", f"toggle 开 reason 不对: {res}"
wait_push(lambda p: p["enabled"] is True, "推送 enabled=true")
print(f"o. on+toggle 同帧 toggle 优先 关→开 → {res['reason']!r}  OK")

# ── 断言 o2：Stopped→Driving 无斜坡——1s 预算（< 旧斜坡 2s）内 skill=walk ──
wait_push(lambda p: p.get("skill") == "walk", "1s 内恢复 Driving（skill=walk）", budget=1.0)
print("o2. 1s 内 skill=walk——从 Stopped 直接回 Driving，无 2 秒斜坡  OK")

# ── 断言 p1：robot.enable {on:true, foo:1} → INVALID_PARAMS ──
send({"jsonrpc": "2.0", "id": 13, "method": "robot.enable",
      "params": {"on": True, "foo": 1}})
r = read_until_response(13)
err = r.get("error") or {}
assert err.get("code") == -32602, f"未知字段应回 INVALID_PARAMS，实际 {r}"
print(f"p1. enable 未知字段 INVALID_PARAMS: {err['message'][:60]}  OK")

# ── 断言 p2：robot.disable 已死 → METHOD_NOT_FOUND ──
send({"jsonrpc": "2.0", "id": 14, "method": "robot.disable"})
r = read_until_response(14)
err = r.get("error") or {}
assert err.get("code") == -32601, f"robot.disable 应回 METHOD_NOT_FOUND，实际 {r}"
print(f"p2. robot.disable → METHOD_NOT_FOUND: {err['message']}  OK")

print()
print("M8 断言 a–p 全部通过")
PY
RC=$?

echo
if [ "$RC" = 0 ]; then
    echo "M8 验收（robot.move + robot.enable 对齐，notification 统一入口）全部通过"
else
    echo "FAIL: 断言脚本退出码 $RC"
fi
exit "$RC"

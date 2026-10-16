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
# Bite 3 断言（订阅入口改为 robot.subscribe {hz} + SubscribeResult ack；
# 推送 method 仍是 robot.state。hz 收下即丢，推送仍全速率）：
#   入场：robot.subscribe params {} → accepted、walk=velstand.onnx、
#   sitstand=alpha_sitstand.onnx、ground_pick=alpha_ground_pick.onnx、
#   skills=["roulade","kick_left"]，结果里没有 stand / unavailable
#   q. 带 id 的 robot.state → METHOD_NOT_FOUND（-32601），随后推送仍在
#      （订阅没被拆掉，也不因这次请求才开始推送）
#   r. 再发 robot.subscribe {hz:10} → accepted、walk 仍是 velstand.onnx；
#      随后 0.5 秒内推送帧数 ≥ 18（10Hz 在 0.5s 只有约 5 帧，证明没降频）
#   s. robot.subscribe {hz:10, foo:1} → INVALID_PARAMS（-32602），之后推送仍在
#      （解析失败不退订）
#   t. 无 id 的 robot.subscribe {hz:10} → 0.3s 内只有 robot.state 推送，无响应行
# Bite 9 断言（D24 读失败滑行 COAST_TICKS=3；FakeIo 用 env
#   MINIDUCK_FAKE_FAIL_AFTER=<ok>:<fail> 注入一次性中途连挂）：
#   x. 连挂 3 次（=COAST_TICKS）：全程被滑行吸收——推送不断、skill 恒 walk、
#      恰好一段 3 帧重投上一样本、skipped_reads 恒 0、healthy 恒 true
#   y. 连挂 4 次（>COAST_TICKS）：3 帧滑行后第 4 拍真瞎跳拍——
#      skipped_reads 恰好 +1，恢复后 consecutive_read_errors 清零、仍 healthy
# Bite 10 断言（D36：robot.do 只对 fallen/Limp 放开——RPC 照收、控制
# 循环静默丢；!enabled/!homed/未知名仍在门口拒绝，对齐原版 dispatch）：
#   z1. Driving 中 do kick_left → accepted:true 且推送 skill=kick_left
#       （真启动），窗口结束回 walk
#   z2. disable（Stopped）后 do kick_left → accepted:false + 原版
#       !enabled 文案（:4200），且不启动不排队
#   z3. 再 enable 回 Driving：先稳 0.6s skill 恒 walk（拒绝不排队不记仇），
#       再 do kick_left → accepted:true 且真启动（恢复后照常）
#   z4. 未知技能名 → accepted:false 附名单；缺 skill 字段 → INVALID_PARAMS
#       （参数/名字路径不属于本偏差，行为不变）
#   z5. 斜坡途中（RampUp=未 homed）do → accepted:false + 原版 !homed
#       文案（:4202-4204）。时间线要求它在断言 i 之后的斜坡窗口内执行
#   注：「Limp 中 do accepted:true 但静默丢」无端到端断言——FakeIo 无
#   IMU/gravity 注入面，fallen 诱发不了；该路径由单测覆盖
#   （edges_when_driving / phase_homed / do_verdict，见 z2 处注释）。
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

# 入场第一件事：hello + robot.subscribe（协议规定，lib.rs 注释）。
# 推送帧的 method 仍是 robot.state；ack 是 SubscribeResult，不回显 hz。
send({"jsonrpc": "2.0", "id": 1, "method": "hello"})
r = read_until_response(1)
assert r["result"]["service"] == "miniduckd", r
send({"jsonrpc": "2.0", "id": 2, "method": "robot.subscribe", "params": {}})
r = read_until_response(2)
res = r.get("result") or {}
assert res.get("accepted") is True, r
assert res.get("walk") == "velstand.onnx", r
assert res.get("sitstand") == "alpha_sitstand.onnx", r
assert res.get("ground_pick") == "alpha_ground_pick.onnx", r
assert res.get("skills") == ["roulade", "kick_left"], r
assert "stand" not in res, r
assert "unavailable" not in res, r
print(f"握手 + robot.subscribe ack walk={res['walk']} skills={res['skills']}  OK")

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

# ── 断言 z5：斜坡途中（未 homed）robot.do → 拒绝，原版文案 ──
# D36 只放开 fallen/Limp；门口仍按情形拒 !enabled/!homed（原版
# reference/robotd/src/main.rs:4196-4204）。此刻处于 RampUp（100 拍=2s）
# 窗口内，homed=false。时间线要求此断言只能在 i 之后、斜坡结束前跑，
# 故编号在 z 段而位置在这里。
send({"jsonrpc": "2.0", "id": 28, "method": "robot.do", "params": {"skill": "kick_left"}})
r = read_until_response(28)
res = r.get("result", {})
assert res.get("accepted") is False, f"斜坡途中应拒绝，实际 {r}"
assert res.get("reason") == "the robot is still going to its home pose; try again in a moment", \
    f"reason 应逐字对齐原版 :4202-4204，实际 {res}"
print(f"z5. RampUp 中 do → accepted:false，reason={res['reason']!r}（原版 homed 拒绝）  OK")

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

# ── 断言 q：带 id 的 robot.state 不是请求方法，订阅也不被拆掉 ──
send({"jsonrpc": "2.0", "id": 15, "method": "robot.state"})
r = read_until_response(15)
err = r.get("error") or {}
assert err.get("code") == -32601, f"robot.state 应回 METHOD_NOT_FOUND，实际 {r}"
frames = read_pushes(0.2)
assert frames, "robot.state 请求之后收不到推送，订阅被拆掉了"
print(f"q. robot.state 带 id → -32601，随后仍有 {len(frames)} 帧推送  OK")

# ── 断言 r：hz=10 收下但不降频 ──
send({"jsonrpc": "2.0", "id": 16, "method": "robot.subscribe", "params": {"hz": 10}})
r = read_until_response(16)
res = r.get("result") or {}
assert res.get("accepted") is True, r
assert res.get("walk") == "velstand.onnx", r
assert "hz" not in res, r
frames = read_pushes(0.5)
assert len(frames) >= 18, f"0.5s 只有 {len(frames)} 帧，像是降到了 10Hz"
print(f"r. 再订阅 hz=10，walk={res['walk']}，0.5s 内 {len(frames)} 帧（≥18，全速率）  OK")

# ── 断言 s：未知字段 INVALID_PARAMS，且不退订 ──
send({"jsonrpc": "2.0", "id": 17, "method": "robot.subscribe",
      "params": {"hz": 10, "foo": 1}})
r = read_until_response(17)
err = r.get("error") or {}
assert err.get("code") == -32602, f"未知字段应回 INVALID_PARAMS，实际 {r}"
frames = read_pushes(0.2)
assert frames, "解析失败之后收不到推送，订阅被退掉了"
print(f"s. subscribe 未知字段 -32602，随后仍有 {len(frames)} 帧推送  OK")

# ── 断言 t：无 id 的 robot.subscribe 静默丢弃，不产生响应行 ──
send({"jsonrpc": "2.0", "method": "robot.subscribe", "params": {"hz": 10}})
frames = read_pushes(0.3)
assert frames, "没收到推送"
print(f"t. 无 id robot.subscribe 静默，{len(frames)} 帧内只有 robot.state 推送  OK")

# ── 断言 u：静止时 safeToRestart safe:true，reason 省略 ──
send({"jsonrpc": "2.0", "id": 18, "method": "robot.safeToRestart"})
r = read_until_response(18)
res = r.get("result", {})
assert res.get("safe") is True, r
assert "reason" not in res, r
print("u. 静止时 safeToRestart safe:true，reason 省略  OK")

# ── 断言 v：无 id robot.move {vx:0.1} 后，safeToRestart safe:false ──
send({"jsonrpc": "2.0", "method": "robot.move", "params": {"vx": 0.1}})
deadline = time.time() + 1.0
while True:
    msg = readline()
    assert msg.get("method") == "robot.state", msg
    if close(twist_of(msg["params"]), [0.1, 0.0, 0.0]):
        break
    assert time.time() < deadline, "move 通知没生效"
send({"jsonrpc": "2.0", "id": 19, "method": "robot.safeToRestart"})
r = read_until_response(19)
res = r.get("result", {})
assert res.get("safe") is False, r
assert res.get("reason") == "the robot is walking", r
print("v. move 后 safeToRestart safe:false reason='the robot is walking'  OK")

# ── 断言 w：robot.modelApi → model_api:2 ──
send({"jsonrpc": "2.0", "id": 20, "method": "robot.modelApi"})
r = read_until_response(20)
res = r.get("result", {})
assert res.get("model_api") == 2, r
print("w. robot.modelApi → model_api:2  OK")

# ── 断言 z1：Driving 中 robot.do kick_left → accepted 且真启动 ──
# D36 基线：恒收语义下正常路径不受影响。kick_left 窗口 25 拍 = 0.5s
#（src/scheduler.rs），自选一次性技能，跑完自动回 walk，不留坐姿锁存。
send({"jsonrpc": "2.0", "id": 21, "method": "robot.do", "params": {"skill": "kick_left"}})
r = read_until_response(21)
res = r.get("result", {})
assert res.get("accepted") is True, r
assert res.get("skill") == "kick_left", r
wait_push(lambda p: p.get("skill") == "kick_left", "技能启动（skill=kick_left）")
wait_push(lambda p: p.get("skill") == "walk", "kick 窗口结束回 walk", budget=2.0)
print("z1. Driving 中 do kick_left → accepted 且推送 skill=kick_left→walk  OK")

# ── 断言 z2：Stopped（未 enable）中 robot.do → 拒绝，原版文案 ──
# D36 只放开 fallen/Limp：enabled=false 仍在门口拒绝（原版
# reference/robotd/src/main.rs:4196-4200）。「Limp 中 do accepted:true
# 但静默丢」的端到端断言 FakeIo 做不了——FakeIo 无 IMU/gravity 注入面
#（src/io.rs:142-187，read 恒报直立重力 [0,0,-1]），fallen 诱发不了；
# 该路径维持单测覆盖（control.rs edges_when_driving 的 Limp 分支 +
# phase_homed 的 Limp=true + scheduler.rs do_verdict(true,true,...) 照收）。
send({"jsonrpc": "2.0", "id": 22, "method": "robot.enable", "params": {"on": False}})
r = read_until_response(22)
assert (r.get("result") or {}).get("accepted") is True, r
wait_push(lambda p: p["enabled"] is False, "推送 enabled=false")
send({"jsonrpc": "2.0", "id": 23, "method": "robot.do", "params": {"skill": "kick_left"}})
r = read_until_response(23)
res = r.get("result", {})
assert res.get("accepted") is False, f"D36 不改 !enabled 的拒绝，实际 {r}"
assert res.get("reason") == "the policy is not driving — press Start on the pad", \
    f"reason 应逐字对齐原版 :4200，实际 {res}"
# 拒绝不排队（原版 :5769 "a refusal must not queue"）：此刻无任何启动。
frames = read_pushes(0.4)
assert all(p.get("skill") is None for p in frames), "被拒的请求不该启动"
print(f"z2. Stopped 中 do → accepted:false，reason={res['reason']!r}（原版 enabled 拒绝）  OK")

# ── 断言 z3：恢复 Driving 后——拒绝/丢弃都不留痕，新请求照常工作 ──
send({"jsonrpc": "2.0", "id": 24, "method": "robot.enable", "params": {"on": True}})
r = read_until_response(24)
assert (r.get("result") or {}).get("accepted") is True, r
wait_push(lambda p: p.get("skill") == "walk", "1s 内恢复 Driving（skill=walk）", budget=1.0)
# 不记仇：z2 被拒的请求不得在此补发——稳 0.6s（> kick 窗口）skill 恒 walk。
frames = read_pushes(0.6)
assert all(p.get("skill") == "walk" for p in frames), "被拒绝的请求不得排队补发"
send({"jsonrpc": "2.0", "id": 25, "method": "robot.do", "params": {"skill": "kick_left"}})
r = read_until_response(25)
assert (r.get("result") or {}).get("accepted") is True, r
wait_push(lambda p: p.get("skill") == "kick_left", "恢复后技能启动")
wait_push(lambda p: p.get("skill") == "walk", "kick 窗口结束回 walk", budget=2.0)
print("z3. 恢复 Driving：旧请求不补发，新 do accepted 且真启动  OK")

# ── 断言 z4：名字/参数路径不属于本偏差——未知名仍拒绝附名单，缺字段仍 INVALID_PARAMS ──
send({"jsonrpc": "2.0", "id": 26, "method": "robot.do", "params": {"skill": "kick_right"}})
r = read_until_response(26)
res = r.get("result", {})
assert res.get("accepted") is False, r
reason = res.get("reason", "")
assert "kick_right" in reason and "roulade" in reason, f"拒绝要附名单: {res}"
send({"jsonrpc": "2.0", "id": 27, "method": "robot.do", "params": {}})
r = read_until_response(27)
err = r.get("error") or {}
assert err.get("code") == -32602, f"缺 skill 应回 INVALID_PARAMS，实际 {r}"
print(f"z4. 未知名 accepted:false 附名单；缺字段 INVALID_PARAMS  OK")

print()
print("M8 断言 a–w + z1–z5 全部通过")
PY
RC=$?

# ── 断言 x/y：D24 读失败滑行（COAST_TICKS=3，reference/robotd/src/main.rs:137）──
# 重启 daemon 注入一次性中途连挂。MINIDUCK_FAKE_FAIL_AFTER=300:k：
# 前 300 次 read 正常（enable+斜坡 ~165 次，进入 Driving 后 ~2.8s 爆发），
# 之后连挂 k 次。帧流在 Driving 中采 4s，覆盖爆发与恢复。
if [ "$RC" = 0 ]; then
  cleanup; sleep 1
  MINIDUCK_FAKE_FAIL_AFTER=300:3 ./target/debug/miniduckd 2>>/tmp/m8-daemon.log &
  DAEMON_PID=$!
  sleep 1
  [ -S /tmp/miniduckd.sock ] || { echo "FAIL: daemon 没起来（/tmp/m8-daemon.log）"; exit 1; }
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
    deadline = time.time() + budget
    while time.time() < deadline:
        msg = readline()
        if msg.get("id") == rid and ("result" in msg or "error" in msg):
            return msg
        assert msg.get("method") == "robot.state", f"等响应时混进意外帧: {msg}"
    raise AssertionError(f"等 id={rid} 响应超时")

def identical_runs(frames):
    # 相邻帧 positions 逐位相等的最大连续段长分布。滑行拍重投上一样本，
    # 所以 k 个滑行拍产生 k+1 帧的重复段（1 帧新鲜 + k 帧重投）；
    # f64 精确相等——正常行走每拍位置都变，不可能出现自然重复段。
    runs, run = [], 1
    for a, b in zip(frames, frames[1:]):
        if b["positions"] == a["positions"]:
            run += 1
        else:
            runs.append(run); run = 1
    runs.append(run)
    return runs

send({"jsonrpc": "2.0", "id": 1, "method": "hello"})
read_until_response(1)
send({"jsonrpc": "2.0", "id": 2, "method": "robot.subscribe", "params": {}})
read_until_response(2)
send({"jsonrpc": "2.0", "id": 3, "method": "robot.enable", "params": {"on": True}})
read_until_response(3)
deadline = time.time() + 8.0
while True:
    msg = readline()
    assert msg.get("method") == "robot.state", msg
    if msg["params"].get("skill") == "walk":
        break
    assert time.time() < deadline, "没进入 Driving"

frames = []
deadline = time.time() + 4.0
while time.time() < deadline:
    msg = readline()
    assert msg.get("method") == "robot.state", msg
    frames.append(msg["params"])

runs = identical_runs(frames)
assert 4 in runs, f"未见 4 帧连续重投段（1 新鲜 + 3 滑行），runs: max={max(runs)}"
assert max(runs) <= 4, f"滑行窗只许 3 拍（4 帧段），出现 {max(runs)} 帧连续重复"
assert all(p.get("skill") == "walk" for p in frames), "滑行期间策略掉线了"

send({"jsonrpc": "2.0", "id": 4, "method": "robot.health"})
r = read_until_response(4)
res = r["result"]
assert res["reads"] > 303, f"注入点（read 301-303）没跑过？reads={res['reads']}"
assert res["skipped_reads"] == 0, \
    f"≤COAST_TICKS 的连挂不许跳拍，skipped_reads={res['skipped_reads']}"
assert res["consecutive_read_errors"] == 0, "恢复后连续错误应清零"
assert res["healthy"] is True, res
print(f"x. 3 连挂全程滑行：{len(frames)} 帧 skill 恒 walk，恰好 {max(runs)} 帧重投段，"
      f"skipped_reads=0  OK")
PY
  RC=$?
fi

if [ "$RC" = 0 ]; then
  cleanup; sleep 1
  MINIDUCK_FAKE_FAIL_AFTER=300:4 ./target/debug/miniduckd 2>>/tmp/m8-daemon.log &
  DAEMON_PID=$!
  sleep 1
  [ -S /tmp/miniduckd.sock ] || { echo "FAIL: daemon 没起来（/tmp/m8-daemon.log）"; exit 1; }
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
    deadline = time.time() + budget
    while time.time() < deadline:
        msg = readline()
        if msg.get("id") == rid and ("result" in msg or "error" in msg):
            return msg
        assert msg.get("method") == "robot.state", f"等响应时混进意外帧: {msg}"
    raise AssertionError(f"等 id={rid} 响应超时")

send({"jsonrpc": "2.0", "id": 1, "method": "hello"})
read_until_response(1)
send({"jsonrpc": "2.0", "id": 2, "method": "robot.subscribe", "params": {}})
read_until_response(2)
send({"jsonrpc": "2.0", "id": 3, "method": "robot.enable", "params": {"on": True}})
read_until_response(3)
deadline = time.time() + 8.0
while True:
    msg = readline()
    assert msg.get("method") == "robot.state", msg
    if msg["params"].get("skill") == "walk":
        break
    assert time.time() < deadline, "没进入 Driving"

frames = []
deadline = time.time() + 4.0
while time.time() < deadline:
    msg = readline()
    assert msg.get("method") == "robot.state", msg
    frames.append(msg["params"])

runs, run = [], 1
for a, b in zip(frames, frames[1:]):
    if b["positions"] == a["positions"]:
        run += 1
    else:
        runs.append(run); run = 1
runs.append(run)
assert 4 in runs, f"4 连挂应有 4 帧段（1 新鲜 + 3 滑行），runs: max={max(runs)}"
assert max(runs) <= 4, f"滑行窗只许 3 拍（4 帧段），出现 {max(runs)} 帧连续重复"
assert all(p.get("skill") == "walk" for p in frames), "滑行期间策略掉线了"

send({"jsonrpc": "2.0", "id": 4, "method": "robot.health"})
r = read_until_response(4)
res = r["result"]
assert res["reads"] > 304, f"注入点（read 301-304）没跑过？reads={res['reads']}"
assert res["skipped_reads"] == 1, \
    f"第 4 拍（超 COAST_TICKS）必须真瞎跳拍，skipped_reads={res['skipped_reads']}"
assert res["consecutive_read_errors"] == 0, "恢复后连续错误应清零"
assert res["healthy"] is True, res
print(f"y. 4 连挂：3 帧滑行 + 第 4 拍跳拍（skipped_reads=1），恢复后仍 healthy  OK")
PY
  RC=$?
fi

echo
if [ "$RC" = 0 ]; then
    echo "M8 验收（robot.move + robot.enable + robot.subscribe + safeToRestart/modelApi 对齐 + D24 滑行 + D36 robot.do fallen/Limp 照收，notification 统一入口）全部通过"
else
    echo "FAIL: 断言脚本退出码 $RC"
fi
exit "$RC"

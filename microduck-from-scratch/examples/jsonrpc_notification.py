#!/usr/bin/env python3
"""JSON-RPC request / notification 双形态最小演示（M8 概念文档配套）。

单文件自包含：主线程起 server 线程，client 依次发三种帧，打印 server
实际回了什么。host 直接跑：python3 examples/jsonrpc_notification.py

三种帧：
  1. request（带 id）   —— 服务端必须应答，id 回显
  2. notification（无 id）—— 服务端照常执行，但一个字节都不回
  3. 非法 JSON          —— 没法解析出 id，按规范回 "id":null 的错误
"""
import json
import os
import socket
import threading
import time

PATH = "/tmp/jsonrpc-demo.sock"
state = {"value": 0.0}  # server 侧的唯一状态，模拟机器人的 twist


def server():
    if os.path.exists(PATH):
        os.remove(PATH)
    srv = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    srv.bind(PATH)
    srv.listen(1)
    conn, _ = srv.accept()
    f = conn.makefile("rw")
    for line in f:
        try:
            frame = json.loads(line)
        except json.JSONDecodeError as e:
            # 帧本身没解析出来，没有 id 可回显 —— 规范规定此时回 null
            f.write(json.dumps({"jsonrpc": "2.0", "id": None,
                                "error": {"code": -32700, "message": str(e)}}) + "\n")
            f.flush()
            continue
        if frame.get("method") == "set":
            state["value"] = frame["params"]["value"]
        if "id" in frame:  # 有 id = request，必须应答
            f.write(json.dumps({"jsonrpc": "2.0", "id": frame["id"],
                                "result": state["value"]}) + "\n")
            f.flush()
        # 无 id = notification：已经执行完了，到此为止，什么都不回
    srv.close()


t = threading.Thread(target=server, daemon=True)
t.start()

s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.settimeout(0.5)  # 半秒收不到就认为"没有回复"
for _ in range(200):  # 等 server 线程 bind 完（上次运行可能留了旧 socket 文件）
    try:
        s.connect(PATH)
        break
    except (FileNotFoundError, ConnectionRefusedError):
        time.sleep(0.01)
else:
    raise RuntimeError("server 没起来")

# 不用 makefile：它一旦超时就不能再读（"cannot read from timed out object"）。
# 手动缓冲 recv，按 \n 切行（NDJSON：每帧一行）。
buf = b""


def readline():
    global buf
    while b"\n" not in buf:
        chunk = s.recv(4096)
        assert chunk, "对端断了连接"
        buf += chunk
    line, buf = buf.split(b"\n", 1)
    return line.decode()


def send_raw(raw, note):
    s.sendall(raw.encode() + b"\n")
    try:
        reply = readline()
    except TimeoutError:
        reply = "（0.5s 内什么都没收到）"
    print(f"{note}\n  发出: {raw}\n  收到: {reply}\n")


send_raw('{"jsonrpc":"2.0","id":1,"method":"set","params":{"value":0.3}}',
         "1. request：带 id，服务端应答并回显 id")
send_raw('{"jsonrpc":"2.0","method":"set","params":{"value":0.7}}',
         "2. notification：无 id，服务端静默执行")
send_raw('{"jsonrpc":"2.0","id":2,"method":"get"}',
         "3. 再发一个 request 查状态——证明通知 0.7 真的落了")
send_raw('{这根本不是 JSON',
         "4. 非法帧：解析不出 id，错误响应的 id 是 null")

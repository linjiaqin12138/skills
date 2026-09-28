#!/usr/bin/env python3
"""duck_body.py — MuJoCo 仿真体，TCP + NDJSON 服务（M4）。

控制侧（miniduckd --sim host:port）每 20ms 一拍：
  read  → 仿真步进一个控制拍（20ms，内部按 timestep 细分），回传感数据
          （响应带 gain/torque 回显，M5 起）
  write → 收 15 维总线序目标角（嘴在 index 9，仿真体没有嘴，丢弃）
  set_gain/set_torque → 执行器增益缩放/卸力（M5：跌倒软倒验收用）
  push  → 给躯干加水平速度扰动（M5：把鸭子推倒的验收用）
  body  → 只查躯干位置不步进（验收测位移用）
协议形状对齐原版 duck-control/src/sim.rs：op 标签帧、hello 握手带
protocol 版本号和关节数校验。帧格式细节是教程自定（偏差 D4，M8 对齐）。

单位即机器人自己的单位：弧度、rad/s；IMU 已解算到躯干系
（gyro 取 MJCF gyro 传感器，gravity 由躯干四元数旋转世界重力得到，
四元数 wxyz 序）。read 响应带 body_pos（躯干世界系 x/y/z），供验收测位移。
"""

import argparse
import io
import json
import os
import socket
import socketserver
import sys
import threading
import time

# 容器里没有显示器，离屏渲染走软件 GL。必须在 import mujoco 之前设好。
os.environ.setdefault("MUJOCO_GL", "osmesa")

import mujoco
import numpy as np

PROTOCOL = 1
# 总线序（15，含嘴）；仿真关节序（14，无嘴）。映射：总线 i<9 → 关节 i，i>9 → 关节 i-1。
NUM_BUS_JOINTS = 15
MOUTH_INDEX = 9
CONTROL_PERIOD = 0.02  # 一个控制拍 = 20ms 仿真时间

JOINT_NAMES = [
    "left_hip_yaw", "left_hip_roll", "left_hip_pitch", "left_knee", "left_ankle",
    "neck_pitch", "head_pitch", "head_yaw", "head_roll",
    "right_hip_yaw", "right_hip_roll", "right_hip_pitch", "right_knee", "right_ankle",
]
TRUNK = "trunk_base"
STAND_KEY = "STAND"


class DuckBody:
    def __init__(self, scene_path: str):
        self.model = mujoco.MjModel.from_xml_path(scene_path)
        self.data = mujoco.MjData(self.model)
        # 失稳时 MuJoCo 默认 autoreset 会把状态清零、时钟归零——调试时表现为
        # "时间倒流"。关掉它：摔倒就是摔倒，验收看的是真实位移。
        self.model.opt.disableflags |= mujoco.mjtDisableBit.mjDSBL_AUTORESET
        self.substeps = max(1, round(CONTROL_PERIOD / self.model.opt.timestep))

        self.joint_qadr = [self.model.joint(n).qposadr.item() for n in JOINT_NAMES]
        self.joint_vadr = [self.model.joint(n).dofadr.item() for n in JOINT_NAMES]
        # 执行器由 RL 模型的 <actuator> 段按 JOINT_NAMES 文档序逐个声明，
        # ctrl 序即关节序（已在验收中实测 assert）。
        assert self.model.nu == len(JOINT_NAMES), f"nu={self.model.nu}"
        self.act_ids = list(range(len(JOINT_NAMES)))
        self.trunk_id = self.model.body(TRUNK).id
        self.gyro_adr = self.model.sensor("imu_gyro").adr.item()

        key = self.model.key(STAND_KEY).id
        mujoco.mj_resetDataKeyframe(self.model, self.data, key)
        # 起步即有力矩：真机的舵机一直挂着，仿真从瘫软开始会在第一拍前摔地上。
        self.data.ctrl[:] = self.data.qpos[self.joint_qadr]
        mujoco.mj_forward(self.model, self.data)

        # M5：模拟真机舵机 RAM 里的 torque/gain 跨进程存活——仿真体启动
        # 默认 torque on、gain=200（满增益），daemon 不碰它们也能站住。
        self.gain = 200
        self.torque = True

        # 物理与渲染共用 MjData：read 步进和画面采样必须互斥，否则画面撕在半拍上。
        self.lock = threading.Lock()
        self.renderer = None

        self._apply_gains()

    def _apply_gains(self):
        """把 gain(0..200)/torque 落到执行器参数上。

        位置执行器的力 = kp*(ctrl-qpos) - kv*qvel，存在 gainprm[0]=kp、
        biasprm[1]=-kp、biasprm[2]=-kv（MuJoCo position actuator 约定）。
        映射：kp_sim = 8.0 × gain/200，kv 同比例（0.25 × gain/200）——
        8.0/0.25 是 M4 实测调出的站稳增益（偏差 D21），gain=200 对应它。
        torque off 直接清零三处：舵机卸力，鸭子在重力下瘫软。
        直接改 gainprm/biasprm 而不缩 ctrl：目标角保持不变，恢复时无跳变。
        """
        scale = (self.gain / 200.0) if self.torque else 0.0
        kp, kv = 8.0 * scale, 0.25 * scale
        with self.lock:
            self.model.actuator_gainprm[:, 0] = kp
            self.model.actuator_biasprm[:, 1] = -kp
            self.model.actuator_biasprm[:, 2] = -kv

    def render_frame(self):
        """离屏渲染一帧 JPEG。跟踪相机锁定躯干，走远了镜头跟着走。"""
        if self.renderer is None:
            from PIL import Image  # 仅 viewer 路径需要
            self._Image = Image
            self.renderer = mujoco.Renderer(self.model, height=480, width=640)
            cam = mujoco.MjvCamera()
            cam.type = mujoco.mjtCamera.mjCAMERA_TRACKING
            cam.trackbodyid = self.trunk_id
            cam.distance = 0.8
            cam.elevation = -15
            cam.azimuth = 90  # 侧面跟拍：前进方向在画面里是左右，步态最可读
            self._cam = cam
        with self.lock:
            self.renderer.update_scene(self.data, self._cam)
            rgb = self.renderer.render()
        buf = io.BytesIO()
        self._Image.fromarray(rgb).save(buf, format="JPEG", quality=70)
        return buf.getvalue()

    def step_tick(self):
        with self.lock:
            for _ in range(self.substeps):
                mujoco.mj_step(self.model, self.data)

    def bus_order(self, values):
        """14 维关节序 → 15 维总线序（嘴位补 0）。"""
        out = np.zeros(NUM_BUS_JOINTS)
        out[:MOUTH_INDEX] = values[:MOUTH_INDEX]
        out[MOUTH_INDEX + 1:] = values[MOUTH_INDEX:]
        return out.tolist()

    def sensors(self):
        d, m = self.data, self.model
        quat = d.xquat[self.trunk_id].copy()  # wxyz，躯干世界系姿态
        rot = np.empty(9)
        mujoco.mju_quat2Mat(rot, quat)
        rot = rot.reshape(3, 3)
        gravity = rot.T @ np.array([0.0, 0.0, -1.0])  # 世界重力旋进躯干系，单位化
        gyro = d.sensordata[self.gyro_adr:self.gyro_adr + 3].copy()
        return {
            "positions": self.bus_order(d.qpos[self.joint_qadr]),
            "velocities": self.bus_order(d.qvel[self.joint_vadr]),
            "imu": {
                "gyro": gyro.tolist(),
                "gravity": gravity.tolist(),
                "quat": quat.tolist(),
            },
            "body_pos": d.xpos[self.trunk_id].tolist(),
            "sim_time": d.time,
            # M5：回显当前增益/出力状态，验收据此断言跌倒卸力（gain==50）。
            "gain": self.gain,
            "torque": self.torque,
        }

    def write_targets(self, targets):
        t = np.asarray(targets, dtype=float)
        if t.shape != (NUM_BUS_JOINTS,):
            return f"targets must be {NUM_BUS_JOINTS} numbers"
        joints = np.concatenate([t[:MOUTH_INDEX], t[MOUTH_INDEX + 1:]])
        with self.lock:
            self.data.ctrl[self.act_ids] = joints
        return None

    def handle(self, req):
        op = req.get("op")
        if op == "hello":
            if req.get("protocol") != PROTOCOL:
                return {"error": f"simulator speaks protocol {PROTOCOL}"}
            if req.get("joints") != NUM_BUS_JOINTS:
                return {"error": f"simulator body has {NUM_BUS_JOINTS} bus joints"}
            return {"protocol": PROTOCOL, "joints": NUM_BUS_JOINTS}
        if op == "read":
            # 节拍由控制侧驱动：每次 read 步进 20ms 仿真时间再回报。
            self.step_tick()
            return self.sensors()
        if op == "write":
            err = self.write_targets(req.get("targets", []))
            return {"error": err} if err else {}
        if op == "set_gain":
            gain = req.get("gain")
            if not isinstance(gain, int) or not 0 <= gain <= 200:
                return {"error": "gain must be an integer in 0..=200"}
            self.gain = gain
            self._apply_gains()
            return {}
        if op == "set_torque":
            on = req.get("on")
            if not isinstance(on, bool):
                return {"error": "on must be a bool"}
            self.torque = on
            self._apply_gains()
            return {}
        if op == "push":
            # 验收用扰动：给躯干水平速度加 vx/vy（m/s，世界系），把鸭子推倒。
            # free joint 的 qvel[0:3] 即躯干世界系线速度。
            vx = float(req.get("vx", 1.5))
            vy = float(req.get("vy", 0.0))
            with self.lock:
                self.data.qvel[0] += vx
                self.data.qvel[1] += vy
            return {"ok": True}
        if op == "body":
            # 只查躯干世界系位置，不步进（验收脚本用）。
            return {"body_pos": self.data.xpos[self.trunk_id].tolist(),
                    "sim_time": self.data.time}
        return {"error": f"unknown op: {op}"}


class Handler(socketserver.StreamRequestHandler):
    def handle(self):
        body: DuckBody = self.server.body
        for line in self.rfile:
            try:
                req = json.loads(line)
                resp = body.handle(req)
            except Exception as e:  # 坏帧回错误而不是断线，让对端自己决定重连
                resp = {"error": f"{type(e).__name__}: {e}"}
            self.wfile.write((json.dumps(resp) + "\n").encode())
            self.wfile.flush()


PAGE = b"""<!doctype html><meta charset="utf-8"><title>duck-body</title>
<body style="margin:0;background:#111;display:flex;justify-content:center">
<img src="/stream" style="max-width:100vw;max-height:100vh">"""


def serve_view(body: DuckBody, port: int):
    """MJPEG over HTTP：浏览器打开 http://localhost:<port>/ 即看实时画面。

    为什么不用 MuJoCo 自带 viewer：容器没有显示器也没有 X，
    而浏览器一定有。渲染是离屏软件渲染（osmesa），与物理线程用同一把锁。
    """
    from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

    class View(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == "/":
                data = PAGE
                self.send_response(200)
                self.send_header("Content-Type", "text/html")
            elif self.path == "/stream":
                self.send_response(200)
                self.send_header("Content-Type",
                                 "multipart/x-mixed-replace; boundary=frame")
                self.end_headers()
                try:
                    while True:
                        jpg = body.render_frame()
                        self.wfile.write(b"--frame\r\nContent-Type: image/jpeg\r\n"
                                         + f"Content-Length: {len(jpg)}\r\n\r\n".encode()
                                         + jpg + b"\r\n")
                        self.wfile.flush()
                        time.sleep(0.04)  # ~25fps，够看清步态
                except (BrokenPipeError, ConnectionResetError):
                    pass
                return
            else:
                self.send_response(404)
                data = b""
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def log_message(self, *a):
            pass

    ThreadingHTTPServer(("0.0.0.0", port), View).serve_forever()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=7801)
    ap.add_argument("--host", default="127.0.0.1")
    ap.add_argument("--scene", default=None)
    ap.add_argument("--view-port", type=int, default=None,
                    help="开启 MJPEG 网页画面（浏览器看仿真），如 7802")
    args = ap.parse_args()
    scene = args.scene or (sys.path[0] + "/assets/scene.xml")

    body = DuckBody(scene)

    if args.view_port:
        threading.Thread(target=serve_view, args=(body, args.view_port),
                         daemon=True).start()
        print(f"view: http://localhost:{args.view_port}/", file=sys.stderr, flush=True)

    class Server(socketserver.ThreadingTCPServer):
        allow_reuse_address = True
        daemon_threads = True

    with Server((args.host, args.port), Handler) as server:
        server.body = body
        print(f"duck-body listening on {args.host}:{args.port} "
              f"(timestep={body.model.opt.timestep}, substeps={body.substeps})",
              file=sys.stderr, flush=True)
        server.serve_forever()


if __name__ == "__main__":
    main()

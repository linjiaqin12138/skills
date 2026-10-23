"""M9a Bite 1 视觉抽查渲染：STAND 姿态离屏渲染。

用法（容器内）：
    MUJOCO_GL=osmesa /opt/sim-venv/bin/python \
        book/m09a-handmade-mjcf/figures/render_stand.py
产物：book/m09a-handmade-mjcf/figures/handmade_stand.png
"""
import os

import mujoco
from PIL import Image

REPO_ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "..", "..", "..")
SCENE = os.path.join(REPO_ROOT, "sim", "assets", "scene_handmade.xml")
OUT = os.path.join(REPO_ROOT, "docs", "milestones", "m9a-handmade-mjcf",
                   "assets", "dst", "handmade_stand.png")

model = mujoco.MjModel.from_xml_path(SCENE)
data = mujoco.MjData(model)
key_id = mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_KEY, "STAND")
mujoco.mj_resetDataKeyframe(model, data, key_id)
mujoco.mj_forward(model, data)

camera = mujoco.MjvCamera()
camera.lookat = [0, 0, 0.10]
camera.distance = 0.45
camera.azimuth = 135
camera.elevation = -12

renderer = mujoco.Renderer(model, height=480, width=640)
renderer.update_scene(data, camera=camera)
Image.fromarray(renderer.render()).save(OUT)
print(f"written: {OUT}")

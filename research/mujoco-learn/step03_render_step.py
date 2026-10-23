"""Tutorial step 3: render + mj_step animation (with a hinge joint)."""
from pathlib import Path

import imageio.v2 as imageio
import mujoco
import numpy as np

OUT = Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

# --- static frame: light + mj_forward ---
xml_static = """
<mujoco>
  <worldbody>
    <light name="top" pos="0 0 1"/>
    <geom name="red_box" type="box" size=".2 .2 .2" rgba="1 0 0 1"/>
    <geom name="green_sphere" pos=".2 .2 .2" size=".1" rgba="0 1 0 1"/>
  </worldbody>
</mujoco>
"""
model = mujoco.MjModel.from_xml_string(xml_static)
data = mujoco.MjData(model)
model.vis.scale.framelength = 2.0
model.vis.scale.framewidth = 0.15
scene_option = mujoco.MjvOption()
scene_option.frame = mujoco.mjtFrame.mjFRAME_WORLD  # RGB = XYZ world axes
with mujoco.Renderer(model) as renderer:
    mujoco.mj_forward(model, data)  # propagate state before drawing
    renderer.update_scene(data, scene_option=scene_option)
    imageio.imwrite(OUT / "step03_static.png", renderer.render())
print("wrote", OUT / "step03_static.png")

# --- animate: body + hinge joint, gravity swings it ---
xml_swing = """
<mujoco>
  <worldbody>
    <light name="top" pos="0 0 1"/>
    <body name="box_and_sphere" euler="0 0 -30">
      <joint name="swing" type="hinge" axis="1 -1 0" pos="-.2 -.2 -.2"/>
      <geom name="red_box" type="box" size=".2 .2 .2" rgba="1 0 0 1"/>
      <geom name="green_sphere" pos=".2 .2 .2" size=".1" rgba="0 1 0 1"/>
    </body>
  </worldbody>
</mujoco>
"""
model = mujoco.MjModel.from_xml_string(xml_swing)
data = mujoco.MjData(model)
model.vis.scale.framelength = 2.0
model.vis.scale.framewidth = 0.15
print("nq (DoF positions) =", model.nq, "  nv (DoF velocities) =", model.nv)
print("timestep =", model.opt.timestep, "s")

scene_option = mujoco.MjvOption()
scene_option.flags[mujoco.mjtVisFlag.mjVIS_JOINT] = True
scene_option.frame = mujoco.mjtFrame.mjFRAME_WORLD  # draw world XYZ (R/G/B)
# want body-local axes instead? use: mujoco.mjtFrame.mjFRAME_BODY

duration = 3.8
framerate = 60
frames = []
mujoco.mj_resetData(model, data)
with mujoco.Renderer(model) as renderer:
    while data.time < duration:
        mujoco.mj_step(model, data)  # x_{t+h} = f(x_t)
        if len(frames) < data.time * framerate:
            renderer.update_scene(data, scene_option=scene_option)
            frames.append(renderer.render().copy())

video_path = OUT / "step03_swing.mp4"
imageio.mimsave(video_path, frames, fps=framerate)
print("frames =", len(frames), "  final qpos =", np.asarray(data.qpos))
print("wrote", video_path)

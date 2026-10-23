"""Tutorial step 4: DoF / freejoint / two examples / timestep vs energy.

本步要学：
1. MuJoCo 用「加法」自由度：不加 joint 就不动；freejoint = 6 DoF
2. 例子 A：翻滚陀螺 tippe top（freejoint + keyframe）
3. 例子 B：混沌摆（多铰链，初值微扰会散开）
4. 时间步长越小，能量守恒越好（离散化误差）
"""
from pathlib import Path

import imageio.v2 as imageio
import matplotlib.pyplot as plt
import mujoco
import numpy as np

OUT = Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)


def save_video(path, frames, fps):
    imageio.mimsave(path, frames, fps=fps)
    print("wrote", path, f"({len(frames)} frames)")


# ---------------------------------------------------------------------------
# A) Tippe top — freejoint = 6 DoF (qpos 却是 7：位置3 + 四元数4)
# ---------------------------------------------------------------------------
tippe_top = """
<mujoco model="tippe top">
  <option integrator="RK4"/>
  <asset>
    <texture name="grid" type="2d" builtin="checker" rgb1=".1 .2 .3"
     rgb2=".2 .3 .4" width="300" height="300"/>
    <material name="grid" texture="grid" texrepeat="8 8" reflectance=".2"/>
  </asset>
  <worldbody>
    <geom name="floor" size=".2 .2 .01" type="plane" material="grid"/>
    <light pos="0 0 .6"/>
    <camera name="closeup" pos="0 -.1 .07" xyaxes="1 0 0 0 1 2"/>
    <body name="top" pos="0 0 .02">
      <freejoint/>
      <geom name="ball" type="sphere" size=".02"/>
      <geom name="stem" type="cylinder" pos="0 0 .02" size="0.004 .008"/>
      <geom name="ballast" type="box" size=".023 .023 0.005" pos="0 0 -.015"
       contype="0" conaffinity="0" group="3"/>
    </body>
  </worldbody>
  <keyframe>
    <key name="spinning" qpos="0 0 0.02 1 0 0 0" qvel="0 0 0 0 1 200"/>
  </keyframe>
</mujoco>
"""
model = mujoco.MjModel.from_xml_string(tippe_top)
data = mujoco.MjData(model)
print("=== tippe top ===")
print("nq =", model.nq, "  nv =", model.nv, "  (6 DoF: qpos has 7 because orientation is a quaternion)")
mujoco.mj_resetDataKeyframe(model, data, 0)
print("qpos (keyframe) =", np.asarray(data.qpos))
print("qvel (keyframe) =", np.asarray(data.qvel))

duration, framerate = 7.0, 60
frames = []
mujoco.mj_resetDataKeyframe(model, data, 0)
with mujoco.Renderer(model) as renderer:
    while data.time < duration:
        mujoco.mj_step(model, data)
        if len(frames) < data.time * framerate:
            renderer.update_scene(data, camera="closeup")
            frames.append(renderer.render().copy())
save_video(OUT / "step04_tippe_top.mp4", frames, framerate)

# stem height + angular velocity curves
timevals, angular_velocity, stem_height = [], [], []
mujoco.mj_resetDataKeyframe(model, data, 0)
while data.time < duration:
    mujoco.mj_step(model, data)
    timevals.append(data.time)
    angular_velocity.append(data.qvel[3:6].copy())
    stem_height.append(data.geom("stem").xpos[2])

fig, ax = plt.subplots(2, 1, figsize=(6, 6), sharex=True)
ax[0].plot(timevals, angular_velocity)
ax[0].set_title("tippe top angular velocity")
ax[0].set_ylabel("rad/s")
ax[1].plot(timevals, stem_height)
ax[1].set_title("stem height")
ax[1].set_xlabel("time (s)")
ax[1].set_ylabel("m")
fig.tight_layout()
fig.savefig(OUT / "step04_tippe_top_plots.png", dpi=120)
plt.close(fig)
print("wrote", OUT / "step04_tippe_top_plots.png")


# ---------------------------------------------------------------------------
# B) Chaotic pendulum — 4 hinges, sensitive to tiny qvel noise
# ---------------------------------------------------------------------------
chaotic_pendulum = """
<mujoco>
  <option timestep=".001">
    <flag energy="enable" contact="disable"/>
  </option>
  <default>
    <joint type="hinge" axis="0 -1 0"/>
    <geom type="capsule" size=".02"/>
  </default>
  <worldbody>
    <light pos="0 -.4 1"/>
    <camera name="fixed" pos="0 -1 0" xyaxes="1 0 0 0 0 1"/>
    <body name="0" pos="0 0 .2">
      <joint name="root"/>
      <geom fromto="-.2 0 0 .2 0 0" rgba="1 1 0 1"/>
      <geom fromto="0 0 0 0 0 -.25" rgba="1 1 0 1"/>
      <body name="1" pos="-.2 0 0">
        <joint/>
        <geom fromto="0 0 0 0 0 -.2" rgba="1 0 0 1"/>
      </body>
      <body name="2" pos=".2 0 0">
        <joint/>
        <geom fromto="0 0 0 0 0 -.2" rgba="0 1 0 1"/>
      </body>
      <body name="3" pos="0 0 -.25">
        <joint/>
        <geom fromto="0 0 0 0 0 -.2" rgba="0 0 1 1"/>
      </body>
    </body>
  </worldbody>
</mujoco>
"""
model = mujoco.MjModel.from_xml_string(chaotic_pendulum)
data = mujoco.MjData(model)
print("\n=== chaotic pendulum ===")
print("nq =", model.nq, "  nv =", model.nv)

n_seconds, framerate = 6, 30
n_frames = int(n_seconds * framerate)
frames = []
mujoco.mj_resetData(model, data)
data.joint("root").qvel = 10
with mujoco.Renderer(model, 240, 320) as renderer:
    for i in range(n_frames):
        while data.time * framerate < i:
            mujoco.mj_step(model, data)
        renderer.update_scene(data, camera="fixed")
        frames.append(renderer.render().copy())
save_video(OUT / "step04_pendulum.mp4", frames, framerate)


# ---------------------------------------------------------------------------
# C) Timestep vs energy conservation (same pendulum, no friction)
# ---------------------------------------------------------------------------
SIM_DURATION = 10.0
TIMESTEPS = np.power(10, np.linspace(-2, -4, 5))
fig, ax = plt.subplots(figsize=(7, 4))
for dt in TIMESTEPS:
    model.opt.timestep = float(dt)
    n_steps = int(SIM_DURATION / model.opt.timestep)
    sim_time = np.zeros(n_steps)
    energy = np.zeros(n_steps)
    mujoco.mj_resetData(model, data)
    data.qvel[0] = 9
    for i in range(n_steps):
        mujoco.mj_step(model, data)
        sim_time[i] = data.time
        energy[i] = data.energy[0] + data.energy[1]
    ax.plot(sim_time, energy, label=f"dt = {1000 * dt:g} ms")
    print(f"energy run: {n_steps} steps at dt = {1000 * dt:g} ms")

ax.set_title("total energy vs timestep (should be flatter when dt is smaller)")
ax.set_ylabel("Joule")
ax.set_xlabel("second")
ax.legend(frameon=True)
fig.tight_layout()
fig.savefig(OUT / "step04_energy_vs_dt.png", dpi=120)
plt.close(fig)
print("wrote", OUT / "step04_energy_vs_dt.png")

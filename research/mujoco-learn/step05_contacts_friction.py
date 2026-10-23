"""Tutorial step 5: contacts + friction.

本步要学：
1. freejoint 物体落地后产生接触；可画出接触点/接触力
2. mj_contactForce 读每个接触的力（法向 + 切向摩擦）
3. geom 的 friction 不同 → 斜面上滑得不一样
"""
from pathlib import Path

import imageio.v2 as imageio
import matplotlib.pyplot as plt
import mujoco
import numpy as np

OUT = Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)
rng = np.random.default_rng(0)


def save_video(path, frames, fps):
    imageio.mimsave(path, frames, fps=fps)
    print("wrote", path, f"({len(frames)} frames)")


# ---------------------------------------------------------------------------
# A) Free body on floor — visualize contact points & forces
# ---------------------------------------------------------------------------
free_body_MJCF = """
<mujoco>
  <asset>
    <texture name="grid" type="2d" builtin="checker" rgb1=".1 .2 .3"
    rgb2=".2 .3 .4" width="300" height="300" mark="edge" markrgb=".2 .3 .4"/>
    <material name="grid" texture="grid" texrepeat="2 2" texuniform="true"
    reflectance=".2"/>
  </asset>
  <worldbody>
    <light pos="0 0 1" mode="trackcom"/>
    <geom name="floor" type="plane" pos="0 0 -.5" size="2 2 .1" material="grid"
     solimp=".99 .99 .01" solref=".001 1"/>
    <body name="box_and_sphere" pos="0 0 0">
      <freejoint/>
      <geom name="red_box" type="box" size=".1 .1 .1" rgba="1 0 0 1"
       solimp=".99 .99 .01" solref=".001 1"/>
      <geom name="green_sphere" size=".06" pos=".1 .1 .1" rgba="0 1 0 1"/>
      <camera name="fixed" pos="0 -.6 .3" xyaxes="1 0 0 0 1 2"/>
      <camera name="track" pos="0 -.6 .3" xyaxes="1 0 0 0 1 2" mode="track"/>
    </body>
  </worldbody>
</mujoco>
"""
model = mujoco.MjModel.from_xml_string(free_body_MJCF)
data = mujoco.MjData(model)
print("=== free body contacts ===")
print("nq =", model.nq, "  nv =", model.nv)

options = mujoco.MjvOption()
mujoco.mjv_defaultOption(options)
options.flags[mujoco.mjtVisFlag.mjVIS_CONTACTPOINT] = True
options.flags[mujoco.mjtVisFlag.mjVIS_CONTACTFORCE] = True
options.flags[mujoco.mjtVisFlag.mjVIS_TRANSPARENT] = True
model.vis.scale.contactwidth = 0.1
model.vis.scale.contactheight = 0.03
model.vis.scale.forcewidth = 0.05
model.vis.map.force = 0.3

n_frames, height, width = 200, 240, 320
frames = []
mujoco.mj_resetData(model, data)
data.qvel[3:6] = 5 * rng.standard_normal(3)
with mujoco.Renderer(model, height, width) as renderer:
    for i in range(n_frames):
        while data.time < i / 120.0:  # 1/4x real time
            mujoco.mj_step(model, data)
        renderer.update_scene(data, camera="track", scene_option=options)
        frames.append(renderer.render().copy())
save_video(OUT / "step05_contacts.mp4", frames, fps=30)


# ---------------------------------------------------------------------------
# B) Contact force analysis plots
# ---------------------------------------------------------------------------
n_steps = 499
sim_time = np.zeros(n_steps)
ncon = np.zeros(n_steps)
force = np.zeros((n_steps, 3))
velocity = np.zeros((n_steps, model.nv))
penetration = np.zeros(n_steps)
acceleration = np.zeros((n_steps, model.nv))
forcetorque = np.zeros(6)

mujoco.mj_resetData(model, data)
data.qvel[3:6] = 2 * rng.standard_normal(3)
for i in range(n_steps):
    mujoco.mj_step(model, data)
    sim_time[i] = data.time
    ncon[i] = data.ncon
    velocity[i] = data.qvel[:]
    acceleration[i] = data.qacc[:]
    for j, c in enumerate(data.contact):
        mujoco.mj_contactForce(model, data, j, forcetorque)
        force[i] += forcetorque[0:3]
        penetration[i] = min(penetration[i], c.dist)

_, ax = plt.subplots(3, 2, sharex=True, figsize=(10, 10))
lines = ax[0, 0].plot(sim_time, force)
ax[0, 0].set_title("contact force")
ax[0, 0].set_ylabel("Newton")
ax[0, 0].legend(list(lines), ("normal z", "friction x", "friction y"))

ax[1, 0].plot(sim_time, acceleration)
ax[1, 0].set_title("acceleration")
ax[1, 0].set_ylabel("(meter,radian)/s/s")
ax[1, 0].legend(["ax", "ay", "az", "αx", "αy", "αz"])

ax[2, 0].plot(sim_time, velocity)
ax[2, 0].set_title("velocity")
ax[2, 0].set_ylabel("(meter,radian)/s")
ax[2, 0].set_xlabel("second")
ax[2, 0].legend(["vx", "vy", "vz", "ωx", "ωy", "ωz"])

ax[0, 1].plot(sim_time, ncon)
ax[0, 1].set_title("number of contacts")
ax[0, 1].set_yticks(range(6))

ax[1, 1].plot(sim_time, force[:, 0])
ax[1, 1].set_yscale("log")
ax[1, 1].set_title("normal (z) force - log scale")
ax[1, 1].set_ylabel("Newton")
mg = model.body("box_and_sphere").mass[0] * (-model.opt.gravity[2])
ax[1, 1].plot(sim_time, np.ones(n_steps) * mg, label="m*g", linewidth=1)
ax[1, 1].legend()

ax[2, 1].plot(sim_time, 1000 * penetration)
ax[2, 1].set_title("penetration depth")
ax[2, 1].set_ylabel("millimeter")
ax[2, 1].set_xlabel("second")
plt.tight_layout()
plt.savefig(OUT / "step05_contact_plots.png", dpi=120)
plt.close()
print("wrote", OUT / "step05_contact_plots.png")
print("max contacts in run:", int(ncon.max()))


# ---------------------------------------------------------------------------
# C) Friction: two boxes on a tilted floor (μ=default vs μ=0.33)
# ---------------------------------------------------------------------------
friction_MJCF = """
<mujoco>
  <asset>
    <texture name="grid" type="2d" builtin="checker" rgb1=".1 .2 .3"
     rgb2=".2 .3 .4" width="300" height="300" mark="none"/>
    <material name="grid" texture="grid" texrepeat="6 6"
     texuniform="true" reflectance=".2"/>
  </asset>
  <default>
    <geom type="box" size=".05 .05 .05"/>
    <joint type="free"/>
  </default>
  <worldbody>
    <light name="light" pos="-.2 0 1"/>
    <geom name="floor" type="plane" size=".5 .5 10" material="grid"
     zaxis="-.3 0 1" friction=".1"/>
    <camera name="y" pos="-.1 -.6 .3" xyaxes="1 0 0 0 1 2"/>
    <body pos="0 0 .1">
      <joint/>
      <geom/>
    </body>
    <body pos="0 .2 .1">
      <joint/>
      <geom friction=".33"/>
    </body>
  </worldbody>
</mujoco>
"""
model = mujoco.MjModel.from_xml_string(friction_MJCF)
data = mujoco.MjData(model)
print("\n=== friction compare ===")
print("floor friction =", model.geom("floor").friction)
print("box0 friction  =", model.geom(1).friction)  # first free box geom
print("box1 friction  =", model.geom(2).friction)

n_frames, height, width = 60, 300, 300
frames = []
with mujoco.Renderer(model, height, width) as renderer:
    mujoco.mj_resetData(model, data)
    for i in range(n_frames):
        while data.time < i / 30.0:
            mujoco.mj_step(model, data)
        renderer.update_scene(data, camera="y")
        frames.append(renderer.render().copy())
save_video(OUT / "step05_friction.mp4", frames, fps=30)

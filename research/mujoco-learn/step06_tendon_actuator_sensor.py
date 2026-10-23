"""Tutorial step 6: tendons, actuators, sensors.

本步要学：
1. site：挂点（锚点 / 挂钩 / IMU）
2. spatial tendon：两 site 之间的「绳子」，可限长
3. motor actuator：对关节施加 ctrl（力矩/力，经 gear）
4. accelerometer sensor：读 site 上的加速度；球棒击中时尖峰明显
"""
from pathlib import Path

import imageio.v2 as imageio
import matplotlib.pyplot as plt
import mujoco
import numpy as np

OUT = Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

MJCF = """
<mujoco>
  <asset>
    <texture name="grid" type="2d" builtin="checker" rgb1=".1 .2 .3"
     rgb2=".2 .3 .4" width="300" height="300" mark="none"/>
    <material name="grid" texture="grid" texrepeat="1 1"
     texuniform="true" reflectance=".2"/>
  </asset>

  <worldbody>
    <light name="light" pos="0 0 1"/>
    <geom name="floor" type="plane" pos="0 0 -.5" size="2 2 .1" material="grid"/>
    <site name="anchor" pos="0 0 .3" size=".01"/>
    <camera name="fixed" pos="0 -1.3 .5" xyaxes="1 0 0 0 1 2"/>

    <geom name="pole" type="cylinder" fromto=".3 0 -.5 .3 0 -.1" size=".04"/>
    <body name="bat" pos=".3 0 -.1">
      <joint name="swing" type="hinge" damping="1" axis="0 0 1"/>
      <geom name="bat" type="capsule" fromto="0 0 .04 0 -.3 .04"
       size=".04" rgba="0 0 1 1"/>
    </body>

    <body name="box_and_sphere" pos="0 0 0">
      <joint name="free" type="free"/>
      <geom name="red_box" type="box" size=".1 .1 .1" rgba="1 0 0 1"/>
      <geom name="green_sphere" size=".06" pos=".1 .1 .1" rgba="0 1 0 1"/>
      <site name="hook" pos="-.1 -.1 -.1" size=".01"/>
      <site name="IMU"/>
    </body>
  </worldbody>

  <tendon>
    <spatial name="wire" limited="true" range="0 0.35" width="0.003">
      <site site="anchor"/>
      <site site="hook"/>
    </spatial>
  </tendon>

  <actuator>
    <motor name="my_motor" joint="swing" gear="1"/>
  </actuator>

  <sensor>
    <accelerometer name="accelerometer" site="IMU"/>
  </sensor>
</mujoco>
"""

model = mujoco.MjModel.from_xml_string(MJCF)
data = mujoco.MjData(model)
print("=== tendon / actuator / sensor ===")
print("nu (actuators) =", model.nu, "  ntendon =", model.ntendon, "  nsensor =", model.nsensor)
print("actuator:", model.actuator(0).name, "→ joint", model.actuator(0).trntype)
print("tendon range:", model.tendon_range[0])
print("sensor dim:", model.sensor_dim[0], "  adr:", model.sensor_adr[0])

# static frame
with mujoco.Renderer(model, 480, 480) as renderer:
    mujoco.mj_forward(model, data)
    renderer.update_scene(data, camera="fixed")
    imageio.imwrite(OUT / "step06_setup.png", renderer.render())
print("wrote", OUT / "step06_setup.png")

# drive bat with constant ctrl, record accelerometer
n_frames = 180
height, width = 240, 320
fps = 60.0
frames = []
times = []
sensordata = []

mujoco.mj_resetData(model, data)
data.ctrl[0] = 20  # constant motor command

with mujoco.Renderer(model, height, width) as renderer:
    for i in range(n_frames):
        while data.time < i / fps:
            mujoco.mj_step(model, data)
            times.append(data.time)
            sensordata.append(data.sensor("accelerometer").data.copy())
        renderer.update_scene(data, camera="fixed")
        frames.append(renderer.render().copy())

imageio.mimsave(OUT / "step06_hit.mp4", frames, fps=fps)
print("wrote", OUT / "step06_hit.mp4", f"({len(frames)} frames)")

sensordata = np.asarray(sensordata)
times = np.asarray(times)
fig, ax = plt.subplots(figsize=(7, 4))
ax.plot(times, sensordata, label=[f"axis {v}" for v in ["x", "y", "z"]])
ax.set_title("Accelerometer values (bat hits show up as spikes)")
ax.set_ylabel("meter/second^2")
ax.set_xlabel("second")
ax.legend(frameon=True, loc="lower right")
fig.tight_layout()
fig.savefig(OUT / "step06_accel.png", dpi=120)
plt.close(fig)
print("wrote", OUT / "step06_accel.png")
print("accel peak |a| =", float(np.linalg.norm(sensordata, axis=1).max()))

"""Tutorial step 7: cameras + a bit of advanced rendering.

本步要学：
1. XML 里的命名相机：fixed（钉在 body 上）vs track（跟着 body 平移）
2. 自由相机 MjvCamera：用 lookat / distance / azimuth / elevation 绕看
3. 进阶渲染一瞥：depth、segmentation
"""
from pathlib import Path

import imageio.v2 as imageio
import mujoco
import numpy as np

OUT = Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)
rng = np.random.default_rng(1)


def save_video(path, frames, fps):
    imageio.mimsave(path, frames, fps=fps)
    print("wrote", path, f"({len(frames)} frames)")


MJCF = """
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
model = mujoco.MjModel.from_xml_string(MJCF)
data = mujoco.MjData(model)
print("=== cameras ===")
print("ncam =", model.ncam, " names =", [model.camera(i).name for i in range(model.ncam)])


def rollout_named(camera_name, out_name, n_frames=120, fps=30):
    frames = []
    mujoco.mj_resetData(model, data)
    data.qvel[3:6] = 4 * rng.standard_normal(3)
    with mujoco.Renderer(model, 240, 320) as renderer:
        for i in range(n_frames):
            while data.time < i / 60.0:  # slow-mo capture
                mujoco.mj_step(model, data)
            renderer.update_scene(data, camera=camera_name)
            frames.append(renderer.render().copy())
    save_video(OUT / out_name, frames, fps)


rollout_named("fixed", "step07_cam_fixed.mp4")
rollout_named("track", "step07_cam_track.mp4")


# --- free camera: orbit around lookat ---
cam = mujoco.MjvCamera()
mujoco.mjv_defaultCamera(cam)
cam.lookat[:] = [0, 0, -0.2]
cam.distance = 1.2
cam.elevation = -25

n_frames, fps = 90, 30
frames = []
mujoco.mj_resetData(model, data)
data.qvel[3:6] = 3 * rng.standard_normal(3)
with mujoco.Renderer(model, 240, 320) as renderer:
    for i in range(n_frames):
        while data.time < i / 60.0:
            mujoco.mj_step(model, data)
        cam.azimuth = 20 + 360 * (i / n_frames)  # full orbit
        renderer.update_scene(data, camera=cam)
        frames.append(renderer.render().copy())
save_video(OUT / "step07_cam_orbit.mp4", frames, fps)
print("orbit used lookat/distance/azimuth/elevation on MjvCamera")


# --- depth + segmentation stills ---
mujoco.mj_resetData(model, data)
mujoco.mj_forward(model, data)
with mujoco.Renderer(model, 240, 320) as renderer:
    renderer.update_scene(data, camera="fixed")
    imageio.imwrite(OUT / "step07_rgb.png", renderer.render())

    renderer.enable_depth_rendering()
    renderer.update_scene(data, camera="fixed")
    depth = renderer.render()
    depth = depth - depth.min()
    near = depth[depth <= 1]
    if near.size:
        depth = depth / (2 * near.mean())
    depth_u8 = (255 * np.clip(depth, 0, 1)).astype(np.uint8)
    imageio.imwrite(OUT / "step07_depth.png", depth_u8)
    renderer.disable_depth_rendering()

    renderer.enable_segmentation_rendering()
    renderer.update_scene(data, camera="fixed")
    seg = renderer.render()
    geom_ids = seg[:, :, 0].astype(np.float64) + 1
    geom_ids = geom_ids / max(geom_ids.max(), 1.0)
    imageio.imwrite(OUT / "step07_seg.png", (255 * geom_ids).astype(np.uint8))
    renderer.disable_segmentation_rendering()

print("wrote", OUT / "step07_rgb.png")
print("wrote", OUT / "step07_depth.png")
print("wrote", OUT / "step07_seg.png")

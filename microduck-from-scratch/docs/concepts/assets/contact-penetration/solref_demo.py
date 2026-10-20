import mujoco

TEMPLATE = """
<mujoco>
  <option timestep="0.005"/>
  <worldbody>
    <geom name="floor" type="plane" size="0 0 0.1" solref="{sr}"/>
    <body name="box" pos="0 0 0.3">
      <freejoint/>
      <geom type="box" size="0.05 0.05 0.05" mass="1.0" solref="{sr}"/>
    </body>
  </worldbody>
</mujoco>
"""

for tag, sr in [("软接触 (0.10, 1)",  (0.10, 1.0)),
                ("默认   (0.02, 1)",  (0.02, 1.0)),
                ("硬接触 (0.002, 1)", (0.002, 1.0))]:
    m = mujoco.MjModel.from_xml_string(TEMPLATE.format(sr=f"{sr[0]} {sr[1]}"))
    d = mujoco.MjData(m)
    d.qvel[2] = -4.0                      # 4 m/s 向下初速度，模拟大冲击
    worst = 0.0
    for _ in range(int(2.0 / m.opt.timestep)):
        mujoco.mj_step(m, d)
        worst = min(worst, d.qpos[2] - 0.05)   # 盒子底面 z
    print(f"{tag}: 冲击中底面最深 z = {worst:+.4f} m")

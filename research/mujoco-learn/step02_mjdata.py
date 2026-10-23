"""Tutorial step 2: MjData — state + derived quantities (need kinematics)."""
import mujoco

xml = """
<mujoco>
  <worldbody>
    <geom name="red_box" type="box" size=".2 .2 .2" rgba="1 0 0 1"/>
    <geom name="green_sphere" pos=".2 .2 .2" size=".1" rgba="0 1 0 1"/>
  </worldbody>
</mujoco>
"""
model = mujoco.MjModel.from_xml_string(xml)
data = mujoco.MjData(model)

print("time =", data.time)
print("qpos =", data.qpos)  # empty: no joints yet
print("qvel =", data.qvel)
print("geom_xpos (before kinematics):\n", data.geom_xpos)

mujoco.mj_kinematics(model, data)
print("geom_xpos (after mj_kinematics):\n", data.geom_xpos)
print("green_sphere xpos =", data.geom("green_sphere").xpos)

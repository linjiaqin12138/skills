"""Tutorial step 1: MJCF → mjModel, named access."""
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

print("ngeom =", model.ngeom)
print("geom_rgba =\n", model.geom_rgba)
print("green_sphere rgba =", model.geom("green_sphere").rgba)
print("geom names =", [model.geom(i).name for i in range(model.ngeom)])

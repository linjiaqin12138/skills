# mujoco 最小示例：一个单摆自由下落 1 秒，每 0.1s 打印摆角。
# 跑法（容器内）：python3 docs/concepts/assets/pendulum.py
import math
import mujoco

XML = """
<mujoco>
  <option timestep="0.01"/>
  <worldbody>
    <body name="pendulum" pos="0 0 1">
      <joint name="hinge" type="hinge" axis="0 1 0"/>
      <geom type="capsule" fromto="0 0 0 0 0 -0.5" size="0.02" mass="1"/>
    </body>
  </worldbody>
</mujoco>
"""

m = mujoco.MjModel.from_xml_string(XML)   # MJCF 字符串 → 编译成模型
d = mujoco.MjData(m)                       # 模型的动态状态（角度、速度、时间）
d.qpos[0] = math.radians(30)               # 初始：从竖直方向偏 30°

for i in range(11):
    mujoco.mj_step(m, d, nstep=10)         # 每 10 小步 × 0.01s = 前进 0.1s
    print(f"t={d.time:.1f}s  angle={math.degrees(d.qpos[0]):+.1f}deg")

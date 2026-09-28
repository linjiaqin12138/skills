#!/usr/bin/env python3
"""舵机 gain / torque 最小演示：P 增益是弹簧硬度，卸力是断动力。

一个单关节机械臂（一根杆），目标姿态水平（theta=0），重力往下拽。
舵机是位置伺服：输出力矩 = kp*(目标-当前) - kd*角速度——一根弹簧，
kp 就是弹簧硬度。三种工况：
  kp=8   满增益（对应鸭子 gain=200）：有点下垂，但扛得住
  kp=2   软倒增益（对应鸭子 gain=50）：明显下垂，但不和地板较劲
  卸力    kp/kd 清零（torque off）：没有弹簧，杆子垂下去挂在那
（kd 里含关节摩擦，所以卸力的杆子荡几下会停。）
"""

import math

G = 5.0       # 重力力矩强度
DT = 0.001    # 积分步长 1ms


def simulate(kp, kd, seconds=8.0):
    theta, omega = 0.0, 0.0     # 从水平姿态出发
    trace = []
    steps = int(seconds / DT)
    for i in range(steps + 1):
        torque = kp * (0.0 - theta) - kd * omega - G * math.cos(theta)
        # 半隐式欧拉：先更新速度，再用新速度更新角度（比显式欧拉稳）
        omega += torque * DT
        theta += omega * DT
        if i % 1000 == 0:       # 每 1s 记一个点
            trace.append((i * DT, theta))
    return trace


for name, kp, kd in [("kp=8 （满增益 gain=200）", 8.0, 2.5),
                     ("kp=2 （软倒增益 gain=50）", 2.0, 1.5),
                     ("卸力（torque off，只剩摩擦）", 0.0, 0.6)]:
    trace = simulate(kp, kd)
    pts = " ".join(f"{t:.0f}s:{math.degrees(th):+4.0f}deg" for t, th in trace)
    steady = math.degrees(trace[-1][1])
    print(f"{name}\n  {pts}\n  → 稳态 {steady:+.0f}deg")

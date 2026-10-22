"""四元数最小实验：固定 z 轴。

只用 Python 3 标准库。每组输出承担一个观察任务：
[1] 角度 -> 四个数
[2] 四个数 -> 作用于向量
[3] q 与 -q 作用于向量得到同一结果
[4] 不归一化的连乘会让范数漂移
"""

import math
import struct


def q_mul(a, b):
    aw, ax, ay, az = a
    bw, bx, by, bz = b
    return (
        aw * bw - ax * bx - ay * by - az * bz,
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
    )


def rotate(q, v):
    """用 q 旋转向量 v：q * (0, v) * q 的共轭。"""
    p = (0.0, v[0], v[1], v[2])
    conjugate = (q[0], -q[1], -q[2], -q[3])
    return q_mul(q_mul(q, p), conjugate)[1:]


def z_axis_q(degrees):
    """只绕 z 轴：x = y = 0，w = cos(theta/2)，z = sin(theta/2)。"""
    half = math.radians(degrees) / 2
    return (math.cos(half), 0.0, 0.0, math.sin(half))


def full_angle(q):
    """从 q 还原整角的 cos 和 sin：cos = w^2 - z^2，sin = 2 w z。"""
    w, z = q[0], q[3]
    return w * w - z * z, 2 * w * z


def norm(a):
    return math.sqrt(sum(x * x for x in a))


def scale(a, s):
    return tuple(x * s for x in a)


def normalize(a):
    return scale(a, 1 / norm(a))


def from_axis_angle(axis, degrees):
    axis = normalize(axis)
    half = math.radians(degrees) / 2
    return (math.cos(half), *scale(axis, math.sin(half)))


def fmt(v):
    return "[" + ", ".join(f"{0 if abs(x) < 5e-10 else x:.4f}" for x in v) + "]"


def f32(x):
    """舍入到 IEEE 754 单精度，模拟用 f32 存姿态。"""
    return struct.unpack("f", struct.pack("f", x))[0]


def drift(steps, single_precision):
    """连乘 steps 次，每步绕略微不同的轴转 20 度，中间不归一化。"""
    q = (1.0, 0.0, 0.0, 0.0)
    for i in range(steps):
        axis = (0.01 * math.sin(i * 0.001), 1.0, 0.01 * math.cos(i * 0.0007))
        s = from_axis_angle(axis, 20)
        if single_precision:
            s = tuple(f32(c) for c in s)
            q = tuple(f32(c) for c in q_mul(q, s))
        else:
            q = q_mul(q, s)
    return q


print("[1] 绕 z 轴：角度 -> 四个数（w, x, y, z）")
print("角度      q                            cos(theta)   sin(theta)")
for deg in (0, 90, 180, 360, 720):
    q = z_axis_q(deg)
    c, s = full_angle(q)
    print(f"{deg:>4} deg  {fmt(q)}  {c:>9.4f}  {s:>9.4f}")

print("\n[2] q(90 deg) 作用于 v = (1, 0, 0)")
q90 = z_axis_q(90)
v = (1.0, 0.0, 0.0)
print("脚本的 rotate() =", fmt(rotate(q90, v)))
c, s = full_angle(q90)
print(f"2 维公式  x' = x*cos - y*sin, y' = x*sin + y*cos, cos = {c:.4f}, sin = {s:.4f}")
print(f"手算      (1*cos - 0*sin, 1*sin + 0*cos, 0) = ({c:.4f}, {s:.4f}, 0)")

print("\n[3] q 与 -q 作用到向量：结果相同")
v = (0.3, -0.4, 0.5)
q0 = z_axis_q(0)
q360 = z_axis_q(360)
print("q(0 deg)   =", fmt(q0))
print("q(360 deg) =", fmt(q360), " <- 正好是 -q(0 deg)")
print("rotate(q0)   =", fmt(rotate(q0, v)))
print("rotate(-q0)  =", fmt(rotate(scale(q0, -1), v)))
print("rotate(q360) =", fmt(rotate(q360, v)))

print("\n[4] 不归一化的连乘：范数漂移")
step = from_axis_angle((0.0, 1.0, 0.0), 20)
q = (1.0, 0.0, 0.0, 0.0)
for _ in range(100000):
    q = q_mul(q, step)
print(f"float64  同一个步长连乘 1e5 次：范数 = {norm(q):.16f}")
print(f"float64  轴微变连乘 1e5 次：范数 = {norm(drift(100000, False)):.16f}")
print(f"float32  轴微变连乘 1e5 次：范数 = {norm(drift(100000, True)):.16f}")

noisy = scale(step, 1 + 1e-6)
q = (1.0, 0.0, 0.0, 0.0)
for _ in range(100000):
    q = q_mul(q, noisy)
print(f"每步输入范数 1+1e-6，连乘 1e5 次：范数 = {norm(q):.6f}")
print(f"  用它旋转 (1,0,0) 得到 {fmt(rotate(q, (1.0, 0.0, 0.0)))}")
print(f"  长度应为 1，实测 {norm(rotate(q, (1.0, 0.0, 0.0))):.6f}（范数的平方）")
print("  归一化之后：", fmt(normalize(q)))

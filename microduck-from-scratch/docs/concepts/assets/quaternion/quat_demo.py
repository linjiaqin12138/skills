import math


def add(a, b):
    return tuple(x + y for x, y in zip(a, b))


def scale(a, s):
    return tuple(x * s for x in a)


def norm(a):
    return math.sqrt(sum(x * x for x in a))


def normalize(a):
    return scale(a, 1 / norm(a))


def q_mul(a, b):
    aw, ax, ay, az = a
    bw, bx, by, bz = b
    return (
        aw * bw - ax * bx - ay * by - az * bz,
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
    )


def from_axis_angle(axis, degrees):
    axis = normalize(axis)
    half = math.radians(degrees) / 2
    return (math.cos(half), *scale(axis, math.sin(half)))


def to_axis_angle(q):
    w, x, y, z = normalize(q)
    angle = math.degrees(2 * math.acos(max(-1, min(1, w))))
    s = math.sqrt(max(0, 1 - w * w))
    if s < 1e-10:
        return (1, 0, 0), 0.0
    return (x / s, y / s, z / s), angle


def rotate(q, v):
    p = (0, *v)
    conjugate = (q[0], *scale(q[1:], -1))
    return q_mul(q_mul(q, p), conjugate)[1:]


def nlerp(a, b, t):
    return normalize(add(scale(a, 1 - t), scale(b, t)))


def slerp(a, b, t):
    dot = sum(x * y for x, y in zip(a, b))
    if dot < 0:
        a, dot = scale(a, -1), -dot
    if dot > 0.9995:
        return nlerp(a, b, t)
    theta = math.acos(min(1, dot))
    return add(
        scale(a, math.sin((1 - t) * theta) / math.sin(theta)),
        scale(b, math.sin(t * theta) / math.sin(theta)),
    )


def fmt(v):
    return "[" + ", ".join(f"{0 if abs(x) < 5e-10 else x:.4f}" for x in v) + "]"


def z_angle(q):
    angle = math.degrees(2 * math.atan2(q[3], q[0]))
    return 0.0 if abs(angle) < 5e-10 else angle


print("[1] 轴角 -> quaternion -> 轴角（wxyz）")
q = from_axis_angle((1, 2, 2), 80)
axis, angle = to_axis_angle(q)
print("q =", fmt(q))
print("axis =", fmt(axis), f"angle = {angle:.4f} deg")

print("\n[2] q 与 -q 是同一姿态")
q0 = from_axis_angle((0, 0, 1), 0)
q360 = from_axis_angle((0, 0, 1), 360)
v = (0.3, -0.4, 0.5)
print("q(0 deg)   =", fmt(q0))
print("q(360 deg) =", fmt(q360))
print("rotate(q)   =", fmt(rotate(q, v)))
print("rotate(-q)  =", fmt(rotate(scale(q, -1), v)))
print("rotate(q360)=", fmt(rotate(q360, v)))

print("\n[3] 复合：qz * qx 表示先转 qx，再转 qz")
qx = from_axis_angle((1, 0, 0), 90)
qz = from_axis_angle((0, 0, 1), 90)
v = (0, 1, 1)
print("分两次旋转   =", fmt(rotate(qz, rotate(qx, v))))
print("qz * qx     =", fmt(rotate(q_mul(qz, qx), v)))
print("反序 qx * qz =", fmt(rotate(q_mul(qx, qz), v)))

print("\n[4] 未归一化的 quaternion 连乘会漂移")
step = scale(from_axis_angle((0, 1, 0), 20), 1.001)
drift = (1, 0, 0, 0)
for _ in range(100):
    drift = q_mul(drift, step)
print(f"连乘 100 次后的范数 = {norm(drift):.6f}")
drift = normalize(drift)
print(f"归一化后的范数       = {norm(drift):.6f}")

print("\n[5] 350 deg -> 10 deg：nlerp 未校正符号，slerp 走短弧")
a = from_axis_angle((0, 0, 1), 350)
b = from_axis_angle((0, 0, 1), 10)
print("t     nlerp角度    slerp角度")
for t in (0, 0.25, 0.5, 0.75, 1):
    print(f"{t:.2f}  {z_angle(nlerp(a, b, t)):10.4f}  {z_angle(slerp(a, b, t)):10.4f}")

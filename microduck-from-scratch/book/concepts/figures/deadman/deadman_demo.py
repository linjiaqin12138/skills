#!/usr/bin/env python3
"""deadman 开关最小演示：司机失联，机器自己停下。

模拟一个 50Hz 的控制循环（每拍 20ms）：
  - "司机"（手柄/遥控端）每 100ms 发一条意图：vx=0.15
  - 接收端只信 500ms 内到过的意图；超龄就把速度清零
  - t=2.0s 时司机"猝死"（程序崩了/掉线了），之后再也没有意图

没有 deadman 的对照组：最后一条意图永远有效，机器人永远走下去。
"""

TICK = 0.02          # 控制拍 20ms（50Hz）
DEADMAN = 0.5        # 意图保鲜期 500ms
RESEND = 0.1         # 司机每 100ms 重发一次意图
CRASH_AT = 2.0       # 司机在 t=2.0s 失联


def run(use_deadman):
    intent_vx, last_intent_at = 0.0, None
    next_send = 0.0
    vx = 0.0
    out = []
    t = 0.0
    while t < 3.5:
        # 司机侧：到点就发意图（2.0s 后司机没了）
        if t >= next_send and t < CRASH_AT:
            intent_vx, last_intent_at = 0.15, t
            next_send += RESEND
        # 接收侧：每拍决定用多大的速度
        if use_deadman and (last_intent_at is None or t - last_intent_at > DEADMAN):
            vx = 0.0          # 意图超龄：清零
        else:
            vx = intent_vx    # 意图新鲜：照走
        out.append((t, vx))
        t += TICK
    return out


def show(name, trace):
    # 每 0.3s 采一个点打印
    pts = [(t, vx) for (t, vx) in trace if abs(t / 0.3 - round(t / 0.3)) < TICK / 2]
    line = " ".join(f"{t:.1f}s:{vx:.2f}" for t, vx in pts)
    print(f"{name}\n  {line}")


show("有 deadman：", run(use_deadman=True))
show("无 deadman：", run(use_deadman=False))

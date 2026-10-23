#!/usr/bin/env python3
# examples/boot_counter_order.py —— python3 examples/boot_counter_order.py 可跑
#
# 演示 pending.json（boot counter）为什么必须在 swap 之前 arm：
# 把一次 apply 拆成四步，在每一步之后模拟 kill -9，重启后按磁盘上的
# pending 恢复。对比两种顺序的结局：arm 在 swap 前 vs arm 在 swap 后。
# 判据：新版已在跑（current 指向新版）但 pending 不存在 = 没人记账，
# 重启后永远没人把它撤回去——哪怕新版是坏的。

STEPS = ["arm_pending", "swap_current", "spawn_new", "confirm_clear_pending"]
ARM_FIRST = STEPS
ARM_LATE = ["swap_current", "spawn_new", "arm_pending", "confirm_clear_pending"]


def run(order, crash_after, new_is_bad):
    current, pending, live = "1.0.0", None, "1.0.0"
    for step in order:
        if step == "arm_pending":
            pending = {"version": "2.0.0", "previous": "1.0.0", "boots": 0}
        elif step == "swap_current":
            current = "2.0.0"
        elif step == "spawn_new":
            live = "2.0.0"
        elif step == "confirm_clear_pending":
            pending = None
        if step == crash_after:
            break  # kill -9 落在这里：之后的步骤没发生
    # 重启恢复：只认磁盘上的 pending。在 = 重跑健康门裁决；不在 = 照常从
    # current 起 daemon（engine.rs:215-219），谁也不觉得有事发生。
    if pending is not None:
        outcome = "回滚 1.0.0" if new_is_bad else "confirm 2.0.0"
    else:
        outcome = f"重启从 current 起 {current}，无人记账"
    return f"current={current} live={live} pending={'有' if pending else '无'} → {outcome}"


for crash in ["arm_pending", "swap_current", "spawn_new"]:
    print(f"--- kill -9 落在 {crash} 之后（新版 2.0.0 是坏的）---")
    print(f"  arm 在 swap 前: {run(ARM_FIRST, crash, new_is_bad=True)}")
    print(f"  arm 在 swap 后: {run(ARM_LATE, crash, new_is_bad=True)}")
print()
print("（假设新版 2.0.0 是坏的。注意「arm 在 swap 后」列 swap_current/")
print(" spawn_new 两行：新代码在跑，pending 还没写——崩溃恢复失明。）")

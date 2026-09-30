#!/usr/bin/env python3
# examples/atomic_symlink_swap.py —— python3 examples/atomic_symlink_swap.py 可跑
#
# 演示「换 symlink 指向」的两种做法在并发读者眼里的差别：
#   1. 先删再建：unlink(current) → symlink(current) —— 中间有一段 current 不存在
#   2. 临时链接 + rename(2)：symlink(.current.tmp) → rename 覆盖 current —— 原子
# 一个读者线程全程经 current 读版本号，统计它读到「不存在」的次数。

import os
import shutil
import sys
import tempfile
import threading

root = tempfile.mkdtemp(prefix="symlink-swap-demo-")
os.makedirs(f"{root}/releases/1.0.0")
os.makedirs(f"{root}/releases/2.0.0")
current = f"{root}/current"
tmp_link = f"{root}/.current.tmp"
os.symlink("releases/1.0.0", current)

stop = threading.Event()
missing = 0  # 读者撞见 current 不存在的次数
reads = 0


def reader():
    global missing, reads
    while not stop.is_set():
        reads += 1
        try:
            os.readlink(current)  # 消费方经 current 读：指向谁就读谁
        except FileNotFoundError:
            missing += 1


def swap_delete_create(target):
    os.unlink(current)
    os.symlink(target, current)


def swap_rename(target):
    if os.path.lexists(tmp_link):
        os.unlink(tmp_link)
    os.symlink(target, tmp_link)
    os.rename(tmp_link, current)  # 同一目录内 rename 是原子的


def run(label, swap):
    global missing, reads
    missing, reads = 0, 0
    stop.clear()
    t = threading.Thread(target=reader)
    t.start()
    for i in range(2000):
        swap(f"releases/{'2.0.0' if i % 2 else '1.0.0'}")
    stop.set()
    t.join()
    print(f"{label}: 读者读 {reads} 次，撞见 current 不存在 {missing} 次")


run("先删再建        ", swap_delete_create)
run("临时链接+rename ", swap_rename)

shutil.rmtree(root)

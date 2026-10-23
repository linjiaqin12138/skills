"""M9a 数据提取（一次性工具，不进验收链路）：

在容器内跑（无头、纯计算）：
    docker compose exec -T rust /opt/sim-venv/bin/python scripts/m9a_extract_official.py

1. 编译官方 robot_groundcontact.xml，导出逐 body 质量/质心/惯量（body frame）。
2. 按 body 聚合 collision/visual geom 的 body 系 AABB（近似基本几何体尺寸用）。
3. 官方 scene_groundcontact 推倒/落体对照：复现 D26 穿透数据。
"""
import os
import sys

import mujoco
import numpy as np

ASSETS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "sim", "assets")
ROBOT = os.path.join(ASSETS, "robot_groundcontact.xml")
SCENE = os.path.join(ASSETS, "scene_groundcontact.xml")

model = mujoco.MjModel.from_xml_path(ROBOT)

print("=== 关节（模型顺序） ===")
for j in range(model.njnt):
    print(f"  [{j}] {mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_JOINT, j)} "
          f"type={model.jnt_type[j]} range={model.jnt_range[j]}")

print("\n=== body 树（编译后顺序） ===")
for b in range(1, model.nbody):
    name = mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_BODY, b)
    parent = mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_BODY, model.body_parentid[b])
    print(f"  [{b}] {name}  parent={parent} pos={model.body_pos[b]} quat={model.body_quat[b]}")

print("\n=== 逐 body 惯量（body 系；body_inertia 为主轴系对角值） ===")
total = 0.0
for b in range(1, model.nbody):
    name = mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_BODY, b)
    m = model.body_mass[b]
    total += m
    print(f"  {name}: mass={m:.6g} ipos={model.body_ipos[b]} "
          f"inertia={model.body_inertia[b]} iquat={model.body_iquat[b]}")
print(f"  总质量 = {total:.6f} kg")

print("\n=== 逐 body geom AABB 聚合（body 系；由官方编译模型量得，仅供近似 sizing） ===")
for b in range(1, model.nbody):
    name = mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_BODY, b)
    lo_col = np.full(3, np.inf); hi_col = np.full(3, -np.inf)
    lo_vis = np.full(3, np.inf); hi_vis = np.full(3, -np.inf)
    for g in range(model.ngeom):
        if model.geom_bodyid[g] != b:
            continue
        gname = mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_GEOM, g) or ""
        center = model.geom_aabb[g][:3]
        half = model.geom_aabb[g][3:6]
        # aabb 是 geom 系轴对齐；转到 body 系取 8 角点
        pos = model.geom_pos[g]
        quat = model.geom_quat[g]
        rot = np.zeros(9)
        mujoco.mju_quat2Mat(rot, quat)
        rot = rot.reshape(3, 3)
        corners = []
        for sx in (-1, 1):
            for sy in (-1, 1):
                for sz in (-1, 1):
                    c = pos + rot @ (center + half * np.array([sx, sy, sz]))
                    corners.append(c)
        corners = np.array(corners)
        lo, hi = corners.min(axis=0), corners.max(axis=0)
        collides = model.geom_contype[g] != 0 or model.geom_conaffinity[g] != 0
        if collides:
            lo_col = np.minimum(lo_col, lo); hi_col = np.maximum(hi_col, hi)
            tag = "COLL"
        else:
            lo_vis = np.minimum(lo_vis, lo); hi_vis = np.maximum(hi_vis, hi)
            tag = "vis"
        print(f"    {tag} geom[{g}] {gname} body_lo={lo.round(4)} body_hi={hi.round(4)}")
    if np.isfinite(lo_col).all():
        print(f"  => {name} COLL-union lo={lo_col.round(4)} hi={hi_col.round(4)} "
              f"size={(hi_col - lo_col).round(4)}")
    if np.isfinite(lo_vis).all():
        print(f"  => {name} VIS -union lo={lo_vis.round(4)} hi={hi_vis.round(4)} "
              f"size={(hi_vis - lo_vis).round(4)}")

print("\n=== 地板接触参数（官方 scene） ===")
sm = mujoco.MjModel.from_xml_path(SCENE)
print(f"  timestep={sm.opt.timestep}")
floor_gid = mujoco.mj_name2id(sm, mujoco.mjtObj.mjOBJ_GEOM, "floor")
print(f"  floor: solref={sm.geom_solref[floor_gid]} solimp={sm.geom_solimp[floor_gid]} "
      f"friction={sm.geom_friction[floor_gid]} condim={sm.geom_condim[floor_gid]}")
# 官方机器人 collision geom 的接触参数
for g in range(sm.ngeom):
    if sm.geom_contype[g] or sm.geom_conaffinity[g]:
        gname = mujoco.mj_id2name(sm, mujoco.mjtObj.mjOBJ_GEOM, g) or ""
        bname = mujoco.mj_id2name(sm, mujoco.mjtObj.mjOBJ_BODY, sm.geom_bodyid[g])
        if bname != "world":
            print(f"  robot geom {gname or g} body={bname} solref={sm.geom_solref[g]} "
                  f"solimp={sm.geom_solimp[g]} friction={sm.geom_friction[g]}")

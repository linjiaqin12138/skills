"""M9a Bite 1 验收核心：自建 MJCF（duck_handmade）对照官方 robot_groundcontact。

断言组：
  A. 结构：14 hinge 关节同名同序、body 树同构、nu=14、碰撞 geom 全 box
     （视觉挂官方 STL 网格，contype/conaffinity=0 不进物理）
  B. 惯量对账：总质量 ≤3%；逐 body 质量 ≤5%、质心 ≤2mm、对角惯量 ≤15%
     （阈值为代理假设：显式 <inertial> 直写官方编译值，误差应≈0，阈值防回归）
  C. STAND keyframe：重置后脚掌碰撞盒最低点贴地（|z| ≤ 5mm），与官方同姿势对比
  D. D26 穿透：卧姿落体 + 站姿推倒，稳定后所有碰撞 geom 最低点 z ≥ -1mm；
     官方 scene_groundcontact 同条件跑对照组（只记录，不断言）
"""
import os
import sys

import mujoco
import numpy as np

ASSETS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "assets")
OFFICIAL_SCENE = os.path.join(ASSETS, "scene_groundcontact.xml")
HANDMADE_SCENE = os.path.join(ASSETS, "scene_handmade.xml")

JOINT_NAMES = [
    "left_hip_yaw", "left_hip_roll", "left_hip_pitch", "left_knee", "left_ankle",
    "neck_pitch", "head_pitch", "head_yaw", "head_roll",
    "right_hip_yaw", "right_hip_roll", "right_hip_pitch", "right_knee", "right_ankle",
]

failures = []


def check(ok, label, detail=""):
    print(f"  [{'PASS' if ok else 'FAIL'}] {label}" + (f"  {detail}" if detail else ""))
    if not ok:
        failures.append(label)


def geom_corner_min_z(model, data):
    """所有碰撞 geom（contype 或 conaffinity 非零）8 角点的最低世界 z。"""
    worst = np.inf
    worst_name = None
    for g in range(model.ngeom):
        if model.geom_contype[g] == 0 and model.geom_conaffinity[g] == 0:
            continue
        if model.geom_bodyid[g] == 0:
            continue  # 地板自己不算
        center = data.geom_xpos[g]
        rot = data.geom_xmat[g].reshape(3, 3)
        half = model.geom_size[g]
        for sx in (-1, 1):
            for sy in (-1, 1):
                for sz in (-1, 1):
                    z = (center + rot @ (half * np.array([sx, sy, sz])))[2]
                    if z < worst:
                        worst = z
                        worst_name = f"geom#{g}"
    return worst, worst_name


def settle(model, data, seconds):
    for _ in range(int(seconds / model.opt.timestep)):
        mujoco.mj_step(model, data)


def drop_test(scene_path, label):
    """卧姿落体：躯干放平（绕 y 轴转 90°）从 z=0.10 释放，稳定 2s 后查穿透。"""
    model = mujoco.MjModel.from_xml_path(scene_path)
    data = mujoco.MjData(model)
    data.qpos[:3] = [0, 0, 0.10]
    data.qpos[3:7] = [0.7071068, 0, 0.7071068, 0]  # 绕 y 转 90°，躯干躺平
    settle(model, data, 2.0)
    z, name = geom_corner_min_z(model, data)
    trunk_z = data.qpos[2]
    print(f"  [{label}] 落体后: 最低碰撞角点 z={z:+.4f} ({name}), 躯干 z={trunk_z:+.4f}")
    return z


def push_test(scene_path, label):
    """推倒：STAND 起，躯干给 2.0 m/s 水平初速度，稳定 3s 后查穿透。"""
    model = mujoco.MjModel.from_xml_path(scene_path)
    data = mujoco.MjData(model)
    key_id = mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_KEY, "STAND")
    mujoco.mj_resetDataKeyframe(model, data, key_id)
    data.qvel[0] = 2.0
    settle(model, data, 3.0)
    z, name = geom_corner_min_z(model, data)
    trunk_z = data.qpos[2]
    print(f"  [{label}] 推倒后: 最低碰撞角点 z={z:+.4f} ({name}), 躯干 z={trunk_z:+.4f}")
    return z


def main():
    print("== A. 结构 ==")
    off = mujoco.MjModel.from_xml_path(OFFICIAL_SCENE)
    new = mujoco.MjModel.from_xml_path(HANDMADE_SCENE)

    contact_types = {int(new.geom_type[g]) for g in range(new.ngeom)
                     if new.geom_bodyid[g] != 0
                     and (new.geom_contype[g] != 0 or new.geom_conaffinity[g] != 0)}
    check(contact_types == {int(mujoco.mjtGeom.mjGEOM_BOX)},
          "碰撞 geom 全 box（视觉 STL 不进物理）", f"实际类型: {sorted(contact_types)}")
    n_mesh = sum(1 for g in range(new.ngeom)
                 if new.geom_type[g] == mujoco.mjtGeom.mjGEOM_MESH)
    check(n_mesh == 70, "视觉网格 geom 数 = 70（官方 STL 照抄）", f"实际 {n_mesh}")

    # 视觉网格位姿对账：编译后 geom pos/quat 与官方逐位比对（防照抄转写回归，
    # 2026-10-20 曾抓出 8 处 quat 符号抄错——肉眼在 XML 里几乎不可见）
    def mesh_geom_table(m):
        tbl = {}
        for g in range(m.ngeom):
            if m.geom_type[g] != mujoco.mjtGeom.mjGEOM_MESH:
                continue
            body = mujoco.mj_id2name(m, mujoco.mjtObj.mjOBJ_BODY, m.geom_bodyid[g])
            mesh = mujoco.mj_id2name(m, mujoco.mjtObj.mjOBJ_MESH, m.geom_dataid[g])
            tbl.setdefault((body, mesh), []).append(g)
        return tbl

    off_tbl = mesh_geom_table(off)
    vis_worst = 0.0
    for g in range(new.ngeom):
        if new.geom_type[g] != mujoco.mjtGeom.mjGEOM_MESH:
            continue
        body = mujoco.mj_id2name(new, mujoco.mjtObj.mjOBJ_BODY, new.geom_bodyid[g])
        mesh = mujoco.mj_id2name(new, mujoco.mjtObj.mjOBJ_MESH, new.geom_dataid[g])
        cands = off_tbl.get((body, mesh), [])
        best = min(
            max(np.abs(new.geom_pos[g] - off.geom_pos[og]).max(),
                min(np.abs(new.geom_quat[g] - off.geom_quat[og]).max(),
                    np.abs(new.geom_quat[g] + off.geom_quat[og]).max()))
            for og in cands) if cands else np.inf
        vis_worst = max(vis_worst, best)
    check(vis_worst <= 1e-5, "视觉网格位姿与官方编译值对账", f"最差 {vis_worst:.2e}")

    new_joints = [mujoco.mj_id2name(new, mujoco.mjtObj.mjOBJ_JOINT, j) for j in range(new.njnt)]
    check(new_joints[1:] == JOINT_NAMES and new_joints[0] == "trunk_base_freejoint",
          "关节名同序（freejoint + 14 hinge）", f"实际: {new_joints}")
    check(all(new.jnt_type[j] == mujoco.mjtJoint.mjJNT_HINGE for j in range(1, new.njnt)),
          "14 关节全 hinge")
    off_bodies = sorted(mujoco.mj_id2name(off, mujoco.mjtObj.mjOBJ_BODY, b) for b in range(1, off.nbody))
    new_bodies = sorted(mujoco.mj_id2name(new, mujoco.mjtObj.mjOBJ_BODY, b) for b in range(1, new.nbody))
    check(new_bodies == off_bodies, "body 树同构（15 body 同名）")
    off_parents = [(mujoco.mj_id2name(off, mujoco.mjtObj.mjOBJ_BODY, b),
                    mujoco.mj_id2name(off, mujoco.mjtObj.mjOBJ_BODY, off.body_parentid[b]))
                   for b in range(1, off.nbody)]
    new_parents = [(mujoco.mj_id2name(new, mujoco.mjtObj.mjOBJ_BODY, b),
                    mujoco.mj_id2name(new, mujoco.mjtObj.mjOBJ_BODY, new.body_parentid[b]))
                   for b in range(1, new.nbody)]
    check(sorted(new_parents) == sorted(off_parents), "body 父子关系同构")
    check(new.nu == 14, "nu=14", f"实际 nu={new.nu}")
    new_acts = [mujoco.mj_id2name(new, mujoco.mjtObj.mjOBJ_ACTUATOR, a) for a in range(new.nu)]
    check(new_acts == JOINT_NAMES, "执行器同序", f"实际: {new_acts}")
    off_ranges = {mujoco.mj_id2name(off, mujoco.mjtObj.mjOBJ_JOINT, j): off.jnt_range[j].copy()
                  for j in range(off.njnt)}
    range_ok = all(np.allclose(new.jnt_range[j], off_ranges[new_joints[j]]) for j in range(1, new.njnt))
    check(range_ok, "关节 range 全对齐")
    dof_ok = all(
        np.isclose(new.dof_armature[new.jnt_dofadr[j]], 0.0018)
        and np.isclose(new.dof_damping[new.jnt_dofadr[j]], 0.053)
        and np.isclose(new.dof_frictionloss[new.jnt_dofadr[j]], 0.0048)
        for j in range(1, new.njnt))
    check(dof_ok, "关节 damping/frictionloss/armature 对齐 chosen_actuator 编译值")
    act_ok = all(
        np.isclose(new.actuator_gainprm[a, 0], 0.55)
        and np.allclose(new.actuator_forcerange[a], [-0.96, 0.96])
        for a in range(new.nu))
    check(act_ok, "执行器 kp=0.55 / forcerange=±0.96 对齐")

    print("== B. 惯量对账（左官方 / 右自建） ==")
    print(f"  {'body':<18} {'质量%':>7} {'质心mm':>8} {'惯量max%':>9}")
    total_off = off.body_mass[1:].sum()
    total_new = new.body_mass[1:].sum()
    worst_mass = worst_ipos = worst_inertia = 0.0
    for b in range(1, off.nbody):
        name = mujoco.mj_id2name(off, mujoco.mjtObj.mjOBJ_BODY, b)
        bn = mujoco.mj_name2id(new, mujoco.mjtObj.mjOBJ_BODY, name)
        d_mass = abs(new.body_mass[bn] - off.body_mass[b]) / off.body_mass[b] * 100
        d_ipos = np.abs(new.body_ipos[bn] - off.body_ipos[b]).max() * 1000
        d_inertia = (np.abs(new.body_inertia[bn] - off.body_inertia[b])
                     / off.body_inertia[b]).max() * 100
        worst_mass = max(worst_mass, d_mass)
        worst_ipos = max(worst_ipos, d_ipos)
        worst_inertia = max(worst_inertia, d_inertia)
        print(f"  {name:<18} {d_mass:>6.3f}% {d_ipos:>7.3f}m {d_inertia:>8.3f}%")
    d_total = abs(total_new - total_off) / total_off * 100
    print(f"  总质量: 官方 {total_off:.6f} kg / 自建 {total_new:.6f} kg（差 {d_total:.3f}%）")
    check(d_total <= 3.0, "总质量误差 ≤3%", f"实测 {d_total:.3f}%")
    check(worst_mass <= 5.0, "逐 body 质量误差 ≤5%", f"最差 {worst_mass:.3f}%")
    check(worst_ipos <= 2.0, "逐 body 质心误差 ≤2mm", f"最差 {worst_ipos:.3f}mm")
    check(worst_inertia <= 15.0, "逐 body 对角惯量误差 ≤15%（代理假设阈值）",
          f"最差 {worst_inertia:.3f}%")

    print("== C. STAND keyframe 贴地 ==")
    for label, m in (("官方", off), ("自建", new)):
        d = mujoco.MjData(m)
        mujoco.mj_resetDataKeyframe(m, d, mujoco.mj_name2id(m, mujoco.mjtObj.mjOBJ_KEY, "STAND"))
        mujoco.mj_forward(m, d)
        foot_ids = [g for g in range(m.ngeom)
                    if (mujoco.mj_id2name(m, mujoco.mjtObj.mjOBJ_GEOM, g) or "").endswith("foot_collision")]
        zs = []
        for g in foot_ids:
            center = d.geom_xpos[g]
            rot = d.geom_xmat[g].reshape(3, 3)
            half = m.geom_size[g]
            for sx in (-1, 1):
                for sy in (-1, 1):
                    for sz in (-1, 1):
                        zs.append((center + rot @ (half * np.array([sx, sy, sz])))[2])
        print(f"  [{label}] STAND 脚掌最低点 z={min(zs):+.5f}")
        if label == "自建":
            check(abs(min(zs)) <= 0.005, "STAND 脚掌贴地 |z| ≤5mm", f"实测 {min(zs):+.5f}")

    print("== D. D26 穿透（阈值：稳定后最低碰撞角点 z ≥ -1mm） ==")
    z_new_drop = drop_test(HANDMADE_SCENE, "自建")
    z_off_drop = drop_test(OFFICIAL_SCENE, "官方对照")
    z_new_push = push_test(HANDMADE_SCENE, "自建")
    z_off_push = push_test(OFFICIAL_SCENE, "官方对照")
    check(z_new_drop >= -0.001, "自建·落体不穿透", f"z={z_new_drop:+.4f}")
    check(z_new_push >= -0.001, "自建·推倒不穿透", f"z={z_new_push:+.4f}")
    print(f"  对照组记录：官方落体 z={z_off_drop:+.4f} / 官方推倒 z={z_off_push:+.4f}"
          "（D26 原状：官方网格接触在大冲击下可穿透至 z≈-0.111）")

    print()
    if failures:
        print(f"M9a Bite 1 验收 FAIL（{len(failures)} 项）: {failures}")
        sys.exit(1)
    print("M9a Bite 1 验收 PASS：结构 / 惯量对账 / 贴地 / 不穿透 全绿")


if __name__ == "__main__":
    main()

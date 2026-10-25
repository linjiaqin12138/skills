"""项目内引导练习「手搓一条 2 关节腿链」验收。

用法（仓库根）：
  python exercises/mjcf-leg-chain/accept.py               # 验收作业 exercise_leg.xml
  python exercises/mjcf-leg-chain/accept.py --solution    # 验收参考解答
  python exercises/mjcf-leg-chain/accept.py path/to.xml   # 验收任意文件

断言组：
  A. 结构：编译通过；2 个动力 body（thigh/shank）+ 可选 1 个静态挂载 body；
     njnt=2 全 hinge（轴 0 1 0，矢状面）；nu=2 且 ctrl 序 = hip 后 knee；
     碰撞 geom 全 box；有地板；有 STRAIGHT keyframe；knee 真的限位，且限位区间
     覆盖 B 组目标角
  B. 执行器跟踪：STRAIGHT 起 ctrl=[0.5, -0.5] rad，step 2s，
     两关节角到达目标 ±0.1 rad
  C. STRAIGHT keyframe：扰动 qpos 后 mj_resetDataKeyframe 回存，逐位一致

阈值来源（全是代理假设，理由写在断言旁）：
  - ±0.1 rad / 2s：kp 取个位数时稳态误差 ≈ 重力矩/kp ≈ 0.01/5 = 0.002 rad，
    收敛时间常数 << 1s；阈值宽松到只抓「执行器没接上 / 方向反了 / 力矩被
    forcerange 掐死」，不惩罚任何合理的参数选择。
"""
import os
import sys

import mujoco
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
HOMEWORK_XML = os.path.join(HERE, "exercise_leg.xml")
SOLUTION_XML = os.path.join(HERE, "exercise_leg_solution.xml")

JOINT_NAMES = ["hip_pitch", "knee"]      # ctrl 序与关节序都应是这个
DYNAMIC_BODIES = ["thigh", "shank"]
TARGETS = {"hip_pitch": 0.5, "knee": -0.5}  # B 组目标角（rad）
TRACK_TOL = 0.1                            # rad，见 docstring 阈值来源
TRACK_SECONDS = 2.0

failures = []


def check(ok, label, detail="", hint=""):
    print(f"  [{'PASS' if ok else 'FAIL'}] {label}" + (f"  {detail}" if detail else ""))
    if not ok:
        if hint:
            print(f"       → 怎么办: {hint}")
        failures.append(label)


def parse_args(argv):
    if "--solution" in argv:
        return SOLUTION_XML, "参考解答"
    for a in argv:
        if not a.startswith("-"):
            return a, f"作业（{a}）"
    return HOMEWORK_XML, "作业"


def joint_id(model, name):
    return mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_JOINT, name)


def main():
    xml_path, who = parse_args(sys.argv[1:])

    if not os.path.isfile(xml_path):
        print(f"找不到文件: {xml_path}")
        if xml_path == HOMEWORK_XML:
            print("还没创建作业文件——任务说明见教程 book/m09a-handmade-mjcf/")
            print("（动手练习：手搓一条 2 关节腿链）。从零手写，别复制参考解答；")
            print("卡住可以对照 exercises/mjcf-leg-chain/exercise_leg_solution.xml 找差距。")
        sys.exit(2)

    try:
        model = mujoco.MjModel.from_xml_path(xml_path)
    except Exception as e:
        print(f"XML 编译失败: {xml_path}\n{e}")
        print("→ 怎么办: 按报错行号回去改 XML；常见坑是标签没闭合、属性名拼错、")
        print("  或 MuJoCo 3.x 下 joint 写了 range 却忘了 autolimits/limited。")
        sys.exit(1)

    print(f"验收对象: {who}  {xml_path}")
    print("== A. 结构 ==")

    # nbody 计数：worldbody 算 1；thigh/shank 各算 1；静态挂载 body（无关节）
    # 若存在也算 1。所以 nbody 合法值是 3（腿直接挂 worldbody）或 4（经挂载点）。
    dynamic = [b for b in range(1, model.nbody) if model.body_jntnum[b] > 0]
    dynamic_names = sorted(
        mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_BODY, b) for b in dynamic)
    check(dynamic_names == sorted(DYNAMIC_BODIES),
          "动力 body = thigh + shank（带关节的 body 恰好 2 个）",
          f"实际: {dynamic_names}",
          "检查 <body> 嵌套：腿应只有大腿、小腿两节各带一个 <joint>；"
          "多出来通常是给挂载点也写了关节（挂载点应是静态 body，不写关节），"
          "少了通常是 body 名拼错或关节写到了 geom 行里。")
    check(model.nbody in (3, 4),
          "nbody = 3 或 4（worldbody + thigh + shank + 可选静态挂载点）",
          f"实际 nbody={model.nbody}",
          "worldbody 固定算 1 个 body；每个 <body> 标签各算 1 个，无关节的"
          "静态挂载 body 也计入。数不对就回去数 <body> 标签。")
    extra_static_ok = all(
        model.body_jntnum[b] == 0
        for b in range(1, model.nbody) if b not in dynamic)
    check(extra_static_ok, "除 thigh/shank 外的 body 全是静态（无关节）",
          hint="挂载点要固定在空中：body 里别写 <joint>，靠父体（worldbody）"
               "天然静止来固定。")

    check(model.njnt == 2, "njnt=2", f"实际 njnt={model.njnt}",
          hint="检查 <joint> 声明个数与位置：thigh 里一个 hip_pitch，"
               "shank 里一个 knee，共两个。")
    jids = {name: joint_id(model, name) for name in JOINT_NAMES}
    check(all(j >= 0 for j in jids.values()),
          "关节名 hip_pitch / knee 存在",
          f"实际关节: {[mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_JOINT, j) for j in range(model.njnt)]}",
          hint="name 属性逐字母对照：hip_pitch、knee。")
    if all(j >= 0 for j in jids.values()):
        check(all(model.jnt_type[j] == mujoco.mjtJoint.mjJNT_HINGE for j in jids.values()),
              "两关节全 hinge",
              hint="type=\"hinge\"；练习只要单自由度转动，别用 ball/free。")
        axis_ok = all(np.allclose(model.jnt_axis[j], [0, 1, 0]) for j in jids.values())
        check(axis_ok, "关节轴 0 1 0（矢状面内摆动）",
              hint="axis=\"0 1 0\"：绕 y 轴转，腿在 x-z 平面（矢状面）内前后摆。")
        thigh_id = mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_BODY, "thigh")
        shank_id = mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_BODY, "shank")
        if thigh_id >= 0 and shank_id >= 0:
            check(model.body_parentid[shank_id] == thigh_id,
                  "shank 嵌套在 thigh 里（knee 连小腿与大腿）",
                  hint="<body name=\"shank\"> 要写在 <body name=\"thigh\"> 的内部，"
                       "不是平级——嵌套层级就是身体的连接关系。")
        # 两条分工：下面这条查「有没有限位」，再下面一条查「限位区间够不够走到目标角」。
        # 只查覆盖会把「压根没限位」判成 PASS（covered 语义是 未限位 or 覆盖），所以缺一条独立的存在性断言。
        check(bool(model.jnt_limited[jids["knee"]]),
              "knee 带 range 限位（jnt_limited 为真）",
              f"limited={bool(model.jnt_limited[jids['knee']])}",
              hint="knee 少了 range 限位：在 <joint name=\"knee\" ...> 上加 "
                   "range=\"-1.4 0.05\"。默认按度解析，要按弧度就在文件头的 "
                   "<compiler angle=\"radian\"> 里声明。")
        for name, target in TARGETS.items():
            j = jids[name]
            limited = model.jnt_limited[j]
            covered = (not limited) or (model.jnt_range[j][0] <= target <= model.jnt_range[j][1])
            check(covered, f"{name} 限位覆盖跟踪目标 {target} rad",
                  f"limited={bool(limited)} range={model.jnt_range[j].tolist()}",
                  hint=f"{name} 限位收得太窄，够不到 {target} rad——B 组「走到目标角」"
                       "本身就悬了：限位是软约束，力矩够大时会被压穿，看着到位其实是"
                       "执行器硬顶过去的。把 range 下界放开（参考解答取 -1.4）。")

    check(model.nu == 2, "nu=2", f"实际 nu={model.nu}",
          hint="<actuator> 段写两个 <position>，一个管 hip_pitch 一个管 knee。")
    if model.nu == 2:
        trn_ok = all(model.actuator_trntype[a] == mujoco.mjtTrn.mjTRN_JOINT
                     for a in range(model.nu))
        check(trn_ok, "执行器全挂在 joint 上（<position joint=\"...\">）",
              hint="用 <position joint=\"关节名\">，不要用 motor 或 tendon 传动。")
        if trn_ok:
            act_joint_names = [
                mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_JOINT,
                                  model.actuator_trnid[a, 0])
                for a in range(model.nu)]
            check(act_joint_names == JOINT_NAMES,
                  "ctrl 序 = hip 后 knee（ctrl[0] 管 hip_pitch，ctrl[1] 管 knee）",
                  f"实际: {act_joint_names}",
                  hint="<actuator> 里声明的顺序就是 ctrl 数组的下标顺序："
                       "把 hip 的 position 写在 knee 前面。")
    model_joint_order = [
        mujoco.mj_id2name(model, mujoco.mjtObj.mjOBJ_JOINT, j) for j in range(model.njnt)]
    check(model_joint_order == JOINT_NAMES,
          "关节文档序 = hip 后 knee（qpos[0]=hip_pitch，qpos[1]=knee）",
          f"实际: {model_joint_order}",
          hint="XML 里 body 的嵌套/书写顺序决定 qpos 顺序：thigh（含 hip_pitch）"
               "写在 shank（含 knee）外面/前面。")

    coll_types = {int(model.geom_type[g]) for g in range(model.ngeom)
                  if model.geom_bodyid[g] != 0
                  and (model.geom_contype[g] != 0 or model.geom_conaffinity[g] != 0)}
    check(coll_types == {int(mujoco.mjtGeom.mjGEOM_BOX)},
          "碰撞 geom 全 box（腿身几何即真实形状，惯量交给引擎算）",
          f"实际类型: {sorted(coll_types)}",
          hint="每个动力 body 一个 <geom type=\"box\">；本练习不用 mesh，"
               "也不用写 <inertial>——box 就是真实形状，引擎会从 density 算惯量。")
    has_floor = any(model.geom_bodyid[g] == 0
                    and model.geom_type[g] == mujoco.mjtGeom.mjGEOM_PLANE
                    for g in range(model.ngeom))
    check(has_floor, "存在地板（worldbody 下的 plane）",
          hint="<worldbody> 里加 <geom type=\"plane\" size=\"2 2 0.1\"/>。")

    key_id = mujoco.mj_name2id(model, mujoco.mjtObj.mjOBJ_KEY, "STRAIGHT")
    check(key_id >= 0, "存在 keyframe「STRAIGHT」",
          hint="<keyframe><key name=\"STRAIGHT\" qpos=\"0 0\"/></keyframe>，"
               "qpos 个数 = 关节数。")

    print("== B. 执行器跟踪（STRAIGHT → ctrl=[0.5, -0.5]，2s 内到 ±0.1 rad） ==")
    if key_id < 0 or model.nu != 2:
        check(False, "跟踪实验可运行", hint="先修 A 组：缺 STRAIGHT keyframe 或执行器数不对。")
    else:
        data = mujoco.MjData(model)
        mujoco.mj_resetDataKeyframe(model, data, key_id)
        data.ctrl[:] = [TARGETS[name] for name in JOINT_NAMES]
        for _ in range(int(TRACK_SECONDS / model.opt.timestep)):
            mujoco.mj_step(model, data)
        check(bool(np.all(np.isfinite(data.qpos))), "2s 后数值未发散（qpos 有限）",
              f"qpos={data.qpos.tolist()}",
              hint="发散了通常是 kp 太大 + timestep 不配比，或惯量写了离谱的小值；"
                   "练习里不写 <inertial> 就碰不到后者。")
        for name in JOINT_NAMES:
            j = jids[name]
            got = data.qpos[model.jnt_qposadr[j]]
            err = abs(got - TARGETS[name])
            check(err <= TRACK_TOL, f"{name} 跟踪到 {TARGETS[name]:+.1f} ±0.1 rad",
                  f"实测 {got:+.4f}（误差 {err:.4f}）",
                  hint="到不了位按序排查：① 执行器 joint=\"...\" 拼写（没挂上则该关节"
                       "只受重力）；② kp 是否太小（个位数量级）；③ forcerange 是否把"
                       "力矩掐死在重力矩（~0.01 N·m）以下；④ 关节 damping 是否大到僵住。")

    print("== C. STRAIGHT keyframe 逐位还原 ==")
    if key_id < 0:
        check(False, "keyframe 还原", hint="先在 <keyframe> 里声明 STRAIGHT。")
    else:
        data = mujoco.MjData(model)
        mujoco.mj_resetDataKeyframe(model, data, key_id)
        saved = data.qpos.copy()
        data.qpos[:] += 0.37  # 任意扰动，验证还原与当前状态无关
        data.qvel[:] = 0.11
        mujoco.mj_resetDataKeyframe(model, data, key_id)
        check(bool(np.array_equal(data.qpos, saved)),
              "扰动后 resetDataKeyframe 回 STRAIGHT，qpos 逐位一致",
              f"存档 {saved.tolist()} / 还原后 {data.qpos.tolist()}",
              hint="检查 <key name=\"STRAIGHT\" qpos=\"0 0\"/>：qpos 个数必须等于"
                   "关节数（2 个），多写少写都会错位。")

    print()
    if failures:
        print(f"腿链练习验收 FAIL（{len(failures)} 项）: {failures}")
        sys.exit(1)
    print(f"腿链练习验收 PASS（{who}）：结构 / 跟踪 / keyframe 全绿")


if __name__ == "__main__":
    main()

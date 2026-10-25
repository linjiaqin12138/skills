"""M9a Bite 3 A/B 组：BAM 参数对账与可选动作延迟。

在容器内由 accept/m09a-b03.sh 调用，不启动 daemon。
"""
import os
import sys

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "impl", "sim"))
import duck_body


failures = []


def check(ok, label, detail=""):
    print(f"  [{'PASS' if ok else 'FAIL'}] {label}" + (f"  {detail}" if detail else ""))
    if not ok:
        failures.append(label)


def main():
    scene = os.path.join(os.path.dirname(__file__), "..", "impl", "sim", "assets", "scene_handmade.xml")

    print("== A. BAM 参数对账 ==")
    check(duck_body.BAM_MOTOR_NAME == "xl330", "motor_name=xl330")
    check(duck_body.BAM_MODEL == "m6", "model=m6")
    check(duck_body.BAM_KP_FW == 200.0, "kp_fw=200", f"实际 {duck_body.BAM_KP_FW}")
    check(duck_body.BAM_VIN_RANGE == (6.5, 8.2), "训练 vin_range=(6.5, 8.2)")
    check(duck_body.BAM_VIN_DROP_GAIN_RANGE == (0.0, 0.2),
          "训练 vin_drop_gain_range=(0.0, 0.2)")
    check(duck_body.BAM_DELAY_RANGE == (3, 6), "训练 delay lag range=(3, 6) 控制拍")
    check(duck_body.BAM_VIN_MIN == 6.0, "vin_min=6.0")
    check(duck_body.BAM_MAX_CURRENT is None, "训练/回放默认无电流限制")
    check(duck_body.BAM_TIMESTEP == 0.005, "BAM 物理步长 0.005s")

    body = duck_body.DuckBody(scene, use_bam=True)
    check(body.bam_model.actuator.kp == 200.0, "默认实例 kp=200")
    check(body.bam_model.actuator.vin == 7.4, "CPU 回放默认 vin=7.4")
    check(body.bam_model.actuator.max_current is None, "默认实例 max_current=None")
    check(body.bam_ctrl.vin_drop_gain == 0.1, "CPU 回放默认 vin_drop_gain=0.1")
    check(body.bam_ctrl.vin_min == 6.0, "实例 vin_min=6.0")
    check(body.model.opt.timestep == 0.005 and body.substeps == 4,
          "20ms 控制拍 = 4 个 BAM 子步")
    check(body.delay_lag == 0, "CPU 回放默认 delay=0（对齐 infer_policy）")
    foot_geoms = {}
    for side in ("left", "right"):
        ids = [g for g in range(body.model.ngeom)
               if (duck_body.mujoco.mj_id2name(body.model, duck_body.mujoco.mjtObj.mjOBJ_GEOM, g) or "")
               .startswith(f"{side}_foot_collision")]
        foot_geoms[side] = ids
        check(len(ids) >= 2, f"{side} 脚掌碰撞拆成至少 2 个 primitive", f"实际 {len(ids)}")
        check(all(body.model.geom_type[g] == duck_body.mujoco.mjtGeom.mjGEOM_BOX for g in ids),
              f"{side} 脚掌碰撞仍全 box，不用 mesh")
        friction = [body.model.geom_friction[g, 0] for g in ids]
        check(np.allclose(friction, [1.0] * len(ids)),
              f"{side} 脚掌滑动摩擦回到官方 CPU 值 1.0", f"实际 {friction}")
    official_scene = os.path.join(os.path.dirname(__file__), "..", "impl", "sim", "assets",
                                  "scene_groundcontact.xml")
    official = duck_body.mujoco.MjModel.from_xml_path(official_scene)
    official_friction = [official.geom(name).friction[0]
                         for name in ("left_foot_collision", "right_foot_collision")]
    check(np.allclose(official_friction, [1.0, 1.0]),
          "官方场景足底摩擦保持 1.0", f"实际 {official_friction}")

    print("== B. 显式实验参数与延迟队列 ==")
    custom = duck_body.DuckBody(
        scene, use_bam=True, kp_fw=120.0, vin=6.8, vin_drop_gain=0.17, delay_lag=3,
    )
    check(custom.bam_model.actuator.kp == 120.0, "kp_fw 可显式覆盖", "实际 %.1f" % custom.bam_model.actuator.kp)
    check(custom.bam_model.actuator.vin == 6.8, "vin 可显式覆盖")
    check(custom.bam_ctrl.vin_drop_gain == 0.17, "vin_drop_gain 可显式覆盖")
    expected_force = 6.8 * custom.bam_model.kt.value / custom.bam_model.R.value
    force_ok = np.allclose(custom.model.actuator_forcerange,
                           np.tile([-expected_force, expected_force], (custom.model.nu, 1)))
    check(force_ok, "forcerange 随 vin 重算", f"±{expected_force:.6f} N·m")
    check(custom.delay_lag == 3, "delay=3 控制拍已启用")

    stand = custom.bam_ctrl.q_target.copy()
    targets = [stand + (i + 1) * 0.01 for i in range(4)]
    early = []
    for target in targets[:3]:
        err = custom.write_targets(custom.bus_order(target))
        early.append(err is None and np.allclose(custom.bam_ctrl.q_target, stand))
    check(all(early), "前 3 拍仍执行 STAND，不提前使用新目标")
    err = custom.write_targets(custom.bus_order(targets[3]))
    check(err is None and np.allclose(custom.bam_ctrl.q_target, targets[0]),
          "第 4 拍执行第 1 拍目标（lag=3，15 槽总线契约不变）")

    if failures:
        print(f"A/B 参数验收 FAIL: {failures}")
        raise SystemExit(1)
    print("A/B 参数验收 PASS：BAM 对账 / 参数覆盖 / 延迟队列全绿")


if __name__ == "__main__":
    main()

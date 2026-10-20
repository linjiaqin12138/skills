# M9a Bite 1 验收档案：自建 MJCF 身体模型（零 STL）

日期：2026-10-20

## 验收命令

```bash
bash scripts/accept-m9a-bite1.sh
# SIM_PYTHON 默认 ~/microduck-verify/.venv/bin/python；容器内 SIM_PYTHON=/opt/sim-venv/bin/python
```

## 实测输出（2026-10-20 宿主机，mujoco 3.15.0）

- A 结构：10/10 PASS——零 `<mesh>`；freejoint+14 hinge 同名同序；body 树/父子关系同构；nu=14 执行器同序；关节 range 全对齐；damping/frictionloss/armature = 0.053/0.0048/0.0018（chosen_actuator 编译值）；kp=0.55、forcerange=±0.96
- B 惯量对账：总质量 0.737243 kg 两侧逐位一致；逐 body 质量最差 0.005%、质心最差 0.001mm、对角惯量最差 0.041%（阈值 3%/5%/2mm/15%，显式 `<inertial>` 直写官方编译值，误差≈0 符合预期）
- C STAND 贴地：自建脚掌最低点 z=+0.00065，与官方同姿势逐位一致（碰撞盒=网格 AABB 的直接证据）
- D D26 穿透：自建落体 z=-0.0002、推倒 z=-0.0002（≥-1mm 阈值）；官方对照组同条件 z=-0.0137（13.7mm 穿透；M5 大冲击极端工况记录为 z≈-0.111）

结论：**PASS 全绿**（D26 收敛）。

## 涉及文件

| 文件 | 说明 |
|---|---|
| `sim/assets/duck_handmade.xml` | 自建身体 MJCF：15 body、14 hinge、14 position 执行器、10 碰撞盒 + 15 视觉盒，零 mesh |
| `sim/assets/scene_handmade.xml` | 场景包装：地板/灯光/imu_gyro/STAND keyframe |
| `sim/accept_m9a_bite1.py` | 验收核心（A–D 四组断言） |
| `scripts/accept-m9a-bite1.sh` | 验收入口（SIM_PYTHON 可换容器 venv） |
| `scripts/m9a_extract_official.py` | 官方模型数据提取一次性工具（惯量/AABB 数据来源） |
| `sim/assets/PROVENANCE.md` | 追加自建资产来源说明 |
| `docs/deviations.md` | D26 标记已收敛 |
| `assets/dst/handmade_stand.png` | 自建模型 STAND 渲染图（视觉抽查） |

## 与官方模型的残余差异（供偏差簿参考）

- 碰撞几何是官方碰撞网格的 AABB 包围盒，比真实网格**只大不小**；对行走/技能的影响在 Bite 2 闭环验收里实测判定
- 视觉外观为盒体拼接，无 STL 保真度（教学模型定位，永久保留）
- 官方 geom#7（trunk，contype=2/conaffinity=2）实际不与任何 geom 产生接触，自建模型未复制该无效 geom
- 未移植官方 RL 传感器组（orientation/angular-velocity/imu_lin_vel/imu_accel/root_angmom）——duck_body.py 只读 `imu_gyro`；训练侧需要时在 M9b+ 再加

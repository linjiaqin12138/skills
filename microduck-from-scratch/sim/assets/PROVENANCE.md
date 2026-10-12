# sim/assets 资产来源

- `robot_walk.xml` 与 `assets/*.stl`（43 个网格）来自
  [pollen-robotics/microduck_rl](https://github.com/pollen-robotics/microduck_rl)，
  路径 `src/mjlab_microduck/robot/microduck/`，稀疏克隆日期 2026-09-26。
- 许可证：**Apache License 2.0**（见该仓库根目录 `LICENSE`）。
- `robot_walk.xml` 逐字未改（`diff` 可验证）。`scene.xml` 是本教程自写的场景
  包装（地板/灯光/keyframe/一个 gyro 传感器），视觉设定照搬同仓库
  `scene_walk.xml`。
- `robot_groundcontact.xml`：同仓库同路径，逐字拷入日期 **2026-10-12**
  （SitStand / Roulade / GroundPick 训练身体；精选碰地 collision geom）。
  `scene_groundcontact.xml` 是本教程自写的场景包装（地板/灯光/`imu_gyro`/
  STAND keyframe；视觉照搬同仓库 `scene.xml`）。BAM 默认加载此场景。
- 旧版教程骨架（主仓库 kinematics 的无 geom 版本 + 自补碰撞盒）已被替换，
  历史见 `docs/deviations.md` D21 行与 `docs/milestones/m4-sim-walk/acceptance.md`。

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
- `duck_handmade.xml`（M9a Bite 1，2026-10-20）：本教程**从零手写**的鸭子身体
  MJCF——零 STL 引用，碰撞/视觉全为基本几何体 box。惯量（质量/质心/惯量主轴）
  取自官方 `robot_groundcontact.xml` 编译输出（Apache-2.0 数值数据），碰撞盒
  取官方 10 个碰撞网格的 AABB；关节树/关节参数/执行器参数与官方逐位对齐。
  数据提取工具：`scripts/m9a_extract_official.py`。目的：摆脱 `assets/*.stl`
  的 CC BY-NC-SA 许可，并收敛 D26（mesh 接触穿透）。
- `scene_handmade.xml`：本教程自写的场景包装（地板/灯光/`imu_gyro`/STAND
  keyframe），组织方式对齐 `scene_groundcontact.xml`。

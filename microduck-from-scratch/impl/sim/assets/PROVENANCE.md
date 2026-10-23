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
- `duck_handmade.xml`（M9a Bite 1，2026-10-20；同日视觉修订）：本教程**从零手写**的鸭子身体
  MJCF。惯量（质量/质心/惯量主轴）取自官方 `robot_groundcontact.xml` 编译输出
  （Apache-2.0 数值数据），显式 `<inertial>` 写入；碰撞盒取官方 10 个碰撞网格的
  AABB（box，D26 收敛靠它）；关节树/关节参数/执行器参数与官方逐位对齐。
  **视觉 geom 为官方 STL 网格 + 材质**（geom 行逐字照抄 `robot_groundcontact.xml`，
  CC BY-NC-SA 4.0）——2026-10-20 用户裁决：盒体拼接外观太丑，视觉层改用官方
  STL；视觉 geom contype=0/conaffinity=0 不进物理，碰撞/惯量/关节树仍是自建
  零 STL 数据。物理核心（碰撞+惯量）不依赖 STL；视觉层与本仓已在用的
  `robot_walk.xml`/`robot_groundcontact.xml` 同级依赖同一批 STL。
  数据提取工具：`scripts/m9a_extract_official.py`。目的：收敛 D26（mesh 接触穿透）。
- `scene_handmade.xml`：本教程自写的场景包装（地板/灯光/`imu_gyro`/STAND
  keyframe），组织方式对齐 `scene_groundcontact.xml`。

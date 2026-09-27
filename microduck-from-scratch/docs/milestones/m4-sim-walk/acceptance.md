# M4 仿真行走 — 验收档案

## 交付物

- `sim/duck_body.py`：MuJoCo TCP 服务器（NDJSON，op 标签帧：hello/read/write/body）。read = 步进一个控制拍（20ms = 20×1ms 物理步）后回 15 关节位置/速度 + 躯干系 IMU（gyro 传感器 + 四元数解算 gravity）+ body_pos；write 收 15 维总线序目标（嘴位丢弃）。起步 STAND keyframe（= DEFAULT_POSITION），起步即挂力矩。关闭 MuJoCo autoreset（失稳不再静默清零时钟）。
- `sim/assets/robot_walk.xml`：原版资产复制件，补了躯干/足部碰撞盒（原文件无任何 geom）。
- `sim/assets/scene.xml`：地板 + 位置执行器（kp=8 kv=0.25）+ 关节默认（armature=0.001 damping=0.03 frictionloss=0.005）+ STAND keyframe + gyro 传感器。
- `src/io.rs`：`SimIo`（TCP，200ms 超时，TCP_NODELAY，任何错误丢连接由控制循环下一拍重连）+ `impl RobotIo for Box<dyn RobotIo>`。新增 3 个单测（传感帧解析、断线重连、拒绝即错误）。
- `src/main.rs`：`--sim host:port` 选 SimIo（否则 FakeIo）；新增 `robot.drive {vx, vyaw}`（第一个 mutating 调用，SO_PEERCRED 按 D8 留 M5/M7）。
- `src/control.rs`：`spawn` 增加 `SharedCommand` 参数，观测 command 块取真值（收敛 D18）。插值/低通/action_scale 行为未动。
- `src/bin/mini-duckctl.rs`：`drive <vx> <vyaw> [--secs N]`（带 --secs 时到时自动发零命令停车）。
- `scripts/accept-m4.sh`：一键验收（先杀残留），在容器内跑全流程。

## 验收实测

`docker compose exec rust cargo build && docker compose exec rust cargo test`：

```
Finished `dev` profile [unoptimized + debuginfo] target(s)
test result: ok. 18 passed; 0 failed; 0 ignored
```

`docker compose exec rust bash scripts/accept-m4.sh`（两遍）：

```
站稳后 body(x y z t): 0.1006 -0.0259 0.1256 5.0     （z≈0.12 站立，5s 内漂移 0.1m）
drive 后 body(x y z): 1.5528 0.4031 0.1295
== 结果：10 秒前进位移 = 1.452 m（门槛 0.5m）==  PASS

第二遍：位移 = 1.427 m  PASS
```

`--sim` 与 FakeIo 下 health/subscribe 输出一致（healthy:true，subscribed:true）。

## 复跑（2026-09-26，模型换血为 microduck_rl 官方训练资产后）

- `sim/assets/robot_walk.xml` + `sim/assets/assets/`（43 个 STL）来自 microduck_rl
  `src/mjlab_microduck/robot/microduck/`（克隆 2026-09-26，Apache-2.0，
  来源记录见 `sim/assets/PROVENANCE.md`）。真实惯量/真实逐关节阻尼/足部 mesh 碰撞/
  真实外形（头壳、橙脚、舵机可见）。教程自补的盒子 geom 与拍脑袋的关节 default 已删。
- 对 RL 原文件的唯一改动：`chosen_actuator` 类 `<position>` 的 kp 0.55→8、
  kv 0→0.25（原因见该处注释与 D21：kp=0.55 静态保持需 0.45 rad 下垂，站不住；
  训练真实执行器是 BAM kp_fw=200，XML PD 只是降级路径）。inertial/joint range 未动。
- 脚-地接触：训练侧由 mjlab `FULL_COLLISION` CollisionCfg 在 Python 里开
  （foot_collision 网格 condim=3/priority=1/friction=1.0）；我们不改 RL 文件，
  在 scene.xml 用两个显式 `<pair geom1="floor">` 达到同一件事。
- `duck_body.py` 新增 `--view-port` MJPEG 网页画面（osmesa 离屏渲染 + Pillow，
  跟踪相机锁躯干）。实测帧：`/tmp/duck_real.jpg`（真鸭子外形确认）。
- `cargo test`：18 passed, 0 failed。
- `scripts/accept-m4.sh`（真实模型复跑）：

```
站稳后 body(x y z t): 0.0306 -0.0486 0.1187 5.0
drive 后 body(x y z): 0.9272 -0.1633 0.1230
== 结果：10 秒前进位移 = 0.897 m（门槛 0.5m）==  PASS
```

步速 0.09 m/s 比盒子模型的 0.15 慢（真实碰撞+阻尼+forcerange ±0.96Nm 的代价），
仍高于门槛。

1. **robot_walk.xml 无 geom 无执行器**：它是给训练管线包外层场景用的（带 geom 的版本在 microduck_rl，未随本仓库发布）。碰撞盒为教程近似，记 D21。
2. **数值失稳**：关节惯量 1e-5 量级，kp=8 裸位置伺服 ω·dt 越出积分器稳定域 → QACC NaN。加 armature=0.001（XL330 经 288:1 减速箱的折算转子惯量量级，不是纯调参）后稳定。
3. **MuJoCo autoreset 默认开**：失稳后状态清零、时钟归零，调试时表现为"时间倒流"。`mjDSBL_AUTORESET` 关掉。
4. **执行器阻尼扫参**：damping=0.1 冻住不动（过阻尼），0.02 起步后摔倒，0.03 + kv=0.25 走出 0.15 m/s ≈ 指令速度。
5. 容器内无 pip：`get-pip.py --break-system-packages` 后装 mujoco / onnxruntime（调试复刻控制环用）。

## 偏差变动

- D18 收敛（robot.drive → 共享 Command → obs command 块）。
- D4 正式引入（帧细节自定，M8 对齐）。
- D21 新增（执行器/碰撞几何近似，实测达标）。

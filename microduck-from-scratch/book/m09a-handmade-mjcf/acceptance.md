# M9a 验收档案：自建 MJCF 与 BAM 闭环

日期：2026-10-20（初版同日，视觉修订同日）

## 验收命令

```bash
docker compose -f impl/docker-compose.yml exec rust bash /work/accept/m09a-b01.sh
# 脚本自带解释器探测：容器内 /opt/sim-venv/bin/python，宿主机回落仓内 .venv/bin/python
```

## 实测输出（2026-10-20 宿主机，mujoco 3.16.0，视觉修订后复跑）

- A 结构：12/12 PASS——碰撞 geom 全 box（编译后枚举接触几何类型）；视觉网格 geom=70（官方 STL 照抄）且位姿与官方编译值逐位对账（最差 0.00e+00）；freejoint+14 hinge 同名同序；body 树/父子关系同构；nu=14 执行器同序；关节 range 全对齐；damping/frictionloss/armature = 0.053/0.0048/0.0018（chosen_actuator 编译值）；kp=0.55、forcerange=±0.96
- B 惯量对账：总质量 0.737243 kg 两侧逐位一致；逐 body 质量最差 0.005%、质心最差 0.001mm、对角惯量最差 0.041%（阈值 3%/5%/2mm/15%，显式 `<inertial>` 直写官方编译值，误差≈0 符合预期）
- C STAND 贴地：自建脚掌最低点 z=+0.00065，与官方同姿势逐位一致（碰撞盒=网格 AABB 的直接证据）
- D D26 穿透：自建落体 z=-0.0002、推倒 z=-0.0002（≥-1mm 阈值）；官方对照组同条件 z=-0.0137（13.7mm 穿透；M5 大冲击极端工况记录为 z≈-0.111）

视觉修订（同日，用户裁决：盒体视觉太丑→改挂官方 STL）后 B/C/D 数值与初版逐位一致——视觉 geom contype/conaffinity=0 不进物理。A 组首条断言相应从「XML 文本零 `<mesh>`」改为「编译后碰撞 geom 全 box + 视觉网格=70」。

结论：**PASS 全绿**（D26 收敛）。

## 涉及文件

| 文件 | 说明 |
|---|---|
| `impl/sim/assets/duck_handmade.xml` | 自建身体 MJCF：15 body、14 hinge、14 position 执行器、10 碰撞盒（自建 AABB）+ 70 视觉网格 geom（官方 STL 照抄） |
| `impl/sim/assets/scene_handmade.xml` | 场景包装：地板/灯光/imu_gyro/STAND keyframe |
| `impl/sim/accept_m9a_bite1.py` | 验收核心（A–D 四组断言） |
| `accept/m09a-b01.sh` | 验收入口（SIM_PYTHON 可换容器 venv） |
| `impl/scripts/m9a_extract_official.py` | 官方模型数据提取一次性工具（惯量/AABB 数据来源） |
| `impl/sim/assets/PROVENANCE.md` | 资产来源台账（视觉 STL 的 CC BY-NC-SA 许可登记在此） |
| `spec/deviations.md` | D26 标记已收敛 |
| `figures/render_stand.py` + `figures/handmade_stand.png` | STAND 渲染脚本与产物（视觉抽查） |

## 与官方模型的残余差异（供偏差簿参考）

- 碰撞几何是官方碰撞网格的 primitive 近似，比真实网格**只大不小**；Bite 3 把每侧鞋底细分为薄承重底层 + 主体上层两个 box，在官方摩擦 1.0 下实测站立/直走，其他技能仍未覆盖
- 视觉挂官方 STL（CC BY-NC-SA 4.0）：用户裁决外观优先；视觉不进物理，物理核心零 STL 成果保留；商用分发需替换视觉资产
- 官方 geom#7（trunk，contype=2/conaffinity=2）实际不与任何 geom 产生接触，自建模型未复制该无效 geom
- 未移植官方 RL 传感器组（orientation/angular-velocity/imu_lin_vel/imu_accel/root_angmom）——duck_body.py 只读 `imu_gyro`；训练侧需要时在 M9b+ 再加

## Bite 3：BAM 对账与自建模型闭环（2026-10-25）

验收入口：`accept/m09a-b03.sh`；独立实测：`tutor/runs/m09a-b03.txt`。

- A/B：BAM 电机、模型、增益、电压、压降、电流限制与 5ms 步长对账通过；每侧脚掌为 2 个 box、滑动摩擦均回到官方 1.0；显式参数覆盖和 15 槽总线上的 lag=3 目标队列通过
- C：默认场景确认为 `scene_handmade.xml`；3 次全新进程均站立并以 vx=0.8 行走 10 秒，dx=2.604783/2.604783/2.605613m，最小值与中位数均为 2.605m
- 回归：102 Rust tests、`accept/m09a-b01.sh`、显式固定官方 `scene_groundcontact.xml` 后的 `accept/m08-b06.sh` 全绿
- D：handmade sit 两次全新进程全循环通过；坐姿 z=0.0644237，起身 z=0.1159861，均回 walk/fallen=false
- D70 修订后收敛：摩擦代理值 2.0 已移除，残余是鞋底 primitive 近似；不声称接触点逐点等价
- 覆盖边界：handmade roulade、连续坐姿窗口、非直走、横移、转向、扰动、随机种子与真机未验收；历史失败归因旧单 AABB 鞋底，当前 XML 修订后消失

结论：**PASS（用户 2026-10-25 确认）**。决定 `decisions/0001-handmade-bam-replay.md` 已采纳；教程见 `b03-bam-handmade.html`。

## Bite 4：D4 sim 协议面对齐（2026-10-25）

验收入口：`accept/m09a-b04.sh`；独立实测：`tutor/runs/m09a-b04.txt`。

- A–D：PROTOCOL=1 握手；线上 `gain`{kp}/`torque`{on}；旧名失败；15 槽 + trunk 重力；教学附加 `push`/`body` 仍可用
- E：102 Rust tests 全绿（含 `sim_io_set_gain_and_torque_frames`）
- F：BAM 默认 `scene_handmade` 冒烟，enable 后 `skill=walk` 且 `fallen=false`
- D4 收敛；`slow`/`tof` 不扩面；决定 `decisions/0002-sim-protocol-d4.md`（起草，等用户确认后改已采纳）
- 教程：`b04-sim-protocol.html`（第 21 章）

结论：**PASS（用户 2026-10-25 确认；反馈本 Bite 过简）**。决定 `0002` 已采纳。

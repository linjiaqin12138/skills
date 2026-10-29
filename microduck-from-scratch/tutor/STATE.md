# STATE — microduck 从零手搓教程断点

> 新会话续传入口：读本文件 + user-prefs.md（项目级路线与先修能力表）+ spec/deviations.md + questions.md + spec/features.md + book/ 最新一章，然后从"下一步"继续。若当前模式是先修学习，再读 `tutor/prerequisites/<slug>.md`。

- **原项目**：https://github.com/pollen-robotics/microduck （参考克隆在 `upstream/`，只读）
- **当前模式**：项目实现
- **返回位置**：无（当前在项目实现）
- **当前先修记录**：`tutor/prerequisites/ppo-reward-dr.md` — **出口已达**；返回摘要：PPO 三步、奖励权重=评分标准、DR 为健壮；误区：非选优淘汰、优势非开跑前算、投机形态不唯一
- **当前位置**：**M9f Bite 1 云冒烟已过；长训进行中（2026-10-29）**——`m9f_train.py` + handmade；补了 foot sites + IMU sensors；`tutor/runs/m09f-b01.txt` 有 `smoke_ok=1`；云上 `2048×1000` 后台跑（ETA≈25min）。`0011` 仍起草。
- **云机**：`ssh -p 23 root@117.50.221.204`；工作区 `/cloud/cloud-ssd1/microduck-m9f`；日志 `runs/m09f-train.log`。
- **M9e 已取消**。
- **下一步**：等长训结束 → 拉 checkpoint → verifier `accept/m09f-b01.sh` → 课文。
- **用户裁决（2026-10-29）**：M9e 取消；handmade + **自写训练脚本**。
- **安全**：会话中曾暴露 wandb API key——请轮换。
- **M9c 已收官（2026-10-29）**：Bite 1 FallPredictor（D23/D32）+ Bite 2 相对 odometry；决定 `0006`/`0007` 已采纳；档案 `book/m09c-fall-recovery/acceptance.md`。`robot.state` 仍无 odom（D50→M10）。
- **M9c Bite 2 已确认（2026-10-29）**：相对 odometry（局部双足 FK + ALPHA16）；`accept/m09c-b02.sh` 全绿；教程第 26 章；`0007` 已采纳。
- **M9c Bite 1 已确认（2026-10-26）**：FallPredictor + Limp/Posing（D23/D32 收敛）；默认 OFF，`MINIDUCK_LIMP_FALL=1`；教程第 25 章；`0006` 已采纳。
- **M9a Bite 1 已交付（2026-10-20）**：自建 MJCF 身体模型全绿，D26 收敛。M8/Phase 4 已收官（2026-10-15），复盘在 `tutor/phases/phase-4.md`。
- **M8 Bite 1–5 已交付**：C1 接口对齐 + C2 诊断。见 `book/m08-convergence/acceptance.md`
- **M8 Bite 6 已交付（2026-10-12）**：BAM M6 XL330 默认接入（`/opt/sim-venv` + `better-actuator-models`，场景 `scene_groundcontact`，timestep 0.005）；D21/D34 收敛；D35 部分收敛（结束后 upright，中段 fallen 残余）；新登记 **D68**（冻结 bringup → twist 上升沿 push 0.30 踢步态）。教程 `b06-bam.html`（第 14 章）
- **M8 Bite 8 已交付（2026-10-14，用户裁决 0.8 方案收尾）**，教程 `b08-walk-verdict.html`（第 15 章）已发布。行走验收为 vx=0.8 ×10s ≥2.0m；0.15/0.3 不走是公开 velstand 权重现状。BAM 步进在 write，读/写拍序递推等价（Q83）。D68 已收敛。
- **M8 Bite 9 已交付（2026-10-15）**：教程 `b09-phase4-fixes.html`（第 16 章），D13/D20/D24/D65 收敛；新登记 D69 挂 M10。
- **2026-10-20 补漏与练习**：第 18 章补搭建实操；第 19 章 `b02-leg-exercise.html` 已交付。练习 2026-10-25 迁到 `exercises/mjcf-leg-chain/`；用户报告已完成，仓内无作业文件，未复验。
- **2026-10-22 第 18 章读者三问补讲**（Q84–Q86）：惯量字段、joint 三属性、qpos 已补正文；广义坐标概念页未排期。
- **环境备注**：验收与数据提取容器内跑；viewer 用仓内 `.venv/`。compose 容器实际挂在项目名 `impl`（`miniduck-rust`）；`-p microduck-from-scratch` 有时找不到 running service。

## 里程碑进度

| 里程碑 | 状态 |
|---|---|
| M0 IPC 骨架 | ✅ 验收通过（book/m00-ipc-skeleton/，教程+验收档案） |
| M1 身体模型+FakeIo | ✅ 验收通过（book/m01-body-fakeio/，教程+验收档案） |
| M2 观测向量 61 维 | ✅ 验收通过（book/m02-obs-vector/，教程+验收档案） |
| M3 单策略推理（站立） | ✅ 验收通过（book/m03-stand-policy/，教程+验收档案） |
| M4 仿真行走（MuJoCo over TCP） | ✅ 验收通过（book/m04-sim-walk/，教程+验收档案） |
| M5 安全层（deadman/限位/跌倒） | ✅ 验收通过（book/m05-safety/，35 单测+accept-m5 六项+accept-m4 回归全绿） |
| M6 技能调度器（多策略优先级链） | ✅ 验收通过（book/m06-scheduler/，教程+验收档案；49 单测+accept-m6+accept-m5 回归全绿）。新：scheduler.rs（Cascade 纯状态机+Scheduler 持策略槽）、robot.do/skills/mouth/head、state 载荷 skill 字段、busy 抑制 Limp。权重：fetch-m6.sh 下 4 个（全 feedforward，D20 备注）。物理现实：ground_pick/kick 物理通过；sit/roulade 调度正确但物理摔倒（D34/D35，M8 随 D21 裁决） |
| M7 支线：迷你 updaterd | ✅ 验收通过（book/m07-updaterd/，教程+验收档案；60 单测（49 旧+11 新）+accept-m7 五场景 39 断言全绿）。新：impl/src/updater/ 十模块、bin/mini-updaterd、duckctl update 子命令组、accept/m07-b01.sh；miniduckd 零改动。新开 D37–D41，D6 移交 D37/D38 |
| M8 收敛验收 | ✅ 全部交付（2026-10-15）：Bite 1–6 + 8（C 线 + BAM + 行走裁决）+ Bite 9（D13/D20/D24/D65）+ Bite 10（D36）；教程第 9–17 章；Phase 4 复盘在 tutor/phases/phase-4.md |
| M9 仿真与训练（批次 2） | **进行中**：M9a/M9b/M9c 已收官。下一步 M9d（GPU）/ M9e |
| M10 驱动与硬件（批次 3） | 未开始（规划：真机驱动编程，硬件自购、可能非官方件；对齐 duck-control 硬件面。**此批完成后代码完全对齐原项目**） |
| M11 结构与外壳（批次 4） | 未开始（规划：自学 3D 建模与打印，与代码无关） |

## 关键路径与命令

- 手搓代码：`impl/`（`impl/Cargo.toml` / `impl/src/`）。`impl/src/lib.rs` 协议（Request.id 可缺省=notification；MoveParams、EnableParams、**SubscribeParams/SubscribeResult（M8 Bite 3）**）、`impl/src/main.rs` miniduckd（robot.enable{on,toggle}/move/health/**subscribe** + M6: do/skills/mouth/head；notification 统一入口；**robot.disable 已删（M8 Bite 2）**；**带 id 的 robot.state = METHOD_NOT_FOUND（M8 Bite 3）**）、`impl/src/safety.rs` Safety（唯一写句柄，M5）、`impl/src/fall.rs` FallPredictor+LandingDetector（M9c Bite 1）、`impl/src/control.rs` 50Hz 控制任务（**M9c 起六阶段 Held/RampUp/Driving/Limp/Posing/Stopped**；跌倒反射默认 OFF，`MINIDUCK_LIMP_FALL=1` 开启；普通 disable 直接回 home，恢复中 disable 抱持实测姿态；Driving 走 driving_tick）、`impl/src/scheduler.rs` 技能调度器（Cascade 纯状态机 + Scheduler 持 4 槽策略 + **reset() 清 LSTM 槽（M8 Bite 2）**）、`impl/src/policy.rs` 单网络推理（pub reset + reset_calls）、`impl/src/obs.rs` 61 维观测、`impl/src/io.rs` RobotIo(read/write/set_gain/set_torque/imu_ready)+FakeIo（write 精确回写，io.rs:340-344）+SimIo、`impl/src/model.rs` 关节常量+嘴映射、`impl/src/bin/mini-duckctl.rs` CLI（**enable [on|off]**/move/do/skills/mouth/head/state + update 子命令组；**disable 子命令已删**）、`impl/src/updater/` 十模块 + `impl/src/bin/mini-updaterd.rs`（socket /tmp/mini-updaterd.sock，env MINIDUCK_UPDATER_ROOT/GATE_MS/SOCK/EXIT_AFTER_SWAP；**M8 Bite 4 新增 `impl/src/updater/robot.rs` — updaterd 的 robotd 客户端 + apply 最前面的预检**）
- **M5 后行为变化**：daemon 启动不再自动站立（Held 抱持启动姿态），要 `mini-duckctl enable` 才使能；`move` 单次意图 500ms 后被 deadman 清零，CLI `--secs` 期间每 100ms 重发意图；socket 文件 0660
- **M6 后行为变化**：Driving 阶段嘴被 robot.mouth 意图覆写（默认 0 → −5°，其他阶段嘴跟随抱持/斜坡目标）；robot.do 只在 enabled 且 Driving 时接受；head 无 deadman
- **M9c Bite 1 后行为变化**：跌倒反射默认 OFF；显式开启后 busy/sitting 清预测去抖，不再直接由 fallen 触发 Limp。Limp/Posing 期间 homed=true、moving=true，完成后清策略反馈回 Driving
- **M8 Bite 2 后行为变化**：`enable off` 不再卸 torque——机器人直接回 home 保持上电抱持（Stopped），再 enable 立即回 Driving 无斜坡；全循环无 set_torque(false) 路径（卸力出口留给未来 robot.relax，D53）；任何方法的无 id 帧不应答（意图静默应用/其余静默丢弃）；robot.disable 彻底消失（METHOD_NOT_FOUND）
- **M8 Bite 3 后行为变化**：订阅只走带 id 的 `robot.subscribe`，ack 是启动时冻住的策略名单（`accepted` 恒 true，不回显 hz）；推送 method 仍是 `robot.state`、仍约 50Hz（发 `hz:10` 也不降频）；带 id 的 `robot.state` 得到 METHOD_NOT_FOUND，不会因此开始或停止推送；无 id 的 `robot.subscribe` 静默丢弃
- **M8 Bite 4 后行为变化**：`mini-duckctl update apply` 在做任何事之前先问 miniduckd「现在重启安全吗」「model_api 对不对」，不通过就 -32003 拒绝且磁盘与 current 都不动；安全判定 = 走路中（Driving 且 busy 或有 twist / 斜坡 / 瘫软）且没跌倒；model_api 必须是 2。注意 daemon 不可达时 safeToRestart 放行但 model_api 会被拒（D65 待裁决）
- M4 仿真体：`impl/sim/duck_body.py`（MuJoCo TCP+NDJSON，资产在 `impl/sim/assets/`；M5 新增 op：set_gain/set_torque/push，read 回显 gain/torque）。可视化两路：`--view-port 7802` MJPEG 网页（osmesa 离屏，只读，http://localhost:7802/ ）；`--viewer` MuJoCo 原生 viewer（glfw，容器里配 `impl/scripts/duck-vnc.sh` 的 Xvfb+x11vnc+noVNC，浏览器 http://localhost:6080/vnc.html 可鼠标交互推鸭子，详见 tutor/dev-container.md「交互式仿真」；compose 已映射 6080）。镜像含 mujoco+pillow+libosmesa6+xvfb+x11vnc+novnc+websockify，容器重建不丢
- 权重与 Runtime：`bash impl/scripts/fetch-m3.sh`（代理 `http://127.0.0.1:7890`）。产物 `impl/policies/velstand.onnx`、`impl/third_party/onnxruntime/`（均 gitignore，换机器要重跑 fetch-m3.sh + fetch-m6.sh）。容器环境变量 `ORT_DYLIB_PATH=/work/impl/third_party/onnxruntime/lib/libonnxruntime.so`
- 参考克隆：`upstream/`（只读）。浅克隆走同一代理（环境已迁回原生 Linux，代理在 127.0.0.1:7890；容器内 PyPI 直连可达）
- 构建/测试：在容器里做，见 `tutor/dev-container.md`。`docker compose -f impl/docker-compose.yml exec rust cargo build && docker compose -f impl/docker-compose.yml exec rust cargo test`。不要用 `bash -lc` 包 cargo，否则 PATH 会丢
- M4 验收：`docker compose -f impl/docker-compose.yml exec rust bash /work/accept/m04-b01.sh`。M5：`accept/m05-b01.sh`（sim 端口 7803）。M6：`accept/m06-b01.sh`（sim 端口 7804；权重先跑 `bash impl/scripts/fetch-m6.sh`）。M7：`accept/m07-b01.sh`（五场景 39 断言；根目录 /tmp/m7-demo 隔离，不需要仿真；**本机 D1/E1 有时序 flake，见下**）。M8：`accept/m08-b01.sh`（FakeIo 无需仿真；断言 a–w）+ `accept/m08-b04.sh`（Bite 4 updater 预检端到端，6 断言，根目录 /tmp/m8-updater-demo 隔离，不需要仿真）。两者都先 `cargo build --bins`：`cargo test` 不更新 `impl/target/debug/miniduckd`
- 最新架构图：`book/arch/src/current.json` + `book/arch/current.svg`（M9c Bite 2：新增 odometry.rs，control/FrameSnapshot 挂内部估计）；子图 `book/arch/odometry.svg` / `book/arch/fall-recovery.svg`；本课差分 `book/m09c-odometry/figures/arch-diff.svg`
- **M9c Bite 2 后行为变化**：控制循环持有相对里程计，新鲜且 IMU-ready 时更新；估计进内部 FrameSnapshot，RPC/`robot.state` 不加 odom；滑行/盲拍/disable/跌倒恢复均不清零估计
- 概念深读页（`book/concepts/`，`.draft.md` 为源稿、`.html` 为产物；**新页按 skill 现行规范走「一概念一文件夹」**：`book/concepts/<slug>/<slug>.draft.md` + `<slug>.html` + `assets/`）：`inertia-tensor/inertia-tensor.html`（2026-10-22 按 concept-writer 现行规范重写：教学合同 + 验收承诺开篇、8 面板学习链、4 张图（F1 哑铃手算 / F2 串轴 / F3 绕 z 旋转滑块交互 / F4 主轴与最终存法）、LaTeX 公式、5 题自测、4 个工程坑；冷读评审过一轮并按断点修订——图号补 F3、F2 箭头改双头不声明正负、观察任务③改「转到主轴系」、滑块矩阵加 x/y/z 表头、自测改覆盖三件事且选项不可抄正文。演示脚本 `book/concepts/figures/inertia-tensor/inertia_demo.py`；旧平铺文件 `concepts/inertia-tensor.{draft.md,html}` 已删，第 18 章链接改指新路径并整页重渲）、`quaternion.html`（2026-10-21 新增：wxyz/xyzw 顺序、半角、双覆盖 q≡-q、slerp vs nlerp、演示脚本 `figures/quaternion/quat_demo.py`；M1 背景卡片 + M9a 第 18 章惯量张量卡与坑 5 已挂链接）、`contact-penetration.html`、`servo-gain-torque.html`
- 仿真：默认 `/opt/sim-venv/bin/python sim/duck_body.py`（BAM + `scene_handmade.xml`，容器内，工作目录 `/work/impl`）；宿主机仅 viewer 用仓内 `.venv/bin/python impl/sim/duck_body.py --viewer`，`--no-bam` 回退 scene.xml。M9a 闭环验收 `accept/m09a-b03.sh`（端口 7809，3 次站立+行走 + 2 次 sit）；M8 官方物理回归 `accept/m08-b06.sh` 已显式固定 scene_groundcontact.xml。诊断：`accept/m08-b05.sh`（强制 no-bam）
- **当前仿真行为（M9a Bite 3 已验收）**：BAM 默认自建身体，固定 kp=200/vin=7.4/vin_drop=0.1/delay=0；可用 CLI 显式改参数和 20ms 拍延迟。每侧鞋底细分为两个 box，滑动摩擦回到官方 1.0，D70 已收敛。3 次 vx=0.8 行走均约 2.605m；2 次 handmade sit 全循环通过；handmade roulade 未验收。官方 groundcontact 的 sit/roulade/行走回归仍全绿。BAM read 不步进、write 后 step_tick，非 BAM 读时步进；vx=0.15/0.3 不迈步仍是上游 velstand 权重现状。
- Rust 工具链：容器 `miniduck-rust`，rustc 1.99.1（`rust:1-bookworm`，DaoCloud 镜像；2026-10-06 换机重建后从 1.98.1 升到 1.99.0；2026-10-20 为补 /opt/sim-venv 重建镜像，基础镜像更新到 1.99.1）。宿主机不装 rustup
- HF Hub 直连本机超时，下载走镜像或上述代理

## 环境迁移（2026-10-06 换电脑）

- 本机 docker 是 **rootless 模式**（context=rootless）：容器 uid 0 映射到宿主机当前用户。compose 已加 `user: "0:0"`（容器内以 root 跑，产物落回宿主机属 jqlin）；Dockerfile 的 dev 用户是 rootful 模式备用的。详见 tutor/dev-container.md
- 容器已重建（`docker compose -f impl/docker-compose.yml up -d --build`），权重/ORT 已重下（fetch-m3/m6），基线 81 单测全绿
- **accept-m7 已知 flake**：D1「没有 Busy」/ E1「pending」在本机间歇失败（新机器更快，场景 D 同版本 apply 在 sleep 1 窗口内瞬间 committed，Busy 窗口抓空；基线上同样复现，与代码改动无关）。Bite 2 期间实测一次全绿一次 FAIL=2。待办：D1 改轮询 Busy 窗口或同版本 apply 短路（非 M8 范围）

## 用户背景与偏好

见 `~/.clone-from-scratch-tutor/user-prefs.md`（用户级背景与学习路径偏好）+ `tutor/user-prefs.md`（项目级：整机闭环路线、硬件与 BAM 侦察结论、先修能力表）；流程规则已上收 SKILL.md。要点：里程碑代码由 coder subagent 构建、主 agent 复核验收；终态必须与原项目一致；路线为先修学习且未达出口时不启动对应 coder。

偏差簿：D1–D41 + D64–D68（D42–D63 为功能清单缺口号）。**Phase 4 裁决已落账（2026-10-15，见 spec/deviations.md 顶部裁决段）**：A 组核销（D1 等）；永久保留 D10/D27/D40/D43③；Bite 9 就地收敛中 D13/D20/D24/D65；挂 M9：D4/D23/D26/D28/D29/D30/D32/D33/D35 残余/「0.3 不走」上游权重（odometry 估计器已由 M9c Bite 2 交付，D50 字段仍挂 M10）；挂 M10：D5/D7/D12/D15（目标值改 31）/D25/D37–D41/D42–D63/D66/D67；挂 M11：D31。**D36 漏列待补问**。D2 豁免维持；padd/pad.* 不豁免改 M10 再评估（用户有 Xbox 手柄）。

## M6 复核更正（2026-09-28，教程写作时发现）

交接记录曾称「原版 --sim 是无物理回显（upstream/robotd/src/main.rs:240）」——**误读**：该注释挂在 `--fake` 上；原版 `--sim` 是真 MuJoCo（upstream/docs/design/simulation.md 声称 sitstand 在原版仿真里能起身并保持直立）。事实：原版仿真体 duck-body 在 microduck_rl 仓库、不在浅克隆里，我们的 `impl/sim/duck_body.py` 是手搓的这一半。这加强了 D34/D35 → D21（BAM 执行器动力学缺失）的归因。详见 book/m06-scheduler/b01-scheduler.md 第 4 节末。

## M5 侦察发现（2026-09-27，纠正了 M5 预案的两个错误前提）

1. **robotd 无 SO_PEERCRED**：原版 robotd 连接处理零凭据校验，socket 0660 文件权限就是全部鉴权（robotd/src/main.rs:60）；SO_PEERCRED+is_mutating 是 configd/updaterd 的模式，且 mutating 指"改软件/配置"，robot.move/enable 都不算。D8 按 0660 收敛，SO_PEERCRED 留给 M7。
2. **跌倒卸力不在 Safety 内部**：safety.rs 的 apply 明确无跌倒门——fallen 是纯报告，卸力是 robotd 层 LimpFall 状态机用普通 targets+gain 调 apply 完成的（原版 limp_fall 默认 OFF）。我们的跌倒响应照此放控制循环（D23 登记简化）。
3. 原版启动语义：Bringup=Limp，adopt_startup_pose 抱住读到的姿态，绝不 set_torque（舵机 RAM 的 torque 跨进程存活），显式 robot.enable 才动。M5 已对齐（D9 收敛）。

## 决策记录（用户裁决过的事）

1. 实现语言：**Rust**（跟随原版）
2. 硬件闭源不影响教程：验收走 FakeIo/仿真路线
3. 教程规则已更新：终态对齐原版（注释/文档/死代码不复刻），中间里程碑允许偏离但须登记收敛点
4. `impl/examples/` 是教程配套资产（概念示例代码），不是里程碑交付物，M8 收敛验收对照原版结构时不计入偏差
5. 教程硬标准：领域专有概念先大白话引入 + 1~2 段实测跑通的最小代码示例，再进项目实现
6. **M8 范围与后续路线（2026-10-04 用户裁决）**：M8 走 C 线——只对齐非硬件依赖内容（robot.move+vy / robot.enable{on,toggle} / robot.subscribe 入口 / updater safeToRestart 预检）+ 移植 BAM（Rhoban/bam 解析式模型，官方蓝图 microduck_rl scripts/infer_policy.py；先跑 kp=0.55 一行实验验证归因）让 sit/roulade 物理通过，D21/D34/D35 在 M8 收敛。**其余功能缺口（D42–D63 及偏差簿残余项）不做豁免收尾**，分配到后续批次：M9 仿真与训练、M10 驱动与硬件（此批完成后代码完全对齐原项目，D52 crate 边界/配置面等在此批或 M9 收敛）、M11 结构与外壳（无代码）。总目标=整机工程闭环，详见 tutor/user-prefs.md
7. **CLI disable 一并删除（2026-10-06 M8 Bite 2）**：daemon 删 robot.disable 是铁律 8 强制（原版没有）；用户裁决 CLI 也不留别名——关就是 `mini-duckctl enable off`，accept/m05-b01.sh A1c 等脚本跟着改
8. **disable 语义差异立即对齐（2026-10-06 M8 Bite 2）**：复核发现原版 disable 不卸 torque、无斜坡、直接命令回 home 保持上电（此前未登记，新登记 D64）。用户裁决不挂账、本 Bite 立即对齐——RampDown 退役、Stopped 上岗；代价登记在案：robot.relax 落地前卸 torque 无出口（D53 缺口待排期）
9. **D68 单独开 Bite 清掉（2026-10-13）**：用户裁决不挂到 M9，并选择策略在倒下之前进环。代码已做成 `--sim` 第一拍 Driving。站立 z≈0.117 通过。坐姿起身 50 拍后进 Limp（z≈0.042），Bite 7 未交付。没有写回冻结或注速度。D35 / D65 不动。
10. **起身交接另开 Bite 8（2026-10-14）**：1 秒窗口保持与原项目一致。下一 Bite 只对齐「先写目标、再步进」，用来让起身结束时身体进入 velstand 接得住的状态。不把 D68 装回。
11. **Bite 8 行走验收按 0.8 方案收尾（2026-10-14）**：用户肉眼验证官方窗口（0.3 不走、P 键注速度能走、打补丁 0.8 能走）+ 我们鸭子同条件复现（0.8 能走）后裁决：行走验收改 vx=0.8 ×10s 前移 ≥2.0m（实测 4.9m），验收目标=集成栈能驱动步态；0.3 不走登记为上游 velstand 权重现状（非我方偏差，不进豁免清单），M9 自训策略时真正解决。D68 收敛。
12. **Phase 4 版本基准（2026-10-15）**：reference 以磁盘克隆 daemon-v0.14.1（API_VERSION=31，origin/main HEAD，commit 344925c）为终态对齐基准；清单原按 API 37 检出写成，行号证据回填修正，API 32–37 差量留 M10 终态盘点。
13. **Phase 4 就地对齐组（2026-10-15，M8 Bite 9）**：D13 IoError 分类、D24 COAST_TICKS=3、D65 model_api None 放行、D20 删 LSTM 死代码。D20 前提由用户确认后查证：HF pollen-robotics/microduck-policies 全部 tag（v1–v8+main）只有同 10 个 onnx，全量探针验证均 1-in/1-out feedforward——原版 LSTM 分支从未被自家模型触发，故删；M9 自训若用 recurrent 再写回。
14. **Phase 4 批次挂账（2026-10-15）**：D23/D32（FallPredictor）挂 M9；D31 音频族**不豁免**挂 M11 再评估；odometry 从建议豁免改挂 M9；padd/pad.* 不豁免挂 M10 再评估（用户有 Xbox 手柄）；account.* 并入 D60；D61 扩为协议级错误码；D10/D27/D40/D43③ 永久保留（理由在 spec/deviations.md 各行）。
15. **M9 算力路线（2026-10-16）**：主力云 GPU（Vast.ai RTX 5090，~$0.39/h；模板 NVIDIA CUDA devel、筛 CUDA Max Good ≥12.9/驱动≥575、磁盘≥60GB、SSH 直连）；HF Jobs 备选；本 VM 无 GPU（VMware），3060 Ti Windows 机未采用。**GPU 步骤全部后置**：先 CPU 侧学仿真与训练管线，上机器时再开训练里程碑。
16. **M9 拆分 v2（2026-10-16 用户裁决）**：原 M9a「环境+MJCF」被认为低估了仿真搭建的学习量，重拆为五里程碑并 GPU 后置：M9a 自建 MJCF（CPU）→ M9b 训练管线解剖（CPU）→ M9c 调度对齐① FallPredictor/odometry（CPU，可交错）→ M9d 云上训练（GPU）→ M9e Velocity 全链路闭环+调度对齐②。方案与挂账覆盖映射在 tutor/m9-plan.md。
17. **M9a 视觉挂官方 STL（2026-10-20 用户裁决）**：初版自建 MJCF 视觉也是盒体拼接，用户裁决太丑改为官方 STL 视觉网格（geom 逐字照抄 robot_groundcontact.xml）。物理核心（碰撞 AABB 盒+惯量+关节树）仍零 STL、D26 收敛成果不变；视觉 geom contype/conaffinity=0 不进物理。代价：视觉层挂 CC BY-NC-SA——但本仓 BAM 默认路径本就加载同一批 STL，无新增依赖面；商用分发需替换视觉资产。验收 A 组首条断言从「XML 零 `<mesh>`」改为「编译后碰撞 geom 全 box + 视觉网格=70」。
18. **仿真宿主化（2026-10-20 用户裁决）**：M9 手搓阶段需要交互感受仿真，仿真在宿主机跑（`--viewer` 原生窗口），容器 noVNC 太卡不再用于交互。accept 系列脚本是无人值守验收，**维持容器内跑不动**（曾顺手改成双环境，用户裁决改回）。宿主机 venv 重建为 `~/microduck-verify/.venv`（uv + Python 3.12，见环境备注）。**该规则已被决策 20 收窄**：宿主机只留 viewer。
19. **M9a 补漏：实操节 + 动手练习（2026-10-20 用户裁决）**：用户读完第 18 章反馈「没学到怎么搭模型、没有构建命令」。诊断成立——本章只教了「为什么这么搭」，搭建动作全在后台。用户裁决双补：教程补「搭建实操」五步循环节 + **用户亲手从零手写 2 关节腿链**（路线=项目内引导）。练习刻意不写 `<inertial>`，与 duck_handmade 直写惯量形成对照。**2026-10-25 路径迁到 `exercises/mjcf-leg-chain/`**（作业 `exercise_leg.xml`，参考解答 `exercise_leg_solution.xml`，验收 `accept/m09a-b02.sh`），不再放 `impl/`。
20. **环境归属（2026-10-20 用户裁决，覆盖决策 18 的后半段）**：**跟仿真界面无关的一律容器内跑**——数据提取 `m9a_extract_official.py`、全部 `accept-*.sh`、模型编译与数值断言都在容器用 `/opt/sim-venv/bin/python`；**只有用户要亲眼看的（viewer 交互）在宿主机**。宿主机 Python 环境从 `~/microduck-verify/.venv` 迁到**仓库根 `.venv/`**，只装 viewer 所需，已 gitignore，建法写进 `tutor/dev-container.md`「宿主机 viewer 环境」。`accept/m09a-b01.sh` / `accept/m09a-b02.sh` 自带解释器探测。旧目录 `~/microduck-verify/.venv` **未删，等用户确认**。
21. **对齐 skill 先修布局（2026-10-25）**：按 clone-from-scratch-tutor 新增的 exercises / prerequisites / 先修能力表约定，把腿链练习迁出正式实现树；STATE 增加「当前模式 / 返回位置 / 当前先修记录」。

# STATE — microduck 从零手搓教程断点

> 新会话续传入口：读本文件 + user-prefs.md（项目级路线）+ deviations.md + questions.md + feature-inventory.md + docs/milestones/ 最新一章，然后从"下一步"继续。

- **原项目**：https://github.com/pollen-robotics/microduck （参考克隆在 `reference/`，只读）
- **当前位置**：**M9a Bite 1 已交付（2026-10-20）**——自建 MJCF 身体模型全绿，D26 收敛。M8/Phase 4 已收官（2026-10-15）。Bite 10（D36 对齐原版 robot.do 门口裁决）：102 单测 + accept-m8 a–w+z1–z5+x/y + accept-m8-updater 8/8 + accept-m8-bam 四项全绿，教程 `tutorial-bite-10-do-verdict.html`（第 17 章）已抽查交付。**复盘落盘 `docs/phase-4.md`**：收敛验收结论 + 8 条原作者决策失效边界 + 5 条可迁移原则 + 检验题（机械臂 OTA 场景，待用户作答点评）。feature-inventory 批次归属回填完毕。
- **M8 Bite 1–5 已交付**：C1 接口对齐 + C2 诊断。见 `milestones/m8-convergence/acceptance.md`
- **M8 Bite 6 已交付（2026-10-12）**：BAM M6 XL330 默认接入（`/opt/sim-venv` + `better-actuator-models`，场景 `scene_groundcontact`，timestep 0.005）；D21/D34 收敛；D35 部分收敛（结束后 upright，中段 fallen 残余）；新登记 **D68**（冻结 bringup → twist 上升沿 push 0.30 踢步态）。教程 `tutorial-bite-6-bam.html`（第 14 章）
- **M8 Bite 8 已交付（2026-10-14，用户裁决 0.8 方案收尾）**，教程 `tutorial-bite-8-walk-verdict.html`（第 15 章）已发布。行走不动点结案：双侧实验坐实"0.15/0.3 不走"是公开 velstand 权重对训练分布内小命令不响应（官方 infer_policy.py 与我们 dx 都 ≈0.005；给初速度或 vx=0.8 双侧都能走，用户双边肉眼验证）。行走验收改为 vx=0.8 ×10s 前移 ≥2.0m（实测 4.9m 量级）。BAM 步进在 write（与官方主循环同序、read 无副作用；**机制订正 Q83**：读/写两种步进顺序递推等价，回退实验双向全绿，"先写再步进让起身接得住"的解释作废）。**D68 已收敛**（残余"0.3 不走"= 上游权重现状，M9 自训策略解决）。
- **M8 Bite 9 已交付（2026-10-15，Phase 4 就地对齐组）**，教程 `tutorial-bite-9-phase4-fixes.html`（第 16 章）：D13（IoError 四变体，对齐 duck-control/src/io.rs:50-71）、D20（**修订裁决：保留 LSTM 分支**=原版 loader 能力面，补合成 (3,3) LSTM 夹具运行时覆盖；实证：官方全 tag v1–v8+main + 7 社区采样模型全 FF）、D24（COAST_TICKS=3 滑行，robotd/src/main.rs:137；consecutive_read_errors 每次失败即计、skipped_reads 语义收窄为真瞎跳拍）、D65（model_api None 放行）。**新登记 D69**（盲拍 hold 写入 + RESET_AFTER_PAUSE=200ms + BUS_DROP_QUIET 未移植，挂 M10）。
- **下一步**：**M9a Bite 1 已交付（2026-10-20）**：自建 MJCF 身体模型（`sim/assets/duck_handmade.xml` + `scene_handmade.xml`——零 STL、全基本几何体、显式 inertial 对齐官方编译值、10 碰撞盒=官方碰撞网格 AABB），**D26 已收敛**（自建落体/推倒 z=-0.0002 vs 官方对照 -0.0137），验收 `scripts/accept-m9a-bite1.sh` 全绿，档案 `docs/milestones/m9a-handmade-mjcf/acceptance.md`。下一步：**M9a Bite 2**（BAM 参数对账 microduck_constants.py:163-176 + 自建模型闭环：velstand 站立 + vx=0.8 ×10s ≥2.0m）。M9 五里程碑拆分在 `docs/m9-plan.md`；microduck_rl 侦察报告在 `docs/recon-microduck-rl.md`。**环境备注**：2026-10-14 重建镜像后 `/opt/sim-venv` 首次真正进镜像；mujoco 3.16。宿主机可直接跑仿真+daemon（`~/microduck-verify/.venv` 有 mujoco+bam+onnx；二进制 bookworm 构建）。宿主机玩法：`~/microduck-verify/.venv/bin/python sim/duck_body.py --viewer --port 7801` + `ORT_DYLIB_PATH=$PWD/third_party/onnxruntime/lib/libonnxruntime.so target/debug/miniduckd --sim 127.0.0.1:7801` + `target/debug/mini-duckctl move 0.8 0 0 --secs 10`。

## 里程碑进度

| 里程碑 | 状态 |
|---|---|
| M0 IPC 骨架 | ✅ 验收通过（milestones/m0-ipc-skeleton/，教程+验收档案） |
| M1 身体模型+FakeIo | ✅ 验收通过（milestones/m1-body-fakeio/，教程+验收档案） |
| M2 观测向量 61 维 | ✅ 验收通过（milestones/m2-obs-vector/，教程+验收档案） |
| M3 单策略推理（站立） | ✅ 验收通过（milestones/m3-stand-policy/，教程+验收档案） |
| M4 仿真行走（MuJoCo over TCP） | ✅ 验收通过（milestones/m4-sim-walk/，教程+验收档案） |
| M5 安全层（deadman/限位/跌倒） | ✅ 验收通过（milestones/m5-safety/，35 单测+accept-m5 六项+accept-m4 回归全绿） |
| M6 技能调度器（多策略优先级链） | ✅ 验收通过（milestones/m6-scheduler/，教程+验收档案；49 单测+accept-m6+accept-m5 回归全绿）。新：scheduler.rs（Cascade 纯状态机+Scheduler 持策略槽）、robot.do/skills/mouth/head、state 载荷 skill 字段、busy 抑制 Limp。权重：fetch-m6.sh 下 4 个（全 feedforward，D20 备注）。物理现实：ground_pick/kick 物理通过；sit/roulade 调度正确但物理摔倒（D34/D35，M8 随 D21 裁决） |
| M7 支线：迷你 updaterd | ✅ 验收通过（milestones/m7-updaterd/，教程+验收档案；60 单测（49 旧+11 新）+accept-m7 五场景 39 断言全绿）。新：src/updater/ 十模块、bin/mini-updaterd、duckctl update 子命令组、accept-m7.sh；miniduckd 零改动。新开 D37–D41，D6 移交 D37/D38 |
| M8 收敛验收 | ✅ 全部交付（2026-10-15）：Bite 1–6 + 8（C 线 + BAM + 行走裁决）+ Bite 9（D13/D20/D24/D65）+ Bite 10（D36）；教程第 9–17 章；Phase 4 复盘在 docs/phase-4.md |
| M9 仿真与训练（批次 2） | **进行中（2026-10-16 开工）**：拆 M9a–M9e 五个里程碑、GPU 后置（方案 docs/m9-plan.md）。**M9a Bite 1 已交付（2026-10-20）**：自建 MJCF 身体模型（零 STL、D26 收敛），验收 accept-m9a-bite1.sh 全绿；Bite 2（BAM 对账+闭环行走）当前。后续：M9b 训练管线解剖（CPU）；M9c 调度对齐① FallPredictor/odometry（CPU）；M9d 云上训练（GPU，Vast.ai 5090）；M9e Velocity 全链路闭环+调度对齐② |
| M10 驱动与硬件（批次 3） | 未开始（规划：真机驱动编程，硬件自购、可能非官方件；对齐 duck-control 硬件面。**此批完成后代码完全对齐原项目**） |
| M11 结构与外壳（批次 4） | 未开始（规划：自学 3D 建模与打印，与代码无关） |

## 关键路径与命令

- 手搓代码：本目录根（Cargo.toml / src/）。`src/lib.rs` 协议（Request.id 可缺省=notification；MoveParams、EnableParams、**SubscribeParams/SubscribeResult（M8 Bite 3）**）、`src/main.rs` miniduckd（robot.enable{on,toggle}/move/health/**subscribe** + M6: do/skills/mouth/head；notification 统一入口；**robot.disable 已删（M8 Bite 2）**；**带 id 的 robot.state = METHOD_NOT_FOUND（M8 Bite 3）**）、`src/safety.rs` Safety（唯一写句柄，M5）、`src/control.rs` 50Hz 控制任务（**M8 Bite 2 起五阶段 Held/RampUp/Driving/Limp/Stopped——RampDown 退役，disable=直接回 home 保持 torque on**；Driving 走 driving_tick；edge_transition/ramp_done_phase 纯函数 :307/:322）、`src/scheduler.rs` 技能调度器（Cascade 纯状态机 + Scheduler 持 4 槽策略 + **reset() 清 LSTM 槽（M8 Bite 2）**）、`src/policy.rs` 单网络推理（pub reset + reset_calls）、`src/obs.rs` 61 维观测、`src/io.rs` RobotIo(read/write/set_gain/set_torque/imu_ready)+FakeIo（write 精确回写，io.rs:340-344）+SimIo、`src/model.rs` 关节常量+嘴映射、`src/bin/mini-duckctl.rs` CLI（**enable [on|off]**/move/do/skills/mouth/head/state + update 子命令组；**disable 子命令已删**）、`src/updater/` 十模块 + `src/bin/mini-updaterd.rs`（socket /tmp/mini-updaterd.sock，env MINIDUCK_UPDATER_ROOT/GATE_MS/SOCK/EXIT_AFTER_SWAP；**M8 Bite 4 新增 `src/updater/robot.rs` — updaterd 的 robotd 客户端 + apply 最前面的预检**）
- **M5 后行为变化**：daemon 启动不再自动站立（Held 抱持启动姿态），要 `mini-duckctl enable` 才使能；`move` 单次意图 500ms 后被 deadman 清零，CLI `--secs` 期间每 100ms 重发意图；socket 文件 0660
- **M6 后行为变化**：Driving 阶段嘴被 robot.mouth 意图覆写（默认 0 → −5°，其他阶段嘴跟随抱持/斜坡目标）；robot.do 只在 enabled 且 Driving 时接受；head 无 deadman；busy（技能/ground_pick/起身中）抑制跌倒 Limp 转移
- **M8 Bite 2 后行为变化**：`enable off` 不再卸 torque——机器人直接回 home 保持上电抱持（Stopped），再 enable 立即回 Driving 无斜坡；全循环无 set_torque(false) 路径（卸力出口留给未来 robot.relax，D53）；任何方法的无 id 帧不应答（意图静默应用/其余静默丢弃）；robot.disable 彻底消失（METHOD_NOT_FOUND）
- **M8 Bite 3 后行为变化**：订阅只走带 id 的 `robot.subscribe`，ack 是启动时冻住的策略名单（`accepted` 恒 true，不回显 hz）；推送 method 仍是 `robot.state`、仍约 50Hz（发 `hz:10` 也不降频）；带 id 的 `robot.state` 得到 METHOD_NOT_FOUND，不会因此开始或停止推送；无 id 的 `robot.subscribe` 静默丢弃
- **M8 Bite 4 后行为变化**：`mini-duckctl update apply` 在做任何事之前先问 miniduckd「现在重启安全吗」「model_api 对不对」，不通过就 -32003 拒绝且磁盘与 current 都不动；安全判定 = 走路中（Driving 且 busy 或有 twist / 斜坡 / 瘫软）且没跌倒；model_api 必须是 2。注意 daemon 不可达时 safeToRestart 放行但 model_api 会被拒（D65 待裁决）
- M4 仿真体：`sim/duck_body.py`（MuJoCo TCP+NDJSON，资产在 `sim/assets/`；M5 新增 op：set_gain/set_torque/push，read 回显 gain/torque）。可视化两路：`--view-port 7802` MJPEG 网页（osmesa 离屏，只读，http://localhost:7802/ ）；`--viewer` MuJoCo 原生 viewer（glfw，容器里配 `scripts/duck-vnc.sh` 的 Xvfb+x11vnc+noVNC，浏览器 http://localhost:6080/vnc.html 可鼠标交互推鸭子，详见 docs/dev-container.md「交互式仿真」；compose 已映射 6080）。镜像含 mujoco+pillow+libosmesa6+xvfb+x11vnc+novnc+websockify，容器重建不丢
- 权重与 Runtime：`bash scripts/fetch-m3.sh`（代理 `http://127.0.0.1:7890`）。产物 `policies/velstand.onnx`、`third_party/onnxruntime/`（均 gitignore，换机器要重跑 fetch-m3.sh + fetch-m6.sh）。容器环境变量 `ORT_DYLIB_PATH=/work/third_party/onnxruntime/lib/libonnxruntime.so`
- 参考克隆：`reference/`（只读）。浅克隆走同一代理（环境已迁回原生 Linux，代理在 127.0.0.1:7890；容器内 PyPI 直连可达）
- 构建/测试：在容器里做，见 `docs/dev-container.md`。`docker compose exec rust cargo build && docker compose exec rust cargo test`。不要用 `bash -lc` 包 cargo，否则 PATH 会丢
- M4 验收：`docker compose exec rust bash scripts/accept-m4.sh`。M5：`accept-m5.sh`（sim 端口 7803）。M6：`accept-m6.sh`（sim 端口 7804；权重先跑 `bash scripts/fetch-m6.sh`）。M7：`accept-m7.sh`（五场景 39 断言；根目录 /tmp/m7-demo 隔离，不需要仿真；**本机 D1/E1 有时序 flake，见下**）。M8：`accept-m8.sh`（FakeIo 无需仿真；断言 a–w）+ `accept-m8-updater.sh`（Bite 4 updater 预检端到端，6 断言，根目录 /tmp/m8-updater-demo 隔离，不需要仿真）。两者都先 `cargo build --bins`：`cargo test` 不更新 `target/debug/miniduckd`
- 最新架构图：`docs/milestones/m8-convergence/arch.d2` + `arch.svg`（Bite 6：BAM+groundcontact；arch-diff 相对 Bite 5：duck_body 醒目，control/scheduler/io 次醒目）
- 仿真：默认 `/opt/sim-venv/bin/python sim/duck_body.py`（BAM；宿主机可用 `~/microduck-verify/.venv/bin/python` 直接跑，`--viewer` 原生窗口不卡）；`--no-bam` 回退。物理验收：`bash scripts/accept-m8-bam.sh`（端口 7806；行走 vx=0.8 ≥2.0m）。诊断：`bash scripts/accept-m8-bam-diag.sh`（强制 no-bam）
- **当前仿真行为（Bite 8 已验收）**：默认仍是 BAM+groundcontact，roulade 窗口 2.0s。冻结和 push 0.30 不在代码里。`--sim` 且策略已加载时第一拍就是 Driving。假总线仍是 Held→RampUp。BAM 的 `read` 不步进，`write` 写入 `q_target` 后再 `step_tick`；非 BAM 仍是读时步进。站立、坐姿起身、roulade 结束直立、vx=0.8 行走（10s 前移 ≈4.9m）都过。vx=0.15/0.3 不迈步是上游 velstand 权重现状（官方双侧一致），M9 自训策略解决。
- Rust 工具链：容器 `miniduck-rust`，rustc 1.99.0（`rust:1-bookworm`，DaoCloud 镜像；2026-10-06 换机重建后从 1.98.1 升到 1.99.0）。宿主机不装 rustup
- HF Hub 直连本机超时，下载走镜像或上述代理

## 环境迁移（2026-10-06 换电脑）

- 本机 docker 是 **rootless 模式**（context=rootless）：容器 uid 0 映射到宿主机当前用户。compose 已加 `user: "0:0"`（容器内以 root 跑，产物落回宿主机属 jqlin）；Dockerfile 的 dev 用户是 rootful 模式备用的。详见 docs/dev-container.md
- 容器已重建（`docker compose up -d --build`），权重/ORT 已重下（fetch-m3/m6），基线 81 单测全绿
- **accept-m7 已知 flake**：D1「没有 Busy」/ E1「pending」在本机间歇失败（新机器更快，场景 D 同版本 apply 在 sleep 1 窗口内瞬间 committed，Busy 窗口抓空；基线上同样复现，与代码改动无关）。Bite 2 期间实测一次全绿一次 FAIL=2。待办：D1 改轮询 Busy 窗口或同版本 apply 短路（非 M8 范围）

## 用户背景与偏好

见 `skills/clone-from-scrach-tutor/user-prefs.md`（用户级背景）+ `docs/user-prefs.md`（项目级：整机闭环路线、硬件与 BAM 侦察结论）；流程规则（类比/章节结构/行文分层/画图/代码风格/用词）已上收 SKILL.md。要点：里程碑代码由 coder subagent 构建、主 agent 复核验收；终态必须与原项目一致（铁律 8/9）。

偏差簿：D1–D41 + D64–D68（D42–D63 为功能清单缺口号）。**Phase 4 裁决已落账（2026-10-15，见 deviations.md 顶部裁决段）**：A 组核销（D1 等）；永久保留 D10/D27/D40/D43③；Bite 9 就地收敛中 D13/D20/D24/D65；挂 M9：D4/D23/D26/D28/D29/D30/D32/D33/D35 残余/odometry/「0.3 不走」上游权重；挂 M10：D5/D7/D12/D15（目标值改 31）/D25/D37–D41/D42–D63/D66/D67；挂 M11：D31。**D36 漏列待补问**。D2 豁免维持；padd/pad.* 不豁免改 M10 再评估（用户有 Xbox 手柄）。

## M6 复核更正（2026-09-28，教程写作时发现）

交接记录曾称「原版 --sim 是无物理回显（reference/robotd/src/main.rs:240）」——**误读**：该注释挂在 `--fake` 上；原版 `--sim` 是真 MuJoCo（reference/docs/design/simulation.md 声称 sitstand 在原版仿真里能起身并保持直立）。事实：原版仿真体 duck-body 在 microduck_rl 仓库、不在浅克隆里，我们的 `sim/duck_body.py` 是手搓的这一半。这加强了 D34/D35 → D21（BAM 执行器动力学缺失）的归因。详见 milestones/m6-scheduler/tutorial.md 第 4 节末。

## M5 侦察发现（2026-09-27，纠正了 M5 预案的两个错误前提）

1. **robotd 无 SO_PEERCRED**：原版 robotd 连接处理零凭据校验，socket 0660 文件权限就是全部鉴权（robotd/src/main.rs:60）；SO_PEERCRED+is_mutating 是 configd/updaterd 的模式，且 mutating 指"改软件/配置"，robot.move/enable 都不算。D8 按 0660 收敛，SO_PEERCRED 留给 M7。
2. **跌倒卸力不在 Safety 内部**：safety.rs 的 apply 明确无跌倒门——fallen 是纯报告，卸力是 robotd 层 LimpFall 状态机用普通 targets+gain 调 apply 完成的（原版 limp_fall 默认 OFF）。我们的跌倒响应照此放控制循环（D23 登记简化）。
3. 原版启动语义：Bringup=Limp，adopt_startup_pose 抱住读到的姿态，绝不 set_torque（舵机 RAM 的 torque 跨进程存活），显式 robot.enable 才动。M5 已对齐（D9 收敛）。

## 决策记录（用户裁决过的事）

1. 实现语言：**Rust**（跟随原版）
2. 硬件闭源不影响教程：验收走 FakeIo/仿真路线
3. 教程规则已更新：终态对齐原版（注释/文档/死代码不复刻），中间里程碑允许偏离但须登记收敛点
4. `examples/` 是教程配套资产（概念示例代码），不是里程碑交付物，M8 收敛验收对照原版结构时不计入偏差
5. 教程硬标准：领域专有概念先大白话引入 + 1~2 段实测跑通的最小代码示例，再进项目实现
6. **M8 范围与后续路线（2026-10-04 用户裁决）**：M8 走 C 线——只对齐非硬件依赖内容（robot.move+vy / robot.enable{on,toggle} / robot.subscribe 入口 / updater safeToRestart 预检）+ 移植 BAM（Rhoban/bam 解析式模型，官方蓝图 microduck_rl scripts/infer_policy.py；先跑 kp=0.55 一行实验验证归因）让 sit/roulade 物理通过，D21/D34/D35 在 M8 收敛。**其余功能缺口（D42–D63 及偏差簿残余项）不做豁免收尾**，分配到后续批次：M9 仿真与训练、M10 驱动与硬件（此批完成后代码完全对齐原项目，D52 crate 边界/配置面等在此批或 M9 收敛）、M11 结构与外壳（无代码）。总目标=整机工程闭环，详见 docs/user-prefs.md
7. **CLI disable 一并删除（2026-10-06 M8 Bite 2）**：daemon 删 robot.disable 是铁律 8 强制（原版没有）；用户裁决 CLI 也不留别名——关就是 `mini-duckctl enable off`，accept-m5.sh A1c 等脚本跟着改
8. **disable 语义差异立即对齐（2026-10-06 M8 Bite 2）**：复核发现原版 disable 不卸 torque、无斜坡、直接命令回 home 保持上电（此前未登记，新登记 D64）。用户裁决不挂账、本 Bite 立即对齐——RampDown 退役、Stopped 上岗；代价登记在案：robot.relax 落地前卸 torque 无出口（D53 缺口待排期）
9. **D68 单独开 Bite 清掉（2026-10-13）**：用户裁决不挂到 M9，并选择策略在倒下之前进环。代码已做成 `--sim` 第一拍 Driving。站立 z≈0.117 通过。坐姿起身 50 拍后进 Limp（z≈0.042），Bite 7 未交付。没有写回冻结或注速度。D35 / D65 不动。
10. **起身交接另开 Bite 8（2026-10-14）**：1 秒窗口保持与原项目一致。下一 Bite 只对齐「先写目标、再步进」，用来让起身结束时身体进入 velstand 接得住的状态。不把 D68 装回。
11. **Bite 8 行走验收按 0.8 方案收尾（2026-10-14）**：用户肉眼验证官方窗口（0.3 不走、P 键注速度能走、打补丁 0.8 能走）+ 我们鸭子同条件复现（0.8 能走）后裁决：行走验收改 vx=0.8 ×10s 前移 ≥2.0m（实测 4.9m），验收目标=集成栈能驱动步态；0.3 不走登记为上游 velstand 权重现状（非我方偏差，不进豁免清单），M9 自训策略时真正解决。D68 收敛。
12. **Phase 4 版本基准（2026-10-15）**：reference 以磁盘克隆 daemon-v0.14.1（API_VERSION=31，origin/main HEAD，commit 344925c）为终态对齐基准；清单原按 API 37 检出写成，行号证据回填修正，API 32–37 差量留 M10 终态盘点。
13. **Phase 4 就地对齐组（2026-10-15，M8 Bite 9）**：D13 IoError 分类、D24 COAST_TICKS=3、D65 model_api None 放行、D20 删 LSTM 死代码。D20 前提由用户确认后查证：HF pollen-robotics/microduck-policies 全部 tag（v1–v8+main）只有同 10 个 onnx，全量探针验证均 1-in/1-out feedforward——原版 LSTM 分支从未被自家模型触发，故删；M9 自训若用 recurrent 再写回。
14. **Phase 4 批次挂账（2026-10-15）**：D23/D32（FallPredictor）挂 M9；D31 音频族**不豁免**挂 M11 再评估；odometry 从建议豁免改挂 M9；padd/pad.* 不豁免挂 M10 再评估（用户有 Xbox 手柄）；account.* 并入 D60；D61 扩为协议级错误码；D10/D27/D40/D43③ 永久保留（理由在 deviations.md 各行）。
15. **M9 算力路线（2026-10-16）**：主力云 GPU（Vast.ai RTX 5090，~$0.39/h；模板 NVIDIA CUDA devel、筛 CUDA Max Good ≥12.9/驱动≥575、磁盘≥60GB、SSH 直连）；HF Jobs 备选；本 VM 无 GPU（VMware），3060 Ti Windows 机未采用。**GPU 步骤全部后置**：先 CPU 侧学仿真与训练管线，上机器时再开训练里程碑。
16. **M9 拆分 v2（2026-10-16 用户裁决）**：原 M9a「环境+MJCF」被认为低估了仿真搭建的学习量，重拆为五里程碑并 GPU 后置：M9a 自建 MJCF（CPU）→ M9b 训练管线解剖（CPU）→ M9c 调度对齐① FallPredictor/odometry（CPU，可交错）→ M9d 云上训练（GPU）→ M9e Velocity 全链路闭环+调度对齐②。方案与挂账覆盖映射在 docs/m9-plan.md。

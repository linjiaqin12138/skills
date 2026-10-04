# STATE — microduck 从零手搓教程断点

> 新会话续传入口：读本文件 + deviations.md + questions.md + docs/milestones/ 最新一章，然后从"下一步"继续。

- **原项目**：https://github.com/pollen-robotics/microduck （参考克隆在 `reference/`，只读）
- **当前位置**：Phase 3 进行中，Bite 8（M7 支线：迷你 updaterd）已通过验收，教程章已落盘（milestones/m7-updaterd/）
- **下一步**：用户说"继续"→ Phase 4 / M8 收敛验收 + 权衡复盘（全部里程碑已走完，M8 前可先问用户是否要补问/复习）

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
| M8 收敛验收 | 未开始 |

## 关键路径与命令

- 手搓代码：本目录根（Cargo.toml / src/）。`src/lib.rs` 协议、`src/main.rs` miniduckd（robot.enable/disable/drive/health/state + M6: do/skills/mouth/head）、`src/safety.rs` Safety（唯一写句柄，M5）、`src/control.rs` 50Hz 控制任务（Held/RampUp/Driving/Limp/RampDown 五阶段；M6 起 Driving 走 driving_tick）、`src/scheduler.rs` 技能调度器（M6：Cascade 纯状态机 + Scheduler 持 4 槽策略）、`src/policy.rs` 单网络推理（pub reset + reset_calls）、`src/obs.rs` 61 维观测、`src/io.rs` RobotIo(read/write/set_gain/set_torque/imu_ready)+FakeIo+SimIo、`src/model.rs` 关节常量+嘴映射、`src/bin/mini-duckctl.rs` CLI（enable/disable/drive/do/skills/mouth/head/state + M7: update check/status/log/apply/rollback）、**M7 新增** `src/updater/` 十模块（paths/manifest/store/journal/gate/lock/peer/engine/ipc/mod）+ `src/bin/mini-updaterd.rs`（socket /tmp/mini-updaterd.sock，env MINIDUCK_UPDATER_ROOT/GATE_MS/SOCK/EXIT_AFTER_SWAP）
- **M5 后行为变化**：daemon 启动不再自动站立（Held 抱持启动姿态），要 `mini-duckctl enable` 才使能；`drive` 单次意图 500ms 后被 deadman 清零，CLI `--secs` 期间每 100ms 重发意图；socket 文件 0660
- **M6 后行为变化**：Driving 阶段嘴被 robot.mouth 意图覆写（默认 0 → −5°，其他阶段嘴跟随抱持/斜坡目标）；robot.do 只在 enabled 且 Driving 时接受；head 无 deadman；busy（技能/ground_pick/起身中）抑制跌倒 Limp 转移
- M4 仿真体：`sim/duck_body.py`（MuJoCo TCP+NDJSON，资产在 `sim/assets/`；M5 新增 op：set_gain/set_torque/push，read 回显 gain/torque）。可视化两路：`--view-port 7802` MJPEG 网页（osmesa 离屏，只读，http://localhost:7802/ ）；`--viewer` MuJoCo 原生 viewer（glfw，容器里配 `scripts/duck-vnc.sh` 的 Xvfb+x11vnc+noVNC，浏览器 http://localhost:6080/vnc.html 可鼠标交互推鸭子，详见 docs/dev-container.md「交互式仿真」；compose 已映射 6080）。镜像含 mujoco+pillow+libosmesa6+xvfb+x11vnc+novnc+websockify，容器重建不丢
- 权重与 Runtime：`bash scripts/fetch-m3.sh`（代理 `http://127.0.0.1:7890`）。产物 `policies/velstand.onnx`、`third_party/onnxruntime/`（均 gitignore）。容器环境变量 `ORT_DYLIB_PATH=/work/third_party/onnxruntime/lib/libonnxruntime.so`
- 参考克隆：`reference/`（只读）。浅克隆走同一代理（环境已迁回原生 Linux，代理在 127.0.0.1:7890；容器内 PyPI 直连可达）
- 构建/测试：在容器里做，见 `docs/dev-container.md`。`docker compose exec rust cargo build && docker compose exec rust cargo test`。不要用 `bash -lc` 包 cargo，否则 PATH 会丢
- M4 验收：`docker compose exec rust bash scripts/accept-m4.sh`（M5 已更新：先 enable 再行走）。M5 验收：`docker compose exec rust bash scripts/accept-m5.sh`（六项；sim 端口 7803）。M6 验收：`docker compose exec rust bash scripts/accept-m6.sh`（sim 端口 7804；权重先跑 `bash scripts/fetch-m6.sh`）。M7 验收：`docker compose exec rust bash scripts/accept-m7.sh`（五场景 39 断言；根目录 /tmp/m7-demo 隔离，不需要仿真）
- 最新架构图：`docs/milestones/m7-updaterd/arch.d2` + `arch.svg`（唯一权威，M7 验收后完整架构；arch-diff.d2/.svg 为相对 m6 的变动图）。d2/dagre，`d2 --layout dagre arch.d2` 可复现；每个里程碑按 SKILL「架构图约定」发 arch + arch-diff 两张图
- Rust 工具链：容器 `miniduck-rust`，rustc 1.98.1（`rust:1-bookworm`，DaoCloud 镜像）。宿主机不装 rustup
- HF Hub 直连本机超时，下载走镜像或上述代理

## 用户背景与偏好

见 `skills/clone-from-scrach-tutor/user-prefs.md`（用户级：背景/语言/通用流程）+ `docs/user-prefs.md`（项目级：整机闭环总目标、四条 track、硬件与 BAM 侦察结论）。要点：默认直讲概念、不主动做 TS 类比（2026-09-26 推翻旧约定）；语言三层规则；里程碑代码由 coder subagent 构建、主 agent 复核验收；终态必须与原项目一致（铁律 8/9）。

偏差簿：41 条登记（D1–D41）。D2 已豁免。D16、D17 已在 M3 收敛，D18 已在 M4 收敛，D8/D9/D11/D14/D19/D22 已在 M5 收敛，D3 已在 M6 收敛，D5/D12 部分收敛（残余留 M8），D6 已移交 D37/D38。M5 新开 D23–D27。M6 新开 D28–D36（含 D36：robot.do 在 fallen 时 RPC 侧拒绝 vs 原版静默丢）。M7 新开 D37（无 systemd，updaterd 兼任 supervisor）、D38（无 minisign，只 sha256）、D39（LocalDir 单一 source/未压缩 tar）、D40（SO_PEERCRED-lite）、D41（无 hooks/transcript/subscribe/self-update 等）。

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

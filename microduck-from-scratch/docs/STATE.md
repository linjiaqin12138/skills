# STATE — microduck 从零手搓教程断点

> 新会话续传入口：读本文件 + deviations.md + questions.md + docs/milestones/ 最新一章，然后从"下一步"继续。

- **原项目**：https://github.com/pollen-robotics/microduck （参考克隆在 `reference/`，只读）
- **当前位置**：Phase 3 进行中，Bite 5（M4）已通过验收，教程章已落盘
- **下一步**：用户说"继续"→ 构建 Bite 6（里程碑 M5：安全层。deadman/限位/跌倒卸力，对齐原版 safety.rs；同时收敛 D9/D11/D12/D14/D19，robot.drive 是第一个 mutating 调用，SO_PEERCRED 校验到位）

## 里程碑进度

| 里程碑 | 状态 |
|---|---|
| M0 IPC 骨架 | ✅ 验收通过（milestones/m0-ipc-skeleton/，教程+验收档案） |
| M1 身体模型+FakeIo | ✅ 验收通过（milestones/m1-body-fakeio/，教程+验收档案） |
| M2 观测向量 61 维 | ✅ 验收通过（milestones/m2-obs-vector/，教程+验收档案） |
| M3 单策略推理（站立） | ✅ 验收通过（milestones/m3-stand-policy/，教程+验收档案） |
| M4 仿真行走（MuJoCo over TCP） | ✅ 验收通过（milestones/m4-sim-walk/，教程+验收档案） |
| M5 安全层（deadman/限位/跌倒） | 未开始 |
| M6 技能调度器（多策略优先级链） | 未开始 |
| M7 支线：迷你 updaterd | 未开始 |
| M8 收敛验收 | 未开始 |

## 关键路径与命令

- 手搓代码：本目录根（Cargo.toml / src/）。`src/lib.rs` 协议、`src/main.rs` miniduckd、`src/control.rs` 50Hz 控制任务、`src/policy.rs` velstand 推理、`src/obs.rs` 61 维观测、`src/io.rs` RobotIo+FakeIo+SimIo、`src/model.rs` 关节常量、`src/bin/mini-duckctl.rs` CLI（含 drive）
- M4 仿真体：`sim/duck_body.py`（MuJoCo TCP+NDJSON，资产在 `sim/assets/`）。加 `--view-port 7802` 开 MJPEG 网页画面（osmesa 离屏渲染，浏览器 http://localhost:7802/ ；compose 已映射 7802）。依赖 mujoco+pillow+libosmesa6 已打进 Dockerfile，容器重建不丢
- 权重与 Runtime：`bash scripts/fetch-m3.sh`（代理 `http://127.0.0.1:7890`）。产物 `policies/velstand.onnx`、`third_party/onnxruntime/`（均 gitignore）。容器环境变量 `ORT_DYLIB_PATH=/work/third_party/onnxruntime/lib/libonnxruntime.so`
- 参考克隆：`reference/`（只读）。浅克隆走同一代理（环境已迁回原生 Linux，代理在 127.0.0.1:7890；容器内 PyPI 直连可达）
- 构建/测试：在容器里做，见 `docs/dev-container.md`。`docker compose exec rust cargo build && docker compose exec rust cargo test`。不要用 `bash -lc` 包 cargo，否则 PATH 会丢
- M4 验收：`docker compose exec rust bash scripts/accept-m4.sh`（一键：sim→daemon→站稳→drive 10s→断言位移≥0.5m）
- Rust 工具链：容器 `miniduck-rust`，rustc 1.98.1（`rust:1-bookworm`，DaoCloud 镜像）。宿主机不装 rustup
- HF Hub 直连本机超时，下载走镜像或上述代理

## 用户背景与偏好

见 `skills/clone-from-scrach-tutor/user-prefs.md`。要点：类比用 TypeScript/Node.js 生态；里程碑代码由 coder subagent 构建、主 agent 复核验收；终态必须与原项目一致（铁律 8/9）。

偏差簿：22 条登记（D1–D22）。D2 已豁免。D16、D17 已在 M3 收敛，D18 已在 M4 收敛。M4 新开 D4（正式引入）、D21（几何/惯量/阻尼已随模型换血收敛到 microduck_rl 训练资产，残余仅执行器 XML PD vs BAM）。M4 代码评审新开 D22（quat 宽松解析，M5 换严格 copy4）。

## 决策记录（用户裁决过的事）

1. 实现语言：**Rust**（跟随原版）
2. 硬件闭源不影响教程：验收走 FakeIo/仿真路线
3. 教程规则已更新：终态对齐原版（注释/文档/死代码不复刻），中间里程碑允许偏离但须登记收敛点
4. `examples/` 是教程配套资产（概念示例代码），不是里程碑交付物，M8 收敛验收对照原版结构时不计入偏差
5. 教程硬标准：领域专有概念先大白话引入 + 1~2 段实测跑通的最小代码示例，再进项目实现

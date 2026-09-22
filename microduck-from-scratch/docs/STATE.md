# STATE — microduck 从零手搓教程断点

> 新会话续传入口：读本文件 + deviations.md + questions.md + docs/milestones/ 最新一章，然后从"下一步"继续。

- **原项目**：https://github.com/pollen-robotics/microduck （参考克隆在 `reference/`，只读）
- **当前位置**：Phase 3 进行中，Bite 2（M1）已通过验收
- **下一步**：用户说"继续"→ 构建 Bite 3（里程碑 M2：61 维观测向量构建，见 docs/phase-2.md 里程碑表）

## 里程碑进度

| 里程碑 | 状态 |
|---|---|
| M0 IPC 骨架 | ✅ 验收通过（milestones/m0-ipc-skeleton/，教程+验收档案） |
| M1 身体模型+FakeIo | ✅ 验收通过（milestones/m1-body-fakeio/，教程+验收档案） |
| M2 观测向量 61 维 | 未开始 |
| M3 单策略推理（站立） | 未开始 |
| M4 仿真行走（MuJoCo over TCP） | 未开始 |
| M5 安全层（deadman/限位/跌倒） | 未开始 |
| M6 技能调度器（多策略优先级链） | 未开始 |
| M7 支线：迷你 updaterd | 未开始 |
| M8 收敛验收 | 未开始 |

## 关键路径与命令

- 手搓代码：本目录根（Cargo.toml / src/）。`src/lib.rs` 协议、`src/main.rs` miniduckd、`src/control.rs` 50Hz 控制任务、`src/io.rs` RobotIo+FakeIo、`src/model.rs` 关节常量、`src/bin/mini-duckctl.rs` CLI
- 构建/测试：`export PATH="$HOME/.cargo/bin:$PATH" && cargo build && cargo test`
- 验收：`./target/debug/miniduckd &` → `./target/debug/mini-duckctl health` / `timeout 3 ... subscribe` → `pkill -x miniduckd`
- Rust 工具链：rustc 1.98.1 经 rustup 安装；cargo 源已配 rsproxy 镜像（`~/.cargo/config.toml`）
- HF Hub 直连本机超时，M3 下载策略权重需走镜像（hf-mirror.com）或代理——届时处理

## 用户背景与偏好

见 `skills/clone-from-scrach-tutor/user-prefs.md`。要点：类比用 TypeScript/Node.js 生态；里程碑代码由 coder subagent 构建、主 agent 复核验收；终态必须与原项目一致（铁律 8/9）。

## 决策记录（用户裁决过的事）

1. 实现语言：**Rust**（跟随原版）
2. 硬件闭源不影响教程：验收走 FakeIo/仿真路线
3. 教程规则已更新：终态对齐原版（注释/文档/死代码不复刻），中间里程碑允许偏离但须登记收敛点
4. `examples/` 是教程配套资产（概念示例代码），不是里程碑交付物，M8 收敛验收对照原版结构时不计入偏差
5. 教程硬标准：领域专有概念先大白话引入 + 1~2 段实测跑通的最小代码示例，再进项目实现

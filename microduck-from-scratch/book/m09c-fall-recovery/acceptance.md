# M9c 验收档案：策略面调度语义对齐①

> 范围：D23/D32 FallPredictor + 相对 odometry。教程分两章目录：`m09c-fall-recovery/`、`m09c-odometry/`；本文件挂在里程碑首章目录。

## Bite 1：FallPredictor + Limp/Posing（2026-10-26）

验收入口：`accept/m09c-b01.sh`；实测：`tutor/runs/m09c-b01.txt`。

- FakeIo / sim 构造跌倒：预测触发、Limp→Posing→Driving、busy/sitting 门控
- 默认 OFF；`MINIDUCK_LIMP_FALL=1` 显式开启
- 决定：`decisions/0006-m9c-fall-recovery.md`（已采纳）
- 教程：`b01-fall-recovery.html`（第 25 章）

结论：**PASS（已确认）**。D23/D32 收敛。

## Bite 2：相对 odometry（2026-10-26 交付 · 2026-10-29 确认）

验收入口：`accept/m09c-b02.sh`；实测：`tutor/runs/m09c-b02.txt`。

- A：默认 Rust 单测 126 全绿（含 odometry / control 集成）
- B/C：live 行走 / 转身（ignored 测，脚本显式跑）
- D：M8 全回归
- 决定：`decisions/0007-m9c-odometry.md`（已采纳）
- 教程：`m09c-odometry/b01-odometry.html`（第 26 章）

结论：**PASS（已确认）**。估计器进内部 FrameSnapshot；`robot.state` 无 odom（D50→M10）。

## 里程碑收官复核（2026-10-29）

| 计划承诺 | 结果 |
|---|---|
| D23 + D32 FallPredictor / Limp / Posing | 已收敛 |
| 相对 odometry（机载估计） | 已收敛（局部 FK + ALPHA16） |
| D50 `RobotState.odom` 字段 | **不在本里程碑**；仍挂 M10 |
| D55 通用 kinematics crate | **不在本里程碑**；仍挂 M10；0007 失效边界要求日后替换局部 FK |

清单外功能：无。局部 FK 相对上游通用 crate 的偏离已记入 `0007`，不是漏项。

# M9b 验收档案：训练管线解剖

## Bite 1：Velocity 契约对账（2026-10-25）

验收入口：`accept/m09b-b01.sh`；实测：`tutor/runs/m09b-b01.txt`。

- A：`obs::` 5 测全绿（布局求和、块偏移、嘴排除）
- B/C：官方 twist 范围 / TIP=0.15 / scale=1.0；61D 块地图宽度和=61
- D：可选 pytest 跳过（无 RL venv），不算失败
- 决定：`decisions/0003-m9b-velocity-contract.md`（起草）
- 教程：`b01-velocity-contract.html`（第 22 章）

结论：**PASS（用户继续；反馈本课过薄，仅保留契约附录）**。决定 `0003` 已采纳。

## Bite 2：奖励与课程（2026-10-25）

验收入口：`accept/m09b-b02.sh`；实测：`tutor/runs/m09b-b02.txt`。

- A/B：Velocity 关键奖励权重 + action_rate/standing 课程两端点断言
- C：离散 walk/roll 场景；默认 walk>roll；关 upright 并放大 track_lin 后 roll 反超
- D：track_lin std≈0.316「0.3 不走」线索
- 决定：`decisions/0004-m9b-reward-curriculum.md`（起草）
- 教程：`b02-reward-curriculum.html`（第 23 章）

结论：**脚本 PASS；课文降级（2026-10-25 用户选路线 2）**——不重写第 23 章，直接开 Bite 3。决定 `0004` 已采纳（脚本可留 CI）。

## Bite 3：操作判断（2026-10-25）

验收入口：`accept/m09b-b03.sh`；实测：`tutor/runs/m09b-b03.txt`。

- A：7 题操作判断全过（冒烟 / resume / warm_start / export / manifest / 无 GPU）
- B：train_cli、export、manifest 入口文件存在
- C：无 uv sync / 无 GPU 要求
- 决定：`decisions/0005-m9b-ops-manual.md`（起草）
- 教程：`b03-ops-manual.html`（第 24 章，**无代码领读**）

结论：**PASS（等用户确认）**。

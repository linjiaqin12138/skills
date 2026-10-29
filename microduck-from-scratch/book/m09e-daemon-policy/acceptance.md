# M9e Bite 1 验收档案

- 日期：2026-10-29
- 决定：`decisions/0010-m9e-walk-action-scale.md`（起草 → 用户确认后改已采纳）
- 脚本：`accept/m09e-b01.sh`
- 跑证：`tutor/runs/m09e-b01.txt`
- 权重：`tutor/artifacts/m09d/policy_export.onnx`
- 课文：`book/m09e-daemon-policy/b01-daemon-selftrain.html`（第 29 章）

## 结论

| 项 | 结果 |
|---|---|
| scale 单测 | `walk_action_scale_defaults_and_overrides` PASS |
| 站 | z≈0.120，skill=walk，fallen=false |
| 走 vx=0.3×10s | forward≈**1.33 m**，z≈0.119，walk False |
| accept | `bash accept/m09e-b01.sh` PASS（容器） |

## 环境

- `MINIDUCK_POLICY` → M9d 自训 ONNX
- `MINIDUCK_WALK_ACTION_SCALE=1.0`
- 默认 BAM 场景 `scene_handmade.xml`

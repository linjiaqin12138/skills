# M9d Bite 2 验收档案

- 日期：2026-10-29
- 决定：`decisions/0009-m9d-infer-headless.md`（起草 → 本机肉眼确认后改已采纳）
- 脚本：`accept/m09d-b02.sh`、`impl/scripts/m9d_infer_headless.py`、`impl/scripts/m9d_infer_local.sh`
- 跑证：`tutor/runs/m09d-b02.txt`（云 headless）
- 权重：`tutor/artifacts/m09d/policy_export.onnx`（已从云机 scp）
- 课文：`book/m09d-cloud-train/b02-infer-stand-walk.html`（第 28 章）

## 结论

| 项 | 结果 |
|---|---|
| export | `scripts/export.py … model_999.pt` → `eval/policy_export.onnx` |
| 站 BAM 8s（云 headless） | fell=false，min_z≈0.114 |
| 走 BAM 0.3×10s（云 headless） | fell=false，forward≈**1.06 m** |
| 走 no-bam 0.3×10s | fell=false，forward≈0.44 m |
| accept | `bash accept/m09d-b02.sh` PASS |
| 本机 viewer | **待学员**：`bash impl/scripts/m9d_infer_local.sh`（需 `vendor/microduck_rl` + `uv sync`） |

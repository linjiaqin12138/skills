# M9d Bite 2 验收档案

- 日期：2026-10-29
- 决定：`decisions/0009-m9d-infer-headless.md`（起草 → 用户确认后改已采纳）
- 脚本：`accept/m09d-b02.sh`、`impl/scripts/m9d_infer_headless.py`
- 跑证：`tutor/runs/m09d-b02.txt`
- 课文：`book/m09d-cloud-train/b02-infer-stand-walk.html`（第 28 章）

## 结论

| 项 | 结果 |
|---|---|
| export | `scripts/export.py … model_999.pt` → `output.onnx` / `eval/policy_export.onnx` |
| 站 BAM 8s | fell=false，min_z≈0.114 |
| 走 BAM 0.3×10s | fell=false，forward≈**1.06 m** |
| 走 no-bam 0.3×10s | fell=false，forward≈0.44 m（侧偏更大） |
| accept | `bash accept/m09d-b02.sh` PASS |

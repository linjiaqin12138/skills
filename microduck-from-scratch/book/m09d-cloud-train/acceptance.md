# M9d Bite 1 验收档案

- 日期：2026-10-29
- 决定：`decisions/0008-m9d-cloud-4090.md`（起草 → 用户确认后改已采纳）
- 脚本：`accept/m09d-b01.sh`、`impl/scripts/m9d_cloud_smoke.sh`
- 跑证：`tutor/runs/m09d-b01.txt`
- 课文：`book/m09d-cloud-train/b01-cloud-smoke.html`（第 27 章）

## 结论

| 项 | 结果 |
|---|---|
| 云机 | 国内 RTX 4090，数据盘 `/cloud/cloud-ssd1` |
| 栈 | Python 3.12 / torch 2.9.1+cu128（uv，非镜像 2.1.3） |
| 冒烟 | `64×5` PASS（含 wandb login） |
| accept | `bash accept/m09d-b01.sh` PASS |

## 首训（已完成，2026-10-29）

- 命令：`2048` envs × `1000` iter；墙钟约 25 min；日志 `/cloud/cloud-ssd1/microduck-m9d/logs/first_train.log`
- wandb 项目：https://wandb.ai/linjiaqin8-na/mjlab_microduck
- wandb run：https://wandb.ai/linjiaqin8-na/mjlab_microduck/runs/weqfvj7f
- 终态：`logs/rsl_rl/velocity/2026-10-29_08-00-43_velocity/model_999.pt`；末步 mean_reward ≈ 109.15
- 脚本：`impl/scripts/m9d_first_train.sh`

## Bite 2（已交付，待用户确认）

见 [`acceptance-b02.md`](acceptance-b02.md)：官方 export + headless 站/走；`accept/m09d-b02.sh` PASS。

# M9f Bite 1 验收档案

- 日期：2026-10-29
- 决定：`decisions/0011-m9f-handmade-own-train.md`（起草 → 用户确认后改已采纳）
- 脚本：`accept/m09f-b01.sh`、`impl/scripts/m9f_train.py`、`impl/scripts/m9f_handmade_robot.py`、`impl/scripts/m9f_replay.py`
- 跑证：`tutor/runs/m09f-b01.txt`（`smoke_ok=1`）
- 课文：`book/m09f-handmade-train/b01-own-train-handmade.html`（第 30 章）

## 结论

| 项 | 结果 |
|---|---|
| 入口 | 自写 `m9f_train.py`（非 `uv run train`） |
| 身体 | `duck_handmade.xml` via `HANDMADE_ROBOT_CFG` |
| 冒烟 | 云 4090：`64×5`，`smoke_ok=1` |
| accept | `bash accept/m09f-b01.sh` PASS |

## 长训（已完成，2026-10-29）

- 命令：`2048` envs × `4000` iter，`save-interval=100`
- 墙钟约 1h14m；日志 `/cloud/cloud-ssd1/microduck-m9f/runs/m09f-train-4k.log`（`train_finished=1`）
- 目录：`/cloud/cloud-ssd1/microduck-m9f/logs/train_2048x4000_si100_20261029_102513/`
- 终局：`model_3999.pt`；本机副本 `tutor/artifacts/m09f/checkpoints/`

## HD replay（已完成，2026-10-29）

- `MUJOCO_GL=egl` + `m9f_replay.py`，1280×720，41 个 checkpoint，`replay_ok=1`
- 远端包：`/cloud/cloud-ssd1/microduck-m9f/runs/m09f-replay-4k-hd.tar.gz`
- 本机：`tutor/artifacts/m09f/videos/replay_hd/`（解压后的 `model_*.mp4`）

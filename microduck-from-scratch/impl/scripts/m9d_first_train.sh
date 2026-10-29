#!/usr/bin/env bash
# M9d：官方首训（冒烟通过后再跑）。
# 默认 2048 envs × 1000 iter —— 4090 24GB 保守起步；可改 NUM_ENVS / MAX_ITERS。
set -euo pipefail
export PATH="${HOME}/.local/bin:${PATH}"
WORK="${WORK:-/cloud/cloud-ssd1/microduck-m9d}"
RL_DIR="${RL_DIR:-$WORK/microduck_rl}"
LOG_DIR="${LOG_DIR:-$WORK/logs}"
NUM_ENVS="${NUM_ENVS:-2048}"
MAX_ITERS="${MAX_ITERS:-1000}"
TASK="${TASK:-Mjlab-Velocity-Flat-MicroDuck}"

mkdir -p "$LOG_DIR"
cd "$RL_DIR"

nvidia-smi -L
uv run python -c 'import torch; assert torch.cuda.is_available(); print(torch.__version__, torch.cuda.get_device_name(0))'

echo "train $TASK envs=$NUM_ENVS iters=$MAX_ITERS"
exec uv run train "$TASK" \
  --env.scene.num-envs "$NUM_ENVS" \
  --agent.max_iterations "$MAX_ITERS"

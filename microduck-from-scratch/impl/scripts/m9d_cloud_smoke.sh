#!/usr/bin/env bash
# M9d Bite 1：在云 GPU 上克隆 microduck_rl、uv sync、官方冒烟。
# 在数据盘工作目录执行（本机探测为 /cloud/cloud-ssd1）。
#
# 用法：
#   export WORK=/cloud/cloud-ssd1/microduck-m9d
#   bash m9d_cloud_smoke.sh
#
# 预期：
#   - nvidia-smi 可见 GPU
#   - uv sync 成功（torch==2.9.1）
#   - uv run train … --env.scene.num-envs 64 --agent.max_iterations 5 退出 0
#   - 写出 smoke-ok 标记文件
set -euo pipefail

WORK="${WORK:-/cloud/cloud-ssd1/microduck-m9d}"
RL_DIR="$WORK/microduck_rl"
MARKER="$WORK/smoke-ok.txt"
TASK="${TASK:-Mjlab-Velocity-Flat-MicroDuck}"

mkdir -p "$WORK"
cd "$WORK"

echo "== GPU =="
nvidia-smi -L
nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader

if ! command -v uv >/dev/null 2>&1; then
  echo "== install uv =="
  curl -LsSf https://astral.sh/uv/install.sh | sh
  export PATH="$HOME/.local/bin:$PATH"
fi
command -v uv
uv --version

if [ ! -d "$RL_DIR/.git" ]; then
  echo "== clone microduck_rl =="
  git clone --depth 1 https://github.com/pollen-robotics/microduck_rl.git "$RL_DIR"
fi
cd "$RL_DIR"

echo "== uv sync =="
# 锁版以仓内 pyproject/uv.lock 为准；不要用镜像 conda 的 torch 2.1.3
uv sync

echo "== versions =="
uv run python -c 'import torch,sys; print("python",sys.version.split()[0]); print("torch",torch.__version__); print("cuda",torch.version.cuda); print("gpu",torch.cuda.get_device_name(0) if torch.cuda.is_available() else "NONE")'

echo "== smoke train 64×5 =="
uv run train "$TASK" \
  --env.scene.num-envs 64 \
  --agent.max_iterations 5

{
  echo "smoke_ok=1"
  echo "task=$TASK"
  echo "host=$(hostname)"
  date -u +"%Y-%m-%dT%H:%M:%SZ"
  nvidia-smi -L
  uv run python -c 'import torch; print("torch",torch.__version__,"cuda",torch.version.cuda)'
} > "$MARKER"

echo "SMOKE PASS → $MARKER"
cat "$MARKER"

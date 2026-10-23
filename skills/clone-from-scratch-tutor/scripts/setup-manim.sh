#!/usr/bin/env bash
# manim 链路：建 venv 并安装本 skill 的 requirements.txt。
# 用法：scripts/setup-manim.sh [venv 目录，默认 ./.venv]
#
# 系统依赖需要 sudo，先让用户执行一次：
#   sudo apt-get update && sudo apt-get install -y ffmpeg \
#     texlive-latex-base texlive-latex-recommended texlive-fonts-recommended \
#     texlive-latex-extra pkg-config libcairo2-dev python3-dev libpango1.0-dev
set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV="${1:-.venv}"

python3 -m venv "$VENV"
"$VENV/bin/pip" install -U pip
"$VENV/bin/pip" install -r "$SKILL_DIR/requirements.txt"
"$VENV/bin/manim" --version

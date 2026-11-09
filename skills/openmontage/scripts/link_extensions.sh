#!/usr/bin/env bash
set -euo pipefail

skill_dir=$(cd "$(dirname "$0")/.." && pwd)
repo_root=$(cd "$skill_dir/../.." && pwd)
upstream="$repo_root/vendor/OpenMontage"

if [ ! -d "$upstream" ]; then
  echo "缺少上游仓库：git clone https://github.com/calesthio/OpenMontage.git vendor/OpenMontage" >&2
  exit 1
fi

link() {
  local source="$1"
  local target="$2"
  mkdir -p "$(dirname "$target")"
  if [ -L "$target" ] || [ -e "$target" ]; then
    rm "$target"
  fi
  ln -s "$source" "$target"
}

link "$skill_dir/provider_extensions/tools/video/ofox_video.py" "$upstream/tools/video/ofox_video.py"
link "$skill_dir/pipeline_defs/short-drama.yaml" "$upstream/pipeline_defs/short-drama.yaml"
link "$skill_dir/provider_extensions/skills/pipelines/short-drama" "$upstream/skills/pipelines/short-drama"

echo "OpenMontage 扩展链接已更新：$upstream"

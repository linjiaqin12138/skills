#!/usr/bin/env bash
# 下载 M6 调度器所需策略权重：sitstand / ground_pick / roulade / kick_left。
# 文件名已对照 HF API（models/pollen-robotics/microduck-policies/tree/v5）确认。
# 直连 huggingface 可能超时；走代理（同 fetch-m3.sh）。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROXY="${https_proxy:-${HTTPS_PROXY:-http://localhost:7890}}"
export http_proxy="$PROXY" https_proxy="$PROXY" HTTP_PROXY="$PROXY" HTTPS_PROXY="$PROXY"

curl_retry() {
  # -C -：代理中途断流时可续传。
  curl -fL --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 30 -C - "$@"
}

POLICY_DIR="$ROOT/policies"
mkdir -p "$POLICY_DIR"

BASE="https://huggingface.co/pollen-robotics/microduck-policies/resolve/v5"
for name in alpha_sitstand.onnx alpha_ground_pick.onnx roulade.onnx ball_kick_left.onnx; do
  FILE="$POLICY_DIR/$name"
  if [[ -s "$FILE" ]]; then
    echo "skip: $FILE already present ($(wc -c < "$FILE") bytes)"
  else
    echo "fetching $name ..."
    curl_retry -o "$FILE" "$BASE/$name"
    echo "wrote $FILE ($(wc -c < "$FILE") bytes)"
  fi
done

echo "fetch-m6 done."

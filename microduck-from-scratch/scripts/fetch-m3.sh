#!/usr/bin/env bash
# 下载 M3 所需：velstand.onnx + Linux x64 ONNX Runtime (>=1.23)。
# 直连 huggingface/github 可能超时；走 WSL→Windows 代理。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROXY="${https_proxy:-${HTTPS_PROXY:-http://localhost:7890}}"
export http_proxy="$PROXY" https_proxy="$PROXY" HTTP_PROXY="$PROXY" HTTPS_PROXY="$PROXY"

curl_retry() {
  # -C -：代理中途断流时可续传（ORT tgz 曾卡在 92%）。
  curl -fL --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 30 -C - "$@"
}

# --- velstand.onnx ---
POLICY_DIR="$ROOT/policies"
POLICY_FILE="$POLICY_DIR/velstand.onnx"
mkdir -p "$POLICY_DIR"
if [[ -s "$POLICY_FILE" ]]; then
  echo "skip: $POLICY_FILE already present ($(wc -c < "$POLICY_FILE") bytes)"
else
  echo "fetching velstand.onnx ..."
  curl_retry -o "$POLICY_FILE" \
    "https://huggingface.co/pollen-robotics/microduck-policies/resolve/v5/velstand.onnx"
  echo "wrote $POLICY_FILE ($(wc -c < "$POLICY_FILE") bytes)"
fi

# --- onnxruntime (linux x64) ---
ORT_VER="1.23.2"
ORT_TGZ="onnxruntime-linux-x64-${ORT_VER}.tgz"
ORT_URL="https://github.com/microsoft/onnxruntime/releases/download/v${ORT_VER}/${ORT_TGZ}"
ORT_ROOT="$ROOT/third_party/onnxruntime"
ORT_SO="$ORT_ROOT/lib/libonnxruntime.so"

if [[ -e "$ORT_SO" ]] || [[ -L "$ORT_SO" ]]; then
  echo "skip: $ORT_SO already present"
else
  echo "fetching $ORT_TGZ ..."
  TMP="$(mktemp -d)"
  trap 'rm -rf "$TMP"' EXIT
  curl_retry -o "$TMP/$ORT_TGZ" "$ORT_URL"
  mkdir -p "$ORT_ROOT"
  tar -xzf "$TMP/$ORT_TGZ" -C "$TMP"
  SRC="$TMP/onnxruntime-linux-x64-${ORT_VER}"
  # 摊平成 third_party/onnxruntime/{lib,include,...}，方便 ORT_DYLIB_PATH 固定。
  cp -a "$SRC"/. "$ORT_ROOT"/
  # 保证能找到不带版本号的 so 名（ort load-dynamic 默认找它）。
  if [[ ! -e "$ORT_SO" ]]; then
    REAL="$(find "$ORT_ROOT/lib" -maxdepth 1 -name 'libonnxruntime.so*' | head -n1 || true)"
    if [[ -n "$REAL" ]]; then
      ln -sfn "$(basename "$REAL")" "$ORT_SO"
    else
      echo "error: no libonnxruntime.so* under $ORT_ROOT/lib" >&2
      exit 1
    fi
  fi
  echo "installed ONNX Runtime $ORT_VER -> $ORT_SO"
  rm -rf "$TMP"
  trap - EXIT
fi

echo "fetch-m3 done."
echo "  policy: $POLICY_FILE"
echo "  runtime: $ORT_SO"
echo "  export ORT_DYLIB_PATH=$ORT_SO"

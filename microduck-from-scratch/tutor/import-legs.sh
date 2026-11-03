#!/usr/bin/env bash
# 逐个导入腿部零件，结果追加到 exercises/m11b-legs/import-log.jsonl
set -uo pipefail
ROOT=/home/jqlin/Project/skills/microduck-from-scratch
LOG="$ROOT/exercises/m11b-legs/import-log.jsonl"
mkdir -p "$ROOT/exercises/m11b-legs"
touch "$LOG"

for f in "$ROOT"/vendor/microduck-ft-v2_2-legs/*.SLDPRT; do
  name=$(basename "$f")
  if grep -F "\"file\":\"$name\"" "$LOG" | grep -q '"status":"imported"'; then
    echo "skip $name"
    continue
  fi
  echo "=== $name ==="
  out=$(bash "$ROOT/tutor/import-one.sh" "$f" 2>&1 | tail -2)
  echo "$out"
  if [[ "$out" == *"imported $name "* ]]; then
    url=$(echo "$out" | grep -o 'https://cad.onshape.com[^ ]*')
    echo "{\"file\":\"$name\",\"status\":\"imported\",\"url\":\"$url\"}" >> "$LOG"
  else
    echo "{\"file\":\"$name\",\"status\":\"failed\",\"detail\":$(printf '%s' "$out" | python3 -c 'import json,sys;print(json.dumps(sys.stdin.read()))')}" >> "$LOG"
  fi
done
echo "batch done"

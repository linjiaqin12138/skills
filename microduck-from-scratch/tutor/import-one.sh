#!/usr/bin/env bash
# 用法: bash tutor/import-one.sh <SLDPRT 绝对路径>
set -uo pipefail
ROOT=/home/jqlin/Project/skills/microduck-from-scratch
AB="/home/jqlin/Project/skills/bin/agent-browser --cdp 9222"
f="$1"
name=$(basename "$f")
stem="${name%.SLDPRT}"

close_dialogs() {
  $AB eval "(()=>{let n=0;for(const b of Array.from(document.querySelectorAll('button'))){const t=b.textContent.trim();if((t==='Cancel'||t==='Close')&&(b.offsetWidth||b.offsetHeight)){b.click();n++}}return n})()" >/dev/null 2>&1
}

$AB tab t1 >/dev/null 2>&1
close_dialogs
sleep 1
$AB find text "Create" click >/dev/null 2>&1
$AB find text "Import files" click >/dev/null 2>&1
sleep 1
if ! $AB upload '#upload-file' "$f" >/dev/null 2>&1; then
  echo "upload-failed $name"; exit 1
fi
sleep 3
$AB find text "Import to a single document" click >/dev/null 2>&1
$AB eval "(()=>{const bs=Array.from(document.querySelectorAll('button')).filter(b=>b.textContent.trim()==='Import'&&(b.offsetWidth||b.offsetHeight));if(!bs.length)return 'no-button';bs[bs.length-1].click();return 'clicked'})()"
for i in $(seq 1 24); do
  sleep 5
  url=$($AB eval "Array.from(document.querySelectorAll('a[href*=\"/documents/\"]')).map(a=>[a.textContent.trim(),a.getAttribute('href')]).find(x=>x[0].startsWith('$stem'))?.[1]||''" 2>/dev/null | tr -d '"')
  if [ -n "$url" ]; then
    echo "imported $name https://cad.onshape.com$url"
    exit 0
  fi
done
echo "timeout $name"
exit 2

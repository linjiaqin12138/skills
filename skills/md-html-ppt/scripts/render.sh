#!/usr/bin/env bash
# Render Marp deck.md → deck.html, then inject Mermaid so <div class="mermaid"> works offline-ish (CDN).
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <deck.md> [out.html]" >&2
  exit 2
fi

in="$1"
out="${2:-${in%.md}.html}"
dir="$(cd "$(dirname "$in")" && pwd)"
base="$(basename "$in")"
out_abs="$(cd "$(dirname "$out")" 2>/dev/null && pwd)/$(basename "$out")"
mkdir -p "$(dirname "$out_abs")"

if command -v marp >/dev/null 2>&1; then
  MARP=(marp)
elif [[ -x "$HOME/.local/node/bin/marp" ]]; then
  MARP=("$HOME/.local/node/bin/marp")
else
  MARP=(npx --yes @marp-team/marp-cli)
fi

"${MARP[@]}" "$dir/$base" -o "$out_abs" --html --allow-local-files

# Inject Mermaid once (idempotent)
if ! grep -q 'data-md-html-ppt-mermaid' "$out_abs"; then
  tmp="$(mktemp)"
  # shellcheck disable=SC2016
  python3 - "$out_abs" "$tmp" <<'PY'
import pathlib, sys
src, dst = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2])
html = src.read_text(encoding="utf-8")
snippet = """
<script data-md-html-ppt-mermaid type="module">
import mermaid from "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs";
mermaid.initialize({ startOnLoad: false, securityLevel: "loose", theme: "neutral" });
const run = () => mermaid.run({ querySelector: "div.mermaid, pre.mermaid, code.language-mermaid" });
if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", run);
else run();
</script>
"""
if "</body>" in html:
    html = html.replace("</body>", snippet + "</body>", 1)
else:
    html += snippet
dst.write_text(html, encoding="utf-8")
PY
  mv "$tmp" "$out_abs"
fi

echo "$out_abs"

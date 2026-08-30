# -*- coding: utf-8 -*-
"""Convert 独立开发者成功模式深度拆解.md to print-ready HTML (A4, Paged.js)."""
import re, html, sys
from markdown_it import MarkdownIt

SRC = "/mnt/agents/output/独立开发者成功模式深度拆解.md"
OUT = "/mnt/agents/output/独立开发者成功模式深度拆解.html"

raw = open(SRC, encoding="utf-8").read()

# ---------- split body / references ----------
m = re.search(r"^# 参考文献\s*$", raw, re.M)
assert m, "references heading not found"
body_md = raw[: m.start()]
refs_md = raw[m.end():]

# ---------- parse references ----------
refs = {}
for line in refs_md.splitlines():
    line = line.strip()
    rm = re.match(r"^\[(\d+)\]\s+(.*)$", line)
    if rm:
        refs[int(rm.group(1))] = rm.group(2)
assert len(refs) == 401, f"expected 401 refs, got {len(refs)}"
assert set(refs) == set(range(1, 402)), "ref ids not 1..401"

def render_ref(text):
    text = html.escape(text)
    # linkify trailing URL
    def linkify(mm):
        url = mm.group(0)
        return f'<a href="{url}">{url}</a>'
    text = re.sub(r"https?://[^\s<>]+", linkify, text)
    return text

ref_items = "\n".join(
    f'<li id="ref-{n}">[{n}] {render_ref(refs[n])}</li>' for n in range(1, 402)
)

# ---------- drop the leading title block (cover carries it) ----------
# everything before the first "## " chapter heading
first_h2 = re.search(r"^## ", body_md, re.M)
body_md = body_md[first_h2.start():]

# ---------- citation markers -> superscript links ----------
cite_ids = set(int(n) for n in re.findall(r"\[\^(\d+)\^\]", body_md))
missing = sorted(c for c in cite_ids if c not in refs)
assert not missing, f"citation ids without refs: {missing}"
body_md = re.sub(r"\[\^(\d+)\^\]",
                 r'<a class="cite" href="#ref-\1">[\1]</a>', body_md)

# ---------- figures: group image with adjacent caption/title ----------
lines = body_md.splitlines()
out_lines = []
fig_no = 0
i = 0
pending_fig_title = None  # bold 图 title line waiting for its image
while i < len(lines):
    line = lines[i]
    # bold figure-title line directly before an image (with blank lines between)
    mt = re.match(r"^\*\*(图[^*]+)\*\*\s*$", line)
    if mt:
        # look ahead: only blank lines then an image line?
        j = i + 1
        while j < len(lines) and not lines[j].strip():
            j += 1
        if j < len(lines) and re.match(r"^!\[.*\]\(.*\)\s*$", lines[j]):
            pending_fig_title = mt.group(1).strip()
            i += 1
            continue
    mi = re.match(r"^!\[(.*)\]\(([^)]+)\)\s*$", line)
    if mi:
        alt, src = mi.group(1), mi.group(2)
        fig_no += 1
        # caption: next non-empty line
        j = i + 1
        while j < len(lines) and not lines[j].strip():
            j += 1
        caption = None
        note = None
        consumed = j - 1  # blank lines after image are skipped anyway
        if j < len(lines):
            mc = re.match(r"^\*(.+)\*\s*$", lines[j])
            if mc and lines[j].strip().startswith("*图"):
                caption = mc.group(1).strip()
                consumed = j
            elif lines[j].strip().startswith("图注："):
                note = lines[j].strip()
                consumed = j
        title = pending_fig_title
        pending_fig_title = None
        if title is None and caption is None and alt.strip():
            title = alt.strip()
        cap_parts = []
        if title:
            cap_parts.append(f"<strong>{html.escape(title)}</strong>")
        if caption:
            cap_parts.append(caption)  # already contains cite links
        if note:
            cap_parts.append(note if "<a " in note else html.escape(note))
        fig = [f'<figure id="fig-{fig_no}">',
               f'<img src="{html.escape(src)}" alt="{html.escape(alt)}">']
        if cap_parts:
            fig.append("<figcaption>" + " ".join(cap_parts) + "</figcaption>")
        fig.append("</figure>")
        out_lines.append("\n".join(fig))
        i = consumed + 1
        continue
    out_lines.append(line)
    i += 1
assert fig_no == 15, f"expected 15 figures, got {fig_no}"
body_md = "\n".join(out_lines)

# ---------- table captions: **表 ...** whole-line -> placeholder ----------
# (merged into <table><caption> after markdown rendering so the caption is
#  part of the table box and can never be orphaned/clipped at a page break)
new_lines = []
for line in body_md.splitlines():
    mt = re.match(r"^\*\*(表.+)\*\*\s*$", line)
    if mt:
        new_lines.append(f'<!--TABLECAPTION:{mt.group(1)}-->')
    else:
        new_lines.append(line)
body_md = "\n".join(new_lines)

# ---------- inline bold: pre-convert to <strong> ----------
# CommonMark delimiter flanking rules fail on CJK punctuation (e.g. "。**这"),
# leaving literal "**" in output; convert all remaining pairs explicitly.
body_md = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", body_md)
assert "**" not in body_md, "unbalanced ** markers remain"

# ---------- headings -> explicit HTML with ids; collect TOC ----------
toc = []  # (level, id, text)
ch_no = 0
sub_no = 0
new_lines = []
for line in body_md.splitlines():
    mh = re.match(r"^(#{2,4})\s+(.*)$", line)
    if mh:
        level = len(mh.group(1))
        text = mh.group(2).strip()
        plain = re.sub(r"<[^>]+>", "", text)
        plain = html.escape(plain)
        clsattr = ""
        if level == 2:
            ch_no += 1
            sub_no = 0
            hid = f"ch{ch_no}"
            toc.append((2, hid, plain))
            if ch_no == 1:
                clsattr = ' class="first-chapter"'
        elif level == 3:
            sub_no += 1
            hid = f"ch{ch_no}-{sub_no}"
            toc.append((3, hid, plain))
        else:
            hid = ""
        idattr = f' id="{hid}"' if hid else ""
        new_lines.append(f"<h{level}{idattr}{clsattr}>{text}</h{level}>")
    else:
        new_lines.append(line)
body_md = "\n".join(new_lines)
assert ch_no == 11, f"expected 11 chapters, got {ch_no}"

# ---------- markdown -> html ----------
md = MarkdownIt("default", {"html": True}).enable("table")
body_html = md.render(body_md)

# merge caption placeholders into following <table>
def _merge_caption(m):
    cap = m.group(1)
    # 表 10-2: Paged.js/Chromium print drops the first body row when this
    # table's fragment starts near a page bottom; force a fresh page start.
    cls = ' class="force-break"' if "表 10-2" in cap else ""
    return f'<table{cls}><caption class="table-caption">{cap}</caption>'
body_html = re.sub(
    r"<!--TABLECAPTION:(.*?)-->\s*<table>",
    _merge_caption,
    body_html)
leftover = body_html.count("<!--TABLECAPTION:")
assert leftover == 0, f"{leftover} table captions failed to merge"

# ---------- TOC html ----------
toc_rows = []
for level, hid, text in toc:
    cls = "toc-l1" if level == 2 else "toc-l2"
    toc_rows.append(
        f'<li class="{cls}"><a href="#{hid}"><span class="t">{text}</span>'
        f'<span class="dots"></span></a></li>')
toc_html = "\n".join(toc_rows)

CSS = """
@page {
    size: A4;
    margin: 2.5cm 2cm;
    @top-center { content: string(doctitle); font-size: 9pt; color: #8B7355; font-family: "Noto Sans SC","Noto Sans CJK SC",sans-serif; }
    @bottom-center { content: counter(page); font-size: 9pt; color: #666; }
}
@page :first { margin: 0; @top-center { content: none; } @bottom-center { content: none; } }
@page cover { margin: 0; @top-center { content: none; } @bottom-center { content: none; } }
@page toc { @top-center { content: none; } @bottom-center { content: none; } }

html { -webkit-print-color-adjust: exact; }
body {
    margin: 0; padding: 0;
    font-family: "Noto Serif SC","Noto Serif CJK SC",Georgia,serif;
    font-size: 11pt; line-height: 1.6; color: #333;
    text-align: justify; text-align-last: left;
    string-set: doctitle "";
    orphans: 2; widows: 2;
}

/* ---------- cover ---------- */
.cover {
    page: cover;
    width: 210mm; height: 297mm; margin: 0;
    position: relative; overflow: hidden;
    page-break-after: always;
    background: #F5F1EA;
}
.cover .frame {
    position: absolute; top: 12mm; left: 12mm; right: 12mm; bottom: 12mm;
    border: 0.6mm solid #8B7355;
}
.cover .frame-inner {
    position: absolute; top: 16mm; left: 16mm; right: 16mm; bottom: 16mm;
    border: 0.2mm solid #C4B7A6;
}
.cover .corner { position: absolute; width: 14mm; height: 14mm; background: #8B7355; }
.cover .corner-tl { top: 12mm; left: 12mm; }
.cover .corner-br { bottom: 12mm; right: 12mm; }
.cover .corner2 { position: absolute; width: 8mm; height: 8mm; background: #C4B7A6; }
.cover .corner-tr { top: 12mm; right: 12mm; }
.cover .corner-bl { bottom: 12mm; left: 12mm; }
.cover .block-top {
    position: absolute; top: 40mm; left: 50%; transform: translateX(-50%);
    width: 120mm; height: 2.2mm; background: #C4B7A6;
}
.cover .block-bottom {
    position: absolute; bottom: 52mm; left: 50%; transform: translateX(-50%);
    width: 60mm; height: 1.2mm; background: #8B7355;
}
.cover-content {
    position: absolute; top: 44%; left: 50%;
    transform: translate(-50%, -50%);
    width: 150mm; text-align: center; text-align-last: center;
}
.cover-kicker {
    font-family: "Noto Sans SC","Noto Sans CJK SC",sans-serif;
    font-size: 11pt; letter-spacing: 0.5em; color: #8B7355;
    margin: 0 0 10mm 0; text-indent: 0.5em;
}
.cover-title {
    font-family: "Noto Serif SC","Noto Serif CJK SC",Georgia,serif;
    font-size: 30pt; font-weight: 700; color: #4a3f30;
    margin: 0 0 6mm 0; line-height: 1.35;
    string-set: doctitle content();
}
.cover-subtitle {
    font-family: "Noto Sans SC","Noto Sans CJK SC",sans-serif;
    font-size: 13pt; color: #6d5f4b; margin: 0 0 4mm 0;
}
.cover-tagline { font-size: 10.5pt; color: #8a7d68; margin: 0; }
.cover-meta { margin-top: 22mm; }
.cover-meta .date {
    font-family: "Noto Sans SC","Noto Sans CJK SC",sans-serif;
    font-size: 12pt; color: #4a3f30; letter-spacing: 0.2em; margin: 0;
    text-align: center; text-align-last: center;
}

/* ---------- toc ---------- */
.toc-page { page: toc; page-break-after: always; }
.toc-page { text-align: left; text-align-last: left; }
.toc-page h1 {
    font-family: "Noto Sans SC","Noto Sans CJK SC",sans-serif;
    font-size: 18pt; color: #4a3f30; text-align: center; text-align-last: center;
    margin: 0 0 0.7em 0; padding-bottom: 0.3em;
    border-bottom: 2px solid #8B7355;
}
.toc-page ul { list-style: none; margin: 0; padding: 0; column-count: 2; column-gap: 1.3em; }
.toc-page li { margin: 0; break-inside: avoid; }
.toc-page a {
    text-decoration: none; color: #333;
    display: flex; align-items: baseline;
    word-break: normal;
}
.toc-page a .t { flex: 0 1 auto; }
.toc-page a .dots {
    flex: 1 1 auto; min-width: 1.2em; margin: 0 0.35em;
    border-bottom: 1px dotted #b0a48e;
}
.toc-page a::after { content: target-counter(attr(href url), page); color: #666; flex: 0 0 auto; }
.toc-l1 { font-weight: 700; font-size: 9.5pt; margin-top: 0.4em; line-height: 1.3; }
.toc-l1 a { font-weight: 700; }
.toc-l1 a::after { font-weight: 700; color: #4a3f30; }
.toc-l2 { font-size: 8pt; line-height: 1.28; padding-left: 1.1em; margin-top: 0.08em; }
.toc-l2 a { color: #444; }

/* ---------- headings ---------- */
h2 {
    page-break-before: always;
    font-family: "Noto Sans SC","Noto Sans CJK SC",sans-serif;
    font-size: 18pt; color: #4a3f30; line-height: 1.4;
    margin: 0 0 0.8em 0; padding-bottom: 0.35em;
    border-bottom: 1.5px solid #8B7355;
    page-break-after: avoid;
    text-align: left; text-align-last: left;
}
h2.first-chapter { page-break-before: auto; }
h3 {
    font-family: "Noto Sans SC","Noto Sans CJK SC",sans-serif;
    font-size: 14pt; color: #5a4c38;
    margin: 1.4em 0 0.6em 0; page-break-after: avoid;
    text-align: left; text-align-last: left;
}
h4 {
    font-family: "Noto Sans SC","Noto Sans CJK SC",sans-serif;
    font-size: 12pt; color: #333;
    margin: 1.1em 0 0.5em 0; page-break-after: avoid;
    text-align: left; text-align-last: left;
}
p { margin: 0.55em 0; }
hr { display: none; }
blockquote {
    margin: 0.8em 0; padding: 0.4em 1em;
    border-left: 3px solid #C4B7A6; background: #faf8f4; color: #555;
}
strong { color: #222; }

/* ---------- citations ---------- */
a.cite { color: #000; text-decoration: none; vertical-align: super; font-size: 0.72em; line-height: 0; }

/* ---------- tables ---------- */
table {
    border-collapse: collapse; width: 100%; max-width: 100%;
    margin: 0.4em 0 1em 0; font-size: 9.5pt; line-height: 1.45;
    border-top: 1.8px solid #333; border-bottom: 1.8px solid #333;
}
thead { display: table-header-group; }
thead th {
    border-bottom: 1px solid #333; padding: 4px 6px;
    text-align: left; font-family: "Noto Sans SC","Noto Sans CJK SC",sans-serif;
    font-size: 9.5pt; background: #faf8f4;
}
tbody td { padding: 4px 6px; border-bottom: 0.4px solid #ddd; vertical-align: top; }
tbody tr:last-child td { border-bottom: none; }
tr { page-break-inside: avoid; }
table.force-break { page-break-before: always; }
caption.table-caption {
    caption-side: top; font-size: 10pt; font-weight: 700;
    text-align: center; text-align-last: center;
    padding: 0.2em 0 0.4em 0; color: #222;
}

/* ---------- figures ---------- */
figure {
    margin: 1em auto; text-align: center; text-align-last: center;
    page-break-inside: avoid; max-width: 100%;
}
/* fig-15 hits a Paged.js boundary-drop bug (unbreakable figure exactly at
   page end gets lost); forcing it to a fresh page is deterministic. */
#fig-15 { page-break-before: always; }
figure img { max-width: 85%; max-height: 40vh; height: auto; }
figcaption {
    font-size: 9pt; color: #555; line-height: 1.45;
    margin-top: 0.4em; text-align: center; text-align-last: center;
}

/* ---------- references ---------- */
.references-section { page-break-before: always; }
.references-section h2 { page-break-before: auto; }
ul.references {
    list-style: none; margin: 0; padding: 0;
    column-count: 2; column-gap: 1.5em;
    font-size: 9pt; line-height: 1.45;
}
ul.references li {
    padding-left: 2em; text-indent: -2em;
    margin-bottom: 0.3em; word-break: break-all;
    text-align: left; text-align-last: left;
    -webkit-column-break-inside: avoid; break-inside: avoid;
}
ul.references a { color: #333; text-decoration: none; word-break: break-all; }

/* ---------- overflow guards ---------- */
pre, table, figure, img, svg, blockquote { max-width: 100%; box-sizing: border-box; }
a { word-break: break-all; }
code { word-break: break-word; font-family: "Noto Sans Mono", monospace; font-size: 0.9em; }
"""

COVER = """
<div class="cover">
  <div class="frame"></div>
  <div class="frame-inner"></div>
  <div class="corner corner-tl"></div>
  <div class="corner corner-br"></div>
  <div class="corner2 corner-tr"></div>
  <div class="corner2 corner-bl"></div>
  <div class="block-top"></div>
  <div class="block-bottom"></div>
  <div class="cover-content">
    <p class="cover-kicker">深度研究报告</p>
    <h1 class="cover-title">独立开发者成功模式深度拆解</h1>
    <p class="cover-subtitle">8种成功模式 × 15个真实案例 × 401个信息来源</p>
    <p class="cover-tagline">一人公司的黄金时代与幸存者偏差：按分发引擎拆解独立开发的胜负手</p>
    <div class="cover-meta">
      <p class="date">2026年8月26日</p>
    </div>
  </div>
</div>
"""

TOC_PAGE = f"""
<div class="toc-page">
  <h1>目　录</h1>
  <ul>
{toc_html}
    <li class="toc-l1"><a href="#references"><span class="t">参考文献</span><span class="dots"></span></a></li>
  </ul>
</div>
"""

REFS_SECTION = f"""
<div class="references-section">
<h2 id="references">参考文献</h2>
<ul class="references">
{ref_items}
</ul>
</div>
"""

html_doc = f"""<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8">
<title>独立开发者成功模式深度拆解</title>
<link href="https://fonts.googleapis.com/css2?family=Noto+Serif+SC:wght@400;700&family=Noto+Sans+SC:wght@400;700&display=swap" rel="stylesheet">
<style>{CSS}</style>
</head>
<body>
{COVER}
{TOC_PAGE}
<div class="content">
{body_html}
</div>
{REFS_SECTION}
</body>
</html>
"""

with open(OUT, "w", encoding="utf-8") as f:
    f.write(html_doc)
print(f"OK: {OUT}")
print(f"chapters={ch_no}, toc_entries={len(toc)}, figures={fig_no}, refs={len(refs)}, cites_used={len(cite_ids)}")

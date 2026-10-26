#!/usr/bin/env python3
import argparse
import html
import json
import unicodedata
from pathlib import Path


def esc(value):
    return html.escape(str(value), quote=True)


def label_width(label):
    """白底宽度按字符实际占宽估算：全角 13px，其余 6.6px。"""
    units = 0.0
    for character in label:
        if unicodedata.east_asian_width(character) in {"W", "F"}:
            units += 13.0
        else:
            units += 6.6
    return max(36.0, units + 12)


def path_d(points):
    return "M " + " L ".join(f"{x} {y}" for x, y in points)


def render(data, output, diff=False):
    width, height = data["width"], data["height"]
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<style>text{font-family:"Noto Sans CJK SC","Noto Sans",Arial,sans-serif}.title{font-size:26px;font-weight:700;fill:#0F172A}.subtitle{font-size:15px;fill:#475569}.container-label{font-size:16px;font-weight:700;fill:#334155}.node-label{font-size:16px;font-weight:700}.node-line{font-size:13px}.edge-label{font-size:12px;fill:#334155}.legend{font-size:13px;fill:#475569}</style>',
        '<defs>',
    ]
    colors = {"gray":"#64748B","pink":"#DB2777","amber":"#D97706","light":"#CBD5E1"}
    for name, color in colors.items():
        out.append(f'<marker id="arrow-{name}" markerWidth="9" markerHeight="7" refX="8" refY="3.5" orient="auto"><polygon points="0 0,9 3.5,0 7" fill="{color}"/></marker>')
    out += ['</defs>', f'<rect width="{width}" height="{height}" fill="#FFFFFF"/>',
            f'<text class="title" x="40" y="45">{esc(data["title"])}</text>',
            f'<text class="subtitle" x="40" y="75">{esc(data["subtitle"])}</text>']

    for c in data["containers"]:
        stroke = "#CBD5E1" if diff else "#94A3B8"
        out.append(f'<rect id="{esc(c["id"])}" x="{c["x"]}" y="{c["y"]}" width="{c["width"]}" height="{c["height"]}" rx="12" fill="#F8FAFC" fill-opacity="0.45" stroke="{stroke}" stroke-width="1.5" data-graph-role="container" data-container-id="{esc(c["id"])}" data-graph-bounds="{c["x"]},{c["y"]},{c["x"]+c["width"]},{c["y"]+c["height"]}"/>')
        out.append(f'<text class="container-label" x="{c["x"]+16}" y="{c["y"]+25}">{esc(c["label"])}</text>')

    for e in data["edges"]:
        status = e.get("status", "unchanged")
        if diff:
            color = colors["pink"] if status == "new" else colors["amber"] if status == "changed" else colors["light"]
        else:
            color = colors["pink"] if status == "new" else colors["amber"] if status == "changed" else colors["gray"]
        marker = "pink" if color == colors["pink"] else "amber" if color == colors["amber"] else "light" if color == colors["light"] else "gray"
        dash = ' stroke-dasharray="6 4"' if e.get("dash") else ""
        bends = max(0, len(e["points"]) - 2)
        out.append(f'<path id="{esc(e["id"])}" d="{path_d(e["points"])}" fill="none" stroke="{color}" stroke-width="1.8"{dash} marker-end="url(#arrow-{marker})" data-graph-role="edge" data-edge-id="{esc(e["id"])}" data-source="{esc(e["source"])}" data-target="{esc(e["target"])}" data-edge-kind="flow" data-bends="{bends}" data-route-stretch="1"/>')

    for n in data["nodes"]:
        status = n.get("status", "unchanged")
        if diff:
            fill = "#DB2777" if status == "new" else "#F59E0B" if status == "changed" else "#E2E8F0"
            stroke = "#9D174D" if status == "new" else "#B45309" if status == "changed" else "#CBD5E1"
            color = "#FFFFFF" if status in ("new", "changed") else "#64748B"
        else:
            fill, stroke, color = n["fill"], n["stroke"], n["color"]
        x, y, w, h = n["x"], n["y"], n["width"], n["height"]
        out.append(f'<g data-graph-role="node" data-node-id="{esc(n["id"])}" data-graph-bounds="{x},{y},{x+w},{y+h}">')
        out.append(f'<rect id="{esc(n["id"])}" x="{x}" y="{y}" width="{w}" height="{h}" rx="8" fill="{fill}" stroke="{stroke}" stroke-width="1.8"/>')
        out.append(f'<text class="node-label" x="{x+w/2}" y="{y+27}" text-anchor="middle" fill="{color}">{esc(n["label"])}</text>')
        for i, line in enumerate(n["lines"]):
            out.append(f'<text class="node-line" x="{x+w/2}" y="{y+53+i*23}" text-anchor="middle" fill="{color}">{esc(line)}</text>')
        out.append('</g>')

    for e in data["edges"]:
        x, y = e["label_pos"]
        label = e["label"]
        estimate = label_width(label)
        out.append(f'<rect x="{x-estimate/2:.1f}" y="{y-13}" width="{estimate:.1f}" height="19" rx="4" fill="#FFFFFF" fill-opacity="0.94" data-graph-role="decoration" data-owner="{esc(e["id"])}"/>')
        out.append(f'<text class="edge-label" x="{x}" y="{y+1}" text-anchor="middle">{esc(label)}</text>')

    legend_y = height - 50
    if diff:
        items = [("#DB2777", "新增"), ("#F59E0B", "改动"), ("#E2E8F0", "未动（灰化）")]
    else:
        items = data.get("legend") or [
            ("#64748B", "数据 / 控制"),
            ("#DB2777", "M9c 新增"),
            ("#94A3B8", "虚线：异步 / 进程控制"),
        ]
    x = 50
    for color, label in items:
        out.append(f'<rect x="{x}" y="{legend_y}" width="24" height="14" rx="3" fill="{color}" stroke="#94A3B8"/>')
        out.append(f'<text class="legend" x="{x+32}" y="{legend_y+12}">{esc(label)}</text>')
        x += 210
    out.append('</svg>')
    Path(output).parent.mkdir(parents=True, exist_ok=True)
    Path(output).write_text("\n".join(out) + "\n", encoding="utf-8")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input")
    parser.add_argument("output")
    parser.add_argument("--diff", action="store_true")
    args = parser.parse_args()
    render(json.loads(Path(args.input).read_text(encoding="utf-8")), args.output, args.diff)


if __name__ == "__main__":
    main()

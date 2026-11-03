"""从参考件网格切出中面轮廓，输出每条闭合环的面积/包围盒/圆心/半径。

用途：把第三方 SolidWorks 件（我们只有网格）反推成「作图步骤 + 约束清单」。
对有机外形，外轮廓通常要靠样条/圆弧描述；对孔，输出圆心与半径即可直接标尺寸。

用法：
    .venv/bin/python exercises/m11b-legs/slice_reference.py <stl> [--z 0.0]
"""

from __future__ import annotations

import argparse
import json
import math
import struct
from collections import defaultdict
from pathlib import Path


def load_triangles(path: Path):
    data = path.read_bytes()
    count = struct.unpack_from('<I', data, 80)[0]
    tris = []
    for i in range(count):
        v = struct.unpack_from('<12fH', data, 84 + 50 * i)
        tris.append((tuple(v[3:6]), tuple(v[6:9]), tuple(v[9:12])))
    return tris


AXES = {'x': 0, 'y': 1, 'z': 2}


def slice_segments(tris, level, axis='z'):
    """返回切面与三角面的交线段端点（已按 1e-4 网格焊接）。

    axis='z' 时切面平行于 XY 平面；axis='x'/'y' 时退化为切平面的那条坐标轴。
    """
    idx = AXES[axis]
    keep = [k for k in (0, 1, 2) if k != idx]
    segs = []
    for tri in tris:
        hits = []
        for i in range(3):
            p1, p2 = tri[i], tri[(i + 1) % 3]
            if (p1[idx] - level) * (p2[idx] - level) < 0:
                f = (level - p1[idx]) / (p2[idx] - p1[idx])
                pt = [p1[k] + f * (p2[k] - p1[k]) for k in range(3)]
                hits.append((pt[keep[0]], pt[keep[1]]))
        if len(hits) == 2:
            segs.append((hits[0], hits[1]))
    return segs


def weld(segments, tol=1e-4):
    """把端点焊接成节点，返回邻接表。"""
    adj = defaultdict(list)
    nodes = []

    def node(p):
        key = (round(p[0] / tol), round(p[1] / tol))
        for idx, k in enumerate(nodes):
            if k == key:
                return idx
        nodes.append(key)
        return len(nodes) - 1

    for p1, p2 in segments:
        a, b = node(p1), node(p2)
        if a == b:
            continue
        adj[a].append(b)
        adj[b].append(a)
    return adj, nodes


def trace_loops(adj):
    seen = set()
    loops = []
    for start in list(adj):
        if start in seen:
            continue
        loop, cur, prev = [], start, None
        while True:
            if cur in seen:
                break
            seen.add(cur)
            loop.append(cur)
            nxt = None
            for cand in adj[cur]:
                if cand != prev and cand not in seen:
                    nxt = cand
                    break
            if nxt is None:
                for cand in adj[cur]:
                    if cand == start:
                        nxt = start
                        break
            if nxt is None:
                break
            prev, cur = cur, nxt
        if len(loop) >= 3:
            loops.append(loop)
    return loops


def loop_stats(loop, nodes):
    pts = [(nodes[i][0] * 1e-4, nodes[i][1] * 1e-4) for i in loop]
    area = 0.0
    for i in range(len(pts)):
        x1, y1 = pts[i]
        x2, y2 = pts[(i + 1) % len(pts)]
        area += x1 * y2 - x2 * y1
    area = abs(area) / 2
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    cx = sum(xs) / len(xs)
    cy = sum(ys) / len(ys)
    r_mean = sum(math.dist(p, (cx, cy)) for p in pts) / len(pts)
    r_max = max(math.dist(p, (cx, cy)) for p in pts)
    r_min = min(math.dist(p, (cx, cy)) for p in pts)
    perim = sum(math.dist(pts[i], pts[(i + 1) % len(pts)]) for i in range(len(pts)))
    return {
        'area': area, 'centroid': (cx, cy),
        'bbox': (min(xs), min(ys), max(xs), max(ys)),
        'r_mean': r_mean, 'r_max': r_max, 'r_min': r_min,
        'perimeter': perim, 'points': len(pts),
    }


def part_extents(tris):
    xs = [v[0] for t in tris for v in t]
    ys = [v[1] for t in tris for v in t]
    zs = [v[2] for t in tris for v in t]
    return {'x': (min(xs), max(xs)), 'y': (min(ys), max(ys)), 'z': (min(zs), max(zs))}


def pick_axis(tris):
    """选最薄的方向做切面轴（垂直于最薄方向切出主轮廓）。"""
    ext = part_extents(tris)
    sizes = {k: ext[k][1] - ext[k][0] for k in ext}
    return min(sizes, key=lambda k: sizes[k]), ext


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('stl', type=Path)
    ap.add_argument('--z', type=float, default=None, help='切面高度，默认取中面')
    ap.add_argument('--axis', choices=['x', 'y', 'z'], default=None, help='切面轴，默认自动选最薄方向')
    args = ap.parse_args()

    tris = load_triangles(args.stl)
    zs = [v[2] for t in tris for v in t]
    axis, ext = pick_axis(tris)
    if args.axis:
        axis = args.axis
    lo, hi = ext[axis]
    z = args.z if args.z is not None else (lo + hi) / 2

    adj, nodes = weld(slice_segments(tris, z, axis))
    loops = [loop_stats(l, nodes) for l in trace_loops(adj)]
    loops = [s for s in loops if s['area'] > 0.05]
    loops.sort(key=lambda s: s['area'], reverse=True)

    print(f'file: {args.stl.name}')
    print(f'triangles={len(tris)}  extents x={ext["x"][1]-ext["x"][0]:.3f} '
          f'y={ext["y"][1]-ext["y"][0]:.3f} z={ext["z"][1]-ext["z"][0]:.3f}  '
          f'slice axis={axis} level={z:.4f}')
    print(f'closed loops: {len(loops)}\n')
    for i, s in enumerate(loops):
        bb = s['bbox']
        print(f'[{i:2d}] area={s["area"]:9.3f} mm^2  perimeter={s["perimeter"]:8.3f}  pts={s["points"]:4d}')
        print(f'      centroid=({s["centroid"][0]:8.3f}, {s["centroid"][1]:8.3f})  '
              f'bbox=({bb[0]:7.3f},{bb[1]:7.3f})-({bb[2]:7.3f},{bb[3]:7.3f})  '
              f'size=({bb[2]-bb[0]:6.3f} x {bb[3]-bb[1]:6.3f})  '
              f'r(mean/min/max)={s["r_mean"]:6.3f}/{s["r_min"]:6.3f}/{s["r_max"]:6.3f}')


if __name__ == '__main__':
    main()


def rdp(points, eps):
    """Ramer-Douglas-Peucker 简化闭合折线。"""
    pts = list(points)
    if len(pts) < 3:
        return pts

    def d(p, a, b):
        ax, ay = a
        bx, by = b
        px, py = p
        dx, dy = bx - ax, by - ay
        if dx == 0 and dy == 0:
            return math.dist(p, a)
        t = max(0.0, min(1.0, ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)))
        return math.dist(p, (ax + t * dx, ay + t * dy))

    stack = [(0, len(pts) - 1)]
    keep = {0, len(pts) - 1}
    while stack:
        s, e = stack.pop()
        if e <= s + 1:
            continue
        idx = max(range(s + 1, e), key=lambda i: d(pts[i], pts[s], pts[e]))
        if d(pts[idx], pts[s], pts[e]) > eps:
            keep.add(idx)
            stack.append((s, idx))
            stack.append((idx, e))
    return [pts[i] for i in sorted(keep)]


def dump_simplified(path: Path, z=None, eps=0.6, out=None):
    """输出外轮廓简化锚点与内环参数，写 JSON。"""
    tris = load_triangles(path)
    axis, ext = pick_axis(tris)
    lo, hi = ext[axis]
    zz = z if z is not None else (lo + hi) / 2
    adj, nodes = weld(slice_segments(tris, zz, axis))
    loops = trace_loops(adj)
    stats = []
    for loop in loops:
        pts = [(nodes[i][0] * 1e-4, nodes[i][1] * 1e-4) for i in loop]
        s = loop_stats(loop, nodes)
        if s['area'] <= 0.05:
            continue
        s['points_xy'] = pts
        stats.append(s)
    stats.sort(key=lambda s: s['area'], reverse=True)
    outer = stats[0]
    anchors = rdp(outer['points_xy'], eps)
    result = {
        'file': path.name,
        'slice_axis': axis,
        'slice_level': zz,
        'extents': {k: [round(v[0], 3), round(v[1], 3)] for k, v in ext.items()},
        'outer': {
            'area': outer['area'], 'perimeter': outer['perimeter'],
            'bbox': outer['bbox'], 'anchor_count': len(anchors),
            'anchors': [[round(x, 3), round(y, 3)] for x, y in anchors],
        },
        'inner_loops': [
            {k: (list(v) if isinstance(v, tuple) else v) for k, v in s.items() if k != 'points_xy'}
            for s in stats[1:]
        ],
    }
    if out:
        Path(out).write_text(json.dumps(result, ensure_ascii=False, indent=2))
    return result

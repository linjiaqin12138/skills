#!/usr/bin/env bash
# 在 microduck-from-scratch/ 仓库根执行。
# 预期：校准板 STL 为毫米单位、40×30×5mm、封闭网格；3MF 与 bgcode 已保存。
set -euo pipefail
cd "$(dirname "$0")/.."
python3 - <<'PY'
from pathlib import Path
import struct
from collections import Counter
root = Path('exercises/m11a-calibration')
for name in ('calibration-plate.stl', 'calibration-plate.3mf', 'calibration-plate.bgcode'):
    assert (root / name).is_file(), f'missing {name}'
data = (root / 'calibration-plate.stl').read_bytes()
n = struct.unpack_from('<I', data, 80)[0]
assert len(data) == 84 + 50 * n
verts = []
edges = Counter()
for i in range(n):
    values = struct.unpack_from('<12fH', data, 84 + 50 * i)
    tri = [tuple(values[j:j + 3]) for j in (3, 6, 9)]
    verts.extend(tri)
    for a, b in ((tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])):
        edges[tuple(sorted((a, b)))] += 1
size = [max(v[j] for v in verts) - min(v[j] for v in verts) for j in range(3)]
assert all(abs(a - b) < 1e-5 for a, b in zip(size, (40, 30, 5))), size
assert all(count == 2 for count in edges.values())
print(f'M11a Bite 1 PASS: 40×30×5mm; {n} triangles; watertight; STL/3MF/bgcode present')
PY

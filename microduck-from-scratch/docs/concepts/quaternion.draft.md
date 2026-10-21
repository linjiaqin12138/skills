---
template: doc
title: quaternion：把姿态装进 4 个数
subtitle: 轴角、双覆盖、复合与插值的物理直觉
---

quaternion 把「绕一根轴转一个角度」打包成 `(w, x, y, z)`。它只描述姿态；`q` 和 `-q` 是同一个姿态的两份数值。

## 一句话版本

quaternion 是轴角的四数打包格式，单位长度时只表示姿态。

```svg
<svg width="760" height="320" viewBox="0 0 760 320" xmlns="http://www.w3.org/2000/svg">
  <rect width="760" height="320" fill="#14181f"/>
  <defs>
    <marker id="quat-static-arrow" markerWidth="8" markerHeight="8" refX="6" refY="3" orient="auto">
      <path d="M0,0 L0,6 L7,3 z" fill="#8b93a3"/>
    </marker>
  </defs>
  <g transform="translate(170,150)">
    <line x1="0" y1="80" x2="0" y2="-90" stroke="#8b93a3" stroke-width="2" marker-end="url(#quat-static-arrow)"/>
    <text x="12" y="-88" fill="#c8cfd9" font-size="16">n</text>
    <path d="M-82,28 A88,48 0 0 1 72,-28" fill="none" stroke="#4f9cf7" stroke-width="2" marker-end="url(#quat-static-arrow)"/>
    <text x="-92" y="65" fill="#4f9cf7" font-size="16">θ</text>
    <g transform="rotate(-25)">
      <circle r="7" fill="#d8a018"/>
      <path d="M-48,0 L42,0 M42,0 L20,-16 M42,0 L20,16" fill="none" stroke="#d8a018" stroke-width="5" stroke-linecap="round"/>
    </g>
    <text x="-72" y="120" fill="#c8cfd9" font-size="14">轴 n + 转角 θ</text>
  </g>
  <line x1="295" y1="150" x2="415" y2="150" stroke="#8b93a3" stroke-width="2" marker-end="url(#quat-static-arrow)"/>
  <text x="337" y="136" fill="#8b93a3" font-size="14">打包</text>
  <g transform="translate(575,150)">
    <circle r="96" fill="#1b2230" stroke="#8b93a3" stroke-width="2"/>
    <ellipse rx="96" ry="32" fill="none" stroke="#3d4b60" stroke-width="1.5"/>
    <line x1="-68" y1="68" x2="68" y2="-68" stroke="#596579" stroke-width="1.5" stroke-dasharray="6 5"/>
    <circle r="4" fill="#8b93a3"/>
    <circle cx="68" cy="-68" r="8" fill="#4f9cf7"/>
    <circle cx="-68" cy="68" r="8" fill="#e25555"/>
    <text x="80" y="-72" fill="#4f9cf7" font-size="17">q</text>
    <text x="-96" y="86" fill="#e25555" font-size="17">-q</text>
    <text x="-52" y="128" fill="#c8cfd9" font-size="14">S³（投影示意）</text>
  </g>
</svg>
```

取单位转轴 `n` 和转角 `θ`，四个分量是：

```text
q = (w, x, y, z)
w = cos(θ/2)
(x, y, z) = sin(θ/2) × n
```

- `w` 记录角度，`(x, y, z)` 同时记录转轴和角度。
- `|q| = 1` 时叫单位 quaternion。
- 所有单位 quaternion 组成四维空间里的三维球面 S³。
- 上图只画点与对径点，不表示 S³ 的真实形状。
- 四个数受单位长度约束，姿态仍只有三个自由度。

```html
<details>
<summary>为什么存半角 θ/2？</summary>
<p>同轴旋转复合时，转角会相加。复数与 quaternion 的乘法也会累加角度贡献。旋转向量时，q 和 q⁻¹ 从两侧作用，角度贡献再加倍。因此 q 存半角，作用到向量上正好得到完整转角。</p>
</details>
```

## 为什么需要

欧拉角、旋转矩阵和轴角都能记录姿态，但复合或插值时各有麻烦。

| 表示 | 记录方式 | 主要麻烦 |
|---|---|---|
| 欧拉角 | 3 个角度 | 顺序敏感；会出现 gimbal lock；同一姿态可有多个角度解，插值可能绕远路 |
| 旋转矩阵 | 9 个数 | 存储冗余；逐分量插值后不再正交，需要重新正交化 |
| 轴角 | 一根轴 + 一个角度 | θ=0 时轴可任选；θ=180° 时轴正负等价；插值时轴可能跳变 |
| quaternion | 4 个数 + 单位长度 | 要处理双覆盖；其余复合、归一化和插值都有直接算法 |

quaternion 的实际好处对应四个动作：

1. **复合**：两次旋转合成一次 quaternion 乘法。
2. **存储**：归一化后，4 个数就是全部姿态状态。
3. **插值**：slerp 沿球面短弧等角插值。
4. **奇异性**：不需要欧拉角的万向节特判。

gimbal lock 不是物体被机械锁住。它表示欧拉角的两个转轴重合，三个角度只剩两个独立转向。

## q 和 -q：同一个姿态

姿态每转 360° 就重复，quaternion 每转 720° 才重复。这个二对一对应叫双覆盖。

```svg
<svg width="760" height="340" viewBox="0 0 760 340" xmlns="http://www.w3.org/2000/svg">
  <rect width="760" height="340" fill="#14181f"/>
  <text x="150" y="38" fill="#c8cfd9" font-size="15">姿态</text>
  <text x="500" y="38" fill="#c8cfd9" font-size="15">quaternion 轨迹（投影）</text>
  <g transform="translate(190,155)">
    <circle r="78" fill="none" stroke="#3d4b60" stroke-width="1.5" stroke-dasharray="5 5"/>
    <circle r="8" fill="#14181f" stroke="#8b93a3" stroke-width="2"/>
    <circle r="2.5" fill="#8b93a3"/>
    <text x="13" y="5" fill="#8b93a3" font-size="14">z</text>
    <g>
      <animateTransform attributeName="transform" type="rotate" values="0;360;720" keyTimes="0;0.5;1" dur="8s" repeatCount="indefinite"/>
      <path d="M0,0 L66,0 M66,0 L48,-13 M66,0 L48,13 M0,0 L0,-42" fill="none" stroke="#d8a018" stroke-width="5" stroke-linecap="round"/>
      <circle cx="0" cy="-42" r="5" fill="#4f9cf7"/>
    </g>
    <text x="-75" y="120" fill="#c8cfd9" font-size="14">0° ≡ 360° ≡ 720°</text>
  </g>
  <g transform="translate(565,155)">
    <circle r="82" fill="#1b2230" stroke="#8b93a3" stroke-width="2"/>
    <ellipse rx="82" ry="28" fill="none" stroke="#3d4b60" stroke-width="1.5"/>
    <circle cx="0" cy="-82" r="6" fill="#4f9cf7"/>
    <circle cx="0" cy="82" r="6" fill="#e25555"/>
    <text x="14" y="-86" fill="#4f9cf7" font-size="17">q</text>
    <text x="14" y="94" fill="#e25555" font-size="17">-q</text>
    <circle r="9" fill="#d8a018">
      <animateMotion path="M0,-82 A82,82 0 1 1 0,82 A82,82 0 1 1 0,-82" dur="8s" repeatCount="indefinite"/>
    </circle>
    <text x="-48" y="128" fill="#c8cfd9" font-size="14">q → -q → q</text>
  </g>
</svg>
```

动画前 4 秒，物体从 0° 转到 360°。姿态已经复原，quaternion 却从 `q` 走到 `-q`。后 4 秒再转一圈，quaternion 才回到 `q`。

`q` 和 `-q` 是 S³ 上的对径点。它们旋转向量的结果完全相同，不是两个姿态。

## 最小可运行示例

这个脚本只用 Python 3 的 `math`，直接打印五组可检查的结果。

完整脚本：[assets/quaternion/quat_demo.py](assets/quaternion/quat_demo.py)。在仓库根目录运行：

```bash
python3 docs/concepts/assets/quaternion/quat_demo.py
```

核心函数如下。完整脚本还包含归一化与两种插值：

```python src=docs/concepts/assets/quaternion/quat_demo.py lines=20-49
```

实测输出（Python 3.14.4）：

```text
[1] 轴角 -> quaternion -> 轴角（wxyz）
q = [0.7660, 0.2143, 0.4285, 0.4285]
axis = [0.3333, 0.6667, 0.6667] angle = 80.0000 deg

[2] q 与 -q 是同一姿态
q(0 deg)   = [1.0000, 0.0000, 0.0000, 0.0000]
q(360 deg) = [-1.0000, 0.0000, 0.0000, 0.0000]
rotate(q)   = [0.6889, 0.0552, -0.1496]
rotate(-q)  = [0.6889, 0.0552, -0.1496]
rotate(q360)= [0.3000, -0.4000, 0.5000]

[3] 复合：qz * qx 表示先转 qx，再转 qz
分两次旋转   = [1.0000, 0.0000, 1.0000]
qz * qx     = [1.0000, 0.0000, 1.0000]
反序 qx * qz = [-1.0000, -1.0000, 0.0000]

[4] 未归一化的 quaternion 连乘会漂移
连乘 100 次后的范数 = 1.105116
归一化后的范数       = 1.000000

[5] 350 deg -> 10 deg：nlerp 未校正符号，slerp 走短弧
t     nlerp角度    slerp角度
0.00    350.0000    -10.0000
0.25    340.1500     -5.0000
0.50    180.0000      0.0000
0.75     19.8500      5.0000
1.00     10.0000     10.0000
```

- 第 2 组：`q(360°) = -q(0°)`，但两者旋转向量的结果相同。
- 第 3 组：`qz * qx` 先转 `qx`，交换顺序会得到别的姿态。
- 第 4 组：连乘让范数漂移，归一化把范数恢复为 1。
- 第 5 组：nlerp 经过 180°，slerp 从 −10° 匀速走到 10°。

跨零本身不是根因。两个输入点的点积为负，表示它们不在同一半球。未校正符号的 nlerp 直接连接这两个点，所以走了 340° 的长路。

## 常见坑

先查分量顺序和符号约定，再检查插值算法。

| 坑 | 后果 | 处理 |
|---|---|---|
| `wxyz` 与 `xyzw` 混用 | 同一个数组被读成别的旋转 | 抄代码前查文档，并用已知旋转做断言 |
| 忘记 `q` 与 `-q` 等价 | slerp 或 nlerp 可能绕远路 | 点积小于 0 时，先把一个输入整体取负 |
| 把 nlerp 当成等角插值 | 路径可以最短，但转速不均匀 | 需要恒定角速度时用 slerp |
| 不归一化 | 连乘后范数漂移，旋转向量可能缩放 | 用单位 quaternion 表示姿态，连乘后归一化 |
| 复合顺序写反 | 得到另一个姿态 | 本页约定 `qz * qx` 表示先 `qx` 后 `qz` |
| 用它代替全部运动状态 | 丢失位置 | 姿态存 quaternion，位置另存三维向量 |

quaternion 不是多一个传感器，也不是多一个自由度。它只是姿态的数值表示。

## 术语表

这 8 个词都围绕「轴角表示姿态，quaternion 负责计算」。

| 术语 | 含义 |
|---|---|
| 姿态 | 物体的朝向，不包含位置 |
| 旋转轴 | 转动时保持不动的直线方向 |
| 转角 | 绕旋转轴转过的角度 |
| 单位 quaternion | 范数为 1 的 quaternion，对应 S³ 上的一个点 |
| 双覆盖 | `q` 与 `-q` 两个点对应同一个姿态 |
| slerp | 球面线性插值；校正符号后沿短弧等角插值 |
| nlerp | 分量线性插值后再归一化；不是等角插值 |
| 万向节死锁（gimbal lock） | 欧拉角的两个转轴重合，丢失一个独立转向 |

## 素材

先看 3Blue1Brown 建立整体直觉，再用 Ben Eater 拖动验证。

- [3Blue1Brown：quaternions 专题](https://www.3blue1brown.com/topics/quaternions)：视频与可视化，最推荐。
- [Ben Eater：交互式 quaternion](https://eater.net/quaternions)：拖动旋转，直接观察四个分量。
- [Wikipedia：Quaternion](https://en.wikipedia.org/wiki/Quaternion)：查定义、公式与历史。
- [Penguin Maths：How quaternions produce 3D rotation](https://penguinmaths.blogspot.com/2019/06/how-quaternions-produce-3d-rotation.html)：想再看一层数学时阅读。
- 本地自制脚本：[quat_demo.py](assets/quaternion/quat_demo.py)，零第三方依赖。

## 自测

先选出你会怎么改，再展开答案。

```html
<details>
<summary><b>1.</b> 某库按 xyzw 返回 [0, 0, 0, 1]，你的代码按 wxyz 读取。单位姿态会被误读成什么？（A）单位姿态　（B）绕 z 轴转 180°　（C）绕 x 轴转 90°　（D）无法表示旋转</summary>
<p><b>答案：B。</b>按 wxyz 读取后，w=0、z=1，所以是绕 z 轴转 180°。原数组按 xyzw 才是单位姿态。A 正是忽略分量顺序的结果。</p>
</details>
<details>
<summary><b>2.</b> 采用本页约定，要先执行 qx，再执行 qz。复合量怎么写？（A）qx * qz　（B）qz * qx　（C）qx + qz　（D）qz - qx</summary>
<p><b>答案：B。</b>右侧的 qx 先作用，左侧的 qz 后作用。脚本里 qz * qx 与分两次旋转得到同一个向量，反序则不同。</p>
</details>
<details>
<summary><b>3.</b> 350° → 10° 的 nlerp 动画绕到 180°，两个输入点的点积为负。先怎么修？（A）把角度都加 360°　（B）把一个 quaternion 整体取负再插值　（C）交换 w 和 x　（D）取消归一化</summary>
<p><b>答案：B。</b>q 与 -q 表示同一姿态。取负不改变端点姿态，却把两个输入点放到同一半球，插值就能走短弧。A 没有处理 quaternion 的符号。</p>
</details>
<details>
<summary><b>4.</b> 符号已校正，nlerp 已走短弧，但转速忽快忽慢。要恒定角速度，选什么？（A）slerp　（B）增加采样点　（C）再归一化一次　（D）改成旋转矩阵</summary>
<p><b>答案：A。</b>slerp 按球面圆弧的角度取参数，因此角速度恒定。nlerp 只保证每个采样点是单位 quaternion，不保证匀速。</p>
</details>
<details>
<summary><b>5.</b> 要同时保存物体的位置和姿态，哪组状态够？（A）只要 4 个 quaternion 分量　（B）3 个位置分量 + 4 个 quaternion 分量　（C）只要 3 个欧拉角　（D）只要 3 个位置分量</summary>
<p><b>答案：B。</b>quaternion 只记录姿态，位置要另存三维向量。单位长度是约束，不会凭空多出位置状态。</p>
</details>
```

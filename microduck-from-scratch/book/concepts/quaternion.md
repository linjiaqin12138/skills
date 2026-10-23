---
template: doc
title: 四元数：四个数怎么表示一个旋转
subtitle: 绕 z 轴的最小模型，以及 q 与 -q 为什么等价
---

四元数是四个实数按 $i^2 = j^2 = k^2 = -1$ 这套规则相乘得到的一种数，写成 $q = w + xi + yj + zk$。

它记姿态的方式是：标量部记转角的一半，向量部记转轴。

这里的姿态只描述朝向，不含位置；位置另用三维向量存。

绕 $z$ 轴转 $90^\circ$ 对应的就是 $q = (0.7071, 0, 0, 0.7071)$，作用到向量 $(1, 0, 0)$ 得到 $(0, 1, 0)$。

读完这一页你要能做三件事：

- 写出定义和 $i, j, k$ 乘法表，说清 $w$ 与 $(x, y, z)$ 各记什么。
- 手算两个四元数的乘积，以及共轭、范数、逆。
- 算出向量被转到哪里，指出 $(w, z)$ 图上的姿态点，并解释 $q$ 与 $-q$ 等价。

## A 四元数是什么

四元数是四个实数按 $i^2 = j^2 = k^2 = -1$ 这套规则相乘得到的一种数。

写成 $q = w + xi + yj + zk$，$w, x, y, z$ 都是实数。$i, j, k$ 是三个新单位，专门用来记方向。

$i, j, k$ 之间的乘法表。行是左边的因子，列是右边的因子：

| × | $i$ | $j$ | $k$ |
|---|---|---|---|
| $i$ | $-1$ | $k$ | $-j$ |
| $j$ | $-k$ | $-1$ | $i$ |
| $k$ | $j$ | $-i$ | $-1$ |

三条最常用的推论：

- 自己平方都是 $-1$：$i^2 = j^2 = k^2 = -1$。
- 顺着转一圈得正号：$ij = k$，$jk = i$，$ki = j$。
- 交换顺序得负号：$ji = -k$，$kj = -i$，$ik = -j$。

```svg
<svg width="420" height="210" viewBox="0 0 420 210" xmlns="http://www.w3.org/2000/svg" style="font-family:var(--font-sans);display:block;margin:0 auto">
  <defs>
    <marker id="qa-cycle" markerWidth="10" markerHeight="10" refX="8" refY="3.5" orient="auto"><path d="M0,0 L0,7 L8,3.5 z" fill="var(--ink-2)"/></marker>
  </defs>
  <text x="210" y="24" fill="var(--ink)" font-size="15" font-weight="600" text-anchor="middle">顺着走一圈：ij = k，jk = i，ki = j</text>

  <circle cx="80" cy="100" r="30" fill="var(--fill)" stroke="var(--ink-2)" stroke-width="1.5"/>
  <circle cx="210" cy="100" r="30" fill="var(--fill)" stroke="var(--ink-2)" stroke-width="1.5"/>
  <circle cx="340" cy="100" r="30" fill="var(--fill)" stroke="var(--ink-2)" stroke-width="1.5"/>
  <text x="80" y="109" fill="var(--ink)" font-size="24" font-style="italic" text-anchor="middle">i</text>
  <text x="210" y="109" fill="var(--ink)" font-size="24" font-style="italic" text-anchor="middle">j</text>
  <text x="340" y="109" fill="var(--ink)" font-size="24" font-style="italic" text-anchor="middle">k</text>

  <line x1="112" y1="100" x2="176" y2="100" stroke="var(--ink-2)" stroke-width="1.5" marker-end="url(#qa-cycle)"/>
  <line x1="244" y1="100" x2="306" y2="100" stroke="var(--ink-2)" stroke-width="1.5" marker-end="url(#qa-cycle)"/>
  <path d="M340,130 C340,172 80,172 80,130" fill="none" stroke="var(--ink-2)" stroke-width="1.5" marker-end="url(#qa-cycle)"/>
  <text x="144" y="84" fill="var(--ink-2)" font-size="14" text-anchor="middle">ij = k</text>
  <text x="275" y="84" fill="var(--ink-2)" font-size="14" text-anchor="middle">jk = i</text>
  <text x="210" y="196" fill="var(--ink-2)" font-size="14" text-anchor="middle">ki = j</text>
</svg>
```

图里三个圆圈是 $i$、$j$、$k$ 三个单位，带箭头的线是乘法关系。反着走要变号：$ji = -k$，$kj = -i$，$ik = -j$。

连着三个一起乘是 $-1$：$ijk = (ij)k = k \cdot k = -1$。

乘法就是分配律加这张表。例如：

$$
(1 + i)(1 + j) = 1 + j + i + ij = 1 + i + j + k
$$

展开时每一项都查一次表。乘法还满足结合律，$(ab)c = a(bc)$，所以括号可以随意加。

## B 每个量记着什么

四元数拆成两半：标量部 $w$ 和向量部 $(x, y, z)$。姿态用单位四元数记。

把向量部写成一个三维向量 $\mathbf{n} = (x, y, z)$，四元数就常写成一对数 $(w, \mathbf{n})$。它和四元组 $(w, x, y, z)$ 是同一个数的两种写法，后文为了和代码里的数组对齐，统一用四元组 $(w, x, y, z)$。

单位四元数指范数为 1 的四元数，范数怎么算见 C 面板。这时两个部分各有分工：

$$
w = \cos(\theta/2), \qquad (x, y, z) = \sin(\theta/2)\,\hat{n}
$$

- $w$ 记转角 $\theta$ 的一半。
- 向量部的方向 $\hat{n}$ 记转轴。

$\hat{n}$ 是转轴方向的单位向量，长度为 1；向量部的长度是 $\sin(\theta/2)$。

为什么是半角不是整角：F 面板把 $v' = q\,v\,q^{*}$ 展开后会出现 $w^2 - z^2$ 和 $2wz$ 这两个量，它们正好是 $\cos\theta$ 和 $\sin\theta$。存半角，展开时自动还原成整角。

## C 四元数怎么算

加法和数乘逐分量做；乘法按分配律加 $i, j, k$ 表展开；共轭把向量部取负；范数是长度；逆是共轭除以范数的平方。

加法与数乘没有别的规矩：

$$
q_1 + q_2 = (w_1 + w_2) + (x_1 + x_2)i + (y_1 + y_2)j + (z_1 + z_2)k
$$

$$
r q = (rw) + (rx)i + (ry)j + (rz)k
$$

乘法：把两个括号里的和相乘，再用分配律和 A 面板的表展开。展开完是这四条：

$$
\begin{aligned}
q_1 q_2 = &\ (w_1 w_2 - x_1 x_2 - y_1 y_2 - z_1 z_2) \\
&+ (w_1 x_2 + x_1 w_2 + y_1 z_2 - z_1 y_2)\, i \\
&+ (w_1 y_2 - x_1 z_2 + y_1 w_2 + z_1 x_2)\, j \\
&+ (w_1 z_2 + x_1 y_2 - y_1 x_2 + z_1 w_2)\, k
\end{aligned}
$$

共轭：向量部取负，标量部不动。$q^* = w - xi - yj - zk$。

范数：$|q| = \sqrt{w^2 + x^2 + y^2 + z^2}$。

逆：$q^{-1} = \dfrac{q^*}{|q|^2}$。单位四元数的逆就是共轭。

为什么这么定：把 $q$ 和 $q^*$ 相乘，交叉项两两抵消，只剩 $w^2 + x^2 + y^2 + z^2$，也就是 $q\,q^* = |q|^2$。两边再除以 $|q|^2$ 就是 1。手算一遍：

$$
(1 + i)(1 - i) = 1 - i^2 = 2 = |1 + i|^2
$$

所以 $(1 + i)^{-1} = (1 - i)/2$。

## D 一个具体旋转：绕 z 轴 90°

绕 $z$ 轴转 $90^\circ$，物体的标记点 P 从 $(1, 0, 0)$ 走到 $(0, 1, 0)$。图里画的就是这一次旋转。

```svg
<svg width="780" height="380" viewBox="0 0 780 380" xmlns="http://www.w3.org/2000/svg" style="font-family:var(--font-sans)">
  <defs>
    <marker id="qa-ink3" markerWidth="9" markerHeight="9" refX="7" refY="3" orient="auto"><path d="M0,0 L0,6 L7,3 z" fill="var(--ink-3)"/></marker>
    <marker id="qa-ink2" markerWidth="9" markerHeight="9" refX="7" refY="3" orient="auto"><path d="M0,0 L0,6 L7,3 z" fill="var(--ink-2)"/></marker>
    <marker id="qa-ink" markerWidth="9" markerHeight="9" refX="7" refY="3" orient="auto"><path d="M0,0 L0,6 L7,3 z" fill="var(--ink)"/></marker>
    <marker id="qa-accent" markerWidth="9" markerHeight="9" refX="7" refY="3" orient="auto"><path d="M0,0 L0,6 L7,3 z" fill="var(--accent)"/></marker>
    <marker id="qa-ok" markerWidth="9" markerHeight="9" refX="7" refY="3" orient="auto"><path d="M0,0 L0,6 L7,3 z" fill="var(--ok)"/></marker>
  </defs>

  <text x="55" y="30" fill="var(--ink)" font-size="15" font-weight="600">俯视图：看 xy 平面，z 轴指向屏幕外</text>
  <text x="430" y="30" fill="var(--ink)" font-size="15" font-weight="600">输入向量与输出向量</text>

  <line x1="55" y1="200" x2="300" y2="200" stroke="var(--ink-3)" stroke-width="1.5" marker-end="url(#qa-ink3)"/>
  <text x="308" y="205" fill="var(--ink-2)" font-size="15">x</text>
  <line x1="190" y1="330" x2="190" y2="70" stroke="var(--ink-3)" stroke-width="1.5" marker-end="url(#qa-ink3)"/>
  <text x="182" y="62" fill="var(--ink-2)" font-size="15" text-anchor="end">y</text>

  <polygon points="190,194 238,194 238,184 256,200 238,216 238,206 190,206" fill="var(--fill)" stroke="var(--ink-2)" stroke-width="2" stroke-linejoin="round"/>
  <polygon points="196,200 196,152 206,152 190,134 174,152 184,152 184,200" fill="var(--fill)" stroke="var(--ink)" stroke-width="2" stroke-linejoin="round"/>
  <circle cx="256" cy="200" r="6" fill="none" stroke="var(--ink-3)" stroke-width="1.5"/>
  <circle cx="190" cy="134" r="7" fill="var(--ink)" stroke="var(--bg)" stroke-width="2"/>
  <text x="200" y="128" fill="var(--ink)" font-size="15">P</text>

  <path d="M278,200 A88,88 0 0 0 190,112" fill="none" stroke="var(--ink-2)" stroke-width="1.5" stroke-dasharray="6 5" marker-end="url(#qa-ink2)"/>
  <text x="128" y="120" fill="var(--ink-2)" font-size="14" text-anchor="end">θ = 90°</text>

  <circle cx="190" cy="200" r="11" fill="var(--bg)" stroke="var(--ink-2)" stroke-width="1.5"/>
  <circle cx="190" cy="200" r="3.5" fill="var(--ink)"/>
  <text x="190" y="356" fill="var(--ink-2)" font-size="13" text-anchor="middle">⊙ = 旋转轴 +z，箭头指向屏幕外</text>

  <line x1="430" y1="200" x2="690" y2="200" stroke="var(--ink-3)" stroke-width="1.5" marker-end="url(#qa-ink3)"/>
  <text x="697" y="205" fill="var(--ink-2)" font-size="15">x</text>
  <line x1="560" y1="320" x2="560" y2="80" stroke="var(--ink-3)" stroke-width="1.5" marker-end="url(#qa-ink3)"/>
  <text x="552" y="72" fill="var(--ink-2)" font-size="15" text-anchor="end">y</text>

  <line x1="560" y1="200" x2="660" y2="200" stroke="var(--accent)" stroke-width="3" marker-end="url(#qa-accent)"/>
  <text x="618" y="222" fill="var(--accent)" font-size="14" text-anchor="middle">v = (1, 0, 0)</text>
  <text x="618" y="240" fill="var(--accent)" font-size="13" text-anchor="middle">输入</text>
  <line x1="560" y1="200" x2="560" y2="90" stroke="var(--ok)" stroke-width="3" marker-end="url(#qa-ok)"/>
  <text x="572" y="100" fill="var(--ok)" font-size="14">v' = (0, 1, 0)</text>
  <text x="572" y="118" fill="var(--ok)" font-size="13">输出</text>

  <circle cx="560" cy="200" r="10" fill="var(--bg)" stroke="var(--ink-2)" stroke-width="1.5"/>
  <circle cx="560" cy="200" r="3" fill="var(--ink)"/>
</svg>
```

左右两个面板画的是同一次旋转。左面板记物体本身怎么转，右面板只记向量被转到哪里，转的是同一个 $\theta$。

左面板（俯视图）的编码：

- 视角：俯视 $xy$ 平面，$z$ 轴指向屏幕外。
- 坐标轴 $x$、$y$：俯视平面里的两条轴。
- ⊙：旋转轴 $+z$，圆圈加一点表示箭头朝屏幕外。
- 指针：物体。同一根指针画了两份，一份在 $0^\circ$ 的位置（沿 $+x$），一份在转过 $90^\circ$ 后的位置（沿 $+y$）。
- 标记点 P：指针末端的小圆点。实心圆点标在 $90^\circ$ 位置，空心圆点标在 $0^\circ$ 位置。
- 虚线弧：从 $0^\circ$ 的位置指向 $90^\circ$ 的位置，标出旋转方向和角度 $\theta = 90^\circ$。

右面板（输入与输出向量）的编码：

- ⊙：同一个旋转轴 $+z$。原点上的圆圈加一点就是它，不是向量的起点标记。
- 水平箭头：输入向量 $v = (1, 0, 0)$，方向是 $+x$。
- 竖直箭头：输出向量 $v' = (0, 1, 0)$，方向是 $+y$。
- 两种颜色只用来把输入和输出分开，不代表别的意思。

$\theta$ 的正方向按右手定则：绕 $+z$ 从 $+x$ 转向 $+y$。俯视时 $+z$ 朝屏幕外，所以 $+\theta$ 在图上是逆时针。

## E 四个数：一个角度变两个数

固定 $z$ 轴时四个数里只有两个非零：$q = (w, 0, 0, z)$。这是 B 面板的一般写法在 $z$ 轴上的特例。

$$
w = \cos(\theta/2), \qquad z = \sin(\theta/2)
$$

角度存的是半角。$x = y = 0$ 是本页的简化，代价是这里只表示绕 $z$ 轴的旋转。

| $\theta$ | $q = (w, 0, 0, z)$ | $(w, z)$ |
|---|---|---|
| $0^\circ$ | $(1, 0, 0, 0)$ | $(1, 0)$ |
| $90^\circ$ | $(0.71, 0, 0, 0.71)$ | $(0.71, 0.71)$ |
| $180^\circ$ | $(0, 0, 0, 1)$ | $(0, 1)$ |
| $360^\circ$ | $(-1, 0, 0, 0)$ | $(-1, 0)$ |
| $720^\circ$ | $(1, 0, 0, 0)$ | $(1, 0)$ |

把 $(w, z)$ 当成平面上的一个点，它落在单位圆 $w^2 + z^2 = 1$ 上。角度 $\theta$ 变成圆上的极角 $\theta/2$。

```svg
<svg width="560" height="400" viewBox="0 0 560 400" xmlns="http://www.w3.org/2000/svg" style="font-family:var(--font-sans)">
  <defs>
    <marker id="qb-ink3" markerWidth="9" markerHeight="9" refX="7" refY="3" orient="auto"><path d="M0,0 L0,6 L7,3 z" fill="var(--ink-3)"/></marker>
  </defs>
  <text x="20" y="28" fill="var(--ink)" font-size="15" font-weight="600">(w, z) 平面：横轴 w，纵轴 z</text>

  <line x1="85" y1="215" x2="420" y2="215" stroke="var(--ink-3)" stroke-width="1.5" marker-end="url(#qb-ink3)"/>
  <text x="428" y="220" fill="var(--ink-2)" font-size="15">w</text>
  <line x1="250" y1="380" x2="250" y2="50" stroke="var(--ink-3)" stroke-width="1.5" marker-end="url(#qb-ink3)"/>
  <text x="242" y="42" fill="var(--ink-2)" font-size="15" text-anchor="end">z</text>

  <circle cx="250" cy="215" r="125" fill="none" stroke="var(--line)" stroke-width="1.5"/>

  <line x1="250" y1="215" x2="338" y2="127" stroke="var(--ink-3)" stroke-width="1" stroke-dasharray="5 4"/>
  <path d="M312,215 A62,62 0 0 0 294,171" fill="none" stroke="var(--ink-2)" stroke-width="1.5"/>
  <text x="250" y="250" fill="var(--ink-2)" font-size="14" text-anchor="middle">极角 = θ/2</text>

  <circle cx="375" cy="215" r="5" fill="none" stroke="var(--ink-2)" stroke-width="1.5"/>
  <text x="386" y="246" fill="var(--ink)" font-size="14">0° 和 720°</text>
  <text x="386" y="262" fill="var(--ink-2)" font-size="14">(1, 0)</text>

  <circle cx="338" cy="127" r="5" fill="none" stroke="var(--ink-2)" stroke-width="1.5"/>
  <text x="350" y="118" fill="var(--ink-2)" font-size="14">90°</text>
  <text x="350" y="136" fill="var(--ink-2)" font-size="14">(0.71, 0.71)</text>

  <circle cx="250" cy="90" r="5" fill="none" stroke="var(--ink-2)" stroke-width="1.5"/>
  <text x="260" y="64" fill="var(--ink-2)" font-size="14">180°</text>
  <text x="260" y="82" fill="var(--ink-2)" font-size="14">(0, 1)</text>

  <circle cx="125" cy="215" r="9" fill="none" stroke="var(--accent)" stroke-width="2" stroke-dasharray="3 3"/>
  <text x="112" y="205" fill="var(--accent)" font-size="14" text-anchor="end">360°</text>
  <text x="112" y="223" fill="var(--accent)" font-size="14" text-anchor="end">(−1, 0)</text>

  <text x="250" y="398" fill="var(--ink-2)" font-size="13" text-anchor="middle">姿态转一整圈，圆上的极角只走半圈</text>
</svg>
```

读这张图：空心小点是固定的参考位置，红色虚线圈是 $360^\circ$ 那个位置。姿态转 $180^\circ$，圆上的点只走了四分之一圈。姿态转 $360^\circ$，点正好落到 $0^\circ$ 点的对面。

G 图右面板画的就是这张 $(w, z)$ 圆，只是点会随角度移动。

## F 这四个数怎样作用到向量

只用 $w$ 和 $z$ 就能算出旋转结果。

$$
\cos\theta = w^2 - z^2, \qquad \sin\theta = 2wz
$$

$$
\begin{aligned}
x' &= x\cos\theta - y\sin\theta \\
y' &= x\sin\theta + y\cos\theta \\
z' &= z
\end{aligned}
$$

这两行是半角公式的现成结果：$\cos^2 a - \sin^2 a = \cos 2a$，$2\sin a \cos a = \sin 2a$。

怎么从 $v' = q\,v\,q^{*}$ 走到这两行：先把三维向量嵌进四元数。标量位为 0 的四元数 $(0, v_x, v_y, v_z)$ 叫纯四元数，它就是三维向量在四元数里的样子。

再用 A、C 面板的规则展开。取 $q = (w, 0, 0, z)$、纯四元数 $v = (0, v_x, v_y, 0)$，先算 $qv$：

$$
qv = (0,\; wv_x - zv_y,\; wv_y + zv_x,\; 0)
$$

再右乘共轭 $q^* = (w, 0, 0, -z)$：

$$
(qv)q^* = (0,\; (w^2 - z^2)v_x - 2wz\,v_y,\; (w^2 - z^2)v_y + 2wz\,v_x,\; 0)
$$

第一步里，$q$ 的向量部和 $v$ 相乘会多出两个含 $z$ 的项；第二步 $z$ 变号后，它们正好凑成 $-2wz\,v_y$ 和 $2wz\,v_x$。

把 $w = \cos(\theta/2)$、$z = \sin(\theta/2)$ 代进去，$w^2 - z^2$ 变成 $\cos\theta$、$2wz$ 变成 $\sin\theta$，于是就得到开头那三行。半角的用处到这里才清楚。

拿具体数真展开一遍：$q = (0.7071, 0, 0, 0.7071)$，向量 $v = (1, 0, 0)$ 嵌成纯四元数 $(0, 1, 0, 0)$。

$$
qv = (0,\ 0.7071,\ 0.7071,\ 0) \quad\longrightarrow\quad (qv)q^* = (0,\ 0,\ 1,\ 0)
$$

所以 $v' = (0, 1, 0)$，和二维公式的结果一样。

手算一遍 $\theta = 90^\circ$：

- $q = (0.7071, 0, 0, 0.7071)$。
- $\cos\theta = 0.7071^2 - 0.7071^2 = 0$，$\sin\theta = 2 \times 0.7071 \times 0.7071 = 1$。
- $v = (1, 0, 0)$ 代入：$v' = (1 \times 0 - 0 \times 1,\ 1 \times 1 + 0 \times 0,\ 0) = (0, 1, 0)$。

通用写法是 $v' = q\,v\,q^{*}$：先算 $qv$，再右乘共轭 $q^*$。脚本里的 `rotate` 就是这么算的。

下面这段可以直接抄下来跑：只用 Python 3 标准库，`q_mul`、`rotate`、`z_axis_q`、`full_angle` 四个函数都在里面，`rotate` 用到的 `q_mul` 也在同一段里。`q_mul` 就是 C 面板那条乘法公式的逐项写法。

```python src=book/concepts/figures/quaternion/quat_demo.py lines=10-41
```

## G 双覆盖：转 360° 就回到原样

姿态转 $360^\circ$ 回到原样，四个数却走到了圆上的对径点。拖滑块或点关键帧按钮，看两个面板一起动。

```html
<style>
#qd-wrap{font-family:var(--font-sans);color:var(--ink)}
#qd-wrap .qd-box{display:flex;gap:18px;flex-wrap:wrap;align-items:flex-start}
#qd-wrap svg{display:block;max-width:100%;height:auto}
#qd-wrap .qd-cap{font-size:13px;color:var(--ink-2);margin:4px 0 0}
#qd-wrap .qd-ctrl{margin-top:12px;display:flex;gap:8px;align-items:center;flex-wrap:wrap}
#qd-wrap button{font:inherit;font-size:13px;padding:2px 10px;border-radius:6px;border:1px solid var(--line);background:var(--fill);color:var(--ink);cursor:pointer}
#qd-wrap button:hover{border-color:var(--accent);color:var(--accent)}
#qd-wrap input[type=range]{flex:1 1 260px;min-width:200px}
#qd-wrap table{border-collapse:collapse;font-size:13px;margin-top:10px;font-family:var(--font-mono)}
#qd-wrap td{padding:2px 14px 2px 0;white-space:nowrap}
#qd-wrap td.k{font-family:var(--font-sans);color:var(--ink-2)}
</style>
<div id="qd-wrap">
  <div class="qd-box">
    <div>
      <svg id="qd-left" width="320" height="300" viewBox="0 0 320 300" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <marker id="qd-ink3" markerWidth="9" markerHeight="9" refX="7" refY="3" orient="auto"><path d="M0,0 L0,6 L7,3 z" fill="var(--ink-3)"/></marker>
          <marker id="qd-ink2" markerWidth="9" markerHeight="9" refX="7" refY="3" orient="auto"><path d="M0,0 L0,6 L7,3 z" fill="var(--ink-2)"/></marker>
        </defs>
        <text x="10" y="20" fill="var(--ink)" font-size="13" font-weight="600">俯视图：z 轴指向屏幕外</text>
        <line x1="30" y1="150" x2="290" y2="150" stroke="var(--ink-3)" stroke-width="1.5" marker-end="url(#qd-ink3)"/>
        <text x="296" y="155" fill="var(--ink-2)" font-size="13">x</text>
        <line x1="160" y1="260" x2="160" y2="40" stroke="var(--ink-3)" stroke-width="1.5" marker-end="url(#qd-ink3)"/>
        <text x="153" y="34" fill="var(--ink-2)" font-size="13" text-anchor="end">y</text>
        <circle cx="160" cy="150" r="10" fill="none" stroke="var(--ink-2)" stroke-width="1.5"/>
        <circle cx="160" cy="150" r="3" fill="var(--ink)"/>
        <path id="qd-arc" d="" fill="none" stroke="var(--ink-2)" stroke-width="1.5" stroke-dasharray="6 5" marker-end="url(#qd-ink2)"/>
        <g id="qd-obj">
          <polygon points="160,144 208,144 208,134 226,150 208,166 208,156 160,156" fill="var(--fill)" stroke="var(--ink)" stroke-width="2" stroke-linejoin="round"/>
          <circle cx="226" cy="150" r="6" fill="var(--ink)" stroke="var(--bg)" stroke-width="1.5"/>
          <text x="228" y="136" fill="var(--ink)" font-size="13">P</text>
        </g>
        <circle cx="160" cy="150" r="10" fill="var(--bg)" stroke="var(--ink-2)" stroke-width="1.5"/>
        <circle cx="160" cy="150" r="3" fill="var(--ink)"/>
        <text x="160" y="288" fill="var(--ink-2)" font-size="12" text-anchor="middle">⊙ = 旋转轴 +z（指向屏幕外）</text>
      </svg>
      <div class="qd-cap">物体绕 +z 轴转动，标记点 P 跟着走</div>
    </div>
    <div>
      <svg id="qd-right" width="340" height="320" viewBox="0 0 340 320" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <marker id="qd-cink3" markerWidth="9" markerHeight="9" refX="7" refY="3" orient="auto"><path d="M0,0 L0,6 L7,3 z" fill="var(--ink-3)"/></marker>
        </defs>
        <text x="10" y="20" fill="var(--ink)" font-size="13" font-weight="600">(w, z) 平面：当前四个数在这里</text>
        <line x1="45" y1="160" x2="300" y2="160" stroke="var(--ink-3)" stroke-width="1.5" marker-end="url(#qd-cink3)"/>
        <text x="306" y="165" fill="var(--ink-2)" font-size="13">w</text>
        <line x1="165" y1="285" x2="165" y2="40" stroke="var(--ink-3)" stroke-width="1.5" marker-end="url(#qd-cink3)"/>
        <text x="172" y="36" fill="var(--ink-2)" font-size="13">z</text>
        <circle cx="165" cy="160" r="100" fill="none" stroke="var(--line)" stroke-width="1.5"/>
        <line id="qd-rad" x1="165" y1="160" x2="265" y2="160" stroke="var(--ink-3)" stroke-width="1" stroke-dasharray="4 4"/>
        <circle cx="265" cy="160" r="5" fill="none" stroke="var(--ink-2)" stroke-width="1.5"/>
        <text x="278" y="148" fill="var(--ink-2)" font-size="12">0°</text>
        <circle cx="235.7" cy="89.3" r="5" fill="none" stroke="var(--ink-2)" stroke-width="1.5"/>
        <text x="250" y="70" fill="var(--ink-2)" font-size="12">90°</text>
        <circle cx="165" cy="60" r="5" fill="none" stroke="var(--ink-2)" stroke-width="1.5"/>
        <text x="140" y="64" fill="var(--ink-2)" font-size="12" text-anchor="end">180°</text>
        <circle cx="65" cy="160" r="10" fill="none" stroke="var(--accent)" stroke-width="2" stroke-dasharray="3 3"/>
        <text x="60" y="142" fill="var(--accent)" font-size="12" text-anchor="end">360°</text>
        <circle id="qd-dot" cx="265" cy="160" r="6.5" fill="var(--accent)" stroke="var(--bg)" stroke-width="2"/>
        <text x="165" y="308" fill="var(--ink-2)" font-size="12" text-anchor="middle">圆上的极角 = θ/2</text>
      </svg>
      <div class="qd-cap">同一个姿态在圆上可能有两个点</div>
    </div>
  </div>

  <div class="qd-ctrl">
    <span>姿态角 θ</span>
    <input id="qd-slider" type="range" min="0" max="720" step="1" value="0">
    <span id="qd-theta" style="font-family:var(--font-mono);min-width:52px">0°</span>
    <button type="button" data-set="0">0°</button>
    <button type="button" data-set="90">90°</button>
    <button type="button" data-set="180">180°</button>
    <button type="button" data-set="360">360°</button>
    <button type="button" data-set="720">720°</button>
    <button type="button" data-step="-1">−1°</button>
    <button type="button" data-step="1">+1°</button>
  </div>

  <table>
    <tr><td class="k">q = (w, x, y, z)</td><td id="qd-q"></td></tr>
    <tr><td class="k">−q</td><td id="qd-nq"></td></tr>
    <tr><td class="k">cosθ = w² − z²</td><td id="qd-c"></td></tr>
    <tr><td class="k">sinθ = 2 w z</td><td id="qd-s"></td></tr>
    <tr><td class="k">v = (1, 0, 0) 转成</td><td id="qd-v"></td></tr>
    <tr><td class="k">当前姿态</td><td id="qd-pose"></td></tr>
  </table>
</div>
<div id="qd-legend" style="font-size:13px;color:var(--ink-2);margin-top:10px;max-width:760px">
  <div style="color:var(--ink);font-weight:600;margin-bottom:2px">图里的编码</div>
  <div>左面板（俯视图）：⊙ 是旋转轴 +z，圆圈加一点表示箭头朝屏幕外。指针是物体，末端的小圆点是标记点 P。虚线弧从 0° 的位置画到当前位置，它扫过的角度就是物体转过的角度。</div>
  <div>右面板（(w, z) 平面）：<b style="color:var(--accent)">实心点</b>是当前四个数在圆上的位置。<b style="color:var(--ink-2)">虚线半径</b>从圆心指向这个点，方向就是当前极角 θ/2。<b style="color:var(--ink-2)">空心小点</b>是固定的参考位置，标着 0°、90°、180°。<b style="color:var(--accent)">红色虚线圈</b>是 360° 位置，也就是 0° 点的对径点；它比实心点大一圈，套在一起时说明当前姿态正好是 360°。</div>
</div>
<script>
(function () {
  var slider = document.getElementById('qd-slider');
  var obj = document.getElementById('qd-obj');
  var arc = document.getElementById('qd-arc');
  var dot = document.getElementById('qd-dot');
  var rad = document.getElementById('qd-rad');
  var out = {
    theta: document.getElementById('qd-theta'),
    q: document.getElementById('qd-q'),
    nq: document.getElementById('qd-nq'),
    c: document.getElementById('qd-c'),
    s: document.getElementById('qd-s'),
    v: document.getElementById('qd-v'),
    pose: document.getElementById('qd-pose')
  };
  var CX = 160, CY = 150, ROBJ = 88, RCIRC = 100, CCX = 165, CCY = 160;

  function f(x) {
    if (Math.abs(x) < 5e-5) x = 0;
    return x.toFixed(4);
  }

  function update() {
    var th = parseFloat(slider.value);
    var rad0 = th * Math.PI / 180;
    var half = rad0 / 2;
    var w = Math.cos(half), z = Math.sin(half);
    var c = w * w - z * z, s = 2 * w * z;

    obj.setAttribute('transform', 'rotate(' + (-th) + ' ' + CX + ' ' + CY + ')');

    var rr = ((th % 360) + 360) % 360;
    if (rr === 0) {
      arc.setAttribute('d', '');
    } else {
      var rrad = rr * Math.PI / 180;
      var x0 = CX + ROBJ, y0 = CY;
      var x1 = CX + ROBJ * Math.cos(rrad), y1 = CY - ROBJ * Math.sin(rrad);
      var large = rr > 180 ? 1 : 0;
      arc.setAttribute('d', 'M' + x0 + ',' + y0 + ' A' + ROBJ + ',' + ROBJ + ' 0 ' + large + ' 0 ' + x1 + ',' + y1);
    }

    var px = CCX + RCIRC * Math.cos(half), py = CCY - RCIRC * Math.sin(half);
    dot.setAttribute('cx', px);
    dot.setAttribute('cy', py);
    rad.setAttribute('x2', px);
    rad.setAttribute('y2', py);

    out.theta.textContent = th + '°';
    out.q.textContent = '(' + f(w) + ', 0.0000, 0.0000, ' + f(z) + ')';
    out.nq.textContent = '(' + f(-w) + ', 0.0000, 0.0000, ' + f(-z) + ')';
    out.c.textContent = f(c);
    out.s.textContent = f(s);
    out.v.textContent = '(' + f(c) + ', ' + f(s) + ', 0.0000)';
    if (th === 0) {
      out.pose.textContent = '初始姿态';
    } else if (rr === 0) {
      var turns = Math.round(th / 360);
      out.pose.textContent = '与 0° 相同（q = ' + (turns % 2 ? '−q(0°)' : 'q(0°)') + '）';
    } else {
      out.pose.textContent = '相对 0° 转过 ' + rr + '°';
    }
  }

  slider.addEventListener('input', update);
  var btns = document.querySelectorAll('#qd-wrap button');
  for (var i = 0; i < btns.length; i++) {
    btns[i].addEventListener('click', function () {
      var set = this.getAttribute('data-set');
      var step = this.getAttribute('data-step');
      var v;
      if (set !== null) {
        v = parseFloat(set);
      } else {
        v = parseFloat(slider.value) + parseFloat(step);
      }
      if (v < 0) v = 0;
      if (v > 720) v = 720;
      slider.value = v;
      update();
    });
  }
  update();
})();
</script>
```

关键帧的数值：

| $\theta$ | $q = (w, 0, 0, z)$ | 圆上的点 | 姿态 |
|---|---|---|---|
| $0^\circ$ | $(1, 0, 0, 0)$ | $(1, 0)$ | 初始 |
| $90^\circ$ | $(0.71, 0, 0, 0.71)$ | $(0.71, 0.71)$ | 与 $0^\circ$ 差 $90^\circ$ |
| $180^\circ$ | $(0, 0, 0, 1)$ | $(0, 1)$ | 与 $0^\circ$ 差 $180^\circ$ |
| $360^\circ$ | $(-1, 0, 0, 0)$ | $(-1, 0)$ | 与 $0^\circ$ 相同 |
| $720^\circ$ | $(1, 0, 0, 0)$ | $(1, 0)$ | 与 $0^\circ$ 相同 |

两个面板共用同一个 $\theta$。左边的物体转到 $90^\circ$，右边的点只走了 $45^\circ$ 极角。

## H 为什么 q 与 -q 是同一个姿态

把 $q$ 的四个数全部取负，作用到任何向量的结果都不变。两条证据都摆在这里。

代数：$w$ 变成 $-w$，$z$ 变成 $-z$，$w^2 - z^2$ 和 $2wz$ 都不变，所以 $v'$ 不变。

几何：$(w, z)$ 圆上的对径点，极角差 $180^\circ$，对应的姿态角差 $360^\circ$。姿态转满一圈，回到的就是原样。

姿态每 $360^\circ$ 重复一次，四个数每 $720^\circ$ 才重复一次。两个数对应一个姿态，这叫双覆盖。

记住这条：看到 $q$ 和 $-q$ 同时出现，它们指的是同一个姿态。

## I 自己跑一遍

脚本只用 Python 3 标准库，四组输出各管一件事。完整文件：[book/concepts/figures/quaternion/quat_demo.py](figures/quaternion/quat_demo.py)。在仓库根目录运行：

```bash
python3 book/concepts/figures/quaternion/quat_demo.py
```

第 4 组的漂移是真实实验，没有人为给每步乘系数。循环就是 `q = q_mul(q, step)` 连做 1e5 次，中间不归一化。`step` 每步由轴角生成，轴的方向每步有微小变化，模拟物体在空中连续转动。

实测输出（Python 3.14.4）：

```text
[1] 绕 z 轴：角度 -> 四个数（w, x, y, z）
角度      q                            cos(theta)   sin(theta)
   0 deg  [1.0000, 0.0000, 0.0000, 0.0000]     1.0000     0.0000
  90 deg  [0.7071, 0.0000, 0.0000, 0.7071]     0.0000     1.0000
 180 deg  [0.0000, 0.0000, 0.0000, 1.0000]    -1.0000     0.0000
 360 deg  [-1.0000, 0.0000, 0.0000, 0.0000]     1.0000    -0.0000
 720 deg  [1.0000, 0.0000, 0.0000, 0.0000]     1.0000    -0.0000

[2] q(90 deg) 作用于 v = (1, 0, 0)
脚本的 rotate() = [0.0000, 1.0000, 0.0000]
2 维公式  x' = x*cos - y*sin, y' = x*sin + y*cos, cos = 0.0000, sin = 1.0000
手算      (1*cos - 0*sin, 1*sin + 0*cos, 0) = (0.0000, 1.0000, 0)

[3] q 与 -q 作用到向量：结果相同
q(0 deg)   = [1.0000, 0.0000, 0.0000, 0.0000]
q(360 deg) = [-1.0000, 0.0000, 0.0000, 0.0000]  <- 正好是 -q(0 deg)
rotate(q0)   = [0.3000, -0.4000, 0.5000]
rotate(-q0)  = [0.3000, -0.4000, 0.5000]
rotate(q360) = [0.3000, -0.4000, 0.5000]

[4] 不归一化的连乘：范数漂移
float64  同一个步长连乘 1e5 次：范数 = 0.9999999999999049
float64  轴微变连乘 1e5 次：范数 = 0.9999999999958722
float32  轴微变连乘 1e5 次：范数 = 0.9977069364907649
每步输入范数 1+1e-6，连乘 1e5 次：范数 = 1.105171
  用它旋转 (1,0,0) 得到 [-1.1477, 0.0000, 0.4177]
  长度应为 1，实测 1.221403（范数的平方）
  归一化之后： [0.1736, 0.0000, -0.9848, 0.0000]
```

看输出时盯这几处：

- 第 1 组：θ 从 0° 转到 360°，q 在圆上只走了半圈。
- 第 2 组：两条路径算出的 `v'` 完全一样。
- 第 3 组：三行 `rotate` 结果一模一样。
- 第 4 组：float64 几乎不漂，float32 明显漂。

第 4 组的三个数字对应三种情况，都不是人为给每步乘系数：

- float64 存 1e5 步，范数只差 1e-13 量级，几乎看不出来。
- 换成 float32 每步舍入一次，范数差到 2e-3。
- 每步输入的四元数范数是 $1 + 10^{-6}$ 时（输出里印作 1+1e-6），连乘把这个误差放大 1e5 倍，范数到 1.105。

未归一化的 $q$ 旋转向量时，会把向量长度乘上范数的平方。范数 1.105 时，长度变成 1.221。

最后一行是同一个连乘结果除掉自己的范数。起点是 $(1, 0, 0, 0)$，每步乘的步长范数是 $1 + 10^{-6}$。走 1e5 步之后归一化，就得到那一行。

所以姿态存成单位四元数，连乘之后归一化。

## J 容易踩的坑

| 坑 | 现象 | 处理 |
|---|---|---|
| `wxyz` 与 `xyzw` 混用 | 同一个数组读成另一个旋转 | 抄代码前查文档里的分量顺序 |
| 忘了 $q$ 与 $-q$ 等价 | 同一个姿态出现两份数值 | 先看两个 $q$ 是不是互为相反数 |
| 右边忘了取共轭 | 算出 $q\,v\,q$，不是旋转 | 旋转是 $v' = q\,v\,q^{*}$ |
| 不归一化 | 向量长度被悄悄放大 | 每次连乘后归一化 |
| 只存四元数 | 位置丢了 | 姿态用四元数，位置另存三维向量 |

quaternion 只是姿态的数值表示。它不多一个传感器，也不多一个自由度。

## K 自测

先在心里选定一个答案，再展开解析。

<details>
<summary><b>1.</b> 四元数里 $i, j, k$ 满足 $i^2 = j^2 = k^2 = -1$，且 $ij = k$。那 $ji$ 等于什么？（A）$k$　（B）$-k$　（C）$1$　（D）$-1$</summary>
<p><b>答案：B。</b>A 面板的规则是交换两个因子要变号，所以 $ji = -k$。要自己验也可以拿 $k^2$ 入手：$(ij)(ij) = i(ji)j$。若 $ji = k$，右边是 $i(k)j = (ik)j = (-j)j = 1$，和 $k^2 = -1$ 矛盾；取 $ji = -k$ 时右边是 $i(-k)j = (j)j = -1$，对得上。选 A 是以为乘法可交换。</p>
</details>

<details>
<summary><b>2.</b> $q = (0, 1, 0, 0)$ 是单位四元数。它表示绕哪根轴转多少度？（A）绕 $x$ 轴转 $180^\circ$　（B）绕 $y$ 轴转 $180^\circ$　（C）绕 $x$ 轴转 $90^\circ$　（D）绕 $z$ 轴转 $180^\circ$</summary>
<p><b>答案：A。</b>标量部 $w = 0 = \cos(\theta/2)$，所以 $\theta/2 = 90^\circ$，$\theta = 180^\circ$。这里存的是半角，别忘了乘 2。向量部 $(1, 0, 0)$ 是单位向量，方向就是转轴，即 $+x$。选 C 忘了半角；选 D 把向量部读成了 $z$ 分量。</p>
</details>

<details>
<summary><b>3.</b> 手算 $(1 + i)(1 - j)$。（A）$1 + i - j + k$　（B）$1 + i - j - k$　（C）$1 - i + j - k$　（D）$1 + i + j + k$</summary>
<p><b>答案：B。</b>用分配律：$(1+i)(1-j) = 1 - j + i - ij$。查 A 面板的表得 $ij = k$，所以 $-ij = -k$。合起来是 $1 + i - j - k$。选 A 把 $ij$ 记成了 $-k$；选 D 是把减号丢了；选 C 是 $i$ 和 $j$ 的符号都反了。</p>
</details>

<details>
<summary><b>4.</b> 按 F 面板展开出来的式子，取 $w = 0.6$、$z = 0$、$v = (0.8, 0.6, 0)$，求 $v'$。（A）$(0.288, 0.216, 0)$　（B）$(0.800, 0.600, 0)$　(C) $(0.480, 0.360, 0)$　（D）$(0.216, 0.288, 0)$</summary>
<p><b>答案：A。</b>先算 $w^2 - z^2 = 0.36$、$2wz = 0$。代入展开式：$x' = 0.36 \times 0.8 - 0 \times 0.6 = 0.288$，$y' = 0.36 \times 0.6 + 0 \times 0.8 = 0.216$，$z' = 0$。注意这个 $q$ 的范数是 0.6 而不是 1，结果长度被缩成 $0.36$，正是 I 面板讲的归一化问题。选 B 把它当成单位四元数；选 C 把 $w^2 - z^2$ 当成了 $w$。</p>
</details>

<details>
<summary><b>5.</b> 把交互图的滑块停在 $135^\circ$，右面板的实心点落在圆上的哪个位置？（A）$(0.9239, 0.3827)$　（B）$(0.3827, 0.9239)$　(C) $(-0.3827, 0.9239)$　（D）$(0.7071, 0.7071)$</summary>
<p><b>答案：B。</b>圆上的极角等于 $\theta/2$，$135^\circ$ 对应极角 $67.5^\circ$。于是 $w = \cos 67.5^\circ = 0.3827$，$z = \sin 67.5^\circ = 0.9239$，点落在第一象限、靠近正 $z$ 轴。选 A 是把 $w$ 和 $z$ 写反了；选 C 是极角算成了 $112.5^\circ$；选 D 是停在 $90^\circ$ 上。</p>
</details>

<details>
<summary><b>6.</b> 把交互图的滑块拖到 $360^\circ$，右面板的实心点停在哪？它作用到 $v = (1, 0, 0)$ 的输出又是什么？（A）停在 $(1, 0)$，输出是 $(1, 0, 0)$　（B）停在 $(-1, 0)$，输出是 $(1, 0, 0)$　（C）停在 $(-1, 0)$，输出是 $(-1, 0, 0)$　（D）停在 $(0, -1)$，输出是 $(0, 1, 0)$</summary>
<p><b>答案：B。</b>分两步预测。点：极角 = $\theta/2 = 180^\circ$，所以点落在圆的最左端 $(-1, 0)$，和 $0^\circ$ 的点 $(1, 0)$ 是一对对径点。输出：$360^\circ$ 对应的 $q = (-1, 0, 0, 0)$，代入还原公式得 $\cos\theta = w^2 - z^2 = 1 - 0 = 1$，$\sin\theta = 2wz = 0$，于是 $v' = (1 \times 1 - 0 \times 0,\ 1 \times 0 + 0 \times 1,\ 0) = (1, 0, 0)$，和 $0^\circ$ 时一模一样。选 C 是把"点走到对面"错当成"向量也转到了对面"；选 A 是以为极角等于姿态角。</p>
</details>

<details>
<summary><b>7.</b> 按图 D 的约定：俯视 $xy$ 平面，⊙ 表示 $+z$ 轴指向屏幕外，指针在 $0^\circ$ 时指向 $+x$。物体绕 $+z$ 转 $+90^\circ$（右手定则）后，标记点 P 在图上走到哪一侧？（A）仍指向 $+x$，⊙ 指向屏幕外说明物体没转　（B）指向 $+y$，图上是逆时针　（C）指向 $-x$　（D）指向 $-y$，图上是顺时针</summary>
<p><b>答案：B。</b>右手定则绕 $+z$：从 $+x$ 转向 $+y$。俯视时 $+z$ 朝屏幕外，也就是朝观察者，所以 $+\theta$ 在图上看是逆时针。选 D 是把旋转方向搞反了，那等于绕 $-z$ 转；选 A 是以为 ⊙ 表示轴在屏幕里面。</p>
</details>

<details>
<summary><b>8.</b> 让 float32 和 float64 各自连乘 1e5 步，每一步的步长四元数都精确到各自精度。预测：哪一个的范数离 1 更远？差多少个数量级？为什么？（A）float32 更远，差约 10 个数量级：它每步的舍入误差在 $10^{-7}$ 量级，1e5 步累积到 $10^{-3}$ 量级；float64 每步在 $10^{-16}$ 量级，累积到 $10^{-13}$ 量级　（B）float64 更远，它的有效位更多，误差累积更快　（C）差不多，步数相同则舍入误差总量相同　（D）float32 更远，但原因是每步输入的四元数不再是单位长度</summary>
<p><b>答案：A。</b>有效位越少，每步能记住的精度越少，舍入误差就越大。1e5 步把每步的误差累起来，float32 落到 $10^{-3}$ 量级，float64 还在 $10^{-13}$ 量级，差约 10 个数量级。选 D 把原因搞错了：float32 那一组的每步输入仍是单位四元数，漂的是浮点舍入；脚本里另一行"每步输入范数 1+1e-6"才是输入自带误差的情况，那一组用 float64 也会漂。选 C 忽略了每步误差的量级差别。</p>
</details>

## L 接下来

这一页只回答一件事：四元数是什么、怎么算、怎样表示旋转、为什么 $q$ 与 $-q$ 等价。

后面的内容属于后续页：任意转轴的旋转证明、两次旋转谁先谁后、姿态之间怎么插值（slerp、nlerp）。本页固定 $z$ 轴，是它们的前置。

想多看几眼（可选，不影响上面的任何一题）：

- [3Blue1Brown：quaternions 专题](https://www.3blue1brown.com/topics/quaternions)
- [Ben Eater：交互式 quaternion](https://eater.net/quaternions)
- [Wikipedia：Quaternion](https://en.wikipedia.org/wiki/Quaternion)

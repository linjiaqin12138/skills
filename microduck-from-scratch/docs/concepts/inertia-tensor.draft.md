---
template: doc
title: 惯量张量：转动惯量为什么不是一个数
subtitle: 从标量 mr² 到「3 个对角值 + 一个主轴朝向」
---

推一个箱子走直线，只需要知道质量。让它在空中转起来，只给质量就不够了：绕不同的轴转，「难推动」的程度不同，还会互相串扰。完整描述转动惯性的是一个 3x3 矩阵——惯量张量。这一页讲清它的结构，以及仿真器为什么最后只存 3 个数加 1 个朝向。

## 一句话版本

- 刚体绕不同的轴转，「难推动」的程度不同。
- 完整描述要一个 3x3 对称矩阵，叫惯量张量。
- 把坐标系转到一组特殊方向（主轴）上，矩阵只剩对角 3 个数。
- 主轴相对刚体的朝向，另用一个四元数记录。
- 仿真器存的转动惯量 = 3 个对角数 + 1 个朝向四元数。

## 为什么不是一个数

标量转动惯量 I = mr² 只回答「绕固定轴」的问题。刚体自由转动时，转轴可以是任何方向。

1. 方向不同，数值不同。一根细杆绕长轴转很省力，绕短轴转很费力。一个数说不全。
2. 质量分布歪的时候，绕 x 轴转也会带出 y 轴方向的角动量。这种「串轴」写在矩阵非对角元里，叫惯量积。
3. 数学结论：3x3 实对称矩阵总能靠旋转坐标系变成对角矩阵。这组坐标轴叫主轴，对角值叫主惯量。
4. 主轴系下没有「串轴」，转动方程最简。仿真器因此直接存对角化后的结果。

```svg
<svg width="660" height="250" viewBox="0 0 660 250" xmlns="http://www.w3.org/2000/svg">
  <rect width="660" height="250" fill="#14181f"/>
  <g transform="translate(165,125)">
    <line x1="-90" y1="0" x2="90" y2="0" stroke="#e25555" stroke-width="1.5"/>
    <line x1="0" y1="-80" x2="0" y2="80" stroke="#e25555" stroke-width="1.5"/>
    <text x="94" y="4" fill="#e25555" font-size="12">x</text>
    <text x="4" y="-84" fill="#e25555" font-size="12">y</text>
    <g transform="rotate(30)">
      <line x1="-70" y1="0" x2="70" y2="0" stroke="#8b93a3" stroke-width="6"/>
      <circle cx="-70" cy="0" r="22" fill="#d8a018"/>
      <circle cx="70" cy="0" r="14" fill="#d8a018"/>
      <line x1="-100" y1="0" x2="100" y2="0" stroke="#4f9cf7" stroke-width="1.5" stroke-dasharray="6 4"/>
      <line x1="0" y1="-70" x2="0" y2="70" stroke="#4f9cf7" stroke-width="1.5" stroke-dasharray="6 4"/>
    </g>
    <text x="104" y="58" fill="#4f9cf7" font-size="12">主轴</text>
    <text x="-150" y="110" fill="#c8cfd9" font-size="13">坐标轴 ≠ 主轴：非对角元不为零</text>
  </g>
  <g transform="translate(495,125)">
    <g>
      <line x1="-70" y1="0" x2="70" y2="0" stroke="#8b93a3" stroke-width="6"/>
      <circle cx="-70" cy="0" r="22" fill="#d8a018"/>
      <circle cx="70" cy="0" r="14" fill="#d8a018"/>
      <line x1="-100" y1="0" x2="100" y2="0" stroke="#4f9cf7" stroke-width="1.5"/>
      <line x1="0" y1="-70" x2="0" y2="70" stroke="#4f9cf7" stroke-width="1.5"/>
      <text x="104" y="4" fill="#4f9cf7" font-size="12">I₁ 轴</text>
      <text x="4" y="-74" fill="#4f9cf7" font-size="12">I₂ 轴</text>
    </g>
    <text x="-150" y="110" fill="#c8cfd9" font-size="13">坐标轴 = 主轴：只剩 3 个对角值</text>
  </g>
</svg>
```

同一个刚体，两种坐标系。左边：坐标轴没对准质量分布，惯量矩阵的非对角元不是零。右边：坐标轴沿主轴，矩阵只剩对角 3 个数，但这 3 个数只对「转过来的坐标系」成立——所以要再记一个朝向。

## 最小示例

三个质点拼一个歪哑铃，先算出 3x3 惯量张量，再做特征值分解。特征值就是 3 个主惯量，特征向量就是主轴朝向。脚本在 `assets/inertia-tensor/inertia_demo.py`，用 python3 + numpy 跑：

```python title="inertia_demo.py"
import numpy as np

# 三个质点（质量 m，位置 p），拼一个歪的哑铃形
pts = [(2.0, np.array([ 1.0, 0.3, 0.0])),
       (2.0, np.array([-0.6, 0.8, 0.2])),
       (1.0, np.array([ 0.2,-0.5, 0.9]))]
M = sum(m for m, _ in pts)
com = sum(m*p for m, p in pts) / M

# 质心系下累加 3x3 惯量张量
I = np.zeros((3, 3))
for m, p in pts:
    r = p - com
    I += m * ((r @ r) * np.eye(3) - np.outer(r, r))

vals, vecs = np.linalg.eigh(I)   # 特征值 = 主惯量，特征向量 = 主轴
print("惯量张量 I =\n", I)
print("主惯量（对角 3 个数）:", vals)
print("主轴矩阵（每列一根轴）:\n", vecs)
print("转进主轴系 R.T @ I @ R =\n", vecs.T @ I @ vecs)
```

实测输出（numpy 2.x，保留 4 位小数）：

```text
惯量张量 I =
 [[1.684  0.8    0.32 ]
  [0.8    3.112  0.572]
  [0.32   0.572  3.692]]        ← 非对角元不为零：坐标轴不是主轴
主惯量（对角 3 个数）: [1.3242 2.9198 4.244 ]
主轴矩阵（每列一根轴）:
 [[ 0.9156  0.2909  0.2775]
  [-0.4011  0.7072  0.5822]
  [-0.0268 -0.6444  0.7642]]
转进主轴系 R.T @ I @ R =
 [[1.3242 0.     0.    ]
  [0.     2.9198 0.    ]
  [0.     0.     4.244 ]]       ← 对角化完成
```

最后一行印证「3 个数 + 一个朝向」：对角值来自特征值，朝向来自特征向量（四元数只是更紧凑的存储写法）。

## 素材

- MuJoCo 官方 XML 参考的 `<inertial>` 元素（mass / pos / quat / diaginertia 字段定义）：[mujoco.readthedocs.io XMLreference](https://mujoco.readthedocs.io/en/stable/XMLreference.html#body-inertial)
- 主轴选取的对称性法则与对角化推导（中文）：[知乎：一般性刚体惯量主轴的选取](https://zhuanlan.zhihu.com/p/137394465)
- 严格的动力学推导（英文讲义 §3.2）：[David Tong, The Motion of Rigid Bodies](https://www.damtp.cam.ac.uk/user/tong/dynamics/three.pdf)

## 自测

```html
<details>
<summary><b>1.</b> 一个 L 形零件，你拿到的惯量矩阵里非对角元不是零。这说明什么？（A）数据一定错了，非对角元必须是零　（B）正常——你用的坐标轴没对准它的主轴　（C）零件有负质量　（D）测量仪器坏了</summary>
<p><b>答案：B。</b>非对角元（惯量积）记录的是「串轴」：坐标系和主轴不重合时它们必然出现。把坐标系旋转到主轴方向，矩阵自然只剩对角值。A 是最常见的误判——把「非对角非零」当成数据错误。</p>
</details>
<details>
<summary><b>2.</b> 仿真器存「3 个对角值 + 1 个朝向四元数」，而不是直接存 3x3 矩阵的 6 个独立元素。根本原因是什么？（A）省内存　（B）主轴系下转动方程最简，存的就是对角化后的现成结果　（C）四元数比矩阵精确　（D）历史习惯</summary>
<p><b>答案：B。</b>刚体转动方程在主轴系里没有交叉项，数值积分最干净。既然算之前总要先对角化，不如直接存对角化后的结果：3 个主惯量 + 主轴朝向。7 个数反而比 6 个多，A 不成立。</p>
</details>
<details>
<summary><b>3.</b> 把一个刚体的 3 个主惯量都放大 2 倍，质量和质心不动。它在仿真里会怎样？（A）下落变慢　（B）同样的力矩下，角加速度减半　（C）接触摩擦变大　（D）没有变化</summary>
<p><b>答案：B。</b>转动版牛顿第二定律：力矩 = 惯量 × 角加速度。惯量翻倍，同样的力矩只能产生一半角加速度——转起来更「钝」。平移运动只由质量决定，A 错。</p>
</details>
<details>
<summary><b>4.</b> 两个刚体质量相同、质心相同，惯量张量可以差很多吗？（A）不可能，质量和质心定了惯量就定了　（B）可以，惯量还取决于质量分布离轴多远　（C）可以，但只差一个常数倍　（D）只在外形是球时才可能</summary>
<p><b>答案：B。</b>惯量 ∝ 质量 × 到轴距离的平方。同样 1 kg，压成球和拉成哑铃，主惯量差几个量级。质量和质心只是质量分布的最低阶信息，定不了惯量。</p>
</details>
```

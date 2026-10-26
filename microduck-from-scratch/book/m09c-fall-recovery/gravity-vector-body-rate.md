# 机体系重力投影变化率：\(\dot{g}=-\omega\times g\)

> 笔记目的：从零推到文档里的公式与 `rate` 代码，并把推导中易卡的点写清楚。  
> Anki 卡牌（learning 仓）：`nndl/cards/刚体旋转数学.json`

---

## 0. 符号与坐标系

| 符号 | 含义 |
|------|------|
| 世界系 (world) | 相对地面固定；重力方向不变 |
| 机体系 (body) | 跟着机器人 / IMU 一起转 |
| \(g\) / \(g_{\mathrm{body}}\) | 重力方向在**机体系**中的投影（单位向量）。直立时约 \((0,0,-1)\) |
| \(g_{\mathrm{world}}\) | 重力方向在世界系中的表示，**恒定** |
| \(\omega\) | 陀螺仪测到的**机体系**角速度 (rad/s) |
| \(\omega_{\mathrm{world}}\) | 同一角速度在世界系中的分量 |
| \(R(t)\) | 旋转矩阵：把**机体系向量**变到**世界系** |
| \([\omega]_{\times}\) | \(\omega\) 的反对称矩阵，满足 \(\omega\times v=[\omega]_{\times}v\) |

坐标变换：

\[
g_{\mathrm{world}} = R\, g_{\mathrm{body}},
\qquad
g_{\mathrm{body}} = R^{T}\, g_{\mathrm{world}}
\]

（用到 \(R^{-1}=R^{T}\)，见第 4 节。）

---

## 1. 目标公式从哪来（直观版）

同一个物理方向，在世界系里看不变，在机体系里看会变（因为机体在转）。刚体上有经典关系：

\[
\Bigl(\frac{d\vec{v}}{dt}\Bigr)_{\mathrm{world}}
=
\Bigl(\frac{d\vec{v}}{dt}\Bigr)_{\mathrm{body}}
+ \omega\times\vec{v}
\]

对重力方向：左边 = 0（世界系重力方向不随时间变），于是

\[
0 = \dot{g}_{\mathrm{body}} + \omega\times g
\quad\Rightarrow\quad
\dot{g}_{\mathrm{body}} = -\omega\times g
\]

下面不用“接受经典关系”，用旋转矩阵把它推出来，并解释叉积、\(R^{-1}=R^{T}\) 等为什么成立。

---

## 2. 为什么速度是 \(\omega\times r\)？（平移？为什么是叉积？）

### 2.1 我们关心的是姿态，不是质心平移

完整刚体运动里，某点速度可以写成：

\[
\dot{r} = \dot{r}_{O} + \omega\times r_{\mathrm{rel}}
\]

- \(\dot{r}_{O}\)：原点（参考点）的**平移**速度  
- \(\omega\times r_{\mathrm{rel}}\)：**转动**带来的速度  

旋转矩阵 \(R(t)\) 只描述**朝向**。重力投影、机体固定方向如何相对世界变化，由姿态决定；**纯平移不改变各轴朝向，也不改变“竖直向下”相对机体的投影关系**。推 \(\dot{R}\) 时只保留转动部分，故出现 \(\omega\times\cdots\)。

### 2.2 纯转动时，速度为什么是叉积？

想象位置向量 \(\vec{r}\) 的一端在原点附近固定在刚体上，刚体以角速度 \(\vec{\omega}=\Omega\hat{n}\) 转动。

- 速度 \(\vec{v}\) **垂直于** \(\vec{r}\)，也**垂直于**转轴 \(\hat{n}\)  
- 圆周半径是到转轴的垂直距离：\(|\vec{r}|\sin\theta\)  
- 速率：\(|\vec{v}|=\Omega\cdot|\vec{r}|\sin\theta=|\vec{\omega}|\,|\vec{r}|\,\sin\theta\)

叉积 \(\vec{\omega}\times\vec{r}\) 的几何定义正好是：

- 大小：\(|\omega||r|\sin\theta\)  
- 方向：垂直于 \(\omega\) 与 \(r\)，右手定则  

因此（世界系下）：

\[
\dot{r} = \omega_{\mathrm{world}}\times r
\]

### 2.3 叉积怎么算？

**分量公式**（\(a\times b\)）：

- x：`a_y * b_z - a_z * b_y`
- y：`a_z * b_x - a_x * b_z`
- z：`a_x * b_y - a_y * b_x`

**反对称矩阵**（使 \(a\times b=[a]_{\times}b\)）：

```
[a]_× = [  0   -a_z   a_y ]
        [  a_z   0   -a_x ]
        [ -a_y   a_x   0  ]
```

性质（直接对矩阵转置验证）：

\[
[\omega]_{\times}^{T} = -[\omega]_{\times}
\]

这对应叉积的反交换律 \(\omega\times v=-(v\times\omega)\)。

---

## 3. 推出 \(\dot{R}=R[\omega]_{\times}\)

### 3.1 机体上固定的向量

取机体系中**分量不变**的向量 \(b\)（粘在机体上，\(\dot{b}=0\)）。世界系分量：

\[
r = R\, b
\]

对时间求导（乘积法则，\(b\) 当常数）：

\[
\dot{r} = \dot{R}\, b
\]

### 3.2 接上转动公式

\[
\dot{r} = \omega_{\mathrm{world}}\times r = \omega_{\mathrm{world}}\times(Rb)
\]

角速度在两系之间同样用 \(R\) 变换：

\[
\omega_{\mathrm{world}} = R\omega
\quad\Leftrightarrow\quad
\omega = R^{T}\omega_{\mathrm{world}}
\]

于是：

\[
\dot{R}\, b = (R\omega)\times(Rb)
\]

### 3.3 关键恒等式：\((Ru)\times(Rv)=R(u\times v)\)

**几何理解：**  
叉积由“面积大小 + 垂直于平面 + 右手朝向”唯一确定。旋转 \(R\)（\(\det R=1\)）保长度、夹角与右手系。因此：对两个向量做相同旋转再叉积，等于先叉积再做相同旋转。

**代数要点（\(R\in SO(3)\)）：**  
\(R^{T}R=I\) 且 \(\det R=1\)，有

\[
(Ru)\times(Rv)=(\det R)\,R^{-T}(u\times v)=R(u\times v)
\]

令 \(u=\omega\)，\(v=b\)：

\[
\dot{R}\, b = R(\omega\times b) = R[\omega]_{\times}\, b
\]

对任意此类 \(b\) 成立，故

\[
\dot{R} = R[\omega]_{\times}
\]

---

## 4. 为什么 \(R^{-1}=R^{T}\)？

\(R\) 的三列 \(\mathbf{c}_1,\mathbf{c}_2,\mathbf{c}_3\) 是机体系三个单位基向量在世界系中的坐标。

旋转保长度、保角度，故这三列：

- 各自长度为 1：\(\mathbf{c}_i\cdot\mathbf{c}_i=1\)  
- 两两垂直：\(i\neq j\) 时 \(\mathbf{c}_i\cdot\mathbf{c}_j=0\)  

即 \(\mathbf{c}_i\cdot\mathbf{c}_j=\delta_{ij}\)。

而 \((R^{T}R)_{ij}\) 恰好是第 \(i\) 列与第 \(j\) 列的点积，所以

\[
R^{T}R = I \quad\Rightarrow\quad R^{-1}=R^{T}
\]

因此 \(g_{\mathrm{body}}=R^{T}g_{\mathrm{world}}\) 合法。

---

## 5. 对 \(g_{\mathrm{body}}\) 求导（主结论）

世界系重力恒定：\(\dot{g}_{\mathrm{world}}=0\)。

\[
g_{\mathrm{body}} = R^{T} g_{\mathrm{world}}
\]

两边对 \(t\) 求导：

\[
\dot{g}_{\mathrm{body}} = \dot{R}^{T}\, g_{\mathrm{world}}
\]

代入 \(\dot{R}=R[\omega]_{\times}\)：

\[
\dot{R}^{T} = (R[\omega]_{\times})^{T} = [\omega]_{\times}^{T} R^{T} = -[\omega]_{\times} R^{T}
\]

故

\[
\dot{g}_{\mathrm{body}}
= -[\omega]_{\times} R^{T} g_{\mathrm{world}}
= -[\omega]_{\times} g_{\mathrm{body}}
= -\omega\times g_{\mathrm{body}}
\]

即文档结论：

\[
\dot{g} = -\omega\times g
\]

这也与第 1 节“经典关系 + 世界系重力不变”移项结果一致。

---

## 6. 代码里的 `rate`：为什么看 z 分量？

\[
\dot{g} = -\omega\times g
\]

\(\omega\times g\) 的 **z 分量**为：

\[
(\omega\times g)_z = \omega_x g_y - \omega_y g_x
\]

因此重力投影 z 分量的变化率：

\[
\dot{g}_z = -(\omega_x g_y - \omega_y g_x)
\]

写成代码符号：

```text
rate = -(gyro_x * gravity_y - gyro_y * gravity_x)
```

### 物理含义

- 直立：\(g\approx(0,0,-1)\)，\(g_z\approx -1\)  
- 倾斜：\(|g_z|\) 变小，\(g_z\) 从 \(-1\) 往 \(0\) 走（数值变大）  
- `rate > 0`：\(\dot{g}_z>0\)，倾覆在**加剧**  
- `rate < 0`：在**回正**

只取 z：因为直立姿态主要由 \(g_z\) 刻画；监控 \(\dot{g}_z\) 就是在看“竖直投影是否正在变坏 / 变好”。

---

## 7. 推导链一览

```text
世界系重力不变
    → (dg/dt)_world = 0

刚体固定点/固定方向（相对原点）纯转动
    → ṙ = ω_world × r
    （平移不进姿态方程）

r = R b,  ω_world = R ω,  (Ru)×(Rv)=R(u×v)
    → Ṙ = R [ω]_×

旋转列正交归一
    → R^{-1} = Rᵀ
    → g_body = Rᵀ g_world

对 g_body 求导 + [ω]_×ᵀ = -[ω]_×
    → ġ_body = -ω × g

取 z 分量
    → rate = -(gyro_x * gravity_y - gyro_y * gravity_x)
```

---

## 8. 易混点速查

| 疑问 | 一句话 |
|------|--------|
| 为什么左边是 0？ | 世界系里重力方向不随时间变 |
| 为什么可以忽略平移？ | \(R\) 只描述朝向；重力投影随姿态变，不随平移变 |
| 为什么是叉积？ | 圆周速度 ⊥ 轴与径矢，大小 \(\Omega r\sin\theta\)，正是 \(\omega\times r\) |
| 为什么 \(R^{-1}=R^{T}\)？ | 列向量单位正交 \(\Rightarrow R^{T}R=I\) |
| 为什么 \([\omega]_{\times}^{T}=-[\omega]_{\times}\)？ | 反对称矩阵的定义/转置直接验算 |
| 为什么 \((Ru)\times(Rv)=R(u\times v)\)？ | 旋转与叉积可交换（保定向） |
| 为什么看 \(g_z\)？ | 直立用 \(g_z\approx-1\) 刻画；\(\dot{g}_z\) 表示倾覆加剧或回正 |

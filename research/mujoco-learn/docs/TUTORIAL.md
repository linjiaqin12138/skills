# MuJoCo 入门复习教程

基于官方 [Python tutorial](https://github.com/google-deepmind/mujoco/blob/main/python/tutorial.ipynb)，本地脚本在仓库根目录 `step01`–`step07`。  
本文把学习路线、概念与**你走过的所有坑点**写在一起，下次复习不必再问一遍。

---

## 0. 环境与运行

```bash
cd /path/to/mujoco-learn
# 已有 .venv 时可跳过
# python3.12 -m venv .venv && .venv/bin/pip install mujoco numpy pillow imageio imageio-ffmpeg matplotlib

MUJOCO_GL=egl .venv/bin/python step01_mjmodel.py
MUJOCO_GL=egl .venv/bin/python step02_mjdata.py
MUJOCO_GL=egl .venv/bin/python step03_render_step.py
MUJOCO_GL=egl .venv/bin/python step04_dof_examples.py
MUJOCO_GL=egl .venv/bin/python step05_contacts_friction.py
MUJOCO_GL=egl .venv/bin/python step06_tendon_actuator_sensor.py
MUJOCO_GL=egl .venv/bin/python step07_cameras.py
```

产物在 `out/`。无显示环境用 `MUJOCO_GL=egl`（本机 Mesa EGL）。

**交互示意页（补画面感）**

```bash
xdg-open docs/concepts/solref-solimp/index.html
xdg-open docs/concepts/texrepeat-texuniform/index.html
```

---

## 1. 一条主线（先建立这张图）

```text
XML (MJCF)
  → MjModel   不变的「说明书」：有哪些 body/geom/joint、尺寸、材质…
  → MjData    会变的「当前状态」：time / qpos / qvel，以及算出来的量
  → mj_kinematics / mj_forward   把状态「摊开」成世界坐标等派生量
  → mj_step   按物理往前走一小步：x_{t+h} = f(x_t)
  → Renderer  画出来（RGB / depth / segmentation）
```

| 对象 | 记什么 |
|------|--------|
| **body** | 会动的刚体节点（有质量，被 joint 带着动） |
| **geom** | 贴在 body 上的形状（碰撞 + 外观）；可多个 geom 属于同一 body |
| **joint** | 子 body 相对父 body「被允许怎么动」（加法自由度） |
| **site** | 无质量挂点（腱、传感器、受力） |
| **actuator** | 控制通道；`<motor>` 是一种，还可作用在 tendon 上 |
| **sensor** | 读数；结果在 `data.sensordata` |

**body / geom 关系**

```text
body          ← 运动学树上的刚体
  ├─ joint
  ├─ geom × N ← 形状跟着 body 动
  ├─ site
  └─ body …
```

动的是 body，画的/碰的是 geom。`worldbody` 下直接挂的 geom（地板）焊在世界上。

---

## 2. 分步学什么

| 步 | 脚本 | 要带走的一句话 |
|----|------|----------------|
| ① | `step01_mjmodel.py` | XML → `MjModel`；按名访问 `model.geom("…")` |
| ① | `step02_mjdata.py` | `MjData` 是状态；派生量要算才有 |
| ② | `step03_render_step.py` | 渲染 + `mj_step`；没 joint 就不动 |
| ③ | `step04_dof_examples.py` | freejoint / 例子 / 步长越小能量越守恒 |
| ④ | `step05_contacts_friction.py` | 接触力可视化；`friction` 影响滑动 |
| ⑤ | `step06_tendon_actuator_sensor.py` | 绳 + 电机 `ctrl` + 加速度计 |
| ⑥ | `step07_cameras.py` | 命名相机 / 自由相机 / depth / seg |

官方入门主线到 step07 结束。notebook 里还有相机矩阵、往 scene 画轨迹、dominos 电影机位等进阶尾巴，可选。

---

## 3. MjModel / MjData / kinematics（step01–02）

### 3.1 kinematics 是什么词？`mj_kinematics` 干什么？

- **kinematics（运动学）**：只谈位置/姿态/速度关系，**不谈力**。  
- **dynamics（动力学）**：力、加速度、`mj_step` 走的路。

`mj_kinematics(model, data)`：根据当前 `qpos`（和模型里的父子几何）算出世界系位姿，写入如 `data.geom_xpos`。  
不算力、不推进时间。日常也可用更「全」的 `mj_forward`。

### 3.2 多调几次 kinematics，结果一样吗？

**一样**——只要 `model` 和 `qpos` 没变。它是「按当前状态重算派生量」，不是「往前走一步」。

### 3.3 为什么 `MjData(...)` 时 `geom_xpos` 还是 0？不是声明时就该有值吗？

分层：

| 层 | 例子 | 何时有 |
|----|------|--------|
| 模型说明书 | XML 里 geom 的相对 `pos` | `MjModel` 编译后就有 |
| 状态 | `time/qpos/qvel` | `MjData` 创建时有初值 |
| 派生量 | `geom_xpos` | **显式** `mj_kinematics` / `mj_forward` / `mj_step` 之后 |

创建 `MjData` ≈ 分配工作区并清零派生量，**不算物理**。  
无关节模型里 `qpos` 为空；绿球的 `[0.2,0.2,0.2]` 在模型里，不在 `qpos` 里——要 kinematics 才写进 `geom_xpos`。

### 3.4 `geom_xpos` 含义

形状 `(ngeom, 3)`：每个 geom 在**世界坐标系**下的中心位置 `[x,y,z]`。  
也可 `data.geom("green_sphere").xpos`。未传播前不可信。

---

## 4. 渲染、步进、关节（step03）

### 4.1 这一步到底在学什么？

1. 画出来：`Renderer` + `mj_forward` + `update_scene` + `render`  
2. 往前算：`mj_step`  
3. 能动：给 body 加 **joint**（有 DoF）

### 4.2 DoF

**Degree of Freedom（自由度）**：独立能动的量有几个。  
- hinge → 1；`freejoint` → 6（`nv=6`，`qpos` 却是 7：xyz+四元数）  
- 无 joint → DoF=0 → 怎么 `mj_step` 画面也不动。

### 4.3 joint：hinge / free / damping

**joint = 子 body 相对父 body 被允许怎么动**（MuJoCo 用加法：不加就焊死）。

| 类型 | 含义 |
|------|------|
| `hinge` | 绕一根轴转（门轴）。`axis`、`pos`（轴位置） |
| `free` / `<freejoint/>` | 6 DoF 刚体自由运动 |
| `damping` | 关节粘滞：阻力 ∝ 速度，越大越「肉」 |

`name="swing"` 只是名字（摆/挥），可改成任意字符串。

### 4.4 `axis="1 -1 0"` 是指向 (1,-1) 吗？

方向是 **`(1,-1,0)`**（Z=0）。长度无所谓。  
写在**父 body 局部系**；父有 `euler` 时，世界系方向还要再转。  
`pos` 是铰链支点，不是轴向。

### 4.5 `euler="0 0 -30"`

欧拉角（度）：绕父系轴转。`0 0 -30` = 绕 Z −30°。  
位姿相对**父 body**。

### 4.6 `mjVIS_JOINT` / 坐标轴

```python
scene_option.flags[mujoco.mjtVisFlag.mjVIS_JOINT] = True  # 画关节轴
scene_option.frame = mujoco.mjtFrame.mjFRAME_WORLD       # 世界系 RGB=XYZ
# 或 mjFRAME_BODY：body 局部轴
```

红=X，绿=Y，蓝=Z。可用 `model.vis.scale.framelength/framewidth` 调粗细。

### 4.7 `duration * framerate` 是总帧数吗？每次 step 一帧？

约等于目标帧数，但 **不是一步一帧**。  
默认 `timestep=0.002s`，视频 60fps → 约每 16.7ms 采一帧，中间多次 `mj_step`。  
`if len(frames) < data.time * framerate` 按仿真时间采帧。

### 4.8 `Renderer(model, 240, 320)`

第 2 个 **height**，第 3 个 **width**（像素）→ 320×240 图。

---

## 5. 选项、贴图、相机 XML、keyframe（step04 及相关）

### 5.1 `<option timestep>` / `<flag>` / `<default>`

```xml
<option timestep=".001">
  <flag energy="enable" contact="disable"/>
</option>
<default>
  <joint type="hinge" axis="0 -1 0"/>
  <geom type="capsule" size=".02"/>
</default>
```

| 项 | 含义 |
|----|------|
| `timestep` | 每次 `mj_step` 推进的仿真时间（秒） |
| `energy=enable` | 算 `data.energy`（势能/动能），才能画守恒 |
| `contact=disable` | 关掉碰撞，便于看理想守恒/混沌 |
| `<default>` | 后面未写明的 joint/geom 继承这些默认 |

### 5.2 `integrator`

数值积分器：`Euler`（默认，快）、`RK4`（同 dt 通常更准）等。与 `timestep` 一起决定精度。

### 5.3 `model="tippe top"`

根标签上的**显示名**，可不写，不参与物理。

### 5.4 texture / material / builtin=checker

- **texture**：贴图数据；`builtin="checker"` = 内置**棋盘格**（checker = 棋盘），配 `rgb1/rgb2`  
- **material**：引用 texture，设 `texrepeat` / `texuniform` / `reflectance`，再给 geom 挂 `material="…"`  
- 只改颜色可直接 geom `rgba`，不必建 material  

**`texrepeat="rx ry"`**：U/V 方向重复次数，越大越密。  

**`texuniform`（2D）**（官方定义）：

- `false`：在**整个物体表面**重复 N 次 → 物体拉长，格子被扯扁  
- `true`：按**空间单位**重复 N 次 → 与物体大小无关，格子尺度更匀  

**`reflectance`**：反光强弱，0 更哑。

交互复习：`docs/concepts/texrepeat-texuniform/index.html`

### 5.5 plane 的 `size`

平面**碰撞无限大**；`size="a b c"` 主要管**怎么画**：

- a,b：局部 X/Y **半尺寸**（可见约 `2a × 2b`）  
- c：网格线间距一类（可视化）  

不要当成球/盒子那种「实体尺寸」。

### 5.6 geom 常用 `type`

`plane` / `sphere` / `capsule` / `ellipsoid` / `cylinder` / `box` / `mesh` / `hfield` / `sdf` …  
`contype="0" conaffinity="0"`：不参与碰撞；`group`：显示分组。

### 5.7 `<camera name="closeup" …>` 与 `camera="closeup"`

XML 里定义的相机；`update_scene(data, camera="closeup")` **按名字引用**，不是渲染器魔法模式。  
常用：`pos`、`xyaxes`（相机 X/Y 轴方向各 3 个数）、`mode`（`fixed`/`track`/`trackcom`）、`fovy`。

### 5.8 keyframe

预存整份状态（`qpos`/`qvel`…）。  
`mj_resetDataKeyframe(model, data, 0)` 载入第 0 个。  
freejoint 的 qpos(7)=xyz+四元数；qvel(6)=线速度+角速度。

### 5.9 `np.power(10, np.linspace(-2, -4, 5))`

在 \([-2,-4]\) 均匀取 5 个指数，得约 `[0.01 … 0.0001]` 秒的一组 `timestep`，用来对比精度。

---

## 6. 接触：solref / solimp / friction（step05）

接触是**软约束**，不是绝对刚体瞬间弹开。

### 6.1 `solref`（常见两数：`timeconst dampratio`）

| 参数 | 直觉 |
|------|------|
| timeconst 小 | 更硬、纠错更快 |
| dampratio ~1 | 较稳；&lt;1 易弹；&gt;1 更肉 |

### 6.2 `solimp`（可写 3 或 5 个数）

完整：`d0  dmax  width  midpoint  power`（后两默认 0.5, 2）

教程 `solimp=".99 .99 .01"`：

| | 含义 |
|--|------|
| d0=0.99 | 刚接触时阻抗（越近 1 越硬） |
| dmax=0.99 | 穿透到 width 时阻抗 |
| width=0.01 | 过渡的穿透尺度（米） |

**一句话**：solref 管「弹簧多硬、多肉」；solimp 管「约束有多用力，及随穿透如何变硬」。  

交互复习：`docs/concepts/solref-solimp/index.html`  
官方：[Solver parameters](https://mujoco.readthedocs.io/en/stable/modeling.html#solver-parameters)

### 6.3 接触可视化与读数

- flag：`mjVIS_CONTACTPOINT` / `CONTACTFORCE` / `TRANSPARENT`  
- `data.ncon`：接触个数；`data.contact`；`c.dist`：穿透（负=陷入）  
- `mj_contactForce`：前三元常作接触系下法向 + 两切向摩擦  

### 6.4 `friction` / `zaxis`

geom 摩擦系数（可 1～3 个数）。斜面用法向 `zaxis` 撬斜地面。  
灯/相机 `mode="trackcom"` 跟质心；`track` 跟父 body。

---

## 7. 腱、执行器、传感器（step06）

### 7.1 执行器就是电机吗？作用在 tendon 上呢？

**不完全是。** actuator = 控制注入通道；`<motor>` 是一种。  

- 作用在 **joint**：像关节电机拧轴  
- 作用在 **tendon**：像肌肉/绞盘**拉绳**，拉力沿腱传到 site  

### 7.2 `gear`

传动比：默认 motor 下 \(\tau \approx \mathrm{gear}\times\mathrm{ctrl}\)。  
`gear=1, ctrl=20` ≈ 20 量级力矩（再叠加阻尼等）。

### 7.3 `data.ctrl`

- 长度 = **`model.nu`**（执行器个数）  
- 第 i 维 = 第 i 个 actuator 的输入  
- 对默认 motor：数值按力矩/拉力理解，**不是角度**  
- step06：`nu=1`，`ctrl[0]=20` 驱动 `swing`

### 7.4 `model.nu` / `trntype` / `tendon_range` / `sensor_dim` / `sensor_adr`

| 字段 | 含义 |
|------|------|
| `nu` | 执行器个数 = `len(data.ctrl)` |
| `trntype` | 传到哪：`mjTRN_JOINT` / `TENDON` / … |
| `tendon_range` | 每条腱 `[min,max]` 长度限制（配合 `limited`） |
| `sensor_dim` | 该传感器输出几个标量（加速度计=3） |
| `sensor_adr` | 在扁平数组 `data.sensordata` 中的起始下标 |

用名字更方便：`data.sensor("accelerometer").data`。

### 7.5 spatial tendon / site

两 site 之间的空间绳；`range="0 0.35"` 最长 0.35m。  
site：锚点/挂钩/IMU，无质量。

---

## 8. 相机与深度分割（step07）

### 8.1 命名相机 vs 自由相机

- XML `<camera name="fixed|track" …>` + `update_scene(..., camera="track")`  
- `track`：跟着父 body 平移；`fixed`：相对父系钉死（父在动则相机随父坐标系动）  
- **`MjvCamera`**：`lookat` / `distance` / `azimuth` / `elevation`

| 参数 | 含义 |
|------|------|
| `lookat=[0,0,-2]` | 注视世界点 (0,0,-2) |
| `distance` | 到 lookat 的距离（远近） |
| `elevation` | 俯仰角（度），负=俯视 |
| `azimuth` | 水平方位角（度），绕 lookat 转 |

以 lookat 为球心的球坐标。

### 8.2 为什么要 depth / segmentation？

| 模式 | 像素是什么 | 意义 |
|------|------------|------|
| RGB | 颜色 | 给人看 |
| depth | 距离（米） | 测距、点云、抓取；少受纹理骗 |
| segmentation | 对象 ID/类型 | 掩码、检测、仿真免费真值 |

教程里画出来是为了建立「渲染不只是拍照」的直觉；训练时常直接用数组而非调色后的 PNG。

### 8.3 depth 可视化那段在算什么？

```python
depth = depth - depth.min()          # 最近处归零（相对深度）
near = depth[depth <= 1]             # 取出「较近」像素
if near.size:                        # 有近处像素才除，防空/除零
    depth = depth / (2 * near.mean())  # 用近处尺度做对比度
depth_u8 = (255 * np.clip(depth, 0, 1)).astype(np.uint8)
```

**只为存成好看的灰度图**，不是改物理。`enable_segmentation_rendering()` 后 `seg[:,:,0]` 为 ID，再映射灰度同理。

---

## 9. 速查表（复习用）

| 我当时的问题 | 一句话答案 |
|--------------|------------|
| kinematics？ | 运动学；算位姿，不谈力、不推进时间 |
| 多调 kinematics？ | 状态不变则结果不变 |
| 为何 Data 里 xpos 为 0？ | 派生量要显式算 |
| geom_xpos？ | 世界系 geom 中心 |
| DoF？ | 独立能动维数；靠 joint 加出来 |
| hinge/free/damping？ | 转门 / 自由飞 / 关节粘滞 |
| euler？ | 度，相对父 body 的姿态 |
| 一步一帧？ | 否；dt 与 fps 不同 |
| plane size？ | 主要管画多大；碰撞仍无限 |
| checker？ | 棋盘格 |
| texrepeat vs texuniform？ | 重复几次 vs 按物体还是按米铺 |
| solref vs solimp？ | 软硬阻尼 vs 阻抗随穿透 |
| solimp 三个数？ | d0, dmax, width |
| 执行器=电机？ | 不；还可拉腱 |
| gear / ctrl？ | 增益；ctrl 长度=nu，motor 下像力矩 |
| camera="closeup"？ | 引用 XML 里同名相机 |
| lookat/distance/elevation？ | 看哪、多远、俯仰 |
| depth 那段 if near.size？ | 防空；其余是可视化归一化 |
| depth/seg 意义？ | 给几何/身份信息，常供算法 |

---

## 10. 自测（费曼）

用自己的话答；答不清就回到对应章节或交互页。

1. `MjModel` 和 `MjData` 谁不该每步乱改？派生量何时可信？  
2. 只有 geom、没有 joint 的模型，`mj_step` 一万次画面动吗？为什么？  
3. freejoint 的 `nq` 和 `nv` 为什么一个 7 一个 6？  
4. 地板拉很长后格子变扁，该先改 `texrepeat` 还是开 `texuniform`？  
5. `solref` 和 `solimp` 各管什么？  
6. `data.ctrl[0]=20` 在 step06 里物理上大致是什么？  
7. depth 图和 segmentation 图，算法里通常要的是「好看灰度」还是原始数组？

---

## 11. 文件索引

```text
step01_mjmodel.py          MjModel、按名访问
step02_mjdata.py           MjData、mj_kinematics
step03_render_step.py      渲染、hinge、mj_step、坐标轴
step04_dof_examples.py     tippe top、混沌摆、能量 vs dt
step05_contacts_friction.py 接触力、摩擦
step06_tendon_actuator_sensor.py 腱/电机/加速度计
step07_cameras.py          相机、depth、segmentation
out/                       图片与视频产物
docs/concepts/solref-solimp/          交互：接触软硬
docs/concepts/texrepeat-texuniform/   交互：贴图铺法
docs/TUTORIAL.md           本复习文档
```

外部文档：[MuJoCo Python](https://mujoco.readthedocs.io/en/stable/python.html) · [Modeling / Solver](https://mujoco.readthedocs.io/en/stable/modeling.html#solver-parameters) · [XML Reference](https://mujoco.readthedocs.io/en/stable/XMLreference.html)

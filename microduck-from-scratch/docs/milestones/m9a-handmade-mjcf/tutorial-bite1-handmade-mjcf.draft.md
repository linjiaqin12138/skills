---
template: sheet
title: M9a Bite 1：自建 MJCF 身体模型（第 18 章）
subtitle: 自建 duck_handmade.xml：惯量直写官方编译值、碰撞用包围盒、视觉挂官方 STL、D26 穿透收敛
source: clone-from-scratch-tutor · 第 18 章
---

M9 要自己训策略，先得有一个能随便摔、随便改、物理核心不依赖受限资产的仿真身体。本章从零手写一个：15 个刚体、14 个关节、80 个几何体。其中 10 个是自建碰撞盒、70 个是官方视觉网格，物理行为与官方模型逐项对账。

![自建模型 STAND 姿态：官方 STL 视觉网格的鸭子站立在地板上](/home/jqlin/Project/skills/microduck-from-scratch/docs/milestones/m9a-handmade-mjcf/assets/dst/handmade_stand.png)

```callout ok 本章交付
`duck_handmade.xml`：碰撞全 box 基本几何体、惯量直写官方编译值、视觉挂官方 STL 网格（geom 逐字照抄）。总质量 0.737243 kg 与官方逐位一致；落体/推倒穿透 -0.0002m（官方对照 -0.0137m）。D26 收敛。
```

## 问题

0.12m 高的鸭子在仿真里被推倒，躯干能穿进地板 11cm——比它整条腿还长。这是 M5 验收实测记录（偏差 D26，极端工况 z≈-0.111）。病灶在官方模型身上：碰撞几何是网格，大冲击下网格-平面接触修不回穿透。

M9 批次开局的处境：

1. **训练要用仿真身体**。M9 的目标是甩开现成 ONNX、自己训策略，身体模型是训练的地基。
2. **官方身体由 43 个 STL 网格定义**，许可是 CC BY-NC-SA（禁止商用）。走自定义硬件路线，物理地基不能建在受限资产上。
3. **网格接触本身有伤**（上面的穿透）。换基本几何体碰撞，顺手修掉 D26。

穿透不是渲染错误：仿真里的接触是弹簧-阻尼软约束，穿透量就是弹簧压缩量。为什么穿透合法、多少算合理，见深读页 [接触求解与穿透](../../concepts/contact-penetration.html)。

## 背景：MJCF 是什么

**【背景卡片】MJCF（MuJoCo XML 模型格式）**

- **一句话**：用 XML 把刚体树、关节、几何、惯量、执行器、传感器写成一个文件，物理引擎照着建模型。
- **为什么需要**：物理引擎不猜。质量多少、转轴朝哪、哪里会撞，全部要显式给出。
- **本项目在哪用**：`sim/assets/` 下官方模型与自建模型都是 MJCF；`sim/duck_body.py` 启动时加载场景 MJCF。

关键元件一张表：

| 元件 | 管什么 |
|---|---|
| `<body>` | 一个刚体，嵌套成树 |
| `<joint>` / `<freejoint>` | 关节；freejoint = 相对世界的 6 自由度 |
| `<geom>` | 几何体，分碰撞与视觉两种角色 |
| `<inertial>` | 质量、质心、惯量主轴（显式直写） |
| `<actuator>` | 执行器，本章用 position 位置伺服 |
| `<sensor>` / `<site>` | 传感器及其安装点 |
| `<keyframe>` | 存档姿势，一键复位用 |

最小示例（一个球 + 地板 + 一个存档姿势，需 mujoco）：

```python title="mjcf_demo.py"
import mujoco

xml = """
<mujoco>
  <worldbody>
    <geom type="plane" size="1 1 0.1"/>
    <body name="ball" pos="0 0 0.5">
      <freejoint/>
      <geom type="sphere" size="0.1" mass="0.5"/>
    </body>
  </worldbody>
  <keyframe>
    <key name="HOME" qpos="0 0 0.5 1 0 0 0"/>
  </keyframe>
</mujoco>
"""

m = mujoco.MjModel.from_xml_string(xml)
d = mujoco.MjData(m)
print(f"body 数={m.nbody}  关节数={m.njnt}  自由度={m.nv}")

for _ in range(100):          # 0.2 秒
    mujoco.mj_step(m, d)
print(f"0.2s 后球心 z={d.qpos[2]:.4f}（在往下掉）")

kid = mujoco.mj_name2id(m, mujoco.mjtObj.mjOBJ_KEY, "HOME")
mujoco.mj_resetDataKeyframe(m, d, kid)   # 一键回到存档
mujoco.mj_forward(m, d)
print(f"reset 回 keyframe 后 z={d.qpos[2]:.4f}")
```

实测输出（mujoco 3.15.0）：

```text
body 数=2  关节数=1  自由度=6
0.2s 后球心 z=0.3018（在往下掉）
reset 回 keyframe 后 z=0.5000
```

keyframe 就是「存档姿势」：记一组关节角，任何时刻一键复位。本章后面 STAND 存档是仿真体每次启动的站姿。深入阅读：[MuJoCo 官方 XML 参考](https://mujoco.readthedocs.io/en/stable/XMLreference.html)。

## 背景：惯量为什么是「3 个数 + 一个朝向」

**【背景卡片】惯量张量（压缩版）**

- **一句话**：刚体绕不同的轴转，「难推动」程度不同；完整描述要一个 3x3 矩阵，转到主轴方向后只剩 3 个对角数，主轴朝向再用一个四元数记录。
- **为什么需要**：仿真器做刚体积分时每步都要它；给错惯量，鸭子摔起来的姿势就不对。
- **本项目在哪用**：`duck_handmade.xml` 每个 body 一个 `<inertial mass=... pos=... quat=... diaginertia=...>`，四个字段分别对应质量、质心、主轴朝向、3 个主惯量。

这张卡是压缩版。卡片看不懂就先去读深读页 [惯量张量](../../concepts/inertia-tensor.html)，里面有从 3x3 矩阵到对角化的完整推导和可运行示例。卡里那句「主轴朝向用一个四元数记录」里的四元数（wxyz 顺序、为什么是半角、转 360° 怎么读），另有深读页 [四元数](../../concepts/quaternion.html)。

## 搭建实操：从空白文件到能跑的模型

结论一句话：**MJCF 不是代码，没有「构建」这一步——它是数据文件，「搭建」是一个五步循环**。改完 XML 任何一处，都走一遍同一圈，任何一步红了就回第 2 步：

```flow LR
[提取数据] -> [手写 XML] -> [编译验证] -> [眼睛验收] -> [行为验收]
[编译验证] -> [手写 XML]: 报错
[眼睛验收] -> [手写 XML]: 看着不对
[行为验收] -> [手写 XML]: 断言红
```

先说清五步各在哪台机器上跑：**提取、编译、行为验收在容器里，只有第 4 步眼睛验收在宿主机**。宿主机那份 Python 环境只为 viewer 而建，别拿它跑验收。

**第 1 步 · 从官方模型提取数据。** 自建模型里每个数字都要有出处，出处就是官方模型的编译输出（无头计算，容器内跑）：

```bash
docker compose exec -T rust /opt/sim-venv/bin/python scripts/m9a_extract_official.py
```

脚本打印官方关节树、逐 body 质量/质心/惯量、碰撞网格 AABB 尺寸，共 172 行。截取躯干两行：

```text
trunk_base: mass=0.199224 ipos=[-2.26332e-02 -2.70828e-05  2.80064e-03] inertia=[1.45931614e-04 1.40798872e-04 9.85265141e-05] iquat=[0.66486572 0.2383807  0.23501207 0.66775561]
=> trunk_base COLL-union lo=[-0.0469 -0.0273 -0.0442] hi=[-0.013   0.0273  0.0396] size=[0.0339 0.0546 0.0838]
```

`mass` `ipos` `inertia` `iquat` 四个数抄进 `<inertial>` 的四个字段，就是决策卡片 1 的「直写官方编译值」；碰撞 box 的 `size` 取 COLL-union 行打印值，就是决策卡片 2 的「包围盒」。这一步不抓错，但它是唯一「对」的数据源——手抖抄错一个数字，后面 B 组对账会红。

**第 2 步 · 手写 XML。** 纯文本编辑，无命令。骨架 + 逐 body 填四个惯量数 + box 尺寸 + 视觉 geom 行，具体写法见「代码领读」。

**第 3 步 · 编译验证。** MJCF 的「编译」就是把 XML 交给物理引擎建模型——语法错、网格路径错、引用不存在的名字，全在这一步报：

```bash
docker compose exec -T rust /opt/sim-venv/bin/python -c "import mujoco; m = mujoco.MjModel.from_xml_path('sim/assets/scene_handmade.xml'); print('body =', m.nbody, ' 关节 =', m.njnt, ' 执行器 =', m.nu)"
```

实测输出：

```text
body = 16  关节 = 15  执行器 = 14
```

比「15 刚体 14 关节」各多 1：worldbody 是引擎自带的世界根，躯干的 freejoint 本身也算一个关节。数字对不上，说明树结构写岔了。

**第 4 步 · 眼睛验收。** 编译过 ≠ 长得对。viewer 打开宿主机原生窗口，鼠标可直接拖拽推鸭子：

```bash
.venv/bin/python -m mujoco.viewer --mjcf=sim/assets/scene_handmade.xml
```

`.venv` 是仓库根的宿主机环境，只装 viewer 需要的东西（建法见 `docs/dev-container.md`「宿主机 viewer 环境」，里面也解释了为什么验收不跑它）。

本章「坑 5」的 8 处四元数符号抄错、右腿板件悬空，就是这步肉眼抓到的——位姿写错编译不报错，只有眼睛看得见。

**第 5 步 · 行为验收。** 眼睛只能看姿势对不对，物理对不对靠断言（同样在容器里跑）：

```bash
docker compose exec rust bash scripts/accept-m9a-bite1.sh
```

A 结构 / B 惯量 / C 贴地 / D 穿透四组，断言明细见「验收」一节。任何一组红，回第 2 步改 XML，重新走圈。

## 设计：惯量是数据，碰撞才讲形状，视觉只管好看

设计结论一句话：**惯量直写官方编译值，碰撞取包围盒，视觉照抄官方 STL**。数据、形状、外观三件事各管各的，互不污染。数据流水线：

```flow LR
(官方 robot_groundcontact.xml) -> [m9a_extract_official.py]: 编译，读 mjModel
[m9a_extract_official.py] -> [duck_handmade.xml]: 惯量值 + 包围盒尺寸
(官方 robot_groundcontact.xml) -> [duck_handmade.xml]: 视觉 geom 逐字照抄
[duck_handmade.xml] -> [scene_handmade.xml]: include 进场景
[scene_handmade.xml] -> [accept_m9a_bite1.py]: A-D 四组断言
(官方 robot_groundcontact.xml) --> [accept_m9a_bite1.py]: 对照基准
```

关节树与官方逐位对齐（trunk_base 三分叉，共 15 刚体 14 关节）：

```tree
trunk_base | 躯干，挂 freejoint + imu site
  yaw2roll | left_hip_yaw
    hip_l | left_hip_roll
      upper_leg_left | left_hip_pitch
        leg | left_knee
          ankle_left | left_ankle
  neck | neck_pitch
    neck_pitch | 挂 head_pitch（官方命名如此）
      yaw_roll_motion | head_yaw
        jaw_soft | head_roll
  bearing_roll | right_hip_yaw
    hip_l_2 | right_hip_roll
      upper_leg_right | right_hip_pitch
        leg_2 | right_knee
          ankle_right | right_ankle
```

**契约先行**：`duck_body.py` 不改一个字，但它点名依赖四个名字——STAND keyframe、`imu_gyro` 传感器、`imu` site、14 个关节名同序（`sim/duck_body.py:58-64`、`:171-173`）。策略权重按关节顺序读写，顺序错了动作就张冠李戴。自建模型的设计约束先由这份契约定死，再谈形状。

## 决策：数据、形状、外观各管各的

**【决策卡片 1】惯量从哪来：直写官方编译值，不让引擎从几何体算**

- **决策点**：每个 body 的质量/质心/惯量主轴怎么定。
- **备选**：A. 给 box 设密度，MuJoCo 从几何体自动算；B. 用 `<inertial>` 显式直写官方编译输出。
- **选择理由**：自建模型的碰撞几何是假的（包围盒），从假几何算惯量只能得到假惯量。官方 XML 是 Apache-2.0，编译输出是纯数值数据，可以直接用。验收对账误差约等于零（最差：质量 0.005%、质心 0.001mm）。
- **放弃的成本**：选 A 能白得「模型自洽」——改几何形状惯量自动跟着变。没选它，以后改 box 尺寸必须手工同步 `<inertial>`，忘了同步就会得到一对互相矛盾的形状与惯量。
- **失效边界**：将来换自研构型（M10 自定义硬件），官方惯量数据失效，要重新测量或计算。

**【决策卡片 2】碰撞几何：包围盒 box，不保真网格**

包围盒（AABB）= 把网格所有顶点的 x/y/z 各取 min/max 围成的盒子。它只大不小：任何方向都不会比原网格更「瘦」。

- **决策点**：碰撞几何用什么形状。
- **备选**：A. 沿用 STL 网格；B. 每个碰撞网格取包围盒写成 box。
- **选择理由**：碰撞层零 STL 绕开 CC BY-NC-SA——碰撞是物理的接触面，是训练真正依赖的部分。box-平面是解析求交，每个角点约束稳定；网格-平面靠采样接触点，大冲击修不回穿透（D26 的病灶）。实测：同条件推倒，官方 -0.0137m，自建 -0.0002m。
- **放弃的成本**：选 A 能白得接触点保真——脚掌曲面可以滚转、边缘圆润。没选它，脚是方盒子，边缘先着地可能改变步态；这条成本是否兑现，Bite 2 行走验收实测判定。
- **失效边界**：Bite 2 行走不达标时，退而求其次：把脚掌拆成多个 box 拼贴合形状。

**【决策卡片 3】视觉几何：照抄官方 STL，不把盒体坚持到底（2026-10-20 修订）**

初版本章连视觉也是盒体拼接，交付后用户裁决：太丑，改回官方 STL。

- **决策点**：视觉 geom（contype=0、conaffinity=0，纯观赏不进物理）用什么。
- **备选**：A. 每 body 一个包围盒 box，零 STL 到底；B. 官方视觉网格 geom 行逐字照抄。
- **选择理由**：盒体鸭丑到认不出部件，教学截图和演示失去对照价值。视觉 geom 不进物理——碰撞层规避 CC 资产的成果（决策卡片 2）完全不受影响。且本仓 BAM 默认路径（`scene_groundcontact.xml`）本来就加载同一批 STL，视觉层没有引入新的依赖面。
- **放弃的成本**：选 A 能白得「整个模型零受限资产」。没选它，视觉层挂 CC BY-NC-SA——教程衍生物若要商用分发，视觉资产需替换；物理核心（碰撞 + 惯量 + 关节树）仍是自建零 STL 数据。
- **失效边界**：将来需要整体摆脱受限资产（如商用整机），视觉网格也要自建，届时连材质配色一起重来。

## 代码领读

顺序跟着「设计」走：参数默认值 → 躯干（惯量直写 + 契约 + 视觉网格）→ 腿链 → 执行器 → 场景存档 → 验收探针。

关节与执行器参数三件套写在 default 类里，全树继承。damping/frictionloss/armature 与 position kp/forcerange 取官方 chosen_actuator 编译值。视觉/碰撞两类几何的分离也写在这几行里：visual 类是 mesh 不进接触，collision 类是 box：

```xml src=sim/assets/duck_handmade.xml lines=23-34 hl=25-26
```

躯干 body：freejoint 提供 6 自由度；`<inertial>` 四字段直写官方编译值；`imu` site 是 duck_body.py 读 gyro 的安装点；视觉 geom 是官方 STL 网格行照抄（下面只贴第一行，躯干共 10 行），碰撞盒 = 官方电池网格的包围盒：

```xml src=sim/assets/duck_handmade.xml lines=37-45 hl=39-41
```

左腿链第一节：body 的 pos/quat 是相对父体的安装位姿（官方编译值），关节 range 逐位对齐。整树 15 body 的文档序就是关节序，下游 velstand 权重按这个顺序读写。视觉网格行跟在关节声明后面：

```xml src=sim/assets/duck_handmade.xml lines=59-67 hl=62-63
```

14 个 position 执行器按 JOINT_NAMES 文档序逐个声明——ctrl 序即关节序（duck_body.py 启动时 assert 过）：

```xml src=sim/assets/duck_handmade.xml lines=330-345
```

场景包装：`imu_gyro` 传感器挂 imu site；STAND 存档的 14 个关节角与官方逐位一致。躯干高度 0.12 沿用官方——包围盒最底面就是网格最底面，着地点不变：

```xml src=sim/assets/scene_handmade.xml lines=31-43 hl=39-42
```

验收的穿透探针：遍历所有碰撞 geom 的 8 个角点，取世界系最低 z。包围盒几何让「最低点」可以解析枚举，不用采样。视觉网格 contype/conaffinity 全 0，探针天然跳过：

```python src=sim/accept_m9a_bite1.py lines=37-56 hl=49-52
```

「碰撞必须是 box」这条验收锁的是编译后的模型，不是 XML 文本。修订后文本里全是 `<mesh>`，但碰撞 geom 的类型集合仍然只能是 box：

```python src=sim/accept_m9a_bite1.py lines=96-103 hl=96-100
```

视觉网格同样按编译后对账：按（body, 网格名）配对，geom 位姿与官方编译值逐位比。坑 5 的 8 处符号错误就是这条断言抓出来的：

```python src=sim/accept_m9a_bite1.py lines=105-131 hl=131
```

## 验收

入口命令（容器内跑；脚本在容器内自动用 `/opt/sim-venv/bin/python`，在宿主机则回落到仓内 `.venv/bin/python`）：

```bash
docker compose exec rust bash scripts/accept-m9a-bite1.sh
```

四组断言，2026-10-20 宿主机实测（mujoco 3.16.0，视觉修订后复跑，全绿）：

| 组 | 断言 | 实测 |
|---|---|---|
| A 结构 | 碰撞 geom 全 box；视觉网格 geom=70 且位姿与官方编译值逐位对账；freejoint+14 hinge 同名同序；body 树同构；nu=14；range/damping/frictionloss/armature/kp/forcerange 全对齐 | ok 12/12 PASS（视觉对账最差误差 0.00e+00） |
| B 惯量 | 总质量 ≤3%；逐 body 质量 ≤5%、质心 ≤2mm、惯量 ≤15% | ok 总质量 0.737243 kg 两侧逐位一致；最差 0.005% / 0.001mm / 0.041% |
| C 贴地 | STAND 复位后脚掌最低点 \|z\| ≤5mm | ok z=+0.00065，与官方逐位一致 |
| D 穿透 | 落体+推倒稳定后最低碰撞角点 z ≥ -1mm | ok 自建 -0.0002 / -0.0002；官方对照 -0.0137 |

B 组阈值是防回归的代理假设：既然惯量直写官方编译值，误差本该约等于零，实测确实如此。C 组是「包围盒最底面 = 网格最底面」的直接证据。D 组官方对照组只记录不断言——-0.0137m 就是 D26 的可复现现场（M5 极端工况曾记录 z≈-0.111）。

回归：本 Bite 不动 duck_body.py 与 Rust 侧，旧验收不受影响。视觉修订（决策卡片 3）只加 contype=0/conaffinity=0 的 geom，B/C/D 组数值与修订前逐位一致。

## 与原版差异 · 常见坑与展望

偏差簿变动：

| 偏差 | 状态 | 机制 |
|---|---|---|
| D26 sim 推倒穿透地板 | ok 已收敛（本 Bite） | 包围盒 box 接触替掉网格-平面接触 |

无新登记。与官方模型的残余差异（不阻塞，留给后续 Bite 判定）：

| 差异 | 处置 |
|---|---|
| 碰撞盒比真实网格只大不小 | Bite 2 行走验收实测判定影响 |
| 视觉挂官方 STL（CC BY-NC-SA 4.0） | 用户裁决（2026-10-20）：外观优先；视觉不进物理，物理核心零 STL 成果保留；商用分发时需替换视觉资产 |
| 官方 trunk 无效 geom（contype/conaffinity=2 实际不产生接触） | 未复制 |
| 官方 RL 传感器组未移植（duck_body.py 只读 imu_gyro） | M9b 训练侧需要时再加 |

实测踩过的坑：

1. **关节文档序是接口**。打乱顺序模型照样编译。但策略权重按序读写，动作会张冠李戴。验收 A 组「同名同序」就是防这条。
2. **`<inertial>` 的 quat 是惯量主轴朝向**，不是 geom 姿态，也不是 body 姿态。三个朝向各管各的。
3. **包围盒只大不小是特性**。贴地验收靠的正是「最底面不变」；把盒子「修瘦」反而会改变着地点。
4. **断言语义要跟着裁决变**。视觉换 STL 后，原来的「XML 文本零 `<mesh>`」断言直接变红。它锁的是旧决策的实现痕迹，不是真实意图。真实意图是「碰撞必须是 box」。改成编译后枚举碰撞 geom 类型，断言才重新对准意图。
5. **四元数照抄会错在符号上，肉眼不可见**。视觉修订第一版转抄 70 行 geom，8 处 quat 符号抄错。XML diff 里不显眼，渲染出来右腿板件悬空。修复后把视觉位姿对账固化成 A 组断言，实测误差 0.00e+00，再犯当场变红。四元数怎么读、wxyz 顺序、符号约定（哪些位是负的）见深读页 [四元数](../../concepts/quaternion.html)。

展望：Bite 2 把 BAM 参数与 microduck_constants 对账，自建模型接 velstand 站立 + vx=0.8 走 10s 前移 ≥2.0m；Bite 3 仿真协议面对齐官方 duck-body。

## 自测

```html
<details>
<summary><b>1.</b> 你给某个 body 的碰撞盒调大了 2mm，忘了同步它的 &lt;inertial&gt;。模型行为会怎样？（A）惯量自动按新盒子重算　（B）惯量保持原值——它是直写的数据，与几何无关　（C）编译报错　（D）质量变成盒子的体积乘以密度</summary>
<p><b>答案：B。</b>显式 &lt;inertial&gt; 存在时 MuJoCo 不从几何算惯量（这正是决策卡片 1 的设计）。代价也在这里：形状与惯量从此是两份要手工同步的数据，A 和 D 描述的是没写 &lt;inertial&gt; 时的行为。</p>
</details>
<details>
<summary><b>2.</b> 假如当初保留官方 STL 网格碰撞（备选 A），下面哪个说法最准？（A）D26 照样能靠调 solref 收敛　（B）许可问题可以靠注明出处解决　（C）D26 病灶仍在（网格-平面接触大冲击修不回穿透），且 CC BY-NC-SA 依然禁止商用　（D）行走一定更稳，因为接触点保真</summary>
<p><b>答案：C。</b>穿透是网格接触几何层面的问题，调参数只能缓解不能根除（官方模型同条件实测 -0.0137m）；CC BY-NC-SA 的限制与是否注明出处无关。D 把「保真」与「稳定」混为一谈。</p>
</details>
<details>
<summary><b>3.</b> 有人把右腿 5 个关节在 XML 里挪到左腿前面（名字没改）。验收和运行时分别会发生什么？（A）编译失败　（B）A 组「同名同序」抓出；假如没有 A 组，velstand 权重按序读写会张冠李戴，鸭子抽风但无报错　（C）没影响，反正名字对得上　（D）只是渲染顺序变了</summary>
<p><b>答案：B。</b>MJCF 文档序决定关节序与 ctrl 序，duck_body.py 的 JOINT_NAMES 按序寻址（sim/duck_body.py:164-169）。名字全对、顺序错了，编译和加载都正常——错的是每个动作落到哪条腿上。</p>
</details>
<details>
<summary><b>4.</b> 脚掌包围盒比真实鞋底网格大一圈，STAND 贴地 z 为什么还能与官方逐位一致？（A）纯属巧合　（B）包围盒各方向只大不小，最底面就是网格最底面；官方 STAND 的躯干高度本来就按该着地点标定　（C）验收阈值 5mm 太松　（D）MuJoCo 自动吸附到地板</summary>
<p><b>答案：B。</b>AABB 的 min/max 取自网格顶点，最底面 z 与网格相同。官方 STAND 躯干高 0.12 按真实着地点标定，包围盒不改变最底面，所以逐位一致。这也是「只大不小」被列为特性的原因。</p>
</details>
<details>
<summary><b>5.</b> Bite 2 行走验收如果不达标，按决策卡片 2 的失效边界，下一步应该是什么？（A）碰撞改回 STL 网格　（B）把脚掌拆成多个 box 拼贴合形状　（C）调大 solref timeconst　（D）放弃自建模型</summary>
<p><b>答案：B。</b>失效边界写明：包围盒代价（方盒子脚）若兑现为行走不达标，退路是更细的基本几何体分解，而不是回到网格——回网格会把 D26 一起请回来。C 方向反了：调大 timeconst 是调软，穿透更大。</p>
</details>
<details>
<summary><b>6.</b> 视觉修订后验收 A 组为什么不断言「XML 里没有 &lt;mesh&gt;」了？（A）mesh 加载太慢，验收跳过　（B）那条断言锁的是实现痕迹；真实意图是「碰撞必须是 box」，编译后枚举碰撞 geom 类型才是对准意图的断言　（C）MuJoCo 不支持文本检查　（D）视觉网格不影响物理，所以什么都不用验</summary>
<p><b>答案：B。</b>文本级断言在视觉挂 STL 后必然变红，但模型并没有变坏——锁错了层。编译后检查「接触几何类型集合 == {box}」在两种视觉方案下都正确。D 错在放弃验证：碰撞层回退成网格正是要防的回归。</p>
</details>
```

## 细节展开

读者提问记录，一段一条，默认折叠。

```html
<details>
<summary><b>这一章没有构建命令，我到底学了什么？</b></summary>
<p>本章原本教的是「为什么这么搭」：三个设计决策（惯量直写官方编译值、碰撞包围盒、视觉照抄官方 STL）加一套验收方法——搭建动作当时全部发生在后台，一个命令都没写，这是本章的缺口。现已补入「搭建实操：从空白文件到能跑的模型」一节，把操作循环补全。关键是先建立一条认知：MJCF 是数据文件不是代码，没有「构建」这一步；搭建 = 提取数据 → 写 XML → 编译验证 → viewer 目检 → 行为验收的五步循环，每改一处走一圈。另外加了练习 Bite，让你从空白文件亲手搭一条 2 关节腿链，把这五步亲手走一遍。</p>
</details>
```

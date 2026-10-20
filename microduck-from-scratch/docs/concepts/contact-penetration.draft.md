---
template: doc
title: 接触求解与穿透：仿真的地板为什么会被踩穿
subtitle: 接触不是墙，是一根弹簧
---

一个 0.12m 高的小机器人在仿真里被推倒，躯干能穿透地板 11cm——比它的腿还长。这不是渲染错误，也不是网格坏了：物理引擎的「接触」本来就不是一面硬墙。这一页讲清接触力的来源，以及穿透多少算合理。

## 一句话版本

- 仿真里的接触是弹簧-阻尼器，不是硬墙。
- 物体陷进接触面一点，求解器拿这个穿透量算出推回去的力。
- 冲击越大、弹簧越软、步长越粗，陷得越深。
- MuJoCo 里 `solref` 管弹簧的软硬与回弹快慢，`solimp` 管阻抗随穿透深度的曲线。

## 为什么允许穿透

1. 真刚体不存在。现实中的接触都有微小变形，接触力来自变形。
2. 时间是离散的。每 5ms 才检查一次接触，发现时穿透已经发生。
3. 穿透量就是弹簧的压缩量。不允许穿透就没有接触力，物体永远感受不到地板。

所以穿透是模型的一部分，不是 bug。问题只剩下：陷多深算合理。答案由接触参数和几何决定——基本几何体（box、球、平面）之间的接触是解析求交，每个角点都有稳定的约束；网格（mesh）接触依赖采样出来的接触点，大冲击下容易失稳。

```svg
<svg width="660" height="300" viewBox="0 0 660 300" xmlns="http://www.w3.org/2000/svg">
  <rect width="660" height="300" fill="#14181f"/>
  <line x1="40" y1="220" x2="620" y2="220" stroke="#8b93a3" stroke-width="3"/>
  <g stroke="#8b93a3" stroke-width="1.5">
    <line x1="60" y1="220" x2="48" y2="238"/><line x1="110" y1="220" x2="98" y2="238"/>
    <line x1="160" y1="220" x2="148" y2="238"/><line x1="210" y1="220" x2="198" y2="238"/>
    <line x1="260" y1="220" x2="248" y2="238"/><line x1="310" y1="220" x2="298" y2="238"/>
    <line x1="360" y1="220" x2="348" y2="238"/><line x1="410" y1="220" x2="398" y2="238"/>
    <line x1="460" y1="220" x2="448" y2="238"/><line x1="510" y1="220" x2="498" y2="238"/>
    <line x1="560" y1="220" x2="548" y2="238"/><line x1="610" y1="220" x2="598" y2="238"/>
  </g>
  <text x="480" y="255" fill="#8b93a3" font-size="13">地板（平面 geom）</text>
  <g>
    <rect x="280" y="150" width="100" height="70" fill="#d8a018" rx="3">
      <animate attributeName="y" values="60;60;205;214;205;60;60" keyTimes="0;0.35;0.55;0.62;0.7;0.9;1" dur="3s" repeatCount="indefinite"/>
    </rect>
    <rect x="280" y="220" width="100" height="0" fill="#e25555" opacity="0.55">
      <animate attributeName="height" values="0;0;0;6;0;0;0" keyTimes="0;0.35;0.55;0.62;0.7;0.9;1" dur="3s" repeatCount="indefinite"/>
    </rect>
  </g>
  <line x1="330" y1="255" x2="330" y2="290" stroke="#4f9cf7" stroke-width="2">
    <animate attributeName="opacity" values="0;0;0;1;0;0" keyTimes="0;0.5;0.58;0.62;0.72;1" dur="3s" repeatCount="indefinite"/>
  </line>
  <text x="345" y="280" fill="#4f9cf7" font-size="13">接触力 ∝ 穿透量
    <animate attributeName="opacity" values="0;0;0;1;0;0" keyTimes="0;0.5;0.58;0.62;0.72;1" dur="3s" repeatCount="indefinite"/>
  </text>
  <text x="40" y="40" fill="#c8cfd9" font-size="14">红条 = 穿透（被夸张画大）；穿透出现，接触力才出现</text>
</svg>
```

## solref：两个数管软硬

`solref = (timeconst, dampratio)`，描述接触弹簧怎么把穿透「修」回去：

| 参数 | 含义 | 调小的效果 |
|---|---|---|
| `timeconst` | 修正时间常数（秒） | 弹簧更硬、穿透更小，但要求步长够细，否则抖动 |
| `dampratio` | 阻尼比 | 1 = 临界阻尼不弹；小于 1 会回弹 |

`solimp` 是阻抗曲线：同一个穿透量在「刚陷进去」和「陷得很深」时产生的力不同，五个数控制这条曲线的形状。默认值对大多数场景够用，穿透出问题先查 `solref` 和步长。

## 最小示例

一个 1 kg 的盒子以 4 m/s 砸向地板，只改 `solref` 的 timeconst，看冲击中最深陷多少。脚本在 `assets/contact-penetration/solref_demo.py`（需 mujoco）：

```python title="solref_demo.py"
import mujoco

TEMPLATE = """
<mujoco>
  <option timestep="0.005"/>
  <worldbody>
    <geom name="floor" type="plane" size="0 0 0.1" solref="{sr}"/>
    <body name="box" pos="0 0 0.3">
      <freejoint/>
      <geom type="box" size="0.05 0.05 0.05" mass="1.0" solref="{sr}"/>
    </body>
  </worldbody>
</mujoco>
"""

for tag, sr in [("软接触 (0.10, 1)",  (0.10, 1.0)),
                ("默认   (0.02, 1)",  (0.02, 1.0)),
                ("硬接触 (0.002, 1)", (0.002, 1.0))]:
    m = mujoco.MjModel.from_xml_string(TEMPLATE.format(sr=f"{sr[0]} {sr[1]}"))
    d = mujoco.MjData(m)
    d.qvel[2] = -4.0                      # 4 m/s 向下初速度，模拟大冲击
    worst = 0.0
    for _ in range(int(2.0 / m.opt.timestep)):
        mujoco.mj_step(m, d)
        worst = min(worst, d.qpos[2] - 0.05)   # 盒子底面 z
    print(f"{tag}: 冲击中底面最深 z = {worst:+.4f} m")
```

实测输出（mujoco 3.15.0）：

```text
软接触 (0.10, 1): 冲击中底面最深 z = -0.1549 m
默认   (0.02, 1): 冲击中底面最深 z = -0.0236 m
硬接触 (0.002, 1): 冲击中底面最深 z = -0.0091 m
```

同一个盒子、同一个冲击，只改接触软硬，穿透深度差 17 倍。「穿透多少」首先是参数问题，其次才是几何问题。

## 素材

- MuJoCo 官方文档接触建模（solref/solimp 的数学定义）：[mujoco.readthedocs.io Computation · Contact](https://mujoco.readthedocs.io/en/stable/computation/index.html#contact)
- 中文参数解读（弹簧振子模型推导）：[知乎：Mujoco 中 solimp 和 solref 参数](https://zhuanlan.zhihu.com/p/618727893)
- 面向调参的英文心智模型：[ROBOLAWEB：MuJoCo Solver Guide: Solref & Solimp](https://robolaweb.gitbook.io/robolaweb-docs/basic-concept/mujoco-solver-guide-solref-and-solimp)

## 自测

```html
<details>
<summary><b>1.</b> 同事看到仿真里盒子陷进地板 1mm，说是 bug 要修。你怎么回应？（A）对，刚体不该穿透，必须修到 0　（B）穿透是接触力的来源，1mm 量级合理，要查的是穿透是否随冲击失控　（C）把地板加厚 1mm 补偿　（D）关掉碰撞重开</summary>
<p><b>答案：B。</b>软约束模型里穿透量就是弹簧压缩量，为零意味着没有接触力。合理的验收标准是「给定冲击下穿透有界且可复现」，不是零。A 会把人带进无限调参的死胡同。</p>
</details>
<details>
<summary><b>2.</b> 同一模型，推倒时穿透 14mm。把 solref 的 timeconst 从 0.02 调到 0.10，穿透会怎么变？（A）变小　（B）变大——timeconst 大是软接触，修正慢　（C）不变，solref 只管摩擦　（D）变负得更多，因为时间常数放大误差</summary>
<p><b>答案：B。</b>timeconst 是修正时间常数：越大越软，弹簧把穿透修回去越慢，冲击中陷得越深。实测数据里 0.10 比 0.02 的穿透深 6 倍。方向记反是最常见的调参错误。</p>
</details>
<details>
<summary><b>3.</b> 同样冲击下，mesh（网格）脚 vs box（盒子）脚，哪个穿透更可控？（A）mesh，三角面贴合真实　（B）box，基本几何体与平面是解析求交，每个角点约束稳定　（C）一样，接触求解器不区分几何类型　（D）看渲染效果决定</summary>
<p><b>答案：B。</b>box-plane 的接触点解析可算，求解器拿到的是完整稳定的约束集。mesh 接触依赖从三角面采样的接触点，大冲击下接触点集合剧变，容易修不回来。保真度和稳定性是两回事。</p>
</details>
<details>
<summary><b>4.</b> 为了压穿透，把 timeconst 调到 0.0001（极硬）而不动步长，最可能出现什么？（A）穿透归零且无副作用　（B）接触抖动、能量爆炸——弹簧太硬，5ms 步长积分不动它　（C）摩擦消失　（D）模型加载失败</summary>
<p><b>答案：B。</b>硬弹簧配粗步长是数值不稳定的经典组合：每一步都过冲，表现为接触处抖动甚至发散。压穿透的正确顺序是先减小步长或检查几何，再调接触参数。</p>
</details>
```

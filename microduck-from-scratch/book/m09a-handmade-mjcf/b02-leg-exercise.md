---
template: sheet
title: M9a 动手练习：手搓一条 2 关节腿链（第 19 章）
subtitle: 从空白文件写到能跑的 MJCF：契约、三组验收、六个破坏实验
source: clone-from-scratch-tutor · 第 19 章
---

交付一个文件：`sim/assets/exercise_leg.xml`——一条 2 关节腿链。

验收脚本替你查三组断言：结构、执行器跟踪、keyframe 还原。全绿才算过。

写完这一个文件，第 18 章那套五步循环你就亲手走了一遍。

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="620" height="430" viewBox="0 0 620 430" font-family="system-ui, -apple-system, sans-serif">
  <rect width="620" height="430" fill="#11151c"/>
  <text x="150" y="30" fill="#e8ecf2" font-size="15" text-anchor="middle" font-weight="600">STRAIGHT</text>
  <text x="150" y="48" fill="#8b97a8" font-size="11" text-anchor="middle">qpos = 0 0</text>
  <text x="410" y="30" fill="#e8ecf2" font-size="15" text-anchor="middle" font-weight="600">跟踪目标</text>
  <text x="410" y="48" fill="#8b97a8" font-size="11" text-anchor="middle">hip +0.5 / knee -0.5</text>

  <line x1="30" y1="340" x2="590" y2="340" stroke="#5b6472" stroke-width="2"/>
  <g stroke="#39404c" stroke-width="1">
    <line x1="30" y1="340" x2="44" y2="352"/><line x1="70" y1="340" x2="84" y2="352"/>
    <line x1="110" y1="340" x2="124" y2="352"/><line x1="150" y1="340" x2="164" y2="352"/>
    <line x1="190" y1="340" x2="204" y2="352"/><line x1="230" y1="340" x2="244" y2="352"/>
    <line x1="270" y1="340" x2="284" y2="352"/><line x1="310" y1="340" x2="324" y2="352"/>
    <line x1="350" y1="340" x2="364" y2="352"/><line x1="390" y1="340" x2="404" y2="352"/>
    <line x1="430" y1="340" x2="444" y2="352"/><line x1="470" y1="340" x2="484" y2="352"/>
    <line x1="510" y1="340" x2="524" y2="352"/><line x1="550" y1="340" x2="564" y2="352"/>
  </g>
  <text x="588" y="358" fill="#8b97a8" font-size="11" text-anchor="end">地板 plane</text>

  <rect x="142" y="62" width="16" height="16" fill="#8a8f99" stroke="#0b0e13"/>
  <rect x="135.5" y="70" width="29" height="90" fill="#3f7fd0" stroke="#0b0e13"/>
  <rect x="139" y="160" width="22" height="90" fill="#e08a4a" stroke="#0b0e13"/>
  <circle cx="150" cy="70" r="5" fill="#11151c" stroke="#e8ecf2" stroke-width="1.5"/>
  <circle cx="150" cy="70" r="1.6" fill="#e8ecf2"/>
  <circle cx="150" cy="160" r="5" fill="#11151c" stroke="#e8ecf2" stroke-width="1.5"/>
  <circle cx="150" cy="160" r="1.6" fill="#e8ecf2"/>
  <text x="172" y="66" fill="#e8ecf2" font-size="11">hip_pitch</text>
  <text x="172" y="156" fill="#e8ecf2" font-size="11">knee</text>
  <text x="170" y="120" fill="#8b97a8" font-size="10">5 cm</text>
  <line x1="150" y1="250" x2="150" y2="340" stroke="#4c5563" stroke-width="1" stroke-dasharray="3 3"/>
  <text x="142" y="300" fill="#8b97a8" font-size="10" text-anchor="end">离地 5 cm</text>

  <rect x="402" y="62" width="16" height="16" fill="#8a8f99" stroke="#0b0e13"/>
  <rect x="395.5" y="70" width="29" height="90" fill="#3f7fd0" stroke="#0b0e13" transform="rotate(28.6479 410 70)"/>
  <rect x="355.9" y="149" width="22" height="90" fill="#e08a4a" stroke="#0b0e13"/>
  <circle cx="410" cy="70" r="5" fill="#11151c" stroke="#e8ecf2" stroke-width="1.5"/>
  <circle cx="410" cy="70" r="1.6" fill="#e8ecf2"/>
  <circle cx="366.9" cy="149" r="5" fill="#11151c" stroke="#e8ecf2" stroke-width="1.5"/>
  <circle cx="366.9" cy="149" r="1.6" fill="#e8ecf2"/>
  <line x1="410" y1="70" x2="410" y2="340" stroke="#39404c" stroke-width="1" stroke-dasharray="3 3"/>
  <text x="430" y="96" fill="#e8ecf2" font-size="11">0.5 rad = 28.6°</text>
  <text x="430" y="112" fill="#8b97a8" font-size="10">大腿倒向 -x</text>
  <text x="346" y="195" fill="#8b97a8" font-size="10" text-anchor="end">小腿仍竖直</text>

  <rect x="40" y="382" width="13" height="13" fill="#3f7fd0" stroke="#0b0e13"/>
  <text x="60" y="393" fill="#c8d0dc" font-size="11">thigh 大腿</text>
  <rect x="165" y="382" width="13" height="13" fill="#e08a4a" stroke="#0b0e13"/>
  <text x="185" y="393" fill="#c8d0dc" font-size="11">shank 小腿</text>
  <rect x="290" y="382" width="13" height="13" fill="#8a8f99" stroke="#0b0e13"/>
  <text x="310" y="393" fill="#c8d0dc" font-size="11">mount 挂载点</text>
  <circle cx="440" cy="388" r="6" fill="none" stroke="#8b97a8" stroke-width="1.2"/>
  <circle cx="440" cy="388" r="1.8" fill="#8b97a8"/>
  <text x="452" y="393" fill="#c8d0dc" font-size="11">转轴 = y 轴（指向纸外）</text>
</svg>
```

## 任务书：一个文件，一条腿

**交付物就一个文件**：`sim/assets/exercise_leg.xml`，一条 2 关节腿链，悬空挂在挂载点上。

先看它长什么样。左边是 `STRAIGHT` 存档姿势，qpos 全 0。右边是 B 组要它追到的角度：大腿转 +0.5 rad，小腿转 -0.5 rad。两者抵消，小腿在世界系里仍竖直。

契约一共七条。这七条是验收脚本逐条检查的东西，也是下游代码读这个模型时依赖的接口。

| 契约 | 约定 |
|---|---|
| 关节名 | `hip_pitch`（大腿）、`knee`（小腿） |
| 关节类型 | 两个都是 `hinge`，轴都是 `0 1 0` |
| ctrl 序 | `ctrl[0]` 管 hip，`ctrl[1]` 管 knee（声明顺序即下标顺序） |
| qpos 序 | `qpos[0]` 是 hip_pitch，`qpos[1]` 是 knee（body 嵌套顺序即关节顺序） |
| 几何 | 每个动力 body 一个 `box` geom，不用 mesh |
| 地板 | worldbody 下必须有一块 `plane` |
| 存档 | keyframe 名字 `STRAIGHT`，qpos 全 0 |

body 嵌套层级就是身体的连接关系，缩进即关节链：

```tree list
worldbody
  floor | 地板，plane
  mount | 静态挂载点，body 里不写 joint
    thigh | 大腿，joint hip_pitch
      thigh_box | box geom，1.6cm 见方 × 5cm 长
      shank | 小腿，joint knee
        shank_box | box geom，1.2cm 见方 × 5cm 长
```

从下面这段骨架开始写。标 `TODO` 的字段自己定，其余照抄：

```xml title="exercise_leg.xml 骨架"
<mujoco model="exercise_leg">
  <compiler angle="radian" autolimits="true" />

  <worldbody>
    <geom name="floor" type="plane" size="2 2 0.1" />
    <body name="mount" pos="0 0 TODO_MOUNT_HEIGHT">
      <body name="thigh" pos="0 0 0">
        <joint name="hip_pitch" type="hinge" axis="0 1 0"
               damping="TODO_DAMPING" />
        <geom name="thigh_box" type="box" pos="0 0 TODO_THIGH_CENTER"
              size="TODO_W TODO_W TODO_HALF_LEN" density="TODO_DENSITY" />
        <body name="shank" pos="0 0 TODO_SHANK_OFFSET">
          <joint name="knee" type="hinge" axis="0 1 0" damping="TODO_DAMPING"
                 range="TODO_RANGE" />
          <geom name="shank_box" type="box" pos="0 0 TODO_SHANK_CENTER"
                size="TODO_W TODO_W TODO_HALF_LEN" density="TODO_DENSITY" />
        </body>
      </body>
    </body>
  </worldbody>

  <actuator>
    <position name="hip_pitch_servo" joint="hip_pitch"
              kp="TODO_KP" forcerange="TODO_FR" />
    <position name="knee_servo" joint="knee"
              kp="TODO_KP" forcerange="TODO_FR" />
  </actuator>

  <keyframe>
    <key name="STRAIGHT" qpos="0 0" />
  </keyframe>
</mujoco>
```

`TODO` 处的取值有个舒适区。下表的「建议」栏来自本次验收实测，参考解答的取值在第三栏。你可以照抄，也可以挑别的。

| 量 | 建议 | 参考解答 | 备注 |
|---|---|---|---|
| 关节阻尼 damping | 0.01 ~ 0.1 | 0.02 | 写 0 会数值发散 |
| 位置伺服 kp | 1 ~ 20 | 5 | 稳态误差 ≈ 重力矩 / kp |
| 力矩上限 forcerange | ±0.05 ~ ±0.3 | ±0.3 | 要大于重力矩（约 0.01 N·m） |
| 大腿 | 长 5cm、见方 1.6cm | 同左 | 几何即真实形状 |
| 小腿 | 长 5cm、见方 1.2cm | 同左 | |
| 密度 density | 200 ~ 5000 | 1000 | 整档实测全过 |
| 挂载高度 | 0.15 ~ 0.20m | 0.15 | 要高于整条腿长，留出离地间隙 |

box 的 `size` 写半长。见方 1.6cm 写 `0.008`，长 5cm 写 `0.025`。

写完走一遍[第 18 章的五步循环](tutorial-bite1-handmade-mjcf.html#panel-D)：编译验证 → 眼睛验收 → 行为验收。本章下面两节就是第 4 步和第 5 步的命令。

## 怎么跑：验收在容器，眼睛在宿主机

**两个动作，两个地方。** 验收脚本在容器里跑，viewer 在宿主机跑。

```bash
# 验收（容器内）
docker compose exec rust bash scripts/accept-m9a-leg-exercise.sh              # 你的作业
docker compose exec rust bash scripts/accept-m9a-leg-exercise.sh --solution   # 参考解答
```

```bash
# 看它甩（宿主机）
.venv/bin/python -m mujoco.viewer --mjcf=sim/assets/exercise_leg.xml
```

仓库根的 `.venv` 只为 viewer 而建，验收别在宿主机跑。脚本自带解释器探测：容器内用 `/opt/sim-venv/bin/python`，宿主机回落仓内 `.venv/bin/python`，两个都没有就报错退出。

退出码告诉你现在处在哪一步：

| 退出码 | 含义 | 屏幕上 |
|---|---|---|
| 0 | 三组断言全绿 | 末行「腿链练习验收 PASS（作业）：结构 / 跟踪 / keyframe 全绿」 |
| 1 | 编译失败或断言红 | 逐条 `[FAIL]`，每条下面跟一行「→ 怎么办」 |
| 2 | 作业文件还没建 | 「找不到文件 …/sim/assets/exercise_leg.xml」+「还没创建作业文件……」 |

一轮的顺序是这样，红了按提示回改，改完再跑：

```flow LR
[写 exercise_leg.xml] -> [容器内跑验收]
[容器内跑验收] -> [宿主机 viewer 目检]: 退出码 0
[容器内跑验收] -> [照「→ 怎么办」改]: 退出码 1
[容器内跑验收] -> [先建文件]: 退出码 2
```

## 验收标准：三组断言各管什么

**全绿的意思是：结构对、腿听话、存档准。** 三组一共 21 条断言，A 组 17 条、B 组 3 条、C 组 1 条，下面按组归纳。

A 组（结构）检查模型长成契约写的样子：

| 编号 | 断言 | 通过意味着什么 |
|---|---|---|
| A1 | 两个动力 body 叫 thigh / shank，shank 嵌在 thigh 里 | 关节链的连接关系对：膝关节连的是大腿和小腿 |
| A2 | 关节名、类型、轴、ctrl 序、qpos 序、nu=2、碰撞 geom 全 box、有地板 | 下游按数组下标读写动作，下标错就张冠李戴；碰撞全 box 保证几何即实形 |
| A3 | knee 的限位覆盖目标角 -0.5 rad，存在 STRAIGHT keyframe | 关节声明的区间装得下验收要的角度，存档姿势在位 |

B 组（执行器跟踪）检查腿真的听话：

| 编号 | 断言 | 通过意味着什么 |
|---|---|---|
| B1 | 2s 后 qpos 全是有限数 | 仿真没有数值爆炸 |
| B2 | hip 到 +0.5 ±0.1 rad、knee 到 -0.5 ±0.1 rad | 执行器把关节拉到了指令角。参考解答实测 +0.4994（误差 0.0006）和 -0.5000（误差 0.0000） |

C 组（keyframe）检查存档姿势能精确复位：

| 编号 | 断言 | 通过意味着什么 |
|---|---|---|
| C1 | 扰动 qpos 后回 STRAIGHT，qpos 逐位一致 | 起始状态可复现，训练与回放才对得上 |

阈值 ±0.1 rad 定得很宽。它只抓三类坏：执行器没接上、方向反了、力矩被掐死。你的参数随便选，只要落进上一张表的舒适区就行。

## 对照点：这次为什么不用手写惯量

**练习里一个 `<inertial>` 都不写，第 18 章的 `duck_handmade.xml` 却每个 body 都有。** 差别只有一个：几何是不是真的形状。

| | 第 18 章 duck_handmade.xml | 本练习 exercise_leg.xml |
|---|---|---|
| 碰撞几何 | 官方网格的包围盒，假的 | 一个 box 就是腿的实形，真的 |
| 惯量来源 | 直写官方编译值 | 引擎按 density × 几何自己算 |
| 为什么 | 从假几何算惯量，得到的是假惯量 | 几何即实形，算出来的就对了 |
| 质量 | 官方值，逐位对账 | density=1000 时大腿 12.8g、小腿 7.2g |

判据一句话：**几何真不真，决定惯量能不能交给引擎算。**

这也是 A 组那条「碰撞 geom 全 box」的意义所在。换成 mesh 或者随便什么形状，你就没法用「几何即实形」这条理由省掉惯量了。

## 破坏实验：改一个数，看系统怎么坏

**每个实验改一个数、跑一条命令、看一个现象。** 下面六组数字都是容器内实测的（mujoco 3.16.0），不是推测。

| 实验 | 改哪个数 | 跑哪条命令 | 预期现象（含实测） |
|---|---|---|---|
| 1 阻尼放大 10 倍 | `damping` 0.02 → 0.2 | 验收 + viewer | 验收照样全绿（退出码 0），腿却变肉。阶跃到 0.45 rad 从 17 步（0.034s）变成 152 步（0.304s），过冲 +0.0103 rad 消失；自由摆动停稳从 16.6s 变成 30s 仍未停 |
| 2 阻尼写 0 | `damping` = 0 | 验收 | MuJoCo 报 "The simulation is unstable"，数据被重置回初始态。hip 和 knee 都停在 +0.0000，误差 0.5000，B 组红 2 项，退出码 1 |
| 3 力矩掐死 | `forcerange` ±0.3 → ±0.001 | 验收 | hip 只到 +0.0743（误差 0.4257），knee -0.0985。B 组红 2 项，退出码 1 |
| 4 ctrl 序写反 | 两个 `<position>` 换行 | 验收 | A 组红「ctrl 序 = hip 后 knee，实际 ['knee','hip_pitch']」；hip 收到 -0.4994、knee 收到 +0.4921，共 3 项红，退出码 1 |
| 5 删掉 knee 执行器 | 删掉 knee 那行 `<position>` | 验收 | A 组红「nu=2 实际 nu=1」，B 组红「跟踪实验可运行」，共 2 项红，退出码 1 |
| 6 限位收太紧 | knee `range` -1.4 0.05 → -0.2 0.05 | 验收 | A 组红「knee 限位覆盖跟踪目标 -0.5 rad」，B 组却绿（knee 实测 -0.4947）。退出码 1 |

实验 6 挖出一条反直觉的事：**限位是软约束，伺服器能把它压穿。** 同一条限位（-1.4 rad），执行力矩越大，越压不住它：

| forcerange | ±0.3 | ±0.1 | ±0.05 | ±0.02 | ±0.01 | ±0.005 |
|---|---|---|---|---|---|---|
| knee 最终角（命令 -2.0） | -1.9891 | -1.9891 | -1.9362 | -1.5849 | -0.9762 | -0.4924 |

力矩掉到小于 ±0.01 时，伺服器干脆拉不动腿了，关节停在半路。限位写不写、限在哪里，管的是「声明」；关节实际停在哪里，管的是「力矩余量」。

还有一个角度单位坑：MJCF 的 `range` 默认按角度解析。不写 `angle="radian"`，`-1.4` 会被当成 -1.4 度，编译后是 -0.0244 rad。此时 A 组红「knee 限位覆盖跟踪目标 -0.5 rad」，退出码 1。

## 自测

```html
<details>
<summary><b>1.</b> 你把 knee 的 range 从 -1.4 0.05 收到 -0.2 0.05，验收 A 组红了、B 组却绿。为什么？（A）range 只是注释，不影响物理　（B）限位是软约束，kp=5 的伺服器把关节压过了限位；声明的区间与实际停不停得住是两件事　（C）B 组的 ±0.1 阈值太松　（D）MuJoCo 根本不读 range 属性</summary>
<p><b>答案：B。</b>实测 range=-0.2 0.05 时 knee 仍然到了 -0.4947 rad，越过限位 0.29 rad。range 是约束求解器里的软约束，力矩够大就压得穿。C 把两组断言的职责搞反了，D 与实测矛盾：把 range 改错单位时 A 组立刻变红。</p>
</details>
<details>
<summary><b>2.</b> 参考解答一个 &lt;inertial&gt; 都不写，第 18 章的 duck_handmade.xml 每个 body 都有。差别在哪？（A）练习的盒子更小　（B）练习里 box 就是腿的实形，惯量可由 density 算出；第 18 章的 box 是包围盒，几何是假的，惯量必须直写官方值　（C）第 18 章的作者偷懒　（D）两章的 MuJoCo 版本不同</summary>
<p><b>答案：B。</b>判据是「几何真不真」。第 18 章的碰撞盒是官方网格的包围盒，从假几何算惯量只会得到假惯量，所以直写编译值；练习里每个 box 就是腿的实形，density=1000 时引擎算出的 thigh 12.8g、shank 7.2g 就是对的。</p>
</details>
<details>
<summary><b>3.</b> 你把两个 &lt;position&gt; 的声明顺序换了。模型照样编译，viewer 照样能动，真正坏掉的是哪一层？（A）渲染　（B）ctrl 数组下标与关节的对应关系：ctrl[0] 管到 knee 去了　（C）质量与惯量　（D）地板的高度</summary>
<p><b>答案：B。</b>实测 ctrl 序写反后 hip 收到 -0.4994、knee 收到 +0.4921，指令与动作整体对调。MJCF 的声明顺序就是数组下标顺序，这类错编译期不报错，只有靠断言或肉眼发现。</p>
</details>
<details>
<summary><b>4.</b> damping 从 0.02 调到 0.2，验收全绿，但腿明显变肉。哪个说法对？（A）阻尼只影响渲染　（B）阻尼消耗的能量变少，同样的扰动要更久才停；验收只看 2s 末的角度，抓不到这个差别　（C）阻尼改变了刚体质量　（D）阻尼会改写关节限位</summary>
<p><b>答案：B。</b>实测阶跃到 0.45 rad 从 17 步变成 152 步，自由摆动停稳从 16.6s 变成 30s 仍未停，而 2s 末的落点两者都到 +0.4994。这条就是「验收只覆盖它断言的那几件事」。</p>
</details>
<details>
<summary><b>5.</b> 作业文件还没建就跑了验收，退出码 2、屏幕提示「还没创建作业文件」。改到 XML 语法写错时，退出码是多少？（A）0　（B）1　（C）2　（D）3</summary>
<p><b>答案：B。</b>退出码 2 专指「作业文件不存在」。XML 编译失败走 1，脚本先打印报错行号再提示常见原因：标签没闭合、属性名拼错、MuJoCo 3.x 下写了 range 却忘了 autolimits。</p>
</details>
```

## 参考解答：卡住时的对照清单

```html
<details>
<summary><b>卡住了再看：对照 <code>sim/assets/exercise_leg_solution.xml</code>，先自己查这五处</b></summary>
<p>1. 关节嵌套层级：<code>shank</code> 要写在 <code>thigh</code> 内部，不是平级。</p>
<p>2. ctrl 声明顺序：<code>&lt;actuator&gt;</code> 里 hip 的 <code>&lt;position&gt;</code> 写在 knee 前面。</p>
<p>3. knee 的 range 要覆盖跟踪目标 -0.5 rad。</p>
<p>4. kp 与 forcerange 的量级：kp 取个位数，forcerange 要大于重力矩（约 0.01 N·m）。</p>
<p>5. keyframe 的 qpos 个数 = 关节数，也就是 2 个。</p>
</details>
```

## 细节展开

读者提问留白。本章暂未收到读者提问。上一轮那条「想学仿真模型怎么搭建，但里面没有构建命令」记在[第 18 章的细节展开](tutorial-bite1-handmade-mjcf.html#panel-K)，不重复记在这里。

```html
<details>
<summary><b>常见卡点：三条验收抓不住的坑</b></summary>
<p>验收只断言它写下来的那几件事。这三处它抓不住，得靠自己守。</p>
<p>一、阻尼放大十倍，验收照样全绿，因为 B 组只看 2s 末的角度。要看差别，跑上面的实验 1。</p>
<p>二、限位收太紧，A 组红、B 组绿。要看软约束的完整行为，看 forcerange 扫描那张表。</p>
<p>三、knee 干脆不写 range，A 组也放行。那条断言的语义是「限位区间要覆盖目标角」，关节没有区间时它没有可判的东西。关节能折多深，是你的契约，不是脚本替你守的。</p>
</details>
```

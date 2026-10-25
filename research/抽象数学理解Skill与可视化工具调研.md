# 抽象数学理解 Skill 与可视化工具调研

> 调研日期：2026-10-24。Star、fork、最后推送取自当天 GitHub API。
> 需求：读邱锡鹏《神经网络与深度学习》时，一句抽象话落地不了（例子：「行列式是空间体积的伸缩」）。希望先在二维、三维里数形结合，用一个能手算的例子对上式子，而不是把推导拆成记忆卡片。
> 已有笔记：同目录 [`家教类Skill调研.md`](家教类Skill调研.md)（通用家教，2026-10-23）、[`manim与画图类Skill探索复盘.md`](manim与画图类Skill探索复盘.md)（2026-10-14 成片，2026-10-20 不装 Manim 的 canvas 成片）。本文不重复那两篇的安装踩坑。

## 结论

没有一个该设成默认的「数学教师」技能。抽象句子按这个顺序处理：

| 顺序 | 用什么 | 什么时候停 |
|---|---|---|
| 1 | [3Blue1Brown《线性代数的本质》](https://www.3blue1brown.com/topics/linear-algebra) | 标准线代概念（行列式、特征值、零空间、基变换）已有成片，先看再回到自己的数 |
| 2 | 本机 `explain-it`，教法按 `math-intuition-builder` 收窄 | 仍没有画面。只要一个 \(2\times2\) 或 \(3\times3\) 数值例子，面积或体积和式子对得上 |
| 3 | 本机已验证的 canvas HTML | 这个例子要留下来拖着看。中文、坐标轴、真实数值用这条，不装 Manim |
| 4 | `manim-composer` 再 `manim-video` | 重点是公式本身在变形，并且明确说要视频 |

默认不装：`sigma`（会把一句话开成一门课）、整套 [panpanc/math-skills](https://github.com/panpanc/math-skills)（只值得抄 `abstraction-levels` 的梯子）、[MathLens](https://github.com/sagittaprolifer245/MathLens)（题目进、配音视频出，比一个可核对的例子重）。

父仓库的 star 不等于其中某一个 skill 的采用量。下表「仓库」一列是整个仓库。

## 收藏清单

学习仓库的引用表在 `~/Project/learning/skills/README.md`。这里是同一批条目的出处。内容本体不拷进本仓库。

### 把概念讲懂

| 名称 | 仓库 | 2026-10-24 | 协议 | 取哪一块 |
|---|---|---|---|---|
| explain-it | 本机 `~/Project/skills/skills/explain-it/` | 自有 | 自有 | 整个 skill。先诊断「没画面 / 缺前置 / 连不成结构」 |
| math-intuition-builder | [parcadei/Continuous-Claude-v3](https://github.com/parcadei/Continuous-Claude-v3) | 3947 star / 302 fork，推送 2026-01-26 | MIT | 只拷 `.claude/skills/math/math-intuition-builder/`。3947 是整个上下文管理仓库，不是这个 skill |
| abstraction-levels | [panpanc/math-skills](https://github.com/panpanc/math-skills) | 39 star / 2 fork，推送 2026-07-18 | API 未标协议 | 只用这一个 skill。同仓库还有十几支，会写成长文课程 |
| explain-math-intuition | [tgkprog/AI-snippets](https://github.com/tgkprog/AI-snippets/blob/main/Skill.md) | 0 star，推送 2026-03-18 | LGPL-2.1 | 单文件。工程直觉框架，不画三维 |
| math-explain | [gyy0592/claude-config](https://github.com/gyy0592/claude-config/blob/main/skills/math-explain/SKILL.md) | 7 star / 1 fork，推送 2026-05-21 | API 未标协议 | 个人配置仓库里的一个 skill。未通读全文，只登记触发面 |
| sigma | [sanyuan0704/sanyuan-skills](https://github.com/sanyuan0704/sanyuan-skills/blob/main/skills/sigma/SKILL.md) | 父集合 3959 star，推送 2026-05-11；旧地址 [sanyuan0704/sigma](https://github.com/sanyuan0704/sigma) 11 star，说明已迁走 | MIT | 通用家教。2026-10-23 已在家教调研里筛过，本文不把它当数学默认项 |

### 把画面做出来

| 名称 | 仓库 | 2026-10-24 | 协议 | 备注 |
|---|---|---|---|---|
| 《线性代数的本质》 | [3blue1brown.com/topics/linear-algebra](https://www.3blue1brown.com/topics/linear-algebra) | 引擎 [3b1b/manim](https://github.com/3b1b/manim) 95578 star | MIT（引擎） | 现成片，不是 skill。行列式那集就是平行六面体的体积 |
| Manim Community | [ManimCommunity/manim](https://github.com/ManimCommunity/manim) | 41806 star / 3222 fork，推送 2026-10-23 | MIT | 本机 2026-10-14 装到的是 Community v0.22.0。`from manim import *` |
| ManimGL | [3b1b/manim](https://github.com/3b1b/manim) | 同上 95578 | MIT | `from manimlib import *`。和 Community 不兼容。三维交互原型更接近 3b1b 原版 |
| manim-composer | [adithya-s-k/manim_skill](https://github.com/adithya-s-k/manim_skill) | 1203 star / 98 fork，推送 2026-01-23 | MIT | `skills/manim-composer/`。只写 `scenes.md`。同仓库还有 `manimce-best-practices`、`manimgl-best-practices` |
| manim-video | [browser-use/video-use](https://github.com/browser-use/video-use/tree/main/skills/manim-video) | 父仓库 29612 star，推送 2026-10-08 | MIT | 父仓库是视频编程代理。skill 在 `skills/manim-video/`。本机已渲出约 59 秒草稿 |
| mathVideoMaker | [GordenSun/mathVideoMaker](https://github.com/GordenSun/mathVideoMaker) | 294 star / 57 fork，推送 2026-06-03 | API 未标协议 | skill 目录 `skills/math-explainer/`。同时出 MP4 和可拖参数的单文件 HTML |
| manim-community-deep-skill | [Skheiller/manim-community-deep-skill](https://github.com/Skheiller/manim-community-deep-skill) | 16 star / 1 fork，推送 2026-02-09 | MIT | 口头描述 → ManimCE 场景。明确排除 ManimGL |
| manim-video-lab | [ApliroAI/manim-video-lab](https://github.com/ApliroAI/manim-video-lab) | 23 star / 2 fork，推送 2026-07-05 | MIT | Docker 里跑 Community v0.20.1，本机可以不装 LaTeX。先写导演包再写代码 |
| MathLens | [sagittaprolifer245/MathLens](https://github.com/sagittaprolifer245/MathLens) | 0 star / 1 fork，推送 2026-10-12 | API 未标协议 | 题目截图 → 分析 → HTML → 分镜 → TTS → Manim。八步流水线 |
| canvas 逐帧成片 | 无独立仓库 | 本机 2026-10-20 实测 | — | 脚本随产物删除。步骤写在 Manim 复盘末尾。1600×900、30fps、92 秒，CPU 约 4 分钟 |

## 讲懂类：各自适合哪一种卡壳

### explain-it（本机已有，第一选择）

路径：`~/Project/skills/skills/explain-it/SKILL.md`。

它不把原话再讲一遍。先问从哪一句开始不懂，再分成三种：

| 类型 | 信号 | 手段 |
|---|---|---|
| 没画面 | 「脑子里没有画面」「太抽象」 | 一个能看的例子。数学概念点名可以上 Manim；通用概念先搜现成讲解 |
| 缺前置 | 从某一术语开始断 | 往回补一层，并标出先不用管什么 |
| 连不成结构 | 每句都懂，合起来不懂 | 一张对照或一条因果，外加正例和反例 |

红线里写了：类比是桥，过了桥要给正确定义；一次只补一个缺口；结尾让用户复述。

用在行列式这种句子上时，开口要加约束，否则它会再找一个类比：

> 用 explain-it。卡在「行列式是空间体积的伸缩」，类型是没画面。请给一个 3×3 数值例子：三条棱的坐标、体积、和 det 的计算如何对上。不要新的比喻，不要出记忆卡片。

### math-intuition-builder（教法骨架，仓库本身不是数学项目）

文件：[math-intuition-builder/SKILL.md](https://github.com/parcadei/Continuous-Claude-v3/blob/main/.claude/skills/math/math-intuition-builder/SKILL.md)。

[Continuous-Claude-v3](https://github.com/parcadei/Continuous-Claude-v3) 的主体是 Claude Code 的上下文管理，3947 star 来自那里。这个 skill 很短，触发语包括 “what does this mean geometrically” 和 “concrete example of”。步骤是 Mason 的特殊化与一般化：用自己的话重述，代一个小数，先猜答案的形状，画图，降到更低维，再找已经会的相似结构。输出是一份检查点：具体数例、图、简化版本、一句直觉。

它不渲染。适合抄进学习仓库将来的 `stuck-line`，不必克隆整个 Continuous-Claude。

### abstraction-levels（只抄梯子，不装整套课）

仓库：[panpanc/math-skills](https://github.com/panpanc/math-skills)。作者在 [r/mathematics](https://www.reddit.com/r/mathematics/comments/1rto631/selfpromotion_i_opensourced_9_agent_skills_for/) 说动机是：教科书太干，随手问聊天模型的数学讲解质量不够，所以把教学形状写进 skill，产物落成 markdown，不留在对话里。

和「一句抽象话」直接有关的是 `abstraction-levels`：同一个跑通的例子，从具体层爬到抽象层，每层把同一句话重说一遍。行列式可以是：格子面积，平行四边形面积，平行六面体体积，\(n\) 维仍是这个体积，\(\det\neq0\) 即可逆。

同仓库其余技能是另一件事，默认不装：

| Skill | 产物 |
|---|---|
| `bottom-up` / `course-bottom-up` | 从微积分和线代往上搭的长文，或一整门课 |
| `deep-explain` | 每个符号、每个构造选择、每个可能的混淆点 |
| `theorem` | 把证明写成能读的文本：动机、关键一步、失败尝试 |
| `reinvent-from-scratch` | 用失败的尝试把概念重新发明一遍 |
| `dependency-map` | 先修关系图 |
| `mental-models` / `field-mental-models` | 一个领域里反复出现的推理模式 |
| `generalization-ladder` / `connect-concepts` / `term-origins` | 推广、概念关系、词源 |

调用例子（在该仓库约定下）：`/abstraction-levels determinant as signed volume`。Cursor 里把对应的 `SKILL.md` 放进 `.cursor/skills/abstraction-levels/` 后点名使用。

### explain-math-intuition（接到 NNDL / SLAM，不负责画图）

单文件：[Skill.md](https://github.com/tgkprog/AI-snippets/blob/main/Skill.md)。0 star。顺序固定成八步：这个量当初要解决什么工程问题，每个符号是系统里的什么东西，核心运算在做什么，它坐在哪条流水线里，计算机实际怎么算，什么时候会坏，出现在哪个模型里，论文里见到什么式子就该认出来。

最小二乘、条件数、高斯先验下的 MAP，适合走这八步。它给不出平行六面体。

### math-explain（只登记，未通读）

[SKILL.md](https://github.com/gyy0592/claude-config/blob/main/skills/math-explain/SKILL.md) 的描述是：对单个概念、一步或一条方程做严格讲解；用户说 “explain mathematically” 或觉得上一次讲解在公式上含糊时使用。仓库是个人 Claude 配置（7 star）。没有逐行核对它的步骤，不作为推荐项。

### sigma（通用家教，不是这一个句子的工具）

skill 正文在 [sanyuan-skills 的 sigma](https://github.com/sanyuan0704/sanyuan-skills/blob/main/skills/sigma/SKILL.md)。旧仓库 README 写明已迁走。规则是不直接给答案、每轮 1–2 个问题、大约八成掌握才进入下一个概念，并在 `sigma/{topic}/` 写学习档案和概念图。触发面是 “teach me / 我想学”。

家教调研里已经把它和 learn-coach、Bloom、agent-tutor 放在一起筛过。对着 NNDL 的某一行，它会另起一条学习路线。要家教式追问时去那篇，不在本文的默认顺序里。

## 画面类：先现成片，再 canvas，最后 Manim

### 现成片

[Essence of Linear Algebra](https://www.3blue1brown.com/topics/linear-algebra) 把矩阵看成把空间搬走，行列式是被搬走之后的有向体积。这和「先二维面积、再三维体积、更高维不再另画」是同一幅图。标准概念先看这一套，不为它重新渲染。

### 本机 canvas（矩阵 + 中文 + 真实数值）

复盘 2026-10-20 的结论：要解决的往往是「一张静态图讲成动画」，不是 LaTeX 逐项变形。做法是自包含 HTML，`canvas` 画三维，页面暴露 `window.seek(t)`，无头 Chrome 按秒数截帧，ffmpeg 合成。中文用 `Noto Sans CJK SC` 的 `fillText`，不用 LaTeX。实测 92 秒成片约 4 分钟 CPU。脚本已删，要复现时按复盘里的三步重写，大约 200 行。

行列式的三条棱、最小二乘的投影，放这里，成品放学习仓库的 `nndl/animations/`。

### Manim 技能（公式变形时再用）

Community 和 ManimGL 是两套代码。本机复盘走的是 Community。

已经在本机跑通、优先沿用的是两条：

1. [manim-composer](https://github.com/adithya-s-k/manim_skill/blob/main/skills/manim-composer/SKILL.md)：先研究 aha、再问听众和篇幅，写出分场的 Visual / Content / Narration。不写代码。
2. [manim-video](https://github.com/browser-use/video-use/blob/main/skills/manim-video/SKILL.md)：`plan.md` → 一场一个类 → `manim -ql` 草稿 → ffmpeg 拼接。模式表里有概念讲解、公式推导、三维。依赖是 Python 3.10+、Manim CE、LaTeX、ffmpeg。中文不能写进 `MathTex`。

安装与踩坑以 [`manim与画图类Skill探索复盘.md`](manim与画图类Skill探索复盘.md) 为准。当时是浅克隆到 `~/Project/skills/.tmp-skill-explore/`，没有挂进 Cursor 全局 skill。

其余三条登记备用，不替换上面两条：

| Skill | 和已跑通链路的差别 |
|---|---|
| [mathVideoMaker](https://github.com/GordenSun/mathVideoMaker) 的 `skills/math-explainer/` | 视频之外强制一个可拖参数的 HTML；用文字化的布局检查代替看图纠错。294 star，比 composer 加 video 更完整，但本机还没跑 |
| [manim-community-deep-skill](https://github.com/Skheiller/manim-community-deep-skill) | 短链路：蓝图、代码、预览、导出命令。16 star |
| [manim-video-lab](https://github.com/ApliroAI/manim-video-lab) | Docker 镜像 `manimcommunity/manim:v0.20.1`，适合不想在本机装 cairo / LaTeX 的时候。写代码前要先有导演包 |

[MathLens](https://github.com/sagittaprolifer245/MathLens) 是八道工序加上配音。0 star，2026-10-12 仍有推送。一道竞赛题要成片时再看，不用于「这句话什么意思」。

## 和 NNDL 学习循环怎么接

画面负责知道这行在说哪件事。书上的下一行仍是代数，白纸重推不交给这些技能。

一次卡壳在仓库里最多留下：笔记里的一个数值例子，或者 `nndl/animations/` 里的一页 HTML。记忆卡片仍只收定义和当引理用的结论式。`sigma`、`course-bottom-up`、MathLens 都不进入这个循环。

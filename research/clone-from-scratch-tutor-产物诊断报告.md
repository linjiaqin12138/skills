# clone-from-scratch-tutor 产物诊断报告

> 触发：2026-10-16 用户对 `skills/clone-from-scratch-tutor` 的两条产品反馈——①「语言有些死板晦涩，对我这个 ADHD 不友好」②「有些东西应该作图/做动画但是没有，现在还是倾向于文字」。
> 对象：`microduck-from-scratch`（M0–M8）与 `linux-0.11-from-scratch`（M0–M2）的全部已交付产物，以及该 skill 自身的规则文本。
> 上游背景：`research/answer-me-with-html-用法与踩坑记录.md`（「集成：clone-from-scratch-tutor」一节记录了 tutorial 产出改为 am 渲染 HTML 的过程）。
> 方法：主 agent 直测 + 3 个子 agent 分头精读（microduck 教程文风 / linux 教程与概念文档 / 流程产物合规），**所有关键断言均由主 agent 独立复核**，复核方法与结果见 §2。

---

## 1. 结论摘要

两条反馈的根因**完全不同**，不能用同一条规则去修。

| | 反馈一（语言死板） | 反馈二（该作图没作图） |
|---|---|---|
| 根因 | **根本没有规则**，且现有偏好反向拉扯 | **规则存在但措辞是许可不是义务**，且管辖范围不含教程页 |
| 阻塞点 | 全仓 `ADHD\|注意力\|多动` **零命中**，`user-prefs.md` 无任何记录 | SKILL.md 概念深读一节写的是「才自制 HTML 演示动画或分帧图」——「才」是许可 |
| 修复方向 | 新增认知负荷硬指标 + 把 STE 切 strict | 把「该作图」写成义务，并扩到教程页 |

另有一个**字节错配**现象，是反馈二的量化底账：全部 9 篇教程里 **35%–60% 的文件体积是同一张糊掉的全局架构图**（base64 内联），而真正该画的示意图一个都没有。**图预算花在了错误的图上。**

流程层面查出 3 个问题，其中 2 个经用户澄清后归为**历史遗留**（skill 中途优化过，规则当时不存在），1 个**仍会复发**。

---

## 2. 调查范围与复核

### 2.1 直测项（主 agent 亲自执行）

```bash
# 动画能力
grep -oE '@keyframes|animation:|<canvas|<svg|<img' */docs/milestones/*/*.html   # 全 9 篇

# base64 内联图占比
for f in */docs/milestones/*/*.html; do
  b=$(grep -oE 'data:image/(svg\+xml|png);base64,[A-Za-z0-9+/=]+' "$f" | awk '{s+=length($0)}END{print s+0}')
  echo "$b $(stat -c%s $f) $f"
done

# 折叠能力
grep -rc '<details' */docs/milestones/*/*.html        # 全 9 篇

# ADHD / 视觉偏好是否被记录
grep -rniE 'adhd|注意力|多动' --include='*.md' skills/ */docs/

# 概念素材是否被引用
for a in obs-layout system-order filter-state lowpass; do
  grep -rl "$a" --include='*.md' --include='*.html' . | grep -v /assets/
done

# skill 规则演进时间线
git log --format='%h %ad %s' --date=short -- skills/clone-from-scrach-tutor/
```

### 2.2 子 agent 报告的复核结果

| 子 agent 断言 | 复核方式 | 结果 |
|---|---|---|
| microduck `docs/arch/` 不存在 | `ls` | ✅ 确认，目录都没有 |
| microduck `phase-1.md` 无架构总图 | 读全文（12 行） | ✅ 确认 |
| m0–m4 无任何架构图 | `ls` 五个里程碑目录 | ✅ 确认，只有 `acceptance.md` + `tutorial.md` |
| microduck 功能清单是 M8 补做 | 读 `feature-inventory.md` 前 12 行 | ✅ 原文自承「**本文档是 M8 追认生成的全量功能清单**……本应在 Phase 1 产出」 |
| linux STATE.md 计数过期 | 读 `STATE.md:36` | ✅ 「42 项，已交付 1 / 未交付 41」，而同一行括号里写着「M1 已通」——自相矛盾 |
| 偏差条数 | `grep -c` | ✅ microduck 49 / linux 16 |
| m2 教程 Panel J 空白 | `grep 暂无` | ✅ 确认 |
| m2 只有 bite1 | `ls` | ✅ 确认，且 Panel G 已预告 bite2 |
| m1 `arch.html` 退化为静态壳 | 对比 m0/m1 的行数与交互函数数 | ✅ m0 = 245 行 / 8 处 `<svg`+`addEventListener`；m1 = 41 行 / **0 处** |
| `linear-framebuffer/index.html` 无真图 | `grep -oE '<svg[^>]{0,80}'` | ✅ 仅 2 个 `width="14" height="14"` 的图标 |
| `boot-from-power-on.md` 有自测题 | 读 L148 | ✅ 且用了 `<details>` 折叠参考答案 |

**复核期间推翻了一条我自己的中途结论**：我在口头汇报时把 `linear-framebuffer.md` 列为「✅ 有链接的正���样板」——链接确实存在，但落地页里零张真实示意图，只把 markdown 又抄了一遍。**它不是可复制的样板。**

### 2.3 未采信的

子 agent 关于「base64 图占 Panel C 体积 88–91%」的说法**未采信**，实测为**全文占比 35%–60%**（base64 字符数 / 文件字节数）。Panel C 的占比确实高，但把两张图算成单面板的 91% 是把其它面板也算进去了。以实测值为准。

---

## 3. 反馈一：语言死板，对 ADHD 不友好

### 3.1 根因：偏好从未被记录，且现有默认反向

`skills/clone-from-scrach-tutor/user-prefs.md` 现有三节：技术背景、学习目标、构建环境。**没有 ADHD。** 全仓 grep 零命中。

而「学习目标」一节写的是：

> 不是背套路，要能「知其所以然」——目标是将来能在这个领域创新、质疑既有设计。因此：① 每个「为什么」必须讲到失效边界；② 多做「破坏实验」；③ Phase 4 的权衡复盘和检验题是重点，不能水。

这是一条**为深度专注优化**的目标。ADHD 读者需要的是相反的次序——先抓结论、先看图、再按需深入。两条都不算错，但**默认权重应该反过来**，而且当前没有任何铁律在管这件事。

### 3.2 硬指标：篇幅与面板数

| 教程 | 面板数 | 中文字数 | 单面板最大中文字数 |
|---|---:|---:|---:|
| m8-bite4-safe-to-restart | 6 | 3,058 | 1,037 |
| m8-bite5-bam-diag | 3 | 3,532 | 1,606 |
| m8-bite6-bam | 7 | 5,028 | **2,834** |
| m8-bite8-walk-verdict | 8 | 4,051 | 1,082 |
| m8-bite9-phase4-fixes | 9 | 3,852 | 1,110 |
| m8-bite10-do-verdict | 5 | 2,860 | 885 |
| m1-bite1-framebuffer | 8 | 2,562 | 776 |
| m1-bite2-printk-scroll | **10** | 3,972 | 1,172 |
| m2-bite1-gdt-idt | **10** | 3,625 | 1,212 |

`am` 自身的组件选择规则写着 **「With more than 8 panels, split the page or cut panels」**（`vendor/answer-me-with-html/SKILL.md` §4）——两篇已违反自己依赖的工具的规则。

**折叠能力：全 9 篇 `<details>` / `<summary>` 计数均为 0。** 所有面板默认摊开，读者无法「收起已读部分」。

### 3.3 工具里已有检查器，没开

`am render` 内置 **STE 受控写作检查**，默认 `style: 80`（只警告）：

| 检查项 | 中文阈值 |
|---|---|
| 步骤句长 | ≤35 字 |
| 说明句长 | ≤45 字 |
| 每段句数 | ≤6 句 |
| 轻动词 | `进行优化`→`优化`、`加以说明`→`说明` |
| 连续「的」 | ≤3 个 |
| 陈词滥调 | `赋能` / `闭环` / `至关重要` |
| 模糊量 | `尽快` / `若干` / `大概` / `多次` |
| 一词多写 | `单击`→`点击`、`入参`→`参数` |
| 语态 | 主动语态；步骤用祈使句 |

教程页面里确实有 `am-ste` 标记（抽查 m2-bite1：命中 10 次），**说明检查在跑，警告没人管**。切到 `style: strict` 是一个参数的事。

### 3.4 具体的文风症状（microduck m8 实测）

- **预告式开头每篇必中**。如 bite-5 开篇：「本 Bite 只做一件事：用可切换的基线增益跑 A/B 对照……**不移植 BAM，不改 Rust**」。ADHD 读者还没看到问题就先被打预防针。
- **「我们没要到这份…」自我对话式否定句**。bite-5 决策卡片的「放弃的成本」栏连续 5 句以此开头；bite-8、bite-10 同。读者要的是「选了什么、为什么」，不是内部会议记录。
- **长句**。bite-9 实测一段 220 字符，把 7 个社区模型名串在一个括号里：`apirrone、witcheer、DenRakEiw、RemiFabre、jmvargascruz、Datawhale、craigm26`——读者读完名字才找到结论「也全是 feedforward」。

### 3.5 概念文档缺自测题

| 概念文档 | 自测题 |
|---|---|
| `boot-from-power-on.md` | ✅ L148，且 `<details>` 折叠参考答案（全仓 .md 里唯一一处折叠） |
| `linear-framebuffer.md` | ❌ |
| `gdt-long-mode.md` | ❌ |
| `idt-exception-frame.md` | ❌ |
| `elf-relro-got.md` | ❌（9 行水稿） |

SKILL.md 的概念深读结构明确要求「一道自测题」，**4/5 没有**。ADHD 读者读完既没有折叠细节、也没有「我懂了吗」的钩子。

---

## 4. 反馈二：该作图没作图

### 4.1 量化底账：图预算花在了错误的图上

| 教程 | base64 内联图（字符） | 张数 | 占文件 |
|---|---:|---:|---:|
| m1-bite1-framebuffer | 166,702 | 3 | **51%** |
| m1-bite2-printk-scroll | 91,252 | 2 | 35% |
| m2-bite1-gdt-idt | 199,300 | 2 | **53%** |
| m8-bite4-safe-to-restart | 215,872 | 2 | 56% |
| m8-bite5-bam-diag | 250,748 | 2 | 48% |
| m8-bite6-bam | 250,748 | 2 | **60%** |
| m8-bite8-walk-verdict | 134,864 | 2 | 46% |
| m8-bite9-phase4-fixes | 126,866 | 1 | 43% |
| m8-bite10-do-verdict | 126,870 | 1 | **44%** |

**全 9 篇 `@keyframes` = 0、`animation:` = 0、`<canvas>` = 0。** 唯一的 `<img>` 就是上面那张架构图。

而 m8 六篇用的是**同一对** `arch.svg` / `arch-diff.svg`。教程自己都承认图糊了：

> 本图是 4614×5581 的大图，在页面栏宽里缩到约 7%、文字会糊成灰条；点链接用浏览器全尺寸打开。（`tutorial-bite-4-safe-to-restart.html:800-801`）

**每篇 30%–60% 的篇幅，是同一张没人看得清的全局架构图。**

### 4.2 根因：规则的措辞是许可，且管辖范围不含教程页

SKILL.md「概念深读文档」一节原文：

> **先搜现成素材，再自制**：通用概念先上网找现成讲解（B 站、YouTube、官方文档、科普动画），把链接放进文档；只有项目专属的代码逻辑（例如「插值零点为什么从第一次 read 成功起算」）网上没有现成讲解，**才**自制 HTML 演示动画或分帧图。

两个问题：

1. **「才」是许可不是义务**——不是「位域/内存布局/时序这类题材必须出图」，而是「实在找不到现成的才允许自己做」。
2. **管辖范围写死是 `docs/concepts/*.md`**，教程 HTML 页面完全不在射程内。用户的 GDT 抱怨发生在教程页。

后果是 **11 个高价值题材**全部落在规则射程外：descriptor 各位、GDT 表项、选择子 index/TI/RPL、IDT 门描述符 16 字节、栈帧 64 字节布局、显存 pitch vs width×bpp、8N1 波形、页表权限位、boot 七棒接力时序、滚屏 `ptr::copy` 重叠搬运、字形 lookup 到写像素。

### 4.3 你手动验证过这个模式，但没变成规则

commit `0022864`（2026-10-07）：

```
m0-boot: 补充串口/ELF 问答进细节展开，新增 16550 位图与 hello 波形动画，素材归入 assets/{src,dst}
  + serial-16550.svg（寄存器位图总览，fireworks 风格）
  + serial-hello-wave.gif（hello 逐比特 8N1 波形动画，matplotlib 生成）
```

**「16550 位图」与「GDT 的位含义」是同一类东西。** 你 10-07 已手动要求过一次、效果也对。但那次 commit 只改了**文件摆放约定**（`assets/{src,dst}`），没有把「该作位图/动画」写成义务。

### 4.4 素材做了，但没接到读者面前

| 素材目录 | 大小 | 被谁引用 |
|---|---:|---|
| `microduck/.../obs-layout/` | 365KB（d2 + svg + png） | ❌ **全仓零引用** |
| `microduck/.../system-order/` | 125KB | ❌ **全仓零引用** |
| `microduck/.../filter-state/` | 122KB | ⚠️ 仅 `questions.md` |
| `microduck/.../lowpass/` | 5KB | ⚠️ 仅 `questions.md` |
| `microduck/.../servo-gain-torque/` | 13KB | ✅ m8-bite-5 / bite-6 链接过去 |
| `linux/.../linear-framebuffer/` | 104KB | ⚠️ 链接存在，但落地页**零真图**（只有 2 个 14×14 图标） |
| `linux/.../elf-relro-got/` | 130KB | ✅ 有图 |

**microduck 做了 9 个素材目录，4 个从未被任何教程或概念文档引用。** linux 做了 2 个，1 个是空壳。

### 4.5 概念文档逐篇：该不该有图

| 文档 | 题材性质 | 现状 |
|---|---|---|
| `gdt-long-mode.md` | **位域**（64 位 descriptor、16 位选择子） | ❌ grep `assets\|.svg\|.png\|.gif\|<img\|![\|图` **零命中** |
| `idt-exception-frame.md` | **字节偏移**（栈帧 64 字节、门描述符 16 字节） | ❌ 仅 L78-83 一段 4 行 ASCII，未标偏移、未对比「含/不含 error code」 |
| `boot-from-power-on.md` | **时序**（七棒接力单向时间链） | ❌ 全文字；El Torito 三模式是 1/2/3 列表，本该 3 列并排对比 |
| `linear-framebuffer.md` | **内存布局**（pitch ≠ width×bpp/8） | ❌ 给了 24 行 Node 代码 + `console.log`，本该二维显存网格图 |

### 4.6 教程页里该画没画的（部分）

- **m8-bite5**：BAM 电压→电流→力矩链，19 个 MathJax `<math>` 块静态堆着，本该是逐步高亮动画；PD 弹簧 vs 电机是并排对比图的最佳题材，只有两条并排公式。
- **m8-bite4**：apply 流水线预检位置，只有一行 ASCII `swap 前写盘]watch 50Hz × 4s]A[VerifyingBC[arm pending...`。
- **m1-bite2**：滚屏 `ptr::copy` 重叠搬运，「朴素正序循环逐字节拷会读到刚写过的值」——3 秒动画能讲清，现在是一句话。
- **m2-bite1**：GDT Accessed 位引爆三连重启（只读页 → CPU 隐式回写 → #PF → 升级 #DF → triple fault），因果链零张图。

---

## 5. 流程层面：会复发 vs 历史遗留

用户澄清「skill 中途优化过，早期没有这些规则」，据此重新归类。

### 5.1 规则演进时间线（决定性证据）

| 日期 | commit | SKILL.md 行数 | 架构图/功能清单规则段数 |
|---|---|---:|---:|
| 09-19 | `999ab50` | 88 | 0 |
| 09-26 | `aef6aa4` | 154 | 0 |
| 09-28 | `f0a3659` | 173 | 3 |
| **10-03** | `a9edbfb`「流程修订与教学偏好沉淀」 | **191** | **12** |
| 10-05 | `eb347a4`「规则上收 SKILL」 | 200 | 12 |
| 10-11 | `d1cab8d` | 189 | 10 |

microduck m0–m4 交付于 **09-22 ~ 09-28**，全部早于 10-03 的大改。

### 5.2 归类

| 问题 | 归类 | 理由 |
|---|---|---|
| microduck m0–m4 无架构图 | **历史遗留** | 交付时规则尚不存在；m6/m7/m8 已合规 |
| microduck 功能清单晚至 M8 | **历史遗留** | 同上。清单本身自承此事 |
| linux 功能清单用 P0/P1/STUB/FORK 自定义分级（非规定三列表格） | **⚠️ 仍会复发** | 当前 SKILL.md 明确要求三列；linux 起于 10-03 前后仍偏离 |
| linux 清单未逐条抄（74 项 syscall 只按子系统给标签） | **⚠️ 仍会复发** | 导致无法回答铁律 8 的核心问题 |
| linux STATE.md 未交付计数过期 | **⚠️ 仍会复发** | L36「已交付 1 / 未交付 41」与同行括注「M1 已通」自相矛盾 |
| linux `m2` 只交付 bite1，Panel G 已预告 bite2 但文件不存在 | **⚠️ 仍会复发** | 教程页自带预告却无对应产物 |
| 架构图交互性三档不齐（m0 活 SVG / m1 退化为静态壳 / m2 无 arch.html） | **⚠️ 仍会复发** | 无一致性要求 |

关于 linux 清单里的 **STUB** 类（8 个 syscall 标注「原版就是 `return -ENOSYS`，我们对齐行为 = 同样返回 ENOSYS」）：这个论证**在技术上站得住**——对齐行为确实属于复刻而非豁免。但铁律 8 要求豁免必须经用户批准，这 8 项未经逐条裁决就定了性，**属于流程口子而非技术错误**。

---

## 6. 工具能力盘点：存在但从未接通

| 能力 | 位置 | 使用情况 |
|---|---|---|
| **`am video`**（3b1b 风格讲解视频） | `vendor/answer-me-with-html/references/video.md` | **0 次**。一个 `##` 一幕、`>` 旁白一行一拍、旁白写 `[名字]` 镜头自动推近高亮、同名节点跨幕平滑移动、TTS 配音、可导 MP4/WebM |
| **STE strict** | `am render --style strict` | **未启用**。检查在跑（页面有 `am-ste`），警告全被忽略 |
| 自绘 SVG + SMIL 动画 | `research/answer-me-with-html-用法与踩坑记录.md` §自绘 SVG 动画配方 | 仅 m0 用过一次（`serial-hello-wave.gif` 走 matplotlib 而非 SMIL） |
| 页面 Reply / Remarks 机制 | `am` 页面自带 | 页面有 `am-reply`（14 处），但提问记录仍走文档流程，未利用页内回传 |

`am video` 的闸门写死在它自己的 SKILL.md §7：

> Use only when the user explicitly asks for a video……Do not produce a video unasked in always-on mode either.

tutorial 流水线里用户不会主动喊「做个视频」，所以**这道闸永远是关的**。能力在，管道没接上。

---

## 7. 问题清单（按优先级）

| # | 问题 | 性质 | 证据 |
|---|---|---|---|
| 1 | ADHD 偏好零记录，现有「深度专注」目标反向拉扯 | 缺规则 | §3.1 |
| 2 | 「该作图」措辞是许可、范围不含教程页 | 规则太软 | §4.2 |
| 3 | 11 个位域/布局/时序题材零图 | 规则太软的后果 | §4.2 §4.5 |
| 4 | 素材做了 4/9 成孤儿 | 无验收挂钩 | §4.4 |
| 5 | 图预算 35–60% 花在糊掉的同一张全局图上 | 产出取向 | §4.1 |
| 6 | STE 检查在跑但未切 strict | 未启用现成工具 | §3.3 |
| 7 | `am video` 能力 0 使用 | 未启用现成工具 | §6 |
| 8 | 全 9 篇 0 折叠、2 篇超 8 面板 | 缺结构约束 | §3.2 |
| 9 | 概念文档 4/5 无自测题 | 规则未执行 | §3.5 |
| 10 | linux 清单非三列表格、未逐条抄、STUB 未经裁决 | 格式偏离 | §5.2 |
| 11 | linux STATE.md 计数自相矛盾 | 数据陈旧 | §5.2 |
| 12 | m2 预告 bite2 但产物缺失 | 闭环缺失 | §5.2 |
| 13 | 架构图交互性三档不齐 | 一致性 | §5.2 |
| 14 | m2 教程 Panel J 整面板空白 | 产出质量 | §2.2 |

---

## 8. 待裁决

本报告只做调查，**未修改任何 skill 文件**。以下三项需要你拍板后才动手：

1. **ADHD 相关的硬指标写多严**——是给 STE 加严（切 strict），还是另立「单面板 ≤N 字 / 超过 M 字必须配图」这类新指标？
2. **「该作图」清单是穷举还是分类**——穷举题材表（GDT 位域、栈帧偏移、显存布局…），还是按「位域 / 内存布局 / 时序 / 状态机 / 空间关系」五类通则？
3. **存量产物回填到什么程度**——m2–m8 已有 9 篇教程要不要补图补折叠，还是只保证新产出符合新规则？

---

## 附：本次调查未做的事

- 未修改 `skills/clone-from-scratch-tutor/` 下任何文件
- 未修改任何 `docs/` 产物
- 未回填任何缺失的架构图或自测题
- 子 agent 报告中「Panel C 占 88–91%」的说法已实测修正为全文 35–60%，见 §2.3

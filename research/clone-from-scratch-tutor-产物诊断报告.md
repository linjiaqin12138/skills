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

1. **ADHD 相关的硬指标写多严**——是给 STE 加严（切 strict），还是另立「单面板 ≤N 字 / 超过 M 字必须配图」这类新指标？**（已裁，2026-10-18：STE 切 strict + 文风红线 + 自测题硬指标，落地见 §9.14）**
2. **「该作图」清单是穷举还是分类**——穷举题材表（GDT 位域、栈帧偏移、显存布局…），还是按「位域 / 内存布局 / 时序 / 状态机 / 空间关系」五类通则？**（已裁，2026-10-16：两个选项都没选，用户给了一条判定规则——「如果文字是为了描述图，就应该画成图」，见 §9.1）**
3. **存量产物回填到什么程度**——m2–m8 已有 9 篇教程要不要补图补折叠，还是只保证新产出符合新规则？**（部分已裁，2026-10-16：概念文档这一类必须回填 am 页面；教程页补图补折叠仍待裁，见 §9.1）**

---

## 9. 讨论与裁决记录（2026-10-16 下午，报告发布后的续谈）

本节起作为 skill 优化讨论的流水账：每轮讨论的裁决、报告框架修正、新议题都记在这里。

### 9.1 用户裁决（第一轮）

1. **概念解释的独立文档必须用 answer-me-with-html 出 HTML 页面**（「必须」，含存量回填）。这是对 §3.5 的升级——报告原只查了概念文档缺自测题，「概念文档格式页面化」整条是新规则。
2. **架构图分层**：顶层图只画进程/模块间的关系与通信（miniduckd / mini-duckctl / mini-updaterd / 仿真体），每个进程的内部子模块单独画图；方块里只写名字和极简标签，禁止堆大段文字。触发点：microduck m8 的 arch.svg 全铺开到 4614×5581，用户「抓不到重点，而且看不清」。
3. **画图原则（用户原话）**：「画图尽可能画，产出的文字描述尽可能简洁不要太多废话。如果文字是为了描述图，就应该画成图。」——§8 Q2 以此裁决，比穷举清单和五类通则都更强：是一条**判定规则**（看文字是不是在描述图），并确认 §4.2 的「才」字措辞必须改成义务。
4. **ADHD 偏好记录到两个项目的 user-prefs**（microduck / linux 各一份，用户级文件同步）。

### 9.2 报告框架修正（第一轮讨论暴露）

- **arch 大图是两个问题，报告只写了一个**。原问题 #5 是「体积/内嵌」问题（35–60% 字节、糊成灰条）；用户指出的是**图的信息设计**问题——不分层、方块啰嗦，即使全尺寸清晰也抓不住重点。两个问题修法不同（换小图 vs 分层拆分）。问题清单增补：
  - **#15 架构图无分层设计**：全铺开单图，方块内堆文字（证据：m8 arch.d2 383 行、每框 5–15 行文字）。
  - **#16 概念文档无页面化要求**：SKILL.md 把概念深读写死为 `docs/concepts/*.md`，不在 am 渲染射程内。
- §4.1/§4.2 的量化与根因分析不变，仍然成立；#15/#16 是补充不是推翻。

### 9.3 新议题：用户级 user-prefs 迁出 skill 目录（用户提议，讨论中）

**提议**：`skills/clone-from-scrach-tutor/user-prefs.md` 不适合放在 skill 目录里，应迁到 `~/.clone-from-scratch-tutor/user-prefs.md` 之类位置，在用户**第一次使用 skill 时**初始化。

**赞同理由**：

- skill 目录是 git 仓且带 `.claude-plugin/` 分发结构，个人数据（技术背景、ADHD、构建环境）混在里面，发布/分享即泄露；迁出后规则（可分发）与个人数据（不落仓）分离。
- 与工具链现有模式一致：am 已经把配置和页面放 `~/.answer-me-with-html/`（`AM_HOME` 可改）。
- skill 若走插件安装/升级，目录内文件会被覆盖或视为只读，个人数据放家里目录才能活过升级。
- **隐私窗口**：ADHD 内容是 2026-10-16 才加进工作区的、尚未 commit；在提交前完成迁移，这类信息就不进 git 历史。

**设计草案（待裁）**：

- 路径：`~/.clone-from-scratch-tutor/user-prefs.md`（与 am 同风格；skill 目录现名有拼写缺漏 `scrach`，新路径用正确拼写）。
- skill 目录里保留 `user-prefs.template.md`（空模板 + 填写引导）；首次使用检测不到家目录文件 → 从模板创建 → 向用户提几个关键问题（技术背景 / 学习目标 / 环境 / 认知偏好）→ 之后对话中持续沉淀（skill 本就有偏好沉淀实践）。
- 项目级 user-prefs 留在各项目 `docs/`（那是项目状态的一部分），但通用内容（ADHD、画图原则）**不重复**，只留指针指向用户级文件——9.4 已写入两个项目文件的 ADHD/画图两节在迁移时应收敛为指针。
- 引用更新点：SKILL.md、coder-subagent.md、writer-subagent.md 中对 `user-prefs.md` 的引用全部改指向新路径。

### 9.4 已发生的改动（第一轮讨论期间，主 agent 在用户明确「只讨论」之前已动手，用户知情后保留、不继续不回滚）

- 改：`skills/clone-from-scrach-tutor/user-prefs.md` 加「认知负荷（ADHD）」「画图原则」两节；`microduck-from-scratch/docs/user-prefs.md`、`linux-0.11-from-scratch/docs/user-prefs.md` 各加「认知负荷与画图」一节（按 §9.3 方案这两节将来应收敛为指针）。
- 改：microduck m8 `arch.d2` 重写为顶层进程通信图（4573×5869 → 2394×681），`arch-diff.d2` 重写为顶层 diff 视图；新增 `arch-miniduckd.d2`（miniduckd 内部 14 节点）、`arch-updaterd.d2`（updaterd 内部 + 更新资产）；四张 SVG 已重新渲染。**未做视觉目检**（agent-browser 依赖未装齐）；6 篇 m8 教程 HTML 仍内嵌旧的 base64 大图，未替换。
- 未碰：skill 规则文件（SKILL.md / 两个 subagent 文件）、教程 HTML、概念文档。

### 9.5 新议题：什么时候出视频，什么时候出图（2026-10-16 用户提问，讨论中）

背景：§6 盘点过 `am video` 能力 0 使用，闸门是「用户明确要求才出」，tutorial 流水线里这道闸永远关着。本议题要给流水线一个自主选择规则。核心观点：**这不是二选一，是三档**。

**三档媒介，按「内容里有没有时间维度」分：**

| 档位 | 回答的问题 | 适用题材 | 成本/维护 |
|---|---|---|---|
| 静态图（d2 / am flow） | 它长什么样？谁跟谁连着？ | 结构、位域、内存布局、层级、并排对比、查表类参考 | 最低；源码 100ms 重渲染，代码变了图跟着变 |
| 循环动画（SMIL / GIF 内嵌页面） | 它是怎么动的？（一个循环能演完） | 波形逐比特、滚屏搬运、symlink 原子交换、滤波收敛 | 中；打开页面即循环播放，无需播放器，QA 只需截两帧对比 |
| 讲解视频（am video） | 它是怎么**一步步**变成这样的？（多拍、有因果链、需要引导注意力） | boot 七棒接力、triple fault 连锁、apply 七阶段、BAM 电压→电流→力矩链 | 最高；脚本+旁白+渲染分钟级，代码变了要重录；QA 坑多（§踩坑记录坑 2：data-step 对齐、SVG 尺寸塌缩） |

**判定规则（草案）**：

1. 内容是**结构/对照/查表** → 静态图。读者需要盯着看、来回比、当参考查——视频强迫读者拖进度条，是倒退。
2. 内容是**过程/时序**，且 3–10 秒一个循环能演完 → 循环动画。不需要解说，循环本身就是重复曝光。
3. 内容是**因果链/多拍过程**，且需要控制节奏、逐个聚光（读者自己看静态图不知道先看哪）→ 视频。ADHD 视角下这是视频的核心价值：**替读者控制注意力**，一次只看一个东西，不让读者在工作记忆里同时扛多个步骤。
4. 一句话版：**图回答「什么样/谁连谁」，动画回答「怎么动」，视频回答「怎么一步步变成这样」。** 如果一句话讲不清「第一步之后发生了什么」，就是视频题材。
5. 默认偏向静态：视频不可 grep、不可 skim、维护贵，升级要有明确理由（题材命中第 3 条）。

**闸门怎么开（草案）**：不动 am 自身「用户明确要求」的闸；把裁决点移到 **Phase 规划阶段**——教程/概念文档规划时列出「本篇含视频：题材 X」，用户在批计划时一次性批准，而不是产到一半临时问。这样闸门有开的机会，又不打断流水线。

**用 §4.2 的 11 个题材验证草案**：descriptor 位域/选择子/门描述符/栈帧布局/显存 pitch/页表权限位 → 静态图（6 个）；8N1 波形/滚屏重叠搬运/字形 lookup 写像素 → 循环动画（3 个）；boot 七棒接力/GDT Accessed 三连重启 → 视频（2 个）。分布合理，没有全倒向视频。

**待裁**：① 三档判定规则是否成立；② 闸门开在 Phase 规划阶段是否可接受；③ 循环动画与视频的边界（「一个循环能演完」）是否够清晰。

### 9.6 议题续：视频工具选型 + 生图 API 的适用边界（2026-10-16 用户反馈，讨论中）

**用户反馈两条**：

1. 之前用 `am video` 出的「视频」是 mermaid 流程图的逐步出现动画——**毫无意义，不如直接画一张 mermaid**。且无中文配音。用户要的「生动讲解视频」是 `research/manim与画图类Skill探索复盘.md` 里 manim-video 做出的那种（59s 惯量/积分片，复盘结论「成片有 aha」）。
2. 有些题材代码画图先天不足，应该用**第三方生图 API**：例「果蝇复眼结构」——代码只能画抽象线条，生图能出半真实、多角度、期刊插图风格。

**对 am video 失败的诊断**：am video 的本质是「组件回放」——flow/er/tree 这些组件（mermaid 同级别）逐步出现。源视觉是流程图，视频就只是流程图分步显示，**相对静态图零信息增量**，违反 §9.5 第 3 条（视频必须提供静态图给不了的东西）。am video 要出好片，视觉必须**为讲解专门构造**（自绘 SVG + data-step/data-key，见踩坑记录），但做到那个程度，投入已接近 manim，天花板却更低。**结论（草案）：流水线里「讲解视频」档的工具从 am video 换成 manim 链路**（manim-composer 分镜 → manim-video 渲染，复盘已验证）；am video 降级为不用，除非素材恰好已是自绘 SVG。

**中文配音**：不是能力问题是环境问题。am video 的 TTS 链是 ElevenLabs key → macOS say / espeak-ng → 都没有则纯字幕；本机 Linux 三者全空（踩坑记录坑 4）。解法：① `edge-tts`（微软免费 TTS，无需 key，中文音色 zh-CN-XiaoxiaoNeural 等质量好）；② ElevenLabs key。manim 链路里配音本来就是独立工序：TTS 出音轨 → ffmpeg 混音。**「讲解视频必须配中文人声」应写成硬指标**（纯字幕片对 ADHD 读者不成立——又要看画面又要读字幕，注意力撕扯）。

**生图 API：赞同，但要画边界**。这给 §9.5 的三档（时间维度）加了第二个轴：**内容抽象 vs 具象**。

| | 抽象/逻辑内容（协议、位域、架构、状态机） | 具象/外观内容（生物结构、硬件外观、外壳形态） |
|---|---|---|
| 首选 | **代码画**（d2/SVG/manim/fireworks） | **真实照片/官方图 > 生图 API** |
| 理由 | 精确是刚需：标签、位偏移、箭头语义必须正确；可编辑可回归 | 真实感是刚需：代码画不出纹理和形态 |

生图 API 的三个坑（决定它不能碰抽象内容）：

1. **文字标签必糊**——生图模型渲染文字不可靠，给它画位域图会得到「看起来对但标签全错」的图，教学场景里**漂亮的错图比抽象的丑图更有害**。
2. **准确性不可验证**——果蝇复眼的小眼数目、结构层次可能是「像那么回事但错了」，必须对照权威来源核验才能用。
3. **不可编辑、不可回归**——改一个标签=重新生成=整张图变样；代码图改一个字重新渲染 100ms。

**混合模式（期刊插图的实际做法）**：生图/照片做底，**标注层用代码画**（SVG 引线+标签叠在上面）——真实感来自底图，准确性来自标注层。两个 from-scratch 项目的题材几乎全是抽象的，生图适用范围很小：硬件外观（XL330、Radxa 板）、M11 外壳 3D 建模概念图；且这些优先用**真实素材**（开源硬件仓有 BOM/装配图/实物照），生图只在「没有现成真实素材且需要真实感」时上。

**修订后的选用矩阵（草案，合并 §9.5）**：

| 题材 | 工具 |
|---|---|
| 抽象 + 静态结构/对照 | d2 / am flow / fireworks（本仓 draw-diagram 已有路由） |
| 抽象 + 单循环过程 | SMIL/GIF 内嵌页面 |
| 抽象 + 多拍因果链讲解 | **manim 链路**（composer → video）+ 中文 TTS（edge-tts） |
| 具象 + 静态 | 真实照片/官方图 → 不够才生图 API，标注层代码画 |
| 具象 + 动态 | 暂不覆盖（视频生成 API 成本/可控性未到可用线，留观察） |

**待裁**：① 视频档换 manim 链路、am video 弃用是否同意；② 中文人声配音写为硬指标（edge-tts 方案）是否同意；③ 生图 API 的边界（不碰抽象内容、标注层代码画、真实素材优先）是否同意；④ 生图用哪家 API（成本/质量权衡）留给真正遇到题材时再裁。

### 9.7 议题续：manim vs am video 再评估 + 多 provider 抽象（2026-10-16 用户提问，讨论中）

**用户三点输入**：① manim 真的比 am video 好吗？② am video 能出 HTML 播放器页面而非 mp4——mp4 文件大、不方便放 git；③ TTS 和生图都要支持多 provider，写抽象接口、实现可替换；工具能力不够就直接改它代码。

**manim vs am video 的诚实对比（修正 §9.6「换成 manim」的草案）**：

| | am video | manim 链路 |
|---|---|---|
| 动画引擎上限 | 低：分步出现 + 按名聚光 + data-key 跨幕形变 | 高：任意形变、镜头运动、公式变换（3b1b 本家） |
| 交付形态 | **HTML 播放器页**：文本+SVG、小、可放 git、可交互（暂停/步进）、内嵌 #am-source | mp4 二进制：大、diff 无意义、git 膨胀 |
| 流水线契合 | 与教程页同一工具链、同一种扩展 md 草稿 | 独立 python 技能 + apt 级重依赖（已装过一次） |
| 中文配音 | 需扩展（见下） | 同样要自己做 TTS 音轨 + ffmpeg 混音，谁也不比谁现成 |

用户此前「am video 毫无意义」的体验**不是引擎上限问题，是内容层问题**——草稿用了组件回放（flow 分步出现）而不是为讲解构造的场景。用自绘 SVG 场景（data-step/data-key），am video 能做到分幕揭示/聚光/形变，对 OS 因果链题材（栈帧里指针移动、GDT 位逐个填）够用；对数学直觉题材（BAM 积分、公式形变）manim 确实不可替代。

**修订草案：am video 打底（改代码补强），manim 只用于高天花板题材。** mp4 的 git 问题标准解法：**脚本（.py/分镜 md）入库，产物 mp4 gitignore，需要时重渲染**——但这对「点开文档就能看」的阅读体验是减分，所以默认档更倾向 HTML 播放器。

**TTS 多 provider：am 里已有抽象，比预想的好**。读 am.mjs 源码确认：`voice: local` + `AM_TTS_URL` 环境变量 = 任何 **OpenAI 兼容 `/v1/audio/speech` 端点**都可接入（另有 `AM_TTS_EXTRA` 传额外参数）；内置链 auto → elevenlabs → system → 纯字幕。两条集成路径：

1. ** shim 法（零 vendor 漂移）**：跑一个几十行的本地适配服务，把 edge-tts / 其它 provider 包装成 OpenAI 兼容端点，am 配 `AM_TTS_URL` 即可。
2. **补丁法（用户已授权改代码）**：直接在 vendor am.mjs 的 `src/video/tts.js` 段加 provider 分支（如 spawn `edge-tts` CLI）。vendor 是上游 clone，补丁必须可重放——三选一纪律：fork 仓维护、仓内存 `.patch` 文件 + 应用脚本、或提上游 PR。**禁止直接手改不留痕**。

provider 顺序、音色、key 都是**个人数据**——配合 §9.3，放 `~/.clone-from-scratch-tutor/` 配置，不进仓。

**生图多 provider：仓里没有现成抽象，需要新写**。草案：一个小 CLI（如 `bin/gen-image`），接口就是「prompt + size/style → 图片文件」；每个 provider 一个小适配模块（env key 名、端点、请求映射）；配置里写 fallback 顺序；遵守复盘流程纪律——**没 key 就停并写明 BLOCKED，不降级硬凑**。TTS 与生图共用同一套「provider 适配 + 用户级配置 + 无 key 即停」模式，manim 配音也复用同一个 TTS 抽象。

**待裁**：① 修订草案（am video 打底 + 改代码，manim 高天花板）是否同意——它推翻了 §9.6 待裁①；② TTS 集成选 shim 法还是补丁法；③ vendor 补丁纪律选哪种（fork / .patch 文件 / 上游 PR）；④ 生图抽象 CLI 放哪（`bin/` vs skill 目录内 scripts/）。

### 9.8 议题续：am 页面能否内嵌 mp4（2026-10-16 用户提问，已查证）

**原生支持：没有。** image 组件白名单只有 PNG/JPG/GIF/WebP/AVIF/SVG（≤5MB），普通页面没有 video 组件；`am video` 的 mp4/webm 是「播放器页面旁边另存一个文件」，不是嵌进普通讲解页。

**但有逃生舱**：am.mjs 里 `RAW_LANGS = {"html","svg"}`—— ```` ```html ```` 块原样嵌入，不消毒。所以 `<video controls src="...">` 可以直接进面板。两种引用方式：

| 方式 | 优点 | 代价 |
|---|---|---|
| 相对路径引用同级 mp4（`<video src="bam.mp4">`） | HTML 保持轻；file:// 直接能播 | 破坏单文件性质，移动/分享 HTML 要带上视频文件；mp4 二进制进 git 的问题仍在 |
| `data:video/mp4;base64,...` 内联 | 保住单文件 | 体积 +33%；多 MB 的 data URL 在浏览器媒体栈里又重又脆，不推荐超过几 MB |

**先例**：m8 教程页已经是混合形态——图片 base64 内联 + 同级 `arch.svg` 链接。同级引用不是新鲜事。

**补丁方向（若做）**：照 image 组件的代码路径给 am 加一个 `video` 组件（小片段 base64 内联 / 大文件拷到页面同级并相对引用，渲染时打印提醒），比手写 ```` ```html ```` 块干净。

**对 git 体积问题的正面回答**：首选不是嵌 mp4，而是 **HTML 播放器页本身就是视频**（文本+SVG，git 友好）——这正是 §9.7 把 am video 列为默认档的理由；manim mp4 走「脚本入库、产物 gitignore」；确实要随文档发的短视频，控制在 720p/几秒内（可 <1MB）或退回 SMIL/GIF 循环动画档。

### 9.9 议题续：mp4 上传 S3 是否更好（2026-10-16 用户提问，讨论中）

**结论草案：S3 优于「mp4 进 git」和「base64 内联」，但对本场景（本地优先的个人学习仓）它不是默认解；正确形态是把「媒体发布」做成又一个 provider 抽象，local-dir 为默认实现，S3 为可选实现。**

**S3 路线**：优点是 git 干净、URL 稳定、页面可分享、读者无需本地重渲染。代价：① **离线即死**——本地教程依赖网络，与「本地优先学习仓」属性冲突；② 链接腐化与版本溯源——git 记页面版本，URL 上的视频可能被覆盖（缓解：content-hash 或里程碑 slug 做 key，**只增不改**）；③ 凭证管理（key 属个人数据，进 `~/.clone-from-scratch-tutor/` 配置）；④ 隐私——仓私有而 bucket 公开即泄露，签名 URL 又会过期腐化；⑤ 流水线多一道上传工序与失败面。

**local-dir 路线（默认）**：媒体放 git 外的本地目录（如 `~/.clone-from-scratch-tutor/media/` 或项目内 gitignore 的 `docs/media/`），页面相对引用或经本地 http 服务引用。零成本、离线安全；代价是不可分享、换机器要重渲染（脚本在 git 里，可重渲染）。

**provider 抽象（与 TTS/生图同模式）**：`publish(video.mp4) → URL`；适配器：`local-dir`（拷贝+相对路径）、`s3`（上传+永久 URL）、预留 `github-release`。页面里写逻辑引用，发布通道由用户级配置决定——本地阅读走 local-dir，要分享时同一份产物重发 S3 即可。纪律沿用：**没配凭证就停，不降级硬凑**。

**待裁**：① 默认 local-dir、S3 可选的抽象是否同意；② S3 key 的不可变约定（content-hash vs 里程碑 slug）；③ 是否需要「分享模式」一键把一批页面引用从 local 重发到 S3。

> **修订（2026-10-16 用户补充）**：用户经常切换设备学习，云端存储不是「分享场景可选」，是**主场景**。默认发布通道改为云端（S3/R2），local-dir 降级为离线后备。零成本替代：Cloudflare R2（免出口流量费，适合媒体）或「媒体专用 git 仓 + jsDelivr CDN」。待裁①③随之改写：① 云端默认、local 后备的抽象；③ 新设备首次拉取媒体的一致性校验（清单+hash）。

### 9.10 议题：am 适合做教程页吗 + 替代品调研 + 内置魔改方案（2026-10-16 用户提问）

**用户的质疑**：am 更适合 AI 问答（一次性答案页），不适合教程页面（结构化、长寿命、多页成系列）。**分析成立**，am 的 Q&A 基因处处可见：默认输出带时间戳进 `~/.answer-me-with-html/pages/`、`--replace` 换路径导致页面评论成孤儿（踩坑记录已踩）、自动开浏览器打断、Reply 机制为问答设计、无系列导航/目录、无进度状态、无折叠组件、无自测题组件、STE 默认只警告。教程需要的是：稳定路径、prev/next + 总目录、章节结构强约束、折叠、测验、媒体组件、严格默认。教程流水线目前靠 writer-subagent 的约定硬扛这些缺口。

**替代品调研（2026-10-16 实搜）**：

| 候选 | 是什么 | 适配度 |
|---|---|---|
| [codebase-to-course](https://github.com/zarazhangrui/codebase-to-course) | 代码仓 → 单文件 HTML 课程：滚动分模块+进度跟踪+键盘导航、代码↔大白话并排、动画可视化、**交互测验**、**术语悬浮注释** | 格式特性与诊断报告开出的药方高度重合（测验↔§3.5 自测题缺失、进度/折叠↔§3.2、术语注释↔背景卡片、动画↔画图原则）。但目标读者是「非技术 vibe coder」，深度标定不符；且是「一门课一个页面」，不是里程碑系列 |
| [tutorial-any-repo](https://github.com/TKONIY/tutorial-any-repo) | 并行 agent 逐文件写教程 → MkDocs Material 站点 → GitHub Pages | 解决「系列导航/全文检索」，但是重型站点路线（构建+托管），粒度是「全仓每个文件」，适合给 reference/ 原版代码仓做文档站（备选项），不是里程碑教程 |
| [Remotion agent skill](https://www.youtube.com/watch?v=EwKCAgt4aKI) | React 编程式视频（提示词→MP4） | 视频赛道候选，与 manim 同类，未深入 |

**没有现成 skill 直接覆盖「clone-from-scratch 里程碑教程页」**——固定章节结构、d2/SVG 嵌入、`src=` 引代码、中文 STE、单文件、可 patch，这个组合只有 am 有底子。

**结论草案：采纳用户方案——把 am 收编为 clone-from-scratch-tutor 的内置引擎并魔改**（不再是 vendor 上游，而是自有代码）。理由：改动清单已远超「补丁」量级——① 教程模式（固定章节面板、稳定文件名、prev/next+总目录页）② video 组件（§9.8）③ TTS 多 provider（§9.7）④ 折叠组件 ⑤ 测验组件（自测题硬指标）⑥ STE strict 默认 + ADHD 指标 ⑦ 云端媒体发布（§9.9 修订）⑧ 用户级配置（§9.3）。继续当 vendor 跟上游，合并成本只会越来越高；上游方向（Q&A 工具）与我们方向（教程引擎）已经分叉。技术上有利：单文件、零依赖、MIT 系项目（**fork 前需核实 LICENSE**）。魔改时可从 codebase-to-course 抄交互模式（进度跟踪、键盘导航、术语 tooltip、应用型测验）。§9.7 待裁③（补丁纪律）随之失效，改为「内置 fork，脱离上游」。

**待裁**：① 内置魔改 vs 继续 vendor+补丁；② fork 的代码放哪（`skills/clone-from-scrach-tutor/engine/`？）顺便解决目录拼写 `scrach`；③ codebase-to-course 要不要先克隆试跑一次，量化它的交互模式值不值得抄。

### 9.11 第二轮替代品检索（2026-10-16，搜索配额中途耗尽，部分结果靠直接抓取）

**新增候选**：

| 候选 | 路线 | 判断 |
|---|---|---|
| [@sshaaf/tutorial-skill](https://shaaf.dev/post/2026-05-05-lessons-from-building-my-first-claude-skill-a-code-tutorial-generator/)（npm `npx @sshaaf/tutorial-skill`） | Claude Code skill：`/tutorial analyze` 出 mermaid+md、`build` 出多章 md、HonKit 预览、GitHub Pages | 与 tutorial-any-repo 同属「文档站路线」。作者自陈一条对我们很关键的经验：**重新生成会换章节名和顺序，diff 不可用**——说明这类生成器适合「一次生成后人工精修」，我们的里程碑教程本来就是逐 bite 手工策展，路线不冲突 |
| [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills)（1000+ skill 策展清单，Apache-2.0） | 策展目录 | 核心清单里**没有**专用的「代码仓→课程页」生成器；邻居类可借鉴：artifacts-builder（React/Tailwind 复杂 artifact）、**imagen**（Gemini 生图 skill，正是 §9.7 生图 provider 的现成参考）、**Pixelbin-Media-Generation**（85+ 媒体 API 的 provider 抽象实例）、Theme Factory / Canvas Design（视觉打磨）、Skill Seekers（反向：文档站→skill） |
| SkillsLLM（skillsllm.com） | 聚合站 | 备用发现渠道，网页抓取内容稀薄 |

**结论不变**：没有现成 skill 覆盖「里程碑教程页 + 固定章节 + 代码引用 + 中文 STE + 单文件」这个组合；**内置魔改 am** 仍是推荐路线。codebase-to-course 仍是最好的交互模式参考；imagen / Pixelbin 是生图 provider 抽象的实现参考。

**配额恢复后待补检索**：① 「claude skill 教程页 / courseware / interactive lesson」多语言搜索；② skillsllm / claudeskillsmarket 上按 tutorial/course 分类；③ 中文圈（掘金/即刻/GitHub 中文 README）的手写教程页生成方案；④ NotebookLM/Gamma/Felo 等「已有文档→讲解」路线与内置魔改的对比。

### 9.12 codebase-to-course 的实现方式（2026-10-18 读 SKILL.md 源码确认）

**架构：模板 + 子代理写 HTML 片段 + build.sh 组装**，五步：

1. `references/styles.css`、`main.js`、`_footer.html`、`build.sh` **逐字复制，永不重生成**（明文规则：Never regenerate）；
2. `_base.html` 只做**三处替换**：标题、accent 色、nav dots；
3. 模块文件只写 `<section class="module">` 内容，不含 html/head/body/style/script；简单仓顺序写，复杂仓**最多 3 个子代理并行**，每个代理只拿自己的 module brief（brief 预先抽好代码片段+行号，代理不必读代码仓）；
4. `build.sh` 拼出 `index.html`（目录多文件，唯一外部依赖 Google Fonts CDN）；
5. 交互元素全部**在 `main.js` 里实现一次**，模块 HTML 通过 `data-*` 属性接线（chat 容器给 id、flow 动画挂 `data-steps='[...]'` JSON）。

**强制元素清单（写死在 SKILL.md）**：group chat 动画、消息/数据流动画、代码↔大白话对照、每模块至少一道测验、术语首次出现 tooltip——五件套「no exceptions」。这套清单恰好对症本报告 §3.5（自测题缺失、背景卡片缺失）。

**与 am 的架构差异**：

| | codebase-to-course | am |
|---|---|---|
| 版式谁算 | agent 手写 HTML 片段，CSS 设计系统定版式 | 模型写声明式 md，**CLI 算全部布局坐标** |
| 交互扩展 | 新增元素类型 = 写 HTML 模式 + 改 main.js（回到写代码） | 新增组件 = 往 CLI 渲染器加（集中写代码），模型只用 DSL |
| 并行 | brief 预抽片段，3 agent 并行 | 单调用出整页 |
| 产出 | 目录多文件 | 单文件 |

**对我们的借鉴价值（§9.10 待裁③的输入）**：值得抄的是**「固定设计系统 + 资产逐字复制」纪律**和**「强制元素清单」规则**；不必抄的是它把内容层交给模型手写 HTML（token 贵、一致性靠 brief 保证）。魔改 am 的方向因此更清晰：**保留 am 的声明式内容层，补上固定设计系统 + 强制元素清单（折叠/测验/术语卡片/图/进度）+ 教程模式（稳定路径、系列导航）**。

### 9.13 tutorial-any-repo 的实现方式（2026-10-18 读 SKILL.md 源码确认）

**架构：内容层纯 Markdown → MkDocs Material 主题渲染 → `mkdocs build` → `gh-deploy` 上 GitHub Pages。** 没有 HTML 模板概念——**主题本身就是模板**，agent 一行 HTML/CSS 都不写。配套还有：TODO.md 进度追踪表（全覆盖清单）、自审环节（完整性对账 + 每模块抽查 1-2 篇 + 修复）、增量提交、阅读路径（Quick Start 约 2 小时 / 完整 / 按主题）、主页「架构图 + 主流程图 + 代码↔流程映射表」、MathJax 公式、jieba 中文分词搜索。

**致命的不适配点：架构图和流程图要求用 ASCII art**。SKILL.md 核心原则第 8 条明文规定主页必须有 ASCII 架构总览和 ASCII 主流程图。这与用户「如果文字是为了描述图，就应该画成图」的原则**正面冲突**——位域、内存布局、显存 pitch、栈帧偏移这类题材（报告 §4.2 的 11 个）ASCII 画不出来。它的渲染层也只有 pymdownx.details（折叠 admonition），无测验、无代码对照、无进度状态，互动性接近零。

**仍有四样东西值得偷**（都能移植到我们的教程流水线，与它用什么渲染无关）：

1. **TODO.md 清单驱动的全覆盖纪律**——「每个源文件必须一条对应记录，缺一不可」正好对症报告 §5.2 的问题 #10（linux 功能清单未逐条抄，无法回答铁律 8）。
2. **自审环节**：完整性对账 + 每模块抽样质量抽查 + 修复，可作为教程交付前的 QA 关卡。
3. **阅读路径**（速通 / 完整 / 按主题）——里程碑教程成系列后同样需要入口分级。
4. **代码↔流程映射表**（流程步骤 → 源文件 → 教程页）——可直接对应我们教程「代码领读」面板。

**三个候选的分工由此清晰**（综合结论，供 §9.10 待裁①定案参考）：

| 来源 | 贡献给内置引擎的部分 |
|---|---|
| am（收编为引擎） | 声明式内容层 + CLI 排版 + `src=` 引代码 + 中文 STE |
| codebase-to-course | 固定设计系统 + 资产逐字复制 + 强制交互元素清单（测验/术语 tooltip/数据流动画） |
| tutorial-any-repo | 清单驱动全覆盖 + 自审环节 + 阅读路径 + 代码↔流程映射表（**它的渲染路线和 ASCII 图一律不采用**） |

### 9.14 规则落地记录（2026-10-18：用户 8 条优化指令已执行）

用户拍板 8 条并指示执行，已全部落进规则文件（`skills/clone-from-scrach-tutor/`）：

1. **ADHD 次序** → SKILL.md「输出风格 · 认知负荷」+ writer「行文风格 · 呈现次序」（每面板首句结论、图在文前、长细节 `<details>` 折叠或分流「细节展开」）。
2. **文风** → writer 渲染命令固定 `--style strict`（警告不清零不出页）；SKILL.md 与 writer 双处写死红线：禁预告式开头、禁自我对话式否定句、长名单进表格。
3. **自测题 3~5 道选择题** → writer 章节顺序新增「自测」面板（计入 8 面板预算、不可裁、答案折叠）；概念文档结构同要求；主 agent 抽查清单加「自测面板存在且为选择题」。
4. **概念文档只讲概念不提本项目 + 必须 am 出页** → SKILL.md「概念深读文档」整节重写：标题改 `<slug>.html`、删除「回到项目里对应哪几行」（对应关系改由 tutorial 背景卡片负责）、存量 .md 逐步迁移为页面后删除；全仓 `concepts/<slug>.md` 引用批量改 `.html`。
5. **画图义务** → 概念深读文档「该画图必须画图」（位域/内存布局/时序/状态机/空间关系）+ 架构图约定「方块只写名字和极简标签」+ writer 组件选择同步。
6. **架构分层** → 架构图约定新增「分层规则」：顶层只画进程/模块间关系与通信，内部子模块单独成图 `arch-<模块>.svg`，顶层内嵌、子图给链接；「大到要缩小才能看全 = 拆图信号」；简单架构可不分层。主 agent 抽查加「分层合规」；writer 交接清单改为「顶层 + 分层子图 + diff」。
7. **user-prefs 迁移 + 依赖装齐** → 用户级迁至 `~/.clone-from-scratch-tutor/user-prefs.md`（含 ADHD/画图原则/媒介选择全文）；skill 目录留 `user-prefs.template.md`（空模板 + 首次提问清单），原 `user-prefs.md` 删除；microduck/linux 两个项目级文件去重为**纯项目特有内容**（高发图题材等）——**连指针也不写**：skill 执行前置已保证用户级文件被读取，项目文件里再强调一遍是噪音（2026-10-18 用户纠正）；SKILL.md 执行前置/STATE.md 指针同步。依赖：宿主机用户级装 node v22.20.0 + npm 10.9.3（`~/.local/node`，不走 docker）、`vendor/agent-browser-cli` 依赖装齐、`vendor/browsers/chrome-linux64` 补齐 Chrome for Testing 158，am 0.5.0 与 agent-browser 实测可用；vendor 第三方目录约定 + 依赖自检写进 SKILL.md「环境与依赖」。
8. **图/动画/视频选择** → SKILL.md 新增「图 / 循环动画 / 视频的选择」一节（5 条规则 + 视频闸门：题材在 Phase 2/里程碑规划时列出，随计划交用户批准）。

附带收尾：用新装的 agent-browser 对 §9.4 重画的四张 m8 架构图做了视觉目检（arch / arch-miniduckd / arch-updaterd / arch-diff），全部清晰可读——§9.4 的「未做视觉目检」缺口关闭。

**遗留注意**：skill 仓 git 历史仍含旧 `user-prefs.md` 的提交（技术背景/学习目标等；ADHD 内容未曾提交、只在工作区出现过，已随迁移离开该文件）——将来若要发布该仓，需评估历史清洗。

### 9.15 多 subagent 重构（2026-10-18 用户指令，已执行）

SKILL.md 从「双 subagent」改为**多 subagent 分工**，SKILL.md 瘦身为纯编排层，工作规范全部下沉到各 subagent 文件：

- **roster 变为四个**：coder（写代码）→ **verifier（新增，复跑验收命令并出验收结论——主 agent 不再亲自复跑，只复核结论与输出原文是否吻合）** → **diagram（新增，架构图画档）** → writer（写教程）。新增 `verifier-subagent.md`、`diagram-subagent.md` 两个规范文件。
- **四个小节从 SKILL.md 移出**：「架构图约定」→ `diagram-subagent.md`（含工具分工/图风格/分层规则/里程碑两张图/目检纪律，并补了「渲染≠成图，必须亲自看图」的硬纪律）；「概念深读文档」「图/循环动画/视频的选择」「卡片模板」→ `writer-subagent.md`（卡片模板原本两边重复，现只留 writer 一份）。
- SKILL.md 侧保留的挂钩：Phase 1 总图交 diagram subagent；Phase 2 新增「视频题材闸门」（随计划交用户批准）；Phase 3 五步流水线 + 主 agent 抽查清单（架构图与代码不一致=里程碑未交付、分层合规、自测面板）。
- 用户同期手动精简了 SKILL.md（frontmatter 描述、Bite 固定结构「遇到的坑」、互动规则删「为什么」条目等），本次重构以用户精简后的版本为底。

### 9.16 二审 SKILL.md（2026-10-18 用户指出「Bite 固定结构」也是 subagent 领域，已执行）

**Bite 固定结构（6 项清单）删除**。它本身就是漂移的活证据——清单里没有「自测」「背景概念铺垫」，与 writer 的 9 面板章节顺序已经不一致。现由 writer-subagent.md 的「章节顺序」唯一定义 Bite 内容结构，主 agent 交付摘要沿用其压缩版，SKILL.md 只留一句指针。

同轮发现的同类问题与处理：

- **输出风格整块去重**：SKILL.md 里「语言三层规则 / 日常词 / 决策卡片自检」与 writer-subagent.md 重复，「代码风格 / magic number」与 coder-subagent.md 重复——全部从 SKILL.md 删掉，只留主 agent 自己行文要遵守的两条（中文直接了当 + ADHD 红线），其余各归各文件。原则同 §9.14 第 7 条用户纠正：**执行链路已有的环节不复述**。
- **Phase 3「禁止 coder/writer 一轮交付超出单 Bite」改为「禁止任何 subagent」**，覆盖新增的 verifier/diagram。

二审后 SKILL.md 剩余章节均为编排层：执行前置 / 工作区约定 / 环境与依赖 / 铁律 / 功能清单 / Phase 0–4 / 互动规则 / 输出风格（主 agent 版）。

---

## 附：本次调查未做的事

> 注：以下清单是**调查当时**（报告发布时）的快照，依然准确；调查之后发生的改动见 §9.4。

- 未修改 `skills/clone-from-scratch-tutor/` 下任何文件
- 未修改任何 `docs/` 产物
- 未回填任何缺失的架构图或自测题
- 子 agent 报告中「Panel C 占 88–91%」的说法已实测修正为全文 35–60%，见 §2.3

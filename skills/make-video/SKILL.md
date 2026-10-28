---
name: make-video
description: >
  视频制作路由器：按场景选择并调用具体视频 skill（循环教学动画、旁白讲解片、大纲成片等）。
  当用户要做视频、讲解片、Manim 动画、loop 动图嵌课件、3b1b 风格可视化，或 explain-it
  判定媒介为「循环教学动画 / 讲解视频」时使用。先选型再 Read 对应 vendor skill；没有就 prepare。
---

# make-video：视频 skill 路由器

本 skill **不自己渲视频**。职责是：认清场景 → 选对视频 skill → vendor 就绪 → 把执行交给那个 skill。

调用方（含 explain-it）应 `Read skills/make-video/SKILL.md`，不要直接点名某个 Manim 包 unless 用户已经指定。

## 1. 先确认该不该做视频

| 该做视频 | 不该做视频（退回调用方换媒介） |
|----------|--------------------------------|
| 增量来自**时间**或**第三维**（变换、轨迹、场、演化） | 纯结构 / 对比 / 定义 → 静图或文字 |
| 要嵌页面的**无声循环**教学片 | 状态逐步变、要人点「下一步」→ `courseware-builder`（HTML，非视频） |
| 要**旁白 / 长叙事**讲课 | 架构上请求在飞 → `dashmotion`（HTML/SVG） |
| 用户明确说「做个视频 / Manim / 3b1b」 | 现成 B 站/YouTube 已够用 → 给链接即可 |

拿不准时问一句：要「嵌文档里自己转的短片」，还是「带旁白的讲解视频」？

## 2. 认识的视频 skill（场景表）

路径均相对仓库根。第三方默认在 `vendor/<id>/`；没有则按 [references/catalog.md](references/catalog.md) prepare。

| id | 场景 | 产出 | 何时优先 |
|----|------|------|----------|
| **manim-teaching-figures** | 课件里「会动的 figure」：矩阵变换、优化轨迹、注意力/KV 时间演化等 | 无声 **loop mp4**，嵌 `<video autoplay loop muted>`，有接缝房规 | **短循环、嵌概念页/课件** —— 默认首选 |
| **manim-video** | 概念讲解 / 公式推导 / 算法演示 / 架构逐步出现，可加旁白 | 多场景 mp4，PLAN→CODE→RENDER 流水线 | 用户要**旁白或较长叙事片** |
| **claude2video** | 从主题或 Markdown 大纲生成多段旁白课 | 分场景 Manim + TTS 合成 | 已有**大纲/讲义**要一键成片 |
| **manim-skill** | 通用 ManimCE playbook（mobject/动画/3D/排障） | 按描述写 scene 并渲染 | 上面三个都不贴，或只要**底层 Manim 手册** |
| **recursive-math-animator** | 表→柱/折线、公式推导等固定 pattern + 旁白字幕 | 短 mp4（约 17–23s） | 数据/公式 pattern 对得上、要快出片 |

### 选型捷径

```
嵌课件、无声、可循环？ ──是──► manim-teaching-figures
         │
         否，有旁白/讲课？
         │
         ├─ 已有 Markdown 大纲 ──► claude2video
         ├─ 要完整制作流水线 ──► manim-video
         └─ 只要某个数学 pattern 快出 ──► recursive-math-animator

都不像 / 要查 API ──► manim-skill
```

**一次只选一个**执行 skill。不要并行开两个 Manim 流水线。

## 3. 执行步骤

1. 用上表选定 `id`，说一句选型理由（给用户看）。
2. 若 `vendor/<id>/SKILL.md` 不存在 → 按 [references/catalog.md](references/catalog.md) prepare，写 `VENDOR.md`。
3. `Read vendor/<id>/SKILL.md`（及其它它要求打开的 references），**严格按其流程**做：选题 → 写 scene/plan → 渲染 → 自检（如 loop 接缝）。
4. 交付：视频路径 +（若嵌页）HTML 片段/图注；教程项目落盘到文档 assets 并引用。
5. prepare 或渲染失败 → 按 catalog 降级，或退回 explain-it 换 HTML 分步/静图，并说明缺什么。

依赖（Manim、LaTeX、ffmpeg、TTS）一律跟被选 skill 的说明；要 sudo 时装系统包前停下问用户。仓库内 `vendor/` 为唯一 skill 来源，不默认装到用户家目录。

## 4. 与兄弟 skill 的边界

| 需求 | 去哪 |
|------|------|
| 诊断「听不懂」再决定要不要视频 | `skills/explain-it`（它在要视频时再进来） |
| 从零手搓教程课文要讲解片 | `skills/clone-from-scratch-tutor`（批准后进本路由器） |
| 静图 / 架构 SVG | `skills/draw-diagram` |
| 多面板讲解 HTML、非视频 | `vendor/answer-me-with-html` |
| 逐步点控状态演示 | `vendor/courseware-builder` |
| 架构连接线流动 | `vendor/dashmotion` |

## 红线

- 不跳过选型直接手搓 Manim「先做出来再改」
- 不占时间/第三维的内容不做视频
- 短循环嵌页不要用带播放器的长旁白片冒充；长旁白不要硬做成无缝 loop

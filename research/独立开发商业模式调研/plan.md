# 《独立开发者成功模式深度拆解》PDF报告 — 执行计划

## 任务概述
- 交付物：3万字+ 中文深度报告，PDF（A4打印版）+ Markdown 源文件
- 覆盖8种独立开发者成功模式，每种模式10个固定章节
- 图表：时间线甘特图、收入对比柱状图、模式选择决策树、速查表（全部代码生成）
- 风格：务实、有数据支撑、中立、附来源

## Stage 0 — 技能加载与流水线设计（主控）
- 加载 `deep-research-swarm`（研究阶段）、`report-writing`（写作阶段）的 SKILL.md
- 设计研究与写作批次分配

## Stage 1 — 深度研究（deep-research-swarm，Route B 聚焦搜索变体）
按模式分 4 批并行研究（每批 2 种模式，避免 subagent 超载），每批一个 explore/research 子代理：
- Batch 1: 公开建造型（Pieter Levels）+ SEO驱动型（Danny Postma / HeadshotPro）
- Batch 2: 平台寄生型（Shopify插件/Chrome扩展代表案例）+ 垂直SaaS型（Ilia / Perfect Wiki、AI法律工具案例）
- Batch 3: 开源变现型（Vuetify、NativePHP/Simon Hamp）+ 内容/知识变现型（独立开发者培训博主、Rust电子书作者）
- Batch 4: 套利型（阴明/掘金，AI App月赚9万美元）+ 游戏型（LocalThunk/Balatro、Eric Barone/星露谷）

每个研究子代理必须收集（对应报告10章节所需素材）：
1. 开发者背景故事、入行路径
2. 产品0→1时间线（精确到月份的里程碑）
3. 收入数据（具体数字+来源URL+时间点）
4. 产品现状（运营/出售/关闭）
5. 作息/开发节奏（访谈、博客原文）
6. 技术栈与工具链、基础设施成本
7. 获客与增长策略、冷启动方式、增长数据
8. 变现路径、定价、收入构成、利润率
9. 关键决策点、转折点、危机
10. 效率方法（本人原话/访谈）
11. 2026年可行性判断素材（平台政策、AI竞争、市场变化）
- 输出：每个模式一份研究简报（markdown，含来源URL清单）

## Stage 2 — 研究验证与补充
- 抽查关键数字（收入、时间线）的可信度
- 对缺口数据进行补充检索（verifier 子代理）
- 汇总8份研究简报

## Stage 3 — 写作（report-writing）
- 加载 report-writing 技能，按其规范执行
- 章节划分：引言 + 8章（每模式一章，10小节统一结构）+ 综合对比章 + 模式选择速查表
- 分批写作：每章约 3500-4500 字，由写作子代理基于对应研究简报撰写（研究简报随 prompt 传递）
- 主控统一术语、格式、引用风格

## Stage 4 — 图表生成（主控/coder 子代理，Python matplotlib）
- 时间线甘特图（8个案例关键里程碑）
- 收入对比柱状图（峰值月收入/年收入）
- 模式选择决策树（可用 matplotlib 树状图或 mermaid）
- 模式对比矩阵图/速查表（表格 + 雷达图可选）
- 输出 PNG 到 /mnt/agents/output/assets/

## Stage 5 — 组装与PDF（pdf 技能）
- 加载 pdf 技能，用 HTML+Paged.js 路线生成 A4 打印版 PDF
- 同时交付 Markdown 源文件
- 检查字数 ≥3万、图表嵌入、页码目录

## Stage 6 — 交付
- /mnt/agents/output/独立开发者成功模式深度拆解.pdf
- /mnt/agents/output/独立开发者成功模式深度拆解.md
- KIMI_REF 标签

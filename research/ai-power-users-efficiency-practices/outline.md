# 调研大纲：自称 AI 工作高效的国内外博主/实践者

- **专题目录**：`research/ai-power-users-efficiency-practices/`
- **调研日期**：2026-11-18
- **时间范围假设**：重点 2025–2026；2024 起作背景；经典早期材料仅作对照
- **方法假设**（用户已给定范围，未再 AskUserQuestion）：
  1. 先内部知识列候选，再 WebSearch / WebFetch 核验；每人至少 2 个可核验公开出处
  2. 宁缺毋滥：最终 12 人（国内 3 + 国外 9）
  3. RSI 默认 = Recursive Self-Improvement / 用 AI 改进工作流与能力飞轮；源材料若用 Research-Spec-Implement 等另义，在报告中分栏注明
  4. 「高效」以公开自述 + 可复现方法为主，不采信无出处的倍数宣传

---

## Items（定稿）

| ID | 人物 | 领域标签 | 纳入理由 | 最低出处数 |
|---|---|---|---|---|
| P01 | Lauren Tan（土豆姐 / poteto） | Agent 工程 / Cursor 生态 | 种子案例；pstack 公开技能包 | ≥3 |
| P02 | Simon Willison | 独立开发 / 技术写作 | agentic engineering、TDD、验证工具链 | ≥3 |
| P03 | Boris Cherny | Claude Code 创造者 | 并行 agent、CLAUDE.md、验证闭环公开拆解 | ≥2 |
| P04 | Andrej Karpathy | 研究 / 教育 | vibe coding → agentic engineering 论述 | ≥2 |
| P05 | Shawn Wang（swyx） | 播客 / 社区 | Loop 叙事、Latent Space / Build 演讲 | ≥2 |
| P06 | Addy Osmani | 工程实践写作 | 2026 LLM workflow、Loop Engineering | ≥2 |
| P07 | Geoffrey Litt | 设计工程 / 研究 | 「code like a surgeon」、主/次任务分工 | ≥2 |
| P08 | Pieter Levels | Indie hacker | Cursor vibe coding 公开建造 + 收入自述 | ≥2 |
| P09 | IndyDevDan | 内容创作者 / 工程师 | Claude Skills / subagent 教程体量大 | ≥2 |
| P10 | Paul Gauthier | 开源工具作者 | Aider dogfooding + repo map | ≥2 |
| P11 | laike9m（捕蛇者说） | 中文播客 / Google coding agent 观察 | Ep59 Agent 编程趋势 | ≥2 |
| P12 | 清汤饺子 | 中文实践博主 | Cursor+Claude Code 组合工作流长文 | ≥2 |

### 检索后未单列（可作背景/引用）

- **张小珺｜商业访谈录**：产业与投资视角强，个人编码工作流披露弱 → 报告「公开看法」一节引用，不进人物表
- **小红书「AI编程vibe」等**：粉丝与内容可见，但深度可复现步骤不足 → 工具栈附录提及
- **Cat Wu**：与 Boris 同属 Claude Code 创造者材料，并入 P03
- **Devin / OpenCode 营销文**：缺个人可核验实践 → 不单列

---

## 调研字段（每人）

| 字段 | 说明 | 详细度 |
|---|---|---|
| identity | 身份与领域 | 简要 |
| platforms | 代表平台与代表内容（链接） | 详细 |
| tool_stack | IDE / agent / CLI / 记忆 / 自动化 / 笔记 / 浏览器等及关键功能 | 详细 |
| methodology | 效率方法论 / 工作流步骤 | 详细 |
| why_it_works | 本人解释 + 归纳 | 简要 |
| human_ai_pain | 幻觉、上下文、review、一致性、权限、过度依赖等痛点与对策 | 详细 |
| rsi | RSI/飞轮：提示词库、skill、eval、自建工具、反馈环；若另有 RSI 含义则注明 | 详细 |
| transferable | 对读者可迁移要点 | 简要 |
| credibility | 营销吹捧 vs 可复现实践 | 简要 |

---

## 报告章节映射

1. 执行摘要 → 全字段综合
2. 人物图谱与分类 → identity + platforms
3. 工具全景（强调 Claude/Cursor 之外）→ tool_stack 横向
4. 方法论模式归纳 → methodology 聚类
5. 人机协同痛点对照表 → human_ai_pain
6. RSI 专题（含 pstack 深挖）→ rsi + P01 深描
7. 可行动建议 → transferable
8. 局限与偏见 → credibility 汇总

---

## 交付物清单

- [x] `README.md`
- [x] `outline.md`（本文件）
- [x] `report.md`
- [x] `references.md`
- [x] `people/*.md`（每人一页）
- [x] `quotes/*.md`（关键摘录）

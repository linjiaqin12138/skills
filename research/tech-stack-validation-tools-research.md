# 需求拆解与最小验证路径：现成工具与 SKILL 调研

> 调研目的：市面上是否有现成工具/SKILL，能拆解需求、推荐最合适的**最简单**工具栈用于快速验证，而不是上来就开发应用。
> 调研日期：2026-08-30。以下所有工具均经过逐一联网核查。

## 核心结论

**没有现成工具/SKILL 能做到"拆解需求 → 推荐最小验证路径"的完整链路。** 市面上的方案分两类，各有缺口：

- 会"拆解需求"的（spec-driven 类），终点都是写代码——正是要避免的方向
- 会"不开发先验证"的（pretotyping 类），但不做"工具栈映射"

问卷式技术栈推荐器是最接近的品类，但实测它们大多默认往"写代码"方向推（多为获客工具），真正先判断"该不该写代码"的很少。

---

## 一、本仓库已有的 SKILL（最接近目标）

位于 `skills/` 目录，基于 The Minimalist Entrepreneur 理念：

### validate-idea

- 路径：`skills/validate-idea/SKILL.md`
- 功能：先卖后建——定义问题 → 手动交付（concierge）→ 验证付费意愿
- 明确写了 "Don't write code when a spreadsheet works"

### mvp

- 路径：`skills/mvp/SKILL.md`
- 功能：三阶段 Manual → Processized → Productized
- 工具栈建议直接用 Carrd / Airtable / Gumroad / Zapier / Notion

**缺口**：偏"商业建议"，没有把需求拆成工作流步骤、再逐环节映射到具体现成工具的"工具栈选型器"环节。

---

## 二、已验证真实的问卷式工具（在线，免费）

### PlanMySaaS Tech Stack Recommender

- URL：https://www.planmysaas.com/tools/tech-stack-recommender
- 已验证：页面在线，3 个问题（团队技能 / 应用类型 / 预算策略）→ 输出完整架构建议（前端/后端/DB/Auth/托管），免费免注册，支持 PDF 导出
- 适合：非技术创始人、独立开发者

### BoilerplateHub Tech Stack Recommender

- URL：https://boilerplatehub.com/free-tools/tech-stack-recommender
- 已验证：分步问答 → 推荐技术栈 + **直接匹配对应的生产级 Boilerplate**（Auth、计费、邮件已集成）
- 注意：本质是向自家 100+ 模板库导流的工具，但功能属实
- 适合：想跳过搭建阶段直接写业务逻辑的人

### ZTABS Tech Stack Recommender

- URL：https://ztabs.co/tools/tech-stack-recommender
- 已验证：4 步问卷（项目类型 → 优先级 → 团队规模 → 推荐），页面自述是**规则决策矩阵而非 AI**
- 注意：外包公司 ZTABS 的获客工具，推荐结论偏向其自用的 Next.js/Supabase 栈，有商业倾向
- 适合：需要横向对比的决策者

### ❌ LaunchMVP Tech Stack Recommender（不存在）

- 网传的 `https://www.launchmvp.dev/tools/ai-tech-stack-recommender` **系伪造**——`launchmvp.dev` 域名无 DNS 解析记录，网站不可达
- 教训：AI 生成的工具清单需逐条核实，"听起来合理的域名"是常见幻觉模式

---

## 三、已验证真实的 AI 原生构建器

| 工具 | 状态 | 说明 |
|------|------|------|
| [Atoms.dev](https://atoms.dev) | ✅ 已验证 | 多智能体构建器（调研/架构/工程/营销等 8 个 Agent），基于 MetaGPT/MGX，2026 年 1 月由 DeepWisdom 从 MGX 更名升级上线（非全新产品）。Auth、支付、托管一次性搞定；生态较薄，仅支持 Web |
| [Bubble AI Agent](https://bubble.io) | ✅ 真实 | 自然语言描述功能，Agent 理解数据模型并构建真实工作流；平台锁定严重，无法导出代码 |
| [v0.dev](https://v0.dev) / [Lovable](https://lovable.dev) | ✅ 真实 | 描述 UI 生成可编辑的 React/Tailwind 代码，可导出到 Next.js；偏前端，后端需自己接 |

本质：把"技术选型"这一步彻底跳过，直接验证业务假设。

---

## 四、已验证真实的可集成 SKILL

### claude-cto-team / tech-stack-recommender

- 仓库：https://github.com/alirezarezvani/claude-cto-team（MIT 协议，作者 Alireza Rezvani）
- Skill 文件：`skills/tech-stack-recommender/SKILL.md`
- 已验证内容：
  - 按项目类型 + 团队画像的栈推荐表
  - 前端/后端/数据库/基础设施对比表
  - 4 套栈模板（SaaS、电商、ML 产品、实时应用）
  - 反模式清单和迁移风险评估
- 用法：集成到 AI 工作流，让 AI 在写代码前先过一遍架构决策检查
- 局限：终点仍是"选栈写代码"，不含"是否该写代码"的判断

---

## 五、拆解需求类工具（方向相反，仅供参考）

这类工具把需求拆成 spec/plan/tasks，但终点就是写代码：

- [github/spec-kit](https://github.com/github/spec-kit) — GitHub 官方 Spec-Driven Development 工具包（`/speckit.specify` → `plan` → `tasks` → `implement`）
- [Fission-AI/OpenSpec](https://github.com/Fission-AI/OpenSpec) — 面向 AI coding 助手的 SDD
- Kiro（AWS）— spec 驱动的 IDE
- BMAD-METHOD — 多智能体敏捷框架（Analyst/PM/Architect/SM 角色）
- [mattpocock/skills](https://github.com/mattpocock/skills) — 需求澄清、规格、工单拆解

## 六、验证不开发类工具（pretotyping，方向正确但无工具栈映射）

- lyndonkl/claude 的 **prototyping-pretotyping** SKILL — fake door、concierge MVP、paper prototype、落地页烟雾测试
- **Lean Product Discovery** SKILL（基于 Alberto Savoia《The Right It》）— XYZ 假设、YODA 数据、skin-in-the-game 指标

---

## 七、"验证阶梯"决策框架（可操作的替代方案）

最有效的"SKILL"其实是个决策顺序，遇到新需求时逐级往下走：

```
第 0 步：能不能用现有 SaaS 解决？
    ↓ 能 → Airtable / 飞书 / Notion / 金数据，结束
    ↓ 不能 → 下一步

第 1 步：能不能用无代码/低代码搭建？
    ↓ 能 → Retool / Bubble / Dify / 飞书多维表格，1-3 天验证
    ↓ 不能 → 下一步

第 2 步：能不能用 Boilerplate + 托管平台？
    ↓ 能 → BoilerplateHub 匹配模板（Next.js + Supabase + Vercel），1-2 周验证
    ↓ 不能 → 才考虑自定义架构
```

注意：这不是上述推荐工具的内置逻辑（它们大多默认推"写代码"），需要自己主动套用。

---

## 八、缺口与机会

"拆解需求 → 推荐最小验证路径"的完整链路仍是空白。可自建一个 SKILL（如 `simplest-stack`）：

1. 输入需求 → 拆成核心工作流步骤
2. 逐步骤推荐现成工具（表单用 Tally、数据用 Airtable、支付用 Stripe Payment Link、自动化用 Zapier……）
3. 产出"本周末能上线的验证方案"

与本仓库现有 Minimalist Entrepreneur 风格的 skill 一脉相承，可串联使用：`validate-idea` → `simplest-stack` → `mvp`。

---

## 附：验证记录

| 声称 | 结果 |
|------|------|
| PlanMySaaS Tech Stack Recommender | ✅ 真实，功能相符 |
| LaunchMVP Tech Stack Recommender | ❌ 伪造，域名无 DNS 记录 |
| BoilerplateHub Tech Stack Recommender | ✅ 真实，功能相符（含导流目的） |
| ZTABS Tech Stack Recommender | ✅ 真实，但为规则矩阵且偏自家栈 |
| Atoms.dev 多智能体构建器 | ✅ 真实，2026 年 1 月 MGX 更名上线 |
| claude-cto-team tech-stack-recommender | ✅ 真实，SKILL.md 内容相符 |
| Bubble / v0 / Lovable | ✅ 知名真实产品 |

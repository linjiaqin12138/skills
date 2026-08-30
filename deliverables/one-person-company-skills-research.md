# 一人公司软件项目的 Skill 拆分调研

> 调研日期：2026-08。star 数来自调研时点的第三方页面/GitHub API，仅作数量级参考。

## 一、已有的拆分方案（三个流派）

### 1. 创业旅程式：slavingia/skills ★~10k

- 仓库：https://github.com/slavingia/skills
- 作者：Sahil Lavingia（Gumroad 创始人，《The Minimalist Entrepreneur》作者）
- 拆分：按极简创业旅程拆为 10 个技能
  `find-community` → `validate-idea` → `processize` → `mvp` → `first-customers` → `pricing` → `marketing-plan` → `grow-sustainably` → `minimalist-review` → `company-values`
- 上线一周即登 GitHub Trending；已有多个移植/衍生版
- **局限：只覆盖商业侧，不含设计/开发/运维/客服**
- 本仓库 `skills/` 目录的现有技能即源于此

### 2. 公司部门式：agency-agents / contains-studio

- [msitarzewski/agency-agents](https://github.com/msitarzewski/agency-agents) ★~149k
  144 个 AI 专家 Agent 按公司职能部门分 12 部门：Engineering 24 / Marketing 31 / Design 8 / Sales 8 / Product 5 / Testing 8 / Support 6 / Paid Media 7 等
- [contains-studio/agents](https://github.com/contains-studio/agents)
  按"工作室部门"组织的子代理集，围绕"6 天 MVP 冲刺"方法论
- 特点：有开发环，但无创业阶段叙事

### 3. SaaS 全周期式：BRAINIAC SaaS-blueprint ★4

- 仓库：https://github.com/tuliosousapro/SaaS-blueprint
- 拆分：80+ 目录、16 个阶段：
  Idea → Validation → Planning → Design → Development → Infrastructure → Testing → Launch → Acquisition → Distribution → Conversion → Revenue → Analytics → Retention → Growth → Scaling/Exit
- 另带 14 个 Agent Skills（idea-validator、market-research、competitor-analysis 等）
- 热度极低，但阶段划分是最全的结构参考

### 结论

全生命周期拆分已有先例且正热，但呈"两极"：商业方法论（无开发）或部门角色（无创业叙事）。**按"独立开发者造产品的任务线"把设计/开发/运维串起来的方案，目前基本是空档**——市面上的开发 skill 都按技术栈拆（React、Terraform），不按一人公司的工作流拆。

## 二、工程侧成熟/高热 Skill

### 前端设计 & UI/UX

| Skill | 来源 | 说明 |
|---|---|---|
| frontend-design | [anthropics/skills](https://github.com/anthropics/skills)（官方） | 生成高设计质量、避免"AI 味"的前端界面，被 fork/仿写最多 |
| web-design-guidelines | [vercel-labs/agent-skills](https://github.com/vercel-labs/agent-skills)（Vercel 官方） | 按 Web Interface Guidelines 审查 UI（排版/间距/可访问性），skills.sh 安装量最高之一 |
| design-review | [garrytan/gstack](https://github.com/garrytan/gstack)（YC CEO） | 28 技能集，含"AI Slop 检测" |

### 调试 & 工程工作流

| Skill | 来源 | 说明 |
|---|---|---|
| systematic-debugging、TDD、code-review | [obra/superpowers](https://github.com/obra/superpowers) ★10万+ | 最火的工程流技能库，"先定位根因再动手" |
| investigate、review、land-and-deploy | [garrytan/gstack](https://github.com/garrytan/gstack) | 不调查不修复的根因分析；合 PR→CI→部署→健康检查闭环 |

### 测试

- [anthropics/webapp-testing](https://github.com/anthropics/skills)（官方）：Playwright 测试本地 Web 应用
- [testdino-hq/playwright-skill](https://github.com/testdino-hq/playwright-skill)：70+ 生产验证的 Playwright E2E/POM/CI 模式
- [trailofbits/property-based-testing](https://github.com/trailofbits)：属性测试

### 数据库

- [supabase/postgres-best-practices](https://github.com/supabase)（官方）：Postgres 性能/索引最佳实践
- MongoDB 官方技能集：schema 设计、查询优化、自然语言转查询
- [neondatabase/neon-postgres](https://github.com/neondatabase)：Serverless Postgres

### DevOps / 部署

- [hashicorp](https://github.com/hashicorp) 官方 Terraform 技能集（11 个）
- Cloudflare / Netlify / Vercel 各平台官方部署技能
- [wshobson/agents](https://github.com/wshobson/agents) ★~38k：94 插件、203 子代理、175 技能的多工具市场

### 代码审查

- [coderabbitai](https://github.com/coderabbitai) 官方：AI 审查 + 自动修 PR 评论
- [getsentry](https://github.com/getsentry)：结合线上错误/trace 上下文审查
- [NeoLabHQ/review](https://github.com/NeoLabHQ)：多子代理并行 PR 审查

### 安全审查

- [trailofbits](https://github.com/trailofbits) 官方安全技能集（约 20 个，质量标杆）：differential-review、insecure-defaults、static-analysis
- [openai/security-best-practices](https://github.com/openai)：按语言的漏洞审查、威胁建模

### 聚合入口

- [anthropics/skills](https://github.com/anthropics/skills)（官方，~150k stars）
- [VoltAgent/awesome-agent-skills](https://github.com/VoltAgent/awesome-agent-skills)（1000+ 技能索引）
- [ComposioHQ/awesome-claude-skills](https://github.com/ComposioHQ/awesome-claude-skills) ★~74k
- [alirezarezvani/claude-skills](https://github.com/alirezarezvani/claude-skills) ★~25k，380+ 技能

## 三、商业/增长侧成熟/高热 Skill

### 总标杆：coreyhaines31/marketingskills ★~46k

- 仓库：https://github.com/coreyhaines31/marketingskills
- 作者：Corey Haines（Swipe Files 创始人）
- 40+ 营销技能，分 9 类：CRO、内容文案、SEO、付费投放、度量、留存、增长工程、策略（pricing/launch/marketing-psychology）、销售 RevOps
- **关键设计：hub-and-spoke 结构**——一个 `product-marketing` 上下文文件被所有技能读取，技能间互相引用。这是值得借鉴的差异化设计

### 按子领域

| 子领域 | 代表 Skill | 说明 |
|---|---|---|
| 想法验证 | [RohitWaghire/Deep-Market-Reasearch](https://github.com/RohitWaghire/Deep-Market-Reasearch) | 挖 Reddit/Quora/X/HN 痛点，输出 GO/PIVOT/KILL 报告 |
| 想法验证 | [veyralabsgroup/venture-analyst](https://github.com/veyralabsgroup/venture-analyst) | 问题发现→竞对图谱→可行性评分→验证实验设计 |
| MVP 范围 | [contains-studio/agents](https://github.com/contains-studio/agents) | "6 天 MVP 冲刺"；RICE/ICE/MoSCoW 类小技能泛滥，**无爆款，属空白** |
| 定价 | marketingskills 的 `pricing` | 事实标准：价值指标、Good/Better/Best 分层、Van Westendorp |
| SEO | [AgriciDaniel/claude-seo](https://github.com/AgriciDaniel/claude-seo) | 25 子技能 + 18 并行 agent，含 GEO/AEO（AI 搜索优化），最重的专项方案 |
| Product Hunt 发布 | [yoanbernabeu/producthunt-skills](https://github.com/yoanbernabeu/producthunt-skills) | 31 个发布技能，但仅 18 stars |
| Reddit | [piupiuyao/reddit-founder-skill](https://github.com/piupiuyao/reddit-founder-skill) | 写"不像 AI、不被踩"的 Reddit 帖子 |
| 冷触达 | marketingskills 的 `cold-email` + `prospecting`；[felixleezd/cold-message-writer](https://github.com/felixleezd/cold-message-writer) | 邮件/LinkedIn/X DM，含跟进序列 |
| 客服/反馈分诊 | [murphye/agent-skills-customer-service](https://github.com/murphye/agent-skills-customer-service)；anthropics `ticket-triage` | 工单分诊、SLA、情绪识别；现有方案偏企业客服台/GitHub issue |
| 指标分析 | marketingskills 的 `analytics`/`ab-testing`；saas-metrics-coach | MRR/ARR/NRR/churn/LTV:CAC 计算 + 基准解读 |
| 法律 | [zubair-trabzada/ai-legal-claude](https://github.com/zubair-trabzada/ai-legal-claude) | 14 个法律技能：隐私政策、ToS、合同审查、合规审计 |

## 四、中文生态

- [datawhalechina/hello-claw](https://github.com/datawhalechina/hello-claw) ★~2.2k：「龙虾大学：一人公司实战」，把 agency-agents 的 144 个 Agent 注册成可路由的公司职能 skills
- [博客园：《小而美》炼成 10 个技能](https://www.cnblogs.com/ghj1976/p/19824196)：slavingia/skills 中文深度解读
- [腾讯云：一人公司 70+ Skill](https://developer.cloud.tencent.com/article/2712104)：按 8 大行业/职能拆 AI 技能

## 五、对本仓库的建议

### 值得补的子领域（按生态空档排序）

1. **前端设计开发**（已有 `frontend-design`，可对齐 anthropics 官方版做差异化）
2. **MVP 优先级决策**（RICE/ICE/Kano 打分，生态无爆款）
3. **用户反馈分诊（solo 场景）**（现有方案偏企业客服台/GitHub issue）
4. **上线发布编排**（Product Hunt/Reddit/X 一条龙，现有 skill 弱且分散）
5. **通用重构与文档写作**（无统治级选手）
6. **系统化调试/TDD 工作流**（obra/superpowers 已证明需求巨大，但那是工程通用向，可做"一人公司节奏"向的裁剪）

### 结构建议

借鉴 marketingskills 的 hub-and-spoke：

- 增加一个**产品上下文公共文件**（ICP、技术栈、定位、语气），所有 skill 读取它
- 技能间显式交叉引用（如 `pricing` 引用 `first-customers` 的反馈、`launch` 引用 `find-community` 的渠道清单）

### 一人公司完整子领域地图（建议目标态）

```
商业侧（已有）: find-community → validate-idea → mvp → first-customers → pricing
              → marketing-plan → grow-sustainably → minimalist-review
工程侧（待补）: frontend-design → backend-dev → testing → deploy → debug → review → docs
运营侧（待补）: launch（PH/Reddit/X 编排）→ feedback-triage → metrics → legal
```

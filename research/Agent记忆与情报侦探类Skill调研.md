# Agent 记忆方案与「情报侦探类」Skill 调研

> 调研日期：2026-11-03
> 需求拆成两问：
> 1. 当前主流的 Agent Memory 方案有哪些，实现原理是什么？
> 2. 有没有现成 Skill 能做「持续阅读新闻 / 大V博客 / 上市公司公告 / 热搜 → 建立关联 → 像侦探一样推测内幕交易与政策后续」这件事？类似 `last30days` 的还有哪些？
>
> 资料来源：各项目官方仓库、官方文档与 arXiv 论文原文。**未逐一手工部署验证**，性能与质量描述多为项目自述，需自行复测。

---

## 第一部分：Agent Memory 主流方案

### 核心结论

所谓「Agent Memory」并不是让模型参数持续训练，而是：

> 把对话、事件、事实和经验保存到模型外部，在需要时检索、压缩，再注入上下文窗口。

完整系统一般包含五步：

```text
交互记录
   ↓ 提取：什么值得记？
候选记忆
   ↓ 合并：新增 / 更新 / 作废 / 忽略？
持久化存储：原始日志 / 文档 / 向量 / 知识图谱 / 文件
   ↓ 检索：关键词 + 向量 + 时间 + 图遍历 + 重排
相关记忆
   ↓ 注入
模型上下文窗口
```

各方案的主要差异集中在四点：**记成什么结构、谁决定写入、如何处理冲突、如何检索**。

### 主流方案对比

| 方案 | 核心路线 | 最适合 | 主要代价 |
|---|---|---|---|
| Mem0 | LLM 提取事实 + 向量检索 + 自动增删改 | 快速给现有 Agent 加用户长期记忆 | 抽取与冲突判断依赖 LLM，可能误记 |
| Zep / Graphiti | 双时序知识图谱 | 事实不断变化、关系复杂、需追溯历史 | 写入流程重，图数据库运维复杂 |
| Letta / MemGPT | Agent 自己管理分层记忆 | 长期运行、自主工作的 Agent | 自主调用不可控，运行时绑定较强 |
| LangMem | 语义 / 情景 / 程序记忆工具包 | LangGraph 用户、高度定制记忆逻辑 | 存储与检索策略需自己设计 |
| Memobase | 用户画像 + 事件时间线 | 陪伴、教育、客服等用户个性化产品 | 偏用户记忆，不是通用知识记忆 |
| Cognee | 文档/对话转知识图谱，GraphRAG 检索 | 企业知识、研究资料、多源知识融合 | 图谱构建成本高，偏知识工程 |
| Anthropic Memory Tool | Agent 用文件工具维护记忆目录 | 编程、研究、长任务项目状态 | 检索与组织主要靠 Agent，自动索引弱 |
| A-MEM | 自组织、互相关联、持续演化的记忆笔记 | 研究与新型记忆架构 | LLM 调用多，工程成熟度有限 |

### 各方案实现原理

#### 1. Mem0：事实抽取 + 向量检索 + 自动合并

典型的「外接记忆层」，应用主要调 `add()` 与 `search()`。

写入流程：

1. 将最近消息、当前对话和会话摘要交给 LLM；
2. LLM 提取值得长期保留的候选事实（如「用户不吃乳制品」）；
3. 用 Embedding 找出最相似的一批旧记忆；
4. 再让 LLM 决定 `ADD` / `UPDATE` / `DELETE` / `NOOP`；
5. 保存记忆文本与向量，并记录变更历史。

检索：按当前问题做向量搜索，可叠加 user / agent / run 范围过滤及重排，把少量相关事实注入 Prompt。图扩展 Mem0g 额外抽取实体与关系三元组，通过实体邻域 + 三元组语义相似度检索。论文显示图结构对时间关系更有帮助，但并非所有问题都优于纯文本记忆。

- 论文：https://arxiv.org/html/2504.19413
- 开源文档：https://docs.mem0.ai/open-source/overview

**优点**：接入简单、默认流程完整，适合用户偏好与稳定事实。
**问题**：覆盖式更新会丢失历史（「住上海」覆盖「住北京」），若需完整历史必须自行保留时间版本。

#### 2. Zep / Graphiti：带时间有效期的动态知识图谱

Graphiti 是开源图构建框架，Zep 是建立其上的托管上下文服务，两者不是同一产品。

三层结构：

- **Episode**：原始消息 / 文本 / JSON，保留来源；
- **Entity / Fact**：从 Episode 提取的实体、关系与事实；
- **Community**：强关联实体形成的社区及摘要。

写入：提取实体关系 → 向量 + 全文 + LLM 完成消歧去重 → 抽取事实有效时间 → 检查冲突 → 冲突时不抹掉历史，而是给旧关系设置失效时间。

采用**双时序模型**：

- **有效时间**：事实在现实世界中何时成立；
- **系统时间**：系统何时写入或作废。

因此可以同时回答「现在是什么」和「去年这个时候是什么」。

检索：向量语义搜索 + BM25 关键词 + 图邻域遍历，再经 RRF / MMR / 图距离 / Cross-Encoder 重排，组装为带时间范围的上下文。

- Graphiti：https://github.com/getzep/graphiti
- 论文：https://arxiv.org/html/2501.13956

**优点**：适合动态事实、多跳关系、来源追溯与时间推理。
**问题**：每次写入需多轮抽取、消歧与冲突判断，成本与延迟明显高于简单事实库。

#### 3. Letta / MemGPT：把上下文窗口当「内存」

核心类比是操作系统：上下文窗口 ≈ RAM，外部存储 ≈ 磁盘，Agent 通过工具决定换入换出。

当前 Letta 包含：

- **Memory Blocks**：固定在系统提示词中的可编辑文本块（用户状态、人格、当前任务）；
- **消息历史**：完整持久化，旧消息被压缩或移出上下文后仍保留；
- **Archival Memory**：按需语义搜索的长期资料库；
- **Conversation Search**：搜索真实历史消息，而非提炼后的记忆。

关键区别：**记忆管理本身也是 Agent 的任务**。Agent 可改写记忆块、插入归档内容、按需搜索。

- 论文：https://arxiv.org/html/2310.08560
- 记忆模型：https://docs.letta.com/guides/agents/memory
- Archival：https://docs.letta.com/guides/agents/archival-memory

**优点**：记忆显式可见，适合长期运行、需要维护自身状态的 Agent。
**问题**：可能忘写、写错位置或过度调用工具；关键业务状态不应完全交给模型自由维护。

#### 4. LangMem：语义、情景与程序记忆

不是一体化数据库，而是可组合的 LLM 记忆操作原语。

- **语义记忆**：事实、偏好、实体知识；
- **情景记忆**：过去成功或失败的经验，可转为 few-shot 示例；
- **程序记忆**：Agent 应当如何行动（修改系统提示词与行为规则）。

语义记忆又有两种结构：

- **Collection**：大量独立记忆，按需搜索；
- **Profile**：固定 Schema 的单一画像，收到新信息即更新。

写入支持两种模式：

- **Hot path**：对话中由 Agent 主动调用记忆工具，立即生效但增加延迟；
- **Background**：对话后后台提取合并，不阻塞响应。

存储走 LangGraph Store，以 namespace 隔离组织/用户/应用，支持直读、语义搜索与元数据过滤。

- 文档：https://langchain-ai.github.io/langmem/concepts/conceptual_guide/

**突出特点**：不只记「用户是谁、发生过什么」，还能记「以后应该怎么做」。
**风险**：自动改写系统提示词属高权限操作，生产环境需审核、版本控制与回滚。

#### 5. Memobase：用户画像 + 事件时间线

更专注「理解用户」而非保存任意知识。

流程：对话先进每用户独立 Buffer → Buffer 达阈值或空闲后批量处理 → LLM 整理为可配置 Schema 的画像 → 同时保留事件时间线 → 在线请求直读画像或结合相关事件生成上下文。

与 Mem0 的「大量独立事实」不同，它更像一份持续更新的用户档案。

- 仓库：https://github.com/memodb-io/memobase

**优点**：画像可低延迟直接注入，无需每次先做复杂检索。
**局限**：画像结构需提前设计，对通用知识与复杂实体关系表达有限。

#### 6. Cognee：面向知识图谱的记忆流水线

更接近「GraphRAG + Agent Memory」。

`remember()` 两条路径：

- 带 `session_id`：先写会话缓存快速返回，再后台同步到长期图谱；
- 不带：完整执行数据归一化 → 分块 → 实体关系提取 → Embedding → 图增强。

`recall()` 可组合原始文本块、摘要、实体关系邻域、会话缓存，以及时间、代码等专门检索器。

- Remember：https://docs.cognee.ai/core-concepts/main-operations
- Recall：https://docs.cognee.ai/core-concepts/main-operations/recall

**适合**：企业资料、研究知识库、代码库等需要长期积累并融合多源数据的场景。
**代价**：图构建与增强流程较重，对简单用户偏好记忆偏过度。

#### 7. Anthropic：文件式记忆 + 上下文编辑

方案非常直接：给 Agent 一个专用记忆目录，通过文件工具创建、读取、更新、删除记忆；文件由开发者基础设施持久化；同时用 Context Editing 清理过期工具结果以释放上下文。

不强制知识图谱或向量数据库，Agent 自行维护项目状态、研究笔记、调试经验。

- 官方说明：https://www.anthropic.com/news/context-management

**优点**：透明、易检查、易备份，天然适合 Git、权限控制与人工修改。
**问题**：文件越多，组织与检索越依赖 Agent 自身，更像「可维护的外部笔记本」而非自动记忆数据库。

#### 8. A-MEM：自组织、会演化的记忆网络

借鉴卡片盒笔记法（Zettelkasten）：

1. 每次交互生成一条原子记忆笔记；
2. LLM 补充上下文描述、关键词与标签；
3. 向量搜索找到相关旧笔记；
4. LLM 建立新旧笔记之间的链接；
5. 新记忆可反向触发旧笔记描述、标签、关键词的更新。

旧记忆写入后并非不变，整个网络随新经验持续重组。

- 论文：https://arxiv.org/html/2502.12110
- 代码：https://github.com/WujiangXu/AgenticMemory

比固定实体—关系图谱更灵活，但每次写入需更多 LLM 判断；记忆不断被改写后，来源一致性与可审计性更难保证。

### 常见概念辨析

- **会话历史 ≠ 长期记忆**。如 OpenAI Agents SDK Sessions 会在运行前加载历史、运行后保存新消息，并提供截断与压缩，主要解决连续对话，不自动完成事实提取、冲突消解与经验学习。
  https://openai.github.io/openai-agents-python/sessions/
- **RAG ≠ Agent Memory**。普通 RAG 主要检索静态文档；Agent Memory 还要解决持续写入、个性化、遗忘、冲突、时间变化与行为经验。
- **记忆 ≠ 越长越好**。无筛选保存所有内容会带来噪声、矛盾、隐私暴露与检索精度下降。

### 评测参考

不建议仅按各厂商公布的 LOCOMO 分数排名：模型版本、写入方式、检索数量与评测脚本不同，结果未必可横向比较。应使用自己的数据测试**事实更新、时间推理、多会话关联、错误信息拒答、遗忘与删除**。

- LongMemEval（信息提取、多会话推理、时间推理、知识更新、拒答）：https://arxiv.org/abs/2410.10813

---

## 第二部分：情报侦探类 Skill 调研

### 结论

> 目前没有一个成熟 Skill 能单独完成「持续阅读 → 长期记忆 → 人物/公司关系发现 → 多假设推理 → 概率预测 → 事后校准」的完整闭环。
> 但用现有 Skill 组合，已经能搭出相当接近的系统。

### 候选方案总览

| 方案 | 覆盖能力 | 与目标差距 |
|---|---|---|
| last30days | 多平台近期研究、定时 watchlist、SQLite 存储、日/周报、历史研究库 | 缺实体关系图谱与长期假设管理 |
| last30days-cn | 微博/小红书/B站/知乎/抖音/公众号/百度/头条及中文热榜 | 采集聚合为主，缺深层推理 |
| competitor-news-monitor | 持续监控公司、事件去重、重要性判断、一手来源、增量状态 | 不维护复杂人物关系与长期假设 |
| OSINT Skill | 公司/人物调查、实体关联、时间线、迭代 pivot、持久记忆 | 偏主动调查，非无人值守阅读 |
| structured-analytic-skills | 竞争假设、弱信号、来源评级、贝叶斯更新、情景预测、Brier | 是方法库，需外部采集与调度 |
| superforecasting-skill | 可验证预测、概率更新、预测台账、到期结算、校准 | 不负责采集与关系发现 |
| geopolitical-analyst-skill | 实时地缘新闻、人物/国家利益、关系网络、红队、概率情景 | 地缘政治专用，预测能力需回测 |
| Longbridge / cninfo | 上市公司新闻、SEC/巨潮公告、内部人交易、财务 | 是一手数据源，不是完整情报系统 |

### 1. last30days：已超过「一次性查 30 天」

仓库：https://github.com/mvanhorn/last30days-skill

当前能力：

- Reddit、X、YouTube、HN、Polymarket、GitHub、StockTwits 等来源；
- 按互动量、相关性与时效排序；
- 同一事件的跨平台归并；
- `--store` 将结果持续保存到 SQLite；
- `watchlist.py` 定时运行，`briefing.py` 生成日/周简报；
- 本地研究库搜索；
- 发现模式，寻找正在升温的话题；
- 输出结构化 JSON，便于接后续流程。

安装（Claude Code）：

```text
/plugin marketplace add mvanhorn/last30days-skill
/plugin install last30days
```

其他宿主：`npx skills add mvanhorn/last30days-skill -g`。

**判断**：可承担采集层与初级研究库，但保存的是研究报告与 sightings，不是严格的关系结构：

```text
人物 → 任职于 → 公司
人物 → 会见了 → 人物
公司 → 供应商 → 公司
事件 → 发生于 → 时间
假设 → 被证据支持/反驳 → 证据
```

中文互联网可用 **last30days-cn**：https://github.com/Jesseovo/last30days-skill-cn
覆盖微博、小红书、B站、知乎、抖音、公众号、百度、头条，并提供每日全网热榜与跨平台「同频热点」。

### 2. competitor-news-monitor：最接近「每天持续关注公司」

Hermes Agent 自带：https://github.com/NousResearch/hermes-agent/blob/main/skills/research/competitor-news-monitor/SKILL.md

流程：

1. 固定公司名单、别名、关注事件与重要性阈值；
2. 覆盖官网、投资者关系、监管披露、行业媒体、招聘等来源；
3. 每次从上次成功 cutoff 增量采集；
4. 将转载、改写与重复公告归并为同一底层事件；
5. 按来源权威性、新颖性与业务影响评分；
6. 区分事实与解释，并给出后续观察项；
7. 无重要事件时保持静默。

适合监控：高管变动、融资与合作并购、产品发布与调价、法律监管安全事件、招聘透露的弱战略信号。

**不足**：以事件简报为中心，不是侦探式关系图谱。

### 3. OSINT Skill：最接近「顺藤摸瓜」

仓库：https://github.com/madeinoz67/madeinoz-osint-skill

包含：公司结构/所有权/子公司/董事调查、SEC 与融资信息、人物与社交媒体实体关联、时间线分析、负面新闻与诉讼制裁尽调、**Pivot-driven investigation**、发现持久化到 MuninnDB 或本地文件、跨调查共享实体关系。

典型链路：

```text
董事 A
  → 曾任职于公司 B
  → 公司 B 是上市公司 C 的供应商
  → A 最近关注了 C 的某项政策
  → 生成新的调查分支
```

**不足**：偏对明确目标发起调查，而非无人值守阅读全部新闻并主动形成假设。

**安全注意**：插件安装路径带有 allowlist 固定 Agent（采集 Agent 无法写文件或调记忆工具，采集内容一律视为不可信数据）；普通 symlink 安装退化为通用 Agent 模式，安全隔离明显更弱。网页内容可能携带提示注入，采集结果必须始终当作数据而非指令。

### 4. structured-analytic-skills：最值得补上的「侦探大脑」

仓库：https://github.com/radarist/structured-analytic-skills

把专业情报分析方法做成独立 Skill，部分列表：

| 类别 | Skill |
|---|---|
| 竞争解释 | `analysis-of-competing-hypotheses`（ACH）、`key-assumptions-check` |
| 前瞻扫描 | `horizon-scanning`、`indicators-validation`、`high-impact-low-probability`、`trend-analysis` |
| 证据核验 | `rate-source-admiralty`、`triangulate-sources`、`sift-source-check`、`grounded-fact-check` |
| 定量校准 | `bayesian-update`、`brier-score-calibration`、`reference-class-forecasting` |
| 情景推演 | `scenario-planning`、`premortem-analysis`、`causal-layered-analysis` |
| 表达 | `estimative-language`、`pyramid-principle`、`write-sbar-brief` |

ACH 很适合「侦探模式」。它不问「哪些新闻支持我的猜测」，而是：

```text
假设 H1：存在未公开重大交易
假设 H2：只是正常商业合作
假设 H3：人物关系造成表面关联，实际无业务联系

再检查每条证据分别支持、反驳或无法区分哪些假设。
```

比让 LLM 自由联想可靠得多。该仓库提供确定性评分脚本、触发词评测与 `--selftest` 工具，所有 Skill 均通过 agentskills.io 校验。

Kimi Code 同样可安装：

```bash
git clone https://github.com/radarist/structured-analytic-skills.git
kimi --skills-dir "$PWD/structured-analytic-skills/skills"
```

### 5. superforecasting-skill：把「预测未来」变成可检验记录

仓库：https://github.com/deusyu/superforecasting-skill

每个预测必须包含：明确事件、截止时间、第三方可验证的结算条件、初始概率与基准概率、新证据导致的概率变化、最终结果、Brier Score。

反面示例与正面对照：

- ❌「美国政府可能进一步限制某行业。」
- ✅「在 2027-03-31 前，美国政府是否在联邦公报中发布针对 X 类产品的新出口管制最终规则？当前概率 35%。」

用 append-only 的 `events.jsonl` 作为事实来源，禁止结算后改写，防止事后把错误预测重新解释成「当时判断正确」。另有 `sf review` 汇总校准带。

### 6. 地缘政治方向有专门 Skill

仓库：https://github.com/nimaansari/geopolitical-analyst-skill

包含：GDELT 实时新闻、ACLED/UCDP 冲突事件、人物与国家利益分析、联盟与代理人关系网络、制裁与贸易压力、历史类比、红队反驳、多种概率情景、每 6 小时后台监控。

这是「观察总统、政府政策与活动，进而推测未来事件」方向里覆盖最完整的现成项目之一。

**注意**：其「39 个分析模块」「专业级」主要为项目自述，实际预测能力不能据此认定，仍需历史回测与 Brier 校验。

### 7. 上市公司一手数据源

**美股 / 港美股**：https://github.com/longbridge/skills

覆盖公司新闻与公告、SEC EDGAR（10-K / 10-Q / 8-K）、分析师评级、内部人交易、基本面、市场异常与晨报。共 13 个 Skill，分 market-data / technical / derivatives / fundamentals / research / portfolio / quant / watchlist / content / intel / earnings / value-investing。

**A 股**：https://github.com/rollysys/use_cninfo

巨潮资讯网公告查询与下载、PDF 转全文 Markdown、年报/季报/减持/重组等分类、本地缓存、按公司与公告类别检索。

**原则**：这类一手披露应优先于大V转述与热搜结论。

### 8. 记忆层选型

普通向量库只能找到「语义相似」，不一定能发现侦探式关系。更合适的：

- **Mnemon**：https://github.com/mnemon-dev/mnemon —— 时间、实体、因果、语义四类图，`remember / link / recall` 三个原语，支持 Kimi Code 安装（`mnemon setup --target kimi --yes`）；
- **MuninnDB**：https://github.com/scrypster/muninndb —— 实体关联、共同激活形成的 Hebbian 连接、记忆衰减、语义触发器，与上述 OSINT Skill 配合良好；
- **Graphiti** —— 强调事实有效时间、原始来源与历史关系时，仍是最明确的工程选择。

MuninnDB 的「共同激活形成关联」适合发现意外联系，但这种联系只能作为**调查线索**，不能直接当作现实因果关系。

### 其他可选项

- `blogwatcher`（https://github.com/openclaw/openclaw/blob/main/skills/blogwatcher/SKILL.md）：RSS/Atom 博客监控，纯采集层；
- `brand-mention-monitor`（https://github.com/Nimbleway/agent-skills）：品牌提及扫描，含可达性/速度/情绪/风险四维打分与记忆去重，偏营销而非情报；
- `inquisitor`（https://github.com/iamalisson/inquisitor）：假设驱动的问题求解，含失败台账与「同一家族连续两次假设失败必须重构框架」的规则，方法论上与侦探思路契合，但主要面向编码调试；
- `ai-daily-digest`（https://github.com/HarrisHan/ai-daily-digest）：92 个 Karpathy 精选博客的 RSS 日报，零依赖、零 API Key；
- `Polymarket agent-skills`（https://github.com/Polymarket/agent-skills）：预测市场数据接入（Odds 作为市场隐含概率）。

---

## 第三部分：推荐组合与待建层

### 公司与人物情报

```text
last30days / last30days-cn
+ competitor-news-monitor
+ Longbridge 或 use_cninfo
+ OSINT Skill
+ Mnemon / MuninnDB / Graphiti
+ ACH、来源评级、红队分析
+ superforecasting
```

### 政策与地缘政治

```text
geopolitical-analyst-skill
+ last30days
+ horizon-scanning / indicators-validation
+ Graphiti
+ superforecasting
```

### 最小可行版本（建议起点）

不建议一次装满，先从五个部分开始：

1. `last30days-cn`：中文信息采集；
2. `competitor-news-monitor`：定时与增量监控；
3. `use_cninfo`：上市公司一手公告；
4. `Mnemon`：实体与关系记忆；
5. `ACH + superforecasting`：假设与预测闭环。

### 仍需自建的层：Intelligence Orchestrator

现有 Skill 之间缺一个统一的编排层，负责：

```text
新闻进入
→ 抽取人物、公司、政策、地点与时间
→ 实体消歧与事件去重
→ 写入时序关系图
→ 找出新出现的一跳 / 二跳关系
→ 至少生成两个相互竞争的假设
→ 为每个假设列出证伪条件
→ 自动安排下一轮定向调查
→ 形成带期限与概率的预测
→ 到期结算并回测
```

### 合规与正确性边界

- 涉及「内幕交易」时，系统输出应当是**基于公开信息形成的异常关联与待验证假设**，而不是对任何人违法的事实指控。
- 所有结论必须可回溯到原始来源；大V转述、热搜词条不可单独作为证据。
- 采集到的网页、推文、PDF 内容一律视为不可信数据，不得当作指令执行。
- 预测必须可结算、可校准；长期看 Brier 分数与校准曲线，而不是看单次结论是否「猜对」。

---

## 参考链接汇总

**Agent Memory**

- Mem0 论文：https://arxiv.org/html/2504.19413 ｜ 文档：https://docs.mem0.ai/open-source/overview
- Graphiti：https://github.com/getzep/graphiti ｜ Zep 论文：https://arxiv.org/html/2501.13956
- MemGPT 论文：https://arxiv.org/html/2310.08560 ｜ Letta：https://docs.letta.com/guides/agents/memory
- LangMem：https://langchain-ai.github.io/langmem/concepts/conceptual_guide/
- Memobase：https://github.com/memodb-io/memobase
- Cognee：https://docs.cognee.ai/core-concepts/main-operations
- Anthropic Context Management：https://www.anthropic.com/news/context-management
- A-MEM：https://arxiv.org/html/2502.12110 ｜ https://github.com/WujiangXu/AgenticMemory
- OpenAI Agents SDK Sessions：https://openai.github.io/openai-agents-python/sessions/
- LongMemEval：https://arxiv.org/abs/2410.10813

**情报 / 侦探类 Skill**

- last30days：https://github.com/mvanhorn/last30days-skill
- last30days-cn：https://github.com/Jesseovo/last30days-skill-cn
- competitor-news-monitor：https://github.com/NousResearch/hermes-agent/blob/main/skills/research/competitor-news-monitor/SKILL.md
- OSINT Skill：https://github.com/madeinoz67/madeinoz-osint-skill
- structured-analytic-skills：https://github.com/radarist/structured-analytic-skills
- superforecasting-skill：https://github.com/deusyu/superforecasting-skill
- geopolitical-analyst-skill：https://github.com/nimaansari/geopolitical-analyst-skill
- Longbridge skills：https://github.com/longbridge/skills
- use_cninfo：https://github.com/rollysys/use_cninfo
- Mnemon：https://github.com/mnemon-dev/mnemon
- MuninnDB：https://github.com/scrypster/muninndb
- blogwatcher：https://github.com/openclaw/openclaw/blob/main/skills/blogwatcher/SKILL.md
- brand-mention-monitor：https://github.com/Nimbleway/agent-skills
- inquisitor：https://github.com/iamalisson/inquisitor
- ai-daily-digest：https://github.com/HarrisHan/ai-daily-digest
- Polymarket agent-skills：https://github.com/Polymarket/agent-skills

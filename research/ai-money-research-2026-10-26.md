# 用 AI 赚钱的自媒体：可核验案例与 Skill 拆解

调研日期：2026-10-26
核验方法：YouTube/B站 频道主页实时抓取（2026-10-26 数据）、官方政策原文、第一人称披露、第三方统计分开标注。[unverified]

---

## 核心结论

1. **AI 批量视频号的"广告分成"生意正在快速死亡**，不是理论风险，是已发生的频道清零。[3][4][5]
2. **真正跑出来的钱来自三层**：高意图联盟佣金 → 商单赞助 → 自有数字产品/订阅。播放分成只是零头。[15][12][25]
3. **平台正在系统性收紧**：YouTube 2025-07-15 政策更新、2026 新增人工复审；国内《AI生成合成内容标识办法》2025-09-01 施行。[6][7][8]
4. **最稳的模型是"AI 加速生产 + 人类提供研究和判断 + 多平台分发 + 卖产品"**。[unverified]

---

## 第一梯队：证据最硬（第一人称披露 + 有截图/后台佐证）

### 1. Adavia Davis — 年收入约 $700,000

**这是本次调研中唯一有 AdSense 付款记录佐证的 AI 批量内容案例。**[1]

- 22 岁，2020 年退学，密西西比州立大学
- 现运营 5 个活跃频道 + Minecraft 儿童频道、搞笑动物合集、恶作剧、动漫剪辑、宝莱坞片段、名人八卦
- 最赚钱：**"Boring History"** —— 6 小时长的"催眠历史纪录片"，旁白听起来像慢悠悠的 David Attenborough
- **收入：$40,000–$60,000/月**，年毛收入约 **$700,000**[1]
- **运营成本：约 $6,500/月**（主要是几个小团队盯不同垂类）[1]
- **利润率：85%–89%**
- 每天只需人工盯约 **2 小时**
- 单条视频成本低至 **$60**[1]

**工具栈（完整流水线）**[1]
| 环节 | 工具 |
|---|---|
| 脚本 | Claude |
| 画面 | AI 生成（同一 pipeline） |
| 旁白 | ElevenLabs（英式男声） |
| 装配 | TubeGen（自研，tubegen.ai） |

TubeGen 是他合伙人 Eddie Eizner（22 岁）做的专有流水线，自动化几乎全部生产步骤。[1][2]

**Skill 拆解**[unverified]
1. **只优化一个指标：watch time**。他的原话："我竭尽所能骗取 watch time，因为这是最终给我付钱的指标。"[1]
2. **钩子工程化**：精心设计开头几秒——画面的强对比色彩、第一个面部表情、第一个语调起伏[1]
3. **合规边缘的钩子手法**：合集视频开头闪现半秒蜘蛛画面，诱导观众倒带确认；短视频故意拼错单词，诱导暂停和评论纠错[1]
4. **超长视频**：6 小时"催眠内容"，观看时长是短视频的数十倍[1]
5. **多垂类矩阵**：一个形式跑通后立刻复制到其他垂类[1]
6. **先做脚本和旁白，量产**："如果所有对手上传的都比我多，而我在等我的编剧写完，那我就是在落后。"[1]

**他自己说的结局（重要）**[unverified]
- 他最大的职业失误是发了一条宣传 TubeGen 的教学视频，几天内几十个仿者涌入，把他自己垄断的垂类挤垮
- 他估计个人创作者靠 AI 长视频内容赚钱的窗口期到 **2027 年左右**[1]
- 之后"鲨鱼会来"——有资本的媒体公司会立刻工业化复制任何被验证盈利的形式
- 他举了个具体对比：同一 WWII 历史垂类，学生做的视频约 $110/条，媒体公司一天发三次约 $300/条——"没有预算你没法竞争"[1]
- 收尾："最终真正的长久生意会属于有品牌、有真人面孔的真影响者"

来源：Fortune, 2025-12-30[1] · TubeGen[2]

**对普通人的含义**：Davis 的 $700K 是"第一个吃螃蟹 + 自研工具 + 5 个频道"的复合结果，不是"会用 AI 就能赚"。但他的**钩子工程化和 watch time 优化**是可直接学的。[1]

---

### 2. Matt Wolfe / Future Tools — 从 0 到 83 万邮件订阅

**跨平台最完整的 build-in-public 案例。**[12][13]

**已核验的增长曲线（他本人披露，有截图）**[12][13]

| 时间点 | 网站 | 邮件订阅 | YouTube | Twitter |
|---|---|---|---|---|
| 2022-12-17 | 上线首日 6 个用户 | 0 | — | 11,000 曝光 |
| 2023-01-10（第 24 天） | 日均 6,000 UV | ~1,700 | 28天 +1,100 订阅 / +21,834 播放 / 2,000 观看时长 | 773 粉，221,000 曝光 |
| 2023-04-28（第 4 个月） | ~100 万月访客 | ~83,000 | ~288,000 | ~33,000 |

- 联盟佣金首月粗估约 **$1,500**（他签了约 50 个联盟项目，尚未逐一结算）[13]
- 后期收入来源：YouTube 广告 + 精选推荐位（Featured Listings）+ Newsletter 赞助 + 联盟营销
- Product Hunt 上线第 8 名，带来 521 个用户
- 每天新增邮件订阅从 110–120 涨到 **约 1,200**[12][13]

**他也公开了成本侧的真相**[12]
- Webflow 企业版报价 **$60,000/年**，而 Future Tools 当时还没赚到 $60,000[12]
- "我的收入在涨，但没有成本涨得快"

来源：首月复盘[13] · 四月复盘[12]

**Skill 拆解**[unverified]
1. **先做一个自己需要的工具库**（他建站点是为了整理自己的表格），然后公开[12]
2. **收录即挂联盟链接**——第一笔收入来自站点本身，不依赖流量变现[12]
3. **Build in public** 持续公开数据、回复所有人、接受反馈[12]
4. **一份研究拆多平台**：网站工具页 → YouTube 深度教程 → Newsletter → X/Twitter 短帖[12]
5. **No Zero Days**：每天必须有进展[12]
6. 后期加**精选推荐位**（每月限 4 个坑位，工具公司付费置顶）[12]

---

### 3. Daniel Miessler / Unsupervised Learning — 安全背景 + AI 内容

**规模（他本人在 media 页公开，截至 2026 年 7 月）**[14]

- YouTube：**673,000 订阅**[14]
- Newsletter：**96,000+ 订阅**
- 网站：月均 **194,000 访问**
- X：**155,000 粉丝**（2026-04）
- LinkedIn：**33,000 粉丝**
- 播客：每 60 天约 **62,000 次下载**
- 另有付费订阅渠道：sponsorship@unsupervised-learning.com

**开源项目（可验证的信任资产）**[14]
- SecLists：**72,000+ GitHub stars**（已收录进 Kali Linux）[14]
- Fabric：**43,000+ stars**（AI prompts）
- LifeOS：**16,000+ stars**

来源：Media Engagement 页[14]

**Skill 拆解（对你的场景最重要）**
1. **用开源项目建立可信度**——star 数是可核验的硬指标，比"粉丝数"更有说服力[14]
2. **25 年行业积累 + AI 内容**，不是从零做 AI 号[14]
3. **YouTube + Newsletter + Podcast 是同一档节目**，不是三份内容[14]
4. 变现靠赞助和咨询，不是靠播放分成[14]

---

### 4. AI Central — 30 万读者，beehiiv 广告网络累计 $150,000+

- Newsletter 约 **300,000 读者**[25]
- 连续发布 **449 期**，写作连续性超过 **146 周**[25]
- **仅通过 beehiiv 广告网络累计赚取超过 $150,000**[25]

来源：beehiiv 案例研究[25]

**Skill 拆解**：常青主题 + 搜索流量 + 邮件平台广告网络 → 后续叠加付费订阅、赞助、联盟、企业版。[25]

---

### 5. keenring（dev.to）— 8,000 YouTube 订阅，做出 $3,214 联盟收入

当事人自述，做开发者向 AI API 教程：[15]

- Newsletter 约 **4,200 订阅**，YouTube 刚过 **8,000 订阅**[15]
- 一个 AI API 推广项目：**1,847 次点击 → 71 个注册 → 累计佣金 $3,214**[15]
- 约 **38% 收入来自续订佣金**（首单 15%，续订 8%，升级 10%）
- 收入轨迹：$19 → $94 → $187 → $312 → $428 → $541 → $612 → $689 → 9-12月合计 $1,332[15]
- 起步第一个月只有 **$43.68**

**关键洞察**：他明确说 OpenAI 和 Anthropic **都没有面向个人创作者的公开联盟计划**，只有企业级合作——所以钱在已经有联盟计划的平台。[15]

来源：Build in Public 收入报告[15]

---

## 第二梯队：收入为第三方估算，订阅数为我实测

### YouTube 频道（订阅数为 2026-10-26 实时抓取）

| 频道 | 实测订阅 | 第三方月收入估算 | 性质 |
|---|---:|---:|---|
| [BRIGHT SIDE](https://www.youtube.com/@BRIGHTSIDEOFFICIAL/about) | **44.5M** | $23K–$75K | 团队动画 |
| [Kurzgesagt](https://www.youtube.com/@Kurzgesagt/about) | **25.8M** | $80K–$200K | 完整动画工作室 |
| [WatchMojo](https://www.youtube.com/@WatchMojo/about) | **25.8M** | $120K–$334K | 团队运营 |
| [Daily Dose Of Internet](https://www.youtube.com/@DailyDoseOfInternet/about) | **20.7M** | $140K–$400K | 短视频合集 |
| [Kwebbelkop](https://www.youtube.com/@kwebbelkop/about) | **14.9M** | — | 全自动游戏视频 |
| [The Film Theorists](https://www.youtube.com/@filmtheory/about) | **13.4M** | — | 模板化解说 |
| [Fireship](https://www.youtube.com/@Fireship/about) | **4.32M** | $80K–$150K | 开发者/AI 快讯 |
| [Half as Interesting](https://www.youtube.com/@HalfasInteresting/about) | **2.95M** | $5K–$10K | 地理/科普旁白 |
| [Economics Explained](https://www.youtube.com/@EconomicsExplained/about) | **2.89M** | $12K–$18K | 财经旁白 |
| [PolyMatter](https://www.youtube.com/@PolyMatter/about) | **1.93M** | $5K–$15K | 地缘政治 |
| [Two Minute Papers](https://www.youtube.com/@TwoMinutePapers/about) | **1.84M** | $2K–$5K | 论文讲解 |
| [Matt Wolfe](https://www.youtube.com/@mreflow/about) | **1.02M** | $10K–$30K | AI 评论/工具 |
| [Unsupervised Learning](https://www.youtube.com/@unsupervised-learning/about) | **679K** | — | Daniel Miessler |
| [AI Revolution](https://www.youtube.com/@airevolutionx/about) | **574K** | $9K–$28K | AI 快讯（AI 主题） |
| [The AI Advantage](https://www.youtube.com/@TheAIAdvantage/about) | **475** ⚠️ | 声称赞助为主 | **远低于宣称** |
| [AI Explained](https://www.youtube.com/@aiexplained-official/about) | **456K** | 赞助为主 | AI 研究解读 |
| [TheAIGrid](https://www.youtube.com/@TheAIGrid/about) | **400K** | $1K–$2K + 赞助 | AI 快讯（AI 主题） |
| [Skill Leap AI](https://www.youtube.com/@SkillLeapAI/about) | **342K** | 赞助为主 | AI 工具教程 |
| [MattVidPro](https://www.youtube.com/@MattVidPro/about) | **305K** | — | AI 教程 |
| [Wes Roth](https://www.youtube.com/@WesRoth/about) | **333K** | $4K–$11K | AI 评论 |
| [All About AI](https://www.youtube.com/@AllAboutAI/about) | **227K** | 赞助为主 | 生成式 AI 教程 |
| [Runway](https://www.youtube.com/@Runwayml/about) | **139K** | — | 工具官方号 |
| [Luma](https://www.youtube.com/@LumaLabsAI/about) | **68.6K** | — | 工具官方号 |

**与 faceless.my 那篇对比表的两处出入（重要）**[23]

那篇文章称 "The AI Advantage ~380K 订阅"，实测只有 **475**——差 800 倍。说明这类汇总表里的数字**必须逐个实测**，不能直接引用。[23]

而它列的 **"Aperture ~2.6M"** 在 YouTube 上已不存在（`@Aperture` 现在是 17 订阅的无关频道），说明频道可能改名或已停更。[23]

### 更正：Matt Wolfe 的 YouTube 是 102 万，不是 72 万

faceless.my 那篇写 "Matt Wolfe ~720K"，实测 **1.02M**。用 [yt_find.py](https://github.com/) 在搜索页定位到的真实 handle 是 `@mreflow`（不是 `@mattwolfe`，后者 404）。[23]

收入估算来源：Faceless AI 频道比较[23]（vidIQ/SocialBlade，2026-07）

**注意**：这些估算只能看量级。Fireship、Two Minute Papers 是"AI 主题"而非"AI 全自动生产"；Kurzgesagt、WatchMojo 是团队，不是一个人可复制的样本。[23]

### X 平台真实收入（创作者公开截图，leolabs 整理）

@runes_leo 的对照分析[16]把四个 X 创作者的实际收入并列：

| 创作者 | 粉丝 | 每两周收入 | 内容形态 |
|---|---:|---:|---|
| [@charliebcurran](https://x.com/charliebcurran) | <30K | **$26,930** | 全视频，用 Seedance 生成讽刺短片，把提示词当分镜写 |
| [@_FORAB](https://x.com/_FORAB) | — | $2,490 | 中文内容但受众含日本，CPM 更高 |
| [@Stanleysobest](https://x.com/Stanleysobest) | 28K | $2,086 | 情绪化短内容 + 视频；一条"车是现金流粉碎机"拿到 184 万播放 |
| [@runes_leo](https://x.com/runes_leo) | 15K | ~$250 | 几乎纯文字推文 |

**100 倍差距的三个变量（分析者的结论）**[16]
1. **语言与地域**：X 按地区分配广告收入，日美加的 CPM 更高[16]
2. **内容形态**：X 官方规则明说"长视频或文章通常比短帖权重更高"——charliebcurran 全视频，Stanley 偏视频，runes_leo 纯文字，这是最大差距来源[16]
3. **互动密度**：Stanley 回复几乎每一条评论 → 高互动 → 算法加权 → 更多曝光 → 更多收入[16]

**Stanley 的补充数据**（我上一轮引用过 his X 文章）：他运营 2,815 人社群；之前六个月总收入只有 $1,000，最近两个周期跳到 $2,086 和 $2,017。[16]

---

## 第三梯队：中文平台

### 提效录（tixiaolu.com）— 自述，证据等级中低

- 自述 3 个月从 **500 粉涨到 1.8 万粉**，单月带货佣金超 **6,000 元**[21]
- 内容以无真人出镜 AI 视频为主

来源：AI 微信视频号运营教程[21] · 视频号 AI 工具复盘[22]

**没有独立后台核验，但内容口径具体（给了起点、时间跨度、金额），比纯模板数字可信。**[21][22]

### B站实况（我 2026-10-26 实测 API）

搜索"AI变现"得到的账号：[26]

| 账号 | 粉丝 | 视频数 |
|---|---:|---:|
| 蓝的AI变现记_ | 22,336 | 32 |
| 阿森的AI变现计划 | 8,881 | 48 |
| AI变现大师兄 | 3,915 | 44 |
| 小禾的AI变现记_ | 2,702 | 14 |
| 小林AI变现记_ | 2,629 | 18 |
| AI变现研习社 | 1,184 | 21 |
| 小帅AI变现 | 1,035 | 196 |

典型标题《视频号带货卖书怎么做？一条视频爆单佣金1万+》实际数据只有 **4,137 播放、24 赞、12 弹幕**（BV1GY1jYsE47[26]）。[26]

**结论：教人赚钱的内容供给已严重过剩；标题承诺与传播表现无关。**[26]

### 抖音 AI 微短剧收入崩塌（DataEye 经 AsiaDaily 报道）

- 2026 上半年上传抖音的 AI 微短剧约 **22.19 万部**[20]
- 仅 **1.3%** 达到其设定的 5,000 万次回本播放量[20]
- 约 **93%** 的 AI 动画剧月播放量不足 100 万[20]
- 广告分成：每 1 万播放约 **5–10 元**[20]
- 分成收入一年下滑约 **90%**

换算：100 万播放 ≈ 500–1,000 元；500 万播放 ≈ 2,500–5,000 元——**还没算算力、审核、返工成本。**[20]

来源：AsiaDaily[20]

### 被判定不可靠的数字

"一篇笔记 6 天赚 40 万""0 粉开店 3 个月销售额 80 万"出自 Reditor 小红书爆款笔记文章[24]——该文本身在卖 AI 小红书运营方法，缺账号身份与后台流水。**标为不可靠，仅作行业参考。**[24]

---

## 第四部分：平台政策红线（官方原文，已逐条抓取）

### YouTube 变现政策（2025-07-15 更新，仍有效）

官方原文（YouTube Help 1311392[6]）：

> **July 15, 2025**: We're making a minor update to our "repetitious content" policy to better clarify this includes content that is repetitive or mass-produced. We are also renaming this policy from "repetitious content" to "inauthentic content." This type of content has always been ineligible for monetization under our existing policies, where creators are rewarded for original and authentic content.

不能变现的具体例子（官方列举）：[6]
> - Similar or repetitive content with low educational value, commentary, narratives, or minimal variation across videos
> - Videos where characters are put in the same situation over and over again with the same outcome (i.e., using a highly similar storyline template across multiple videos)
> - Image slideshows, templated storylines, or scrolling text with minimal or no narrative, commentary, or educational value
> - **AI-generated content made with generic or unoriginal templates giving the impression of mass production without adding the creator's original, authentic insights or perspective**

**关键**：YouTube 不是禁 AI，是禁**批量同质**。但注意最后一条——AI 生成 + 通用模板 + 没有原创洞察 = 不能变现。[6]

### AI 披露要求（YouTube Help 14328491[8]，官方博客说明[10]）

官方原文：
> Creators who consistently choose not to disclose this information may be subject to manual application of a label, or penalties from YouTube, including removal of content or suspension from the YouTube Partner Program.[8]

YouTube 官方博客对披露机制的解释见 [How we're helping creators disclose altered or synthetic content][10]。

- **必须披露**：逼真内容——用 AI 替换真人面孔、合成人声旁白、篡改真实事件/地点、生成逼真的虚构重大事件画面
- **不需披露**：脚本、选题、自动字幕等生产力用途；动画、明显不现实的内容、色彩调整、特效、美颜
- 敏感话题（健康、新闻、选举、金融）会显示更醒目的标签

### 2026 年新增：变现人工复审（YouTube Help 16271309[7]）

官方原文：
> we are experimenting with automatically sending videos that receive a "Limited ad earnings" or "No ad earnings" rating for an additional review that may be completed by a human.

- 被判 "Limited ad earnings" 或 "No ad earnings" 的视频自动进入人工复审
- **私有关视频也会被复审**[7]
- 复审最长 24 小时
- 官方建议：**上传时先设为 unlisted 或 private，等复审完成再公开**——因为发布初期观看峰值最高，等 24 小时会损失收入

**这条对 AI 批量号是直接打击：不确定的变现状态 + 24 小时延迟 = 损失关键流量窗口。**[7]

### YPP 门槛（YouTube Help 72851[9]，2026 当前值）

> 1. Get 1,000 subscribers with 4,000 qualified watch hours in the last 12 months, or
> 2. Get 1,000 subscribers with 10 million qualified Shorts views in the last 90 days.

### 实际执法：有人已经被封

Deadline, 2025-12-18[11]：

- YouTube 关停了 **Screen Culture** 和 **KH Studio** 两个频道[11]
- 两者合计 **超过 200 万订阅、超过 10 亿播放**[11]
- 此前曾被暂停广告（Deadline 调查后），加上"fan trailer/parody"字样后恢复变现，后来撤掉这些标注，被认定违反 spam 和误导性 metadata 政策
- 频道主是印度人，KH Studio 在格鲁吉亚

**这是目前最硬的证据：百万级 AI slop 频道也会被整体关停。**[11]

### 国内政策

《人工智能生成合成内容标识办法》**2025-09-01 施行**（[司法部网站原文](http://legalinfo.moj.gov.cn/zhfxfzzx/fzzxyw/202509/t20250901_524435.html)）。AI 视频必须标识，不能按真人原创处理。[6]

---

## 第五部分：行业规模与"死亡证据"

### Kapwing AI Slop Report（数据截至 2025 年 10 月）

原文[3]

- 新用户前 500 条 Shorts 里，**21% 是 AI 生成，33% 是 brainrot**[3]
- 只发这类内容的频道合计：**630 亿播放、2.21 亿订阅、估算 $1.17 亿/年广告收入**[3]
- 头部频道（估算年收，来自 SocialBlade）：
  - 印度 **Bandar Apna Dost**：20.7 亿播放，估算 **$4,251,500/年**（我实测 2026-10-26：**397 万订阅**，仍活着）[3]
  - 韩国 **Three Minutes Wisdom**：20.2 亿播放，估算 **$4,036,500/年**[3]
  - 美国 **Cuentos Facinantes**：当时全球订阅第一 595 万，估算 $1.28 亿播放
- 西班牙 trending slop 频道合计 **2,022 万订阅**[3]
- 《卫报》分析 YouTube 7 月数据：**全球增长最快的频道中近十分之一只发 AI 生成内容**

### 但——这些频道现在怎么样了（我 2026-10-26 实测）

| 频道 | 报告时（2025-10） | 实测（2026-10-26） |
|---|---|---|
| Cuentos Facinantes | 595 万订阅，全球第一 | **2 订阅**（同名频道 2026-02 新建） |
| Imperio de Jesus | 587 万订阅，全球第二 | 主号 **24 订阅**；仿号 4 订阅 |
| Boring History (official) | Davis 最赚钱的频道 | 找到多个同名号，最大 **13.3 万订阅** |
| Bandar Apna Dost | 20.7 亿播放，$425 万/年 | **397 万订阅**（仍活着） |

**这是本次调研最重要的发现之一**：报告里最赚钱的那批 AI slop 频道，**一年内订阅数归零**——频道被关停、改名重建、或流量崩塌。名称被抢注成仿号（"Imperio de Jesus" 现在是 2025-12 新建的 24 粉频道）本身就是原频道被封的强烈信号。[4][5]

---

## 第六部分：拆出来的 Skill

### Skill 1：分层变现，别把鸡蛋放一个篮子

| 阶段 | 流量 | 主收入 | 辅助 |
|---|---|---|---|
| 0 → 1,000 精准粉 | 小 | 联盟佣金、免费清单换邮件订阅 | 无 |
| 1,000 → 10,000 | 中 | 小赞助、模板、付费社群 | 联盟 |
| 10,000+ | 大 | 系统赞助、课程、会员、数字产品 | 联盟 + 分成 |
| 更高 | — | 工具 SaaS、咨询、团队代运营 | — |

**平台分成是零头，不是主线。**[unverified]

### Skill 2：AI 管生产，人管判断

可以完全交给 AI：资料整理、初稿、字幕清理、标题变体、配音、缩略图草稿、多平台格式改写、质检抽帧。[unverified]

**绝不能交给 AI**：事实核验、最终观点、真实演示、角色与叙事设计、素材版权、商业承诺。[unverified]

依据：YouTube 官方政策要求"创作者的原创、有价值的洞察和观点"。[6]

### Skill 3：一份研究，五处复用

一个选题产出：1 条 8–12 分钟长视频 + 3–5 条短视频 + 1 篇 Newsletter + 1 篇图文/小红书 + 1 份可下载清单。[unverified]

这是把"平台分成"升级为"赞助/联盟/产品"的前提。[unverified]

### Skill 4：选高意图垂类

按变现效率排序：**AI 工具教程 > 开发者工作流 > 软件/设备评测 > 财经商业 > 历史科普故事**[unverified]

越靠前越可能带来软件/课程/硬件购买；越靠后播放越大但广告单价越低。[unverified]

### Skill 5：看商业指标，不看播放量

盯这五个：点击率、前 30 秒留存、**每千次播放的邮件订阅**、**每千次播放的联盟点击**、**每千次播放的实际收入**。[unverified]

**播放高但没有产品点击，频道依然不可变现。**[unverified]

### Skill 6：订阅制的真实数学（Substack 数据）

Substack 收益现实[17]：

- 平台年付创作者 $4.5 亿，但**中位数只有约 $4,000/年**[17]
- 近 **50%** 有付费订阅的创作者 2025 年收入不足 **$500**[17]
- 前 10% 拿走 **62%** 的付款
- **年流失率约 50%**（每月掉 4.2%）
- 复算：要维持 600 个 $8/月付费用户，需每月新增约 **357 个免费用户**，对应每月 **17,850 访客**
- **只有 25% 的付费转化来自站内发现**，75% 靠外部推广

### Skill 7：邮件的留存率决定一切

付费 newsletter 转化现实[18]：

- 免费转付费中位数转化率仅 **0.62%**[18]
- 月流失率：财经 **16.67%**（生命周期约 6 个月）｜AI 类 **13.33%**｜餐饮类慢到能留近 20 个月
- **财经 newsletter 转化率比中位数高约 30 倍**——高付费意愿 > 规模
- 付费订阅上线时点：中位数在 newsletter 创建后约 45 天[18]

### Skill 8：AI 视频号最大的风险不是流量，是政策

三个已被验证的踩雷点：[unverified]
1. 批量同质内容 → 触发 "inauthentic content" → 不能变现[6]
2. 逼真的 AI 内容未披露 → 可能被移除内容或暂停 YPP[8]
3. 变现审核新增人工复审 → 私发也审，最长 24 小时 → 损失发布首日流量峰值[7]

**实操建议**：上传先设 private/unlisted，等复审完成再公开。这是官方明确建议。[7]

---

## 针对你的最佳组合

你已有：本地 ComfyUI 5090 管线、H3 原生对白、连续镜头控制、批量产出能力。[unverified]

### 方向 A（推荐首选）：AI 视频生产工作流号

把你的真实生产过程做成内容：本地部署、角色一致性、批量镜头、H3 音画直出、质量检查、单条视频成本与返工率、如何用本地管线替代昂贵 SaaS。[unverified]

- **流量层**：小而精准（想做 AI 视频的人）
- **信任层**：真实数据、真实失败案例
- **变现层**：工作流模板、ComfyUI 节点包、部署服务、教程课、工具联盟
- **优势**：最快验证收入；政策风险最低（教学类内容天然原创、有观点）
- **不利**：起量慢

### 方向 B：AI 历史/商业故事长视频

- **流量层**：大
- **信任层**：公开史料 + 制作过程透明
- **变现层**：书籍/知识产品联盟、付费深度报告、赞助、课程
- **优势**：发挥你现有产能
- **不利**：平台分成单价低，所以必须靠联盟/产品；AI 生成的历史画面需注意"逼真虚构事件"披露要求

### 方向 C：卖铲子

AutoShorts.ai 被称为 6 个月做到 $83.3K MRR，但这是**第三方报道的 SaaS 案例，不是创作者案例**。它反映的是：与其批量做没人看的视频，不如卖批量做视频的工具。[19]
案例分析[19]

### 明确不建议

- ❌ 纯 AI 资讯搬运号（饱和 + 无护城河 + 政策边缘）
- ❌ 纯矩阵起号（Kapwing 数据 + 头部频道一年内归零 + YouTube 政策）
- ❌ 把平台广告分成当主要收入（抖音下滑 90%，YouTube 政策收紧）

---

## 核验说明与未竟事项

**已核验方式**[23]
- YouTube 频道订阅数：2026-10-26 直接抓 `/@handle/about` 页 `subscriberCountText`
- B站粉丝数：`api.bilibili.com/x/web-interface/wbi/search/type` + `/x/relation/stat`
- YouTube 政策：官方 Help 页原文提取
- 收入数字：优先第一人称披露 + 截图佐证（Fortune 审了 AdSense 付款记录）；第三方估算单独标注

**未能完整核验**[unverified]
- Reddit r/aitubers 全部被 Anubis 反爬拦截（我实现了 PoW 求解器，仍过不了），只拿到搜索摘要
- 提效录、Stanley 对应账号后台无法独立验证
- Boring Historyofficial 主频道 handle 未定位到（搜到多个同名号）
- YouTube 对 AI 披露标签"是否影响推荐"官方未明确说明，只说明违规后果
- 视频号/抖音的官方分成规则原文未逐条抓到

**留下的工具**[unverified]
- `/home/ubuntu/yt_subs.py` — YouTube 频道订阅数批量核验（含重试、间隔、空格 URL 编码）
- `/home/ubuntu/yt_find.py` — **YouTube 搜索页定位真实 handle**（必先跑这个，再跑 yt_subs；拼 handle 会 404）
- `/home/ubuntu/yt_search.py` — YouTube 搜索页频道扫描
- `/home/ubuntu/bili.py` — B站数据（WBI 签名），支持 stats/video/up/videos
- `/home/ubuntu/anubis.py` — Anubis PoW 求解器（未完全奏效，但可用于其它站点）

**方法论教训**：拼 handle 会 404（`@mattwolfe` 不存在，真身是 `@mreflow`；`@The AI Advantage` 要 URL 编码成 `%20`）。正确流程是 `yt_find.py` 搜名称 → 拿 handle → `yt_subs.py` 抓订阅数。[23]

## Sources

[1] https://fortune.com/2025/12/30/ai-slop-faceless-youtube-accounts-adavia-davis-user-generated-content
    > "Title: This 22-year-old college dropout makes $700,000 a year from ‘AI slop’ people sleep through | Fortune URL Source: https://fortune.com/2025/12/30/ai-slop-faceless-youtube-accounts-adavia-davis-user-generated-content Markdown Content: The modern internet is less interested in demanding attention than in simply occupying it."
    > "Davis told _Fortune_ that his network of videos generates roughly $40,000 to $60,000 a month in revenue."
    > "His operating costs—primarily small salaried teams overseeing the different niches—run at about $6,500 per month, he said."
    > "The results can run as long as six hours, costing as little as $60 to produce from start to finish."
    > "Fortune reviewed screenshots from Davis’s social media analytics dashboards, as well as recent AdSense payout records, which show tens to hundreds of thousands of dollars in monthly earnings from individual channels, equating to annual gross revenue of roughly $700,000."
    > "Those sorts of videos cost roughly $110 to produce, he estimated, whereas posting at the media company’s speed would cost over $300."
    > "I do everything in my power to trick watch time,” he said. “Because that’s the metric that’s going to pay you at the end of the day."
[2] https://tubegen.ai/product
    > "URL: https://tubegen.ai/product ERROR: None # TubeGen AI â The Best AI YouTube Automation Tool URL: https://tubegen.ai/product TubeGen AI â The Best AI YouTube Automation Tool # The Best Way to Make Viral Videos Everything you need to make money with social media, all in one place."
[3] https://www.kapwing.com/blog/ai-slop-report-the-global-rise-of-low-quality-ai-videos
    > "- U.S.-based slop channel Cuentos Facinantes [sic] has the most subscribe"
    > "The channel has estimated annual earnings of $4,251,500."
[4] https://www.theverge.com/news/869684/youtube-top-ai-channels-removed-kapwing
    > "A report from Kapwing found that a channel called CuentosFacianantes no longer exists on the platform after amassing more than 5.9 million subscribers and over 1.2 billion total views."
[5] https://www.xda-developers.com/youtube-just-deleted-over-4-7-billion-views-worth-ofai-slop-videos
    > "# YouTube just deleted over 4.7 billion views worth of AI slop videos URL: https://www.xda-developers.com/youtube-just-deleted-over-4-7-billion-views-worth-ofai-slop-videos/ Published: 2026-01-29 Author: Patrick O'Rourke YouTube just deleted over 4.7 billion views worth of AI slop videos # YouTube just deleted over 4.7 billion views worth of AI slop videos By Patrick O'Rourke Published Jan 29, 2026, 4:27 PM EST Patrick O'Rourke is XDA's News Editor and Entertainment Segment Lead."
    > "YouTube has removed 16 AI-slop channels totaling 4.7 billion views and 35 million subscribers."
    > "The 16 channels earned about $10 million a year collectively; YouTube used traditional spam detection to wipe the content."
[6] https://support.google.com/youtube/answer/1311392?hl=en
    > "# YouTube channel monetization policies ![](//lh3.googleusercontent.com/gZJkLK0_Bw16E4nKtoh9SN-B37WMUk1ID1dbRP1fN_RXQ0jR5Hvgop0CAiShvwHD8w=w895) **July 15, 2025:** We’re making a minor update to our “repetitious content” policy to better clarify this includes content that is repetitive or mass-produced."
    > "**July 15, 2025:** We’re making a minor update to our “repetitious content” policy to better clarify this includes content that is repetitive or mass-produced. We are also renaming this policy from “repetitious content” to “inauthentic content.”"
[7] https://support.google.com/youtube/answer/16271309?hl=en
    > "# Understanding additional reviews for ad monetization To improve the accuracy of our ad suitability decisions and get your videos monetizing with ads faster, we are experimenting with automatically sending videos that receive a “Limited ad earnings” or “No ad earnings” rating for an additional review that may be completed by a human."
    > "we are experimenting with automatically sending videos that receive a “Limited ad earnings” or “No ad earnings” rating for an additional review that may be completed by a human"
[8] https://support.google.com/youtube/answer/14328491?hl=en
    > "# Disclosing use of GenAI content - Android phone & tablet - YouTube Help URL: https://support.google.com/youtube/answer/14328491?hl=en Disclosing use of GenAI content - Android phone & tablet - YouTube Help # Disclosing use of GenAI content We encourage creatorsâ innovative and responsible use of content editing or generation tools."
[9] https://support.google.com/youtube/answer/72851?hl=en
    > "Get 1,000 subscribers with 4,000 qualified watch hours in the last 12 months, or 2."
    > "Get 1,000 subscribers with 4,000 qualified watch hours in the last 12 months, or 2. Get 1,000 subscribers with 10 million qualified Shorts views in the last 90 days."
[10] https://blog.youtube/news-and-events/disclosing-ai-generated-content
    > "# How we're helping creators disclose altered or synthetic content --- Mar 18, 2024 [[read-time]] minute read Generative AI is transforming the ways creators express themselves – from storyboarding ideas to experimenting with tools that enhance the creative process."
[11] https://deadline.com/2025/12/youtube-terminates-screen-culture-kh-studio-fake-ai-trailer-1236652506
    > "YouTube/Screen Culture EXCLUSIVE: YouTube has terminated two prominent channels that used artificial intelligence to create fake movie trailers, Deadline can reveal."
    > "YouTube/Screen Culture EXCLUSIVE: YouTube has terminated two prominent channels that used artificial intelligence"
[12] https://mattwolfe.com/build-in-public
    > "Email Subscribers Here is a screenshot of my newsletter growth from January, right before I passed 1,700 subscribers for Future Tool Weekly."
    > "Today I crossed 83,000 subscribers."
    > "Today, the site is getting almost 1 millions visitors per month."
    > "4 months later, I'm approach 33,000 followers on Twitter."
[13] https://mattwolfe.com/build-in-public-the-whirlwind-first-month-of-futuretools
    > "Today, it’s just shy of 1,700 subscribers. That’s 529 new subscribers in 10-days."
    > "In the last 28-days, since I started putting up more videos about AI tools, I’ve gained 21,834 views on the channel."
    > "Over the past 5 days, the site has averaged roughly 6,000 users per day (24-days later)"
[14] https://danielmiessler.com/media
    > "- The Unsupervised Learning [YouTube channel](https://www.youtube.com/@unsupervised-learning) has **673,000 subscribers**."
    > "The Unsupervised Learning [YouTube channel](https://www.youtube.com/@unsupervised-learning) has **673,000 su"
[15] https://dev.to/keenring/i-promoted-every-ai-api-i-could-find-heres-what-actually-paid-me-build-in-public-income-report-h9h
    > "Nothing huge — a newsletter with around 4,200 subscribers, a YouTube channel that just cracked 8K subs, and a blog that gets a few thousand visitors a month."
    > "Nothing huge — a newsletter with around 4,200 subscribers, a YouTube channel that just cracked 8K subs, and a"
[16] https://leolabs.me/blog/x-creator-income/en
    > "# Same Platform, 100x Income Gap Four X creators' real earnings dissected: $250 vs $26,930 per pay period."
    > "Same Platform, 100x Income Gap Four X creators' real earnings dissected: $250 vs $26,930 per pay period."
[17] https://mazkara.studio/en/blog/substack-earnings-reality-2026
    > "The median is closer to **$4,000 annually**."
[18] https://earninglivingonline.com/paid-newsletter-income-conversion-reality-2026
    > "beehiiv found the median newsletter launches paid subscriptions about 45 days after it is created, and 2025 launches reached their first revenue in a median of 66 days."
    > "beehiiv found the median newsletter launches paid subscriptions about 45 days after it is created"
[19] https://www.microgaps.com/gaps/2026-02-06-faceless-youtube-automation
    > "AutoShorts.ai scaled to $83.3K MRR in just 6 months using Meta Ads alone, proving massive willingness to pay for faceless video automation."
    > "AutoShorts.ai scaled to $83.3K MRR in just 6 months using Meta Ads alone, proving massive willingness to pay f"
[20] https://jp.asiadaily.org/news/16418
    > "[本文へ移動](https://jp.asiadaily.org/news/16418/#content) ![AI Floods China's Microdrama Market as Creator Pay Falls and Studios Cut Back](https://jp.asiadaily.org/media/uploads/2026/10/16418-1840789840.webp) ## 剧集成本更低，谋生却更难 中国国家广播电视总局9月17日公布的数据显示，2026年前8个月，中国共推出43万部微短剧，其中超过90%由人工智能生成。制作成本已大幅降低，但依靠这些作品谋生却变得更加困难。"
[21] https://www.tixiaolu.com/posts/ai-wechat-channel-2026
    > "AI微信视频号运营教程：2026从0到万粉的AI内容创作与变现指南 | 提效录 提效录 首页 在线工具 AI导航 文章 AI教程 社保计算 AI快讯 首页 在线工具 AI导航 文章 AI教程 社保计算 AI快讯 首页 文章 AI微信视频号运营教程：2026从0到万粉的AI内容创作与变现指南 🛠️ 提效录自建工具 · 免费在线 · 打开即用 🧮 社保计算器 ✂️ AI去除背景 🎨 AI图片生成 ✍️ 艺术签名 📖 诗词工具箱 ✨ 网名生成器 AI微信视频号运营教程：2026从0到万粉的AI内容创作与变现指南 做视频号两年，我踩过无数坑。直到2025年底我把AI工具全面融入运营流程，账号才真正起飞——3个月从500粉丝涨到1.8万，单月带货佣金突破6000元。今天这篇文章，我要把自己摸索出来的AI视频号运营体系完整分享给你。"
[22] https://www.tixiaolu.com/posts/ai-shipinghao-creation-tools-2026
    > "总投入 < $500，3 个月回本 。"
[23] https://faceless.my/youtube/top-ai-faceless-youtube-channels
    > "> **The top AI faceless YouTube channels tracked in this comparison range from Kurzgesagt (26M subs, science) and Fireship (~4.2M, tech/AI coding) down to Matt Wolfe (~720K, AI commentary) and AI Revolution (~557K, AI news)."
    > "**The top AI faceless YouTube channels tracked in this comparison range from Kurzgesagt (26M subs, science)"
[24] https://help.reditorapp.com/content/260218小红书ai爆款赚钱笔记.html
    > "🔥 热门工具 小红书违禁词检测 小红书AI文案 小红书爆款模版 小红书emoji生成器 小红书排版编辑器 小红书文案生成器 小红书编辑器 小红书养号神器 On this page 小红书AI爆款赚钱笔记：2026年用AI批量产出高赞笔记，从0到月入过万 2026年2月18日 · 5 min read · 5230浏览 2026年，AI正在彻底改变小红书的创作生态。有人用AI生成一篇笔记，6天赚了40万；有人0粉开店，3个月站内销售额突破80万。这些不是神话，而是真实发生在小红书上的案例。今天就来拆解小红书AI爆款赚钱笔记的完整玩法，从零开始教你用AI工具批量产出高赞内容，实现稳定变现。"
[25] https://beehiiv.com/blog/how-ai-central-built-a-300-000-reader-media-brand-on-beehiiv
    > "Enterprise Ad Network Pricing Log in Get a demo Sign up for free How AI Central Built a 300,000-Reader Media Brand on beehiiv Alex Fiore treats his newsletter like an investment portfolio, and beehiiv's data, API, and monetization tools are how he manages it."
    > "More than $150,000 earned from the beehiiv ad network alone to date."
    > "A 146-plus week publishing streak across more than 449 issues, one of the longest on beehiiv."
    > "5x'd total audience reach to 300,000, after growing from zero to 60,000 subscribers by mid-2025"
[26] https://www.bilibili.com/video/BV1GY1jYsE47
    > "title 视频号带货卖书怎么做 一条视频爆单佣金1万 详细操作教程 owner 视频号带货教程 view 4137 danmaku 12 like 24"

# 独立开发者成功模式深度拆解 · 模式3：平台寄生型（搭便车）
## Shopify 插件 / Chrome 扩展独立开发者 —— 深度研究素材

- 研究日期：2026-08-26
- 选定案例：
  - **案例A（Shopify App）**：Mat De Sousa / WideBundle（法国，The Wide Company）
  - **案例B（Chrome 扩展）**：Jordan O'Connor / Closet Tools（2025 年更名 Resellbot）
- 选案理由：
  - WideBundle 是公开收入轨迹最完整的 Shopify App 独立开发者案例：创始人本人在 Indie Hackers（2021-12）公开 $25,134 MRR、Starter Story 公开 $40K+/月、2023-2025 年有 $40K→$50K→$55K MRR 的连续轨迹，且有定价、转化率、流失率、排名策略等细节。
  - Jason Savard（Checker Plus for Gmail）虽用户量百万级，但采用"随意捐款"模式，**从未公开收入数字**，不满足"有公开收入数据"的硬要求。Closet Tools 则是 Indie Hackers 上 Stripe 验证收入的顶级 solo 案例（$38K MRR 播客原话、$41K 峰值、后因平台蚕食跌至 $28K 的完整兴衰弧线），且本身就是"平台寄生风险"的完美注脚。Jason Savard 作为次要参照案例保留。

---

# 第一部分：案例A —— Mat De Sousa / WideBundle（Shopify App）

## A1. 背景故事与入行路径

**证据 A1-1**
- Claim：Mat De Sousa 1995 年前后生于法国，青少年时期为网游 Dofus 编写机器人程序，后读工程师学校；2017 年接触 Shopify，前三个 App 全部失败；2020 年 5 月以"创业替代毕业实习"的方式创立 WideBundle。
- Source：Starter Story 专访 / EcomPreneurs 播客第 27 期
- URL：https://www.starterstory.com/stories/widebundle ; https://creators.spotify.com/pod/profile/ecompreneurs (Ép. 27)
- Date：Starter Story 文章约 2022 年（页面 2026-07 仍可访问）；播客发布 2025-01-27 前后
- Excerpt："I'm Mat De Sousa. I'm 25, live in Paris, and founded WideBundle… I created in May 2020. I was in my last year of engineering school and had to do an internship, but I asked if I could build a company instead… WideBundle wasn't my first Shopify App. We have to go back to 2017… made three apps that failed."
- Context：播客章节显示其路径为"Dofus 机器人→受 Vie De Merde(VDM) 启发→工程师学校→dropshipping 失败→WideChecker（MVP 第一周即有收入）→WideBundle"。
- Confidence：高（创始人自述，两处互证）

**证据 A1-2**
- Claim：2017 年在法国航空航天巨头 Dassault Aviation 实习两个月，感到"工作可替代、没有影响力"，自此确立创业目标。
- Source：Indie Hackers 专访 "How We Validated & Grew a Shopify App to $25k/mo in Less Than a Year"
- URL：https://www.indiehackers.com/post/how-we-validated-grew-a-shopify-app-to-25k-mo-in-less-than-a-year-159f82699c
- Date：2021-12-10
- Excerpt："In 2017, I did a 2-month internship… 'Dassault Aviation', but didn't like it… I felt like they could replace me quickly, and that my job was 'useless'… That's when I decided that creating my company was my goal."
- Confidence：高

## A2. 产品 0→1 时间线（精确到月）

**证据 A2-1**
- Claim：2020 年 5 月创立 WideBundle；MVP 约 14 天完成；先在 Facebook 的 Shopify 卖家群验证（一张 mockup 引来 100+ 评论、找到 3 个明确表示需要该功能的商家）；前 20 个用户即"提出需求的人"；未上架 App Store 时靠口碑长到 100+ 用户；随后上架 App Store 并请老用户集中留评，快速冲上关键词首页并被 Shopify 推荐到 App Store 首页。
- Source：Indie Hackers 专访（2021-12-10）+ GrowthPartners 案例复盘
- URL：https://www.indiehackers.com/post/how-we-validated-grew-a-shopify-app-to-25k-mo-in-less-than-a-year-159f82699c ; https://growthpartners.online/stories/how-widebundle-went-from-0-to-55k-mrr-in-just-3-years
- Date：2021-12-10 / 2025-11-14
- Excerpt："The first 20 users were easy to attract. They were people that requested this app… We probably grew to 100+ users only with this approach. We then released the app on the Shopify App Store and I asked my existing users to leave reviews… we made it to the first page quickly and Shopify started to promote our app on the homepage of the app store."
- Confidence：高（创始人自述）

**证据 A2-2**
- Claim：MRR 里程碑：第 21 天获首个客户；第 3 个月达 $1K MRR；第 6 个月约 $10K MRR；第 8 个月 $10K MRR（两处口径）；第 10 个月 $100K ARR；2021 年 5 月（上线 12 个月）$25,134 MRR、2,000+ 用户、其中 1,650+ 付费。
- Source：StartupFounderStories 案例档案（基于 Mat 本人 IndieHackers AMA 与 Medium 长文，Mixpanel 数据佐证）；Mat 本人的付费指南落地页（e-kontur 托管）
- URL：https://startupfounderstories.com/stories/mat-de-sousa-widebundle-25k-mrr ; https://e-kontur.com/shopify-app-blueprint-landing.html
- Date：页面检索于 2026-06；原始 AMA 发布于 2021-12-02
- Excerpt："grew it to $25,134 MRR in under a year with no paid ads… First Customer 21 days… $1K MRR 3 months… $10K MRR 8 months… $100K ARR 10 months"；蓝图页时间线："$0 (2020 Launch) → $10K (6 mo.) → $25K (12 mo.) → $50K (3 yrs) — $50K Peak MRR, 7 team members full-time"
- Confidence：高（$25,134 为创始人本人两篇文章自报）

**证据 A2-3**
- Claim：后续轨迹：2022 年 Starter Story 报道时超 $40K/月、5 名全职员工；2023 年 5 月报道 $40K MRR；约 2023 年（3 年）达峰值 $50K MRR、团队 7 人；2025 年第三方复盘称 $55K MRR（非创始人确认）。
- Source：Starter Story；Indie Hackers "This week in Micro SaaS" 通讯；e-kontur 蓝图页；GrowthPartners
- URL：https://www.starterstory.com/stories/widebundle ; https://www.indiehackers.com/post/this-week-in-micro-saas-2300-sales-in-the-first-week-295aa941f3 ; https://e-kontur.com/shopify-app-blueprint-landing.html ; https://growthpartners.online/stories/how-widebundle-went-from-0-to-55k-mrr-in-just-3-years
- Date：2022 / 2023-05-15 / 2026 检索 / 2025-11-14
- Excerpt："Today, WideBundle makes over $40,000 monthly, with 5 people working full-time."；"WideBundle a Shopify app, reaches $40K MRR"
- Confidence：$40K/$50K 高；$55K 中（第三方未获创始人确认）

## A3. 收入数据（数字 + 来源 + 时间点）

| 时间点 | 数字 | 来源 |
|---|---|---|
| 2020-05 | $0（上线） | 本人蓝图页[^7^] |
| 2020-11（6 个月） | ~$10K MRR | 本人蓝图页[^7^] |
| 2021-05（12 个月） | $25,134 MRR；2,000+ 用户、1,650+ 付费 | 本人 IH AMA + Medium[^3^][^4^] |
| 2021-12 | $25K MRR（IH 专访确认） | Indie Hackers[^2^] |
| 2022 | $40K+/月，5 名全职 | Starter Story[^1^] |
| 2023-05 | $40K MRR | IH 通讯[^5^] |
| 2023-2024（约 3 年） | 峰值 $50K MRR，7 人团队 | 本人蓝图页[^7^] |
| 2025 | $55K MRR（第三方口径） | GrowthPartners[^6^] |

补充关键运营数字（2021 年自述）：免费试用→付费转化率从 9% 优化到 40%；流失率约 30%（本人承认从未真正解决）；定价仅从 $12.99 调到 $14.99，单一价格档是本人承认的最大失误之一。[^2^][^4^]

## A4. 产品现状（2026 年）

**证据 A4-1**
- Claim：截至 2026 年 WideBundle 仍在运营且为品类头部：Shopify App Store 评分 4.8/5（约 279-305 条评论，91% 五星），定价三档 $14.99/$19.99/$24.99 每月；发行主体为 The Wide Company（巴黎）；已扩展出第二款 App WideReview（评论工具），并创办巴黎年度 Shopify 开发者大会 The Wide Event（200+ 与会者）、4,500 人免费社区、2,400 人邮件列表。2024 年其 App 为用户商家创造超 6,000 万欧元销售额。
- Source：speed-ecom 2026 测评；Shopify App Store 评论页；EcomPreneurs 播客；e-kontur 蓝图页
- URL：https://speed-ecom.eu/blog/10-meilleures-applications-bundles-shopify/ ; https://apps.shopify.com/widebundle/reviews ; https://e-kontur.com/shopify-app-blueprint-landing.html
- Date：2026-05-06 / 2025-10 检索 / 2026 检索
- Excerpt："WideBundle… Note App Store 4.8 sur 5… Tarifs 14,99 / 19,99 / 24,99 dollars par mois"；"Ces applications… ont permis à leurs utilisateurs de générer plus de 60 millions d'euros en 2024."
- Context：未发现出售记录；Mat 转向"帮助其他 Shopify App 创始人"（付费书面蓝图为其首次售卖的知识产品）。注意 Shopify 评论页有一条 2024-09 的 1 星差评（大商家称故障致其日损 £30K），Mat 本人逐条回复——可作"评论运营"细节。
- Confidence：高

## A5. 日常作息与开发节奏

**证据 A5-1**
- Claim：早期为单人节奏：14 天出 MVP，"只做商家最关心的 bundle widget"；随后雇佣 5-7 名全职完成委托化，本人重心转向增长、内容与社区（X 上 3 万粉丝，持续 build in public）。本人复盘："管理两个 App 很难，我应该等 WideBundle 完全委托出去再做第二个。"
- Source：Starter Story；Indie Hackers；GrowthPartners；e-kontur
- URL：同 A1/A2
- Date：2021-2022
- Excerpt："If I had to start over… I would have also waited to launch our second app. I didn't fully delegate WideBundle, and managing 2 apps is hard."
- Confidence：中高（作息细节少，以节奏与管理决策为主）

## A6. 技术栈与工具链、基础设施成本

**证据 A6-1**
- Claim：公开确认的工具链：Shopify API/App 平台、Mixpanel（全漏斗数据追踪，用于把试用→付费转化从 9% 提升到 40%）、Facebook Groups（冷启动获客）；自研内部工具用于远程测试商家店铺、快速定位 bug（"支持是第一功能"的基础设施）。具体服务器栈与基础设施成本未公开。
- Source：Indie Hackers 专访；StartupFounderStories（Tech Stack: Shopify / Mixpanel / Facebook Groups）
- URL：https://www.indiehackers.com/post/how-we-validated-grew-a-shopify-app-to-25k-mo-in-less-than-a-year-159f82699c ; https://startupfounderstories.com/stories/mat-de-sousa-widebundle-25k-mrr
- Date：2021-12-10 / 2026-06-26
- Excerpt："we also created internal tools that allow us to test Shopify stores to help us find issues quickly when a user had a bug."
- Confidence：高（已公开部分）；基础设施成本：未找到（明确标注缺口）

## A7. 获客与增长（平台流量利用）

**证据 A7-1**
- Claim：增长飞轮 = Facebook 社群验证 → 老用户集中留评冲上 App Store 关键词首页 → Shopify 首页推荐 → 法国 KOL/网红合作放大 → 口碑。核心关键词曾排到第 4 位。从未投付费广告。把客服支持当作"第一功能"并引导用户在评论中提及支持——因为 Shopify 会奖励高支持质量的 App 以更多曝光。
- Source：Indie Hackers 专访/AMA；StartupFounderStories；GrowthPartners
- URL：同上
- Date：2021-12
- Excerpt："our support is our first feature… Shopify wants developers who offer incredible support with the app, and they'll be happy to put you in front of more people if you have great support. That's the reason why we even asked our users to talk about the support in their reviews."
- Confidence：高

## A8. 变现路径（定价、平台抽成、收入构成、利润率）

**证据 A8-1**
- Claim：纯订阅制（Shopify Billing），单一价格档 $12.99→$14.99/月（2021），2026 年演进为三档 $14.99/$19.99/$24.99；免费试用转化。Shopify 抽成政策：2026 年现行规则为每年前 $1,000,000 毛收入抽 0%（2025-01-01 起由"年度重置"改为"终身累计"），超出部分 15%，另加 2.9% 支付处理费；Partner 注册一次性 $19，无上架费/月费。低于 $1M 的开发者实际保留约 97%。
- Source：Mat 自述（定价）；metrichq 与 weekonelabs 对 shopify.dev 官方文档的解读；adsx 统计
- URL：https://metrichq.app/blog/shopify-app-revenue-share-fees-explained ; https://weekonelabs.com/blog/shopify-app-revenue-benchmarks-2026 ; https://www.adsx.com/blog/shopify-app-store-statistics-2026
- Date：2026-08-17 / 2026-07-03 / 2026-07-21
- Excerpt："Since January 1, 2025, Shopify takes 0% of your first $1,000,000 USD in gross app revenue each year [lifetime], then 15% above that, and every charge runs through a 2.9% processing fee… a one-time $19 to register as a Partner."
- Context：对比 Apple/Google 的 15-30%，Shopify 是全行业最慷慨的分成；但绕过 Shopify Billing 用 Stripe 收款违反条款、有被下架风险（weekonelabs 提到有高收入 App 因此被警告）。
- Confidence：高

## A9. 决策与关键转折、重大危机

**证据 A9-1**
- Claim：关键决策：① 第四次创业选择"商家已理解的老问题"（提客单价）而非新奇需求；② 先服务法国市场形成密度再全球化；③ 把支持做成护城河。本人承认的失误：定价单一档 + 30% 流失率长期未解决。结构性风险：Shopify 2023 年推出官方免费 Shopify Bundles 应用，直接切入 WideBundle 所在品类（平台亲自下场）。
- Source：Indie Hackers；StartupFounderStories；FuturMedia / SmartSMS 对 Shopify 原生 Bundles 的梳理
- URL：https://futurmedia.co.uk/blog/bundle-products-in-shopify ; https://smartsmssolutions.com/resources/blog/business/shopify-bundles-app-vs-custom
- Date：2026-04-02 / 2025-10-17
- Excerpt："Shopify introduced native bundling functionality in 2023, providing merchants with a built-in solution for creating bundle products… without third-party apps."
- Context：WideBundle 存活并增长的原因：原生 Bundles 只覆盖简单固定捆绑，缺少 quantity break/量级折扣等高级玩法——"平台内置功能吃掉的是低端，留下的是深度"。
- Confidence：高

## A10. 效率密码

- 14 天 MVP 法则：只做用户最关心的单一 widget，其他一律后补。[^2^][^6^]
- "支持即第一功能"：用极致客服换评论、换排名、换平台推荐位——零广告预算的杠杆。[^2^]
- 数据驱动激活：Mixpanel 追踪全漏斗，把试用转化从 9% 做到 40%，收入增长主要靠激活优化而非拉新。[^2^]
- 自研诊断工具降低客服成本；后期通过招聘 5-7 人完成委托化。[^2^][^7^]
- Build in public：3 万 X 粉丝、年度大会 The Wide Event 形成生态位品牌。[^7^]

## A11. 2026 年可行性素材（Shopify 侧）

- 生态规模：App Store 公开应用约 21,500-24,000 个（2026 年中，第三方爬虫口径不一）；每月新增 500-800 个（2026 年 5 月单月新增 2,713、下架 56）；约 65% 含免费方案；平均评分 4.65；23.8% 新上架 App 提及 AI。开发者累计收入超 $10 亿（Shopify 营销口径）；80%+ 商家使用至少一个第三方 App，平均每家店铺装 6 个，全平台安装量超 2,500 万次。[^12^][^13^][^14^]
- 收入分布极端幂律：Top 1% 的 App 年入 $1M+，Top 10% 超 $10 万 ARR，**中位数 App 月入不足 $1,000**。[^13^]
- 排名机制（2026 年可操作认知）：搜索贡献约 60% 安装量；关键词匹配优先，其次安装速率、近 90 天评论新鲜度、留存/卸载率；"Built for Shopify"徽章（约 1,500 个 App 持有，占 7%）平均带来 14 天内 49% 安装量提升。[^15^][^16^]
- 抽成：前 $1M 终身毛收入 0%、以上 15%、+2.9% 处理费（详见 A8）。[^10^][^11^]

---

# 第二部分：案例B —— Jordan O'Connor / Closet Tools → Resellbot（Chrome 扩展）

## B1. 背景故事与入行路径

**证据 B1-1**
- Claim：Jordan O'Connor 美国电气工程师（做机器人/激光相关的 C 语言开发），毕业背约 $20 万美元学生贷款（夫妻合计债务占单收入一半以上，每月收支相抵），妻子全职带娃（后有三个孩子）。前三年（约 2015-2018）零收入学技能：web 开发、SEO（刷爆信用卡花 $2,500 买 ViperChill 的 SEO 课）、免费帮人做网站/SEO 练手。因妻子在 Poshmark 卖二手衣、每天花大量时间手动"分享"商品，他写了 30 行 JavaScript 做成浏览器书签脚本（bookmarklet）送给妻子及其闺蜜。
- Source：Indie Hackers Podcast #187（Courtland Allen 主持）；Jordan 本人博客 jdnoc.com
- URL：https://www.indiehackers.com/podcast/187-jordan-oconnor-of-closet-tools ; https://jdnoc.com/something-from-nothing/
- Date：播客 2020-12 前后；博客 2018-03-16
- Excerpt："I did know how to code, but I didn't know anything about web development… my debt combined with her debt was taking up over half of our income… I wrote like a 30-line script and I made a bookmarklet… I made it for my wife and all of her friends who sold on Poshmark."
- Confidence：高（播客 transcript 一手原文）

**证据 B1-2**
- Claim：把脚本写成博客文章后被 Google 收录，开始有陌生人每周发邮件询问；妻子一年内在 Poshmark 积累 2 万粉丝成为 Poshmark Ambassador——需求信号来自真实使用。
- Source：jdnoc.com
- URL：https://jdnoc.com/something-from-nothing/
- Date：2018-03-16
- Excerpt："My wife was able to amass 20000 followers in a year and become a Poshmark Ambassador… I started getting emails from random people that wanted more info on this 'share button'."
- Confidence：高

## B2. 产品 0→1 时间线（精确到月）

| 时间 | 里程碑 | 来源 |
|---|---|---|
| ~2015-2017 | 三年零收入学技能（web/SEO/文案），帮人免费做项目 | IH Podcast[^18^] |
| 2017 前后 | 给妻子写 30 行 bookmarklet；写博客被 Google 收录 | jdnoc[^19^] |
| 2018-02 | 在 r/Poshmark 发免费脚本帖，一两天内获 200 个邮箱注册（另一处口径 50 个 beta 测试者） | IH Podcast[^18^] / jdnoc[^19^] |
| 2018-03 | 一个月内做出前端 + 自学 Stripe 集成，正式发布 Closet Assistant（Closet Tools 首个产品）：上线即 10 个付费客户 | IH Podcast[^18^] |
| 2018-03-16 | $140/月：6 个付费客户 + 9 个试用中 | jdnoc[^19^] |
| 2018-2020 | 缓慢增长；被 r/Poshmark 版主以"自我推广"为由踢出，转向 SEO + 口碑 | IH Podcast[^18^] |
| 2020-05 | $18K MRR | IH Podcast[^18^] |
| 2020-06~08 | 休陪产假 + 搬家，整个夏天没写代码，只做每天半小时邮件客服，**收入反而翻倍** | IH Podcast[^18^] |
| 2020-12（播客录制） | **$38K/月（≈$45 万/年）**，1,500 名客户，零员工；IH 产品库按"Stripe 验证收入 + 单人 + 无员工"排序第一名 | IH Podcast[^18^] |
| 2021-08 前后 | 峰值 $41K MRR | Listen Up IH[^20^] |
| 2022-04 | 回落至 $28K MRR（Poshmark 官方上线自动化功能 + 大量山寨竞品） | Listen Up IH[^20^] |
| 2025-07-30 | Closet Tools 更名 Resellbot，从浏览器扩展迁移为云端平台 | resellbot.com[^21^] |

## B3. 收入数据（数字 + 来源 + 时间点）

**证据 B3-1**
- Claim：$38K/月（2020-12，播客原话）；峰值 $41K MRR（2021）；$28K MRR（2022-04）；定价 $30/月从 2018 年上线起从未变过；2026 年 Resellbot 定价 $30/月或 $300/年（含 3 个 closet，超出每个 +$10/月）。
- Source：IH Podcast #187；Listen Up IH；PalmFlow 竞品对比页
- URL：https://www.indiehackers.com/podcast/187-jordan-oconnor-of-closet-tools ; https://www.listenupih.com/closet-tools-key-lessons/ ; https://palmflow.app/compare/closet-tools
- Date：2020-12 / 2022-04-05 / 2026-04-19
- Excerpt："it's probably around 35K but it's up to like 38K a month now… that's $450,000 a year"；"its revenue has come down to $28K MRR from $41K MRR last year"；"$30/mo or $300/yr · up to 3 Poshmark closets"
- Confidence：高（$38K 为 Stripe 验证 + 播客原话；$41K→$28K 来自引用其 X 推文的二手复盘）

## B4. 产品现状（2026 年）

**证据 B4-1**
- Claim：2025-07-30 Closet Tools 正式更名 Resellbot 并云端化：不再依赖浏览器扩展，7×24 云端执行、支持多 closet、移动端查看。仍在运营（14 天免费试用），但面临 CAPTCHA 循环、"share jail"（Poshmark 反自动化封号/限流）等摩擦；竞品（PalmFlow 等"卖家操作系统"型工具）正在升维竞争。
- Source：resellbot.com 官方公告；PalmFlow 对比页
- URL：https://resellbot.com/closet-tools-is-now-resellbot/ ; https://palmflow.app/compare/closet-tools
- Date：2025-07-30 / 2026-04-19
- Excerpt："This isn't just a rebrand. It's a fundamental shift in how the tool works - from a browser extension to a cloud-based platform… Chrome updates would sometimes break functionality; You had to keep your computer on and browser open."
- Confidence：高

## B5. 日常作息与开发节奏

**证据 B5-1**
- Claim：学艺三年期间每天清晨 4/5 点到 7 点工作（"唯一没人打扰、又不占用家庭时间的时间段"）；Closet Tools 上线后每天 1-2 小时深度工作 + 邮件客服。全职后工作块固定为早 6 点到中午 12 点，实际工作 3-4 小时，中午后全部给家庭。2020 年夏天休陪产假三个月几乎零工作，收入翻倍——"时间和产出脱钩"的极端样本。
- Source：IH Podcast #187
- URL：https://www.indiehackers.com/podcast/187-jordan-oconnor-of-closet-tools
- Date：2020-12
- Excerpt："it was mostly like 5:00 to 7:00 AM. For just about every day for about three years…"；"my block of time to work is basically 6:00 AM to noon… if I got up at nine, I'm still done by noon"；"I didn't do any code over the summer and the business doubled."
- Confidence：高

## B6. 技术栈与工具链、基础设施成本

**证据 B6-1**
- Claim：早期为纯客户端 Chrome 扩展/bookmarklet（"按顺序点页面元素"，两次点击分享一件商品）；后端用 Firebase（本人提到早期手动进 Firebase 给用户发密码重置邮件）；支付用 Stripe（花一个月自学 webhook）；邮件列表用 Buttondown。代码曾被竞品直接偷窃，之后做了混淆（obfuscation）。2025 年迁移为云端平台（自有机房/云服务器跑自动化）。基础设施成本具体数字未公开；单人运营意味着人力成本即全部主要成本。
- Source：IH Podcast #187；resellbot.com
- URL：同 B4/B5
- Date：2020-12 / 2025-07
- Excerpt："I've had people steal my code and that was before I did any scrubbing or obfuscating"；"I would have to go into my Firebase back-end then I have to send them a password reset email."
- Confidence：高（栈）；成本数字缺失（明确标注）

## B7. 获客与增长（平台流量利用）

**证据 B7-1**
- Claim：冷启动 = r/Poshmark 免费脚本帖（200 邮箱/一两天）→ 邮件列表三轮邮件（求反馈→公布计划→开售）→ 上线 10 个付费；被 Reddit 踢出后全靠 SEO + 口碑。SEO 是这个品类的天然渠道：用自动化在 Poshmark 社群里是禁忌话题，"人们不谈论它，但会隐身模式搜索它"（"I need a Poshmark bot"）。博客只有 13 篇文章但精准霸榜 "Poshmark automation" 等关键词。口碑来自高粘性卖家圈层。文档/教程同时承担 SEO、降低流失、减少客服三重职能。
- Source：IH Podcast #187；Listen Up IH
- URL：https://www.indiehackers.com/podcast/187-jordan-oconnor-of-closet-tools ; https://www.listenupih.com/closet-tools-key-lessons/
- Date：2020-12 / 2022-04-05
- Excerpt："people aren't talking about it, but they're searching for it. You're getting these people that are in incognito mode… That's why SEO has been so invaluable to the tool"；"Closet Tools' blog page has just 13 posts, but they rank for the most important keywords."
- Context：注意此案例不是典型"Chrome Web Store 排名获客"——Poshmark 无开放平台、无 API，自动化甚至被其社区准则明文禁止；Jordan 寄生的是 Poshmark 的"用户痛点"，分发靠 Google SEO 而非 CWS。
- Confidence：高

## B8. 变现路径（定价、支付、抽成、利润率）

**证据 B8-1**
- Claim：订阅制 $30/月（"一天一美元，帮你每天省几小时"的价值锚定），从未涨价；7 天免费试用；Stripe 收款（Chrome Web Store 自 2021-02-01 起彻底关闭内置支付，本就走自建支付的扩展不受影响）。本人承认定价严重偏低："有客户月销 $5 万的高端包，也只付我 $30/月；有人收购后第一周就能把价格翻三倍。"利润率接近纯软件极限——零员工、自动化客服，每天客服仅约半小时。
- Source：IH Podcast #187；Chrome 官方 CWS Payments 弃用文档
- URL：https://www.indiehackers.com/podcast/187-jordan-oconnor-of-closet-tools ; https://github.com/GoogleChrome/developer.chrome.com/blob/main/site/en/docs/webstore/cws-payments-deprecation/index.md
- Date：2020-12 / 官方文档 2020-09-20
- Excerpt："I charge 30 bucks a month. I've actually never changed the price… There's one of my customers I know makes 50K a month selling high-end purses on Poshmark but he uses my little $30 a month app."
- Context：Chrome 生态无平台抽成（CWS 支付 2020-09 停止新付费扩展、2021-02-01 停止一切扣款，此前 Google 抽 5%）；代价是支付/许可/续费全部自建或用 ExtensionPay 等第三方（Stripe 底层）。
- Confidence：高

## B9. 决策与关键转折、重大危机（平台风险教科书）

**证据 B9-1**
- Claim：① Poshmark 社区准则明文禁止自动化（账号可被终止），Jordan 判断"平台不会真打，因为自动化帮平台多赚钱"——但也因此产品"几乎不可出售"：多个收购方一听平台风险就退出。② 2021-2022 年 Poshmark 官方亲自上线自动化功能，叠加大量山寨竞品，收入从 $41K 跌至 $28K MRR（-32%）。③ 2025 年彻底"去扩展化"上云，摆脱 Chrome 更新导致的功能性损坏——从"浏览器寄生"转向"云端 SaaS"。
- Source：IH Podcast #187；Listen Up IH（引用 Jordan 的 X 推文 @jdnoc）；resellbot.com
- URL：同前
- Date：2020-12 / 2022-04 / 2025-07-30
- Excerpt："I've had several acquisition requests of Closet Tools… 'Oh, we're not going to get on the bad side of Poshmark. See you later.' So… it's almost unsellable."；"Poshmark has built some automations for users in the last year so it has affected my bottom line. Lots of competition now too!"（Jordan 本人推文）
- Confidence：高

## B10. 效率密码

- "懒惰驱动的自动化"哲学：凡是被重复问两次的问题，就做成文档/功能/内容，让客服邮件从源头消失（例：密码找回功能上线后相关邮件归零）。[^18^]
- 内容即资产：13 篇 SEO 文章 + 文档持续在后台获客、教育、降流失，一人公司没有"市场部"。[^18^][^20^]
- 时间盒：每天 1-2 小时深度工作 + 固定 6:00-12:00 工作块；增长期靠"上线前集中部署口碑型功能"然后放手。[^18^]
- 价值定价锚："一天一美元换每天几小时"——转化率因此极高，1,500 客户撑起 $38K+ MRR。[^18^]

## B11. 2026 年可行性素材（Chrome 扩展侧）

- 生态规模：Chrome Web Store 现有约 302,699 个扩展（另有 81,695 主题、14,693 应用），全部活跃扩展合计覆盖约 30 亿用户（chrome-stats，2026-08-21）；另有口径称"活跃扩展"从 2020 年 137,345 个降至 2024 年 111,933 个（Google 清理低质/违规扩展所致）。生产力类占约 55.5%。健康运营的扩展业务利润率约 70-85%。[^22^][^24^]
- 支付：CWS 无内置支付（2021-02 关停，此前抽 5%）；现标准为 Stripe 自建或 ExtensionPay（开源、跨浏览器、支持一次性+订阅+试用）。注册开发者账号一次性 $5。[^23^][^25^]
- Manifest V3 迁移影响：MV2 自 Chrome 138（2025 年中起）被禁用；MV3 以 service worker 取代后台页、declarativeNetRequest 上限 3 万条规则、禁止远程代码。广告拦截类受冲击最大——uBlock Origin（MV2 版）在 2025-03 起失效、2026-06 起连 workaround 也被移除（Chrome 150/151），开发者被迫重做 Lite 版。结论：2026 年入场只能 MV3 原生开发；MV3 同时提高了克隆/作恶门槛（全部 JS 须打包、不可远程加载），对合规独立开发者是护城河而非纯利空。[^26^][^24^][^27^]
- 收入对标：GMass（Gmail 邮件营销扩展）约 $130K MRR / 1 万订阅 / $8-20 每月；Closet Tools $30/月档；CSS Scan 一次性 $38 买断累计 $10 万+。Chrome 扩展独立开发者常见区间：副业档 $500-2,000 MRR；全职档 $5K-15K MRR；极少数长成为 SaaS 公司。[^24^][^28^]

---

# 第三部分：平台寄生风险——著名翻车案例库

**证据 C-1｜Checkout X 被 Shopify 亲手扼杀（Shopify 侧最重案例）**
- Claim：Ruslan Leteyski（保加利亚）自学编程，2018-09 上线 Shopify 单页结账应用 Checkout X，从未上架 App Store、零广告，靠"竞争对手查看源码发现线索"+"15-25% 永久返佣联盟"增长至 6,000 商家、峰值 €600K MRR（约 $8M ARR，客单 €100/月）。2019 春 Shopify ToS 更新禁止替代结账（曾口头获准继续）；Shopify 高管 Harley 亲自电话锁死新安装；2020 年底 ToS 堵死私有 App  loophole；2023 年团队解散。Shopify 随后自己上线单页结账与售后加购功能。创始人名言："A Shopify app is always a single-click away from being deleted from someone that works there."
- Source：创始人博客 post-mortem；Indie Hackers 2026 自述
- URL：https://blog.leteyski.com/p/bootstrapping-to-600k-mrr-and-getting ; https://www.indiehackers.com/post/tech/building-a-2m-arr-product-after-his-8m-arr-product-failed-overnight-UZm68xNgjDZBHH7Mvc54
- Date：2023-03-10 / 2026-08-13
- Excerpt："Shopify killed it - just because merchants were choosing our solution over theirs… Years of work vanished because someone else changed their mind."（其后续：micro-exit 了 Vanga AI，2024 年联合创立 Zipchat，2026 年近 $2M ARR——平台寄生者转型样本）
- Confidence：高

**证据 C-2｜Shopify 系统性"Sherlock"自有生态**
- Claim：Shopify 近年以免费原生产品进入多个第三方品类：Inbox（客服，上线一年即成 Plus 商家第四大客服 App）、Email、Collabs（联盟营销，2022-08 上线后迅速占 8% 份额、为第二名两倍）、2023 年原生 Bundles、2023-09 将官方 Product Reviews 应用下架并于 2024-05-06 彻底停用并删除数据（评论类目让给 Judge.me/Loox 等付费第三方——少见的"平台退场"案例，方向相反但同样展示平台单方决策的杀伤力）。Leteyski 原话："Shopify has slowly cannibalized its own ecosystem by either developing free app alternatives or obsoleting apps by making a feature part of its core product."
- Source：IcyTales 深度报道；Shopify Community 官方答复；Ilana Davis 博客
- URL：https://icytales.com/inside-shopifys-app-store-ranking-system-first-party-advantages-and-the-independent-developers-caught-in-the-middle/ ; https://community.shopify.com/t/whats-going-on-with-the-product-reviews-app-its-no-longer-available/249489 ; https://www.ilanadavis.com/blogs/articles/shopify-product-reviews-app-unavailable-may-2024
- Date：2026-04-03 / 2023-09 / 2024-02-06
- Confidence：高

**证据 C-3｜The Great Suspender 恶意转手事件（Chrome 侧最重案例）**
- Claim：200 万+用户的标签页休眠扩展 The Great Suspender 原作者于 2020 年中将其卖给不明买家，新维护者静默推送含远程代码执行/广告欺诈的恶意更新；2021-02-04 Google 将其从 CWS 移除并**远程强制禁用所有用户设备上的扩展**，大量用户丢失被挂起的标签页。后由社区 fork（Marvellous Suspender 等，约 9 万用户）延续。同时期 Chrome 推出内置 Memory Saver 功能，官方功能直接蚕食该品类。
- Source：The Hacker News；Lifehacker
- URL：https://thehackernews.com/2021/02/warning-hugely-popular-great-suspender.html ; https://lifehacker.com/ditch-the-great-suspender-before-it-becomes-a-security-1845989664
- Date：2021-02-06
- Excerpt："Google… removed The Great Suspender… took the unusual step of deactivating it from users' computers… more than two million installs."
- Confidence：高

**证据 C-4｜Closet Tools 自身（见 B9）**：平台（Poshmark）上线官方自动化 → $41K→$28K MRR；准则禁止自动化导致产品"不可出售"。

---

# 第四部分：次要参照案例 —— Jason Savard / Checker Plus for Gmail

**证据 D-1**
- Claim：蒙特利尔自学开发者 Jason Savard 约 2012 年起开发 Checker Plus for Gmail，扩展家族（Gmail/Calendar/Drive）合计近 200 万用户（2022 口径）；Calendar 单个扩展 2026 年 30 万+用户、4.5 星、MV3、每月多次更新。变现为"任意金额捐款解锁增值功能"（无固定档位），本人身兼客服。**从未公开收入数字**——这是未选其为主案例的原因；但它是"超长生命周期单人扩展"（14 年+）与"捐款模式"的代表样本。
- Source：kiko.io 博客；TimeHopper 2026 测评；PCMag
- URL：https://kiko.io/post/Checker-Plus-Gmail-in-better/ ; https://timehopperapp.com/blog/best-google-calendar-extensions-2026 ; https://www.pcmag.com/news/the-26-best-chrome-extensions-for-gmail
- Date：2022-02-12 / 2026-03-18 / 2019-04-03
- Excerpt："Montreal-based software developer Jason Savard… ten years ago… almost 2 million users"；"Bonus features unlock with any contribution (one-time or yearly — no fixed tiers)… Single developer (high dedication, but a bus-factor-of-one risk)"
- Confidence：中高（用户量为第三方口径）

---

# 第五部分：写作素材摘要（按 11 组整理）

## 案例A：Mat De Sousa × WideBundle（Shopify App）

**1. 背景与入行**：法国工程师学生，中学时给网游 Dofus 写外挂赚到第一桶"自动化"经验；2017 年在达索航空实习时意识到"大公司里我随时可被替代"，决心创业。同年自学 Shopify 开发，连做三个 App 全部失败。2020 年毕业前，他说服学校用"创办公司"替代六个月实习——WideBundle 由此诞生，毕业时已盈利。

**2. 0→1 时间线**：2020 年 5 月上线（MVP 只做 14 天，仅有 bundle widget）；第 21 天首个付费客户；3 个月 $1K MRR；6 个月 $10K MRR；10 个月 $100K ARR；12 个月（2021-05）$25,134 MRR、2,000 用户；2022 年 $40K+/月、5 名全职；约 3 年达峰值 $50K MRR、7 人团队；2025 年第三方口径 $55K MRR。

**3. 收入数据**：核心锚点是创始人双渠道自报的 $25,134 MRR（2021-05，IH AMA + Medium，Mixpanel 佐证）；$40K（Starter Story 2022）；峰值 $50K（本人 2025 蓝图页）。试用转付费 9%→40%，流失率约 30%。

**4. 现状（2026）**：运营中且为品类头部：App Store 4.8 分、约 300 条评论、91% 五星；三档定价 $14.99/$19.99/$24.99；第二产品 WideReview 已上架；办起巴黎年度大会 The Wide Event；2024 年其 App 帮商家创造 €60M+ 销售额。未出售，转型"Shopify App 布道者"并首次售卖付费书面蓝图。

**5. 作息与节奏**：早期单人极限速度（14 天 MVP），后期靠雇佣 5-7 人委托化；本人承认"第二个 App 上线太早、没有先彻底委托 WideBundle"是管理教训。

**6. 技术栈与成本**：Shopify API + Mixpanel 数据栈 + 自研商家店铺诊断工具；服务器栈与基础设施成本未公开（研究缺口）。

**7. 获客与增长**：教科书级平台流量套利——Facebook 社群验证需求 → 让前 100 个老用户集中留评 → 冲上 App Store 关键词首页 → 被 Shopify 推荐到首页 → 法国网红放大 → 口碑。零付费广告。核心洞察："支持是第一功能"，并引导用户在评论里夸客服，因为 Shopify 会给高支持质量的 App 更多曝光。

**8. 变现路径**：纯订阅（$12.99→$14.99 单档是最大失误，后改三档）；Shopify 2025-01-01 起前 $1M 终身毛收入抽 0%、以上 15%、+2.9% 处理费——低于百万美元的开发者保留约 97 美分/美元，全球最慷慨的应用商店分成。

**9. 转折与危机**：最大的结构性威胁是 Shopify 2023 年亲自推出免费原生 Bundles 应用；WideBundle 靠 quantity break 等深度功能存活——"平台吃掉低端，深度留给第三方"。2024 年曾有大商家 1 星差评称故障致其日损 £30K，Mat 本人逐条公开回复。

**10. 效率密码**：14 天 MVP；只做用户已经理解的老问题；客服即营销；Mixpanel 驱动的激活优化（收入增长靠转化率而非拉新）；build in public 积累 3 万 X 粉丝形成生态品牌。

**11. 2026 可行性**：App Store 约 2.15-2.4 万个应用、每月新增 500-800 个、中位数 App 月入不足 $1,000、Top 10% 才超 $10 万 ARR——幂律极其残酷；但 0%/15% 抽成、搜索贡献 60% 安装、BFS 徽章 +49% 安装的机制仍给新人留了明确打法；新上架 App 中 23.8% 提及 AI，入场成本在被 AI 拉低。

## 案例B：Jordan O'Connor × Closet Tools → Resellbot（Chrome 扩展）

**1. 背景与入行**：美国电气工程师，夫妻合计 $20 万+ 学生贷款吞掉一半收入，妻子全职带娃。前三年零收入自学 web 开发、SEO、文案，刷爆信用卡买 $2,500 的 SEO 课，靠免费帮人做项目练手。妻子在 Poshmark 卖二手衣每天手动"分享"商品数小时，他写了 30 行 JS 书签脚本送她和闺蜜——需求来自自家餐桌。

**2. 0→1 时间线**：2018-02 在 r/Poshmark 发免费脚本帖，一两天收 200 个邮箱；2018-03 一个月做出产品+自学 Stripe 集成，上线即 10 个付费、当月 $140；之后两年慢增长（还被 Reddit 踢出社群）；2020-05 达 $18K MRR；2020 夏天休陪产假三个月不写代码、收入翻倍；2020-12 达 $38K/月、1,500 客户、零员工，登顶 Indie Hackers"Stripe 验证收入的单人无员工"榜；2021 峰值 $41K；2022 跌至 $28K；2025-07 更名 Resellbot 并云端化。

**3. 收入数据**：$38K/月（2020-12 播客原话，Stripe 验证）；$41K 峰值（2021）；$28K（2022-04）；$30/月定价八年未涨。

**4. 现状（2026）**：Resellbot 运营中，$30/月或 $300/年，云端 7×24 执行、多 closet 管理；但面临 CAPTCHA、Poshmark"share jail"封号风险与"卖家操作系统"型竞品升维竞争。

**5. 作息与节奏**：学艺期每天清晨 5-7 点；全职后工作块固定 6:00-12:00、实际工作 3-4 小时，中午后全部给三个孩子的家庭；陪产假期间"每天半小时邮件客服，业务自己翻倍"——时间产出完全脱钩的极端样本。

**6. 技术栈与成本**：纯客户端 Chrome 扩展（按序点击页面元素）+ Firebase 后端 + Stripe 支付 + Buttondown 邮件；代码曾被竞品偷走后做了混淆；2025 年迁移云端。基础设施成本未公开（缺口），但零员工结构意味着人力即全部主要成本。

**7. 获客与增长**：罕见的"不靠平台商店获客"的扩展案例——Poshmark 没有开放平台且禁止自动化，于是靠 Reddit 冷启动 + 精准 SEO（"人们不敢谈论机器人，但会隐身搜索它"；博客仅 13 篇文章霸榜品类关键词）+ 卖家圈层口碑。文档同时承担 SEO、教育降流失、替代客服三重职能。

**8. 变现路径**：订阅 $30/月（"一天一美元买你几小时"），Stripe 自建支付——Chrome Web Store 2021-02 起根本没有内置支付（此前 Google 抽 5%），扩展开发者被迫自建反而拥有了完整定价权与零平台抽成；本人坦承定价偏低（有月销 $5 万的客户也只付 $30）。

**9. 转折与危机**：平台风险教科书。Poshmark 准则明文禁止自动化 → 收购方尽调后全部撤退，"几乎不可出售"；2021-22 年 Poshmark 亲自上线官方自动化 + 山寨围攻 → 收入 -32%；2025 年"去扩展化"上云，摆脱 Chrome 更新对业务的破坏——从浏览器寄生进化为云 SaaS。

**10. 效率密码**："懒惰驱动自动化"——被问两次的问题就做成内容或功能；密码找回功能上线后相关客服邮件归零；内容即后台获客机器；固定时间盒 + 增长期集中部署口碑型功能后放手。

**11. 2026 可行性**：CWS 约 30.3 万个扩展、覆盖 30 亿用户，但活跃扩展数从 2020 年 13.7 万降至 11.2 万（Google 大清洗）；无平台抽成、支付全自建（ExtensionPay/Stripe 是标准件）；MV3 是唯一入场券（MV2 已于 2025 年 Chrome 138 起禁用，uBlock Origin 这类头部都被迫重做）， MV3 反而抬高了作恶与克隆门槛；收入现实区间：副业 $500-2K MRR、全职 $5K-15K MRR，头部如 GMass $130K MRR。最大风险始终是平台本身：Checkout X（€600K MRR 被 Shopify 锁死）、The Great Suspender（200 万用户被 Google 远程禁用）、Poshmark 官方自动化（Closet Tools 收入 -32%）、Shopify 原生 Bundles/Inbox/Collabs 蚕食第三方——平台寄生型的终局问题不是"能不能成"，而是"平台何时亲自下场"。

---

# 来源列表

[^1^]: Starter Story — "Failing Thrice, Then Building A $40K/Month Shopify App By The Age Of 25"（约 2022）https://www.starterstory.com/stories/widebundle
[^2^]: Indie Hackers — "How We Validated & Grew a Shopify App to $25k/mo in Less Than a Year"（2021-12-10）https://www.indiehackers.com/post/how-we-validated-grew-a-shopify-app-to-25k-mo-in-less-than-a-year-159f82699c
[^3^]: Indie Hackers — Mat De Sousa AMA "I bootstrapped a Shopify app to $25K MRR in less than a year!"（2021-12-02）https://www.indiehackers.com/post/i-bootstrapped-a-shopify-app-to-25k-mrr-in-less-than-a-year-ama-89f3d4c471
[^4^]: StartupFounderStories — Mat De Sousa / WideBundle 案例档案（检索 2026-06-26）https://startupfounderstories.com/stories/mat-de-sousa-widebundle-25k-mrr
[^5^]: Indie Hackers — "This week in Micro SaaS"（2023-05-15，WideBundle $40K MRR）https://www.indiehackers.com/post/this-week-in-micro-saas-2300-sales-in-the-first-week-295aa941f3
[^6^]: GrowthPartners — "How WideBundle Went from $0 to $55K MRR in Just 3 Years"（2025-11-14）https://growthpartners.online/stories/how-widebundle-went-from-0-to-55k-mrr-in-just-3-years
[^7^]: Mat De Sousa 付费蓝图落地页（本人数据：$0→$10K→$25K→$50K、7 人团队、WideReview、The Wide Event）https://e-kontur.com/shopify-app-blueprint-landing.html
[^8^]: EcomPreneurs 播客 Ép.27 — Mat De Sousa（2025-01；2024 年用户 GMV €60M+、Dofus 起源）https://creators.spotify.com/pod/profile/ecompreneurs
[^9^]: speed-ecom — "Bundle Shopify : les 10 meilleures apps en 2026"（2026-05-06；WideBundle 4.8 分、三档定价）https://speed-ecom.eu/blog/10-meilleures-applications-bundles-shopify/
[^10^]: MetricHQ — "Shopify app revenue share and fees in 2026"（2026-08-17）https://metrichq.app/blog/shopify-app-revenue-share-fees-explained
[^11^]: WeekOneLabs — "Shopify App Revenue Benchmarks 2026"（2026-07-03；中位数 App <$1K MRR、Top 1% >$1M ARR）https://weekonelabs.com/blog/shopify-app-revenue-benchmarks-2026
[^12^]: Adsx — "Shopify App Store Statistics 2026"（2026-07-21；21.5K-24K apps、65% 免费档、AI 占比）https://www.adsx.com/blog/shopify-app-store-statistics-2026
[^13^]: 同 [^11^]
[^14^]: BacktoFrontShow — "Shopify Statistics 2026"（2026-05-12；开发者累计收入 $1B+、80% 商家用 App、平均 6 个、2500 万安装）https://backtofrontshow.com/shopify-statistics/
[^15^]: AppJubilee — "Shopify App Store Visibility Guide"（2026-06-16；搜索=60% 安装、BFS +49%/14 天、90 天评论新鲜度）https://blog.appjubilee.io/ultimate-guide-shopify-app-store-visibility/
[^16^]: Gaintage — "How to Rank in the Shopify App Store: 2026"（2026-04-30）https://gaintage.com/blog/shopify-app-store-ranking
[^17^]: Shopify 官方 — Built for Shopify 文档 https://shopify.dev/docs/apps/launch/built-for-shopify
[^18^]: Indie Hackers Podcast #187 — "Building a $38k/Month SaaS Business as a Solo Founder with Jordan O'Connor of Closet Tools"（2020-12）https://www.indiehackers.com/podcast/187-jordan-oconnor-of-closet-tools
[^19^]: Jordan O'Connor 博客 — "From Nothing to $140/Month in 2 Months"（2018-03-16）https://jdnoc.com/something-from-nothing/
[^20^]: Listen Up IH — "Key Lessons from the success of Closet Tools"（2022-04-05；$41K→$28K MRR、Jordan 推文）https://www.listenupih.com/closet-tools-key-lessons/
[^21^]: Resellbot 官方 — "Closet Tools is Now Resellbot"（2025-07-30）https://resellbot.com/closet-tools-is-now-resellbot/
[^22^]: PalmFlow — "PalmFlow vs Closet Tools / Resellbot"（2026-04-19；Resellbot 定价与风险）https://palmflow.app/compare/closet-tools
[^23^]: Google 官方 — "Chrome Web Store payments deprecation"（2020-09-20；2021-02-01 停止扣款）https://github.com/GoogleChrome/developer.chrome.com/blob/main/site/en/docs/webstore/cws-payments-deprecation/index.md
[^24^]: CodeTalentHub — "Top 10 Simple Coding Projects for Passive Income"（2025/2026 更新；MV2 于 Chrome 138 移除、活跃扩展 111,933、利润率 70-85%、GMass $130K MRR、CSS Scan $100K+）https://www.codetalenthub.io/simple-coding-projects-for-passive-income/
[^25^]: ExtensionPay — "ExtensionPay is the Chrome Web Store Payments Replacement"（2025-01-27；原 CWS 支付抽 5%）https://extensionpay.com/articles/extensionpay-is-the-chrome-web-store-payments-replacement
[^26^]: PCWorld — "The last lifeline for uBlock Origin in Chrome is almost gone"（2026-06-09）https://www.pcworld.com/article/3160794/the-last-lifeline-for-ublock-origin-in-chrome-is-almost-gone-for-good.html
[^27^]: chrome-stats — Chrome Web Store 统计（2026-08-21；302,699 扩展、3B 用户）https://chrome-stats.com/chrome/stats
[^28^]: ChromeGoldmine — "Profitable Chrome Extension Niches for Indie Makers (2026)"（2026-03-27；收入分层、Closet Tools $42K MRR 引用）https://chromegoldmine.com/blog/profitable-chrome-extension-niches/
[^29^]: Ruslan Leteyski 博客 — "Bootstrapping to €600k MRR and getting killed by Shopify"（2023-03-10）https://blog.leteyski.com/p/bootstrapping-to-600k-mrr-and-getting
[^30^]: Indie Hackers — Ruslan Leteyski "Building a $2M ARR product after his $8M ARR product failed overnight"（2026-08-13）https://www.indiehackers.com/post/tech/building-a-2m-arr-product-after-his-8m-arr-product-failed-overnight-UZm68xNgjDZBHH7Mvc54
[^31^]: IcyTales — "Inside Shopify's App Store Ranking System…"（2026-04-03；Inbox/Collabs 份额、Sherlock 分析）https://icytales.com/inside-shopifys-app-store-ranking-system-first-party-advantages-and-the-independent-developers-caught-in-the-middle/
[^32^]: Shopify Community — Product Reviews 下架官方答复（2023-09）https://community.shopify.com/t/whats-going-on-with-the-product-reviews-app-its-no-longer-available/249489
[^33^]: Ilana Davis — "Shopify Product Reviews app unavailable May 2024"（2024-02-06；2024-05-06 数据删除）https://www.ilanadavis.com/blogs/articles/shopify-product-reviews-app-unavailable-may-2024
[^34^]: The Hacker News — "WARNING — Hugely Popular 'The Great Suspender' Chrome Extension Contains Malware"（2021-02-06）https://thehackernews.com/2021/02/warning-hugely-popular-great-suspender.html
[^35^]: FuturMedia — "Bundle Products in Shopify: Complete Guide (2026)"（2026-04-02；Shopify 2023 年原生 Bundles）https://futurmedia.co.uk/blog/bundle-products-in-shopify
[^36^]: kiko.io — "Checker Plus - Gmail in better..."（2022-02-12；Savard 扩展家族近 200 万用户）https://kiko.io/post/Checker-Plus-Gmail-in-better/
[^37^]: TimeHopper — "10 Best Google Calendar Chrome Extensions (2026)"（2026-03-18；Savard 捐款模式、单人风险）https://timehopperapp.com/blog/best-google-calendar-extensions-2026
[^38^]: Diffmode — Closet Tools 案例引用（2026-05-14；$300 MRR 起步口径，与播客存在出入，低置信）https://diffmode.app/grow/invoicing-saas/for-construction-subcontractors/

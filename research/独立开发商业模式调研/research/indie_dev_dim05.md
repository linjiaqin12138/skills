# 模式5：开源变现型（先免费后收费）—— 深度研究素材

**报告主题**：《独立开发者成功模式深度拆解》· 维度5
**研究日期**：2026-08-26
**深挖案例**：A = Vuetify（John Leider，Vue UI 框架）；B = NativePHP（Simon Hamp，用 PHP 构建桌面/移动应用）
**搜索轮次**：30+ 次独立检索（web_search × 5 批 ≥30 条查询 + 5 个一手页面全文阅读）
**证据格式**：Claim / Source / URL / Date / Excerpt / Context / Confidence

---

## 第一部分：案例A —— Vuetify（John Leider）

### A1. 背景故事与入行路径

**证据 1**
- Claim：John Leider 曾服役于美国陆军（2013年1月退役），之后考取 A+/Security+/Network+ 认证进入 IT 行业，早期是 PHP + jQuery + MySQL 的服务端开发者，2014 年底在学习 Laravel 时通过 Laracasts 视频接触到 Vue.js（当时还叫 Seed）并"立刻爱上"。
- Source：Starter Story 专访《I Built A $300K/Year Vue.js Component Library》（John Leider 第一人称自述）
- URL：https://www.starterstory.com/stories/i-built-a-300k-year-vue-js-component-library
- Date：原始采访 2021-08（页面最后更新 2025-12-10）
- Excerpt："I completed my term in the U.S. Army in January of 2013… I primarily used PHP, jQuery, and MySQL… Towards the end of 2014, I came across a library named Laravel… Jeffrey Way used an early alpha version of the Vue.js framework and I instantly fell in love with it."
- Context：独立开发者典型的"非 CS 科班 → 军队 → 自学 Web 开发"路径；他多次公开的照片为 2010 年伊拉克 FOB Falcon 服役照。
- Confidence：高（本人自述）

**证据 2**
- Claim：John 的起点是想练习 Vue 2 beta 的新特性，先做了一个基于 Materialize CSS 的组件库 vue-materials，随后决定从零实现 Material Design 规范，这就是 Vuetify。
- Source：同上（Starter Story）[^1^]；中文二手梳理见硬地骇客[^9^]
- URL：https://hardhacker.com/posts/vuetify
- Date：2023-07-29
- Excerpt："等到 Vue2 beta 发布时，John 希望尝试 Vue2 中的新特性和变化，于是他决定通过构建一个 material design 的组件库来进行这次尝试。"
- Context：动机是"学习新技术 + 自己项目可用"，而非商业计划。
- Confidence：高

### A2. 项目 0→1 时间线（精确到月）

**证据 3**（一手时间线组合）
- 2016-09：第一个 pre-alpha 版本发布（"The first pre-alpha version of Vuetify was published in September of 2016"）[^1^]
- 2016-12-14：第一个公开 alpha 版本以 MIT 协议发布到 GitHub（"On December 14th, 2016, after around 3 to 4 months of development, I released the first alpha version of Vuetify on GitHub under the MIT software license"）[^1^]
- 2017-10：宣布辞职，成为全职开源开发者（"In October of 2017, I announced that I was quitting my job to become a full-time Open Source developer"）[^1^]
- 2018-02：v1.0 正式发布（官方 Roadmap："v1.0 — Released: February 2018… After 18 months and Kael's sanity"）[^10^]
- 2019-02：v1.5（LTS）；2019-07：v2.0 (Arcadia)，核心完全重写（JS→TS、Stylus→Sass、升级到 Material Design 2）[^10^]
- 2022-11-01：v3.0 (Titan) 发布，基于 Vue 3 重写[^10^]
- 2024-01：Vuetify One（生态统一账号/订阅体系）上线；2024-03：Vuetify Bin + Playground；2024-04：Vuetify Snips（首个独立付费产品）[^4^]
- 2025-12：v4.0.0-alpha.0；2026-01：v4.0.0-beta.0[^11^]
- 2026-02-23：Vuetify 4 正式版；2026-06-02：v4.1.0；2026-07-22：Vuetify0（headless 逻辑层）1.0；2026-07-27：v3.13.0 为 v3 最后一个小版本，进入 LTS 至 2027-07[^10^][^11^]
- Source：Vuetify 官方 Roadmap / endoflife.date / 官方博客
- URL：https://vuetifyjs.com/vuetify/roadmap ；https://endoflife.date/vuetify ；https://vuetifyjs.com/en/blog/
- Date：Roadmap 持续更新；endoflife 最后更新 2026-08-14；博客更新至 2026-08-11
- Confidence：高（官方 + 第三方交叉验证）

### A3. 收入数据

**证据 4**（2021 年高峰期，本人披露）
- Claim：2021 年年收入约 $300K/年（Starter Story 标题口径）；结构拆解：Patreon + GitHub Sponsors 合计约 $6,500/月；Open Collective 约 $2,000+/月（供养核心团队）；Vuetify Store（付费主题/周边）从最初 <$100/月 增长到 $15,000–$20,000/月，成为"main revenue generator"；另有 Carbon 文档广告收入和咨询/直接支持收入。起步时捐赠仅 $500+/月。
- Source：Starter Story 专访[^1^] + Starter Story 案例库页[^2^]
- URL：https://www.starterstory.com/stories/i-built-a-300k-year-vue-js-component-library ；https://www.starterstory.com/ideas/vue-js-component-library/success-stories
- Date：2021-08（数据为 2021 年中）
- Excerpt："Over time, monetary support from Patreon and GitHub has grown to a combined total of around $6,500 a month… the store has slowly grown from making less than $100 most months to $15,000 to $20,000… the store has slowly become the main revenue generator for the business."
- Context：注意矛盾点——Starter Story 案例库另一处写"generating an average income of $6,500 per month"与标题 $300K/年（≈$25K/月）不一致；$6,500 只是赞助项，$300K/年是全渠道合计口径。2025-04 Starter Story 邮件摘要写 "$25k/month through a combination of digital products and through Patreon and Github supporters"[^3^]，与 $300K/年口径吻合。
- Confidence：中高（本人采访 + 媒体编辑口径混合，需在报告中注明口径差异）

**证据 5**（2024 年资金危机与反弹）
- Claim：2024 年 6 月 John 公开发帖称 Vuetify 陷入财务"drought"（枯竭期），他被迫外出求职，入职了一家全栈使用 Vue/Vuetify 的公司 Optikka 任 Senior Developer；同时把框架的创意与工程主导权移交给创始期成员 Kael（Watts-Deuchar）。2024 年 9 月官方《State of the Union》披露 Open Collective + GitHub Sponsors 两平台合计恢复到"over $8,000 per month"，靠的是 Abacus、Route4Me、Teamwork 等公司的大额赞助。
- Source：Vuetify 官方博客 State of the Union 2024[^4^]
- URL：https://vuetifyjs.com/zh-Hans/blog/state-of-the-union-2024/
- Date：2024-09-08
- Excerpt："A huge win has been the financial support through Open Collective and GitHub Sponsors, with over $8,000 per month on both platforms… those who followed my post earlier back in June will know that I've been seeking employment in lieu of the financial situation… I connected with a company named Optikka… I have taken a step back… pass creative and engineering control of the framework over to Kael."
- Context：这是开源赞助模式脆弱性的一手证据——即便头部项目也会出现赞助断粮、创始人被迫打工的周期。
- Confidence：高（官方博客）

**证据 6**（2025–2026 现状补充）
- Claim：GitHub 官方 2025 年 6 月文章将 Vuetify 列为受赞助开源项目样板："Sponsors (from $1 to $1,500/month) get logo placement and priority support, and the pool pays core maintainers and contract contributors each month."[^6^]
- Source：GitHub Blog《4 trends shaping open source funding》
- URL：https://github.blog/open-source/maintainers/4-trends-shaping-open-source-funding-and-what-they-mean-for-maintainers/
- Date：2025-06-03
- Confidence：高

**证据 7**（资金分配机制）
- Claim：Vuetify 明确区分资金流向——GitHub Sponsors/Patreon 归 John 与 Heather（夫妻）全职生活；Open Collective 透明基金用于支付核心团队成员费用；早期核心成员另有最高 $1,000/月的津贴，2021 年每月向团队回馈 >$2,500。
- Source：GitHub 仓库 README（各镜像一致）[^5^] + Starter Story[^1^]
- URL：https://github.com/sponsors/johnleider
- Excerpt："Funds donated through GitHub Sponsors and Patreon go directly to support John and Heather's full-time work on Vuetify. Funds donated via Open Collective are managed with transparent expenses…"
- Confidence：高

### A4. 项目现状（2026 年）

**证据 8**
- Claim：截至 2026 年 1 月，Vuetify 有 41,000+ GitHub stars、约 70 万/周 npm 下载（Snyk 2025-12 快照显示周下载 857K 次、41,014 stars、440 贡献者、4 名维护者）；官方 Store 页宣称"over 6 million downloads"。Vuetify 4 已正式发布（2026-02），v3 进入 LTS；团队 2025 年 12 月单月 522 commits 横跨 16 个仓库，为"2025 年最高产月份"；John 仍亲自撰写每月更新博客。
- Source：vue-pdf-viewer.dev 2026 盘点[^13^] / Snyk[^12^] / store.vuetifyjs.com[^8^] / 官方博客[^11^]
- URL：https://www.vue-pdf-viewer.dev/blog/best-vuejs-ui-libraries-for-2026-top-5-picks-bonus/ ；https://security.snyk.io/package/npm/vuetify ；https://store.vuetifyjs.com/pages/about-us ；https://vuetifyjs.com/en/blog/
- Date：2026-01-20 / 2025-12-14 / 持续 / 2026-08-11
- Excerpt："December was our most productive month of 2025 with 522 commits across 16 repositories."
- Confidence：高

**证据 9**（Vuetify One 与 Snips）
- Claim：Vuetify One 是 2024 年 1 月上线的生态统一账号/订阅服务（2025-12 已支持 Google OAuth、全线产品 PWA 化）；Vuetify Snips（2024-04）是首个独立付费产品，销售代码片段订阅（页面显示 $14–$36/user/month 档位），"All sales from Snips go directly back into Vuetify development"。另有 Vuetify Bin、Playground、Issues、Link、MCP server（AI 工具链，2025-10 支持 HTTP transport）等生态产品。
- Source：State of the Union 2024[^4^] / 官方博客[^11^] / Snips 站[^14^]
- URL：https://snips.vuetifyjs.com/marketing/page-sections/pricing-sections
- Confidence：高

### A5. 日常作息与开发节奏

**证据 10**
- Claim：开源维护者的日常被 John 概括为"developing new features, fixing bugs, finding other open source developers, and providing direct support to business and enterprise users"四件事并行；2024 年后他退居"higher-level tasks"，工程一线交给 Kael 与核心团队，团队采取 Open Collective 发票报销制（贡献者提交 invoice 领钱）。
- Source：Starter Story[^1^] / State of the Union 2024[^4^]
- Excerpt："My time was split between developing new features, fixing bugs, finding other open source developers, and providing direct support to business and enterprise users."
- Context：公开月度更新博客显示团队按月发布小版本 + 补丁（如 2026-05 单月 89 commits、4 次 release），属高强度持续维护节奏。
- Confidence：高

### A6. 技术栈与工具链

**证据 11**
- Claim：Vuetify = Vue 3 + TypeScript + Sass/SCSS，MIT 协议；v2 时代完成 JS→TS、Stylus→Sass 迁移；2026 年 v4 默认启用 CSS layers 以便与 Tailwind 共存；Vuetify0 把十年积累的逻辑层抽成 headless 组件库（40 组件、71 composables、24 utilities，5,700+ 单元测试、98.7% 覆盖率）。基础设施成本低——"Open source projects… don't typically incur substantial operating costs"，文档站挂 Carbon 广告反而赚钱。
- Source：官方 Roadmap[^10^] / 博客[^11^] / Starter Story[^1^]
- Confidence：高

### A7. 获客与增长

**证据 12**
- Claim：增长完全依靠生态位卡位（Vue 生态最早的 Material Design 组件库）+ 文档站内容营销（2021 年文档站月均 ~100 万用户、3550 万页面浏览）+ Twitter/Discord 社区 + 与 Vue 生态伙伴（VueJobs、MadeWithVue、Vue Mastery）互相导流。GitHub 官方 2025 年文章总结其赞助成功要素：分层赞助档位（$1–$1,500/月）给 logo 位 + 优先支持，"not necessarily always the tech, but finding the people who care enough"。
- Source：Starter Story（含 GA 截图）[^1^] / GitHub Blog[^6^]
- Excerpt："the overwhelming majority of our traffic comes from developers using our documentation as a resource… increases the chance that they will move into a sales funnel by purchasing a theme from the Vuetify store, becoming a sponsor, or even requesting direct support."
- Confidence：高

### A8. 变现路径

**证据 13**
- Claim：Vuetify 的组合拳 = 赞助（GitHub Sponsors/Patreon/Open Collective/Tidelift）+ 付费主题商店 + Snips 订阅 + 咨询/企业支持（"Book time with the Team"）+ 文档广告（Carbon Ads）。核心哲学："Locking extra features or functionality behind a paywall just feels bad"——拒绝 open-core 功能收费；John 的教训："most companies—in regards to open source—have little to no interest in receiving services in return… a business model based primarily around sponsorship was simply ineffective; at least for me." 真正的转折是把商店做成主力收入。
- Source：Starter Story[^1^] / State of the Union 2024[^4^]
- Excerpt（2024 反思）："if you want to do Open Source full-time, you need to build something that can generate revenue."
- Confidence：高

### A9. 决策与转折

**证据 14**
- Claim：三次关键转折——①2017-10 辞职全职（alpha 反响超预期驱动）；②2019→2022 v2/v3 两次大版本重写（"developer churn"导致组件由不同开发者接力完成，是官方承认的最大挑战之一）；③2024 年财务枯竭危机：6 月公开求职 → 入职 Optikka → 9 月移交创意/工程控制权给 Kael，自己转向高层与"确保不再发生资金危机"的角色。
- Source：State of the Union 2024[^4^] / Starter Story[^1^]
- Excerpt："One of our toughest challenges has been developer churn… While we are still recovering from a tough financial drought, things are looking up."
- Confidence：高

### A10. 效率密码

**证据 15**
- Claim：夫妻店（John + Heather Leider）+ 合同制核心团队的轻结构；用 Open Collective 把"给钱"从创始人中转角色中剥离（透明基金，去除中间人）；把社区维护外包给 Discord 社区与核心团队；把商业产品（Store/Snips）与 MIT 主框架完全隔离——免费的是框架，收费的是内容与服务。
- Source：Meet the team[^7^] / Starter Story[^1^]
- URL：https://v2.vuetifyjs.com/en/about/meet-the-team/ （最后更新 2025-08-08）
- Excerpt："Vuetify (the company) is owned and operated by John and Heather Leider as a full-time Open Source business."
- Confidence：高

---

## 第二部分：案例B —— Simon Hamp 与 NativePHP

### B1. 背景故事与入行路径

**证据 16**
- Claim：Simon Hamp 是英国人，现居西班牙加那利群岛的 Gran Canaria；20+ 年经验的 PHP/Laravel 全栈开发者、工程管理者与创业者；LAMP 老兵，日常用 Mac + PHPStorm；职业主线是"用 PHP/Laravel 帮助创业公司成长、支持大企业"。履历中包含 Elvie（女性健康科技创业公司，他投入了近 6 年）以及后来自建的 Laravel 开发者招聘平台 Laradevs。
- Source：Laravel 官方博客"Artisan of the Day"[^15^] / nativephp.com 咨询页[^19^] / RocketReach 履历聚合[^29^]
- URL：https://laravel.com/blog/old-school-craft-the-artisan-of-the-day-is-simon-hamp ；https://nativephp.com/consulting
- Date：2025-09-03 / 持续
- Excerpt："Full-stack developer, engineering leader and entrepreneur with over 20 years of experience building web and mobile products… He has built a career on growing startups and supporting large enterprises with PHP and Laravel."
- Confidence：高（官方 bio + 履历交叉）

**证据 17**（命运转折点）
- Claim：2022 年底，Simon 从投入近 6 年的创业公司"自愿被裁"（voluntary redundancy），从稳定薪水变为零收入、只有几个月存款跑道。他没有去疯狂接自由职业单子，而是决定"终于把想了多年的点子做出来"——用 PHP 构建可分发的桌面应用。
- Source：Indie Hackers 专访（第一人称）[^16^]；Indie Hustle 长文[^17^]
- URL：https://www.indiehackers.com/post/tech/building-the-impossible-and-making-100k-in-the-first-three-months-n8qj3KIFVnXF1fUFuYaL ；https://www.indiehustle.co/p/making-100k-in-3-months-by-building
- Date：2025-05-22 / 2025-06-16
- Excerpt："At the end of 2022, I took voluntary redundancy from a startup… So I went from a reasonable, regular salary, to no income with only a few months of runway… I've got no savings, no recurring income... but instead of doing the 'right' thing I decided to finally try this idea I'd had for years."
- Confidence：高（本人自述）

### B2. 项目 0→1 时间线（精确到月）

**证据 18**（一手时间线组合）
- 2022 底：离职，开始酝酿想法[^16^]
- 2023-04-01：在 Twitter 发布 POC（静态编译 PHP + Tauri 壳，让完整 Laravel 应用跑在桌面上）；当时他的 Twitter 只有约 1,000 粉丝；Laravel 社区大牛 Marcel Pociot（Beyond Code 联合创始人）主动联系合作[^16^][^17^][^18^]
- 2023-07：Marcel 把 Laracon US（纳什维尔）的演讲主题从 AI 临时换成 NativePHP，向约千名 Laravel 开发者首次公开，"blew people's minds"[^18^][^24^]
- 2023–2024：桌面版以 Electron alpha 先行（为求快），随后回归 Tauri 并重构架构（移除内嵌 web server）；两人边做本职工作边维护[^24^][^26^]
- 2024 年：Marcel 转向其他项目，Simon 独自攻坚 iOS；向多个大会投递"Building mobile apps with PHP"演讲提案以逼自己上线[^18^][^17^]
- 2024-10 底：Laracon EU 接受演讲提案 → 只剩 2 个月把想法变成可演示产品；"每天从早上 6 点工作到凌晨两三点"[^28^][^17^]
- 2025-01：向 App Store 提交首个完全用 PHP/Laravel 构建的 iPhone 应用，3 天后获批——全球首例[^18^]
- 2025-01/02：通过 GitHub Sponsors 开 $200 "Early Adopter" 通行证，几天内 20+ 人付费，"started making money from Day 1"[^18^]
- 2025-02：Laracon EU 台上正式发布 NativePHP for iOS（现场就有大量成交）[^18^][^22^]
- 2025-03：Shane Rosenthal 搞定 Android 端；Simon 与 Shane 联合创立 Bifrost Technology 公司[^18^][^22^]
- 2025-04：NativePHP Desktop 1.0 发布（兼容 Laravel 12.x / PHP 8.4，官方支持 Windows）[^24^]
- 2025-05-02：NativePHP for Mobile v1 正式发售[^23^][^22^]
- 2025-10-17：授权价格下调（Mini 改为随 $19/月 Bifrost 订阅免费赠送；Pro $250→$200/年；Max $1,000→$350/年）[^21^]
- 2025-11-29：Mobile v2 发布；官方披露"送出 1,000 个授权、下载破 10,000、客户超 2,500"，Simon 与 Shane 均已全职[^22^]
- Source：Starter Story Simon 专访[^18^] / nativephp.com 官方博客[^21^][^22^] / Laravel News[^23^] / NashTech 历史梳理[^24^]
- Confidence：高（多个时间点由本人及官方博客交叉确认）

### B3. 收入数据

**证据 19**
- Claim：NativePHP for Mobile 发售后 3 个月收入突破 $100,000（2025-05，Indie Hackers 标注"Revenue $50K a month"）[^16^][^17^]；到 2026 年年中，仅授权销售就接近 $60K/月 的稳定收入，Starter Story 折算约 $720K/年[^18^]。
- Source：Indie Hackers / Indie Hustle / Starter Story
- URL：https://www.indiehackers.com/post/tech/building-the-impossible-and-making-100k-in-the-first-three-months-n8qj3KIFVnXF1fUFuYaL ；https://www.starterstory.com/stories/nativephp-baking-delicious-native-appsnativephp
- Date：2025-05-22（$100K/3 个月）；2026-06-05（$60K/月）
- Excerpt："We're now getting close to $60K/month in steady income just from license sales."（Simon 原话）
- Context：注意矛盾点——Indie Hackers 摘要写"$50K a month"但正文口径是"3 个月 $100K"（≈$33K/月均值），两者并存，报告中应引用为"前 3 个月累计 $100K，2026 年稳定至 ~$60K/月"。桌面版（开源 MIT）本身不直接收费，靠 GitHub Sponsors 与企业赞助（BeyondCode、Laradevs、Sevalla 等）支撑[^25^]。
- Confidence：高（本人两次披露，时间点清晰）

**证据 20**（授权模式细节）
- Claim：采用"build many, release one"式分发授权：Mini = 1 个生产应用、Pro = 10 个、Max = 不限；授权终身有效、含 1 年更新，续费即恢复更新权，不续费仍可继续构建和上架（只是拿不到新版本）。EAP 定价 $50/$150/$250，原计划 5-31 后涨至 $100/$750/$2,500；2025-10 反而大幅降价（Pro $200/年、Max $350/年、Mini 并入 $19/月 Bifrost 订阅免费送）。
- Source：GitHub Discussions #528（Simon 本人）[^20^] / nativephp.com 博客[^21^] / Laravel News[^23^]
- URL：https://github.com/orgs/NativePHP/discussions/528 ；https://nativephp.com/blog/an-update-on-mobile-license-prices
- Date：2025-04-21 / 2025-10-17
- Excerpt："A license grants you the right to distribute a certain number of mobile apps to the stores… Your license is for life. All licenses come with 1 year of updates included."
- Confidence：高（官方定价帖）

### B4. 项目现状（2026 年）

**证据 21**
- Claim：NativePHP 桌面端为 MIT 开源（GitHub ~3.8K stars，2025 年中口径），由 Simon、Marcel 与社区维护；移动端为闭源付费授权，但官方多次声明"长期目标是开源"。2025 年两人已在 Laravel/PHP 社区做了"几十场"线上线下演讲；团队提供付费咨询（nativephp.com/consulting）；2026-03 Vonage 开发者博客仍在做双方创作者访谈，项目活跃。
- Source：GitHub[^25^] / NashTech[^24^] / nativephp.com[^19^][^22^] / Vonage[^26^]
- URL：https://developer.vonage.com/en/blog/weird-science-building-android-apps-with-nativephp
- Date：2026-03-31
- Excerpt："We've been building that for over 3 years (Mac, Linux, Windows). In 2024, Simon cracked the code and compiled PHP for iOS… Soon after, Shane did the same on Android."
- Confidence：高

### B5. 日常作息与开发节奏

**证据 22**
- Claim：冲刺期极端作息——Laracon EU 前两个月"每天早 6 点干到凌晨两三点"（"consumed the last three months of my life"）；日常方法论是纸笔清单："writing things down with a real pen and paper helped me to crystalize my thoughts… scratch through things that I just didn't need to do"。两人都全职后仍高频出差演讲（"dozens of talks in-person and online"）。
- Source：StartupSeries[^28^] / Starter Story[^18^] / 官方博客[^22^]
- URL：https://startupseries.io/mobile-app-builder-made-100k-in-three-months/
- Date：2025-08-21
- Confidence：高

### B6. 技术栈与工具链

**证据 23**
- Claim：桌面版 = 静态编译 PHP 二进制 + Electron/Tauri 壳（用 PHP 内置 web server 通信）；移动端必须静态编译（iOS 不允许多后台进程/起 web server），扩展需编译期内置，换来分发便利与小幅性能提升；用 Swift/Java 写原生桥接。选择 PHP 生态的根本原因：让数百万 Laravel 开发者"零新学习成本"构建原生应用——"No Swift. No Kotlin. No Flutter. No React Native. Just Laravel."
- Source：Vonage 访谈[^26^] / Laravel News[^23^] / Starter Story[^18^]
- Excerpt："Static binaries mean you don't need to 'install' or 'configure' PHP; you can just distribute a single executable… PHP even gets a little bit of a performance boost."
- Confidence：高

### B7. 获客与增长

**证据 24**
- Claim：增长三板斧——①借 Laracon 舞台冷启动（Marcel 2023 Laracon US 临时换题 + Simon 2025 Laracon EU 现场发售，"announce at a conference"成为核心打法）；②build in public：Twitter 连载进度、提前向粉丝"剧透"苹果审批结果；③把利润反哺社区：真金白银赞助工程师、开源项目和 meetup（"We're members of this community and we're investing in it"），并送出 1,000 个免费授权。Simon 的经验之谈："play to your strengths… tap into networks where you've already built up personal trust."
- Source：Starter Story[^18^] / Indie Hackers[^16^] / Indie Hustle[^17^]
- Confidence：高

### B8. 变现路径

**证据 25**
- Claim：与 Vuetify 相反的选择——NativePHP 桌面版保持 MIT 免费攒生态位，移动端直接做闭源付费授权（premium package rather than open source），理由是移动端"needs care, funding, and momentum"；同时辅以免 sponsor 收入（GitHub Sponsors 早鸟票）、Bifrost 构建服务订阅（$19/月起）、咨询服务。收费时点：从第一天就收费（$200 早鸟），不是"先免费攒够再收费"，而是"免费版攒信誉 → 新能力直接收费"。
- Source：Starter Story[^18^] / Laravel News[^23^] / 官方博客[^21^]
- Excerpt："I decided to offer this as a premium package, rather than open source… this completely impossible thing started making money from Day 1."
- Confidence：高

### B9. 决策与转折

**证据 26**
- Claim：四个关键决策——①2022 底放弃"正确的事"（接 freelance），赌多年想法；②2025-01 决定移动端闭源收费（而非沿袭桌面版开源路线）；③邀请 Shane 成为联合创始人而非雇员（"He saw the potential… he came to meet me with a ton of work he'd done on Android"），把 iOS-only 提前一年变成双平台；④2025-10 主动大幅降价扩大盘子（2,500+ 客户后"you don't need to tell me twice that this is the signal to go all-in"）。
- Source：Starter Story[^18^] / 官方博客[^21^][^22^]
- Confidence：高

### B10. 效率密码

**证据 27**
- Claim：两人远程跨国合伙（Simon 在加那利群岛、Shane 数千英里之外），全靠信任与分工（iOS/Android 各管一摊）；全面拥抱 AI 提效："we use AI every single day to keep on making both the business and the product better"；用社区赞助者做最早的付费验证（GitHub Sponsors 早鸟票 = 预售 + 验证二合一）。
- Source：Starter Story[^18^]
- Confidence：高

### B11. 播客/媒体一手来源清单（案例B）
- php[architect] 播客《Community Corner: NativePHP for Mobile With Shane Rosenthal and Simon Hamp》，2025-07-09[^27^]（https://www.phparch.com/podcast/community-corner-nativephp-for-mobile-with-shane-rosenthal-and-simon-hamp/）
- The Bucket 播客（Simon 与 Steven Fox，NativePHP 起源故事），见 Laravel News 引述[^23^]
- Vonage 开发者博客双人访谈，2026-03-31[^26^]
- Laravel 官方博客 Artisan of the Day 人物特写，2025-09-03[^15^]

---

## 第三部分：开源变现整体市场数据（2026 年视角）

### M1. GitHub Sponsors 规模

**证据 28**
- Claim：GitHub Sponsors（2019 年上线）累计向开源维护者支付超过 **$100M**；覆盖 103 个地区、70,000+ 维护者/组织、280,000+ 赞助者（含财富 500 强公司）；增速在加快——第一个 $10M 花了近两年，最近一个 $10M 只花 5 个月；2023 年开放组织级赞助后与 Patreon 打通。
- Source：GitHub 官方博客《$100 million for open source》[^30^]
- URL：https://github.blog/open-source/maintainers/100-million-for-open-source-a-milestone-built-by-the-community/
- Date：2026-07-21
- Excerpt："more than $100 million has been invested in open source maintainers and projects through GitHub Sponsors… supports over 70,000 maintainers and organizations, and includes more than 280,000 sponsors."
- Confidence：高（官方）

### M2. 维护者收入分布（"大多数赚不到钱"的事实数据）

**证据 29**
- Claim：Tidelift 2024《State of the Open Source Maintainer》报告（437 名维护者样本）：**60% 维护者是无偿业余者**，其中 44% 希望获得报酬；仅 12% 是靠维护项目获得全部/大部分收入的"职业维护者"，24% 为半职业；收入来源 Top3 为捐赠平台（25%）、雇主工资（24%）、Tidelift（19%）；48% 感觉不被赏识，**60% 曾退出或考虑退出维护工作**；付费维护者实施安全/维护实践的概率比无偿者高 55%。2023 年报告口径几乎相同（60% 无偿、13% 职业、23% 半职业）。
- Source：Tidelift 2024 报告新闻稿（Business Wire）[^31^] / 2023 报告[^32^] / Tidelift 博客细分[^33^]
- URL：https://www.businesswire.com/news/home/20240917030299/en/ ；https://www.businesswire.com/news/home/20230502005244/en/ ；https://blog.tidelift.com/whos-paying-the-maintainers-donation-programs-employers-and-tidelift
- Date：2024-09-17 / 2023-05-02
- Confidence：高

**证据 30**
- Claim：赞助收入的极端长尾——Open Source Endowment（2026-02-26 上线的"大学捐赠基金式"开源资助基金，初始 $752K，GitHub 前 CEO Thomas Dohmke、HashiCorp 创始人 Mitchell Hashimoto 等 102 人捐助）引用数据：**GitHub Sponsors 上 76% 的档位月收入不足 $100**；独立分析显示活跃 Open Collective 项目年度筹资中位数 **不足 $3,000/年**；少数顶部例外：Evan You 靠赞助年入 $150K+（Patreon 高峰期 $16K/月），但"for every Vue.js, there are thousands of critical libraries earning nothing"。
- Source：Byteiota 对 OSE 的报道[^38^] / ossalt.com 资助模式分析[^39^] / RedMonk 对 Evan You 的采访[^45^]
- URL：https://byteiota.com/open-source-endowment-752k-fund-for-maintainer-burnout/ ；https://ossalt.com/guides/open-source-funding-models-sustainability-2026 ；https://redmonk.com/videos/evan-you/
- Date：2026-03-15 / 2026-03-29 / 2026-06-17
- Excerpt（Evan You 2026 原话）："Vue… downloaded close to 10 million times a week… the amount of sponsorship we get, it's sustainable, but it's definitely not substantial by any means compared to the scale of impact."（Vue 的赞助大约只够养 2 名全职）
- Context：76%<$100/月 与 $3,000/年中位数均出自第三方分析，非平台官方口径，标注为中置信度。
- Confidence：中高

**证据 31**
- Claim：Open Collective 体系 2024 年收到约 **$12.5M** 捐款（回到 2022 年 $12.8M 水平；2023 年曾下滑 23%），向维护者支付 **$9.7M**（同比 +20%）；经 OSC 流转的 GitHub Sponsors 资金 2024 年增长 40% 至 $1M+；Open Collective 全平台管理资金 $40M、1 万+ collectives。
- Source：Open Source Collective 2024/25 董事会报告[^34^] / Open Collective 官网[^35^]
- URL：https://opencollective.com/opensource/updates/2024-25-board-and-strategic ；https://opencollective.com/
- Date：2025-05-30 / 2026 年快照
- Confidence：高（官方财报）

**证据 32**
- Claim：Open Source Pledge（2024-10 由 Sentry 牵头 20+ 组织发起）要求企业按"每全职开发者 $2,000/年"直接付费给维护者；启动时成员合计承诺约 $1.3M；Sentry 自身 2023 年分发 $500K 给 500+ 维护者（2021 年 $155K、2022 年 $260K 逐年递增）。
- Source：Open Source Pledge 仓库[^36^] / Technical.ly[^37^]
- URL：https://github.com/opensourcepledge/opensourcepledge.com ；https://technical.ly/software-development/open-source-pledge-developer-pay/
- Date：2024-10-08
- Confidence：高

### M3. AI 时代开源变现的变化（2026 年可行性素材）

**证据 33**（AI 吞噬"文档流量 → 付费转化"漏斗：Tailwind CSS 危机）
- Claim：2026-01-06/07，Tailwind CSS 创始人 Adam Wathan 在 GitHub PR #2388 评论及后续播客中披露：文档站流量自 2023 年初下降约 40%（因 AI 编程助手直接回答 Tailwind 问题，用户不再访问文档站——而文档站正是 Tailwind UI 付费产品的获客漏斗）；收入暴跌约 80%；Tailwind Labs 裁掉 75% 工程师（4 人中裁 3 人）；Wathan 称之为"the brutal impact AI has had on our business"，并说"we had six months left"（资金只剩 6 个月）。讽刺的是框架本身 7,500 万/月下载、比以往任何时候都流行。
- Source：DevClass 2026-01-08 / Socket.dev 2026-01-08 / 汇总见 AdQuick 引用清单[^40^] 与 Techyverse[^41^]
- URL：https://devclass.com/2026/01/08/tailwind-labs-lays-off-75-percent-of-its-engineers-thanks-to-brutal-impact-of-ai/ ；https://socket.dev/blog/tailwind-css-announces-layoffs
- Date：2026-01-08
- Context：对"先免费后收费"模式的直接警示：当免费层（文档/内容）是付费产品的获客渠道时，AI 答案引擎会切断漏斗。Vuetify 2021 年"文档即漏斗"打法在 2026 年的可行性因此存疑——Vuetify 自己的应对是把 MCP server 做成生态产品（2025-10 起）。
- Confidence：高（多源一致，含一手 PR 评论）

**证据 34**（AI 垃圾内容加重维护负担）
- Claim：curl 项目（Daniel Stenberg）2025 年对 AI 生成的虚假安全报告实行"零容忍"——直接封禁提交者，并公开列出 17 份 AI 编造的 HackerOne 报告；Django 随后跟进类似政策。curl 指南原文："Fake and otherwise made up security problems effectively prevent us from doing real project work… We ban users immediately."
- Source：Socket.dev[^42^]
- URL：https://socket.dev/blog/django-joins-curl-in-pushing-back-on-ai-slop-security-reports
- Date：2025-06-30
- Confidence：高

**证据 35**（License 变更风潮对个人开发者的启示）
- Claim：2023-08-10 HashiCorp 将 Terraform/Vault/Consul 等全线产品从 MPL 2.0 改为 BSL 1.1（点名 Spacelift/env0/Scalr"用我们造的工具与我们竞争不公平"）→ 一个月内社区 fork 出 OpenTofu（Linux 基金会，2026 年 2 月 IBM 以 $6.4B 完成收购 HashiCorp 后 BSL 仍未撤回；OpenTofu 2026 年初月下载破 170 万）；2024-03 Redis 从 BSD 改为 SSPL/RSALv2 → 数天内 AWS/Google/Oracle 等 fork 出 Valkey 捐给 Linux 基金会。这一波"relicensing rebellion"的本质是大公司防云厂商白嫖；对个人开发者的启示是双向的：①纯 MIT/Apache 项目的商业价值可能被大公司零成本攫取（NativePHP 移动端选择闭源授权正是小型对应物）；②但个人项目改 license 没有大公司的议价能力，社区 fork 风险更高，因此"付费授权/双许可"比"事后改 license"更适合独立开发者。
- Source：OneUptime[^43^] / youngju.dev 深度长文[^44^] / Spacelift 等
- URL：https://oneuptime.com/blog/post/2026-03-31-redis-vs-valkey-fork-explained/view ；https://www.youngju.dev/blog/culture/2026-05-16-open-source-license-shifts-2026-bsl-wave-elastic-redis-hashicorp-sentry-valkey-opentofu-deep-dive.en
- Date：2026-03-31 / 2026-05-16
- Confidence：高（事件事实）/ 中（对个人的启示为分析性推断，已在文中标注）

**证据 36**（赞助模式有效性的一手验证）
- Claim：GitHub 开源资助项目负责人 Kevin Crosby 2025-06 总结获资助项目的共性："The most interesting thing is they have fairly engaged communities that appreciate the maintainer and the project. It's not necessarily always the tech, but finding the people who care enough and want to fund the project itself." 并建议维护者"像产品经理一样设计赞助档位"，把"产品/ICP"想清楚。
- Source：GitHub Blog[^6^]
- Confidence：高

**证据 37**（对照组：Evan You / Vue）
- Claim：Evan You 2016 年辞职全职做 Vue 时，Patreon + Strikingly CTO 赞助合计不足 $5,000/月，"远低于工资"仍决定赌一把；2026 年 Vue 每周下载近 1,000 万次，但赞助仅够维持约 2 名全职——"Even that Vue has been an outlier for independent open source… Babel… has historically always struggled with funding."
- Source：dennisrdatanews 访谈整理[^46^] / RedMonk 视频采访[^45^]
- URL：https://dennisrdatanews.netlify.app/post/2023-08-26-evan-you-interview/ ；https://redmonk.com/videos/evan-you/
- Date：2023-08-26 / 2026-06-17
- Confidence：中高（中文转述 + 英文一手视频）

---

## 来源列表

- [^1^]: Starter Story — I Built A $300K/Year Vue.js Component Library（2021-08 采访 / 页面 2025-12-10）https://www.starterstory.com/stories/i-built-a-300k-year-vue-js-component-library
- [^2^]: Starter Story — Vue.js Component Library Success Stories https://www.starterstory.com/ideas/vue-js-component-library/success-stories
- [^3^]: Starter Story 邮件摘要（$25k/month 口径，2025-04-14）https://www.starterstory.com/kid-launches-niche-newsletter
- [^4^]: Vuetify 官方博客 — State of the Union 2024（2024-09-08）https://vuetifyjs.com/zh-Hans/blog/state-of-the-union-2024/
- [^5^]: GitHub Sponsors — @johnleider https://github.com/sponsors/johnleider
- [^6^]: GitHub Blog — 4 trends shaping open source funding（2025-06-03）https://github.blog/open-source/maintainers/4-trends-shaping-open-source-funding-and-what-they-mean-for-maintainers/
- [^7^]: Vuetify — Meet the team（2025-08-08）https://v2.vuetifyjs.com/en/about/meet-the-team/
- [^8^]: Vuetify Store — About us https://store.vuetifyjs.com/pages/about-us
- [^9^]: 硬地骇客 — Vuetify 的开源变现方式（2023-07-29）https://hardhacker.com/posts/vuetify
- [^10^]: Vuetify 官方 Roadmap https://vuetifyjs.com/vuetify/roadmap ；endoflife.date/vuetify（2026-08-14）https://endoflife.date/vuetify
- [^11^]: Vuetify 官方博客索引（2025-11 ~ 2026-08 月度更新）https://vuetifyjs.com/en/blog/
- [^12^]: Snyk — vuetify 包统计（2025-12-14 快照）https://security.snyk.io/package/npm/vuetify
- [^13^]: vue-pdf-viewer.dev — Best Vue UI Libraries 2026（2026-01-20）https://www.vue-pdf-viewer.dev/blog/best-vuejs-ui-libraries-for-2026-top-5-picks-bonus/
- [^14^]: Vuetify Snips 定价页 https://snips.vuetifyjs.com/marketing/page-sections/pricing-sections
- [^15^]: Laravel 官方博客 — Artisan of the Day: Simon Hamp（2025-09-03）https://laravel.com/blog/old-school-craft-the-artisan-of-the-day-is-simon-hamp
- [^16^]: Indie Hackers — Building the impossible and making $100k in the first three months（2025-05-22）https://www.indiehackers.com/post/tech/building-the-impossible-and-making-100k-in-the-first-three-months-n8qj3KIFVnXF1fUFuYaL
- [^17^]: Indie Hustle — Making $100k In 3 Months By Building The Impossible（2025-06-16）https://www.indiehustle.co/p/making-100k-in-3-months-by-building
- [^18^]: Starter Story — I Created A $60K/Month Mobile App Builder（Simon Hamp 专访，2026-06-05）https://www.starterstory.com/stories/nativephp-baking-delicious-native-appsnativephp
- [^19^]: NativePHP — Consulting 团队页 https://nativephp.com/consulting
- [^20^]: GitHub Discussions #528 — NativePHP for mobile Pricing update（2025-04-21）https://github.com/orgs/NativePHP/discussions/528
- [^21^]: NativePHP 官方博客 — An Update on Mobile License Prices（2025-10-17）https://nativephp.com/blog/an-update-on-mobile-license-prices
- [^22^]: NativePHP 官方博客 — Mobile v2 is Here!（2025-11-29）https://nativephp.com/blog/mobile-v2-is-here
- [^23^]: Laravel News — NativePHP for Mobile v1 — Launching May 2（2025-04-23）https://laravel-news.com/nativephp-mobile-v1
- [^24^]: NashTech — How I Built a Health App Using PHPNative（历史梳理，2025-07-11）https://blog.nashtechglobal.com/how-i-built-a-health-app-using-phpnative-with-only-laravel-and-php/
- [^25^]: GitHub — NativePHP/desktop https://github.com/nativephp/desktop
- [^26^]: Vonage Developer — Weird Science: Building Android Apps with NativePHP（2026-03-31）https://developer.vonage.com/en/blog/weird-science-building-android-apps-with-nativephp
- [^27^]: php[architect] 播客 — NativePHP for Mobile（2025-07-09）https://www.phparch.com/podcast/community-corner-nativephp-for-mobile-with-shane-rosenthal-and-simon-hamp/
- [^28^]: StartupSeries — How Simon Hamp Built a Mobile App Builder & Made $100K in Three Months（2025-08-21）https://startupseries.io/mobile-app-builder-made-100k-in-three-months/
- [^29^]: RocketReach — Simon Hamp 履历（Elvie / Laradevs / NativePHP）https://rocketreach.co/simon-hamp-email_51422489
- [^30^]: GitHub Blog — $100 million for open source（2026-07-21）https://github.blog/open-source/maintainers/100-million-for-open-source-a-milestone-built-by-the-community/
- [^31^]: Tidelift 2024 State of the Open Source Maintainer（Business Wire，2024-09-17）https://www.businesswire.com/news/home/20240917030299/en/
- [^32^]: Tidelift 2023 报告（Business Wire，2023-05-02）https://www.businesswire.com/news/home/20230502005244/en/
- [^33^]: Tidelift 博客 — Who's paying the maintainers https://blog.tidelift.com/whos-paying-the-maintainers-donation-programs-employers-and-tidelift
- [^34^]: Open Source Collective — 2024/25 Board Report（2025-05-30）https://opencollective.com/opensource/updates/2024-25-board-and-strategic
- [^35^]: Open Collective 官网统计 https://opencollective.com/
- [^36^]: Open Source Pledge 仓库 https://github.com/opensourcepledge/opensourcepledge.com
- [^37^]: Technical.ly — Open Source Pledge $1.3M（2024-10-08）https://technical.ly/software-development/open-source-pledge-developer-pay/
- [^38^]: Byteiota — Open Source Endowment（2026-03-15）https://byteiota.com/open-source-endowment-752k-fund-for-maintainer-burnout/
- [^39^]: ossalt.com — Open Source Funding Models & Sustainability 2026（2026-03-29）https://ossalt.com/guides/open-source-funding-models-sustainability-2026
- [^40^]: AdQuick 引用清单（汇总 DevClass/Socket.dev 对 Tailwind 的报道，2026-07-08）https://www.adquick.com/blog/ai-agents-make-out-of-home-advertising-more-valuable/
- [^41^]: Techyverse — Tailwind CSS Lost 40% Traffic to AI（2026-06-06）https://www.techyverse.in/blog/saurabh-jadhav/tailwind-css-ai-impact-developer-documentation
- [^42^]: Socket.dev — Django Joins curl in Pushing Back on AI Slop Security Reports（2025-06-30）https://socket.dev/blog/django-joins-curl-in-pushing-back-on-ai-slop-security-reports
- [^43^]: OneUptime — Redis vs Valkey: The Fork Explained（2026-03-31）https://oneuptime.com/blog/post/2026-03-31-redis-vs-valkey-fork-explained/view
- [^44^]: youngju.dev — The 2026 Open Source License Shift（2026-05-16）https://www.youngju.dev/blog/culture/2026-05-16-open-source-license-shifts-2026-bsl-wave-elastic-redis-hashicorp-sentry-valkey-opentofu-deep-dive.en
- [^45^]: RedMonk — Inside the Acquisition: VoidZero Joins Cloudflare with Evan You（2026-06-17）https://redmonk.com/videos/evan-you/
- [^46^]: dennisrdatanews — Evan You 访谈整理（2023-08-26）https://dennisrdatanews.netlify.app/post/2023-08-26-evan-you-interview/

---

## 写作素材摘要（按 11 组整理）

### 案例A：Vuetify（John Leider）——"赞助撑不住梦想，商店才是答案"

**1. 背景与入行**：John Leider 是退伍军人（2013 年 1 月退役），靠自学认证进入 IT，早期是 PHP/jQuery 服务端开发者。2014 年底学 Laravel 时在 Laracasts 视频里邂逅 Vue.js 早期版本，"立刻爱上"。这是典型的非科班独立开发者路径，也是他后来一切故事的地基：没有大厂光环，只有社区嗅觉。[^1^]

**2. 0→1 时间线**：2016 年 9 月发布 pre-alpha，12 月 14 日以 MIT 协议公开 alpha，反响"overwhelming"；2017 年 10 月辞职全职做开源；2018 年 2 月 v1.0；2019 年 v1.5 LTS、7 月 v2.0 核心重写；2022 年 11 月 v3.0 (Titan) 拥抱 Vue 3；2024 年起搭建 Vuetify One / Bin / Playground / Snips 生态产品矩阵；2026 年 2 月 v4 正式版、7 月 headless 逻辑层 Vuetify0 1.0。十年三个大版本重写，是这个案例"长期主义"的主线。[^1^][^10^][^11^]

**3. 收入数据**：2021 年高峰期年收入约 $300K——结构是 Patreon+GitHub Sponsors $6,500/月、Open Collective $2,000+/月（养团队）、付费主题商店 $15K–20K/月（主力）、另有 Carbon 广告与咨询。起步时月捐赠仅 $500。2024 年陷入财务"枯竭期"，6 月被迫公开求职；9 月靠 Abacus、Route4Me、Teamwork 等企业大额赞助回升到 OC+GS 合计 $8,000+/月。口径提醒：$300K/年 ≈ $25K/月是全渠道口径，$6,500/月只是赞助项。[^1^][^3^][^4^]

**4. 现状（2026）**：41K+ stars、70–85 万次/周 npm 下载、440 名贡献者、4 名维护者；v4 已发布、v3 进入 LTS；团队 2025 年 12 月单月 522 commits 创年度纪录。John 仍以月度博客形式对外沟通，但工程主导权已移交 Kael。[^11^][^12^][^13^]

**5. 日常节奏**：创始人日常 = 新功能 + 修 bug + 招募开源开发者 + 企业直接支持四线并行；团队用 Open Collective 发票报销制结算，核心成员曾有最高 $1,000/月津贴。月度 release train（如 2026 年 5 月单月 89 commits、4 次发布）说明这是高强度职业维护，而非业余项目。[^1^][^11^]

**6. 技术栈**：Vue 3 + TypeScript + Sass，MIT；v2 完成 JS→TS 迁移；v4 默认 CSS layers 以兼容 Tailwind；Vuetify0 把十年逻辑层抽成 headless 库（5,700+ 测试、98.7% 覆盖率）。基础设施成本极低，文档站挂广告反而盈利。[^1^][^10^][^11^]

**7. 获客增长**：生态位卡位（Vue 最早的 Material Design 组件库）+ 文档站内容漏斗（2021 年月均 100 万用户、3550 万 PV）+ Twitter/Discord + 生态伙伴互推。GitHub 官方总结其赞助策略：$1–$1,500/月分层档位，给 logo 位与优先支持。[^1^][^6^]

**8. 变现路径**：拒绝 open-core 功能收费（"把功能锁在付费墙后感觉很糟"），选择"框架全免费 + 内容/服务收费"：赞助、主题商店、Snips 订阅（$14–36/用户/月）、咨询、广告五引擎。关键教训："以赞助为主的商业模式根本无效——至少对我是这样"；2024 年他写下全文核心句："想全职做开源，必须造一个能产生收入的东西。"[^1^][^4^][^14^]

**9. 决策与转折**：三次转折——2017 辞职、2019–2022 两次大重写（伴随严重的 developer churn）、2024 资金危机（求职、入职 Optikka、移交控制权）。这是"开源变现并非线性增长"的最佳反例素材。[^4^]

**10. 效率密码**：夫妻店 + 合同制核心团队的极简结构；用 Open Collective 透明基金把"分钱"制度化、去人格化；商业产品与 MIT 主框架彻底隔离，免费攒口碑、收费在周边。[^1^][^7^]

**11. 2026 可行性对照**：Vuetify 的"文档即获客漏斗"正是 Tailwind 2026 年被 AI 击穿的同款模型（Tailwind 文档流量 -40%、收入 -80%、裁员 75%）——这解释了 Vuetify 为何把 MCP server 做成生态产品，也提示报告写作时应把"Vuetify 模式"标注为"AI 前时代验证有效、AI 时代需要打补丁"。[^40^][^41^][^11^]

### 案例B：Simon Hamp / NativePHP——"免费攒生态，新能力从第一天收费"

**1. 背景与入行**：20+ 年 PHP/Laravel 老兵，英国人，现居加那利群岛；在创业公司（Elvie）投入近 6 年，2022 年底"自愿被裁"，零收入、几个月跑道。他没有去接自由职业，而是赌上了酝酿多年的想法：用 PHP 做可分发的原生应用。[^15^][^16^][^29^]

**2. 0→1 时间线**：2023-04-01 发 POC 推文（粉丝仅约 1,000）→ Marcel Pociot 主动合作 → 2023-07 Laracon US 临时换题引爆社区 → 2024 年 Marcel 淡出、Simon 独攻 iOS → 2024-10 Laracon EU 接受演讲，两个月死线冲刺（早 6 点到凌晨两三点）→ 2025-01 全球首个纯 PHP iPhone 应用过审（3 天）→ GitHub Sponsors $200 早鸟票几天卖出 20+ 张 → 2025-02 Laracon EU 台上发布 → 2025-03 与 Shane Rosenthal 创立 Bifrost Technology → 2025-04 桌面版 1.0 → 2025-05-02 移动端 v1 发售 → 2025-10 降价扩容 → 2025-11 v2，2,500+ 客户、下载破万，两人全职。[^16^][^18^][^22^][^23^][^24^]

**3. 收入数据**：移动端发售 3 个月破 $100K；2026 年年中授权销售接近 $60K/月（年化约 $720K）。桌面版保持 MIT 开源，靠 GitHub Sponsors 与企业赞助（BeyondCode、Sevalla 等）输血。数字均由 Simon 本人在 Indie Hackers 与 Starter Story 两次披露，可信度高于一般二手报道；注意"$50K/月"（IH 摘要）与"3 个月 $100K"（正文）的口径差。[^16^][^17^][^18^]

**4. 现状（2026）**：桌面端 MIT 活跃维护（~3.8K stars）；移动端闭源付费授权 + Bifrost 构建服务订阅（$19/月起）+ 咨询三引擎；官方称移动端"长期目标是开源"，但眼下"需要照顾、资金和势头"。2026-03 仍在 Vonage 等平台做深度技术访谈。[^21^][^24^][^26^]

**5. 日常节奏**：冲刺期极端作息 + 纸笔清单聚焦法；全职后把大量时间投在社区演讲（几十场）。他的优先级方法论："写下来的东西，划掉不需要做的。"[^18^][^28^]

**6. 技术栈**：静态编译 PHP 二进制 + Electron/Tauri（桌面）、Swift/Java 桥接（移动）；移动端被迫静态编译反而成了架构优势（单可执行文件、免安装、性能小升）。选择 PHP 的全部理由就是生态位："数百万 Laravel 开发者零学习成本做原生应用"。[^23^][^26^]

**7. 获客增长**：Laracon 舞台冷启动（2023 借 Marcel、2025 自己上台）+ build in public 连载 + 利润反哺社区（真金白银赞助其他开发者/项目/meetup、送 1,000 个授权）。他的总结值得直接引用进报告："在你已建立个人信任的网络里做增长；以陌生面孔进入陌生市场，代价会高得多。"[^18^]

**8. 变现路径**：与 Vuetify 互为镜像——桌面版免费攒生态位与信誉，移动端直接闭源授权收费，且从第一天就收（$200 早鸟 = 预售 + 需求验证二合一）。授权设计精巧：终身授权 + 1 年更新，续费恢复更新、不续可继续用旧版——降低了"订阅疲劳"抵触。2025-10 的主动降价（Max $1,000→$350/年）显示其在"价格 vs 盘子"上的迭代思维。[^18^][^20^][^21^]

**9. 决策与转折**：四个决策点——放弃"正确的事"赌想法；移动端改闭源；拉 Shane 做联合创始人而非雇员；降价扩容。每个都有本人原话佐证。[^18^][^22^]

**10. 效率密码**：跨国两人合伙、按平台分工；全面用 AI 提效（"we use AI every single day"）；用赞助者做早鸟付费验证，把 GitHub Sponsors 当预售渠道用——这是对"赞助"玩法的创造性改造。[^18^]

**11. 2026 可行性素材**：市场侧——GitHub Sponsors 累计 $100M（2026-07，70K 维护者/280K 赞助者），但 76% 档位月入 <$100、OC 项目年筹资中位数 <$3,000、60% 维护者无报酬、60% 考虑过弃坑（Tidelift 2024）；Open Source Pledge（$2,000/开发者/年）与 Open Source Endowment（捐赠基金模式）代表制度化尝试。AI 侧——Tailwind 危机证明"免费文档 → 付费产品"漏斗正在被 AI 答案引擎截断；curl/Django 对 AI 垃圾安全报告的封禁令显示维护成本不降反升。License 侧——HashiCorp→BSL、Redis→SSPL 引发的 fork 潮（OpenTofu/Valkey）说明大公司改协议防白嫖有效，但个人开发者没有同等议价能力，"一开始就把新能力做成付费授权"（NativePHP 路线）比"事后改 license"更现实。两案例合起来给出模式 5 的核心结论：**纯赞助模式天花板极低（Vue 周下载近千万也只够养 2 个全职），可持续的开源变现= 免费层建立生态位 + 一个能独立产生收入的商业层（商店/授权/订阅/服务），且该商业层最好在第一天就被设计出来。**[^30^][^31^][^34^][^38^][^39^][^45^]

---

## 矛盾点与注意事项（供写作时核对）

1. **Vuetify 收入口径**：Starter Story 标题 $300K/年（≈$25K/月）与其案例库页"平均 $6,500/月"矛盾；$6,500 仅为 2021 年 Patreon+GitHub Sponsors 单项。建议报告采用"$300K/年（2021，全渠道），其中赞助仅约 $6,500/月"的表述。[^1^][^2^][^3^]
2. **NativePHP 月收入**：Indie Hackers 摘要写 $50K/月，正文与 Simon 本人口径是"前 3 个月 $100K"；Starter Story 2026-06 为"接近 $60K/月"。三个数字分属不同时点，并非真矛盾，但引用时必须带时间。[^16^][^18^]
3. **NativePHP GitHub stars**：3.8K 为 2025 年中第三方口径[^24^]，2026 年应更高，写作前建议以 GitHub 实时数为准。
4. **Vuetify v1.0 日期**：官方 Roadmap 写 2018-02，endoflife.date 的"1 (LTS)"条目实为 v1.5（2019-02-05）；以 Roadmap 为准。[^10^]
5. **76% 档位 <$100/月、OC 中位数 <$3,000/年**：均出自第三方分析（OSE 报道、ossalt），非 GitHub/Open Collective 官方口径，建议标注"第三方估算"。[^38^][^39^]
6. **Tailwind 裁员细节**：一手出处为 Adam Wathan 在 GitHub PR #2388 的评论（2026-01-06/07）及后续播客，二手报道数字一致（-40% 流量、-80% 收入、裁 3/4 工程师），可信度高；写作时可直接引用 DevClass/Socket.dev 链接。[^40^]

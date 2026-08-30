# 模式8：游戏型（做自己想玩的）——研究素材

> 研究日期：2026-08-26。案例：LocalThunk（Balatro/小丑牌）、Eric Barone（ConcernedApe，Stardew Valley/星露谷物语）。
> 证据格式：Claim / Source / URL / Date / Excerpt / Context / Confidence。行内引用 [^N^] 对应文末来源列表。

---

## 第一部分：证据库

### A. LocalThunk / Balatro

#### A1. 背景与动机

**Claim A1-1**：Balatro 项目创建于 2021 年 12 月 13 日，是 LocalThunk 电脑里 `Learning/Lua/` 目录下名为 `template` 的文件夹；他当时用 IT 工作攒下的约 3 周假期做这个项目，最初想法是做在线多人版 "Big Cheat"（他和朋友自创、基于"大老二/Big 2"和"Cheat"的牌类游戏）。[^1^]
- Source: LocalThunk 本人博客《The Balatro Timeline》
- URL: https://localthunk.com/blog/balatro-timeline-3aarh
- Date: 2025-03-06
- Excerpt: "This is the creation date for template, a folder in my Learning/Lua/ directory… I had saved up about 3 weeks of vacation time from my IT job… The initial idea on Dec 13th was to create an online multiplayer version of Big Cheat."
- Context: 一手时间线，事无巨细记录到发售日。
- Confidence: 高（一手）

**Claim A1-2**：他做游戏的目的是"做出来"而非"被消费"；十年间作品只发给几个朋友玩，Balatro 是他 8 年开发经历里第一次考虑公开发售的游戏。刻意不玩其他 roguelike/牌组构筑游戏（包括直到 2023 年 5 月才首次玩《杀戮尖塔》），理由是"作为业余爱好，天真地探索设计本身就是乐趣所在"——他公开表示《Balatro》是他自己玩过的第一款牌组构筑游戏，并认为这反而是成功原因之一。[^1^][^2^]
- Source: The Balatro Timeline；Rogueliker 采访（中译见旅法师营地）
- URL: https://localthunk.com/blog/balatro-timeline-3aarh ; https://www.iyingdi.com/tz/post/5358091
- Date: 2025-03-06 / 2024-03-07
- Excerpt: "making games is my hobby, releasing them and making money from them is not… I wanted to make mistakes, I wanted to reinvent the wheel."（Timeline, 2021年12月末条目）
- Confidence: 高（一手）

**Claim A1-3**：灵感来源：童年与朋友玩的广东牌戏"大老二"（Big Two）+ Northernlion 游玩《幸运房东（Luck Be a Landlord）》的视频带来的计分制 roguelike 概念 + 纸牌接龙（Solitaire）的"低风险、常青"体验——他称 Balatro 为 "jazz solitaire"。扑克主题只是"引导工具/一层皮"，本人不玩扑克，且因统计学背景厌恶真实赌博。[^1^][^3^][^4^]
- Source: The Balatro Timeline；LocalThunk 博客《Solitaire》；GameDeveloper / NoClip 播客
- URL: https://localthunk.com/blog/balatro-timeline-3aarh ; https://www.gamedeveloper.com/business/localthunk-knew-balatro-needed-to-draw-players-in-with-poker
- Date: 2025-03-06 / 2024-05-02
- Excerpt: "the whole reason I made [Balatro] was, I made it for me. I know it's not a gambling game, I'm comfortable with that fact."
- Confidence: 高（一手）

**Claim A1-4**：匿名原因：并非营销噱头。Playstack 公关总监 Wout van Halderen："他本来就不是喜欢聚光灯的人……人们以为他在模仿 Banksy，其实不是，他只是想安静地做游戏、过自己的生活。" LocalThunk 本人："我当时随便做的决定，后来越来越庆幸——'谢天谢地我能保持低调'。"他也承认匿名的代价是"有点孤独"（难以融入开发者社群）。2026 年仍保持匿名。[^5^][^6^]
- Source: PC Gamer（经 GoNintendo 转引）；The Logic 专访
- URL: https://www.gonintendo.com/contents/46492-balatro-publisher-on-localthunk-s-decision-to-remain-anonymous ; https://thelogic.co/news/balatro-developer-interview/
- Date: 2025-03-20 / 2025-02-06
- Excerpt: "It's really wonderful, but it also means that it's a little lonely."（LocalThunk）
- Confidence: 高

**Claim A1-5**：背景细节：加拿大萨斯喀彻温省 IT 从业者；2026 年 2 月博客《Bad Grades》自述大学最后一年从工程专业退学、计算机课成绩差，夜里写小项目——这些项目成为 Balatro 的基础。"LocalThunk" 名字来源：Lua 语言的 `local` 关键字 + 伴侣学 R 语言时爱用的变量名 "thunk"。[^6^][^7^][^1^]
- Source: The Logic；ixbt.games 转述《Bad Grades》博客；The Balatro Timeline
- URL: https://thelogic.co/news/balatro-developer-interview/ ; https://ixbt.games/en/news/2026/02/24/sozdatel-balatro-podtverdil-rabotu-nad-versiei-11.html
- Date: 2025-02-06 / 2026-02-24
- Confidence: 高（前两者为权威转述一手博客）

#### A2. 时间线（精确到月份/日）

**Claim A2-1**（全部来自 The Balatro Timeline [^1^]，一手，2025-03-06 发布）：
- 2021-12-13：开工（`Learning/Lua/template` 文件夹）
- 2021-12-20：建立 `CardGame` 生产文件夹（至今未改名）；首次尝试像素画
- 2022-01：假期结束，但已"完全沉迷"，晚上+周末开发"Joker Poker"（工作标题）
- 2022-03：完全停工（他的创作习惯：没动力就停，避免 burnout）
- 2022-05：复工；首次考虑 Steam 发售（动机：伴侣博士毕业可能搬省、他需辞职，想拿 Steam 游戏当求职作品集）；诞生 "LocalThunk" 之名
- 2022-08：一位朋友反馈玩了"几十个小时"，促使他大幅扩张规模（手柄/触屏支持、教程角色 Jimbo、配乐、音效等）
- 2023-01：辞职搬家，决定 3–6 个月不找工作、全职开发（用积蓄）
- 2023-03：在 Fiverr 上找 Luis Clemente 做配乐——"当时唯一花过/打算花的钱"；定名 Balatro（朋友没人选这个名字，他自己坚持用）
- 2023-04：做商店素材，支付"可怕的 100 美元"Steam 上架费
- 2023-05 末：首个公开 Beta 上线，毫无水花；5 月底仅 48 个愿望单；同月首次玩《杀戮尖塔》
- 2023-06 初：Playstack 星探 Twitter 私信；Dan Gheesling 直播带动愿望单飙升；6-10 愿望单 183；6 月底 2,440；雇律师谈合同
- 2023-07：正式签约 Playstack；7 月中 Northernlion 被 Dan Gheesling 安利直播玩 50 回合 Demo，LocalThunk 私信送完整 Beta key；7 月底愿望单 28,661
- 2023-08：开始睡眠和心脏问题（持续至发售）
- 2023-09 末：新版（内容限制型）Demo 上线；愿望单 49,791；开始本地化改造
- 2023-10：Steam Next Fest 蹭到大量关注（愿望单 77,380）；与 Playstack 会议口误把 120 张 Joker 说成 150 张，干脆补做 30 张；Maarten De Meyer 加入负责移植
- 2023-11/12：crunch 期；愿望单 87,280 → 94,212
- 2024-01-19：主播邀请赛（Hafu 夺冠）；宣布发售日 2-20；看《The Abyss》时突发焦虑发作（panic attack），次日就医；1 月底愿望单 114,977
- 2024-02-19：媒体评分解禁，PC Gamer 91 分，Metacritic/OpenCritic 均超 90；发售日愿望单 208,401
- 2024-02-20 08:00 PST（提前 15 分钟）：全平台发售
- Confidence: 高（一手）

**Claim A2-2**：发售后销量里程碑：发售后几小时 Steam 售出 5 万份、收入超 60 万美元（"far more money than I've made in my entire life"）；首日 Steam 售出 11.9 万份；72 小时全平台 25 万份；10 天 50 万份；不到 1 个月（3-18）100 万份；2024-08 达 200 万；2024-09-26 手游上线；2024-12 初 350 万；2025-01-21 达 500 万（Playstack CEO Harvey Elliott 在 Pocket Gamer Connects London 宣布，不含 Apple Arcade 下载）。[^1^][^8^][^9^][^10^][^11^]
- Source: The Balatro Timeline；GameDeveloper；Game World Observer；PCGamesInsider
- URL: https://www.gamedeveloper.com/business/localthunk-s-balatro-sells-250-000-copies-in-first-three-days ; https://gameworldobserver.com/2024/03/18/balatro-sales-1-million-copies-first-month-localthunk ; https://www.pcgamesinsider.biz/news/74922/balatro-has-hit-five-million-sales/
- Date: 2024-02-23 / 2024-03-18 / 2025-01-22
- Excerpt: "By the end of launch day, Balatro has sold 119,000 units on Steam alone."（Timeline）
- Confidence: 高

**Claim A2-3**：奖项：2024-11 金摇杆最佳独立游戏+突破奖；2024-12-12 TGA 获最佳独立游戏、最佳首秀独立游戏、最佳手游三项，并获年度游戏提名（史上首个获 GOTY 提名的单人开发游戏）；2025-02 D.I.C.E. 三项；2025-03 GDC Awards 横扫年度游戏、最佳首秀、最佳设计、创新奖；2025-04 BAFTA 最佳首秀游戏（由演员 Ben Starr 代领）。领奖均由 Playstack 代表代为出席，LocalThunk 本人到场但无人知其身份。[^12^][^13^][^14^]
- Source: Epic Games Store News / IMDb；gaming.news；Wikipedia (Balatro)
- URL: https://gaming.news/news/2025-03-20/balatro-wins-game-of-the-year-at-gdc-awards-2025/
- Date: 2024-12-12 ~ 2025-04
- Confidence: 高

**Claim A2-4**：1.1 更新线：2024-08-07 Steam 公告承诺 2025 年推出大型免费玩法更新；2025-02 Bloomberg 访谈透露方向（新 Joker、重做 Matador、调整 Blue Stake），"守口如瓶以便随时 pivot"；2025-09-12 博客《I'm Slow》宣布 2025 年无法交付、改为"做完就发（it's done when it's done）"、全平台免费，自述发售后连续赶工（1.0.1 平衡补丁→手机移植）导致 2024 年底"彻底 burnout"，休息数月后 2025 年初以"每天几小时的业余节奏"复工；2026-02-20 博客《Bad Grades》末尾确认"P.S. Yes I'm still working on 1.1"，仍无日期。为平衡设计，他亲自把游戏打到最高难度 100% 完成度。[^15^][^16^][^17^][^7^]
- Source: GoNintendo 全文转载《I'm Slow》；The Verge；balatrohq.com 核验页
- URL: https://gonintendo.com/contents/52812-balatro-1-1-update-delayed-to-2026 ; https://www.theverge.com/games/883290/balatros-developer-is-still-working-on-the-games-big-1-1-update ; https://balatrohq.com/news/patches/
- Date: 2025-09-12 / 2026-02-23 / 2026-07-15
- Excerpt: "I'm a hobbyist developer at heart and I love to tinker… I am working slowly, but I like it that way."
- Confidence: 高（一手博客全文）

**Claim A2-5**：2026 年动态：2026-02-25 上线 Switch 2 版（Switch 1 玩家享折扣、存档不转移）；2026-06-30 上线 Epic Games Store；发行商 Playstack 于 2026-06-10 完成股权交易——TruFin 将 84.5% 股权以约 1.124 亿英镑（约 1.51 亿美元）售予 IMC 子公司 VantageCo，整体估值约 1.69 亿美元，估值主要锚定 Balatro 的商业表现。[^18^][^19^]
- Source: balatrohq.com 核验；TechTimes / Game World Observer
- URL: https://balatrohq.com/zh/news/latest-updates/ ; https://www.techtimes.com/articles/317459/20260531/balatro-publisher-playstack-acquired-gamespot-fandom-parent-imc-169m.htm
- Date: 2026-05-31 ~ 2026-07-15
- Confidence: 中高（交易事实可靠；估值数字来自 TruFin 公告转述）

#### A3. 收入与商业条款

**Claim A3-1**：定价：Steam/主机 $14.99；手游 $9.99（另进 Apple Arcade 订阅）。发售 1 小时内回本（"reached game-profitability within an hour"，Playstack CEO），8 小时毛收入超 100 万美元，为 Playstack 史上销售最快游戏。制作预算与分成条款未公开。[^20^][^21^][^22^]
- Source: GameDeveloper 转述 Playstack 对 GamesIndustry.biz 披露；Game World Observer；TouchArcade
- URL: https://www.gamedeveloper.com/business/balatro-made-1-million-within-hours-of-release ; https://gameworldobserver.com/2024/02/28/balatro-profitable-1-hour-gross-revenue-over-1-million
- Date: 2024-02-27/28
- Confidence: 高

**Claim A3-2**：手游收入：上线首周 100 万美元、2 个月约 440 万美元（AppMagic）；至 2025-01 净收入超 900 万美元；至 2025-04 约 1,497 万美元（AppMagic）；至 2025-10 累计超 2,500 万美元（PocketGamer.biz Top 50 报道，"an impressive feat for a premium game"）。[^11^][^23^][^24^]
- Source: PocketGamer.biz / app2top 转 AppMagic
- URL: https://www.pocketgamer.biz/the-top-50-mobile-game-makers-of-2025/ ; https://app2top.com/news/publisher-balatro-advised-other-publishers-to-invest-more-in-indie-teams-and-not-to-expect-huge-profits-from-them-279481.html
- Date: 2025-04-09 / 2025-10-07
- Confidence: 中高（第三方估算，口径不一，见文末矛盾点）

**Claim A3-3**：发行商盘子：Playstack 2024 年售出 710 万份、收入同比 +455%；2025 全年毛收入 5,530 万英镑（+24%）、税前利润 1,220 万英镑（+59%）；全目录 Steam 终身毛收入超 1 亿美元；"hit ratio" 超 85%，Balatro 计入时 ROIDC 超 500%（剔除后 300%）。可推断 Balatro 全平台总收入量级在数千万美元（5M 份 × $14.99 名义上限约 7,500 万美元，未扣折扣/平台/发行分成）。[^25^][^26^][^19^]
- Source: PocketGamer.biz；GameDeveloper（TruFin 年报）；app2top
- URL: https://www.pocketgamer.biz/playstack-revenue-surged-52-in-h1-2025-to-307m/ ; http://www.gamasutra.com/business/balatro-publisher-playstack-has-a-hit-ratio-of-more-than-85-percent
- Date: 2025-09-19 / 2026-03-19
- Confidence: 中高（公司财报口径可靠；Balatro 单品收入为推断）

**Claim A3-4**：LocalThunk 对钱的表态（Bloomberg/Jason Schreier 专访，2025-02-07）：钱让他能全职做游戏；和伴侣买了房，但"我们本来在 IT 工作时就量入为出，生活方式没怎么变……我还在尽责，我还有工作要做"。"压力很大但非常充实，可以整天做我热爱的事，想做多久做多久。" 自认幸运："I know I'm lucky in that regard."[^27^][^1^]
- Source: Bloomberg（经 ResetEra/IconEra 转引）；The Balatro Timeline
- URL: https://icon-era.com/threads/bloomberg-interview-with-balatro-developer-localthunk.15875/
- Date: 2025-02-07/10
- Excerpt: "The revenue on that page after just a couple hours was over $600,000, far more money than I've made in my entire life."（Timeline）
- Confidence: 高

#### A4. 作息与健康

**Claim A4-1**：开发期作息：2022 年全职 IT 工作期间，晚上+周末开发（"Evenings and weekends were Joker Poker time"）；2023-01 起辞职全职开发；2023-08 起出现睡眠与心脏问题（"每隔几晚要坐在沙发上睡，躺下会被心脏问题打断"）；2024-01 焦虑发作就医（"医生问我工作是不是压力很大，我都不知道怎么解释"）；发售前后 crunch 模式每天 12 小时；2024 年底彻底 burnout，停工数月；2025 年起以"每天几小时、业余心态"工作，"做完就发"。[^1^][^16^]
- Source: The Balatro Timeline；《I'm Slow》
- URL: https://localthunk.com/blog/balatro-timeline-3aarh ; https://gonintendo.com/contents/52812-balatro-1-1-update-delayed-to-2026
- Date: 2025-03-06 / 2025-09-12
- Confidence: 高（一手自述）

#### A5. 技术栈与成本

**Claim A5-1**：引擎 LÖVE（Love2D，免费开源 zlib 协议）+ Lua，基于他此前为 Autohike 自建的引擎代码改造；移植由 Love2D 专家 Maarten De Meyer 完成；1.0.1f 补丁还更新了 Love2D 引擎修复卡顿。发售前现金支出几乎为零：Fiverr 配乐（Luis Clemente）+ 100 美元 Steam 上架费——"当时唯一花过/打算花的钱"（配乐）。[^1^][^18^][^28^]
- Source: The Balatro Timeline；balatrohq 补丁说明；generalistprogrammer LÖVE 对比
- URL: https://localthunk.com/blog/balatro-timeline-3aarh ; https://generalistprogrammer.com/comparisons/love2d-vs-godot
- Date: 2025-03-06
- Confidence: 高（选择 LÖVE 的"原因"未见其本人明确论述，可归于既有自研引擎延续+零成本）

#### A6. 获客

**Claim A6-1**：纯口碑+主播驱动：2023-06 Dan Gheesling 直播→愿望单飙升；2023-07-19 Northernlion 直播→Steam 关注数单周暴涨近 400%；两版 Demo（先 50 回合限制、后内容限制）+ 两次 Steam Next Fest（2023-10、2024-02）+ 活跃 Discord 社群 + 2024-01-19 主播邀请赛；愿望单曲线：48（2023-05 末）→ 2,440（6 末）→ 28,661（7 末）→ 49,791（9 末）→ 77,380（10 末）→ 94,212（12 末）→ 114,977（2024-01 末）→ 208,401（发售日）。他自述"没做任何该做的营销，至今没给任何人发过求玩邮件"、"就是运气好（lucked in）"。Playstack 贡献：本地化多语言、首日全主机平台移植、媒体评测分发（2-19 评分解禁即 90+ 造势）。[^1^][^29^][^30^]
- Source: The Balatro Timeline；GIGAZINE 转 brothers-in-gaming 分析；GameDeveloper
- URL: https://gigazine.net/gsc_news/en/20250207-balatro-successful-launch-strategy/ ; https://www.gamedeveloper.com/business/localthunk-s-balatro-sells-250-000-copies-in-first-three-days
- Date: 2025-02-07 / 2024-02-23
- Confidence: 高

#### A7. 变现立场与关键决策

**Claim A7-1**：无内购/无季票/无广告/无付费 DLC 的立场（2025-06 社交帖原话）："老实说，我不在 Balatro 里放微交易/季票/广告/100 个 DLC，不只是因为伦理——而是当我玩的其他游戏有这些东西时，我想把电脑扔进洗碗机开强力洗档。""我理解人们为什么加这些，但如果玩家的第一印象被一堆不属于游戏的玩意儿轰炸，你就是在自毁 UX。" 所有联动内容（Friends of Jimbo：巫师3、Among Us、Dave the Diver、吸血鬼幸存者等）全部免费。2024-08 他向 PC Gamer 透露已立遗嘱：Balatro IP 死后也不得卖给赌博公司或赌场。[^31^][^32^][^33^]
- Source: GoNintendo 转引 LocalThunk 原帖；PC Gamer（经 Wikipedia 引）；ithome
- URL: https://gonintendo.com/contents/49240-balatro-dev-explains-why-the-game-doesn-t-have-microtransactions ; https://www.ithome.com/0/793/805.htm
- Date: 2025-06-04 / 2024-09-05
- Confidence: 高

**Claim A7-2**：PEGI 危机：游戏初评 3+，2024 年发售前后被 PEGI 无预警上调至 18+（"显著赌博意象""教授扑克技能"），导致部分欧洲数字商店临时下架（2024-03 初）；LocalThunk 公开抨击双重标准（含真开箱的 EA Sports FC 仅 3+："如果那些游戏被正确评级，我乐意接受这个奇怪的 18+，红色标志还挺酷"）；Playstack/Sold Out 上诉后，2025-02-24 PEGI 改判 12+（"roguelike 牌组构筑包含缓解性的幻想元素"），并承诺细化赌博主题分级标准。LocalThunk："这是 PEGI 好的一步……希望这个改变能让开发者不再被不公平惩罚。"[^34^][^35^][^36^]
- Source: GameDeveloper；Kotaku；allkeyshop
- URL: https://www.gamedeveloper.com/business/balatro-s-contentious-pegi-18-rating-has-been-amended-thanks-to-mitigating-fantastical-elements- ; https://kotaku.com/balatro-18-pegi-rating-wrong-changed-12-esrb-europe-1851766176
- Date: 2025-02-24 / 2025-07-31
- Confidence: 高

**Claim A7-3**：找发行商的决策：曾"强烈考虑单干"，但自知需要帮助处理"所有对外事务、营销、可能的移植"；雇律师谈判、咨询曾与 Playstack 合作的工作室后签约；"他们能替代我 IT 工作的薪水，所以这就是我的工作"。签约时也"压力巨大，不习惯和这么多人打交道"。[^1^]
- Source: The Balatro Timeline
- Confidence: 高（一手）

---

### B. Eric Barone / Stardew Valley

#### B1. 背景与动机

**Claim B1-1**：Eric Barone，1987-12-03 生于洛杉矶，在华盛顿州 Auburn 长大；2011 年毕业于华盛顿大学塔科马分校（UW Tacoma）计算机科学专业；毕业后求职碰壁（"面试了几家都没拿到 offer"），在西雅图派拉蒙剧院做晚间引座员兼职。2012 年开始做 Stardew Valley（原名 Sprout Valley），动机有二：作为编程/游戏设计技能练习以改善就业前景（练手作品集）；作为 Harvest Moon（牧场物语）老粉，认为"没有一作把所有要素完美整合"，想做"和 Harvest Moon 有同样魔力但更有深度和自由度"的游戏。[^37^][^38^][^39^][^40^]
- Source: Wikipedia (Eric Barone / Stardew Valley)；GameDeveloper 2016；activeplayer 引言
- URL: https://en.wikipedia.org/wiki/Eric_Barone ; https://www.gamedeveloper.com/business/indie-rpg-hit-i-stardew-valley-i-sells-over-a-million
- Date: 2016-03/04（采访）
- Excerpt: "My experiences playing Harvest Moon are priceless and have had a deep impact on me, and I wanted my own game to have that kind of power."
- Confidence: 高

#### B2. 时间线与销量

**Claim B2-1**：开发期 2012–2016，约 4.5 年（官网十周年文称"我在这游戏上投入了近 15 年"）；Chucklefish 在开发中途主动找上门签约发行（负责商务、营销、本地化外包与主机/移动移植外包）；2016-02-26 PC 发售；前 2 周 Steam+GOG 售出约 42.5 万份；不到 2 个月破 100 万（精确数 1,007,000，Chucklefish 向 Polygon 确认）；2016 年为 Steam 收入前 24 游戏；2017 年底超 350 万（全平台）；2020-01 达 1,000 万；2021-09 达 1,500 万；2022-03 达 2,000 万（PC 1,300 万）；2024-02 达 3,000 万；2024-12 达 4,100 万（PC 2,600 万、Switch 790 万）；2026-02-26（发售十周年）官宣超 5,000 万份，"没有放缓迹象"。[^41^][^42^][^43^][^44^][^45^][^46^]
- Source: Wikipedia (Stardew Valley)；GameDeveloper 2016；GameDeveloper 2022/2024；stardewvalley.net 十周年官方文
- URL: https://en.wikipedia.org/wiki/Stardew_Valley ; https://www.gamedeveloper.com/business/stardew-valley-sales-grew-to-over-41-million-by-2024-s-end ; https://www.stardewvalley.net/stardew-valley-10-year-anniversary/
- Date: 2016-04-13 / 2024-12-30 / 2026-02-26
- Excerpt: "It's sold 50 million copies with no sign of slowing down."（官网，2026-02-26）
- Confidence: 高（里程碑均为官方/发行商口径）

**Claim B2-2**：收回发行权（逐步自发）：2018-11 底宣布、2018-12-14 起自发 PC/Mac/Linux/PS4/Xbox/PSVita（理由："准备好独立前行"，自发是独立开发者的"梦想"）；2019-10-01 收回 Switch；2021-12 收回 iOS；2022-03-17 收回 Android——至此全平台自发。2019-08 Chucklefish 因《Starbound》无偿用工指控陷入争议，Barone 发布官方声明划清界限："4.5 年开发中我是唯一在 Stardew Valley 上工作的人……Chucklefish 仅担任发行商"（唯一例外：多人更新的网络代码由一名 Chucklefish 员工编写）。[^47^][^48^][^49^][^50^]
- Source: PocketGamer.biz；GameDeveloper；ConcernedApe 官方声明（stardewvalley.net）
- URL: https://www.pocketgamer.biz/stardew-valley-creator-to-self-publish-on-android/ ; https://www.stardewvalley.net/clarification-on-the-relationship-between-chucklefish-and-stardew-valley/
- Date: 2019-08（声明）/ 2022-03-17
- Confidence: 高

**Claim B2-3**：更新与现状：1.6 大更新 2024-03-19（PC）/2024-11-04（全平台）发布，Steam 同时在线创 236,614 纪录；2025-07 Stardew Valley 超越《传送门2》成为 Steam250 史上评分最高游戏（8.87 vs 8.85；好评率约 98%、评论破百万）；2025-08-30 西雅图"Symphony of the Seasons"音乐会上宣布 1.7 更新（"没有日期，没有预估，但它会做"）；2025-09-12 任天堂直面会公布 Switch 2 版（老玩家免费升级，日本区依法收最低价）；2025-12-17 透露 1.7 内容（更多角色/社交内容、新农场类型、"还有很多不想剧透"）；2026-02 再透露 1.7 新增 Clint 和 Sandy 两位可结婚对象、孩子会"更有趣"；同期表示"愿意有一天做 Stardew Valley 2——全新角色、全新世界"。[^51^][^52^][^53^][^54^][^55^]
- Source: Nintendo Life；KitGuru；comicbook/insider-gaming；inara.cz 汇总 PC Gamer 报道；Steam250
- URL: https://www.nintendolife.com/news/2025/09/stardew-valley-creator-announces-version-1-7-update ; https://comicbook.com/gaming/news/first-details-for-stardew-valley-1-7-update-confirmed/ ; https://steam250.com/
- Date: 2025-08-30 ~ 2026-02
- Confidence: 高

**Claim B2-4**：Haunted Chocolatier：2020 年底立项、2021-10 公布，完全单人开发；2024 年因 Stardew 1.6 暂停；2024-12 宣布复工；2025-09-02 博客称本人重心在 Haunted Chocolatier，"甚至可能早于 Stardew 1.7 发售"，但无发售窗口；2026 年 1 月底发博客辟谣（否认"2030 年发售"传闻、否认沦为 Stardew 资料片），称近期效率很高、"做完就发"。[^56^][^57^][^55^]
- Source: GameSpot 汇总；notebookcheck；RPS（经 inara 引）
- URL: https://www.gamespot.com/articles/haunted-chocolatier-release-date-trailer-news/1100-6535251/ ; https://www.notebookcheck.net/After-a-long-silence-Stardew-Valley-developer-shares-an-update-on-his-next-game.1217866.0.html
- Date: 2025-10-07 / 2026-02-02
- Confidence: 高

#### B3. 收入

**Claim B3-1**：定价 $14.99（常有折扣，手游 $4.99/$5）。总收入无官方数字：按 41M 份 × $15 粗算上限约 6.15 亿美元（2024 末）；第三方 Boxleiter 法估算 Steam 单平台毛收入约 5.8 亿美元、开发者净得约 1.71 亿美元（steam-revenue-calculator）；另一计算器估 Steam 净得 4.41 亿美元（区间 3.08–7 亿，未计折扣/区域价/退款，仅供量级参考）。2016 年首年 Steam 收入约 2,400 万美元（当年 Steam 第 16 畅销）。2017 年报道"至少为 Eric 赚到 3,400 万美元"。净资产媒体估计从 4,500 万到 1 亿美元不等（均为猜测）。自发后他保留绝大部分收入（无发行商分成，仅平台抽成）。[^58^][^59^][^60^][^61^]
- Source: anscamobile/networthlist（低权威估算）；steam-revenue-calculator；steampageanalyzer；hjr265 引 Hypernia；ncesc
- URL: https://steam-revenue-calculator.com/app/413150/stardew-valley ; https://www.steampageanalyzer.com/tools/revenue-calculator/413150
- Date: 2025-01 ~ 2026-08
- Confidence: 中（均为第三方估算；官方仅有销量）

#### B4. 作息与支持系统

**Claim B4-1**：4.5 年开发期平均每天 10 小时、每周 7 天（GQ 描述为 12 小时/天）；发售后初期自述每天约 15 小时。期间部分时间与父母同住省钱，后与女友 Amber Hageman 搬到西雅图 Capitol Hill——女友打两份工供养两人，他兼职剧院引座员。2016 年 Gamasutra 专访标题即《The 4 years of self-imposed crunch that went into Stardew Valley》（自我强加的 crunch）。发售后他对 bug 极度负责："如果有人崩溃或遇到 bug，我都觉得是我个人的责任，沉重地压在我身上。"[^62^][^38^][^63^][^64^]
- Source: GQ《Valley Forged》(2018-03-20)；popculturalprecursors 引 2016 访谈；rpgwatch 转 2016 访谈；Wikipedia
- URL: https://www.resetera.com/threads/valley-forged-how-one-man-made-the-indie-video-game-sensation-stardew-valley.30951/ ; https://www.popculturalprecursors.com/p/the-origin-story-of-stardew-valley
- Date: 2016-03 / 2018-03-20
- Excerpt: "On average, I probably worked on it 10 hours a day every day of the week during development. Now that the game is out, I'm probably spending more like 15 hours a day on it."
- Confidence: 高

#### B5. 技术栈

**Claim B5-1**：C# + 微软 XNA 框架（Visual Studio 2010，自写引擎、不用现成引擎），美术用 Paint.NET，音乐/音效用 Propellerhead Reason 制作——代码、美术、音乐、写作、动画全部一人完成。2021 年从已停更的 XNA 迁移到开源 MonoGame（"让游戏面向未来，并让 mod 能使用 4GB 以上内存"）。2025-12-30 向 MonoGame 基金会捐赠 12.5 万美元并承诺每月持续捐助（$1,250/月），基金会称其为"非凡的支持"。另：他开发全程没用版本控制，只定期整包备份。[^65^][^66^][^67^][^45^]
- Source: 德文 Stardew Wiki（技术栈）；80.lv/gigazine；MonoGame 官方公告；官网十周年文
- URL: https://de.stardewvalleywiki.com/Stardew_Valley ; https://80.lv/articles/stardew-valley-creator-donated-usd125-000-to-monogame-c-framework-which-holds-his-game-together ; https://www.stardewvalley.net/stardew-valley-10-year-anniversary/
- Date: 2025-12-30 / 2026-02-26
- Confidence: 高

#### B6. 获客与社区

**Claim B6-1**：冷启动靠 Harvest Moon 社区与开发日志：2012 年起在 stardewvalley.blogspot.com 和官网持续发开发日志，在牧场物语粉丝社区积累首批关注；Chucklefish 负责营销与媒体；发售前后靠 Reddit/Twitter 与口碑扩散，Steam 首周即近 50 万份；后期 Twitch/YouTube 主播与 mod 社区（Nexus）持续引流。2021 年访谈："我不喜欢对 Stardew 做任何确定的承诺，因为人们会拿我的话当真。"[^68^][^43^][^69^]
- Source: 中文 Stardew Wiki（采访列表）；Wikipedia；TechRaptor 5 周年专访
- URL: https://techraptor.net/gaming/features/stardew-valley-5th-anniversary-concernedape-interview
- Date: 2021-02-26
- Confidence: 中高（"Harvest Moon 社区冷启动"为公认叙事，日志网址有一手佐证）

#### B7. 变现哲学与决策

**Claim B7-1**：免费更新誓言（2024-07-22 X 帖原话）："我以家族姓氏的名誉起誓，有生之年绝不对 DLC 或更新收费。截图吧，如果我违背誓言就羞辱我。" 此前 1.5 更新的 Ginger Island 本可做付费资料片/独立游戏，"但我还是把它用在了免费更新里"。游戏无内购、无微交易。[^70^][^69^]
- Source: NintendoDojo 引 ConcernedApe X；TechRaptor
- URL: https://www.nintendojo.com/news/eric-barone-pledges-that-stardew-valley-updates-will-always-be-free
- Date: 2024-07-22
- Confidence: 高

**Claim B7-2**：为何不雇人/保持小团队：2019 年 USgamer 访谈——他偏好独自做事，可以"纠结小细节（fuss over the little details）"而不用顾忌别人；希望未来游戏也自发；2019 年初仅为后续更新组建小团队（1.4 与 Arthur Lee、Alex Erlandson 合作），"没有计划"扩大规模。给独立开发者的建议（2022 AMA）："做你热爱的游戏，让你兴奋到愿意跨越任何障碍去完成它的游戏。用什么软件、什么方法并不重要。有动力，你自会找到路。"[^71^][^48^][^72^]
- Source: Destructoid 转 USgamer (2019-10-08)；GameByte 转 2022-03 AMA
- URL: https://www.destructoid.com/stardew-valley-creator-calls-upcoming-version-1-4-the-everything-update-aims-to-add-massive-changes-throughout-the-game/
- Date: 2019-10-09 / 2022-03-18
- Confidence: 高

---

### C. 独立游戏 Solo 开发成功率（彩票型模式佐证）

**Claim C-1**：Steam 新游供给量：2023 年 14,030 款 → 2024 年 18,474~18,825 款（SteamDB，另有媒体口径 18,825）→ 2025 年约 19,000–21,331 款（SteamDB 21,331 / GameDiscoverCo 20,558 / Gamalytic 20,353 / VG Insights 20,282，口径差异来自下架边缘游戏处理）。2025 年发布游戏中近半数（约 9,300 款）评测数少于 10 条（其中约 2,200 款零评测），仅约 1,200 款（6.2%）超过 500 条评测；2024 年同结构：43.56% 不足 10 评，7.2% 超 500 评。[^73^][^74^][^75^]
- Source: SteamDB releases 统计页；80.lv 引 SteamDB；howtomarketagame (Simon Carless)
- URL: https://steamdb.info/stats/releases/ ; https://80.lv/articles/steam-earned-usd16b-in-2025-but-nearly-half-of-19-000-games-got-under-10-reviews ; https://howtomarketagame.com/2026/01/08/how-many-games-were-released-in-2025/
- Date: 2025-12-15 / 2026-01-08
- Confidence: 高

**Claim C-2**：收入结构（VG Insights 2024 报告，经 Game World Observer）：2024 前 9 个月独立游戏 Steam 毛收入约 40 亿美元、占全平台完整游戏收入 48%；独立游戏占当年 13,007 款新品的 98.9%；"hobbyist"层级（1–2 人团队）典型表现为 2,000–20,000 份销量 / 5 万美元收入；Black Myth: Wukong 一款（10 亿美元）超过其余所有 2024 独立新作总和。Reddit 用户 IndiegameJordan 基于 GameDiscoverCo/Gamalytic 的 2024 分析：约 1.8 万款新游中仅 5,773 款收入 ≥500 美元；独立游戏 84.98% 的收入集中在头部 10% 作品；有发行商的游戏收入中位数 $16,222，自发的仅 $3,285。[^76^][^77^]
- Source: Game World Observer (2024-10-16)；questgamedev 转述分析 (2025-02-21)
- URL: https://gameworldobserver.com/2024/10/16/indie-games-revenue-steam-vs-aaa-titles-vg-insights ; https://www.questgamedev.com/post/indie-game-revenue-on-steam-key-insights-from-2024-data-analysis
- Confidence: 中高

**Claim C-3**：对照案例：同为"一人爆款"的还有 Vampire Survivors（失业期间开发，美术音乐资产包仅花约 1,100 英镑）、Lethal Company（Zeekerss，数月估销千万份）、Animal Well（Billy Basso 单人 7 年含自研引擎）、Undertale、Minecraft 初期——均属极端 outliers。[^78^]
- Source: overbaked.studio 微型工作室综述（2026-08-03）
- URL: https://overbaked.studio/blog/rise-of-micro-studio-games/
- Confidence: 中（综述二手，各数字本身有原始出处）

---

## 第二部分：来源列表

[^1^]: LocalThunk, "The Balatro Timeline", 2025-03-06. https://localthunk.com/blog/balatro-timeline-3aarh
[^2^]: Rogueliker 采访 LocalThunk（中译，旅法师营地）, 2024-03-07. https://www.iyingdi.com/tz/post/5358091
[^3^]: 80 Level, "LocalThunk Shares The Most Important Inspiration For Balatro", 2025-02-28. https://80.lv/articles/localthunk-shares-the-most-important-inspiration-of-balatro
[^4^]: GameDeveloper, "LocalThunk knew Balatro needed to draw players in with poker"（NoClip 播客）, 2024-05-02. https://www.gamedeveloper.com/business/localthunk-knew-balatro-needed-to-draw-players-in-with-poker
[^5^]: GoNintendo 转 PC Gamer 对 Playstack 公关总监 Wout van Halderen 采访, 2025-03-20. https://www.gonintendo.com/contents/46492-balatro-publisher-on-localthunk-s-decision-to-remain-anonymous
[^6^]: The Logic, "How an IT worker from Saskatchewan made the best game of the year", 2025-02-06. https://thelogic.co/news/balatro-developer-interview/
[^7^]: ixbt.games 转述 LocalThunk 博客《Bad Grades》, 2026-02-24. https://ixbt.games/en/news/2026/02/24/sozdatel-balatro-podtverdil-rabotu-nad-versiei-11.html
[^8^]: GameDeveloper, "LocalThunk's Balatro sells 250,000 copies in first three days", 2024-02-23. https://www.gamedeveloper.com/business/localthunk-s-balatro-sells-250-000-copies-in-first-three-days
[^9^]: Game World Observer, "Balatro sells over 1 million copies", 2024-03-18. https://gameworldobserver.com/2024/03/18/balatro-sales-1-million-copies-first-month-localthunk
[^10^]: PCGamesInsider, "Balatro has hit five million sales", 2025-01-22. https://www.pcgamesinsider.biz/news/74922/balatro-has-hit-five-million-sales/
[^11^]: GameSpot, "Balatro Hits Big Sales Milestone After Being Showered With Awards", 2025-01-21. https://www.gamespot.com/articles/balatro-hits-big-sales-milestone-after-being-showered-with-awards/1100-6528912/
[^12^]: gaming.news, "Balatro Wins Game of the Year at GDC Awards 2025", 2025-03-21. https://gaming.news/news/2025-03-20/balatro-wins-game-of-the-year-at-gdc-awards-2025/
[^13^]: IMDb, Balatro Awards 汇总. https://www.imdb.com/title/tt31521616/awards/
[^14^]: Wikiwand/Wikipedia, Balatro 条目. https://www.wikiwand.com/en/articles/Balatro_(video_game)
[^15^]: GoNintendo 全文转载 LocalThunk 博客《I'm Slow》, 2025-09-12. https://gonintendo.com/contents/52812-balatro-1-1-update-delayed-to-2026
[^16^]: PocketGamer.biz, "LocalThunk delays Balatro update to avoid crunch and burnout", 2025-09-16. https://www.pocketgamer.biz/localthunk-delays-balatro-update-to-avoid-crunch-and-burnout/
[^17^]: The Verge, "Balatro's developer is 'still working' on the game's big 1.1 update", 2026-02-23. https://www.theverge.com/games/883290/balatros-developer-is-still-working-on-the-games-big-1-1-update
[^18^]: Balatro HQ, "Balatro Update 2026: What Changed and What We Know About 1.1", 2026-07-15. https://balatrohq.com/news/patches/
[^19^]: TechTimes, "Balatro Publisher Playstack Acquired by GameSpot and Fandom Parent IMC for $169M", 2026-05-31. https://www.techtimes.com/articles/317459/20260531/balatro-publisher-playstack-acquired-gamespot-fandom-parent-imc-169m.htm
[^20^]: GameDeveloper, "Balatro made $1 million within hours of release", 2024-02-27. https://www.gamedeveloper.com/business/balatro-made-1-million-within-hours-of-release
[^21^]: Game World Observer, "Balatro reached profitability in just 1 hour", 2024-02-28. https://gameworldobserver.com/2024/02/28/balatro-profitable-1-hour-gross-revenue-over-1-million
[^22^]: TouchArcade, "'Balatro' Is Coming to Apple Arcade and Also iOS…September 26th", 2024-09-05. https://toucharcade.com/2024/09/05/balatro-mobile-release-date-price-download-apple-arcade/
[^23^]: PocketGamer.biz, "The Top 50 Mobile Game Makers of 2025", 2025-10-07. https://www.pocketgamer.biz/the-top-50-mobile-game-makers-of-2025/
[^24^]: app2top 转 AppMagic 手游收入数据, 2025-04-09. https://app2top.com/news/publisher-balatro-advised-other-publishers-to-invest-more-in-indie-teams-and-not-to-expect-huge-profits-from-them-279481.html
[^25^]: PocketGamer.biz, "Playstack revenue surged 52% in H1 2025 to £30.7m", 2025-09-19. https://www.pocketgamer.biz/playstack-revenue-surged-52-in-h1-2025-to-307m/
[^26^]: GameDeveloper, "Balatro publisher Playstack has a 'hit ratio' of more than 85 percent"（TruFin 年报）, 2026-03-19. http://www.gamasutra.com/business/balatro-publisher-playstack-has-a-hit-ratio-of-more-than-85-percent
[^27^]: Bloomberg (Jason Schreier), "Maker of Smash Indie Hit 'Balatro' Talks About What's Next", 2025-02-07（经 IconEra 转引）. https://icon-era.com/threads/bloomberg-interview-with-balatro-developer-localthunk.15875/
[^28^]: Generalist Programmer, "LÖVE (Love2D) vs Godot"（引擎属性佐证）, 2026-07-30. https://generalistprogrammer.com/comparisons/love2d-vs-godot
[^29^]: GIGAZINE 转 brothers-in-gaming 营销分析, 2025-02-07. https://gigazine.net/gsc_news/en/20250207-balatro-successful-launch-strategy/
[^30^]: Josh Hardy, "From Hobby to Hit: The Incredible Journey of Balatro", 2025-03-08. https://www.joshhardy.co.uk/post/from-hobby-to-hit-the-incredible-journey-of-balatro-and-what-indie-devs-can-learn
[^31^]: GoNintendo 转 LocalThunk 社交帖（无微交易原话）, 2025-06-04. https://gonintendo.com/contents/49240-balatro-dev-explains-why-the-game-doesn-t-have-microtransactions
[^32^]: PC Gamer (Mollie Taylor), "LocalThunk is making sure casinos can't get their hands on his game even after he dies by literally writing it into his will", 2024-08-06（经 Wikipedia 引用）. https://mlei.co/a/wiki.php/Balatro_(video_game)?lang=en
[^33^]: IT之家, "《小丑牌》官宣 9 月 26 日登陆 iOS/安卓，售 9.99 美元"（含遗嘱条款中文报道）, 2024-09-05. https://www.ithome.com/0/793/805.htm
[^34^]: GameDeveloper, "Balatro's contentious PEGI 18 rating has been amended", 2025-02-24. https://www.gamedeveloper.com/business/balatro-s-contentious-pegi-18-rating-has-been-amended-thanks-to-mitigating-fantastical-elements-
[^35^]: Kotaku, "Balatro Is No Longer Rated 18+ In Europe", 2025-07-31. https://kotaku.com/balatro-18-pegi-rating-wrong-changed-12-esrb-europe-1851766176
[^36^]: AllKeyShop Pixel Sundays（PEGI 下架时间线汇总）, 2025-04-13. https://www.allkeyshop.com/blog/pixel-sundays-balatro-news-k/
[^37^]: Wikipedia, "Eric Barone". https://en.wikipedia.org/wiki/Eric_Barone
[^38^]: GameDeveloper, "Indie RPG hit Stardew Valley sells over a million"（引 2016-03 Gamasutra 专访）, 2016-04-13. https://www.gamedeveloper.com/business/indie-rpg-hit-i-stardew-valley-i-sells-over-a-million
[^39^]: ActivePlayer 引 Barone 原话（Harvest Moon 动机）, 2025-04-09. https://activeplayer.io/stardew-valley/
[^40^]: Cavacopedia 转 Wikipedia 开发段落（毕业求职失败、剧院引座员）, 2025-11-02. https://cavac.at/cavacopedia/Stardew%20Valley
[^41^]: Wikipedia, "Stardew Valley". https://en.wikipedia.org/wiki/Stardew_Valley
[^42^]: IBTimes, "PC hit Stardew Valley tops 1 million downloads less [than] two months"（Chucklefish 确认 1,007,000）, 2016-04. https://www.ibtimes.co.uk/pc-hit-stardew-valley-tops-1-million-downloads-less-two-months-1554850
[^43^]: GameDeveloper, "Stardew Valley has topped 41 million lifetime sales", 2024-12-30. https://www.gamedeveloper.com/business/stardew-valley-sales-grew-to-over-41-million-by-2024-s-end
[^44^]: KitGuru, "Stardew Valley has sold over 15 million copies", 2021-09-09. https://www.kitguru.net/gaming/mustafa-mahmoud/stardew-valley-has-sold-over-15-million-copies/
[^45^]: ConcernedApe 官网, "Stardew Valley 10-year Anniversary", 2026-02-26. https://www.stardewvalley.net/stardew-valley-10-year-anniversary/
[^46^]: Udonis Blog, "Stardew Valley Sales Numbers 2016–2026"（里程碑汇总）, 2026-02-17. https://www.blog.udonis.co/mobile-marketing/mobile-games/stardew-valley-sales
[^47^]: PocketGamer.biz, "Stardew Valley creator to self-publish on Android", 2022-03-18. https://www.pocketgamer.biz/stardew-valley-creator-to-self-publish-on-android/
[^48^]: Game World Observer, "Eric Barone now finally self-publishing Stardew Valley on all platforms", 2022-03-18. https://gameworldobserver.com/2022/03/18/eric-barone-now-finally-self-publishing-stardew-valley-on-all-platforms-including-mobile
[^49^]: GameDeveloper, "Chucklefish returns final Stardew Valley publishing duties to ConcernedApe", 2022-03-17. https://www.gamedeveloper.com/business/chucklefish-returns-final-stardew-valley-publishing-duties-to-concernedape
[^50^]: ConcernedApe 官网, "Clarification on the relationship between Chucklefish and Stardew Valley", 2019-08. https://www.stardewvalley.net/clarification-on-the-relationship-between-chucklefish-and-stardew-valley/
[^51^]: Nintendo Life, "Stardew Valley Creator Announces Version 1.7 Update", 2025-09-01/23. https://www.nintendolife.com/news/2025/09/stardew-valley-creator-announces-version-1-7-update
[^52^]: Notebookcheck, "Stardew Valley's Nintendo Switch 2 upgrade is free for existing owners", 2025-09-14. https://www.notebookcheck.net/Stardew-Valley-s-Nintendo-Switch-2-upgrade-is-free-for-existing-owners.1114402.0.html
[^53^]: ComicBook, "First Details for Stardew Valley 1.7 Confirmed", 2025-12-18. https://comicbook.com/gaming/news/first-details-for-stardew-valley-1-7-update-confirmed/
[^54^]: 80 Level / ResetEra, "Stardew Valley Overtakes Portal 2 as Steam's Best-Rated Game", 2025-07-09. https://80.lv/articles/stardew-valley-overtakes-portal-2-as-steam-s-top-rated-game
[^55^]: Inara.cz 汇总 PC Gamer/RPS 2026 年报道（1.7 结婚对象、Stardew 2 意向、HC 辟谣）, 2026-02. https://inara.cz/morps/article/42954/
[^56^]: GameSpot, "Haunted Chocolatier Release Date, Trailers, Gameplay, And Everything Else We Know", 2025-10-07. https://www.gamespot.com/articles/haunted-chocolatier-release-date-trailer-news/1100-6535251/
[^57^]: Notebookcheck, "After a long silence: Stardew Valley developer shares an update on his next game", 2026-02-02. https://www.notebookcheck.net/After-a-long-silence-Stardew-Valley-developer-shares-an-update-on-his-next-game.1217866.0.html
[^58^]: Steam Revenue Calculator（Boxleiter 估算）, 访问于 2026-08. https://steam-revenue-calculator.com/app/413150/stardew-valley
[^59^]: SteamPageAnalyzer 收入计算器（估 Steam 净得 $441M，区间 $308–699M）. https://www.steampageanalyzer.com/tools/revenue-calculator/413150
[^60^]: hjr265 引 Hypernia（2016 年 Steam 收入 $24M）, 2023-10-10. https://hjr265.me/blog/stardew-valley-and-eric-barone/
[^61^]: ncesc（2017 年"至少 $34M"说法）, 访问于 2026-08. https://www.ncesc.com/gaming-faq/did-eric-barone-make-stardew-valley-alone/
[^62^]: GQ, "Valley Forged: How One Man Made the Indie Video Game Sensation Stardew Valley", 2018-03-20（经 ResetEra 转引）. https://www.resetera.com/threads/valley-forged-how-one-man-made-the-indie-video-game-sensation-stardew-valley.30951/
[^63^]: Pop Cultural Precursors, "The origin story of Stardew Valley"（10 小时/天原话）, 2026-04-19. https://www.popculturalprecursors.com/p/the-origin-story-of-stardew-valley
[^64^]: RPGWatch 转 2016-03 Eric Barone 访谈（对 bug 的个人责任感）, 2016-03-11. https://rpgwatch.com/forum/threads/stardew-valley-interview-with-eric-barone.32837/
[^65^]: 德文 Stardew Wiki（C#/XNA/MonoGame/Paint.NET/Reason 技术栈）. https://de.stardewvalleywiki.com/Stardew_Valley
[^66^]: 80 Level, "Stardew Valley Creator Donated $125,000 to MonoGame C# Framework", 2026-01-05. https://80.lv/articles/stardew-valley-creator-donated-usd125-000-to-monogame-c-framework-which-holds-his-game-together
[^67^]: GIGAZINE, "Stardew Valley developer donates $125,000 to MonoGame", 2026-01-08. https://gigazine.net/gsc_news/en/20260108-stardew-valley-donate-monogame/
[^68^]: 星露谷物语中文百科 ConcernedApe 词条（2012 年起采访/日志列表）, 2026-02-10. https://wiki.biligame.com/stardewvalley/ConcernedApe
[^69^]: TechRaptor, "Spring of Year 5: Reflecting on Stardew Valley with Eric 'ConcernedApe' Barone", 2021-02-26. https://techraptor.net/gaming/features/stardew-valley-5th-anniversary-concernedape-interview
[^70^]: NintendoDojo, "Eric Barone Pledges That Stardew Valley Updates Will Always Be Free", 2024-07-22. https://www.nintendojo.com/news/eric-barone-pledges-that-stardew-valley-updates-will-always-be-free
[^71^]: Destructoid 转 USgamer 专访, 2019-10-09. https://www.destructoid.com/stardew-valley-creator-calls-upcoming-version-1-4-the-everything-update-aims-to-add-massive-changes-throughout-the-game/
[^72^]: GameByte 转 2022-03 AMA（"做你热爱的游戏"建议）, 2022-03-18. https://www.gamebyte.com/chucklefish-hands-all-stardew-valley-publishing-rights-back-to-concernedape/
[^73^]: SteamDB, "Steam Game Release Summary by Year", 访问于 2026-08. https://steamdb.info/stats/releases/
[^74^]: 80 Level, "Steam Earned $16B+ in 2025, But Nearly Half of 19,000 Games Got Under 10 Reviews", 2025-12-15. https://80.lv/articles/steam-earned-usd16b-in-2025-but-nearly-half-of-19-000-games-got-under-10-reviews
[^75^]: How To Market A Game (Simon Carless), "How many games were released in 2025?", 2026-01-08. https://howtomarketagame.com/2026/01/08/how-many-games-were-released-in-2025/
[^76^]: Game World Observer, "Indie games come close to AA/AAA games in revenue on Steam…(VG Insights)", 2024-10-16. https://gameworldobserver.com/2024/10/16/indie-games-revenue-steam-vs-aaa-titles-vg-insights
[^77^]: QuestGameDev, "Indie Game Revenue on Steam: Key Insights from 2024 Data Analysis", 2025-02-21. https://www.questgamedev.com/post/indie-game-revenue-on-steam-key-insights-from-2024-data-analysis
[^78^]: Overbaked Studio, "How Do Tiny Teams Make Huge Games? The Micro-Studio Era", 2026-08-03. https://overbaked.studio/blog/rise-of-micro-studio-games/
[^79^]: PocketGamer.biz, "How Playstack bet on Balatro and won big", 2025-03-25. https://www.pocketgamer.biz/how-playstack-bet-on-balatro-and-won-big/
[^80^]: ComicBook, "3 Massively Successful Games No One Saw Coming"（LocalThunk 预计只卖 10 份）, 2026-03-18. https://comicbook.com/gaming/list/3-massively-successful-games-no-one-saw-coming/

---

## 第三部分：写作素材摘要（按 11 组整理）

### 案例一：LocalThunk / Balatro（小丑牌）

**1. 背景**：加拿大萨斯喀彻温省匿名 IT 从业者，大学最后一年从工程退学、计算机课成绩差，但十年间夜里写小游戏自娱，作品只发给朋友——"做游戏是爱好，发售赚钱不是"。2021 年底用 IT 工作攒的 3 周年假开工，最初想做的是和朋友联机玩自刨牌戏"Big Cheat"（基于广东大老二）。匿名是"随便做的决定"，事后极其庆幸；发行商证实他"不是 Banksy 行为艺术，只想安静做游戏"，代价是"有点孤独"。2026 年仍匿名。[^1^][^5^][^6^][^7^]

**2. 时间线**：2021-12-13 开工（Learning/Lua 文件夹）→ 2022 全年晚上+周末开发（3 月曾完全停工）→ 2023-01 辞职全职 → 2023-05 末 Beta 上线仅 48 愿望单 → 2023-06 主播发现+Playstack 星探私信 → 2023-07 签约、Northernlion 直播 → 2023-09/10 新 Demo+Next Fest → 2024-02-20 全平台发售（愿望单 20.8 万）→ 首日 Steam 11.9 万份 → 3 天 25 万 → 10 天 50 万 → 首月 100 万 → 2024-09-26 手游（$9.99+Apple Arcade）→ 2024-12 初 350 万 → 2024-12-12 TGA 三奖+GOTY 提名（史上首个单人开发 GOTY 提名）→ 2025-01-21 达 500 万 → 2025-03 GDC 年度游戏 → 2025-09-12 宣布 1.1 延期至"做完就发" → 2026-02-20 确认仍在做 1.1；2026 年上新 Switch 2（2-25）与 Epic（6-30）；2026-06 发行商 Playstack 以约 1.69 亿美元估值被 IMC 收购。[^1^][^8^][^9^][^10^][^15^][^18^][^19^]

**3. 收入**：$14.99（PC/主机）/$9.99（手游）；发售 1 小时回本、8 小时毛收入破 100 万美元（Playstack 史上最快）；首日几小时内 Steam 收入超 60 万美元——"比我一辈子赚过的钱都多"。手游累计净收入超 2,500 万美元（2025-10）。5M 份×$14.99 名义上限约 7,500 万美元（未扣折扣/平台 30%/发行分成，条款未公开）。本人表态：和伴侣买了房，"生活方式没怎么变，我还在尽责，我还有工作要做"，自认幸运。[^1^][^20^][^21^][^23^][^27^]

**4. 现状（2026）**：仍匿名；以"每天几小时"的业余节奏独自开发 1.1（新 Joker、重做 Matador、调 Blue Stake；为做平衡亲自打到最高难度 100%）；无日期；发行商易主不影响开发。[^17^][^18^]

**5. 作息**：业余期=晚上+周末；2023 辞职后全职；发售前半年 crunch 每天 12 小时，伴严重失眠、心脏问题、惊恐发作（2024-01 就医）；2024 年底 burnout 停工数月；2025 起回归"hobbyist"节奏，"工作慢，但我喜欢这样……5 年后仍能健康兴奋地做游戏"是目标。[^1^][^15^]

**6. 技术栈**：LÖVE（Love2D，免费开源）+ Lua，沿用他为上一个项目自建的引擎代码；发售前现金成本≈Fiverr 配乐费（Luis Clemente）+100 美元 Steam 上架费；移植外包给 Love2D 专家 Maarten De Meyer。[^1^][^18^]

**7. 获客**：零主动营销（"没给任何人发过求玩邮件"）；Demo 两轮迭代+两次 Next Fest+Discord 社群+主播邀请赛；Dan Gheesling 与 Northernlion（2023-07-19 直播使 Steam 关注单周涨近 400%）是关键引爆点；Playstack 负责本地化、首日全平台移植与媒体评测分发（评分解禁即 90+ 造势）。愿望单 48→20.8 万（9 个月）。[^1^][^29^][^30^]

**8. 变现**：纯买断、无内购/季票/广告/付费 DLC；原话："其他游戏塞这些东西时我想把电脑扔进洗碗机开强力洗档"；所有联动（巫师3、Among Us 等）免费；1.1 也将全平台免费。[^31^]

**9. 决策**：保持匿名=减压；签发行商=换薪水+处理所有对外事务（雇律师谈判）；危机=PEGI 把 3+ 无预警上调 18+ 致部分欧洲商店下架（2024-03），他公开抨击双重标准，Playstack 上诉后 2025-02-24 改判 12+；并已立遗嘱：IP 死后也不得卖给赌博公司。[^1^][^31^][^32^][^34^][^35^]

**10. 效率密码**：一人写代码+美术+设计，规则小到商店页 10 秒讲清；音乐、移植、商务全外包；刻意不玩同类游戏保持设计新鲜感（"重新发明轮子也是乐趣"）；150 张 Joker 源于一次口误干脆加做 30 张。[^1^][^4^]

**11. 2026 可行性**：Steam 年上新约 2 万款，近半评测不足 10 条，仅约 6% 超 500 评；1–2 人"hobbyist"典型销量 2,000–20,000 份/收入 5 万美元；独立游戏 85% 收入集中在头部 10%。Balatro 本人预估"只会卖 10 份"——他本人都承认"lucked in"，属彩票型中奖，不可复制但可借鉴其低期望值+低成本+纯热爱结构。[^73^][^74^][^76^][^77^][^80^]

### 案例二：Eric Barone（ConcernedApe）/ Stardew Valley（星露谷物语）

**1. 背景**：1987 年生，UW Tacoma 计算机科学 2011 年毕业，求职屡屡被拒，在派拉蒙剧院做晚间引座员。2012 年开工，初衷是把游戏当编程练手作品集以改善就业；作为牧场物语老粉，"没有一作把一切完美整合"，想做"有同样魔力但更有深度与自由"的游戏。[^37^][^38^][^39^]

**2. 时间线**：2012 开工 → 4.5 年单人开发（代码/美术/音乐/写作全包）→ Chucklefish 中途主动签约 → 2016-02-26 PC 发售 → 2 周 42.5 万 → 2 个月 100.7 万 → 2017 底 350 万 → 2020-01 1,000 万 → 2021-09 1,500 万 → 2022-03 2,000 万 → 2024-02 3,000 万 → 2024-12 4,100 万（PC 2,600 万、Switch 790 万）→ 2026-02-26 十周年官宣超 5,000 万。发行权：2018-12 自发 PC/主机、2019-10 Switch、2021-12 iOS、2022-03 Android，完成全平台自发。2024-03 1.6 大更新（同时在线 23.66 万纪录）；2025-07 超越《传送门2》成 Steam 史上评分第一；2025-08-30 宣布 1.7；2025-12 Switch 2 版免费升级。[^41^][^42^][^43^][^45^][^47^][^51^][^54^]

**3. 收入**：$14.99；无官方总收入。粗算 41M 份×$15 上限约 6.15 亿美元；第三方估 Steam 单平台毛收入约 5.8 亿美元、净得 1.7–4.4 亿美元（口径差异大）。2016 首年 Steam 约 2,400 万美元。全平台自发后仅平台抽成，绝大部分归个人；净资产媒体猜测 $45M–$100M+。[^58^][^59^][^60^]

**4. 现状（2026）**：1.7 更新开发中（新农场类型、角色/社交内容、Clint 与 Sandy 成新可婚对象、孩子更有趣），无日期；Haunted Chocolatier 仍单人开发、"做完就发"、甚至可能早于 1.7 发售；公开表示"愿意有一天做 Stardew Valley 2"。2025-12 向 MonoGame 捐 12.5 万美元+每月 $1,250 持续捐助。[^51^][^53^][^55^][^56^][^66^]

**5. 作息**：4.5 年平均每天 10 小时、每周 7 天（GQ 称 12 小时/天）；发售后每天约 15 小时；"自我强加的 crunch"。靠女友 Amber Hageman 打两份工供养，他自己兼职引座员，曾与母亲同住省钱。对玩家 bug 有强烈个人责任感。[^62^][^63^][^64^]

**6. 技术栈**：C# + XNA（后于 2021 迁 MonoGame），VS2010 自写引擎；Paint.NET 画美术；Propellerhead Reason 作曲；全程不用版本控制，只定期整包备份。成本几乎为零。[^65^][^45^]

**7. 获客**：2012 年起开发日志+牧场物语社区冷启动；Chucklefish 做营销/移植/本地化；Reddit/Twitter+口碑；首周即近 50 万份；长尾靠免费大更新、主播、mod 社区（Nexus）与音乐会/食谱书等 IP 运营。[^68^][^69^]

**8. 变现**：买断制+永久免费大更新。2024-07-22 原话："我以家族姓氏名誉起誓，有生之年绝不对 DLC 或更新收费，截图作证。" 1.5 的姜岛本可做付费资料片，他选择免费送出。[^70^][^69^]

**9. 决策**：拒绝扩张团队——喜欢独自"纠结小细节"，仅 2019 年为后续更新组小团队；收回发行权=自发是独立开发者的梦想+2019 年与陷入《Starbound》无偿用工争议的 Chucklefish 切割（发官方声明强调"4.5 年只有我一个人做这个游戏"）。[^50^][^71^]

**10. 效率密码**：一个人掌握全部环节、无需沟通成本；技能全是边做边学；多次重写推倒重来；发售后十年持续免费更新，把单个产品的长尾做到极致。[^37^][^45^]

**11. 2026 可行性**：10 年 5,000 万份是行业孤例级 outlier（对照：Steam 独立游戏收入中位数自发仅约 $3,285）；其模式成立前提是"4.5 年无收入的豪赌+伴侣供养+恰好击中品类空白"，幸存者偏差极高——但"做自己想玩的游戏+免费更新养口碑"的策略在 2026 年仍是低预算独立开发者最可信的路径。[^77^][^78^]

---

## 附：矛盾点与注意事项（供写作时核对）

1. **Balatro 发售日**：官方与 Steam 为 2024-02-20（提前 15 分钟解锁）；个别价格站记为 02-19（时区差异）。
2. **手游收入口径不一**：AppMagic 净收入 2024-12 约 $420 万 / 2025-01 超 $900 万 / 2025-04 约 $1,497 万 / PocketGamer 2025-10 称累计超 $2,500 万——均为第三方估算。
3. **Stardew 开发起点**：多数权威来源记 2012 年开工、4.5 年；官网十周年文称"投入近 15 年"（含发售前 2011 年底起算与发售后 10 年）；维基另有"over five years"表述。
4. **Stardew 收入/净资产**：无官方数字，$45M/$100M 等均为媒体猜测；两个 Boxleiter 计算器净得估计相差近 3 倍（$171M vs $441M），只能作量级参考。
5. **Eric Barone 日常工时**：10 小时/天（2016 访谈自述）vs 12 小时/天（GQ 2018 描述），引用时建议用本人自述口径。
6. **Steam 年新游数**：2025 年各统计口径 19,000–21,331 不等（下架边缘游戏处理差异）。
7. **部分中文/二手来源**（如亿好科技网称 1.1"新增 15 张卡"等细节）与一手博客不符，未采信；1.1 内容以 Bloomberg 访谈（新 Joker/Matador/Blue Stake）为准。

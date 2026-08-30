# Market Research: ADHD-Friendly Planning App (Body Doubling + Task Breakdown + Gentle Accountability)

**Date:** 2026-08-30 · **Researcher:** Claude (market-research skill) · **Confidence:** Medium (strong qualitative evidence; market-size figures are low reliability)

## 1. Executive Summary

A consumer subscription app for adults with ADHD combining virtual body doubling, AI task breakdown, and shame-free accountability. Verdict: **PIVOT — 6.4/10**. The pain is real, frequent, and emotional; people already pay $8–40/month for partial solutions and complain loudly about all of them. The #1 opportunity: no incumbent bundles task breakdown + gentle accountability + flexible co-focus at an indie-friendly price — Tiimo plans but doesn't body-double, Focusmate body-doubles but doesn't plan, Flow Club costs $40/mo, goblin.tools is free but has zero accountability. The #1 risk: the target user's core symptom *is* abandoning apps after 3 days — retention is the whole game, and real-time body doubling needs two-sided liquidity that even Focusmate struggles to sustain.

## 2. Verdict

# PIVOT — 6.4/10

Pain intensity and demand trend are strong, but competition gap, monetization, and distribution all sit at "real but contested" (6s), and real-time body doubling is a marketplace cold-start problem ill-suited to a solo developer. **Pivot direction:** lead with the solo-buildable wedge — AI task breakdown + asynchronous/gentle accountability (voice-note check-ins, streak-free "return without shame" loops, optional small pre-scheduled co-focus rooms rather than on-demand stranger matching) — and treat live 1:1 body doubling as a later feature, not the core. Steelman against this verdict: one could argue GO, because the pain is genuinely intense, incumbents keep failing on retention and price, and the community is enormous and reachable; but the churn-prone audience plus a crowded field of funded competitors (Tiimo, Inflow) means an undifferentiated entry likely dies on retention, so a focused wedge is the honest recommendation.

### Scorecard

| Dimension | Weight | Score | Key evidence |
|---|---|---|---|
| Pain Intensity | 25% | 7/10 | Emotional, recurring threads across 5+ subreddits; hacky workarounds (whiteboards, 5 alarms, DM-ing strangers for co-work) |
| Demand Trend | 15% | 7/10 | Market reports CAGR ~11–16%; r/ADHD at 2.27M subs; fresh high-engagement threads through 2026 |
| Competition Gap | 20% | 6/10 | Every incumbent has real weaknesses (price hikes, no planning, no accountability, "half-finished"), but the space is crowded |
| Monetization | 15% | 6/10 | Proven $8–40/mo subscriptions; price complaints prove payment; but audience is price-sensitive and scam-burned |
| Market Size | 10% | 6/10 | SOM ~$120K–480K ARR at indie scale — meaningful, not venture-scale |
| Distribution | 15% | 6/10 | Huge reachable communities + app stores + long-tail SEO; but big subs restrict promotion and head keywords are listicle-owned |
| **Weighted total** | | **6.4/10** | |

## 3. The Problem & Who Has It

Adults with ADHD find mainstream todo apps (Todoist, Notion, Google Calendar) unusable: long lists trigger overwhelm, overdue counters trigger shame-driven abandonment, and setup friction kills adoption within days. Target user: English-speaking adults 20–45 with ADHD/AuDHD, employed or studying, already app-hopping. Geography: global English market, app-store distributed. Assumptions made: consumer/prosumer subscription, solo developer, no enterprise motion.

## 4. Pain Point Evidence

- "find a new productivity app, get hyped… use it for 3 days, then completely forget it exists" — r/ProductivityApps, 63 pts / 147 comments, 2026-05 ([source](https://reddit.com/r/ProductivityApps/comments/1t2m4jw/adhd_productivity_apps_endless_cycle_of/)). Same thread: "todoist (the red overdue counter gives me anxiety so i stop opening it)" — workarounds are a physical whiteboard and 5 alarms.
- "Best Productivity Apps for ADHD" — 590 pts / 119 comments, 2026-01, r/ProductivityApps ([source](https://reddit.com/r/ProductivityApps/comments/1qdc92u/best_productivity_apps_for_adhd/)) — sustained demand for recommendations.
- "Most of them sound great but then I stop using them… tell me what to do, help me breakdown tasks" — r/ADHDUK, 78 comments, 2026-01 ([source](https://reddit.com/r/ADHDUK/comments/1qkemzv/adhd_planners_apps_what_actually_work_for_you/)).
- "many of them feel built around pressure, streaks, or rigid routines — which often backfires" — r/AutisticWithADHD, 35 pts / 49 comments, 2026-01 ([source](https://reddit.com/r/AutisticWithADHD/comments/1qbr4vh/what_productivity_tools_or_apps_actually_work_for/)) — direct evidence for the "gentle" angle.
- "Body-doubling is SUCH a miracle" — r/adhdwomen, 82 pts / 42 comments, 2026-06 ([source](https://reddit.com/r/adhdwomen/comments/1u16521/bodydoubling_is_such_a_miracle/)).
- "Body doubling — want to try it but have no idea where to start" — r/ADHD, 54 comments, 2026-04 ([source](https://reddit.com/r/ADHD/comments/1scn98w/body_doubling_want_to_try_it_but_have_no_idea/)).
- "Have you found an app to hold you accountable… I'm desperate enough to look into it" — r/adhdwomen, 2025-09 ([source](https://reddit.com/r/adhdwomen/comments/1nar2kh/have_you_found_an_app_to_hold_you_accountable_and/)).
- "Focusmate is becoming increasingly frustrating — Rising prices, rule violations, and reliability issues" — r/productivity, 49 comments, 2025-10 ([source](https://reddit.com/r/productivity/comments/1ogy7zp/focusmate_is_becoming_increasingly_frustrating/)); also "Focusmate has no people?" r/adhdwomen, 2026-02 ([source](https://reddit.com/r/adhdwomen/comments/1rbdlsd/focusmate_has_no_people/)) — the liquidity problem, from users.
- "Scam targeting ADHD folks" (Fabulous app dark-pattern billing) — r/adhdwomen, **1514 pts** / 140 comments, 2026-01 ([source](https://reddit.com/r/adhdwomen/comments/1q65obz/scam_targeting_adhd_folks/)) — proves willingness to pay *and* deep distrust of subscription dark patterns.
- Users cobble together DIY co-working: "seeking a virtual co-focus partner… voice call every day, 2 hours daily" — r/productivity, 2026-04 ([source](https://reddit.com/r/productivity/comments/1skrtua/seeking_a_virtual_cofocus_partner/)).

## 5. Demand Signals

- Market reports peg the ADHD apps market at ~$0.6–3.2B in 2024–2026 with CAGR ~11–16% ([Strategic Market Research](https://www.strategicmarketresearch.com/market-report/adhd-apps-market), [Business Research Insights](https://www.businessresearchinsights.com/market-reports/adhd-apps-market-118094)) — figures conflict wildly (treat as low reliability) but direction is consistently up.
- r/ADHD has ~2,269,860 subscribers ([reddtrends](https://www.reddtrends.com/r/adhd)); ADHD diagnosis/content volume keeps climbing ([r/science thread, 3999 pts, 2026-07](https://reddit.com/r/science/comments/1v4h560/how_telehealth_is_fuelling_a_surge_in_adhd/)).
- "Body doubling" crossed from ADHD niche to mainstream TikTok/remote-work trend ([Fortune, 2023](https://fortune.com/2023/03/05/body-doubling-parallel-working-tiktok-trend/)) and remains an active query in 2026 threads.
- Funded competitors keep raising: Tiimo raised $4.8M + €3M seed ([Tiimo](https://www.tiimoapp.com/resource-hub/tiimo-raises-4-8m-neurodivergent-planner), [Sifted](https://sifted.eu/articles/tiimo-planning-app-for-neurodivergent-people)); Inflow has raised $16.7M total ([Tracxn](https://tracxn.com/d/companies/inflow/__qAt9iIHoNiUqJkPxrCwJ7cjYY3__wZjjyTOZm8Va4us)) — investors see a growing category.

## 6. Competitor Landscape

| Competitor | What it is | Pricing | Top complaints |
|---|---|---|---|
| Focusmate | 1:1 virtual body doubling | Free 3 sessions/wk; Plus $8/mo (annual) / $12/mo ([pricing](https://www.focusmate.com/pricing/)) | Rising prices, partner no-shows/rule violations, reliability, empty session pool ([reddit](https://reddit.com/r/productivity/comments/1ogy7zp/focusmate_is_becoming_increasingly_frustrating/)) |
| Flow Club | Group body-double sessions | $40/mo or $400/yr ([flown comparison](https://flown.com/blog/deep-work/flown-vs-flow-club)) | Expensive; fixed schedule format |
| Tiimo | Visual planner for neurodivergent users | ~$8/mo, $80/yr ([lifestack](https://lifestack.ai/blog/tiimo-pricing)) | "Is Pro worth it?" hesitation; no body doubling or real accountability |
| goblin.tools (Magic ToDo) | Free AI task breakdown | Free web / cheap one-time app | No accountability, no planning system, no persistence layer people stick with |
| Inflow | CBT-based ADHD learning program | Subscription (funded at $16.7M) | Educational content, not a day-to-day planner |
| Llama Life | Single-task focus timer | Subscription | "Half-finished app with distracting quirks" ([App Store review](https://apps.apple.com/us/app/adhd-organizer-llama-life/id6454469750?see-all=reviews)) |
| Numo | ADHD brain-dump/community app | $7.99–15.99/mo | "$15 per month is more than most people are going to want to pay" ([Play Store](https://play.google.com/store/apps/details?id=io.mindist.well&hl=en_US)) |

**The gaps:**
- Nobody bundles breakdown + gentle accountability + co-focus in one app at ≤$10/mo.
- "Gentle" is a proven differentiator: pressure/streak/shame mechanics actively backfire for this audience, and mainstream apps are built on them.
- Trust is broken: dark-pattern subscription apps (Fabulous) have burned this community — transparent pricing and easy cancel is a positioning weapon.
- Incumbent body doubling requires synchronous stranger matching; users report empty pools — async or small-group models are unexploited.

## 7. Market Size & Money

**Market data:** ADHD apps market estimates range $0.56B–$3.2B (2024–2026) growing 11–16%/yr — wide spread, low reliability, use directionally ([SMR](https://www.strategicmarketresearch.com/market-report/adhd-apps-market), [BRI](https://www.businessresearchinsights.com/market-reports/adhd-apps-market-118094), [TBRC](https://www.thebusinessresearchcompany.com/report/attention-deficit-hyperactivity-disorder-adhd-apps-global-market-report)). Funding signals: Tiimo (~$8M raised), Inflow ($16.7M).

**TAM/SAM/SOM (rough estimate — assumptions shown):**
- TAM: ~750M English-proficient adults globally × ~4% adult ADHD prevalence ≈ 30M people × $70/yr potential spend ≈ **~$2B/yr** (estimate; prevalence assumption consistent with commonly cited 4–5%).
- SAM: subset actively seeking digital planning/focus tools and able to pay ≈ 10% of TAM users ≈ 3M users ≈ **~$210M/yr** (estimate).
- SOM (1–3 yr): indie app reaching 2,000–5,000 paying subscribers at $60–96/yr ≈ **$120K–480K ARR** (estimate). This is a healthy indie-scale outcome, not a venture outcome — stated honestly.

**Monetization:** Comparable price points cluster at $8/mo (Focusmate, Tiimo) to $15–40/mo (Numo, Flow Club). Recommended: freemium (breakdown tool free, à la goblin.tools, as an acquisition engine) + $7–10/mo or ~$70/yr premium (unlimited breakdowns, accountability partner features, co-focus rooms), with radically transparent billing and one-tap cancel as a stated value.

## 8. Distribution Plan of Attack

1. **Reddit (primary, organic):** r/ADHD (~2.27M members), r/adhdwomen, r/AutisticWithADHD, r/ADHD_Programmers, r/ProductivityApps, r/GetStudying. Big subs restrict self-promo — the play is participating, answering the recurring "what app actually works" threads (they appear weekly, 100+ comments), and signature/profile-based discovery. The user ICP literally asks for recommendations constantly.
2. **Free tool as SEO/viral hook:** a goblin.tools-style free "task breakdown" web page targeting long-tail queries like "adhd task breakdown tool", "how to break down tasks adhd", "virtual body doubling app free" — head terms ("adhd planner app") are owned by listicles ([example](https://www.morgen.so/blog-posts/best-daily-planner-for-adhd)) and hard to rank for; the long tail is open.
3. **App Store optimization:** "ADHD planner", "body doubling", "AuDHD" keywords — category exists but no dominant brand owns ASO; Tiimo is the main name. Feasible for an indie.
4. **TikTok/Shorts:** body doubling is an established TikTok trend with mainstream press ([Fortune](https://fortune.com/2023/03/05/body-doubling-parallel-working-tiktok-trend/)) — demo-style content ("watch this app break down my scary task") fits the platform. ADHD creators are an approachable micro-influencer tier.
5. **Discord/communities:** ADHD co-working Discords and study servers — host free co-focus rooms as community seeding.

## 9. Risks & Open Questions

- **Retention paradox:** the audience's core symptom is abandoning tools. Every competitor fails here; no evidence anyone has solved it. This is the existential risk.
- **Market-size data quality:** published figures conflict by 5x; treat TAM as directional only.
- **Two-sided liquidity:** on-demand body doubling needs concurrent users 24/7; even Focusmate users report empty pools. A solo dev cannot bootstrap this reliably — hence the pivot to async/small-group.
- **Trust deficit:** the community has been burned by dark-pattern subscriptions; any billing misstep gets amplified (the 1514-point Fabulous thread).
- **Price sensitivity:** multiple signals that >$15/mo is a hard ceiling for much of this audience.
- **Crowded bottom of the market:** dozens of indie ADHD apps launch monthly on r/ProductivityApps; differentiation must be retention mechanics, not features.

## 10. Your Validation Plan (do these before building)

1. **Landing page smoke test:** headline: "The planner that doesn't punish you for having an ADHD brain — AI task breakdown + gentle accountability"; CTA: "Join the beta — free forever for the first 500". Positive signal: ~10%+ email conversion from cold traffic (interpret with caution based on traffic source). Run $100 of Reddit/TikTok traffic or post organically.
2. **Ask real users (behavior, not opinions):**
   - "Walk me through the last productivity app you abandoned — when did you stop opening it and why?"
   - "What are you currently paying for to manage focus/planning, and what does it cost per month?"
   - "When you last had a task you avoided for a week, what did you actually do to get it done?"
   - "Have you ever paid for body doubling or an accountability partner? What happened?"
   - "What would make you cancel a subscription like this in month one?"
3. **Post in:** r/ADHD ("what actually works for you" threads — ask, don't promote), r/adhdwomen (accountability angle), r/AutisticWithADHD (low-pressure design angle), r/ProductivityApps (they explicitly welcome new-app threads), r/GetStudying (co-study angle). Angle for each: ask about abandonment stories and current spend, link nothing until asked.

## Indie Developer Fit

**Distribution engine:** Reddit + TikTok community marketing plus a free-tool SEO hook — all solo-executable, no sales. **Capital required:** near zero beyond time; free tier costs are LLM API calls (small) — a $200 validation budget suffices. **Solo-build feasibility:** the wedge version (AI breakdown + async accountability + scheduled small rooms) is a standard mobile/web app, well within one developer's reach; on-demand live matching is not, and should be deferred. **Verdict on indie fit:** good, conditional on the pivot — this is a community-distributed consumer subscription with a realistic $120K–480K ARR ceiling, ideal indie economics *if* the retention problem is treated as the core product challenge rather than an afterthought.

## Sources

1. https://reddit.com/r/ProductivityApps/comments/1t2m4jw/adhd_productivity_apps_endless_cycle_of/
2. https://reddit.com/r/ProductivityApps/comments/1qdc92u/best_productivity_apps_for_adhd/
3. https://reddit.com/r/ADHDUK/comments/1qkemzv/adhd_planners_apps_what_actually_work_for_you/
4. https://reddit.com/r/AutisticWithADHD/comments/1qbr4vh/what_productivity_tools_or_apps_actually_work_for/
5. https://reddit.com/r/adhdwomen/comments/1u16521/bodydoubling_is_such_a_miracle/
6. https://reddit.com/r/ADHD/comments/1scn98w/body_doubling_want_to_try_it_but_have_no_idea/
7. https://reddit.com/r/adhdwomen/comments/1nar2kh/have_you_found_an_app_to_hold_you_accountable_and/
8. https://reddit.com/r/productivity/comments/1ogy7zp/focusmate_is_becoming_increasingly_frustrating/
9. https://reddit.com/r/adhdwomen/comments/1rbdlsd/focusmate_has_no_people/
10. https://reddit.com/r/adhdwomen/comments/1q65obz/scam_targeting_adhd_folks/
11. https://reddit.com/r/productivity/comments/1skrtua/seeking_a_virtual_cofocus_partner/
12. https://www.focusmate.com/pricing/
13. https://flown.com/blog/deep-work/flown-vs-flow-club
14. https://lifestack.ai/blog/tiimo-pricing
15. https://www.tiimoapp.com/resource-hub/tiimo-raises-4-8m-neurodivergent-planner
16. https://sifted.eu/articles/tiimo-planning-app-for-neurodivergent-people
17. https://tracxn.com/d/companies/inflow/__qAt9iIHoNiUqJkPxrCwJ7cjYY3__wZjjyTOZm8Va4us
18. https://techcrunch.com/2022/01/17/inflow-a-science-based-app-for-adhd-raises-2-3m-seed-led-by-hoxton-ventures/
19. https://goblin.tools/ToDo
20. https://apps.apple.com/us/app/adhd-organizer-llama-life/id6454469750?see-all=reviews
21. https://play.google.com/store/apps/details?id=io.mindist.well&hl=en_US
22. https://www.strategicmarketresearch.com/market-report/adhd-apps-market
23. https://www.businessresearchinsights.com/market-reports/adhd-apps-market-118094
24. https://www.thebusinessresearchcompany.com/report/attention-deficit-hyperactivity-disorder-adhd-apps-global-market-report
25. https://www.reddtrends.com/r/adhd
26. https://fortune.com/2023/03/05/body-doubling-parallel-working-tiktok-trend/
27. https://reddit.com/r/science/comments/1v4h560/how_telehealth_is_fuelling_a_surge_in_adhd/
28. https://www.morgen.so/blog-posts/best-daily-planner-for-adhd

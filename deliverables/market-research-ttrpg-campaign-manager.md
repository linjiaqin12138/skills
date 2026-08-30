# Market Research: Campaign & Homebrew Content Manager for TTRPG Players

**Date:** 2026-08-30 · **Researcher:** Claude (market-research skill) · **Confidence:** Medium-High

## 1. Executive Summary

The idea is a freemium tool for D&D-style GMs/DMs to organize campaign notes, NPCs, timelines, and homebrew rules for long-running campaigns. The pain is real and recurring — "what do you use to organize your campaign?" threads appear monthly across multiple subreddits — and incumbents (World Anvil, Kanka, LegendKeeper) prove people pay. However, this exact product category is now crowded, including several indie entrants already attacking the "simpler than World Anvil" gap. **Verdict: PIVOT — 6.1/10.** The #1 opportunity is a sharp wedge (session-notes-to-lore pipeline or local-first/offline) rather than a general-purpose manager. The #1 risk is building "World Anvil but simpler" — a positioning at least three competitors already occupy.

## 2. Verdict

# PIVOT — 6.1/10

The raw idea as stated (general campaign + homebrew content manager, freemium) lands in a proven but saturated middle: pain is genuine (6/10), monetization is proven by three paying incumbents (6/10), distribution via communities and SEO long-tail is unusually good for an indie (7/10) — but the competition gap is only moderate (6/10) because the obvious differentiation ("simpler, less overwhelming than World Anvil") is already claimed by LegendKeeper, Inkwarden, DMScribe, and others. **Pivot direction: feature focus** — don't build another wiki-style world bible; build the *working-memory layer for running campaigns* (session recaps → auto-extracted NPCs/plot threads/timeline; offline/local-first). The strongest argument against this PIVOT verdict: Kanka is a 3-person indie team with 400K+ users and World Anvil has 3.5M registered users, which arguably proves the general-purpose category still rewards execution — a sufficiently better product could win without pivoting. But for a *solo* dev with no audience, entering head-on against funded, entrenched tools with network/lock-in effects is a bad bet; a wedge is the realistic path. Evidence base is ~20 solid sources, so no low-confidence flag; market-size numbers are estimates.

### Scorecard

| Dimension | Weight | Score | Key evidence |
|---|---|---|---|
| Pain Intensity | 25% | 6/10 | Monthly recurring "what do you use?" threads across r/DMAcademy, r/DnD, r/onednd, r/SWN; users describe hacky workarounds (giant Google Docs, OneNote, sticky notes) ([1][s1], [2][s2], [3][s3]) |
| Demand Trend | 15% | 6/10 | TTRPG market ~$1.9–2.4B growing ~12% CAGR; threads steady through 2025–2026; no decline signal ([13][s13]) |
| Competition Gap | 20% | 6/10 | World Anvil "overwhelming/overkill", Kanka "limited", LegendKeeper offline issues — but the "simpler" gap is already contested by several players ([6][s6], [9][s9], [18][s18]) |
| Monetization | 15% | 6/10 | Users demonstrably pay $4.50–$25/mo (World Anvil, Kanka, LegendKeeper $90/yr); price complaints = proof of payment ([11][s11], [12][s12], [14][s14]) |
| Market Size | 10% | 5/10 | Bottoms-up SOM ~$100–180K ARR at indie scale — comfortable lifestyle business, not venture scale (estimates in §7) |
| Distribution | 15% | 7/10 | Multiple hungry communities (r/DnD 4.1M, r/DMAcademy 500K+); SEO long-tail proven rankable by indie blogs (inkwarden.app ranks for "world anvil alternative") ([16][s16], [17][s17], [18][s18]) |
| **Weighted total** | | **6.1/10** | |

## 3. The Problem & Who Has It

A GM running a multi-year homebrew campaign accumulates hundreds of NPCs, plot threads, locations, session notes, and custom rules. Generic tools (Google Docs, OneNote, physical notebooks) don't link entities or surface "what did the party learn 8 sessions ago?"; dedicated tools are either overwhelming (World Anvil) or limiting (Kanka free tier). **Assumed user:** the forever-DM — a prosumer hobbyist, English-speaking, runs 1–2 long campaigns, already spends money on the hobby (books, dice, D&D Beyond, VTTs). **Geography:** US/EU-weighted, global English. Assumption: DMs, not players, are the paying persona — players rarely organize campaign lore.

## 4. Pain Point Evidence

- "Obsidian (needs a subscription to be cross-platform), Kanka (kind of limited), World Anvil (overkill)" — the gap in one sentence. r/DMAcademy, 33↑/60 comments, Dec 2025 ([source](https://reddit.com/r/DMAcademy/comments/1pgzvix/what_are_you_using_to_organize_your_campaigns/))
- "The one doc I use for notes is starting to balloon" — Google Drive workflow breaking under a long campaign. r/DMAcademy, 36↑/83 comments, Apr 2026 ([source](https://reddit.com/r/DMAcademy/comments/1sysurl/are_tools_like_world_anvil_worth_it/))
- "I used it for a while, but was overwhelmed with the options" (World Anvil). r/DungeonMasters, 2025 ([source](https://www.reddit.com/r/DungeonMasters/comments/1jfdcti/is_world_anvil_worth_the_subscription/))
- "So overwhelming" — r/WorldAnvil's own users, Jan 2024 ([source](https://www.reddit.com/r/WorldAnvil/comments/18ae6cz/so_overwhelming/))
- "If they are near bankruptcy they can increase their price... they would have my materials hostage" — local/offline data-ownership demand. r/worldbuilding, Feb 2026 ([source](https://reddit.com/r/worldbuilding/comments/1qxgpxw/local_alternative_to_worldanvil/))
- "There is still no real self-hostable alternative to World Anvil" — user built Kanka-CE rather than accept options. r/DnD, Apr 2026 ([source](https://reddit.com/r/DnD/comments/1szarby/a_selfhostable_alternative_to_world_anvil_kanka/))
- "Frustrated with the limitations imposed on the free tier" — Kanka rated 6/10 in a 3-year paid-tools retrospective. r/rpg, 2026 ([source](https://www.reddit.com/r/rpg/comments/1lqpsgl/my_experience_with_paid_ddttrpg_tools_after_3/))
- Steady stream of "what app do you use for campaign notes?" — r/onednd (Jun 2026, 45 comments), r/DMAcademy (Jun 2026), r/DnD (Oct 2025, 106 comments), r/SWN (Aug 2026) ([source](https://reddit.com/r/onednd/comments/1u5l5im/dms_how_do_you_manage_your_campaign_notes/), [source](https://reddit.com/r/DMAcademy/comments/1ttco7p/what_appsystem_do_you_use_for_sessioncampaign/), [source](https://reddit.com/r/DnD/comments/1obwfyh/how_do_you_take_notes_for_dd_show_me_your_setups/))
- Session prep burden ("organizing notes and plots... which part drains your mental energy") tied to DM burnout. r/DnD, 2026 ([source](https://reddit.com/r/DnD/comments/1q3umqf/realistically_how_long_should_a_session_take_to/))
- Price sensitivity is real: user balks at World Anvil's $12/mo or $650 lifetime, asks for anything cheaper with timelines + family trees. r/writing, Nov 2025 ([source](https://www.reddit.com/r/writing/comments/1p84rye/do_you_know_world_anvil_and_its_wort_the_price_if/))

## 5. Demand Signals

- TTRPG market sized at ~$1.9B (2024) → ~$2.4B (2026), projected ~$5.3–6.6B by 2033–2035, ~12% CAGR ([source](https://www.businessresearchinsights.com/market-reports/tabletop-role-playing-game-ttrpg-market-110856))
- Top-of-funnel audience is large: D&D Beyond reports 19M registered users ([source](https://www.reddit.com/r/DnD/comments/1iidqtc/3_million_new_characters_but_19_million_users_is/))
- World Anvil claims 3.5M+ registered users; Kanka claims 400K+ creators in 90+ countries with a 3-person team ([source](https://www.worldanvil.com/), [source](https://kanka.io/about))
- Recency: fresh "how do I organize" threads every month through Aug 2026 — demand is stable-to-growing, not a fad. No declining-niche signals found. (Google Trends data not retrievable from this environment; flagged as a gap.)

## 6. Competitor Landscape

| Competitor | What it is | Pricing | Top complaints |
|---|---|---|---|
| World Anvil | Feature-maximalist worldbuilding suite + campaign manager | Free; $4.50–$25/mo annual; $650 lifetime ([source](https://www.worldanvil.com/pricing)) | "Overwhelming," "overkill," cluttered UX, data-hostage fears ([6][s6], [9][s9]) |
| Kanka | Open-source campaign manager/wiki, generous free tier | Free; $4.99–$24.99/mo ([source](https://kanka.io/pricing)) | "Kind of limited," free-tier restrictions, UI utilitarian ([1][s1], [10][s10]) |
| LegendKeeper | Polished, modern worldbuilding + maps + whiteboards | $9/mo or $90/yr ([source](https://www.legendkeeper.com/pricing)) | Offline mode broken/unreliable; slow feature development ([source](https://www.reddit.com/r/legendkeeper/comments/ramc9n/is_offline_not_working_for_anyone_else/)) |
| Obsidian Portal | Legacy campaign wiki (since ~2007) | Freemium | Dated UX; "legacy" reputation ([source](https://www.obsidianportal.com/)) |
| Obsidian + plugins | Local-first markdown DIY stack | Free (sync $4/mo) | Setup burden; cross-platform sync costs; not TTRPG-structured out of the box ([1][s1]) |
| Inkwarden / DMScribe / Table Canon | Indie "simpler World Anvil" / session-notes / AI-memory entrants | Freemium | Unproven, small; but they show the gap is contested ([source](https://inkwarden.app/blog/world-anvil-alternative-inkwarden), [source](https://dmscribe.com/alternatives/kanka), [source](https://news.ycombinator.com/item?id=49376991)) |

**The gaps:**
- **Working memory, not world bible.** Every incumbent is a wiki; nobody owns the *session loop* — recap → updated NPC states → open plot threads → next-session prep. DMScribe's positioning ("notes from your actual sessions") and HN's "Table Canon, AI campaign memory engine" (Aug 2026) both point at this unowned wedge.
- **Offline / local-first with data ownership.** Recurring explicit demand; LegendKeeper's offline is broken, World Anvil is cloud-only, and users cite "hostage" anxiety.
- **Simplicity ceiling.** "Overwhelming" is World Anvil's defining complaint — but note LegendKeeper, Inkwarden, and Kanka all already sell "simpler," so simplicity *alone* is not a moat.
- **Price umbrella.** $9–12/mo incumbents leave room for a ~$4–5/mo or cheap-lifetime indie tier.

## 7. Market Size & Money

**Market data:** Global TTRPG market ~$1.9–2.4B, ~12% CAGR ([source](https://www.businessresearchinsights.com/market-reports/tabletop-role-playing-game-ttrpg-market-110856)). Kanka proves indie viability: 3-person team, 400K+ users, bootstrapped on subscriptions ([source](https://kanka.io/about)). No VC funding signal in this niche — it's a bootstrapper's market.

**TAM/SAM/SOM (rough estimate — assumptions shown):**
- TAM: ~19M D&D Beyond users × ~20% who DM × ~$60/yr software spend ≈ **$230M/yr** (assumes 1-in-5 players DM; Beyond is D&D-only, other systems add upside)
- SAM: DMs of long-running homebrew campaigns who already use or seek a digital organizer — ~15% of TAM ≈ 570K DMs × $60/yr ≈ **$34M/yr**
- SOM (1–3 yr): capture 2,000–3,000 paying DMs at $50–60/yr via community + SEO ≈ **$100–180K ARR** — a realistic indie-scale outcome, intentionally not inflated

**Monetization:** Proven recurring willingness to pay at $4.50–$25/mo; complaints about World Anvil's pricing are themselves evidence people pay. Recommended: freemium with a $4–6/mo ($40–60/yr) tier gating sync, collaboration, and unlimited campaigns; consider a lifetime deal at launch for cash flow (common and accepted in this niche — World Anvil sells $650 lifetimes).

## 8. Distribution Plan of Attack

1. **Reddit communities (primary):** r/DMAcademy (500K+ members — [source](https://everongames.com/transform-free-dnd-adventures-into-epic-campaigns/)), r/DnD (4.1M — [source](https://tenureandtech.com/2025/06/19/dungeons-dragons-and-higher-ed/)), r/dndnext, r/DnDBehindTheScreen, r/worldbuilding, r/mattcolville, plus system subs (r/Pathfinder2e, r/SWN). The weekly "what tool do you use?" threads are warm inbound — answer helpfully, disclose authorship. Caveat: strict self-promo rules; use weekly promo threads and build comment karma first.
2. **SEO long-tail:** "world anvil alternative", "kanka vs legendkeeper", "how to organize dnd campaign notes" — a solo indie blog (inkwarden.app) already ranks #1-style for "world anvil alternative," proving these keywords are winnable without domain authority. Comparison/alternative pages are the play.
3. **Discord servers:** D&D Beyond's and community DM servers; DM-advice Discords allow tool sharing in designated channels.
4. **YouTube/creator adjacency:** DM-advice channels (Matt Colville ecosystem) drive tool adoption; sponsorships are cheap at small channel sizes.
5. **App stores:** "RPG Notebook" and "Game Master 5e" show moderate, beatable mobile competition — a companion app could capture store search.

## 9. Risks & Open Questions

- **Crowded middle:** at least 6 credible competitors + DIY Obsidian setups; switching costs are high (migration of years of notes), so "10% better" loses.
- **Free-workaround gravity:** many DMs are genuinely fine with Google Docs/OneNote — Pain Intensity is solid but not desperate.
- **WotC platform risk:** D&D Beyond could absorb lightweight campaign-note features; the 2024 edition cycle shows they're investing in digital tools.
- **No Google Trends data** retrieved (network constraints); demand trend relies on market reports + thread recency instead.
- **Price sensitivity:** the audience includes many students/budget hobbyists; the $9–12/mo tier already draws complaints.
- **Unverified:** actual conversion rates and churn for incumbents are unknown; Kanka's 400K users ≠ 400K payers (Kanka disclosed only ~9K active campaigns in 2022 — [source](https://blog.kanka.io/2022/07/16/thoughts-on-the-discussions-surrounding-changes-to-the-free-tier/)).

## 10. Your Validation Plan (do these before building)

1. **Landing page smoke test:** headline: *"Your campaign remembers, even when you don't. Session notes that turn themselves into a living world wiki."*; CTA: *"Join the GM waitlist — free during beta."* Positive signal: ~10%+ email conversion from cold traffic (interpret with caution based on traffic source).
2. **Ask real users (behavior, not opinions):**
   - "What did you use to prep your *last* session, and how long did finding old notes take?"
   - "When did you last lose track of an NPC or plot thread mid-session — what happened?"
   - "What do you currently pay for in your D&D setup (tools, books, VTT), per month?"
   - "Have you ever started migrating notes into a tool and abandoned it? Why?"
   - "What would make you trust a new tool with 2 years of campaign notes?"
3. **Post in:** r/DMAcademy (weekly question/self-promo threads — ask "how do you resurrect a plot thread from 10 sessions ago?"), r/DnDBehindTheScreen (tool-nerd density), r/worldbuilding (angle: local-first data ownership), r/onednd (new-edition DMs rebuilding workflows), and the LegendKeeper/Kanka Discords (listen, don't pitch).

## Indie Developer Fit

Distribution engine: organic communities + SEO long-tail — no enterprise sales needed, and the niche's buyers literally post "what tool should I use?" weekly. Capital required: near-zero (solo SaaS; the AI-recap wedge adds modest LLM API costs, manageable at freemium scale). Solo-build feasibility: high for the wedge version (notes + entity extraction + timeline); the full World-Anvil-style suite is *not* solo-feasible — do not attempt it. **Verdict on indie fit: good, conditional on the pivot** — as a general campaign manager it's a me-too grind against entrenched free tiers; as a focused session-memory tool sold to forever-DMs through Reddit and comparison-keyword SEO, it's a realistic $100K+ ARR solo business.

## Sources

1. [r/DMAcademy — What are you using to organize your campaigns?](https://reddit.com/r/DMAcademy/comments/1pgzvix/what_are_you_using_to_organize_your_campaigns/)
2. [r/DMAcademy — Are tools like 'World Anvil' worth it?](https://reddit.com/r/DMAcademy/comments/1sysurl/are_tools_like_world_anvil_worth_it/)
3. [r/DnD — How do you take notes for D&D?](https://reddit.com/r/DnD/comments/1obwfyh/how_do_you_take_notes_for_dd_show_me_your_setups/)
4. [r/onednd — DMs how do you manage your campaign notes?](https://reddit.com/r/onednd/comments/1u5l5im/dms_how_do_you_manage_your_campaign_notes/)
5. [r/DMAcademy — What App/System do you Use for Session/Campaign Notes?](https://reddit.com/r/DMAcademy/comments/1ttco7p/what_appsystem_do_you_use_for_sessioncampaign/)
6. [r/worldbuilding — Local alternative to WorldAnvil?](https://reddit.com/r/worldbuilding/comments/1qxgpxw/local_alternative_to_worldanvil/)
7. [r/DnD — Kanka Community Edition (self-hostable WA alternative)](https://reddit.com/r/DnD/comments/1szarby/a_selfhostable_alternative_to_world_anvil_kanka/)
8. [r/WorldAnvil — So overwhelming](https://www.reddit.com/r/WorldAnvil/comments/18ae6cz/so_overwhelming/)
9. [r/DungeonMasters — Is world anvil worth the subscription?](https://www.reddit.com/r/DungeonMasters/comments/1jfdcti/is_world_anvil_worth_the_subscription/)
10. [r/rpg — My experience with paid D&D/TTRPG tools after 3+ years](https://www.reddit.com/r/rpg/comments/1lqpsgl/my_experience_with_paid_ddttrpg_tools_after_3/)
11. [World Anvil pricing](https://www.worldanvil.com/pricing)
12. [Kanka pricing](https://kanka.io/pricing) and [Kanka about (400K+ users, 3-person team)](https://kanka.io/about)
13. [Business Research Insights — TTRPG market size](https://www.businessresearchinsights.com/market-reports/tabletop-role-playing-game-ttrpg-market-110856)
14. [LegendKeeper pricing](https://www.legendkeeper.com/pricing)
15. [r/DnD — D&D Beyond 19M users](https://www.reddit.com/r/DnD/comments/1iidqtc/3_million_new_characters_but_19_million_users_is/)
16. [Tenure & Tech — r/DnD 4.1M members](https://tenureandtech.com/2025/06/19/dungeons-dragons-and-higher-ed/)
17. [Everon Games — r/DMAcademy 500K+ members](https://everongames.com/transform-free-dnd-adventures-into-epic-campaigns/)
18. [Inkwarden — World Anvil alternative (indie SEO proof)](https://inkwarden.app/blog/world-anvil-alternative-inkwarden)
19. [Show HN: Table Canon, AI Campaign Memory Engine](https://news.ycombinator.com/item?id=49376991)
20. [DMScribe — Kanka alternative positioning](https://dmscribe.com/alternatives/kanka)
21. [r/writing — World Anvil price complaint](https://www.reddit.com/r/writing/comments/1p84rye/do_you_know_world_anvil_and_its_wort_the_price_if/)
22. [Kanka blog — free tier changes / active campaign counts](https://blog.kanka.io/2022/07/16/thoughts-on-the-discussions-surrounding-changes-to-the-free-tier/)
23. [r/legendkeeper — offline mode broken](https://www.reddit.com/r/legendkeeper/comments/ramc9n/is_offline_not_working_for_anyone_else/)
24. [r/DnD — session prep burden / DM burnout](https://reddit.com/r/DnD/comments/1q3umqf/realistically_how_long_should_a_session_take_to/)

[s1]: https://reddit.com/r/DMAcademy/comments/1pgzvix/what_are_you_using_to_organize_your_campaigns/
[s2]: https://reddit.com/r/DMAcademy/comments/1sysurl/are_tools_like_world_anvil_worth_it/
[s3]: https://reddit.com/r/onednd/comments/1u5l5im/dms_how_do_you_manage_your_campaign_notes/
[s6]: https://reddit.com/r/worldbuilding/comments/1qxgpxw/local_alternative_to_worldanvil/
[s9]: https://www.reddit.com/r/DungeonMasters/comments/1jfdcti/is_world_anvil_worth_the_subscription/
[s10]: https://www.reddit.com/r/rpg/comments/1lqpsgl/my_experience_with_paid_ddttrpg_tools_after_3/
[s11]: https://www.worldanvil.com/pricing
[s12]: https://kanka.io/pricing
[s13]: https://www.businessresearchinsights.com/market-reports/tabletop-role-playing-game-ttrpg-market-110856
[s14]: https://www.legendkeeper.com/pricing
[s16]: https://tenureandtech.com/2025/06/19/dungeons-dragons-and-higher-ed/
[s17]: https://everongames.com/transform-free-dnd-adventures-into-epic-campaigns/
[s18]: https://inkwarden.app/blog/world-anvil-alternative-inkwarden

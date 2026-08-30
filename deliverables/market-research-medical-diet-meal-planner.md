# Market Research: Medical-Diet Meal Planning App (low-FODMAP / renal / diabetic)

**Date:** 2026-08-30 · **Researcher:** Claude (market-research skill) · **Confidence:** Medium

## 1. Executive Summary

A consumer subscription app that generates weekly meal plans and grocery lists for people on restrictive clinical diets (low-FODMAP, renal, diabetic), distributed via SEO long-tail and patient communities. The pain is real, emotional, and well-documented — people describe renal diets as "the thing that's making me depressed" and a rant about a bad renal cookbook pulled 736 upvotes. Existing tools are fragmented: Monash is a "glorified database," Fig is a scanner, Real Plans covers FODMAP but not renal constraints, and nobody handles the very common renal + diabetic comorbidity well. Verdict: **PIVOT (6.0/10)** — the multi-diet generic play is crowded; the wedge worth building is combined-constraint planning (renal-diabetic first), sold at $8–12/mo. Biggest risk: this is a trust-and-accuracy market where a wrong meal plan has medical consequences, and incumbents (Monash, DaVita) own the authority.

## 2. Verdict

# PIVOT — 6.0/10

Pain Intensity (7) carries the score, and Distribution (6) is genuinely indie-friendly — but Competition Gap (5) and Market Size at indie scale (5) hold it below the GO threshold. Real Plans already runs a credible low-FODMAP meal planner at $144/yr, and free authoritative content (DaVita, Kidney Fund, Monash) mops up casual demand. The pivot: **narrow from "all clinical diets" to combined-constraint meal planning — start with the renal + diabetic comorbidity (one of the most common and worst-served diet intersections, heavily searched by caregivers), then expand to other stacking constraints (FODMAP + vegetarian, CKD stage-specific).** Steelman against the verdict: the comorbid renal-diabetic audience skews older and caregiver-mediated, which raises CAC and churn risk, and a solo dev shipping nutrient-constrained meal plans carries real accuracy/liability exposure that a spreadsheet-savvy dietitian competitor could undercut with content alone. If the wedge doesn't validate, this is a content site, not a SaaS.

### Scorecard

| Dimension | Weight | Score | Key evidence |
|---|---|---|---|
| Pain Intensity | 25% | 7/10 | Emotional threads across r/kidneydisease, r/FODMAPS; 736-upvote rant about a fake renal cookbook; people paying for tools they dislike |
| Demand Trend | 15% | 6/10 | Evergreen search long-tail + a funded "food as medicine" wave (Season $34M, ModifyHealth $13.5M, Fay $75M) |
| Competition Gap | 20% | 5/10 | Real weaknesses everywhere (Monash clunky, Fig not a planner, KidneyPal tracker-only), but Real Plans/FODMAP Everyday is a solid incumbent for one diet |
| Monetization | 15% | 6/10 | Users demonstrably pay $60–144/yr (Fig, Real Plans, PlateJoy, Monash), but $20–30/mo called "a fortune" — ceiling ~$12/mo |
| Market Size | 10% | 5/10 | Massive TAM (tens of millions), realistic indie SOM ~$150–300K ARR |
| Distribution | 15% | 6/10 | Active communities (r/FODMAPS 95k, r/diabetes 166k) + deep SEO long-tail ("kidney friendly recipes for stage 3"); app-store keywords moderately contested |
| **Weighted total** | | **6.0/10** | |

## 3. The Problem & Who Has It

People placed on restrictive clinical diets — low-FODMAP (IBS/SIBO), renal (CKD stages 2–5/dialysis), diabetic (type 2, gestational, prediabetes) — must plan every meal around nutrient constraints (sodium, potassium, phosphorus, protein, carbs, FODMAP stacking) they barely understand. The assumed user: a newly diagnosed adult or a caregiver planning for a family member, English-speaking, global (US-weighted). Key assumption: the buyer is a consumer paying out of pocket, not a payer/provider. A second, underappreciated user is the **caregiver** — several of the highest-engagement threads were adult children planning food for parents with CKD + diabetes.

## 4. Pain Point Evidence

- "the diet is the thing that's making me depressed" — stage 5 CKD patient struggling with a dietitian-prescribed diet, r/kidneydisease, 26 upvotes, 34 comments, Mar 2026 ([source](https://reddit.com/r/kidneydisease/comments/1s2op4u/stage_5_diet_is_really_hard/)).
- "I'm IRATE about this renal support cookbook for seniors" — caregiver of an 87-year-old with CKD stage 2 + managed diabetes finds published recipes are fake; asks for "actual, tested, real meals." r/Cooking, **736 upvotes, 126 comments**, Dec 2025 ([source](https://reddit.com/r/Cooking/comments/1pgw667/im_irate_about_this_renal_support_cookbook_for/)).
- "It is the clunkiest app ever… you cannot meal plan… a glorified database" — Monash app refund rant, r/FODMAPS, 37 upvotes, 56 comments, Jun 2026 ([source](https://reddit.com/r/FODMAPS/comments/1u4712d/someone_explain_how_monash_app_is_useful_before_i/)).
- "Right now I'm overwhelmed I don't even want to start eliminating" — new low-FODMAP user can't meal plan, r/FODMAPS, Jun 2026 ([source](https://reddit.com/r/FODMAPS/comments/1u0k0il/need_some_simple_meal_ideas_and_clear_chart_for/)).
- "IBS/Low-FODMAP apps are all expensive and limited… doesn't cost $20-30 a month which is a fortune" — user wants to build an AI meal-planner alternative, r/lowfodmap, Nov 2025 ([source](https://reddit.com/r/lowfodmap/comments/1p64ktm/ibslowfodmap_apps_are_all_expensive_and_limited/)).
- "What can CKD people even eat?" — "trying to get low sodium diet, turns out that's almost impossible," r/kidneydisease, 38 upvotes, Jun 2026 ([source](https://reddit.com/r/kidneydisease/comments/1ufaa3w/what_can_ckd_people_even_eat/)).
- "Any apps that have accurate nutritional data on these substances [sodium, potassium, phosphorus]?" — explicit app request, r/kidneydisease, Oct 2025 ([source](https://reddit.com/r/kidneydisease/comments/1og6bj7/low_phosphorus_low_potassium_low_sodium_diet/)).
- "Meal delivery kits offering a renal diet?" — caregiver "gets a bunch of ads, then enter[s] personal info only to find out a renal diet isn't actually offered," r/kidneydisease, Dec 2025 ([source](https://reddit.com/r/kidneydisease/comments/1pin64j/meal_delivery_kits_offering_a_renal_diet/)).
- "What's in your actual toolkit? … what features are you wishing existed that don't" — newly diagnosed T1D assembling food apps, r/diabetes_t1, Dec 2025 ([source](https://reddit.com/r/diabetes_t1/comments/1pnzm0c/what_toolsapps_do_you_actually_use_for_managing/)).
- "Is there a FREE app that I can take pictures of my food and it tell me if it's good or not?" — gestational diabetes, r/GestationalDiabetes, Mar 2026 ([source](https://reddit.com/r/GestationalDiabetes/comments/1rwbqi7/any_free_apps_to_tell_if_food_is_good_or_not/)).

Evidence is not thin: pain surfaced consistently across three separate condition communities within the last 12 months.

## 5. Demand Signals

- Google autocomplete (via Serper, Aug 2026) shows a deep, evergreen long-tail: "low fodmap meal plan pdf / for a week / elimination phase / vegetarian / uk," "kidney friendly recipes for stage 3 / stage 4 / with ground beef / pdf," "diabetic meal plan free / for a week" — recipe-and-plan queries dominate, i.e., searchers want *plans*, not information.
- Funding momentum in adjacent "food as medicine": Season Health raised a $34M Series A led by a16z ([source](https://www.fiercehealthcare.com/health-tech/food-medicine-startup-season-health-nabs-34m-backed-andreessen-horowitz-cityblocks)); ModifyHealth raised $13.5M (Dec 2024) after ~$12M prior ([source](https://www.prnewswire.com/news-releases/modifyhealth-raises-13-5m-in-funding-to-expand-food-as-medicine-solutions-302314267.html)); nutrition-counseling platforms Fay ($75M total) and Nourish ($35M) raised in 2024–25 ([source](https://www.fiercehealthcare.com/health-tech/startups-fay-and-berry-street-each-bank-50m-growing-investor-appetite-personalized)). Note: this capital is flowing to payer/insurance models, not consumer apps — demand validation for the problem, not the channel.
- Digital diabetes management market estimated at $20.2B in 2025, projected to $69.1B by 2035 (GMInsights) ([source](https://www.gminsights.com/industry-analysis/digital-diabetes-management-market)) — directionally growing, though device/app-clinical heavy.
- Fresh complaint threads are current (Jun 2026 Monash rant; Jun 2026 CKD threads) — the pain is not stale.

## 6. Competitor Landscape

| Competitor | What it is | Pricing | Top complaints |
|---|---|---|---|
| Monash FODMAP Diet | Authoritative FODMAP food database app | ~$8.50 one-time | "Clunkiest app ever," no meal planning, no stacking view, thin recipes, poor non-US coverage ([source](https://reddit.com/r/FODMAPS/comments/1u4712d/someone_explain_how_monash_app_is_useful_before_i/)) |
| Fig (Food Is Good) | Barcode scanner / food-checker for many diets incl. FODMAP | $5.99/mo or ~$60/yr (Fig+) | Limited free scans; "too strict" flags; not a meal planner ([source](https://www.reddit.com/r/FoodAllergies/comments/1fvov11/fig_app_worth_it/)) |
| Real Plans (+ FODMAP Everyday) | Customizable weekly meal planner w/ grocery lists; low-FODMAP recipe partnership | $25/mo, $59/qtr, $144/yr | Price; generalist engine, no renal nutrient math ([source](https://realplans.com/faqs/)) |
| KidneyPal | Renal nutrient tracker (Na/K/P/protein) + AI food analysis | Freemium (5 analyses/day free); Pro ~<$73/yr | "left me feeling that almost every reasonable meal was a problem" — tracking without planning help ([source](https://apps.apple.com/us/app/kidney-pal-renal-diet-tracker/id1523371555)) |
| PlateJoy | Personalized meal planning; CDC-aligned diabetic plans | $12.99/mo, $69/6mo, $99/yr | Generic-personalization depth; dated product ([source](https://www.deliveryrank.com/reviews/platejoy)) |
| ModifyHealth | Low-FODMAP meal *delivery* | $13.45/entrée (~$108+/wk) | Cost, delivery-only model ([source](https://modifyhealth.com/products/m2-low-fodmap-meal)) |
| Free incumbents (DaVita, Kidney Fund "Kidney Kitchen", NKF) | Recipe libraries + guides | Free | Static content; no personalization, planning, or grocery logic ([source](https://davita.com/diet-nutrition/kidney-diet-tips/renal-diet-apps-which-one-should-i-use/)) |

**The gaps:**
- **Combined constraints.** Renal + diabetic is one of the most common comorbidities (diabetes is the leading cause of CKD), yet no consumer tool plans meals simultaneously against carb *and* Na/K/P/protein limits. Same for FODMAP + vegetarian, FODMAP + SIBO-specific.
- **Planning, not tracking/databases.** Monash = database; Fig = scanner; KidneyPal = tracker. Users repeatedly ask for "what do I cook this week + what do I buy."
- **Price tier.** The demonstrated willingness-to-pay cluster is $5–12/mo (Fig $5, PlateJoy $8–13); $25/mo (Real Plans) draws "a fortune" complaints. A focused $8–12/mo plan-generator undercuts the full-featured incumbents.
- **Trustworthy recipes.** A 736-upvote rant about a cookbook with fake recipes shows curation/testing itself is a differentiator.

## 7. Market Size & Money

**Market data:** IBS affects ~5–10% of the global population ([aboutibs.org](https://aboutibs.org/what-is-ibs/facts-about-ibs/)); CKD affects ~37M US adults ([CDC](https://www.cdc.gov/kidney-disease/php/data-research/index.html)); ~38M Americans have diabetes. The Monash app alone claims to have "helped millions" ([source](http://www.monashfodmap.com/ibs-central/i-have-ibs/get-the-app/)).

**TAM/SAM/SOM (rough estimate — assumptions shown):**
- TAM: ~100M English-reading adults on a restrictive clinical diet (US: 37M CKD + 38M diabetes + ~25M IBS, minus overlap, plus UK/CA/AU/IN-English). If 2% ever pay ~$96/yr → **~$190M/yr**.
- SAM: the subset actively searching for meal plans/recipes online — estimate 10M reachable via SEO/communities; at 5% paying $96/yr → **~$48M/yr**.
- SOM (1–3 yr, indie): 2,000–3,000 paying subscribers at ~$96/yr → **~$190K–290K ARR**. A healthy indie outcome, not a venture one. Honest note: reaching even 2,000 subscribers requires ranking for hundreds of long-tail recipe/condition keywords.

**Monetization:** Users demonstrably pay: Fig+ ~$60/yr, Real Plans $144/yr, PlateJoy $99/yr, Monash a paid one-time download, ModifyHealth $100+/week for meals. Complaints about $20–30/mo price points are proof of payment *and* of a ceiling. Recommended: freemium (1 free week-plan/month) → **$9.99/mo or $79/yr**, with a caregiver/household plan upsell. HSA/FSA eligibility and insurance-reimbursed dietitian bundles (the Fay/Nourish model) are a later B2B2C expansion, not the wedge.

## 8. Distribution Plan of Attack

1. **SEO long-tail (primary).** Autocomplete data shows plan/recipe intent with condition+stage+ingredient modifiers ("kidney friendly recipes with ground turkey," "low fodmap meal plan elimination phase"). Competition is diet blogs and nonprofits — beatable with structured, dietitian-reviewed, *plan-generating* pages (interactive tools rank and convert better than articles). Programmatic-but-curated recipe pages per (condition × stage × cuisine) is the moat-adjacent play.
2. **Patient communities (secondary).** r/FODMAPS (~95k members), r/diabetes (~166k), r/lowfodmap (~13.8k), r/kidneydisease, r/GestationalDiabetes, r/prediabetes, plus large Facebook CKD caregiver groups. These subs tolerate genuine help, not promotion — the play is answering "what do I eat" threads with free weekly plans and letting the profile convert. Note: several subs restrict self-promotion; lead with free tools.
3. **App stores (tertiary).** "low fodmap" and "renal diet" keywords are contested but thin — dominated by Monash, Fig, KidneyPal, none of which is a true multi-constraint planner. "renal diabetic meal plan" is nearly uncontested.
4. **Caregiver angle.** The highest-engagement evidence came from caregivers (the 736-upvote thread). "Meal plans for a parent with kidney disease and diabetes" is an emotionally charged, low-competition content cluster.

## 9. Risks & Open Questions

- **Accuracy/liability.** A wrong potassium number harms a dialysis patient. Needs dietitian-reviewed data and explicit "not medical advice" framing; solo-dev legal exposure is real and unquantified here.
- **Authority moat.** Monash (research origin) and DaVita (clinical trust) own credibility; a no-name app must borrow it (dietitian advisors, certified recipes).
- **Adherence/churn.** Clinical diets for IBS are temporary (elimination → reintroduction), capping LTV for the FODMAP segment; CKD/diabetes are chronic but the users skew older.
- **Free-content gravity.** DaVita, NKF, and Kidney Kitchen give away hundreds of recipes; the product must sell *planning and combined-constraint math*, not recipes.
- **Season Health cautionary note:** $34M of a16z money went to the payer/delivery model, not consumer self-pay — smart money did not pick the channel proposed here.
- Subreddit member counts for r/kidneydisease could not be verified directly (Reddit API blocked in this environment); treat community-size data for that sub as approximate.

## 10. Your Validation Plan (do these before building)

1. **Landing page smoke test:** headline: "Weekly meal plans that respect both your kidneys and your blood sugar — grocery list included." CTA: "Get next week's renal-diabetic plan free." Positive signal: ~10%+ email conversion from cold traffic (interpret with caution based on traffic source — community traffic will overperform SEO).
2. **Ask real users (behavior, not opinions):**
   - "Walk me through how you planned this week's meals. What tools, tabs, or papers were involved, and how long did it take?"
   - "What have you already paid for to help with your diet (apps, books, dietitians, meal delivery)? What did it cost and why did you keep or cancel it?"
   - "How many separate restrictions are you juggling at once (e.g., low-sodium + low-potassium + carb limits)? Which combination is hardest?"
   - "When you ignore the diet, what's the reason — too hard to plan, food tastes bad, or cost?"
   - "Who actually does the meal planning in your household — you or a family member?"
3. **Post in:** r/kidneydisease (offer a free "renal + diabetic week plan" PDF, ask what constraint combos people juggle), r/FODMAPS (ask about Monash pain points and what a planner should do that the database doesn't), r/GestationalDiabetes (time-boxed, highly motivated audience — ask what they ate yesterday and how they decided), and a Facebook CKD caregiver group (ask the cookbook-trust question from the 736-upvote thread).

## Indie Developer Fit

**Distribution engine:** SEO long-tail (condition × stage × ingredient recipe/plan pages) + genuinely helpful presence in 5–6 mid-size patient communities. No enterprise sales needed; this is a content-and-community engine one person can run. **Capital required:** near zero — recipe database curation and a dietitian review pass are sweat equity plus a few hundred dollars of consulting. **Solo-build feasibility:** high for the planner/grocery-list core; the hard part is nutrient data accuracy (USDA FoodData Central is free; FODMAP stacking logic needs care) and trust-building, not engineering. **Verdict on indie fit:** good fit *with the pivot* — a focused renal-diabetic (then multi-constraint) planner at $9.99/mo is a realistic $150–300K ARR solo business; the generic all-diets version is a crowded also-ran.

## Sources

1. https://reddit.com/r/FODMAPS/comments/1u4712d/someone_explain_how_monash_app_is_useful_before_i/
2. https://reddit.com/r/FODMAPS/comments/1t6jelj/monash_fodmap_diet_app_worth_it/
3. https://reddit.com/r/FODMAPS/comments/1tumvuo/looking_for_a_sibospecific_lowfodmap_guide_most/
4. https://reddit.com/r/FODMAPS/comments/1u0k0il/need_some_simple_meal_ideas_and_clear_chart_for/
5. https://reddit.com/r/lowfodmap/comments/1p64ktm/ibslowfodmap_apps_are_all_expensive_and_limited/
6. https://reddit.com/r/Cooking/comments/1pgw667/im_irate_about_this_renal_support_cookbook_for/
7. https://reddit.com/r/kidneydisease/comments/1s2op4u/stage_5_diet_is_really_hard/
8. https://reddit.com/r/kidneydisease/comments/1ufaa3w/what_can_ckd_people_even_eat/
9. https://reddit.com/r/kidneydisease/comments/1og6bj7/low_phosphorus_low_potassium_low_sodium_diet/
10. https://reddit.com/r/kidneydisease/comments/1pin64j/meal_delivery_kits_offering_a_renal_diet/
11. https://reddit.com/r/kidneydisease/comments/1rgkik7/i_miss_pizza/
12. https://reddit.com/r/diabetes_t1/comments/1pnzm0c/what_toolsapps_do_you_actually_use_for_managing/
13. https://reddit.com/r/GestationalDiabetes/comments/1rwbqi7/any_free_apps_to_tell_if_food_is_good_or_not/
14. https://www.reddit.com/r/FoodAllergies/comments/1fvov11/fig_app_worth_it/
15. https://modifyhealth.com/blogs/blog/eat-better-and-safer-with-the-food-is-good-fig-app
16. https://realplans.com/faqs/
17. https://apps.apple.com/us/app/kidney-pal-renal-diet-tracker/id1523371555
18. https://davita.com/diet-nutrition/kidney-diet-tips/renal-diet-apps-which-one-should-i-use/
19. https://www.deliveryrank.com/reviews/platejoy
20. https://modifyhealth.com/products/m2-low-fodmap-meal
21. https://www.fiercehealthcare.com/health-tech/food-medicine-startup-season-health-nabs-34m-backed-andreessen-horowitz-cityblocks
22. https://www.prnewswire.com/news-releases/modifyhealth-raises-13-5m-in-funding-to-expand-food-as-medicine-solutions-302314267.html
23. https://www.fiercehealthcare.com/health-tech/startups-fay-and-berry-street-each-bank-50m-growing-investor-appetite-personalized
24. https://www.gminsights.com/industry-analysis/digital-diabetes-management-market
25. https://aboutibs.org/what-is-ibs/facts-about-ibs/
26. https://www.cdc.gov/kidney-disease/php/data-research/index.html
27. http://www.monashfodmap.com/ibs-central/i-have-ibs/get-the-app/
28. https://lowfodmaprecipe.com/reviews/monash-university-fodmap-app-review

# Lauren Tan（AI Engineer CODE）观点与 layered-codebase / product-delivery 优化调研

> 调研日期：2026-11-16  
> 性质：外部演讲要点提炼 + 对本仓库两个 skill 的差距对照与改法建议。**尚未改 skill 正文**；下文「建议改动」供后续落地时按优先级取舍。  
> 主来源：[All Code Is Slop (Except Mine)](https://ai.engineer/code/2026/talks/lauren-tan)（AI Engineer CODE 2026，2026-11-11；主舞台约 1:01:02–1:18:38）；[演讲页附带 transcript](https://ai.engineer/api/code/reading-download/transcripts/lauren-tan.md)。  
> 对照对象：`skills/layered-codebase/`、`skills/product-delivery/`。

---

## 一句话结论

1. **Tan 的可迁移内核**：信任来自「环境」，不是更多 prose。投入顺序是 **代码/API 设计 → lint/类型/测试 → 运行时 verification → 才轮到 AGENTS.md / skills**。纠错优先「消灭整类错误」，而不是再写一行提醒。
2. **本仓库已对齐一半**：`layered-codebase` 已有「能工具化就不写文档」、分层 import 检查、复杂度阶梯；`product-delivery` 已有闸门、真机验收、禁止用「测试绿了」代替证据。
3. **缺口不在再写长规范，而在三块操作化**：① 微妙 API / footgun 的「框架出口 + 硬禁」；② 重复 review → API/lint/test 的升格闭环；③ 用户视角 Feature Map + 验收证据分级（截图/录屏/状态图 > 勾选散文）。
4. **落地优先级建议**：先做 Correction→Constraint 与 Feature Map/证据分级（P0），再补目录语义边界与花园除草 DoD（P1），sources 挂演讲与共享小 reference（P2）。不要照搬「禁 99% 注释」或整包引入 pstack。

---

## 1. 演讲在说什么

### 1.1 背景与主张

- 演讲者：Lauren Tan（公开场合亦称 Potato），SpaceXSI / Grok Bot（录音里作 Grabbot / graphbot）相关工程。
- 标题挑衅点：大量 agent 产出看起来像 slop；目标是在**产量暴涨时仍维持甚至提高质量**。
- 元论题：**信任（trust）**——不是盲信，而是给 agent 环境投足够多的确定性约束与验证，使人能把注意力从「每次人肉抓同一类错」挪开。
- 她给出的数量级（团队自述、非对照实验）：约一个月量级大量 PR（对外报道有「约 11k / 月」等说法；transcript 另有「三千」量级表述，以官方 talk 页综述为准），revert 率约 0.5%；强调可迁移的是**做事顺序**，不是抄数字。

### 1.2 Trust 倒金字塔（约束优先级）

| 优先级 | 层 | 内容 | 性质 |
|---|---|---|---|
| 最高 | Codebase design | 好 API、架构、重构；让错误路径尽量不存在 | 确定性 |
| 高 | Static analysis & tests | lint、类型、编译器、测试 | 确定性 |
| 中 | Verification | 跑真应用、截图/视频、状态图、形式化/属性测试等 | 行为证据 |
| 最低 | Prose | rules、skills、AGENTS.md | 仅留给需要判断力的事 |

**纠错口诀（可写成 skill 硬规则）：**

1. 能否用更好的 API / 架构**消灭**这个错误选择？  
2. 若不能消灭，能否变成 **lint / 类型 / 测试**？  
3. 最后才加一行文档或 AGENTS.md。

人肉 review 里反复出现的同一句评论 = **流程味（smell）**：说明约束还停在「软」层。

### 1.3 花园隐喻：仓库是最大输入

- Agent 输出质量高度依赖输入；**代码库本身是最大输入之一**。
- Workaround、deprecated 路径、为脏补丁写的「合理化注释」会像杂草一样被下一轮 agent **当成既有模式复制**。
- 因此：清理坏样例、收紧默认路径，是提升未来产出，而不只是还历史债。
- 人审时代可以「一棵一棵看新芽」；agent 洪峰下更要靠**地基（目录、边界、检查）**塑形生长方向。

### 1.4 Dune 案例：用框架与目录消灭「微妙 API」

内部客户端框架（讲者称 Dune，类比「Electron 上的 Next.js」）本身技术不炫；价值在于把桌面端踩坑写进**约束与 lint**。

| 问题 | 做法 | 效果 |
|---|---|---|
| `useEffect` 等需 nuance 才用对的 API，agent 易误用 | 产品代码禁掉；仅允许出现在框架内部封装 | 「整类问题从 review 消失」 |
| Renderer 误 import main 进程代码 → 长任务 / jank | 顶层目录字面命名 `electron-main` / `electron-renderer`，dependency analyzer + lint 禁跨界 | 目录给上下文，工具执行边界 |
| 用 timer 等捷径修架构错位，再写 comment 辩解 | 严格限制注释；lint 错误信息指向**根因修法** | 避免「注释 = 做错的许可证」 |

可迁移原则（不必照搬具体禁令）：

- **微妙 API**：不要指望 prose 教会 agent；提供安全出口，并机械拒绝不安全默认。  
- **目录即上下文**：危险边界用可读目录名表达，再用静态检查锁死。  
- **注释政策要本地化**：吸收的是「坏模式的辩解注释有害」，不是全球禁注释。

### 1.5 Verification：审证据，不只是审 diff

- Review 应看行为产物：录屏、截图、状态图、形式化结果等。  
- **Feature map**：从用户视角命名能力与入口（例如「侧边栏」对应哪条路径），避免 agent 猜 UI 语义。  
- 可复用 CLI / driver：既给 agent 稳定操作面，也给团队可维护的测试钩子。  
- 更深验证（模型检测、属性测试、TLA+ 等）是加层，不是银弹；经济上 agent 降低了维护成本，因此 verification 预算可以（也应该）升高。

### 1.6 先硬内环，再上外环自动化

正确顺序：

1. 让 agent 学到的代码变干净；  
2. 约束可执行；  
3. 行为 verification 可靠；  
4. **再**扩 cloud agent / 自动开修 / 自动合入等外环。

反过来：给 slop 内环加外环 = 更快更贵的 slop。工厂隐喻：工厂加速救不了坏工艺。

---

## 2. 对照：两个 skill 现在站在哪

### 2.1 已对齐 Tan 的部分

| Tan 的点 | `layered-codebase` | `product-delivery` |
|---|---|---|
| 能工具化就不写文档 | 信条 6；`templates/tooling.md`；agent-rules「确定性 vs 建议性」 | 闸门硬停、契约/活文档分层，减少「口头对齐」 |
| 结构契约可 review | 分层职责表、review-checklist 阻塞项 | S/M/L 判档、五道闸门、冻结历史包 |
| 依赖/落位可检查 | import-linter / boundaries 等工具映射 | 写码前强制读 layered-codebase |
| 要行为证据 | DoD：贴真实命令与输出 | L/M 验收闸；禁止「测试绿了 / 应该没问题」 |

### 2.2 明显缺口

| Tan 的点 | 缺口表现 |
|---|---|
| 微妙 API / 框架安全面 | layered 谈「层职责」多，几乎不谈「产品代码禁 footgun、只走框架出口」 |
| 目录即语义边界 | 有 `protocols/` 等层名；缺「危险边界目录要可读 + 按名锁 import」的叙事与模板 |
| Correction → Constraint 闭环 | 原则散落，无固定 playbook（问题→消灭/检查→错误文案→清存量） |
| 花园除草 | 有「别复制坏模式」；缺「修完常犯错误必须留下可执行约束 / 隔离坏样例」的 DoD |
| Feature map | product 有 architecture / Demo / verification 勾选；缺**用户视角可操作地图**这一持久产物 |
| 验收证据分级 | 有 verification.md；优先序未钉死（录屏/状态图 > 脚本输出 > 单测绿） |
| 产品侧「微妙开关」 | 已有「未钉角色不写长协议」等；可显式升格为「模糊配置/协议开关 = 产品 footgun」 |
| 外环自动化边界 | 两 skill 未警告「verification 未硬之前勿工厂化刷 PR」 |
| 出处 | `sources.md` 尚未挂本演讲 |

**总判**：方向一致；缺的是把 Tan 的锐度收成**可执行的升格循环与 verification 资产**，而不是再堆一倍 prose。

---

## 3. 建议如何改 skill（未实施）

### 3.1 `layered-codebase`（编码约束侧）

**P0 — Footgun / 微妙 API 一等公民**

- 在 `architecture.md` 或 `agent-rules.md` 增加「Footgun 封禁与框架出口」：  
  - 仓库维护一份「产品代码禁止 / 仅框架层可用」清单（内容本地化，不写死 React）。  
  - 同一类误用 ≥2 次 → 必须：安全封装 **或** lint 硬禁（错误信息指向正确 API）。  
- 与复杂度阶梯并列：升级不仅可以「加层」，也可以「加封禁面」。

**P0 — Correction → Constraint playbook**

- `review-checklist` 阻塞项：重复人肉评论若未升格为 API/lint/test，记为流程债。  
- 短模板：`问题 → 为何 prose 不够 → 设计消灭 or 检查拦截 → lint 文案 → 是否清理存量样例`。

**P1 — 目录即上下文**

- `repo-layout.md`：危险边界用可读目录名；配 dependency 契约示例（按边界名，不只按层名）。  
- 强调 feature/能力共置，降低 agent「就近乱塞」。

**P1 — 花园卫生 DoD**

- 引入 deprecated 必须伴随：警告 + 正确路径示例 +（最好）删除/隔离坏样例。  
- DoD：若本次修了 agent 常犯错误，是否留下**可执行**约束，而不是只改了 AGENTS.md。

**P2 — Verification 分层表述**

- `tooling.md` / DoD 区分结构证据（lint/type/分层/单测）与行为证据（app/CLI 关键路径）。  
- UI/副作用改动：仅结构绿 ≠ 可合并。

### 3.2 `product-delivery`（意图与验收侧）

**P0 — Feature Map 持久产物**

- 挂在 `docs/architecture/` 或并列约定：用户如何到达各能力、入口叫什么、验收怎么点。  
- L 体验/验收默认引用；M 动主路径时更新。  
- 对齐 Tan：减少「sidebar 是哪个文件」式猜测。

**P0 — 验收证据分级**

1. 真机操作产物（录屏 / 截图 / 状态图）  
2. 可复现脚本 + 输出  
3. 测试绿（辅助，不可单独过闸）  

写死：agent 自然语言「已验证」不算证据。

**P1 — 产品 footgun + 约束回流**

- 契约闸：禁止把「需要 nuance 才用对」的开关裸露给实现者（含 agent）。  
- flows 收尾：若因意图/契约歧义返工 ≥1 次，固化进 schema / 生成契约 / Demo 断言 / feature map，**不要只写进 brief 散文**。

**P1 — 外环边界一句话**

- 无可复用 verification（feature map + 真跑路径）前，不鼓励把 L 叙事写成「多 agent 并行刷 PR / 自动合入」。

**P2 — 防 prose 膨胀**

- 能 schema/CI 卡住的，不新增 Grill 条目。  
- `AGENTS-snippet` 继续只留判档、闸门、文档路径；编码细节外链 layered-codebase。

### 3.3 跨 skill 与其它注意

1. **可选共享小文** `constraint-promotion.md`（或两边互链同一段）：统一「设计消灭 → lint → test → 文档」顺序。  
2. **不要照搬禁注释**：吸收「辩解型注释传播坏模式」；公开 API/契约说明仍可保留。  
3. **不要整包引入 pstack**：可借鉴 verification / feature map / eval 思路；本仓库更适合抽两块（升格闭环 + feature map），而不是再嵌一套 playbook 路由器。  
4. **Skill 自身也做花园管理**：references 继续变长时，拆成「可检查清单」+「判断力说明」，避免 agent 断章取义。  
5. **`sources.md`**：两 skill 均可挂本演讲与 talk 页，标明吸收的是 trust pyramid / 微妙 API / 目录上下文 / 证据审查，而非具体 Electron 禁令列表。

---

## 4. 建议落地顺序（若开始改文）

| 优先级 | 项 | 主要落点 |
|---|---|---|
| P0 | Correction→Constraint 流程 + footgun/安全 API 节 | `layered-codebase` |
| P0 | Feature map + 验收证据分级 | `product-delivery` |
| P1 | 目录语义边界示例与 lint 模板；花园除草 DoD | `layered-codebase` |
| P1 | 产品 footgun、约束回流、外环边界 | `product-delivery` |
| P2 | sources 挂演讲；可选共享 constraint-promotion | 两边 |

改法建议：先出短 review 稿（增补段落草案），确认后再改 references，避免一次把 SKILL 主文件撑爆——这本身也符合 Tan「少 prose、多可执行」的方向。

---

## 5. 资料索引

| 资料 | 用途 |
|---|---|
| https://ai.engineer/code/2026/talks/lauren-tan | 官方 talk 页与综述 |
| https://ai.engineer/api/code/reading-download/transcripts/lauren-tan.md | 机器转写全文（有识别错误，细节以 talk 综述为准） |
| https://intelligentartifact.com/posts/building-codebases-that-constrain-coding-agents/ | 第三方摘要：约束优先于 prose |
| https://news.sulat.com/2026/11/16/lauren-tan-ai-engineer-talk/ | 第三方报道：Dune 三例、升格纠错 |
| https://blog.lionet.vn/en/posts/lauren-tan-pstack-verification-agent-engineering | pstack / verification skill 外围笔记（可选延伸，非本调研必改项） |
| `skills/layered-codebase/SKILL.md` 及 `references/`、`templates/` | 现状对照 |
| `skills/product-delivery/SKILL.md` 及 `references/`、`templates/` | 现状对照 |

---

## 6. 本文不覆盖什么

- 未验证 Grok Bot / Dune 私有实现细节；例子均来自公开演讲与二手综述。  
- 未直接修改任何 skill 文件。  
- 未主张把本仓库做成「自动合入工厂」；若要做，前提是本文 P0 verification 资产先落地。
)

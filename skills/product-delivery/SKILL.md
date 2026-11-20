---
name: product-delivery
description: >-
  需求澄清、确认 gate 与文档维护：产品层与变更层分责、S/M/L 分档、
  落地后冻结历史需求、产品架构与图示归属。当用户要定产品/MVP、开新需求、
  做小调整、讨论需求文档怎么组织、担心 agent 与用户意图不对齐、或维护 CHANGELOG/现行契约时使用。
  本 skill 管「做什么、何时停、文档活/冷」，并派角色。确认可写码之后派 roles/implement（其内读 layered-codebase）；
  请人 Approve 前须先派 roles/codereview 对照 layered-codebase，无阻塞再派 roles/accept。
argument-hint: [目标：判档 | 新产品/MVP | 开变更 | 小调整 | 冻结/维护 | 定仓库文档结构]
---

# 产品交付（Product Delivery）

管意图对齐、分档、文档寿命、确认 gate 与验收。不替代目标仓库 `AGENTS.md`，也不写编码规范。本文件只保留使用时机、判档、真相分层、硬停点和路由。

目标：

1. **人只否决，不审散文**：一句意图、一次一条决策字母；体验闸有 GUI 看可点 Demo，无 GUI 看操作/命令清单（**禁止无页面改动仍写 `demo.html`**）；验收闸先派 pd-codereview（layered-codebase），无阻塞后再派 pd-accept 交证据包，**最终过/不过仍由人**。
2. **现状与历史分开**：现行契约随代码维护，变更包落地后冻结。
3. **流程随改动伸缩**：小改走快车道，大改才走指挥者分工。

## 团队

主会话是**指挥者**。干活的是子代理，各读各的角色文件，不直接把散文交给人。硬停点、判档、对人拍板、冻结只留在本文件；角色不得宣布 gate 通过。

派发时 kickoff 第一行是 `Read <product-delivery 根绝对路径>/roles/<role>/SKILL.md`。角色文件再点名要 Read 的 pstack 叶子。不要把整个 `references/pstack/` 塞进 brief。

| 角色 | 何时 | 先读 |
|---|---|---|
| 指挥 | 始终是主会话；冻结也由指挥者做 | 本 skill，外加下面「指挥者要读的叶子」 |
| 现状 | 需要翻库估改动面、续作对齐 | [`roles/discover`](roles/discover/SKILL.md) |
| 体验 | 体验闸前：有 GUI → 可点 Demo；无 GUI → 操作/命令清单（不写 HTML） | [`roles/experience`](roles/experience/SKILL.md) |
| 契约 | 角色已钉、要起草可感知契约 | [`roles/contract`](roles/contract/SKILL.md) |
| 设计 | 分领域 design，或定稿前对照 review | [`roles/design`](roles/design/SKILL.md) |
| 实现 | 用户明确可写码之后 | [`roles/implement`](roles/implement/SKILL.md)，其内再读 [`layered-codebase`](../layered-codebase/SKILL.md) |
| 分层 review | 写码后、请人 Approve 前（有阻塞须先修再复审） | [`roles/codereview`](roles/codereview/SKILL.md)，其内再读 [`layered-codebase`](../layered-codebase/SKILL.md) |
| 验收探路 | 分层 review 无阻塞后、请人拍板前 | [`roles/accept`](roles/accept/SKILL.md) |

S：指挥者独扛，必要时才派 discover；写了码则对照 layered-codebase checklist 自检或派 codereview。M：按需 discover / experience / contract；写码后必派 codereview，再 accept。L：上表默认派，例外仍由 [`flows-L.md`](references/flows-L.md) 拥有。

**人启动，指挥者不得代跑**（可提示一句）：`arena`、`interrogate`、`swarm`、`tdd`、`create-verification-skill`、`maintain-verification-skill`。对照表在 [`references/pstack/README.md`](references/pstack/README.md)。

编排参照 [Cursor subagents](https://cursor.com/docs/subagents) 的 planner → implementer → reviewer（分层）→ verifier。纠正只升格成 lint、schema、API 或 skill，不另写一份教训清单。

### 指挥者要读的叶子

进入本 skill 后 Read 全文（路径相对本文件）：

- [`references/pstack/in-your-own-words/SKILL.md`](references/pstack/in-your-own-words/SKILL.md)
- [`references/pstack/principle-experience-first/SKILL.md`](references/pstack/principle-experience-first/SKILL.md)
- [`references/pstack/principle-never-block-on-the-human/SKILL.md`](references/pstack/principle-never-block-on-the-human/SKILL.md)
- [`references/pstack/principle-encode-lessons-in-structure/SKILL.md`](references/pstack/principle-encode-lessons-in-structure/SKILL.md)

可逆工作不拿来问人；不可逆（force-push、删数据、对客户发消息、未过 gate 的写码）仍停。同一类纠正第二次，升格成结构，不写「记住了」。

人根据探路证据与真机点验说**过**或**不过**。不过则指挥者开会：跨项目的机制改本 skill，项目禁令改该仓库的 lint / schema / API。不写会议纪要。同一类纠正第二次、或人说「开会」，同样开。

## 验收不烧钱

付费模型调用默认不发出。跑真项目时在 Provider 边界做 **trace**：拦截出站请求，留下 request body；探路证据须能说明「人触发 → 请求组好」。

真调用的模型、档位、次数只写目标仓库的 `docs/conventions/project-rules.md`。本 skill 不规定「最便宜」或次数。没有这条项目规则时，不准为了验收发出付费请求。

## 何时使用

- 定义新产品、MVP、能力分期或产品级扩展点。
- 开启新需求、调整对外行为、协议、配置或主交互。
- 做小修改，并判断该走快车道还是补对齐产物。
- 维护产品/架构/现行契约、CHANGELOG、历史变更包或文档目录。
- 检查 agent 是否越过确认 gate 、把推测当结论或把历史文档当现状。

只讨论代码分层、模块内实现与测试归属时，不要用本 skill，改读 layered-codebase。

## 接到任务时

1. **先判档**：按语义影响 × 改动面确定 S / M / L；吃不准先估改动面，仍不准就问用户一句。
2. **读取对应流程**：S/M/L 的步骤与完成标准分别只由 `flows-S` / `flows-M` / `flows-L` 拥有。
3. **到 gate 就停**：等用户明确确认；未通过不得写业务实现或扩大范围。
4. **确认可写码后**：本 skill 停在产品决定。写码派 [`roles/implement`](roles/implement/SKILL.md)。产品语义仍有歧义就停回 gate。
5. **按档收尾**：对照对应 `flows-*`。本期写了业务码 → 先派 [`roles/codereview`](roles/codereview/SKILL.md)（对照 layered-codebase）；有阻塞先修再复审。无阻塞后再派 [`roles/accept`](roles/accept/SKILL.md) 交证据包；付费边界交 trace。最终过/不过仍是人。冻结由指挥者做，不另派角色。

## S / M / L 判档

语义影响看是否改变价值机制、产品级扩展点、核心业务语义或对外承诺；改动面同时计算代码、调用方、活文档与现行契约。

| 档 | 判定 | 流程入口 |
|---|---|---|
| **S** | 语义基本不动，改动面小。可顺带改活文档；**有文档 diff 不等于升档** | [`flows-S.md`](references/flows-S.md) |
| **M** | 对外行为或协议局部变化，或改动面中等，仍是既有能力修补 | [`flows-M.md`](references/flows-M.md) |
| **L** | 新产品/MVP、新能力，改价值机制/产品级扩展点/主交互，或需要分领域设计（有 GUI 才要可点 Demo） | [`flows-L.md`](references/flows-L.md) |

探索后发现多调用点、多契约或主路径受影响就升档；实际只是一行改动加契约再生可降档。估错先停下改档，不沿用原流程硬做。

吃不准档位时只问一句。S 已无歧义不必复述。

## 文档真相分层

| 层 | 典型路径 | 何时改 |
|---|---|---|
| 产品宪法 | `docs/product/`（VISION / MVP / CAPABILITIES…） | 定位、价值机制、能力分期、扩展点承诺变化 |
| 产品架构 | `docs/architecture/`（模块、关系、关键时序、spine、ADR） | 模块边界或协作关系变化 |
| 现行契约 | `docs/contracts/current/` 或由 schema 生成 | 跟代码一起改 |
| 行为流水 | `CHANGELOG.md` | 用户可见行为或对外契约变化 |
| 历史变更包 | `docs/changes/NNNN-slug/` | 落地后冻结，只读 |
| 实现细节 | 代码 / shared schema | 控件与字段细节的最终真相 |

| 想知道 | 直接读取 |
|---|---|
| 产品是干什么的、MVP 验证什么 | `product/*` |
| 有哪些模块、怎么协作、关键时序 | `architecture/*` |
| 现在协议/配置长什么样 | 现行契约或生成物 |
| 当时为什么那样定 | 对应冻结变更包 |
| UI 有没有某个控件 | **代码** |

产品现状从 `product/*` + `architecture/*` + 现行契约 + 代码读取，不从历史变更包拼接。代码与活文档冲突时，以代码为准修活文档；历史包只加取代指针。

各层模板、变更包骨架、契约写法与图归属见 [`doc-model.md`](references/doc-model.md)；目标仓库目录树见 [`docs-layout.md`](templates/docs-layout.md)。

## 五个 gate

gate 是硬停点。「嗯 / 看看 / 差不多 / 先做着」不算明确确认。适用档位、产物、通过标准与反例只由 [`gates.md`](references/gates.md) 拥有。

| gate | 未通过时的停点 |
|---|---|
| **intent** | 不扩写产品细稿，不进入详设或写码 |
| **experience** | 不把契约定稿，不进入业务实现。无 GUI 不得以 HTML Demo 充数 |
| **contract** | 不进入分领域详设，不用实现替用户决定字段与行为 |
| **design** | 不进入编码；定稿前 review 未消化，不得请用户定稿 |
| **acceptance** | 不冻结变更包，不宣称完成。分层 review 仍有阻塞、或有界面未交 pd-accept 证据包，不得请人拍板 |

L 按意图 → 体验 → 契约 → 设计定稿 → 编码 → 分层 review → 验收推进；M/S 的简化适用规则见 `gates.md`，不在主文件另建流程。

## 最高优先级红线

- 未判档、该估改动面却没估，不改产品行为；指代不清先问，不擅自选常见默认。
- 未获明确确认不写业务实现，不「先跑一版再对齐」。
- Grill 一次一问；已确认、建议、待确认必须分开，不把推测写成事实。
- 不回写 Frozen 变更包正文，不推荐历史包作为现行产品说明。
- 对外契约变了必须同步现行契约；共享形状变更相关面（CLI/HTTP/Web/shared）一次对齐；禁止同一 version 内双轨软兼容（对外兼容用显式 version → 分路 validation/parse）；长期行为图必须提升到 `architecture/`。
- 不声称未调查的能力；能用可配置参数档表达的分类，不要写成自由文本猜测或临时代码表。
- 交互 Demo 只确认操作与文案心智，不是视觉规格或成品控件清单。**无 GUI 表面禁止 `demo.html`**；用 ops-inventory + interaction 过体验闸。
- 角色边界与目标 Agent 宿主能力未钉，不写长协议或预发明未经验证的通信开关。
- 对人只呈现一句意图、一次一条 A/B/C；验收给人短证据指针，不请人审散文。`brief` / design 的读者是干活的 agent。
- 本期写了业务码却未派 pd-codereview、或仍有 layered-codebase 阻塞项，不得请人 Approve / 说「待验收」。
- 有界面未交 pd-accept 证据包（无界面时无脚本输出）不得说「待验收」。测试绿、分层 review 无阻塞，都不能单独过 acceptance gate。**最终过/不过必须是人**。
- 付费验收默认 trace。能否真调用、用什么档，只看该仓库 `docs/conventions/project-rules.md`。
- 验收探路优先已有 verify/control 脚本，否则 agent-browser（加命令/日志证据）。不默认录屏。
- 人说不过、打断、或同一纠正第二次：开会升格。禁止原样重试，禁止只写「记住了」。
- 项目专有规则写 `docs/conventions/project-rules.md`。待修事项写 `docs/issues/`（带 frontmatter；**不维护索引表**；关单删文件；列表用 `scripts/list-doc-props.mjs`），由用户写，或用户叫 agent 写。
- 本 skill 维护的文档禁止手写「文件→摘要」索引；发现靠路径命名 + props + 上述脚本。
- L 的长产出默认派对应角色，主会话只留决策、gate 状态与产物指针。角色不得自行开启 arena / interrogate / swarm / 新建 verify skill。

完整行为约束见 [`agent-rules.md`](references/agent-rules.md)。

## 按任务跳转

只加载当前任务所需文件；进入新阶段时再补读，不一次展开全部 references。

| 任务 | 必读 |
|---|---|
| 执行 S / M / L 流程、检查对应 DoD | [`flows-S.md`](references/flows-S.md) / [`flows-M.md`](references/flows-M.md) / [`flows-L.md`](references/flows-L.md) |
| L 的 A–H 阶段、子代理默认派与例外 | [`flows-L.md`](references/flows-L.md) |
| Grill 提问、方案对比、差异化回检 | [`discovery.md`](references/discovery.md) |
| 判断 gate 能否通过、确认话术 | [`gates.md`](references/gates.md) |
| Agent 红线、子代理交接、交付回复 | [`agent-rules.md`](references/agent-rules.md) |
| 文档分层、模板、图归属、变更包结构 | [`doc-model.md`](references/doc-model.md) |
| 流程/文档 review、冻结前检查 | [`review-checklist.md`](references/review-checklist.md) |
| 已经确认可写码 | 派 [`roles/implement`](roles/implement/SKILL.md)（其内读 layered-codebase） |
| 写码后、请人 Approve 前 | 先派 [`roles/codereview`](roles/codereview/SKILL.md)；无阻塞再派 [`roles/accept`](roles/accept/SKILL.md) |
| 派某个角色、查该读哪片 pstack | [`references/pstack/README.md`](references/pstack/README.md) 与 `roles/*/SKILL.md` |
| 接入仓库目录或 AGENTS.md | [`docs-layout.md`](templates/docs-layout.md) / [`AGENTS-snippet.md`](templates/AGENTS-snippet.md) |
| 查看规范出处（运行时不必读） | [`sources.md`](references/sources.md) |

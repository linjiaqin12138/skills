# L：完整澄清车道

适用于：新产品/MVP、新能力、改价值机制或产品级扩展点、主交互重做，或改动面大到需要 Demo + 分领域设计。

gate细则见 [`gates.md`](gates.md)。拷问题库见 [`discovery.md`](discovery.md)。文档模板见 [`doc-model.md`](doc-model.md)。  
子代理行为红线与交接要求见 [`agent-rules.md`](agent-rules.md)；L 的编排细则由本文件拥有。

典型请求：从零 MVP、同级新能力、重做确认模型或主交互。

## 阶段总览

| 阶段 | 主代理 | 默认派角色 | 停等确认 |
|---|---|---|---|
| A Grill | 提问、复述、记决策 | — | intent gate |
| B 范围落盘 | 写/改 product、brief；必要 architecture 意图 | [`pd-discover`](../roles/discover/SKILL.md)（需翻库时） | brief/产品范围 |
| C 交互与 Demo | 定操作清单、验收路径、experience gate话术 | [`pd-experience`](../roles/experience/SKILL.md)；大流程图 | experience gate |
| D 契约 | 先钉角色；对齐可感知契约 | [`pd-contract`](../roles/contract/SKILL.md) | contract gate |
| E 设计 | Grill 待决、合并 design；消化 review | [`pd-design`](../roles/design/SKILL.md) | design 定稿 |
| F 编码 | 集成与歧义升级 | [`pd-implement`](../roles/implement/SKILL.md)（大项目） | （已确认可写码） |
| G 验收 | 主持「过/不过」 | [`pd-accept`](../roles/accept/SKILL.md) | acceptance gate |
| H 冻结 | 标 Frozen、提升图、CHANGELOG | — | — |

列名是 **默认派**，不是「可有可无」。例外见下表。

## 编排细则

主会话只保留**决策与指针**：确认了什么、文件在哪、卡在哪道闸、下一句该问用户什么。产出很长且对用户只需回「路径 + 怎么用」的子流程默认派；子代理交回短报告 + 路径，主代理不把整份 HTML 或大 diff 念进主对话。

| 子流程 | L 默认 | 例外（可主代理做） |
|---|---|---|
| explore 估改动面 / 现状报告 | **派** [`pd-discover`](../roles/discover/SKILL.md) | 改动面已一眼清晰、无需翻库 |
| **整页 / 多分支可点 Demo**（新建或按新意图重做） | **派** [`pd-experience`](../roles/experience/SKILL.md) | — |
| 流程图/状态图大稿 | **派** experience，或 `draw-diagram` 子任务 | 改已有 Mermaid 两三行 |
| 起草 events/api/config 契约表 | **派** [`pd-contract`](../roles/contract/SKILL.md) | 改已有表一两行 |
| 分领域 `design-*` | **派** [`pd-design`](../roles/design/SKILL.md)（按需并行，禁止凑齐） | 用户确认只需一个极短领域且半页内 |
| **design 定稿前：对照代码+文档的矛盾 / 未决可感知点 / 写码歧义 review** | **派** `pd-design` | 无例外 |
| 实现切片 | **派** [`pd-implement`](../roles/implement/SKILL.md)（大项目） | 单文件级补丁 |
| 验收探路（证据包；有界面才用 agent-browser） | **派** [`pd-accept`](../roles/accept/SKILL.md) | 无界面时主代理跑脚本即可 |
| 验收勾选跑脚本 | 可派 accept | 主代理主持「过/不过」对话 |
| 已有 Demo 上的**小补丁**（加一按钮、改文案、加一条已确认分支） | 主代理改即可 | 若补丁牵动整页结构 → 仍派 experience |

`arena` / `interrogate` / `swarm` / `tdd` / `create-verification-skill` / `maintain-verification-skill` 只在用户本轮点名后，由对应角色 Read `references/pstack/` 里的原文。指挥者不代跑。

「可派」不得读成「爱派不派」。L 整页/重做 Demo 必须派；prompt 写明 Demo 只做交互确认、标出「仅演示」控件，且不把整页 HTML 交回主会话。事事派与把长产出堆主会话都属于偏离编排。

## A — Grill 拷问

1. 读 `product/*`、`architecture/*`。
2. 按 [`discovery.md`](discovery.md) 一次一问；需要时给 2～3 方案。
3. 结论写入变更包 `design-decisions.md` 的决策清单。对人只发当前这一条 A/B/C。
4. **intent gate **通过后再进入 B。brief 给人读不算通过。

## B — 范围落盘

1. 更新或创建 `product/VISION`、`MVP`、`CAPABILITIES`（仅宏观变化时）。
2. 若模块清单有变，更新 `architecture/overview` 的意图描述（可先粗）。
3. 开 `changes/NNNN-slug/brief.md`。
4. 需要「现在代码怎样」时**派 explore**，主代理读短报告，不自己沉进全库。
5. 人确认那句意图后再进 C。不请人审 brief。

## C — 交互与 Demo

1. 主代理写/更新 `ops-inventory.md`（阶段 × 动作 × 所见 × 系统反应）——保持短，这是gate输入。
2. `interaction.md`：流程/状态；大图可派，小改主代理。
3. 价值路径仍糊：可用 **`processize`** 先捋手搓步骤。
4. **可点 Demo（整页新建或按新意图重做）→ 默认派 [`pd-experience`](../roles/experience/SKILL.md)**。主代理不要改成自己做。
   - 主代理只给：已确认约束、差异化句子、ops 要点、输出路径、README 要求、禁止事项。
   - prompt 必须写明：只表达交互与文案心智；可以丑；效果可以模拟；演示专用控件在 README 标「仅演示」；**不要**交回整页 HTML。
   - 子代理交回：路径 + README + 主路径怎么点。低保真。多分支在 README 写清。
5. **已有 Demo 的小补丁**（单按钮、文案、已确认的一条分支）：主代理改即可；若牵动整页结构 → 仍派。
6. Demo **不是**视觉规格，也不是成品控件清单。交互只抽原则（如软跳、确认闸、反馈闭环）。成品视觉另开简规或设计向任务。主导航已够用仍加的演示捷径，不得默认进实现。
7. 文案用用户差异化句子回检，不套竞品/旧品类默认（关卡式流程、缺中间产物就锁导航、照搬其按钮文案）。
8. **experience gate**：给人 Demo 路径 + 怎么点；人说「过」或改哪几点。低保真可点 Demo（含「仅演示」模拟按钮）给人路径即可。仅当 Demo 刻意做成与成品 UI 一致、靠 HTML 动画模拟交互时，才另附体验视频（直接出动画页，不派 agent 点网页）。验收闸（G）派 [`pd-accept`](../roles/accept/SKILL.md)。细则见 [`gates.md`](gates.md)。

## D — 契约

1. **先钉角色**（写入 `design-decisions.md`）。未钉不写长协议：
   - 谁改配置、谁执行对外/付费动作、GUI 做什么。
   - 有 Agent：先问目标宿主 **实际能力**（多数只有 stdio，无可靠双向长连接）。默认 one-shot 等待 + 标准输出；不可达时用粘贴指令兜底。
   - 禁止默认 Agent 经 HTTP 调本机服务、直连第三方、自己充当执行器报进度；禁止先发明未经验证的通信开关或推送通道。
2. 从**操作清单与已确认交互原则**反推可感知契约——**起草默认派 [`pd-contract`](../roles/contract/SKILL.md)**；主代理做对齐与 contract gate。演示专用控件不进契约。写法见 [`doc-model.md`](doc-model.md) §3。
3. 草案可放变更包 `contracts/`。配置 vs 运行快照、全局 vs 工程目录、可复用参数档与单次覆盖等概念分开写（名称随项目，原则通用）。
4. **contract gate **通过后再进 E。写法与禁令见 [`gates.md`](gates.md)、[`doc-model.md`](doc-model.md) §3。

## E — 分领域设计并汇总

1. 补全 Grill 待决（技术取舍）；更新 `design-decisions.md`。
2. 用户确认**无遗留异议**、同意拟派领域清单之前，不派 designer。
3. 按需并行派 [`pd-design`](../roles/design/SKILL.md) 写 `design-api.md` / `design-config.md` / `design-module.md` 等；禁止凑齐全派。
4. Mind-set：能简单则简单；扩展点写边界，不预埋空框架。
5. 主代理合并短 `design.md`（仅含本期领域）。design 若已改对外行为，先改契约再进入下一步。
6. **定稿前 review（默认派 `pd-design`，禁止跳过）**——对照：
   - 本期变更包（brief / decisions / design* / 契约草案 / ops）
   - 活文档（`product/*`、`architecture/*`、现行契约）
   - **相关存量代码**（实现真相对齐；冲突以代码为准标出，并建议改哪份活文档）

   只查三类，交回短报告（路径 + 分条列表，勿长文）：
   1. **文档矛盾**：变更包内部、变更包↔活文档、文档↔代码不一致。
   2. **可感知点未决策**：外部可感知行为（形状、错误码、HTTP/CLI/磁盘/WS/退出码、角色边界）仍含糊或标成待确认却当已定。
   3. **写码歧义**：两名实现者会做出不同实现的点（缺默认、缺边界、指代不清、Demo「仅演示」与产品控件未拆清等）。

   主代理：先修文档，或一次一问补决策清单；**不得**带着未消化的阻塞项请用户开始写码，也不得请人读 design。短报告必须交回，可落盘 `plans/design-preflight.md`。
7. 用户说 **开始写码** → 才允许 F。

## F — 编码

确认可写码后，派 [`pd-implement`](../roles/implement/SKILL.md)（其内读 layered-codebase）。产品语义有歧义就停回 gate。

实现中改了对外契约或模块边界，同步现行契约 / `architecture/`。

另核：成品视觉独立于 Demo；演示专用控件未当产品控件。产品若承诺 Agent SKILL，权威路径在本阶段或 H 按仓库约定落地（权威路径 + 可选工具镜像，文件名不写死在本 skill）。

## G — 验收

1. 派 [`pd-accept`](../roles/accept/SKILL.md)。有界面：已有 verify/control 脚本优先，否则用 **agent-browser** 点通主路径，交短证据（截图路径、settle 文案、命令/generation 状态）。无界面：可复现脚本输出。付费边界按 [`gates.md`](gates.md) 走 trace。真调用只按该仓库 `docs/conventions/project-rules.md`。
2. 另核：「仅演示」控件没有变成产品控件。每条已回字母一行：`Dn=字母 仍成立，因为 <证据指针>`。对不上就停。
3. 主会话把短证据交给人；**人说「过」之后再 H**。agent 探路 ≠ 通过。测试绿不能代替人拍板。未探路不得说「待验收」。

## H — 冻结

触发：人说**过**（可结合探路证据与自己真机点验）。未过不得 Frozen、不得宣称完成。人说不过则开会，不原样重试。

1. 变更包 `brief` 状态 → **Frozen**；gate进度写明验收通过（勿停在 Implementing）。
2. 本包推进的 CAP：`docs/product/CAPABILITIES.md` **Next → Launch**（或 brief 目标分期）；MVP 若仍写该能力「未实现」须改掉。
3. 同步 `contracts/current`（若草案有增量）。
4. 长期有效的模块图/时序 **提升**到 `architecture/`。
5. `product/*` 宏观若变了已在 B 处理；此处复核分期与说明是否与 Frozen 一致。
6. CHANGELOG。
7. 若承诺交付 Agent SKILL：核对权威路径已在仓库；回复里点出路径。
8. 若仓库有「Approve 后收尾」清单（如 `AGENTS.md`），按其执行（格式化 / 生成物 / 全量检查等）。
9. 对人只报：探路证据指针、`brief Frozen，CAP-x Launch`、本次动了哪条验收点。不附散文。

## 完成标准

- A–E 的拍板字母已回，核对报告已消化。
- 有界面则 agent-browser 探路证据已交，且人已说「过」；视觉未从 Demo 照抄，演示专用控件未进产品。
- 付费边界有 trace。若发生真调用，符合该仓库 `project-rules.md`。
- 现行契约一致，长期图已入 `architecture/`，`product/*` 宏观变化已更新。
- 变更包已 Frozen；关联 CAP 已 Launch（若适用）；CHANGELOG 已写。
- 若承诺交付 Agent SKILL，权威路径已按仓库约定落地。
- 若有人工纠正，已升格，不是「记住了」。写码是否做完不在本清单。

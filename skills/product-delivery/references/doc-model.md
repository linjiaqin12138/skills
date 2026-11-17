# 文档模型与模板

本文定义各层文档**写什么、不写什么**，以及建议章节。路径相对于目标仓库 `docs/`（或仓库约定的等价根）。

## 1. 产品层 `product/`

### `VISION.md`

- Elevator pitch（一句话：为谁、做什么）
- 核心痛点与当前 workaround
- 价值主张（We help X do Y by Z）
- 明确不做（反目标）
- 关键假设与风险（可短）

不写：API 字段、目录树、某次迭代进度。

### `MVP.md`

- 要验证的价值机制（机制，不是功能清单）
- In / Out
- 成功标准（3～5 条可现场验收）
- 产品级扩展点（边界承诺，不预埋空框架）
- 开放问题（未拍板的）

可内嵌一张主流程 Mermaid。细节步骤放变更包或 `architecture/flows/`。

### `CAPABILITIES.md`

- CAP-xx 列表：名称、用户能做什么、**入口**（用户怎么碰到、改状态用哪条命令或路由）、分期（Launch / Next / Later）、依赖；**对应变更包验收通过后，将该 CAP 从 Next 改为 Launch**（Later 调研包除外）
- 下游变更包链接表（可选）

### `GLOSSARY.md`（可选）

稳定术语：只收跨文档会歧义的词。

## 2. 架构层 `architecture/`

### `overview.md`

- 模块/应用表：名称、职责一句话、代码位置（可选）
- 模块关系 Mermaid（依赖方向）
- 运行实体（进程/CLI/浏览器等）若与模块不完全重合，单独一小节
不写：编码分层教程，也不写只对某一个产品成立的进程模型或模型档位。那些放 `docs/conventions/project-rules.md`。

### `spine.md`

只记「两人独立实现会冲突」的决策，例如：

- 逻辑状态以谁为准
- 谁可以写核心数据
- 扩展点边界
- 同步 vs 异步默认

每条宜有稳定 ID，供变更包引用。

### `flows/*.md`

一篇一个关键时序：前置条件 + Mermaid `sequenceDiagram` + 失败/兜底一句。宁缺毋滥。

### `adr/NNNN-*.md`

标准 ADR：背景、决策、后果。状态：Proposed / Accepted / Superseded。

## 3. 现行契约 `contracts/current/`

按仓库需要拆分，常见：

| 文件 | 内容 |
|---|---|
| `events.md` | 事件名、方向、载荷、触发的用户动作、刷新依据 |
| `api.md` 或生成的 openapi | HTTP/CLI 路径与形状 |
| `config.md` | 配置项、谁设置、默认、是否进持久状态、用户是否可见。不含运行时快照 |

若契约由 schema 生成：手写文件只保留「生成方式 + 入口路径」，避免双源。

### 契约怎么写

读者注意力窄：**短、少无关、不堆扩展预埋**。

| | |
|---|---|
| **可丢** | 重复、废稿、架构空谈、实现猜测 |
| **不可丢** | 数据形状、错误码，以及 HTTP / CLI / 磁盘 / WS / 退出码等一切外部可感知行为 |
| **禁止** | `null` 键；无调用方的 API；把 pid、port 等运行时快照写进配置；为「以后」堆字段 |

分开写，不要挤在一节：配置 vs 运行时快照；全局配置 vs 工程目录；可复用参数档与单次覆盖（名称随项目）。

精简后担心丢了：派子代理只审「外部可感知缺口」。不要靠把文档写长。

角色边界（谁改配置、谁执行生成、GUI / Agent 各做什么）写在 `design-decisions` 或 `spine`，不要散进每条 API 的注释里再讲一遍。Agent 通信按 CLI **实际能力**写；做不到的通道不要先设计开关。

## 4. `CHANGELOG.md`

按日期追加。用户可见行为或对外契约变化写一行；纯内部 refactor 可省略。

```markdown
## YYYY-MM-DD
- web: …
- api: `foo` 重命名为 `bar`（见 contracts/current）
```

## 5. 变更包 `changes/NNNN-slug/`

只描述**这一次**对齐过的变更。落地后 Frozen，不回写正文。

| 文件 | 何时要 |
|---|---|
| `brief.md` | M/L 必有；给编码 agent。人不审 |
| `ops-inventory.md` | L 交互澄清时 |
| `interaction.md` | L；含讨论用 flowchart/state |
| `design-decisions.md` | L 经 Grill 后 |
| `design-*.md` / `design.md` | L 分领域与汇总 |
| `prototypes/` | L experience gate 。交互 Demo + README：确认操作与文案，可以丑。不是视觉规格，不是成品控件清单。演示专用控件标「仅演示」。视觉方向另文 |
| `contracts/`（草案） | 讨论期；Frozen 后以 `current`+代码为准 |
| `plans/verification.md` | L/M 验收对照 |

### `brief.md` 骨架

```markdown
# NNNN：标题
- 状态：Draft | Confirmed | Implementing | Frozen
- 档位：M | L
- 日期：

## 背景与目标
## In / Out
## 验收标准
## 关联
- product / architecture / CAP：
- 取代或相关变更包：（仅指针）
```

`brief` 的读者是编码 agent，不是人。

### `design-decisions.md` 骨架

```markdown
# 设计前置结论：NNNN

## 决策清单
| # | 拍板点 | 推荐 | 结论字母 |

## 待决问题与结论
| # | 问题 | 结论 | 备注 |
|---|---|---|---|

## 约束（供领域设计）
- 本期范围：
- 明确不做：
- 拟派领域：api / config / module / …（按需）

## 遗留项
无 | …
```

### `ops-inventory.md` 表头

| # | 阶段/界面 | 用户动作 | 看到什么 | 系统应发生什么 | MVP？ |

## 6. 图与寿命

图跟真相寿命走，优先用 Mermaid 嵌入 Markdown。画图时可使用 **`draw-diagram`** skill 选型与产出。

| 图在说什么 | 放哪 | 寿命 |
|---|---|---|
| 产品价值/阶段主流程 | `product/MVP.md` 或 `CAPABILITIES.md` 内嵌 | 随产品宪法 |
| 模块框、依赖方向、部署/进程关系 | `architecture/overview.md` | 现行 |
| 跨模块关键时序 | `architecture/flows/*.md` | 现行 |
| 本次变更讨论中的时序/状态 | 变更包 `interaction.md` / `design.md` | 验收后若成长期行为 → **提升**到 `architecture/flows/` |
| API/事件字段说明 | `contracts/current/` | 现行 |
| ADR 背景示意 | `architecture/adr/*` | 随 ADR |

禁止把 Frozen 变更包当现行架构唯一出处，或在 `product/` 里画实现级类图。变更包内的图在 Frozen 前若成为长期行为，复制或改写进 `architecture/`，变更包内保留快照即可。

## 7. 命名

- 变更包：`NNNN` 四位递增 + kebab-case slug；不复用编号。
- 目录名可用 `changes/` 或历史遗留的 `requirements/`；语义都是「变更包」，在 conventions 里声明一种即可。

## 8. 项目规则 `conventions/project-rules.md`

只写这个仓库成立、别的仓库不该继承的规则。例如付费模型档位和次数、进程怎么起、热更新端口是不是产品终态。

本 skill 不抄这些句子。agent 遇到付费验收或运行形态时先读这份文件。没有写明的付费调用，验收时不发出。

## 9. 待修 `issues/`

`docs/issues/NNNN-slug.md`。用户自己写，或用户明确叫 agent 写。一条一个文件：现象、期望、相关路径。不写进产品宪法，也不当成已确认的需求。

agent 只在用户点名某条 issue 时去修。修完把该文件标成 done，不删历史。

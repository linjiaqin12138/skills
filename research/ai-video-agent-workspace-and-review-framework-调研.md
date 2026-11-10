# AI 视频 Agent 工作区 + 审查框架调研与设计

> 调研日期：2026-11-10
> 性质：方向定稿 + 设计决策。三个岔路口（小服务 / 自写模板 / 阻塞方式）已按讨论结论落成**决定**，不是待选项。想改任何一条，单独说明即可。
> 关联仓库：`skills/short-drama/`（执行层）、`skills/video-generation/`（视频模型封装）、`vendor/answer-me-with-html/`（审查语义参照）、`vendor/OpenMontage/`（对照）、Pixelle-Video（对照）。

---

## 一句话结论

1. **需求成立、市场空白**：现成的开源/AI 视频工具都没有「在 GUI 里 review 中间产物 + 指定改动点 + 审批」的闭环。OpenMontage 是只读看板(官方明说 v1 只读)，Pixelle 是死板 workflow + 无审查面，LTX Studio 做到逐镜审查但闭源不可改。
2. **机制已被验证**：你设想的「agent 产出结构化 JSON → 本地网页渲染 → 用户标注 → 裁决流回 → agent 修订」在**代码规划**领域已有成熟参照 —— Beacon(阻塞式 propose/approve)、MeowKit visual-plan(JSON+validate+gate+本地 studio)。
3. **你已领先两步**：执行层 `skills/short-drama/`(tasks.json + 单镜重跑 + workspace)和审查语义 `answer-me-with-html`(Remark 四分类 + Reply 打包)都已跑通。缺的核心只有一根「反馈写回 workspace」的管子和一层「公共信封 schema」抽象。
4. **核心设计原则**：**审查同构、生成异构**。用「公共信封 + 按类型载荷」替代「每种类型一套完全独立 schema」，类型按**产出机制**分(四个)而非按题材分，闸门**稀疏且可绕**。

---

## 1. 现状与缺口：为什么现成工具都不行

| 工具 | 形态 | 中间产物审查 | GUI 指定改动 | 审批闭环 | 可改代码 |
|------|------|--------------|--------------|----------|----------|
| **OpenMontage** | agent + 只读 Backlot 看板 | ✅ 看板展示 | ❌ 无写回(v1 只读) | ⚠️ 靠 chat 回复审批 | ✅ 开源 AGPL，但目录结构庞大 |
| **Pixelle-Video** | Streamlit 三栏配置 | ❌ 只显示进度 | ❌ tool 死板 | ❌ 一次性跑完 | ❌ 别人的项目，Streamlit 上限低 |
| **LTX Studio** | 闭源 Web 平台 | ✅ 逐镜 storyboard | ✅ (平台内) | ✅ | ❌ 闭源 |
| **可灵 / 即梦 / Runway** | 闭源在线 | 部分(下载后自己审) | 受限 | ❌ 无工作流级审批 | ❌ |
| **ComfyUI + 开源模型** | 节点工作流 | ❌ 非导演式 | ❌ 节点不是分镜 | ❌ | ✅ 但形态不符 |

**对比结论**：OpenMontage 在理念分层上最接近(确定性脚本工具 + Markdown 知识 + agent 决策)，它输在审查面没有写回；Pixelle 输在把 workflow 和创意决策焊死，正是我们要避免的反面教材。

---

## 2. 借鉴：已被验证的「agent 提议 → 网页审查 → 裁决回流」机制

### 2.1 Beacon（最直接的对标）
https://www.trybeacon.sh/docs

- agent 调 MCP 工具 `beacon_propose_plan`，**该工具阻塞会话**直到用户裁决。
- 方案渲染到本地页面(`/plan`)：左侧批注面板 + 右侧可编辑画板。用户**内联标注、直接编辑画板** → 点 Approve / Submit feedback / Discard。
- **裁决直接流回终端会话**；反馈则 agent 修订后重新 propose，循环继续。本地优先，数据在 `~/.beacon`。
- **可搬走的机制**：本地小服务 + 阻塞等待 + 标注打包反馈 + 修订循环。

### 2.2 MeowKit visual-plan（JSON 为骨 + 校验 + gate）
https://docs.meowkit.dev/cli/commands

- 规范产物是 `plan.json`(带 schema 版本 `visual-plan/v1`)。
- `validate`(schema + 覆盖闭合校验)、`gate`、`edit`/`view` 起临时 127.0.0.1 React studio 渲染**并编辑**该 JSON。
- **可搬走的点**：JSON 是唯一真相源、schema 带版本号、确定性命令负责校验与门控、本地 studio 是瞬态渲染层。

### 2.3 你仓库里已有的半成品（最应该抄自己）

| 已存在的 | 对应你设想的哪块 |
|----------|------------------|
| `skills/short-drama/pipeline.yaml` 的 `human_approval_default` + `lib` 的 checkpoint gate | 审批闸门 |
| `skills/short-drama/` 的 `tasks.json` + `prompts.md` + `produce_episode.py --shot xx --force` 单镜重跑 | 执行层 + 单镜重跑(最关键价值) |
| `skills/short-drama/SKILL.md` 的逐镜验收清单 | 审查条目模板 |
| `answer-me-with-html` 的面板评论、Remark 四分类(suggestion/keep/question/concern)、Reply 打包 | 审查交互语义 |

---

## 3. 设计：公共信封 + 按类型载荷

### 3.1 为什么不是「每种类型一套独立 schema」

用户最初的直觉是不同视频任务 JSON schema 完全不同。**这个直觉对了一半**：生成机制确实异构(`remember-me` 是代码渲染，小猫医生是视频模型，数字人口播是素材重组+旁白)。但如果让类型和 schema 完全独立，维护成本成**乘法关系** —— 每加一个类型要写新 schema + 新 review HTML + 新 script。这会在更高一层重复「脚本一次性、工程化差」的老问题。

**关键洞察：审查交互是同构的，生成机制才是异构的。**
「给我看这一项 → 我标记它 → 你重做它」对所有视频类型完全一样；不同的只是「一项具体是什么、怎么生成」。

### 3.2 结构

**信封(所有类型共享，由审查渲染器统一吃)：**

```text
workspace envelope JSON
├── workspace_id
├── schema_version         # 如 v1，MeowKit 风格，翻旧 workspace 能跑
├── video_type             # a / b / c / d（见 §4）
├── current_stage          # 当前停在哪个闸门
├── approval_status        # awaiting | pending_round_N
├── cost                   # 累计花费
├── items[]                # 审查项数组，每一项：
│   ├── id
│   ├── kind               # 参考图 / 分镜提示词 / 逐镜成片 / 场景 / 选段…
│   ├── status             # ok / to_regenerate / pending
│   ├── artifact_path / thumb_path
│   ├── prompt / params    # 这项的提示词和接口参数
│   ├── cost
│   └── feedback[]         # 用户标注，写回 workspace（§5.1）
└── verdict.json           # 用户裁决：approve | feedback | discard
```

**载荷(按类型的细节展示 + 生成方式，per-type 只写这个适配器)：**
- type 差异体现在：`items[].kind` 的取值集合、单个 item 怎么被渲染展示、单个 item 靠什么脚本生成。
- 一个审查渲染器吃信封，每种类型只提供一个「载荷细节怎么展示 + 怎么驱动生成脚本」的适配器。

**错误信息：要么拒绝干净，要么按轮归档。** schema 带 `schema_version`，`validate` 失败时明确报「这个 workspace 是旧版」，不静默跑挂。

### 3.3 收益
- 新增类型只写一个适配器(载荷 + 生成脚本)，不重写审查渲染器。
- 审查交互(缩放图、逐项标记、Approve/反馈)写一次，所有类型复用。
- 用户可翻旧 workspace 的 `review/round-N.json` + envelope，原样重新驱动生成。

---

## 4. 视频类型按「产出机制」分，不按题材分

用户列了小猫医生、今天去爬山、sleep-history、人物替换、数字人口播、动作迁移、remember-me 等。按题材是七八个，按**产出机制**只有四个：

| 机制 | 覆盖案例 | 审查对象重点 | 生成脚本形态 |
|------|----------|--------------|--------------|
| **A. 视频模型逐镜生成** | 小猫医生、今天去爬山、短剧 | 参考图、分镜提示词、逐镜成片、跨镜一致性 | `video-generation` 类封装，逐镜 task + 参考图 |
| **B. 代码确定性渲染** | remember-me、hyperframes/remotion MV | 场景 JSON、关键帧、布局安全区、跨界变形 | HTML/Remotion host + Playwright 逐帧 + 抽帧 |
| **C. 素材重组 / 解说** | 数字人口播、视频总结、montage | 选段、旁白、时间轴、字幕时序 | 检索/转写 + TTS + ffmpeg 拼接 |
| **D. 身份 / 动作迁移** | 人物替换、动作迁移 | 逐帧身份一致性、场景保留度 | 替换/动作参考专用调用 |

猫、爬山、睡眠史**不是 type，是 workspace 里的内容**。四个 type 覆盖全部案例，维护得动；十个就崩。

⚠️ 这四类是**映射表**，不要求第一步就全部实现。可以先做 A(你有最多沉淀)，其余按需迭代。

---

## 5. 三个岔路口（已决定）

### 5.1 岔路一：不碰浏览器 localStorage —— 起一个 127.0.0.1 小服务

**决定：起小服务，让批注写回 workspace。** 理由是你的 `answer-me-with-html-用法与踩坑记录.md` 第 52–57 行写得很清楚：

> 反馈存在浏览器 localStorage，key 为 `am-reply:<页面路径>`，不写回 HTML 文件。按源隔离、按路径隔离、`--replace` 重渲后旧路径下的批注成孤儿、只在当前浏览器。

这个对**迭代式审查是致命的**：每轮重渲都丢掉上一轮批注，翻旧 workspace 看不到当时的审查记录 —— 直接违反「用户可以翻出旧 workspace 重新调整生成」。

实现：HTML 里点选/批注 + Approve 按钮，经小服务写入 `workspace/review/round-N.json` 与 `workspace/verdict.json`。批注进文件、可归档、可翻旧账 —— 这正是 Beacon 和 MeowKit 的做法(MeowKit 明确 `edit`/`view` 起本地 studio)：
- 绑定 `127.0.0.1`，TrustedHost 校验(照抄 Backlot 的防 DNS rebinding 思路)。
- 端口默认固定(如 4750)，多个 session 复用一个服务，数据按 workspace 隔离。
- 纯本地，不碰任何远端。

### 5.2 岔路二：媒体审查用自写模板，方案评审继续用 answer-me-with-html

**决定：分开。**
- **媒体联系单(参考图网格 + 逐镜成片 + 内联视频 + 逐项标记控件)**：这是 am 的 Markdown 面板模型表达不了的，**自写 HTML 模板**，直接读 envelope JSON 渲染，控件直接写回 §5.1 的接口。
- **方案评审(第一轮给用户看创意/剧本/费用，纯图文)**：继续用 `answer-me-with-html`。它的 **Remark 四分类语义(suggestion/keep/question/concern)和 Reply 打包格式设计得很好，概念直接复用**到自写模板里。
- 这样 am 负责「图文方案面」，自写模板负责「媒体审查面」，边界清晰。

### 5.3 岔路三：不用 MCP，用轮询拿到阻塞语义

**决定：不引入 MCP，用脚本轮询。** Beacon 的阻塞体验明显优于 OpenMontage 的「写 awaiting_human 然后结束回合」，但阻塞通常要 MCP，而目前是 skill + script 体系。
- 提供 `review.py --wait`：轮询 `workspace/verdict.json`，用户点 Approve 时由小服务写入，脚本返回裁决。
- agent 调用它就自然挂住，无需 MCP，达到同样的「阻塞会话等裁决」效果。
- 裁决值域对齐 Beacon：`approve | feedback | discard`。

---

## 6. 架构骨架与职责边界

### 6.1 组件

```text
agent(思考、拆分、调整)
  │  写出
  ▼
envelope JSON(唯一真相源，带 schema_version)
  │  校验
  ├── validate.py          # schema + 引用覆盖 + 门控闸点检查
  ├── render_contact_sheet # 自写模板：读 envelope → 媒体审查 HTML
  ├── review server        # 127.0.0.1 小服务：接受批注写回 verdict/review-r
  │   ▲
  │   用户
  ├── generate_<type>.py   # 按类型的生成脚本(python 确定性封装 provider/ffmpeg/渲染)
  ├── compose.py           # 拼接、字幕、bgm、封装
  └── skills/<knowledge>   # 无定式技能(审查清单、taste 方向)，不在代码里
```

### 6.2 五件事归骨架，其余全留给 agent

骨架只管 5 件事：**结构(schema) / 校验(validate) / 审查(渲染+标注) / 执行(确定性脚本) / 归档(workspace + round 快照)**。
其余(题材选择、创意、镜头怎么拆、风格)全部留在 agent 思维和 JSON 内容里。

### 6.3 最大风险 & 硬防线

**风险：做成另一个 Pixelle。** 一旦骨架开始承载创意决策(模板库、风格预设、workflow 编排)，就会重复 Pixelle 的死板/有 bug/改不动。

**防线：**
1. 骨架**不存创意**，只存 agent 产出物。
2. agent 随时能**绕过骨架手搓**(骨架要薄到不成为约束)。
3. 每个 workspace 落 `schema_version` + `round-N` 快照，翻旧账可重放。
4. 「生成前先 review 提示词+参考图+费用」这条纪律（AGENTS.md 硬性规则）从**靠 agent 自觉**变成**脚本强制闸门**，反而固化已有规则而非新增。

---

## 7. 落地路径（建议）

不在本文给出 deadline，只给依赖顺序：

1. **定义公共信封 schema v1** + `validate.py`(最薄、基石，先于一切)。
2. **做 type A(视频模型逐镜)适配器** —— 复用 `short-drama` 的 `tasks.json`/`produce_episode.py` 作为生成脚本后端。
3. **写媒体联系单自写模板** + 小服务写回(岔路一、二落地)。
4. **`review.py --wait` 轮询** 接上阻塞语义(岔路三)。
5. 打通一个完整 round：agent 出 envelope → 渲染 → 用户标注 → script 生成 → 单镜重跑 → 拼接 → 归档。
6. 用一次真实短片(如小猫医生重制)验收闭环。
7. 再按需加 type B/C/D 适配器。

## 8. 参考链接

- Beacon(视觉规划 + 阻塞 approve/feedback)：https://www.trybeacon.sh/docs
- MeowKit visual-plan(JSON+validate+gate+本地 studio)：https://docs.meowkit.dev/cli/commands
- OpenMontage Backlot(只读看板参照，"intentionally read-only for v1")：https://github.com/calesthio/OpenMontage/discussions/283
- Pixelle-Video：https://github.com/ATH-MaaS/Pixelle-Video
- LTX Studio(逐镜审查的闭源参照)：https://dupple.com/tools/ltx-studio
- 本地 `answer-me-with-html` 用法与坑：`research/answer-me-with-html-用法与踩坑记录.md`

---

*本文为方向与设计定稿，非成熟实现。优先不引入新依赖/新框架；能用手写脚本 + 复用 short-drama 现有后端起步。*
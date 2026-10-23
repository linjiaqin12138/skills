# 家教类 Skill 调研：learn-coach、Bloom、agent-tutor

> 调研日期：2026-10-23。数字取自当天 GitHub API。
> 需求：用户说出想学的主题，agent 就当家教，一次只推进一步，等用户回应再继续。起因是想按 [MuJoCo 官方 Python tutorial](https://github.com/google-deepmind/mujoco/blob/main/python/tutorial.ipynb) 学。本仓库 `skills/` 与 `.claude/skills/` 里没有这类通用家教。

## 结论

三个都能接「我想学 X」。差别在学习发生的地方，以及过关靠什么。

| | 学习发生在 | 过关靠什么 | 2026-10-23 热度 |
|---|---|---|---|
| [learn-coach](https://github.com/AgentGameLab/learn-coach) | 对话。每轮 ≤300 字 + 1 张图 + 1 个问题，然后停下 | 你用自己的话回讲；缺口由它列出来，再让你重讲 | 0 star，最后推送 2026-08-09 |
| [Bloom](https://github.com/Li-Evan/Bloom) 的 `bloom-tutor` | 课题文件夹里的课文。对话只确认状态 | 你在文里标 `???`、写文末反馈；它读完再写下一篇 | 292 star / 43 fork，最后推送 2026-10-20 |
| [agent-tutor](https://github.com/Mohamed-El-Sharqawy/agent-tutor) | 你自己的笔记库。聊天只做取向和批改摘要 | 笔记里的自测 + 5–8 题测验，70% 才进入下一题 | 4 star，最后推送 2026-08-31 |

三者都是 MIT，都是纯 Markdown skill，不自带运行时。三者都**不会**把任意主题做成「现在运行这条命令，把输出贴回来」的终端实验课。learn-coach 在代码课里会把有设计取舍的几行留给你写；agent-tutor 的安全边界写明课程里不生成可执行脚本。

就 MuJoCo notebook：想在这个聊天里被追问、穿插自己跑 Python，用 learn-coach。想留一套可回头读的中文课文，用 Bloom。想把 notebook 当指定教材、按章节出笔记并测验，用 agent-tutor 的 `learn from <URL>`。

## 筛掉了什么

同一次检索里还有四套，形态接近，但不是这次要写的三个：

- [mattpocock/skills 的 teach](https://github.com/mattpocock/skills/blob/main/skills/productivity/teach/SKILL.md)：当前目录变成学习工作区，一课一个短 HTML。英文 skill。后面有人在此基础上做了 [matheuszambonin/teach-me](https://github.com/matheuszambonin/teach-me) 和 [edheltzel/do-skills 的 do-teach-me](https://github.com/edheltzel/do-skills)。
- [mznmel/learnOpt-skill](https://github.com/mznmel/learnOpt-skill)：一步一个 HTML，页面里写讨论和答案，说「准备好了」才测验。文件名是 `skill.md`，按能力图做爬山式选下一步。
- [sanyuan0704/sanyuan-skills 的 sigma](https://github.com/sanyuan0704/sanyuan-skills/blob/main/skills/sigma/SKILL.md)：任意主题，每轮 1–2 问，约 80% 掌握才前进，规则写明不直接给答案。
- [brianlan/improved-ai-agent 的 turn-by-turn-tutor](https://github.com/brianlan/improved-ai-agent)：从目录或题目出发，一轮一个小单元，问完理解检查再等你。

另外，针对某一个产品的交互教程（例如 [GitHub Copilot CLI quickstart](https://github.com/github/awesome-copilot/blob/main/skills/copilot-cli-quickstart/SKILL.md)）会让用户按课运行命令，但不能换成 MuJoCo。

## learn-coach

仓库：[AgentGameLab/learn-coach](https://github.com/AgentGameLab/learn-coach)。组织账号 AgentGameLab，2026-07-01 创建，2026-08-09 最后推送，0 star、0 fork，MIT，仓库约 13 KB。README 写明后续版本改发到 [AgentGameLab/skills](https://github.com/AgentGameLab/skills)，本仓保留可独立使用。安装方式是克隆进 `~/.claude/skills/learn-coach`。topics 里同时标了 Claude Code、Codex、Cursor、Cline。

文件只有三块：`SKILL.md`（分流 + 七步流水线）、`references/teaching-playbook.md`（好/坏对话范例）、`evals/evals.json`。

触发面很宽：用户说「教我 / 我想学 / 搞懂 / 入门 / 带我过一遍」，甚至没说「教」但意图是学会，就进入教学。查 API、赶工救火、以及用户说「直接告诉我」，直接给答案。逃生门优先级最高：教学中途说「别绕了」就立刻讲清。作者把这条写成对 Khanmigo「永远不给答案」的纠正。

流水线是跨多轮的，一轮只走一步：

1. **摸底。** 能从原话推断水平就跳过。否则最多问 2–3 个经历问题（「你之前用它做过什么」），不问「你懂不懂 X」。
2. **知识地图。** 一张图回答领域长什么样、今天学哪一块、和已知怎么接。配不超过 3 行路线，停下等确认。
3. **先猜后讲。** 每个核心概念讲解前，出一个只靠已有知识就能试的问题，要求把猜测打出来。猜之前不给方向性提示。
4. **最小完整模型。** 3–5 句加一张图，先给一个能用的 80% 模型，并回扣刚才的猜测：哪部分对、从哪一步偏了。然后问是继续还是停在这里。
5. **苏格拉底追问。** 一次只问一个。升级链写死：反问 → 缩小范围的提示 → 第二次仍卡住就直接讲，并说明为什么这里难。答对之后追问边界情况。连续三次不努力要提示，就改问卡在提示的哪一部分；明确说「直接告诉我」仍走逃生门。
6. **费曼回讲。** 核心层讲完，让用户把你当成完全不懂的人讲一遍。回讲时不插话。讲完按事实、概念、逻辑链、有没有自己的例子列缺口，重讲权交回用户。最多两轮，剩下的由 tutor 补。
7. **撤架。** 同类任务连续两次无提示完成，支持降一档（完整示范 → 半成品补关键部分 → 独立做、只 review）。连续卡壳则升一档。主题结束时提议把已掌握点和暴露过的误区写入 `user_learning_profile.md`。

代码课的分工写在第 7 步：样板、配置、CRUD 由 tutor 写；有设计取舍的 5–10 行留给用户写。

语言规则：中文为主，术语第一次出现写成「术语（白话解释）」，类比必须指出失效点。单轮上限 300 字 + 1 张图 + 1 个问题。禁用「懂了吗」，改成复述、预测或改造。错答先肯定对的成分，再指出第一个出错的步骤。

图的优先级：有 visualize 类工具就用 SVG/HTML；纯终端用 ASCII；Mermaid 只写进要落盘的 Markdown。playbook 用 Raft 选举做了整段示范，坏例子是开场就甩 1500 字，或猜之前先泄露「和随机定时器有关」。

状态只在对话里。复杂主题建议在地图之后列 checklist。没有课题文件夹、没有测验分数、没有间隔复习。换一个会话，除非写过 `user_learning_profile.md`，否则进度不在。

## Bloom

仓库：[Li-Evan/Bloom](https://github.com/Li-Evan/Bloom)。作者 Li-Evan，2026-03-10 创建，2026-10-20 最后推送，2026-10-22 仍有元数据更新。292 star、43 fork、1 个 open issue，MIT，主语言 JavaScript，体积约 5 MB。主页 [bloomtutor.app](https://bloomtutor.app)。这是三家里唯一有实际采用量的。

定位引自 Benjamin Bloom 1984 年的 2 Sigma：一对一辅导比常规课堂高约两个标准差。仓库同时给两条入口：Claude Code 里的 skill，以及可自托管的网页（React + FastAPI）。两条走同一条链：大纲 → 一篇课文 → 批注 → 反馈 → 下一篇 → 评估篇 → 总结。本文只看 skill。

`skills/` 下有七个：

| Skill | 作用 |
|---|---|
| `bloom-tutor` | 完整家教：大纲、逐篇课文、`???` 批注、评估、总结 |
| `learn-deep` | 用下面五个镜头过一遍，再帮你选方向 |
| `learn-crossover` | 用已经会的结构类比新概念 |
| `learn-occam` | 判断值不值得学、学多深 |
| `learn-graph` | 画领域知识图和学习路径 |
| `learn-prototype` | 先做出能跑的最小原型，再让用户自己指出缺陷并改 |
| `learn-feynman` | 用回讲验收是否真懂 |

CLI 安装是把 `skills/bloom-tutor` 拷进 `.claude/skills/`，或在 Claude Code 里 `/plugin marketplace add Li-Evan/Bloom` 后安装 `bloom@li-evan`。触发语包括「开个文件夹学 X」「我想学 X」「继续」「我读完了」。全部回复、文档使用中文。

`bloom-tutor` 的守则：

1. 每次只生成一篇文档。输出后必须等用户读完并反馈。即使用户要求，也不批量生成 `01.md`、`02.md`、`03.md`。
2. 开新课题时，同一轮内写出 `syllabus.md` 和首篇 `01.md`。开场不做苏格拉底诊断；理解情况等用户在 `01.md` 的反馈区给出。
3. 大纲深度分简单 / 标准 / 深入，未指定则标准。
4. 用户不能主动要 `summary.md`。总结只在掌握项学完、评估篇读完之后自动出现。
5. 写任何新文档之前，先读该课题全部已有 Markdown、文末反馈，以及全文里的 `???` / `？？？`。
6. 每次对话先读仓库根目录的 `learning-log.jsonl`。
7. 衔接阶段的苏格拉底提问最多 2 轮，每轮 1–2 个问题，到点必须出下一篇。

用户说「我读完了」之后的决策是一条连贯判断：先读批注和反馈；有严重误解就先问清楚（仍 ≤2 轮）；把本篇覆盖的掌握项勾上，并在进度表加一行；若刚读完的是评估篇，生成 `summary.md` 并结束；若大纲已全部勾完，生成只复盘、不加新内容的评估篇；否则生成下一篇正文。太浅的跳过，看不懂的换角度重讲。`???` 的优先级高于文末反馈。

课题目录：

```text
<课题名>/
├── syllabus.md
├── 01.md, 02.md, ...
├── <评估篇>.md          # 首行标记为评估篇
├── pre-summary.md       # 中间产物，学完删除，不向用户展示
└── summary.md
learning-log.jsonl       # 仓库根目录，只追加
```

未指定位置时，课题文件夹建在工作根目录。

`learn-prototype` 是动手的那一支，触发语是「要做 X / 研究 X / 做得不好想改」。流程是：先做出能跑的最小版本；问用户「哪里不好」并且先让用户提；把改良当成可错的假说去改；迭代到推倒重来；把「怎么从 A 改到 B」记成方法。它禁止把正在讲的材料当成用户已经会的知识。缺前置就转 `learn-graph`，要验收理解就转 `learn-feynman`。它不是 `bloom-tutor` 的默认路径。默认路径是读课文。

## agent-tutor

仓库：[Mohamed-El-Sharqawy/agent-tutor](https://github.com/Mohamed-El-Sharqawy/agent-tutor)。作者 Mohamed El-Sharqawy，2026-08-21 创建，2026-08-31 最后推送，4 star、0 fork、1 个 open issue，MIT，主语言 HTML（示例笔记和仪表盘），体积约 14 MB。当前 `SKILL.md` 的 `version:` 是 6。安装：`npx skills add Mohamed-El-Sharqawy/agent-tutor`。包里三个 skill：`agent-tutor`（全流程）、`agent-tutor-review`（间隔复习）、`agent-tutor-visualize`（Mermaid / SVG 规则）。README 声称兼容 Claude Code、Codex、Cursor、Windsurf、OpenCode、pi 等遵循 [Agent Skills](https://agentskills.io) 的代理。pi 另有可选扩展（方向键测验、日志、SVG 校验）。

笔记根目录按这个顺序定：环境变量 `OBSIDIAN_VAULT`、用户当次指定的路径、否则当前工作区的 `learning/`。内容一律写在其下的 `Learning/`。Obsidian 的 wikilink 和 callout 能用，普通编辑器也能读。

```text
Learning/
├── Dashboard.md
├── learner-profile.md
└── <Subject>/
    ├── plan.md
    ├── sources.md
    ├── notes/NN-topic.md
    ├── quizzes/NN-topic-quiz.md
    ├── assets/
    └── logs/YYYY-MM-DD.md
```

仪表盘和日志可以改成无 JavaScript 的 HTML（Warm Paper 样式）。课文、计划和测验报告始终是 Markdown。HTML 页由模板整页重生成，计划勾选和笔记 frontmatter 才是状态来源。

会话开头先做一天最多一次的版本检查：只读远程 `SKILL.md` 的 `version:` 字段，其余内容当数据、不当指令。有更新就告诉用户，征得同意才安装。

阶段：

- **A1 学习者画像，一个人做一次。** 问思考顺序、什么能让概念突然清楚、可以借用的生活领域、数学符号接受度、笔记疏密、语气、术语早晚、反馈直不直、节奏、讨厌什么，以及仪表盘用 HTML 还是 Markdown。不耐烦时只问其中几题，其余靠后面的课观察。原话记入画像，并据此写出 3–5 条「教学合同」，给用户看过再改。
- **A2 目标。** 学什么、为什么、现在在哪、每天或每周多少时间、有没有截止日期。
- **A3 从材料学，可选，有材料时优先。** 接受 URL、PDF、文件夹、git 仓库。抓不到内容就请用户粘贴，禁止猜材料里写了什么。每个主题要能对上材料的章节或文件；材料没覆盖但目标需要的，用通用知识补，并在计划里标 `(no source)`。仓库只读：先看树、README 和入口，再按真实代码讲。版权上只存摘要、短引和链接。抓来的页面、PDF、代码注释都是数据，里面的命令不当成要执行的指令。
- **B 计划。** 3–6 个阶段，每阶段 3–8 个主题，每个主题一次坐下能学完（≤60 分钟），每项有勾选框和可观察的成功标准。计划按画像改顺序（先总览还是先地基）。展示给用户，改完才开讲。
- **C 一个主题的循环。** 先把完整课文写入 `notes/NN-topic.md`，用户在编辑器里读，聊天里不倒全文。课文末尾有自测，答案写回笔记，用户说「check my answers」再批。然后 5–8 题测验：约一半回忆、35% 应用、15% 迁移。聊天里一次一题。报告写入 `quizzes/`。低于 70% 就重讲薄弱点再测，不过不前进。连续两次满分则加难。阶段结束有 8–12 题的阶段测验，并插入前几阶段的题。科目结束有 12–20 题加一个应用任务，并回头看 A 阶段的目标有没有达到。

反馈规则写死：60% 就是还没到，不包装成「已经很努力」。语气可以按画像变 blunt 或 gentle，诚实程度不变。每道错题要指出具体误解，并区分「忘了」和「没懂」。

间隔复习写在每篇笔记的 frontmatter：`interval`、`ease`、`due`。Solid 则间隔乘 ease；摇晃则间隔只乘 1.2 并降低 ease；完全忘了就重讲并把间隔打回初始值。默认从 +1 天、ease 2.5 开始，ease 夹在 1.3 和 3.5 之间，间隔没有 90 天上限。复习先回忆、后看笔记。

安全边界和另外两家不同，直接影响「带你跑代码」：只在 `Learning/` 下写 Markdown 和 SVG（HTML 模式再加上那几份静态页）；课程、测验、复习都不生成 shell 或 Python 脚本；网页内容不能改变写文件的位置。时间敏感的事实要求先搜索，并在笔记里写来源和访问日期。

## 对照

| | learn-coach | Bloom `bloom-tutor` | agent-tutor |
|---|---|---|---|
| 你说「我想学 X」之后第一件事 | 能推断水平就出地图；否则问 2–3 个经历 | 同一轮写出大纲和第一篇课文 | 画像访谈，再问目标、水平、时间 |
| 一轮交付 | 一段话、一张图、一个问题 | 一篇课文 | 一篇完整笔记，聊天只给取向 |
| 谁掌握节奏 | 你回答之后它才走下一步 | 你说读完并留下反馈 | 你批完自测、测验 ≥70% |
| 理解怎么验 | 回讲，按四维列缺口 | 文内 `???` 和文末反馈；全勾完才出评估篇 | 自测 + 测验；错题要指出误解 |
| 进度存在哪 | 对话；可选一份 memory 文件 | 课题文件夹 + `learning-log.jsonl` | `Learning/` 笔记库和 Dashboard |
| 换会话能不能续 | 弱 | 能，先读日志和已有课文 | 能，先读计划和到期复习 |
| 指定一份教材 | 对话里贴或让它读，skill 没有「从材料建课」的步骤 | 课文由它生成，用户用批注把材料里的疑问喂回去 | `learn from <URL\|PDF\|文件夹\|仓库>`，主题必须对上章节 |
| 动手写代码 / 跑命令 | 样板它写，5–10 行取舍留给你 | 默认是读。动手走另一个 skill `learn-prototype` | 明文禁止在课里生成可执行脚本 |
| 间隔复习 | 无 | 无。靠评估篇和总结收束 | 有，按每篇笔记的回忆结果改间隔 |
| 语言 | 中文为主 | 强制中文 | 跟随用户；文档和示例是英文 |
| 维护状态 | 本仓停在 2026-08，新版去组织单仓 | 2026-10 仍在推送 | 2026-08-31 后未见推送；skill 自带每日版本检查 |

## 对 MuJoCo 官方教程意味着什么

教材是 [google-deepmind/mujoco 的 `python/tutorial.ipynb`](https://github.com/google-deepmind/mujoco/blob/main/python/tutorial.ipynb)：一份要在 Python 里加载模型、步进仿真、读状态的笔记本。三个 skill 都能拿它当主题，但默认产物不一样。

- **learn-coach** 会把它拆成对话里的一块一块：先给仿真循环的地图，再让你猜 `mj_step` 前后状态会怎样，再给一个能用的最小模型，最后让你用自己的话讲回去。你要自己跑的单元得在对话里另外说；skill 没有「每步一条命令」的清单。适合想在聊天里被追着问的人。
- **Bloom** 会开一个课题文件夹，大纲加第一篇中文课文，你在文里标不懂的句子。它不会替你把 notebook 一格一格跑完。若目标是写出一个能步进的最小脚本，应显式改走 `learn-prototype`，而不是指望 `bloom-tutor` 的默认课文链。
- **agent-tutor** 最适合「教材已经指定」。`learn from` 那个 notebook URL，计划按笔记本的章节排，每课引用对应小节，抄整本进笔记库是它自己的版权规则所禁止的。它也不会把教程单元格变成要你执行的脚本。适合想留下可复习笔记、并用 70% 门槛卡住跳章的人。

若目标是「官方教程的每一格都由用户在终端跑过、把输出交回来」，这三个都要在使用时额外约定。skill 正文里没有这条回路。

## 调研边界

读过的是三个仓库的 README、主 `SKILL.md`，以及 learn-coach 的 `teaching-playbook.md`、Bloom 的 `learn-prototype/SKILL.md`。没有安装，没有跑一轮真实教学，所以触发是否稳定、一轮会不会仍然讲太多，本文不判断。star 数不能代表教学质量：learn-coach 和 agent-tutor 的说明比 star 数完整，Bloom 的采用量最高，也附带一个本文没有拆的网页应用。

## 来源

- learn-coach README、SKILL.md、teaching-playbook.md：https://github.com/AgentGameLab/learn-coach （读取于 2026-10-23）
- 后续版本所在单仓：https://github.com/AgentGameLab/skills
- Bloom README 与 `skills/bloom-tutor/SKILL.md`、`skills/learn-prototype/SKILL.md`：https://github.com/Li-Evan/Bloom
- agent-tutor README 与 `skills/agent-tutor/SKILL.md`（version 6）：https://github.com/Mohamed-El-Sharqawy/agent-tutor
- 仓库 star、fork、创建日、最后推送、协议：上述三个仓库的 GitHub API，2026-10-23
- MuJoCo 教程：https://github.com/google-deepmind/mujoco/blob/main/python/tutorial.ipynb

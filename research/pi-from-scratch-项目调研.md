# pi-from-scratch 项目调研

> 调研日期：2026-10-04。调研动机：评估 clone-from-scratch-tutor skill 产品化的对标形态——pi-from-scratch 是目前形态上最接近的手工制作样板（一边读文章、一边代码徐徐展开的交互式复刻教程站）。

## 一句话概况

[pi-from-scratch](https://github.com/SaladDay/pi-from-scratch)：把生产级 coding agent **pi**（[earendil-works/pi](https://github.com/earendil-works/pi)，11.2 万 stars）手工重写成约 600 行 TypeScript（nano-pi），并配了一个交互式教程网站（[pi-from-scratch.vercel.app](https://pi-from-scratch.vercel.app)）。全中文，MIT 协议，零运行时依赖。

- 创建：2026-08-09，主要开发在 9 天内完成
- Stars：上线第 9 天 1057 → 2026-10-04 共 1263（爆发集中在头两周，日均 100+）
- 没上过 Hacker News，传播主阵地在中文圈（X 中文 KOL、中文 AI 聚合站收录）

## 作者背景

[SaladDay](https://github.com/SaladDay)，浙江大学（ZJU），Rust 工具出身。他是 **CC Switch**（ccswitch.io，Claude Code/Codex 多 agent 配置切换桌面工具，3 万+ stars）的作者，在 AI coding agent 工具生态里本就有积累。pi-from-scratch 是他 pinned 第 4 位的项目。项目接受了 OpenModel、Cubence 两家 API 中转商的赞助，正文里嵌了软广——中文独立开发者项目的一种变现样本。

## 教程内容

两章 + 一个 trace 实验场，全中文，口语化、初学者友好，自称"保留古法手敲，尽可能没有 ai 味"。开篇引费曼："What I cannot create, I do not understand."

**第一章「先导」**（约 8 分钟）——正是"先导介绍 + 模块关系图 + 技术栈说明"：
- 食用方式说明
- 五个文件各自的职责、对外接口、对应 pi 里的哪个包
- 数据流与依赖关系总结，配 6 张手绘风插图（Excalidraw 质感：圆角手绘边框、米黄纸质底纹、彩色模块块、实线 = runtime import / 虚线 = import type）

**第二章「创造你的 nano-pi」**（约 24 分钟）：沿数据流逐段写代码——agent 是什么 → 怎么跟 LLM 说话（SSE 解析、消息格式）→ 四个工具逐个实现 → 补完 agent loop → 三个边界情况（max_tokens 截断、abort、请求失败）→ context compaction → TUI → 用 20 行 HTTP/SSE server 实验证明 UI 解耦 → CLI 粘合 + session 持久化。章内嵌"语法速查"折叠块扫盲 TS 语法点。

**第三章「trace 跟踪」**：预生成的真实 trace 数据（用 glm-5.2 实跑并带正确性校验），6 个案例覆盖一轮结束、单次工具往返、多工具链、工具失败、max_tokens、Ctrl+C 中断。

## 网站交互与技术实现（最值得抄的部分）

技术栈：**Next.js 16 + React 19 + Tailwind 4 + marked + highlight.js**。**没有用 Monaco/CodeMirror，也没有用任何动画库**——编辑器是 React + CSS 手搓的，动画是 CSS 变量 + Web Animations API。

### 阅读联动机制（核心交互）

- 单页应用，URL hash 路由切章节。桌面端左栏文章 + 右栏钉住的代码面板；移动端代码面板变底部抽屉；顶部阅读进度条
- Markdown 里埋 `<!-- checkpoint: xxx -->` 锚点，滚动越过视口 42% 高度的"阅读线"时激活对应 checkpoint
- **checkpoint = 一份仓库快照**：从最终代码里截取行区间组成该阶段文件内容。构建期有两个 assert 校验：①快照单调递增（只能加行、不能改旧行）②锚点与 checkpoint 一一对应。第一章 7 个 checkpoint，第二章 23 个
- 新增行用 LCS diff 算出，逐行"打字"式浮现（每行延迟 28ms 递增），自动滚动到第一个变更行——这就是"代码徐徐展开"的实现原理
- 代码面板仿 IDE：文件树（新文件高亮、写完打 ✓）、tab 页、行号、语法高亮；右上角有锁，锁定后停止跟随阅读

### Trace 视图

三栏调试器：左 case 列表 / 中只读源码（当前执行行高亮）/ 右 inspector（Core State 变量表、可点击的 Call Stack、当前 Event JSON、完整 Context）。点行号下断点，F10 单步、F5 继续——完全复刻 IDE 调试器的肌肉记忆。全部静态数据，浏览不发起模型请求。

### 内容管线

`prebuild` 脚本把根目录 `docs/*.md`（文章）和 `src/*.ts`（源码）打包成 `content.generated.ts`，文章与代码同源同步——这是它能保证"文章和代码不漂移"的关键工程手段。

## nano-pi 模块划分（约 600 行）

| 文件 | 职责 | 对应 pi 的包 |
|---|---|---|
| `llm.ts` | OpenAI 兼容请求、SSE 流解析、Context 类型 | pi-ai（pi 适配十几个 provider，这里只做一种） |
| `agent.ts` | agent loop 核心：tool_call 循环、边界处理、compaction | pi-agent-core |
| `tools.ts` | 四个纯函数工具：read_file / write_file / edit / run_bash | agent-core 同四件套 |
| `tui.ts` | 88 行终端 UI，只认识 AgentEvent | pi-tui |
| `cli.ts` | 胶水层 + session.jsonl 持久化 | pi-coding-agent |

架构要点：单向依赖（cli 是唯一认识所有模块的）；"前后端分离"——AgentEvent 是 agent 与 UI 之间的协议。理念："删除 pi 的工程细节，留下 pi 的核心思想。"

## 对 clone-from-scratch-tutor 产品化的启示

**值得借鉴的：**
1. "文章先行、代码是副产品"的定位——README 自述"这是一篇文章，不是一本书"，降低了读者心理门槛
2. checkpoint 快照 + LCS diff 的"代码徐徐展开"，且有构建期 assert 保证单调递增——教学工程化的样板
3. trace 调试视图是差异化亮点，把"理解动态执行流"从想象中变成可操作的
4. 极简技术选型：不引入编辑器/动画库，手搓反而保证了动画与阅读节奏的精确配合

**它的天花板（= 我们的机会）：**
1. 纯手工制作，9 天一个项目，不可复制——作者自己在"催更"组件里预告了五六篇后续（token 估算、extensions/skills、多 provider 等），一个半月过去一篇没更
2. 内容只覆盖"快乐路径 + 少量边界"，没有里程碑/验收标准/偏差登记这些"工程化复刻"的结构（对比 CodeCrafters 的测试驱动）
3. 无决策卡片——讲了"怎么做"，较少讲"为什么这么做、放弃了什么"
4. 传播局限在中文圈，没上 HN，英文市场是空白

**结论**：pi-from-scratch 验证了"交互式复刻教程站"形态的需求（两周千星、日均百星），但它本质是一篇精心手工的文章，不是可规模化的产品。"输入任意仓库 → 自动生成这种形态"仍然没人做。

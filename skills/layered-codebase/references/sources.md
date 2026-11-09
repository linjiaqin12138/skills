# 规范出处与延伸阅读

本 skill 的分层/测试部分主要整理自 **OpenQuantMaster** 项目的 `docs/conventions/architecture.md` 与 `docs/conventions/testing.md`（作者自建项目），并做了语言无关化改写。其余部分参考下列公开规范。

## 直接来源

| 来源 | 贡献了什么 | 位置 |
|---|---|---|
| OpenQuantMaster `docs/conventions/architecture.md` | Protocol/Service/UseCase/Provider/Repository 分层、模型与配置归属、Result/ServiceResult 错误模型、事务与 session 规则、Port 的引入条件 | 项目内 `docs/conventions/architecture.md` |
| OpenQuantMaster `docs/conventions/testing.md` | 测试归属表、patch 策略、避免重复覆盖、组合根构建被测对象 | 项目内 `docs/conventions/testing.md` |
| OpenQuantMaster `docs/conventions/documentation.md` | 文档语言与文档结构约定 | 项目内 `docs/conventions/documentation.md` |

## 延伸阅读（已核对原文）

| 规范 | 链接 | 我们借鉴的点 |
|---|---|---|
| Anthropic — Best practices for Claude Code | https://docs.anthropic.com/en/docs/claude-code/best-practices | **确定性 vs 非确定性**：能上 hook/deny rule 的规则就别只写在文档里；AGENTS.md 只写 agent 猜不到的东西；给 agent 一个能自己跑的 check |
| Anthropic — Building effective agents | https://www.anthropic.com/engineering/building-effective-agents | 从最简方案开始，别一上来就上框架；工具/接口要像给新人写 docstring |
| AGENTS.md 开放格式 | https://agents.md/ | agent 指令的统一入口；嵌套目录就近覆盖；"任何人都该告诉新同事的事"都写进去 |
| Twelve-Factor App | https://12factor.net/ | 配置在环境、进程无状态、日志当事件流、依赖显式声明、构建/运行分离 |
| Conventional Commits | https://conventionalcommits.org/en/v1.0.0/ | 提交信息结构；type 与 SemVer 的对应；scope 用于圈定 review 范围 |
| ADR | https://adr.github.io/ | 决策记录的形式与用途（Context / Decision / Consequences） |
| Diátaxis | https://diataxis.fr/ | 文档四象限：tutorial / how-to / reference / explanation |
| mypy strict | https://mypy.readthedocs.io/en/stable/command_line.html | 用类型检查器做确定性约束，而不是靠约定 |
| pytest fixtures | https://docs.pytest.org/en/stable/how-to/fixtures.html | fixture 依赖与分层组织 |
| OpenVLA 仓库布局 | https://github.com/openvla/openvla | 包 / 脚本 / 实验三分离；顶层 Makefile 承载 lint |

## 延伸阅读（知识储备，未在本次抓取核对）

| 规范 | 链接 | 适用场景 |
|---|---|---|
| Feature-Sliced Design | https://feature-sliced.com/ | 前端按功能分层的替代方案 |
| Atomic Design | https://www.bradfrost.com/blog/atomic-web-design/ | 设计系统与组件库 |
| ROS 2 | https://docs.ros.org/ | 机器人软件栈的 package / 接口契约约定 |
| LeRobot | https://huggingface.co/docs/lerobot/ | 具身智能的数据 / 策略 / 评测目录约定 |
| Cookiecutter Data Science | https://cookiecutter-data-science.drivendata.org/ | 数据科学项目结构（raw 只读、产物分离） |
| Nx / Turborepo | https://nx.dev/ · https://turborepo.com/ | monorepo 的包边界与受影响范围计算 |
| Joel Spolsky — Things You Should Never Do (Part I) | https://www.joelonsoftware.com/2000/04/06/things-you-should-never-do-part-i/ | 不要重写，增量改造；"读代码比写代码难" |
| DDD（Eric Evans） | https://domainlanguage.com/ | bounded context、聚合、领域模型边界 |

## 使用建议

本 skill 故意**没有**把上面所有东西都塞进来：

- 分层与错误模型是**硬约束**，来自 OpenQuantMaster。
- 复杂度阶梯是本 skill 相对原文档的**主要增强**：原文档反复强调"不要为凑图引入抽象"，但没有给出可操作的升/降级判据，这里补上了触发条件与降级条件。
- 前端/机器人/ML 的具体目录规范**没有**写进来。本 skill 是语言无关的后端/CLI/服务形态；等真正要在那类项目里落地时，再基于 Feature-Sliced / ROS 2 / LeRobot 另写 profile，不要把它们和后端规则混在一份文档里。

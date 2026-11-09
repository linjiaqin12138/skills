---
name: layered-codebase
description: 用一套语言无关的分层架构（Protocol → Service → UseCase → Provider/Repository）约束代码组织、错误模型、测试归属和 agent 写码边界，让代码结构一致、易 review、可增量演进。当用户要新建项目/新模块、给项目定架构规范、写新功能、review 代码、或抱怨 agent 写码太自由/脚本太散/扩展困难时使用。
argument-hint: [目标：新建项目 | 接入分层 | 写功能 | review 代码 | 审查架构违规]
---

# 分层代码库（Layered Codebase）

一套**语言无关**的工程约束。核心目标有三个：

1. 任何人（或任何 agent）打开仓库，都能在 30 秒内判断"这段代码该放哪、该由谁测、错了该谁负责"。
2. review 时看的是**结构契约**，不是逐行挑错。
3. **架构复杂度按需增长**——先少后多，绝不预先搭满整套分层。

## 核心信条

1. **依赖只向内**：`Protocol → Service → UseCase → Provider / Repository`。反向依赖、跨层 import、同层横向 import 都是违规。
2. **错误码是契约，不是异常类型**。每层拥有自己的错误码命名空间，逐层显式映射；`unknown` 只能做兜底，不能做分支条件。
3. **每个行为只有一个测试归属**：Provider 证传输、Service 证业务、Protocol 证协议映射。同一个规则不许测两遍。
4. **复杂度按需引入**。每种抽象（UseCase / Port / UnitOfWork / Provider / Repository）都有明确的"触发条件"和"不满足就不许加"的红线。
5. **能工具化的约束就不写在文档里**。能进 linter / type checker / test / CI 的规则，做成确定性检查；只有需要判断力的才写进文档。

## 三种使用方式

### A. 新项目 / 新模块：先定层级，再写代码

1. 读 `references/complexity-ladder.md`，判断这个项目现在**该停在第几级**（大概率是 1 或 2 级，不是满级）。
2. 按 `templates/repo-layout.md` 建目录，只建当前层级需要的目录。
3. 把 `templates/AGENTS.md` 复制到仓库根目录并按实际命令改写（保留"装依赖 / 跑测试 / 跑检查 / 提交规范"四段）。
4. 按 `templates/tooling.md` 接上确定性检查（lint / type / test），并把命令写进 AGENTS.md。
5. 声明清楚"未来的升级触发条件"，写进 README 或 ADR，避免后来人误以为分层是随意的。

### B. 写一个新功能：按层落位

1. 判断这个功能属于哪一层（多数功能横跨 Protocol → Service → UseCase → Provider/Repository）。
2. 查 `references/architecture.md` 里该层的职责与禁止事项，确认新代码没有越界。
3. 用 `references/testing.md` 的归属表确定测试放哪、patch 谁、断言什么。
4. 提交前跑 `references/agent-rules.md` 的 Definition of Done。

### C. Review 代码：按清单逐条过

用 `references/review-checklist.md`。它分**阻塞项**（违反结构契约，必须改）和**建议项**（风格与可维护性，可商量）。review 时只报阻塞项 + 建议项，不做无关的重写建议。

## 分层速查

| 层 | 一句话职责 | 明确禁止 |
|---|---|---|
| Protocol | 解析入参、调用 Service、渲染出参、映射错误码到外部协议 | 写业务逻辑、直连 UseCase/Provider/Repository |
| Service | 对外门面，按语义命名方法，组装 UseCase 与依赖 | 通用 `run()`、工作流、事务、校验 UseCase 模型 |
| UseCase | 一个完整应用动作：业务规则、编排、错误映射、重试策略、事务边界 | 一个 UseCase 塞多个动作（`action` 分派）、读全局配置 |
| Provider | 无状态第三方 API/HTTP/SDK 调用，藏传输细节 | 读配置、选账号、管连接生命周期、依赖应用模型 |
| Repository | 数据库读写，接收 session，**不 commit** | 自己开事务、返回 Result、SQL 式命名 |
| utils | 通用无副作用小函数 | import 任何应用层、读配置/网络/DB |
| models | 跨层共享的稳定领域结构 + Result/Error | 变成"所有 Pydantic 类"的回收站 |

细则（模型归属、配置归属、事务、错误映射、Port 触发条件）见 `references/architecture.md`。

## 复杂度阶梯（重点）

**不要一上来就搭完整分层。** 每升一级都要有具体理由：

| 级别 | 新增 | 进入条件（不满足就不许加） |
|---|---|---|
| 0 | 单文件脚本 | 一次性、不可复用、无外部依赖；且必须放进 `scripts/` 或做成 CLI 子命令，不许漂在根目录 |
| 1 | + Service | 出现第二处调用同一段逻辑，或逻辑超过一个函数能说清的范围 |
| 2 | + UseCase | Service 里出现第二条**独立**工作流，或需要独立的事务/重试/错误码策略 |
| 3 | + Provider | 出现第二个外部 API/SDK，或测试需要稳定的 patch 边界 |
| 4 | + Repository | 出现真实持久化需求 |
| 5 | + Port/Adapter | 同一能力出现第二种实现，或确有替换需求 |
| 6 | + UnitOfWork | 重复的事务生命周期代码已经出现，且 UnitOfWork 能真的消除它 |

降级也是允许的：如果某个 UseCase 只有一个调用方、没有独立策略，把它收回 Service 里；不要为了"符合分层图"留着空壳层。

细则见 `references/complexity-ladder.md`。

## 测试速查

标准形态：

```
用组合根构建真实 Service
  → 调用它的语义化方法
  → 只 patch 具体 Provider（外部调用边界）
  → 断言公开的 ServiceResult
```

归属：`tests/services/` 证业务，`tests/providers/` 证传输与解析，`tests/protocols/` 证协议映射与退出码，`tests/utils/` 证小工具，Repository 集成测试用真库。
禁止：mock Repository/session；为每个代码层都建一套平行测试；孤立测模型只为覆盖率。

细则见 `references/testing.md`。

## Agent 写码红线（摘要）

- 新增顶层目录、引入新依赖、改变公开接口、动 `AGENTS.md` 里的命令——**先问再做**。
- 一次改动只做一件事；不顺手重构、不顺手格式化、不顺手升级依赖。
- 不许把业务逻辑写进 Protocol，不许在 Provider 里读配置，不许在 Repository 里 commit。
- 不许用模块级可变全局（全局 client、全局缓存）来"省事"。
- 改完必须给出**可运行的验证证据**（命令 + 输出），不能只说"应该没问题"。

完整红线与理由见 `references/agent-rules.md`。

## Definition of Done

改完代码，逐条确认（有检查命令的必须真的跑过）：

- [ ] 分层落位正确，无反向依赖、无跨层 import（`references/architecture.md`）
- [ ] 新增/变更的错误码有明确归属，且逐层显式映射
- [ ] 每个新行为有测试，且测试归属不重复
- [ ] lint / type check / test 全部通过（命令见 AGENTS.md）
- [ ] 没有引入新依赖、没有新增顶层目录（除非明确批准并已记录）
- [ ] 提交信息符合 Conventional Commits，scope 对应模块
- [ ] 重大决策（新依赖/新抽象层/接口变更）已写 ADR
- [ ] 最终回复里贴出了实际运行的命令与输出

## 参考文件

| 文件 | 内容 |
|---|---|
| `references/architecture.md` | 分层规范全文：各层职责、模型/配置归属、错误模型、事务与 Port 触发条件 |
| `references/complexity-ladder.md` | 复杂度阶梯：每一级何时引入、何时可降级、如何证明"够用了" |
| `references/testing.md` | 测试归属表、patch 策略、禁止的重复覆盖 |
| `references/agent-rules.md` | agent 行为红线、变更规模上限、提交/文档/决策记录规范 |
| `references/review-checklist.md` | review 清单：阻塞项与建议项 |
| `references/sources.md` | 规范出处与延伸阅读 |
| `templates/AGENTS.md` | 目标仓库的 agent 入口模板 |
| `templates/repo-layout.md` | 目录树与每个目录的职责、允许放什么 |
| `templates/tooling.md` | 各语言对应的确定性检查（lint/type/test/CI）映射 |
| `scripts/init_scaffold.py` | 生成目录骨架 + AGENTS.md 草稿 |

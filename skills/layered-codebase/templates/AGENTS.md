# AGENTS.md

> 复制到目标仓库根目录，按实际命令改写。
> **只写 agent 猜不到的东西**：命令、约定、禁区。已经能从代码读出来的结构不要复述。
> 控制在 200 行以内。

## 项目概览

<!-- 一两句：这是什么、给谁用。 -->

## 架构约定

<!-- 指向分层规范：依赖方向、每层职责、错误模型。
     如果规范就在仓库里（docs/conventions/architecture.md），这里只写一行链接 + 最容易被违反的 3 条。 -->

依赖方向：`Protocol → Service → UseCase → Provider / Repository`（只向内）

最容易被违反的三条：

1. 业务逻辑不许写在 Protocol 里。
2. Provider 不许读配置、不许调别的 Provider。
3. Repository 接收 session，不许自己 commit。

当前架构复杂度等级：<!-- 例如 2（Service + UseCase） -->
升级到下一级的触发条件：<!-- 例如 "出现第二个外部 API 时引入 Provider" -->

## 环境准备

```bash
# 安装依赖
<install cmd>

# 准备测试环境（数据库 / 容器 / 密钥）
<setup cmd>
```

## 常用命令

```bash
# 跑测试
<test cmd>

# 跑单个测试文件 / 用例
<test one cmd>

# lint
<lint cmd>

# 类型检查
<type cmd>

# 全部检查（提交前必须跑通）
<check all cmd>
```

## 提交规范

Conventional Commits：`<type>(<scope>): <description>`
type：`feat` `fix` `refactor` `test` `docs` `build` `chore` `perf`
scope：对应模块目录名（如 `services`、`providers`、`protocols/cli`）
破坏性变更加 `!` 并在正文写 `BREAKING CHANGE: ...`

## 测试约定

- 测业务 → Service 层测试，patch 具体 Provider。
- 测传输/解析 → Provider 层测试，patch SDK/HTTP。
- 测协议/退出码 → Protocol 层测试。
- 持久化用真库，不 mock Repository/session。

## 目录约定

<!-- 只列"放错地方"的常见情形，其余指向 docs 或目录树。 -->

- 新逻辑不要放在 `scripts/` 里当业务代码。
- 一次性脚本放 `scripts/`，不要漂在仓库根目录。
- `utils/` 里不许 import 应用层。
- 引擎类组件（agent loop、回测框架等）放 `src/engines/<name>/`，不许放 `utils/`，也不许写成 UseCase。

## 禁区

- 不要新增顶层目录（先问）。
- 不要引入新依赖（先问）。
- 不要顺手重构、格式化全仓库、升级依赖。
- 不要删除或跳过测试来让 CI 变绿。
- 不要提交凭据、token、账号；配置走环境变量。
- 不要用模块级可变全局（全局 client / 全局缓存）。

## 完成定义

提交前必须全部满足：

- [ ] lint / 类型检查 / 测试全部通过（贴命令与输出）
- [ ] 分层落位正确，无反向依赖
- [ ] 新的错误码有归属并逐层映射
- [ ] 新行为有测试且归属不重复
- [ ] 引入依赖 / 新增顶层目录 / 改公开接口 → 已在 PR 里说明
- [ ] 触发 ADR 条件的决策已写 `docs/adr/`

# 测试规范

## 目标

测试要能扛住内部重构；**每个行为只有一个主要测试归属**。优先测稳定的公开表面（Service 的语义化方法、Provider 函数、Protocol 入口、第三方 SDK 边界），而不是私有辅助函数。

## 标准形态

```text
用组合根构建真实的 Service
    → 调用它的语义化方法，跑真实的 Service / UseCase 代码
    → 只 patch 具体 Provider（外部调用边界）
    → 断言公开的 ServiceResult
```

不要为每一层代码都建一套平行测试层：

- Service→UseCase 的委派通常很薄，不值得再来一套 UseCase 测试。
- 组合根不需要独立的测试套件重复 Service 场景（Service 测试通过组合根构建，就等于覆盖了组合）。

## 归属表

| 测试位置 | 负责证明 | 不重复证明 |
|---|---|---|
| `tests/services/` | 完整应用动作：输入/输出校验、工作流、重试/fallback、Provider→Service 错误映射、配置传递 | Provider 传输解析、Protocol 渲染 |
| `tests/providers/` | 请求构造、SDK/HTTP 交互、响应解析、provider 本地归一化、异常→Provider 错误映射 | UseCase 工作流、Service 错误映射 |
| `tests/protocols/` | 入参解析、响应渲染、Service→协议错误码映射、退出码/状态行为、未捕获异常兜底 | 应用工作流、Provider 行为 |
| `tests/engines/` | 可选：引擎自身的行为——循环控制、工具分发、撮合规则、指标、引擎级→provider 级错误映射 | `tests/services/` 里已覆盖的应用场景 |
| `tests/utils/` | 通用工具与装饰器的直接行为 | 应用工作流、穷举实现矩阵 |
| Repository 集成测试 | 真实查询、约束、commit、rollback、fixture | Provider / Protocol 行为 |

规则：

- UseCase 是 Service 测试里的真实组件，**不要**为了直接调 `UseCase.run()` 而建例行的 `tests/use_cases/`。
- **不要**建例行的 `tests/models/` 套件；通过真正使用它的公开边界来验证。
- 只有当某个组件确实独立、无法通过 Service 公开面清晰表达时，才补更低层的测试。

## 避免重复覆盖

每条断言归属一个边界：

- Provider 测试证明：SDK 超时 → `provider_timeout`
- Service 测试证明：`provider_timeout` → 应用错误码，并触发应用级重试/fallback
- Protocol 测试证明：应用错误码 → 该协议的外部码

这三者不是重复。但"同一套工作流/校验规则在 Service 和 UseCase 各测一遍"就是重复。

保留**一个**代表性的端到端 Service 场景来证明接线正确即可，不要把完整场景矩阵复制到 Service、组合根、Protocol 三处。

用到引擎的 UseCase 在 UseCase 层测试，把引擎在**公开门面**处 fake 掉；引擎自身的行为在 `tests/engines/` 里测。不要为引擎建一套重复 `tests/services/` 应用场景的测试。

## 组合根构建

只要存在组合路径，就用组合根构建被测 Service：

```text
app_config = 从外部配置校验得到
service = build_service(app_config)
result = service.execute(合法入参)
```

这证明外部配置字段确实到达了正确的 Service/UseCase Config，也顺带覆盖了组合本身。

只有在**不存在组合路径**、或测试专门针对手工装配的变体时，才直接 new Service。不许为了图省事绕过组合。

## Patch 策略

### Service / UseCase 代码：不 patch

保持真实的 Service、真实 UseCase、真实校验、真实工作流、真实错误映射。不许注入或 mock UseCase 来测 Service。

### 具体 Provider：测应用行为时 patch

Provider 是刻意做薄、做稳的外部调用边界（参数稳定、结果归一化），所以它天然是好的 patch 点，不需要为它造接口。

```text
patch Provider 函数本身（它在 UseCase 里的调用点）
```

- 假的 Provider 返回 Provider 文档规定的 `Result` 与 provider 级数据，**包括 provider 本地归一化**。
- 不许返回 Service 的 Output 模型或协议的响应对象。
- 成功值与各种 Provider 失败（含 `unknown`）都要能伪造，用来跑应用级重试、fallback、默认分支与错误映射。

### SDK 与传输：测 Provider 时 patch

Provider 测试调用真实 Provider，patch 它的 SDK / HTTP client / 网络 / 子进程边界，在这里验证请求参数、原始响应解析、归一化与异常映射。

### 内部辅助函数：不 patch

不许 patch 私有方法、内部校验器、响应整形器。通过公开边界触达它们，这样内部重构不会炸掉测试。

### Repository 与持久化：用真库

**不许**在应用工作流测试里 mock Repository、session 或其返回值。用真实测试库 + fixture/工厂造数据，跑真实查询、约束、commit、rollback。每个测试独立隔离（独立库/schema 或可靠清理）。

不要只靠外层回滚事务包裹被测代码——那会掩盖错误的 commit 与嵌套事务行为。要验证事务隔离、锁与并发语义，必须用与生产相同的数据库引擎；只跑 SQLite 证明不了生产环境的行锁与隔离级别。

需要持久状态的测试算**集成测试**，即使用的是内存数据库。

## 工具函数测试

直接调公开函数/装饰器，断言输出或效果。覆盖正常路径 + 有意义的边界（空输入、非法值、重试耗尽、退避策略）。工具函数不该需要 Service、Provider、网络、数据库、全局配置或复杂 fixture。不许 mock 它们的内部实现，也不许把同样的用例在多层里各测一遍。

## 标记

- `unit`：快，不碰真实网络；可以伪造 Provider/SDK
- `integration`：更宽的接线或真实持久化；除非明确意图，否则不连真实外部服务

## 其他语言对应

规则语言无关，等价物举例：

| 概念 | Python | TypeScript | Go | Java |
|---|---|---|---|---|
| patch 边界 | `monkeypatch` | `vi.spyOn` / `jest.mock` | 函数变量替换（把依赖作为字段） | Mockito / 依赖注入替身 |
| 真实测试库 | pytest fixture + 测试容器 | testcontainers | dockertest / testcontainers-go | Testcontainers |
| 标记 | pytest markers | vitest tags | `testing.Short()` / build tag | JUnit tags |

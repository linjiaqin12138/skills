# 分层架构规范

语言无关。示例用伪代码表达，术语在多数语言里都有对应物（Provider ≈ HTTP/SDK 客户端封装，Repository ≈ DAO/数据访问层，UseCase ≈ 应用服务/命令对象）。

## 1. 依赖方向

```text
Protocol → Service → UseCase → Provider / Repository
```

- 只能向内依赖。
- 同层之间尽量不横向依赖；确需横向时，通过上层（Service）传递，而不是互相 import。
- `utils` 位于所有应用层之下，谁都能用；`utils` 不得 import 应用层。
- `models` 只放跨层共享结构；应用层不得反向 import `protocols`。

## 2. 各层职责

### Protocol（接入适配层）

CLI、HTTP API、MCP、消息队列消费者、Webhook、定时任务触发器。

职责：
- 反序列化入参，做**协议层**的语法/信封校验
- 调用 Service 的语义化方法
- 把 Service 结果序列化成协议响应
- 处理认证与协议级错误
- 把 Service 错误码映射成该协议的外部错误规格
- 作为最后一道边界兜住未捕获异常

禁止：
- 写业务逻辑、编排工作流
- 直接调用 UseCase / Provider / Repository
- 绕过 Service 去拿 UseCase 的 Input/Output 模型
- 直接暴露 Service 错误码或 Provider 错误码给外部（除非协议规格就是它）

### Service（对外门面）

一个领域或一组相关能力的公开门面。

职责：
- 暴露**语义化命名**的方法：`execute()`、`status()`、`create_order()`、`cancel_order()`
- 组装并持有 UseCase，传入各自 Config 与共享依赖（session factory、client factory）
- 拥有本层需要的配置契约（Config）与共享连接工厂

禁止：
- 提供通用 `run()` / `execute(input)` 这种"万能入口"
- 用 `action` 字段或大条件分支把多个独立操作塞进一个方法
- 校验 UseCase 的 Input/Output 模型（那是 `run()` 边界的事）
- 实现工作流、调用 Provider、划定数据库事务
- 实现通用的 `_execute()` 辅助方法吞掉所有调用

### UseCase（应用动作）

一个**完整且独立有意义**的应用动作。

职责：
- 业务规则、工作流编排
- 把下层错误映射成应用错误码
- 应用级重试、跨 Provider fallback、补偿策略
- 数据库事务边界（何时 begin / commit / rollback）
- 拥有自己的 `Input` / `Output` 模型与 `Config`，都放在自己旁边

标准边界：

```text
run(input):
  1. 按 Input 模型校验入参
  2. 执行 _run(typed_input) → Result
  3. 按 Output 模型校验成功结果
  4. 归一化返回数据、记录耗时
```

约束：
- 一个 UseCase = 一个动作。多个独立操作要拆成多个 UseCase。
- 可以编排多个步骤，但**不能**变成 Service 的分派表。
- 直接调用具体 Provider / Repository；只有生命周期归上层所有的依赖（session、client）才注入。
- 不读全局配置、不反查 Service、不依赖协议框架。

### Provider（出站调用）

无状态的第三方 API / HTTP / SDK 调用。

职责：
- 把鉴权、端点、序列化、SDK 细节藏在里面
- 把调用所需的一切（key、token、超时）都作为**显式参数**接收
- 固定端点写在 Provider 内部，不要求调用方传 URL
- 可做**轻量、provider 本地**的归一化：字段改名、时间戳统一、响应过滤——前提是每个调用方都需要、不含业务策略、不丢失信息

禁止：
- 读配置、选账号/用户、选择别的 Provider、管理连接生命周期
- 依赖 UseCase/Service 的 Input/Output 模型或领域模型
- 变成长期存活的服务对象

形态：
- 默认**抛异常**版本 + 用装饰器派生的**返回 Result** 版本（需要按错误码重试/fallback/映射时才用 Result 版）
- 客户端默认在单次调用内创建并关闭；不要模块级全局 client
- 确需长连接复用时，由 Service 持有 client 并显式传给 Provider

### Repository（持久化）

具体的数据库读写函数。

职责：
- 接收当前 session 作为参数
- 执行读写，**不自己开事务、不自己 commit**
- 返回调用方真正需要的东西：实体、领域模型、生成的 ID、普通值或空
- 行到模型的映射归 Repository

禁止：
- 返回 Result / ServiceResult（数据库异常按 ORM 惯例抛出即可）
- SQL 式命名（用 `create_<entity>` / `query_<entity>` / `update_<entity>` / `delete_<entity>`，不要 `insert_record`）
- 自己开事务

### utils（通用工具）

小而可复用、与领域无关的函数和装饰器：重试/退避、时间解析、文本归一化、排序分组。

约束：
- 不依赖配置、环境、网络、数据库、全局状态
- 可以包装副作用（如重试装饰器），但由调用方传入被包装的 effect 和策略
- 按能力命名模块（`utils/retry`、`utils/time`），**不许建 `common.py` 垃圾桶**
- 只被一个调用方使用且是 provider/领域专有的帮助函数，留在原地；真的通用且被复用了才提升到 `utils`

### models（共享结构）

- 放：`Result` / `Error` 这类跨层通用结构，以及**真正跨层共享**的稳定领域概念
- 不放：每个 UseCase 自己的 Input/Output（那些属于 UseCase 旁边）、纯 I/O 形状的请求响应类、所有 Pydantic 类的收集站
- 判据是**语义**而非技术：继承自 BaseModel、有校验，只说明结构，不说明它是领域概念

## 3. 配置归属

```text
全局配置文件（外部配置格式 + 加载逻辑）
    ↓ 组合根读取字段、显式构造组件 Config
Service Config（属于该 Service）
    ↓ Service 显式转换字段
UseCase Config（属于该 UseCase）
```

- 每个 Service / UseCase 拥有自己的配置契约，**类放在自己旁边**。
- 全局配置模块只描述"外部配置长什么样"和"怎么加载"，**不得** import Service/UseCase 的 Config。
- Service 只接受自己的 Config 类型；不要假设 Service Config 的某个嵌套字段结构等于 UseCase 要的。
- 不要写只做转发的 `from_config()` 类方法；组合根直接 `new Service(config)`。只有真有装配逻辑时才加 `build_*()` 工厂。
- 密钥、账号、端点走环境变量或配置注入，不进代码（12-factor 的 Config 原则）。

## 4. 错误模型

### 容器

```text
Error  { code, message, cause? }
Result { success, data?, error? }
ServiceResult extends Result { duration_seconds? }
```

- 只有**一个**通用 `Result` 容器，不要为 Provider/DB/MQ 各造一个同构的 Result 类型。
- `ServiceResult` 的错误码是**应用对外的稳定契约**。

### 码的归属与映射

```text
Provider: provider_quota_exhausted
    → Service: notification_quota_exhausted
        → REST A: UPSTREAM_QUOTA_EXHAUSTED
        → CLI:  NOTIFY_QUOTA
```

- 每层拥有自己的错误码命名空间，**逐层显式映射**，不默认透传。
- `code` 是分支契约，**不要**用异常类名、HTTP 状态码做分支契约。
- 不要为不同 Provider 建异常类继承树；需要抛已分类错误时用一个通用 `AppError(code, message)`。
- `message` 是安全可读的说明；`cause`/原始异常只作内部诊断，**不进外部响应**。

### 分类原则

- Provider 的分类按"调用方能采取什么动作"划分，而不是照抄 HTTP 状态码：
  `provider_timeout` / `provider_unavailable` / `provider_rate_limited` / `provider_quota_exhausted` / `provider_unauthorized` / `provider_forbidden` / `provider_rejected`。
- 未知异常先归 `unknown`，只有真有人需要分支时才细化。
- **禁止拿 `unknown` 做分支条件**。默认分支负责 `unknown`、已识别但无特殊策略的码、尚未出现的码。这样新增一个码不会把老调用方踢出默认路径。

### 重试与 fallback 归属

- Provider 可以内置**小的、固定的、有界**的自重试（只针对明确的瞬时故障，幂等安全）。
- UseCase 拥有请求相关的重试、跨 Provider fallback、Provider 顺序。
- Service / Router 拥有基于配置的 Provider 选择与构造；具体 Provider 不许调用别的 Provider。
- 通用重试/fallback 算法放 `utils`，由调用方传入"哪些码可重试"的谓词。

### 异常

- 业务可预期的失败走 Result；编程缺陷（bug）继续向上抛。
- 不许 catch `BaseException`、不许吞掉取消信号。
- 未捕获异常由 Protocol 兜底：CLI 一般打印原始堆栈，网络型 Protocol 记录堆栈但不对外暴露。

## 5. 事务与会话

- **UseCase 决定**事务何时开始、提交、回滚。
- Repository 通过传入的 session 参与当前事务，**永不**独立 commit。
- 独立 UseCase 默认不开嵌套事务。
- 需要把多个事务性 UseCase 组合成原子操作时，子 UseCase 暴露内部 `execute_in_transaction(session, input)`，父级调它并由父级**唯一**提交。
- **远程调用尽量放在数据库事务之外**。
- 本地事务无法让远程系统与数据库原子化——用幂等键、唯一约束、本地状态、补偿流程来协调。
- 数据库约束（唯一键、外键）作为最后一道一致性防线。

## 6. Port（可选层）

**不要因为"接了第三方"就引入 Port。** 直接调用具体 Provider，直到出现下面任一情况：

- 同一能力出现第二种实现
- 确有替换/测试替身需求
- 接口已经稳定到值得单独定义

引入时的约束：

- Port 只描述调用方真正需要的能力，不要镜像整个第三方 SDK
- Port 定义能力，不选实现；选实现属于 Router / Service 装配
- 保留现有的无状态 Provider 函数，在它们与 Port 之间加 Adapter

## 7. 组合根与资源生命周期

- 长驻服务（HTTP 服务、MQ 消费者、调度器）需要组合根：加载配置、构造共享资源与 Service、注册关闭流程。
- 一次性 CLI 可以直接构造 Service，不必强行加组合根。
- 组合根只做装配，不含业务逻辑。

## 8. 可观测性

- 静态配置与每请求上下文（用户、租户、trace、幂等键）要分开。
- 计时、结构化日志、指标、追踪放在 `run()` 边界。
- 不要把宽泛的 context 对象塞给 Provider，只传这次调用真正需要的值。
- 日志保留诊断异常，但不记录凭据与敏感载荷。

## 9. 异步与可靠性

- 显式写出策略：超时、取消、有界并发、同步 SDK 卸载、限流、分页。
- 重试必须有界，且符合操作的幂等性。
- 可靠投递类流程（下单、支付、发消息）可能需要幂等键、唯一约束、本地状态、补偿、消费端去重或事务性 outbox——这些**都不是默认必需**，按需引入。

## 10. 汇总

Service 用语义化方法聚合相关操作并持有依赖；UseCase 各自拥有一个完整动作、自己的 Config/Input/Output，并拥有工作流、重试/fallback 与事务；Provider 是无状态可 patch 的具体调用，只做 provider 本地归一化；Repository 接收 session、不提交、可直接返回领域模型；utils 只放通用无副作用工具；跨层分类契约是**错误码**而不是异常类层次；Port 及其他可选扩展只在出现具体需求时引入。

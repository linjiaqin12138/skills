# 分层架构规范

语言无关。示例用伪代码表达，术语在多数语言里都有对应物（Provider ≈ HTTP/SDK 客户端封装，Repository ≈ DAO/数据访问层，UseCase ≈ 应用服务/命令对象）。

## 1. 依赖方向

```text
Protocol → Service → UseCase → Provider / Repository
                                   ↘ Engine → Provider / Repository
```

引擎（Engine）是 Provider / Repository 同级的可选组件：应用驱动它，它内部可以使用 provider 与 repository。它既不是 utils，也不是 UseCase，详见第 3 节。

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
- 处理认证与**协议自身**的失败（非法 Host、multipart 不合法、协议版本不支持等）——这些用 Protocol 码，不冒充 Service 码
- 把 Service 错误码映射成该协议的外部错误规格（HTTP status、CLI 退出码、WS close code、响应 body）
- 作为最后一道边界兜住未捕获异常（统一 handler：按 code 映射，或记日志后兜底）

禁止：
- 写业务逻辑、编排工作流
- 直接调用 UseCase / Provider / Repository
- 绕过 Service 去拿 UseCase 的 Input/Output 模型
- 直接暴露 Service 错误码或 Provider 错误码给外部（除非协议规格就是它）
- 把 HTTP status / header / Range / multipart 形状泄漏进 Service 的返回类型

### Service（对外门面）

一个领域或一组相关能力的公开门面。

职责：
- 暴露**语义化命名**的方法：`execute()`、`status()`、`create_order()`、`cancel_order()`
- 组装并持有 UseCase，传入各自 Config 与共享依赖（session factory、client factory）
- 拥有本层需要的配置契约（Config）与共享连接工厂
- 在尚无 UseCase 的级别（Level 1）下，暂时拥有工作流、Provider 调用与错误映射；升级到 UseCase 后这些职责下沉

禁止：
- 提供通用 `run()` / `execute(input)` 这种"万能入口"
- 用 `action` 字段或大条件分支把多个独立操作塞进一个方法
- 校验 UseCase 的 Input/Output 模型（那是 `run()` 边界的事）
- 在已有 UseCase 的级别里实现工作流、调用 Provider、划定数据库事务（那些属于 UseCase）
- 实现通用的 `_execute()` 辅助方法吞掉所有调用
- 返回或持有协议细节（HTTP `statusCode`、header 名、Content-Range 等）；领域含义用 `partial` / `range` 这类词表达，由 Protocol 决定 200/206

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
- 一次函数调用对应一次（或一组紧密的）传输往返：提交、查询状态、下载内容应是**分开的** Provider 函数

禁止：
- 读配置、选账号/用户、选择别的 Provider、管理连接生命周期
- 依赖 UseCase/Service 的 Input/Output 模型或领域模型
- 变成长期存活的服务对象
- **长等待编排**：分钟级 job 轮询、跨多次调用的等待循环、业务超时策略——这些属于 UseCase（Level 1 则在 Service）
- 为「门面好看」包一层无第二种实现的 Provider 类 / 抽象基类

形态：
- **优先具体函数** + 显式 `client`/凭据/超时参数；不要默认写成持有策略的类
- 默认**抛异常**版本 + 需要按错误码重试/fallback/映射时再用 **Result** 版（见第 5 节）
- 分类失败用通用 `AppError(provider_code, message)`，不要为每个 Provider 建专用异常类
- 客户端默认在单次调用内创建并关闭；不要模块级全局 client
- 确需长连接复用时，由 Service 持有 client 并显式传给 Provider
- Provider 仅可做**小的、固定的、有界**瞬时自重试（幂等安全）；不把「等任务完成」当成自重试

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

按能力命名的小模块，分两类。**不许建 `common` / `framework` / `infra` 垃圾桶。**

#### 纯工具

无副作用：重试/退避算法、时间解析、文本归一化、路径字符串前缀判断、排序分组。

约束：
- 不依赖配置、环境、网络、数据库、全局状态
- 可以包装副作用（如重试装饰器），但由调用方传入被包装的 effect 和策略

#### 机制型效应工具（本地 IO，仍无领域）

例如「在给定 root 下 resolve 真实路径」「读文件字节」。特征：

- **无本项目领域词**（不认识 artifact / order / workspace schema）
- 所有边界（`root`、路径、超时）**显式参数**，不读 env / 全局配置
- 失败返回**机制码** Result（`not_found` / `outside_root` / `io_failed`），不抛 Service/领域码
- 由 Service / UseCase 把机制码映射成领域码

**何时独立成模块（关键）：** 语义已是稳定机制 → **立刻**放进 `utils/<capability>`（如 `utils/fs`），**即使只有一个调用方**。不要藏在某个 Service 的私有函数里——agent 发现不了就会再写一份。

**仍留在调用方旁边的：** 还夹着领域含义、或名字/边界还在飘的帮助函数。领域旁路函数等边界清楚、或第二次要写同类逻辑时再抽。

### models（共享结构）

- 放：`Result` / `Error` 这类跨层通用结构，以及**真正跨层共享**的稳定领域概念
- 不放：每个 UseCase 自己的 Input/Output（那些属于 UseCase 旁边）、纯 I/O 形状的请求响应类、所有 Pydantic 类的收集站
- 判据是**语义**而非技术：继承自 BaseModel、有校验，只说明结构，不说明它是领域概念
- `Error` 载体**不含**协议序列化方法（不要 `toResponse()` / 协议 DTO 转换）；序列化只在 Protocol

## 3. 引擎（Engine）

引擎是**由应用驱动的自洽子系统**：agent loop、回测引擎、仿真台、调度运行时。它**既不是 utils，也不是 UseCase**。

### 识别标准

一个组件同时满足以下条件，才是引擎：

- **可以作为独立 SDK 发布**。它有自己的公开 API、自己的版本、自己的测试，脱离本仓库也成立。
- **不含本项目的 domain knowledge**。它不知道 symbol、order、position、notification 是什么；领域词汇留在 UseCase / Service / models。
- **不依赖核心领域模型**。它定义自己的输入输出类型，由调用方负责与应用的类型互转。
- **可以依赖 `providers/`、`utils/` 等通用设施**。这些依赖是它公开契约的一部分，不违反分层。

判定规则：

```text
懂本项目的领域？                 → UseCase（或 Service）
无领域、无副作用（或只包装调用方传入的 effect）？ → utils/ 纯工具
无领域、本地 IO、机制码？         → utils/<capability> 机制型效应工具
无领域知识、但有状态、
且大到能自己管理生命周期？        → Engine
```

所以：**引擎永远不放 `utils/`**。utils 是按能力组织的小工具（纯或机制型）；引擎是一个带自己模型、配置、生命周期和失败模式的子系统。把引擎塞进 `utils/`，等于把一整个系统伪装成工具函数目录，后面一定会出现「状态存哪」「谁来关闭连接」「错误怎么归类」这类烂问题。

### 放置

```text
src/
└── engines/
    ├── agent_loop/
    │   ├── __init__.py       # 只导出门面
    │   ├── engine.py         # AgentLoopEngine：run(request) -> EngineResult
    │   ├── models.py         # 引擎自己的输入/输出与运行期模型
    │   ├── policy.py         # 循环控制：何时停止、何时重试、工具选择
    │   └── providers/        # 该引擎专用的第三方调用
    └── backtest/
        ├── engine.py         # BacktestEngine：run(config) -> BacktestReport
        ├── models.py
        ├── matching.py       # 撮合规则
        ├── metrics.py        # 绩效指标
        └── providers/        # 行情、资金费率等数据源
```

规则：

- `engines/` 是**可选目录**。只有满足上面的识别标准才建；否则该组件属于 `use_cases/`、`providers/` 或 `utils/`。
- 引擎在依赖深度上与 Provider / Repository 同级：可以依赖 `providers/`、`repositories/`、`utils/` 和自己的模型；**不得依赖 `protocols/`、`services/`、`use_cases/`**。应用通过 Service 或 UseCase 调用引擎，引擎不反向调用应用。
- 每个引擎只导出一个门面。调用方只 import 门面，不 import 内部模块，内部结构因此可替换。
- 只被一个引擎使用的 Provider 调用放在该引擎自己的 `providers/` 下；被多个组件共用的才上提到顶层 `providers/`。

### 副作用与生命周期

- 副作用只发生在 Provider 边界。引擎的核心逻辑——循环控制、工具分发、撮合、指标——保持纯粹或把依赖作为参数传入，这样不需要网络和数据库就能测。
- 引擎拥有自己使用的 client / session，通过配置注入，并由引擎负责关闭。**不允许模块级全局 client**。
- 运行模式是显式配置：agent loop 区分 dry-run / sandbox / live；回测引擎区分 live data / synthetic data / replay。
- 影响结果的随机性必须是显式入参。引擎不许调用全局随机数生成器的 seed 设置，那会污染同进程的其他代码。
- 引擎内部循环的重试归引擎；跨多个引擎的编排重试与 fallback 归调用它的 UseCase。
- 引擎返回结果；要不要落库由调用方通过 Repository 决定。
- 引擎要报告自己的成本与耗时：agent loop 的 token 用量与花费，回测引擎的成交与换手。

### 引擎的测试

`tests/engines/` 负责引擎自身的行为：循环控制、工具分发、撮合规则、指标、引擎级错误到 provider 级错误的映射；patch 引擎自己的 Provider，方式与 Service 测试 patch 应用 Provider 相同。

引擎不需要一套重复 `tests/services/` 应用场景的测试。用到引擎的 UseCase 在 UseCase 层测试，把引擎在**公开门面**处 fake 掉。

## 4. 配置归属

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

## 5. 错误模型

### 容器

```text
Error  { code, message, cause? }
Result { success, data?, error? }
ServiceResult extends Result { duration_seconds? }
```

- 只有**一个**通用 Error 载体与**一个**通用 `Result` 容器；不要为 Provider/DB/MQ 各造同构类型或异常类继承树。
- 需要抛已分类错误时用 `AppError(code, message, cause?)`。
- `ServiceResult` / Service 错误码是**应用对外的稳定契约**（再由各 Protocol 映射外形）。
- **可重试性不是 Error 字段。** 是否可重试由调用方按 `code` 语义决定。若某协议想给客户端 `retryable` hint，在 Protocol 的 `code → hint` 映射表生成，不写进抛错点。

### 码的归属与映射

```text
机制工具: not_found
    → Provider: provider_timeout   （若适用）
        → Service: notification_quota_exhausted
            → REST A: UPSTREAM_QUOTA_EXHAUSTED
            → CLI:  NOTIFY_QUOTA
```

- 每层拥有自己的错误码命名空间，**逐层显式映射**，不默认透传。
- `code` 是分支契约，**不要**用异常类名、HTTP 状态码做分支契约。
- Protocol 有自己的码（入参/信封/认证失败）；Service 有自己的码；二者不要糊成一张「全局枚举」却假装只有一层。
- **码表贴模块**：每个 Service / Provider / 机制工具模块旁（或文件顶部）列出本模块会产出的 code；打开文件即可看见契约。`message` 可含上下文（id、path）；固定文案才提成常量。
- `message` 是安全可读的说明；`cause`/原始异常只作内部诊断，**不进外部响应**。
- 序列化成协议 DTO（JSON body、退出码文案）只在 Protocol；Error 载体上不要挂 `toResponse()`。

### throw 还是 Result

不看「是否可预期」，看**调用方要不要在同一控制流里对失败分支**：

| 选 | 何时 |
|---|---|
| **throw** | 失败即结束当前动作；交给上层统一边界处理（Protocol handler / UseCase `run` 边界）。可预期的业务失败也可以 throw。 |
| **Result** | 同一函数内要对失败做不同事：fallback、按码重试、半成功继续、多路径恢复。 |

Level 1（尚无 UseCase）允许 Service 以 throw `AppError` 为主；当同一方法开始需要按码重试/fallback 时，再对该路径改用 Result（或升级 UseCase）。

### catch 规则

- **允许**：入口统一 catch——按 code 映射协议响应 / 打日志 / 再抛；或把未分类异常收成 `unknown`/`internal`。
- **禁止**：在 catch 里做业务策略菜单（`if missing → 领域码 A; else if …`）。那是 Result 的信号；应让下层返回机制码 Result，调用方线性映射。
- 若统一边界要「按 code 调 handler」，handler **必须显式接收上下文**；缺上下文时不要用全局 handler 表硬凑——抛错时带齐字段，或改在现场用 Result。
- 不许 catch `BaseException`、不许吞掉取消信号。
- 未捕获异常由 Protocol 兜底：CLI 一般打印原始堆栈，网络型 Protocol 记录堆栈但不对外暴露。
- **禁止**把 Service 失败一律当成 `unknown`；未映射才走兜底。

### 分类原则

- Provider 的分类按"调用方能采取什么动作"划分，而不是照抄 HTTP 状态码：
  `provider_timeout` / `provider_unavailable` / `provider_rate_limited` / `provider_quota_exhausted` / `provider_unauthorized` / `provider_forbidden` / `provider_rejected`。
- 未知异常先归 `unknown`，只有真有人需要分支时才细化。
- **禁止拿 `unknown` 做分支条件**。默认分支负责 `unknown`、已识别但无特殊策略的码、尚未出现的码。这样新增一个码不会把老调用方踢出默认路径。

### 重试与 fallback 归属

- Provider 可以内置**小的、固定的、有界**的自重试（只针对明确的瞬时故障，幂等安全）。
- UseCase 拥有请求相关的重试、跨 Provider fallback、Provider 顺序、**长轮询/等任务完成**。
- Service / Router 拥有基于配置的 Provider 选择与构造；具体 Provider 不许调用别的 Provider。
- 通用重试/fallback 算法放 `utils`，由调用方传入"哪些码可重试"的谓词。

## 6. 事务与会话

- **UseCase 决定**事务何时开始、提交、回滚。
- Repository 通过传入的 session 参与当前事务，**永不**独立 commit。
- 独立 UseCase 默认不开嵌套事务。
- 需要把多个事务性 UseCase 组合成原子操作时，子 UseCase 暴露内部 `execute_in_transaction(session, input)`，父级调它并由父级**唯一**提交。
- **远程调用尽量放在数据库事务之外**。
- 本地事务无法让远程系统与数据库原子化——用幂等键、唯一约束、本地状态、补偿流程来协调。
- 数据库约束（唯一键、外键）作为最后一道一致性防线。

## 7. Port（可选层）

**不要因为"接了第三方"就引入 Port。** 直接调用具体 Provider，直到出现下面任一情况：

- 同一能力出现第二种实现
- 确有替换/测试替身需求
- 接口已经稳定到值得单独定义

引入时的约束：

- Port 只描述调用方真正需要的能力，不要镜像整个第三方 SDK
- Port 定义能力，不选实现；选实现属于 Router / Service 装配
- 保留现有的无状态 Provider 函数，在它们与 Port 之间加 Adapter

## 8. 组合根与资源生命周期

- 长驻服务（HTTP 服务、MQ 消费者、调度器）需要组合根：加载配置、构造共享资源与 Service、注册关闭流程。
- 一次性 CLI 可以直接构造 Service，不必强行加组合根。
- 组合根只做装配，不含业务逻辑。

## 9. 可观测性

- 静态配置与每请求上下文（用户、租户、trace、幂等键）要分开。
- 计时、结构化日志、指标、追踪放在 `run()` / Protocol 入口边界。
- 不要把宽泛的 context 对象塞给 Provider，只传这次调用真正需要的值。
- 日志级别：`error` / `warn` / `info` / `debug`（需要更细时再加 `trace`）。
- **协议通道 ≠ 日志**：CLI/HTTP 的业务结果走协议输出（如 CLI stdout 的结果 JSON）；诊断走 logger/stderr。不要把给机器解析的协议结果改成「打一行 log」就完事。
- 日志保留诊断异常，但不记录凭据、token、完整敏感载荷（提示词全文、密钥、个人信息等）。

## 10. 异步与可靠性

- 显式写出策略：超时、取消、有界并发、同步 SDK 卸载、限流、分页。
- 重试必须有界，且符合操作的幂等性。
- 可靠投递类流程（下单、支付、发消息）可能需要幂等键、唯一约束、本地状态、补偿、消费端去重或事务性 outbox——这些**都不是默认必需**，按需引入。

## 11. 汇总

Service 用语义化方法聚合相关操作并持有依赖；UseCase 各自拥有一个完整动作、自己的 Config/Input/Output，并拥有工作流、重试/fallback、长等待与事务；Provider 是无状态可 patch 的**具体函数**，只做单次传输与本地归一化，不做长轮询编排；Repository 接收 session、不提交、可直接返回领域模型；utils 含纯工具与机制型效应工具（按能力命名、机制码、可单调用方独立模块）；引擎（Engine）放无领域知识、可独立发布的子系统；跨层分类契约是**错误码**（载体可一、码分属、序列化在 Protocol）；throw 与 Result 按「现场是否要分支」选择；Port 及其他可选扩展只在出现具体需求时引入。

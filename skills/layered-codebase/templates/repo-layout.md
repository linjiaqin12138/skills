# 目录布局模板

**只建当前复杂度等级需要的目录。** 空目录不要建；建了没人用的目录就是"为凑图引入抽象"。

## 全量形态（最高级别，仅作参照，通常用不到）

```text
project_root/
├── src/
│   ├── protocols/          # 接入适配器：cli/、http/、mcp/、consumer/
│   ├── services/           # 对外门面、依赖所有权与装配
│   ├── use_cases/          # 一个应用动作一个文件
│   ├── providers/          # 无状态第三方调用
│   ├── repositories/       # 数据库读写
│   ├── engines/            # 可选：可独立发布的子系统（agent loop、回测引擎）
│   ├── ports/              # 可选：Port（需触发条件才建）
│   ├── adapters/           # 可选：Port 的实现
│   ├── models/             # 跨层共享结构 + Result/Error
│   ├── utils/              # 纯工具 + 机制型效应工具（按能力，如 fs/、retry/）
│   └── config.(ext)        # 外部配置结构与加载
├── tests/
│   ├── services/
│   ├── providers/
│   ├── protocols/
│   ├── utils/
│   └── repositories/
├── docs/
│   ├── adr/
│   └── ...
├── scripts/                # 一次性脚本、运维脚本
└── AGENTS.md
```

## 按等级裁剪

| 级别 | 实际建什么 |
|---|---|
| 0 | `scripts/`（或仓库根的一个明确命名的脚本）+ `tests/`（若值得测） |
| 1 | `src/services/`、`src/protocols/`、`tests/services/` |
| 2 | 加 `src/use_cases/`、`tests/`（不按层建目录，Service 测试里覆盖） |
| 3 | 加 `src/providers/`、`tests/providers/` |
| 4 | 加 `src/repositories/`、`tests/repositories/` |
| 5 | 加 `src/ports/`、`src/adapters/` |
| 6 | 不用新建目录，UnitOfWork 放在 `src/repositories/` 或 `src/use_cases/` 里 |
| 7 | 加 `src/composition.(ext)` 或 `src/bootstrap.(ext)` |
| 8 | 不新建目录，请求上下文放在 `src/models/` 或各层旁边 |

> `engines/` 不在阶梯里——它不是"更深的一层"，而是**另一种组件类型**。是否建 `engines/` 取决于组件是否满足"可独立发布的 SDK、无 domain knowledge、不依赖核心模型"这三条判据（见 `references/complexity-ladder.md` 的组件归属判定）。

## 每个目录放什么 / 不放什么

| 目录 | 放 | 不放 |
|---|---|---|
| `protocols/` | 入参解析、协议校验、调用 Service、结果渲染、错误码映射、认证 | 业务逻辑、直接调 UseCase/Provider/Repository、业务模型定义 |
| `services/` | 语义化方法、UseCase 组装、Config、共享连接工厂 | 通用 `run()`、工作流、事务、UseCase 模型校验 |
| `use_cases/` | 一个动作一个文件：Input/Output/Config + 业务规则 + 编排 + 错误映射 + 事务 | 多动作分派、读全局配置、依赖协议框架 |
| `providers/` | 一个外部源一组**无状态函数**：单次传输、provider 本地归一化 | 配置读取、账号选择、调用其他 Provider、全局 client、应用模型、长轮询编排、无第二种实现的门面类 |
| `repositories/` | 按领域能力命名的读写函数 | 自己开事务、返回 Result、SQL 式命名 |
| `engines/` | 自洽子系统：门面 + 自己的模型 + 策略 + 生命周期；内部可带 `providers/` | 领域知识、依赖 protocols/services/use_cases、把内部模块直接暴露给调用方 |
| `ports/` | 能力接口（需第二种实现时） | 镜像第三方 SDK、选实现 |
| `adapters/` | Port 的具体实现 | 业务逻辑 |
| `models/` | 跨层共享的领域结构、`Result`/`Error` | 各 UseCase 的 Input/Output、所有数据类、协议序列化方法 |
| `utils/` | 纯工具；机制型效应工具（无领域词、显式参数、机制码 Result） | import 应用层、读全局配置、`common`/`framework`/`infra`、领域码 |
| `scripts/` | 一次性脚本、迁移脚本、运维任务 | 被其他模块 import 的业务逻辑 |
| `tests/` | 按归属层镜像 `src/` | 平行测试套件、mock 掉本该真实的部分 |

## 文件命名

- 模块名按**能力**命名，不按类型：`utils/retry`、`utils/fs`、`providers/market_data`、`services/agent`，不要 `utils/common`、`providers/provider`、`services/agent_service`。
- 一个文件一个主要职责；同名类型与文件名对齐（`providers/market_data` 里的函数叫 `fetch_quote`）。
- Repository 函数按领域能力：`create_order` / `query_orders` / `update_order` / `delete_order`，不要 `insert_record` / `select_all`。
- Provider 函数用「动词_名词」：`fetch_quote`、`submit_video_job`、`get_video_job`；一次函数一次传输边界。
- Service 方法用语义化动词短语：`execute` / `status` / `create_order` / `cancel_order`。
- UseCase 类名用「动作 + UseCase」：`ExecuteOrderUseCase`、`QueryStatusUseCase`。
- 跨模块协议字面量（消息 type、协议版本等）放**唯一来源**（共享 schema / 常量模块），调用方引用，不复制字符串。
- 文件名风格在仓库内统一（snake_case 或 kebab-case 二选一，不要混）。

## 一次性脚本怎么办

不要在仓库根目录漂一堆 `fix_data.py`、`test2.py`、`new_script.sh`。三种正确去处：

1. 能变成长期能力的 → 做成 Protocol 层的子命令。
2. 一次性运维任务 → `scripts/` 下，命名说明用途与一次性（如 `scripts/migrate_2026_11_user_ids.ext`）。
3. 临时验证 → 本地临时目录，不要提交。

## 组合根放哪

- `src/composition.(ext)` 或 `src/bootstrap.(ext)`，二选一，全仓库统一。
- 内容只有：加载外部配置 → 构造各 Service Config → 构造 Service → 注册关闭流程。
- 不含业务逻辑，不含条件分支的业务判断。
- 一次性 CLI 可以不设组合根，直接构造 Service。

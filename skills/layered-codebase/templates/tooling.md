# 工具链映射：把规则变成确定性检查

原则：**能在工具里判定的规则，不要只写在文档里。**

每条规则给三个位置：
- **工具**：能自动判定的，交给工具
- **命令**：怎么跑
- **证据**：提交/合并前贴什么

## 分层依赖检查（跨语言）

| 语言 | 工具 | 关键规则 |
|---|---|---|
| Python | `import-linter`（`importlinter`） | 契约：`protocols` 不得 import `use_cases`；`providers` 不得 import `services`；`utils` 不得 import 任何应用层 |
| TypeScript | `eslint-plugin-boundaries` 或 `dependency-cruiser` | 同上，按目录配层 |
| Go | `go list -deps` + 自定义脚本 / `depguard` | 同上 |
| Java | ArchUnit | `ArchRule` 表达"层 A 不得依赖层 B" |
| 任意 | 目录结构 lint 脚本 | 顶层目录白名单 |

命令示例：

```bash
# Python
lint-imports
# TypeScript
depcruise --config .dependency-cruiser.js src
# Java
./mvnw -q test -Dtest=ArchitectureTest
```

## 静态检查 / 类型

| 语言 | lint | 类型检查 | 强制级别 |
|---|---|---|---|
| Python | `ruff` | `mypy --strict` | `mypy` 建议开 strict；`Any`、未标注函数、未用 ignore 都算错 |
| TypeScript | `eslint` + `typescript-eslint` | `tsc --noEmit`（`strict: true`） | strict 必开 |
| Go | `golangci-lint` | 内置 | `errcheck`、`staticcheck` 必开 |
| Java | `checkstyle` / `spotless` | — | — |
| Rust | `clippy` | `cargo check` | — |

规则示例（ruff，`pyproject.toml`）：

```toml
[tool.ruff]
line-length = 100
src = ["src", "tests"]

[tool.ruff.lint]
select = ["E", "F", "I", "N", "UP", "B", "SIM", "ANN", "S", "C4", "RET", "ARG", "PL"]
ignore = ["ANN401"]  # 允许在明确标注 Any 的边界使用

[tool.ruff.lint.per-file-ignores]
"tests/*" = ["S101"]  # 允许 assert

[tool.mypy]
python_version = "3.12"
strict = true
packages = ["src"]
```

## 测试与覆盖率

| 语言 | 跑测试 | 标记 | 真实数据库 |
|---|---|---|---|
| Python | `pytest` | `@pytest.mark.unit` / `integration` | testcontainers / docker-compose |
| TypeScript | `vitest` / `jest` | `test.todo`、tag | testcontainers |
| Go | `go test ./...` | build tag | testcontainers-go |
| Java | JUnit 5 | `@Tag` | Testcontainers |

覆盖率只作为**观察指标**，不作为门禁（否则会逼出无意义的测试）。

## 提交前检查（pre-commit / CI）

推荐组合：

```text
pre-commit（本地，快，拦住大部分）
  ├── ruff check --fix
  ├── ruff format
  ├── mypy
  ├── import-linter
  └── 密钥扫描

CI（权威，合并门禁）
  ├── 上面全部
  ├── 完整测试（含集成测试）
  └── 覆盖率报告（仅报告）
```

git hooks 示例：

```bash
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.6.9
    hooks:
      - id: ruff
        args: [--fix]
      - id: ruff-format
  - repo: https://github.com/pre-commit/mirrors-mypy
    rev: v1.11.2
    hooks:
      - id: mypy
        additional_dependencies: [types-requests]
  - repo: https://github.com/s-weigand/import-linter
    rev: v2.1
    hooks:
      - id: import-linter
```

## Agent 侧的确定性约束

| 想约束的行为 | 手段 |
|---|---|
| 不许改某些目录/文件 | `PreToolUse` hook 拦截，退出码 2 |
| 不许引入新依赖 | CI 检查依赖文件 diff / `postinstall` 守卫脚本 |
| 不许新增顶层目录 | 目录结构 lint |
| 不许提交前不跑检查 | `Stop` hook：检查未通过时把命令与输出退回给 agent |
| 不许读密钥文件 | 权限规则（deny），比文档可靠 |
| 提交信息格式 | `commit-msg` hook + commitlint |

> 经验：**"请务必 X"写在文档里是建议；进 hook 才是约束。** 优先把已经踩过坑的规则升级成工具检查。

## 落地顺序建议

1. 先把 **lint + 类型检查 + 测试** 接上（收益最大、争议最小）。
2. 再加 **依赖方向检查**（分层规范的核心保障）。
3. 再加 **提交前 hook / Stop hook**（把"完成定义"变成自动门禁）。
4. 最后才考虑更细的白名单（顶层目录、依赖新增）。

每加一条检查，都要确认它在**干净仓库上是通过的**，否则会变成天天被跳过的噪音。

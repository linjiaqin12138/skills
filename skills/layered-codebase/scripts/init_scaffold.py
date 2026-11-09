#!/usr/bin/env python3
"""按复杂度阶梯生成目录骨架 + AGENTS.md 草稿。

只建当前等级需要的目录，不多建。

用法:
    python init_scaffold.py <target_dir> --level 2 --lang python [--force]
    python init_scaffold.py . --level 0 --lang generic
"""

from __future__ import annotations

import argparse
from pathlib import Path

LANG_EXT = {
    "python": "py",
    "typescript": "ts",
    "go": "go",
    "java": "java",
    "generic": "ext",
}

LANG_COMMANDS = {
    "python": {
        "install": "pip install -e '.[dev]'",
        "test": "pytest",
        "test_one": "pytest path/to/test_file.py::test_name",
        "lint": "ruff check . && ruff format --check .",
        "type": "mypy src",
        "check_all": "ruff check . && mypy src && pytest",
    },
    "typescript": {
        "install": "pnpm install",
        "test": "pnpm test",
        "test_one": "pnpm vitest run -t \"<name>\"",
        "lint": "pnpm lint",
        "type": "pnpm typecheck",
        "check_all": "pnpm lint && pnpm typecheck && pnpm test",
    },
    "go": {
        "install": "go mod download",
        "test": "go test ./...",
        "test_one": "go test ./pkg/... -run TestName",
        "lint": "golangci-lint run",
        "type": "go vet ./...",
        "check_all": "golangci-lint run && go vet ./... && go test ./...",
    },
    "java": {
        "install": "./mvnw -q dependency:go-offline",
        "test": "./mvnw test",
        "test_one": "./mvnw test -Dtest=ClassName#methodName",
        "lint": "./mvnw spotless:check",
        "type": "./mvnw -q compile",
        "check_all": "./mvnw spotless:check && ./mvnw test",
    },
    "generic": {
        "install": "<install cmd>",
        "test": "<test cmd>",
        "test_one": "<test one cmd>",
        "lint": "<lint cmd>",
        "type": "<type cmd>",
        "check_all": "<check all cmd>",
    },
}

# 每一级新增的源码目录 / 测试目录
SRC_STEPS = [
    (1, ["protocols", "services", "utils"]),
    (2, ["use_cases", "models"]),
    (3, ["providers"]),
    (4, ["repositories"]),
    (5, ["ports", "adapters"]),
]
TEST_STEPS = [
    (1, ["services", "protocols", "utils"]),
    (3, ["providers"]),
    (4, ["repositories"]),
]


def _level_dirs(level: int) -> tuple[str, list[str], list[str]]:
    """返回 (源码根目录, 源码子目录, 测试子目录)。"""
    if level == 0:
        return "", [], []
    src: list[str] = []
    tests: list[str] = []
    for step, dirs in SRC_STEPS:
        if level >= step:
            src.extend(dirs)
    for step, dirs in TEST_STEPS:
        if level >= step:
            tests.extend(dirs)
    return "src", src, tests


LEVELS = {n: _level_dirs(n) for n in range(9)}

NEXT_TRIGGER = {
    0: "出现第二处调用同一段逻辑时，引入 Service（级别 1）",
    1: "Service 里出现第二条独立工作流时，引入 UseCase（级别 2）",
    2: "出现第二个外部 API/SDK，或需要稳定的 patch 边界时，引入 Provider（级别 3）",
    3: "出现真实持久化需求时，引入 Repository（级别 4）",
    4: "同一能力出现第二种实现时，才考虑 Port/Adapter（级别 5）",
    5: "重复的事务生命周期代码出现且 UnitOfWork 能消除它时（级别 6）",
    6: "出现长驻进程需要共享资源与关闭流程时，引入组合根（级别 7）",
    7: "出现多租户/分布式追踪需求时，引入请求上下文（级别 8）",
    8: "已到顶；新增抽象前先回看降级条件",
}

AGENTS_TEMPLATE = """# AGENTS.md

> 只写 agent 猜不到的东西：命令、约定、禁区。结构能从代码读出来的不要复述。
> 控制在 200 行以内。

## 项目概览

{overview}

## 架构约定

依赖方向：`Protocol → Service → UseCase → Provider / Repository`（只向内）

最容易被违反的三条：

1. 业务逻辑不许写在 Protocol 里。
2. Provider 不许读配置、不许调别的 Provider。
3. Repository 接收 session，不许自己 commit。

当前架构复杂度等级：{level}
升级到下一级的触发条件：{next_trigger}

## 环境准备

```bash
{install}
```

## 常用命令

```bash
# 跑测试
{test}

# 跑单个测试
{test_one}

# lint
{lint}

# 类型检查
{type}

# 全部检查（提交前必须跑通）
{check_all}
```

## 提交规范

Conventional Commits：`<type>(<scope>): <description>`
scope 对应模块目录名（如 `services`、`providers`、`protocols/cli`）。
破坏性变更加 `!` 并在正文写 `BREAKING CHANGE: ...`。

## 测试约定

- 测业务 → Service 层测试，patch 具体 Provider。
- 测传输/解析 → Provider 层测试，patch SDK/HTTP。
- 测协议/退出码 → Protocol 层测试。
- 持久化用真库，不 mock Repository/session。

## 目录约定

- 新逻辑不要放在 `scripts/` 里当业务代码。
- 一次性脚本放 `scripts/`，不要漂在仓库根目录。
- `utils/` 里不许 import 应用层。

## 禁区

- 不要新增顶层目录（先问）。
- 不要引入新依赖（先问）。
- 不要顺手重构、格式化全仓库、升级依赖。
- 不要删除或跳过测试来让 CI 变绿。
- 不要提交凭据、token、账号；配置走环境变量。
- 不要用模块级可变全局（全局 client / 全局缓存）。

## 完成定义

- [ ] lint / 类型检查 / 测试全部通过（贴命令与输出）
- [ ] 分层落位正确，无反向依赖
- [ ] 新的错误码有归属并逐层映射
- [ ] 新行为有测试且归属不重复
- [ ] 引入依赖 / 新增顶层目录 / 改公开接口 → 已在 PR 里说明
- [ ] 触发 ADR 条件的决策已写 `docs/adr/`
"""

README_SNIPPET = """
## 架构

本项目采用分层架构，依赖方向 `Protocol → Service → UseCase → Provider / Repository`（只向内）。

当前复杂度等级：{level}

| 目录 | 职责 |
|---|---|
{rows}

升级到下一级的触发条件：{next_trigger}
"""


def build(level: int, lang: str, overview: str) -> tuple[list[str], str, str]:
    src_root, src_dirs, test_dirs = LEVELS[level]
    created: list[str] = []

    if src_root:
        for d in src_dirs:
            created.append(f"{src_root}/{d}/")
    for d in test_dirs:
        created.append(f"tests/{d}/")
    if level == 0:
        created.append("scripts/")
    if level >= 7:
        created.append(f"{src_root}/composition.{LANG_EXT[lang]}")

    cmds = LANG_COMMANDS[lang]
    agents = AGENTS_TEMPLATE.format(
        overview=overview or "<!-- 一两句：这是什么、给谁用。 -->",
        level=level,
        next_trigger=NEXT_TRIGGER[level],
        **cmds,
    )

    rows = ""
    desc = {
        "protocols": "接入适配层：解析入参、调用 Service、渲染结果、错误码映射",
        "services": "对外门面：语义化方法、组装 UseCase、持有共享依赖",
        "use_cases": "一个应用动作一个：业务规则、编排、错误映射、事务边界",
        "providers": "无状态第三方调用：藏传输细节，可被 patch",
        "repositories": "数据库读写：接收 session，不 commit",
        "ports": "可选：能力接口（需第二种实现时才建）",
        "adapters": "可选：Port 的具体实现",
        "models": "跨层共享结构 + Result/Error",
        "utils": "通用无副作用工具（不许 import 应用层）",
    }
    src_root, src_dirs, _ = LEVELS[level]
    if src_root:
        for d in src_dirs:
            rows += f"| `{src_root}/{d}/` | {desc.get(d, '')} |\n"

    readme = README_SNIPPET.format(level=level, rows=rows, next_trigger=NEXT_TRIGGER[level])
    return created, agents, readme


def main() -> int:
    ap = argparse.ArgumentParser(description="生成分层架构目录骨架")
    ap.add_argument("target", help="目标目录")
    ap.add_argument("--level", type=int, default=2, choices=sorted(LEVELS), help="复杂度等级 0-8")
    ap.add_argument("--lang", default="generic", choices=sorted(LANG_EXT), help="语言/生态")
    ap.add_argument("--overview", default="", help="项目一句话描述，写进 AGENTS.md")
    ap.add_argument("--force", action="store_true", help="覆盖已存在的 AGENTS.md / README 片段")
    args = ap.parse_args()

    root = Path(args.target).expanduser().resolve()
    root.mkdir(parents=True, exist_ok=True)
    created, agents, readme = build(args.level, args.lang, args.overview)

    for rel in created:
        if rel.endswith("/"):
            (root / rel).mkdir(parents=True, exist_ok=True)
        else:
            path = root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            if not path.exists():
                path.write_text("", encoding="utf-8")

    agents_path = root / "AGENTS.md"
    if agents_path.exists() and not args.force:
        print(f"skip (exists): {agents_path}")
    else:
        agents_path.write_text(agents, encoding="utf-8")
        print(f"wrote: {agents_path}")

    if args.level >= 7:
        comp = root / "src" / f"composition.{LANG_EXT[args.lang]}"
        if not comp.exists():
            comp.write_text(
                "# 组合根：只做装配，不含业务逻辑\n"
                "# 1. 加载外部配置\n"
                "# 2. 构造各 Service 的 Config\n"
                "# 3. 构造 Service\n"
                "# 4. 注册关闭流程\n",
                encoding="utf-8",
            )
            print(f"wrote: {comp}")

    print("\n创建/确认的目录：")
    for rel in created:
        print(f"  {rel}")
    print("\nREADME 架构片段（自行粘贴到 README.md）：")
    print(readme)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

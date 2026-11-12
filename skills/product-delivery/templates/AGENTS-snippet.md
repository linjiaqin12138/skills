# 可粘贴进目标仓库 AGENTS.md 的段落

按项目改路径即可。

```markdown
## 需求与变更

产品意图、分档（S/M/L）、闸门、文档活/冷与冻结规则：遵循
[product-delivery](https://github.com/linjiaqin12138/skills/tree/main/skills/product-delivery)
（或本组织内该 skill 的路径）。接到改行为的任务时先判档；L/M 未确认前不写业务实现。

现行产品读 `docs/product/` 与 `docs/architecture/`；现行契约读 `docs/contracts/current/`（或生成物）；
`docs/changes/`（或 `docs/requirements/`）在落地后视为历史快照，不回写正文。

## 编码

写业务代码时遵循
[layered-codebase](https://github.com/linjiaqin12138/skills/tree/main/skills/layered-codebase)
及本仓库 `docs/conventions/architecture.md`（若有）。确认可写码之后再实现。
```

若 skill 以 monorepo 相对路径引用，把 URL 换成仓库内相对链接。

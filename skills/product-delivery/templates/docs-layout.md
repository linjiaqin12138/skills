# 目标仓库文档目录树（模板）

接入本规范时按需创建，**不预建空目录**。命名可改，语义保持不变。

```text
docs/
  product/
    VISION.md
    MVP.md
    CAPABILITIES.md
    GLOSSARY.md                 # 可选
  architecture/
    overview.md
    spine.md
    flows/
      <flow-name>.md            # 现行关键时序
    adr/
      0001-<slug>.md
  contracts/
    current/
      events.md                 # 按需
      api.md                    # 或指向生成的 openapi
      config.md
  changes/                      # 或 requirements/，二选一写进 conventions
    0001-<slug>/
      brief.md
      …
  conventions/
    product-delivery.md         # 短指针：读本 skill；写码读 layered-codebase
CHANGELOG.md                    # 仓库根或 docs/CHANGELOG.md
```

字段与章节：[`../references/doc-model.md`](../references/doc-model.md)。  
图归属：主 SKILL「图放哪里」。

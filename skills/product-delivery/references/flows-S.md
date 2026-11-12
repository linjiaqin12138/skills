# S：快车道

适用于：语义基本不动、改动面小。详见主 SKILL 分档示例。

## 步骤

1. **复述范围**（一句）：改什么、不改什么。请求已无歧义可省略。
2. **必要时 5 分钟 explore**：确认真是小面；若发现多调用点/多契约 → **升档**，改走 M/L。
3. **改代码**（若需要）：遵循编码阶段 → 读 [`layered-codebase`](../../layered-codebase/SKILL.md)。
4. **活文档**：
   - 对外行为变了 → `CHANGELOG` 一行
   - 协议/配置形状变了 → 同步 `contracts/current` 或再生生成物
   - 模块关系未变 → 不动 `architecture/overview`
5. **收尾**：按 S 的 DoD 自检；回复里说明改了什么、如何验证。

## 禁止

- 新建完整 L 变更包「走个形式」
- 回写 Frozen 变更包正文
- 静默改 `product/MVP` 或 spine 扩展点
- 把「顺便重构一大圈」塞进同一个 S

## CHANGELOG 约定

- 用户能感知的行为、对外字段/事件名、默认配置语义 → 写
- 纯内部 rename、测试补充、格式化 → 可不写
- 格式见 [`doc-model.md`](doc-model.md) §4

## 完成标准

见主 SKILL Definition of Done（S）。

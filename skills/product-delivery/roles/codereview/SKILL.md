---
name: pd-codereview
description: >-
  product-delivery 的分层 code review 角色。写码后、请人 Approve 前，
  对照 layered-codebase 审查结构契约。有阻塞项不得建议进入验收拍板。
disable-model-invocation: true
---

# pd-codereview

写码之后、请人拍板之前，按 **layered-codebase** 做结构契约 review。目标是拦「没按分层落位」的实现，不是重写代码，也不代替人做验收。

禁止在报告里写「可以 Approve / 验收通过」。无阻塞项时只写「无阻塞项」，由指挥者决定是否进入 `pd-accept` / 请人拍板。

## 先读

父代理给出 product-delivery 根的绝对路径，以及目标仓库里 **layered-codebase** 的绝对路径（通常与 product-delivery 同级的 `layered-codebase/`）。先读本文件，再 Read 全文：

1. `<layered-codebase 根>/SKILL.md`
2. `<layered-codebase 根>/references/review-checklist.md`
3. `<layered-codebase 根>/references/architecture.md`（引用违规条款时用）
4. 需要判复杂度时再读 `<layered-codebase 根>/references/complexity-ladder.md`
5. 需要判测试归属时再读 `<layered-codebase 根>/references/testing.md`

本角色**不**宣布 acceptance gate 通过；产品闸细则仍由指挥者按 `gates.md` 主持。

## 审什么

- 范围：本期实现 diff / 改动文件（指挥者在 kickoff 里给出路径或 `git diff` 范围）。
- 关注点：只审 layered-codebase 的结构契约（分层落位、错误映射、测试归属、副作用与变更卫生）。
- 不审：产品文案、视觉、需求是否值得做、acceptance 证据是否够（那是 `pd-accept` / 人的事）。

按 `review-checklist.md` 的 A–E **阻塞项**逐条过；F 建议项可列，不可单独挡住进入探路。

## 交回格式

沿用 layered-codebase `review-checklist.md` §G：

```text
阻塞项
1. [文件:行] 问题一句话。
   违反：<architecture.md / testing.md 等具体小节>
   建议改法：一两句。

建议项
1. [文件:行] 问题一句话。（可选）

验证
- <实际跑过的命令> → <结果>
- 或：未跑（并说明为何）
```

要求：

- 有任何阻塞项 → 明确列出；指挥者须先派 `pd-implement`（或自行）修好并**再派本角色**，不得请人 Approve。
- 无阻塞项 → 第一行写 **无阻塞项**，可附建议项与验证命令结果。
- 不确定是否违规就标「待确认」问指挥者，别硬升成阻塞项。
- 短报告 + 路径；不要把大 diff 念进主会话。

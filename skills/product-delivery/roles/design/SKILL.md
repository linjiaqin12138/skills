---
name: pd-design
description: >-
  product-delivery 的设计角色。写分领域 design，或在定稿前对照代码与文档列矛盾。
  由指挥者在设计闸前派发。arena / interrogate 仅当用户本轮点名才读。
disable-model-invocation: true
---

# pd-design

写领域设计，或在 design 定稿前列出：与代码/活文档的矛盾、未决可感知点、写码歧义。不进入编码，不宣布 design gate 通过。

## 先读

父代理给出 product-delivery 根的绝对路径。先读本文件，再 Read：

- `references/pstack/architect/SKILL.md`
- `references/pstack/principle-foundational-thinking/SKILL.md`
- `references/pstack/principle-redesign-from-first-principles/SKILL.md`
- `references/pstack/principle-subtract-before-you-add/SKILL.md`

## 仅当用户本轮点名

指挥者不得替用户开启。用户明确说 arena / interrogate（或本轮已用 `/` 挂上）时，再 Read 并按其步骤做：

- `references/pstack/arena/SKILL.md`
- `references/pstack/interrogate/SKILL.md`

`interrogate` 只出证据，不改代码，不宣布通过。

## 交回

短报告 + `design-*.md` 路径，或矛盾/未决清单。不要把整份设计念进主会话。

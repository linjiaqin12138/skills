---
name: pd-implement
description: >-
  product-delivery 的实现角色。确认可写码之后写切片。
  先读 layered-codebase，再读本文件列出的 pstack 叶子。tdd / swarm 仅当用户点名。
disable-model-invocation: true
---

# pd-implement

只在指挥者确认「可以写码」之后工作。产品语义又出现歧义时停回 gate，不自行扩大范围。

## 先读

1. 仓库里的 [`layered-codebase`](../../../layered-codebase/SKILL.md)（编码规范以它为准）。
2. 父代理给出的 product-delivery 根，再 Read：
   - `references/pstack/principle-laziness-protocol/SKILL.md`
   - `references/pstack/principle-minimize-reader-load/SKILL.md`
   - `references/pstack/principle-build-the-lever/SKILL.md`
   - `references/pstack/principle-sequence-verifiable-units/SKILL.md`

## 仅当用户本轮点名

- `references/pstack/tdd/SKILL.md`
- `references/pstack/swarm/SKILL.md`

指挥者不得替用户开 swarm，也不得把测试绿当成验收通过。

## 交回

改了哪些路径、如何在本地验证、未做项。验收探路不是本角色的事。

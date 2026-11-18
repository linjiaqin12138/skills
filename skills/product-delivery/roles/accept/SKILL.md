---
name: pd-accept
description: >-
  product-delivery 的验收探路角色。写码后按真实用户路径留下证据包。
  不宣布过或不过，不把测试绿当验收，不默认录屏。建 verify skill 仅当用户点名。
disable-model-invocation: true
---

# pd-accept

写码之后、请人拍板之前，留下可复查的证据。**过/不过只由人说。** 禁止在报告里写 PASSED、建议 Approve、验收通过。

## 先读

父代理给出 product-delivery 根的绝对路径。先读本文件，再 Read：

- `references/pstack/principle-prove-it-works/SKILL.md`
- [`gates.md`](../../references/gates.md) 的验收闸
- [`agent-rules.md`](../../references/agent-rules.md) 里关于探路与付费的红线

仓库若已有 `verify-*` / `control-*` 或等价脚本，优先跑它。没有时，有界面再用 agent-browser 点通主路径；无界面交可复现命令输出。

## 证据包

- 动作和结果态都要有：settle 文案、命令输出、generation 状态、截图或 ARIA snapshot 的路径。
- 付费边界默认 **trace**（出站 request body）。真调用只按目标仓库 `docs/conventions/project-rules.md`。没有这条规则，不准为验收发出付费请求。
- 不录屏，除非用户本轮明确要求过程录像。

## 仅当用户本轮点名

用来补项目级验证杠杆，不在探路中途自行生成：

- `references/pstack/create-verification-skill/SKILL.md`
- `references/pstack/maintain-verification-skill/SKILL.md`

## 交回

证据路径列表 + 未覆盖的验收句。不写「过」。

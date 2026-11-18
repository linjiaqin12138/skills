# 本目录里的 pstack 摘录

从 Cursor 插件 [pstack 0.15.43](https://github.com/cursor/plugins/tree/main/pstack)（MIT，见 `LICENSE`）拷来的 **SKILL 原文**，给 product-delivery 各角色 `Read`。不是再装一套插件，也不要整目录塞进子代理 brief。

上游正文保持原样。和本仓库闸门、`agent-rules.md` 冲突时，以 product-delivery 红线为准。

## 谁来启动

| 方式 | 含义 |
|---|---|
| **人启动** | 本轮用户用 `/` 挂上，或明确点名。指挥者可以提示，不得自己开跑 |
| **角色 Read** | 指挥者派到该角色后，子代理按角色文件把叶子 **Read 全文** 再干活。叶子默认不会被模型自己从目录拉起 |

人启动（贵、改 harness、或替人做品味）：

- `arena`、`interrogate`、`swarm`、`tdd`
- `create-verification-skill`、`maintain-verification-skill`

其余摘录由对应角色在被派到时 Read。

## 角色 → 叶子

路径相对本目录。角色文件在 `../../roles/<role>/SKILL.md`。

| 角色 | 被派到就 Read | 仅人启动才 Read |
|---|---|---|
| 指挥者 `product-delivery` | `in-your-own-words`、`principle-experience-first`、`principle-never-block-on-the-human`、`principle-encode-lessons-in-structure` | — |
| `pd-discover` | `how`、`why`；续作再加 `recall` | — |
| `pd-experience` | `principle-exhaust-the-design-space` | — |
| `pd-contract` | `architect`、`principle-boundary-discipline`、`principle-type-system-discipline` | — |
| `pd-design` | `architect`、`principle-foundational-thinking`、`principle-redesign-from-first-principles`、`principle-subtract-before-you-add` | `arena`、`interrogate` |
| `pd-implement` | `principle-laziness-protocol`、`principle-minimize-reader-load`、`principle-build-the-lever`、`principle-sequence-verifiable-units` | `tdd`、`swarm` |
| `pd-accept` | `principle-prove-it-works` | `create-verification-skill`、`maintain-verification-skill` |

冻结（flows 的 H）仍由指挥者做，不另派角色，也不读 pstack。

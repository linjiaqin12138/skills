# concept-writer 调度（clone-from-scratch-tutor）

概念页由独立 skill `concept-writer` 撰写（本 skill 的同级目录 `../concept-writer/`）。本文件只规定教程流程里何时调度、交什么、默认哪一档。页怎么写以那个 skill 为准。

## 默认模式

简单模式。prompt 里写明 `模式：简单`。

满足下面任一条才升级，并在 prompt 里写明模式：

- 用户明确要图文页 → 中等
- 用户明确要图文页并且要检查 → max
- 简单模式找不到能用的外部教程，且这个概念是项目专属 → 先问用户要不要升到中等

已有的重 HTML 概念页保持原格式。用户要求重写时，按用户指定的模式重写，不自行换格式。

## 启动

prompt 写明：先读 `../concept-writer/SKILL.md`，再只打开本次模式对应的文件，严格按其执行。

交接给齐：

- 模式
- 目标读者、触发问题、核心问题与内容边界
- 已核实的概念事实、公式、代码、素材路径
- 概念与项目的关系（简单模式的「回到项目」用；中等和 max 的页内不写项目）
- 输出路径：简单模式 `docs/concepts/<slug>.md`；中等和 max 为 `docs/concepts/<slug>/`
- 用户明确提出的学习目标或误解
- 两级偏好文件路径：用户级 `~/.clone-from-scratch-tutor/user-prefs.md`，项目级 `docs/user-prefs.md`

同一个概念文件只有一个作者。

## 验收

简单模式、中等模式：核对成稿在，简单模式还核对「去学」每条都有打开后的标题。不跑三层验收。

max 模式：核对返回里有技术验收、语义验收和冷读结论。缺一项退回 concept-writer，主 agent 不代写。

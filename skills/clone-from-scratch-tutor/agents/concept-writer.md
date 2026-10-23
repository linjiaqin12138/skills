# concept-writer 调度

概念页由同级 skill `concept-writer` 写。本文件只规定教程流程里何时调度、交什么、默认哪一档。页怎么写以那个 skill 为准。

先读 `../concept-writer/SKILL.md`，再只打开本次模式对应的文件。输出路径以本次交接为准，不用那个 skill 里的默认路径。

## 默认模式

简单模式。prompt 里写明 `模式：简单`。

满足任一条才升级，并在 prompt 里写明模式：

- 用户明确要图文页 → 中等
- 用户明确要图文页并且要检查 → max
- 简单模式找不到能用的外部教程，且这个概念是项目专属 → 先问用户要不要升到中等

已有页保持它的格式。用户要求重写时，按用户指定的模式重写。

## 交接

- 模式
- 目标读者、触发问题、核心问题、内容边界
- 已核实的概念事实、公式、代码、素材路径
- 概念与项目的关系。简单模式的「回到项目」用它。中等和 max 的页内不写项目
- 输出路径：
  - 简单：`book/concepts/<slug>.md`
  - 中等和 max：源稿 `book/concepts/<slug>.md`，HTML `book/concepts/<slug>.html`，素材 `book/concepts/figures/<slug>/`
- 用户明确提出的学习目标或误解
- 偏好路径：`~/.clone-from-scratch-tutor/user-prefs.md` 和 `tutor/user-prefs.md`

源稿文件名以交接路径为准。渲染命令与课文相同：优先项目里的 `bin/am-render`，否则用工作区 `vendor/answer-me-with-html/scripts/am.mjs`，风格 strict。不手改 HTML，不用 `am patch`。

同一个概念文件只有一个作者。旧布局项目（只有 `docs/STATE.md`）不改用上面的路径，继续写它原来的概念目录。

## 验收

简单、中等：核对成稿在。简单模式还核对「去学」每条都有打开后的标题。不跑三层验收。

max：核对返回里有技术验收、语义验收和冷读结论。缺一项退回，主 agent 不代写。

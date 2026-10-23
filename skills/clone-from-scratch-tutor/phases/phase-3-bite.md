# Phase 3 · 一个 Bite

一轮只交付一个 Bite：能独立验收的最小闭环，有自己的 `accept/` 脚本和预期结果。一个里程碑本身就是一个小闭环时，它只有一个 Bite。内容多才拆开，唯一标准是每个 Bite 独立可验收，并且一轮的信息量还能读完。经验上一个里程碑 1 到 4 个 Bite。

Bite 编号补零：`b01`、`b02`。目录是 `book/m01-<slug>/`。

## 动手前

主 agent 先写两样，再叫 coder：

1. `decisions/NNNN-<slug>.md`：本 Bite 的关键决定，模板是 `templates/decision.md`。来源填「手搓」，状态填「起草」。没有新决定就明确写「本 Bite 无新决定」。用户确认本 Bite 后改为「已采纳」。
2. `accept/mNN-bKK.sh`：可执行检查，从 `templates/accept.sh` 复制。脚本进入 `impl/`。预期行为写在脚本注释里。

coder 只改 `impl/`。验收脚本和决定要改时退回主 agent。

## 顺序

每步的 prompt 指示子代理先读对应文件，并给全该文件的「输入」。同时给出两个偏好路径。

1. **coder**：[agents/coder.md](../agents/coder.md)。
2. **verifier**：[agents/verifier.md](../agents/verifier.md)。实测只认它写入的 `tutor/runs/mNN-bKK.txt`。主 agent 不复跑脚本，核对结论和这份记录是否一致。不通过就回 coder。
3. **diagram**，架构有变动时：[agents/diagram.md](../agents/diagram.md)。更新 `book/arch/current.svg`，并把副本放到本课目录的 `arch.svg`。变动图放 `book/mNN-<slug>/figures/arch-diff.svg`。架构没变就不画，`current.svg` 保持原样。
4. **concept-writer**，本 Bite 有概念页要写或要改时：[agents/concept-writer.md](../agents/concept-writer.md)。在 writer 之前完成，背景卡片才链得到页。需求来自 Phase 2 标过的不熟概念、writer 退回的需求，或用户说看不懂。writer 退回的需求，主 agent 先用一句话定义问用户熟不熟，不熟才排。与 diagram 可以并行。同一个概念文件同时只有一个作者。
5. **概念页验收**：简单和中等只核对成稿与返回说明。max 核对返回里的技术验收、语义验收和冷读结论，缺一项退回 concept-writer。主 agent 不代写。写作类子代理用主模型。
6. **writer**：[agents/writer.md](../agents/writer.md)。它发现还缺概念页时只在返回里提需求，由主 agent 回到第 4 步。
7. **主 agent 收口**。看这三样：verifier 的结论和 `tutor/runs/` 里的记录一致；diagram 返回的边清单里，对不上代码的那几条（只打开对不上的位置）；浏览器打开本课 HTML，面板不是空的，图在，该动的动画在动，自测是 3 到 5 道选择题。决定的「放弃的成本」必须写出没选的那个方案的好处，写成实现注意事项的，改 `decisions/` 后让 writer 重新嵌入。

然后更新 `spec/features.md` 里本课交付的行和验收脚本链接，更新 `book/index.md` 和 README 的「现在读到」，更新 `tutor/STATE.md`。最后一个 Bite 通过后，复核本里程碑有没有做了清单外的功能，或漏了清单内的承诺。

向用户交付的摘要沿用课文的五节，各用几句话。停下，等用户确认。用户确认后，把本 Bite 里仍为「起草」的决定改为「已采纳」，再开始下一个 Bite。

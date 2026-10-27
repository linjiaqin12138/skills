# Phase 3 · 一个 Bite

一轮只交付一个 Bite：能独立验收的最小闭环，有自己的 `accept/` 脚本和预期结果。一个里程碑本身就是一个小闭环时，它只有一个 Bite。内容多才拆开，唯一标准是每个 Bite 独立可验收，并且一轮的信息量还能读完。经验上一个里程碑 1 到 4 个 Bite。

Bite 编号补零：`b01`、`b02`。目录是 `book/m01-<slug>/`。

## 先修门槛

先看 `tutor/user-prefs.md` 的先修能力表、`tutor/STATE.md` 和本 Bite 相关先修记录。路线为「先修学习」且尚未达到出口时，暂停本 Bite，按 [prerequisite.md](prerequisite.md) 进入家教；不写项目决定与验收，不启动 coder。用户明确跳过后标「用户选择跳过」，才可直接继续。已完成先修的，只取返回摘要中的已掌握能力与残余误区。

## 动手前

门槛通过后，主 agent 先写两样，再叫 coder：

1. `decisions/NNNN-<slug>.md`：仅当本 Bite 有**实现架构取舍**时才写，模板是 `templates/decision.md`。来源填「手搓」，状态填「起草」。流程、验收范围、课文章节取舍不进 `decisions/`，写在 `tutor/` 即可。没有架构决定就明确写「本 Bite 无架构决定」，不要为了有文件而硬写一篇。用户确认本 Bite 后，已有的手搓决定改为「已采纳」。
2. `accept/mNN-bKK.sh`：可执行检查，从 `templates/accept.sh` 复制。脚本进入 `impl/`。预期行为写在脚本注释里。

coder 只改 `impl/`。验收脚本和决定要改时退回主 agent。

## 顺序

每步的 prompt 指示子代理先读对应文件，并给全该文件的「输入」。同时给出两个偏好路径；coder 和验收执行者还要给出两个 `code-rules.md` 路径。

独立 verifier 的开关读 `~/.clone-from-scratch-tutor/user-prefs.md` 和 `tutor/user-prefs.md` 的「验收」一节。项目级覆盖用户级，未配置时默认开启。

1. **coder**：[agents/coder.md](../agents/coder.md)。
2. **行为与规范验收**：开启独立 verifier 时，启动 [agents/verifier.md](../agents/verifier.md)，主 agent 不复跑脚本；关闭时，主 agent 按同一份 verifier 规则执行。两种情况的实测和检查证据都只认 `tutor/runs/mNN-bKK.txt`，不通过就回 coder。关闭只省去独立子代理，不跳过验收脚本或编码规范检查。
3. **diagram**，架构有变动时：[agents/diagram.md](../agents/diagram.md)。更新 `book/arch/current.svg`，并把副本放到本课目录的 `arch.svg`。变动图放 `book/mNN-<slug>/figures/arch-diff.svg`。架构没变就不画，`current.svg` 保持原样。
4. **concept-writer**，本 Bite 有非阻塞概念页要写或要改时：[agents/concept-writer.md](../agents/concept-writer.md)。在 writer 之前完成，背景卡片才链得到页。需求来自 Phase 2 判为不阻塞实操的概念缺口或 writer 退回的需求。writer 退回时，主 agent 先区分课文问题与关键前置缺失；后者回到先修门槛，不用概念页代替先修。与 diagram 可以并行。同一个概念文件同时只有一个作者。
5. **概念页验收**：简单和中等只核对成稿与返回说明。max 核对返回里的技术验收、语义验收和冷读结论，缺一项退回 concept-writer。主 agent 不代写。写作类子代理用主模型。
6. **writer**：[agents/writer.md](../agents/writer.md)。它发现还缺概念页时只在返回里提需求，由主 agent 回到第 4 步。
7. **主 agent 收口**。看这四样：验收结论和 `tutor/runs/` 里的记录一致；diagram 返回的边清单里，对不上代码的那几条（只打开对不上的位置）；课文「验收」是否给出完整 raw 运行命令以及操作、观察、预期现象，而不是用 `accept/` 脚本代替项目运行；浏览器打开本课 HTML，面板不是空的，图在，该动的动画在动，自测是 1 到 5 道选择题。课文里若嵌入了架构决定，其「放弃的成本」必须写出没选方案的好处；写成实现注意事项的，改 `decisions/` 后让 writer 按 writer 规则重写那一节，不要整篇原文回贴。流程类决定本来就不该出现在 HTML 里。

然后更新 `spec/features.md` 里本课交付的行和验收脚本链接，更新 `book/index.md` 和 README 的「现在读到」，更新 `tutor/STATE.md`。最后一个 Bite 通过后，复核本里程碑有没有做了清单外的功能，或漏了清单内的承诺。

向用户交付的摘要按课文实际有的节各用几句话，不要为没有的节编一段。停下，等用户确认。用户确认后，把本 Bite 里仍为「起草」的架构决定改为「已采纳」，再开始下一个 Bite。

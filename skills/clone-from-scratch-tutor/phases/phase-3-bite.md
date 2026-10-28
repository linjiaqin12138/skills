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

独立 verifier 和知识依赖审计的开关都读 `~/.clone-from-scratch-tutor/user-prefs.md` 和 `tutor/user-prefs.md` 的「验收」一节。项目级覆盖用户级，未配置时均默认开启；关闭知识依赖审计不跳过行为验收。

1. **coder**：[agents/coder.md](../agents/coder.md)。
2. **行为与规范验收**：开启独立 verifier 时，启动 [agents/verifier.md](../agents/verifier.md)，主 agent 不复跑脚本；关闭时，主 agent 按同一份 verifier 规则执行。两种情况的实测和检查证据都只认 `tutor/runs/mNN-bKK.txt`，不通过就回 coder。关闭只省去独立子代理，不跳过验收脚本或编码规范检查。
3. **知识依赖审计**，仅开关开启时由主 agent 执行。对照本 Bite 最终代码 diff、此前课文里已讲解过的概念与 `book/concepts/` 已有页面、读者能力（`tutor/user-prefs.md` 的先修能力表），检查核心执行路径首次出现的机制、指令、数据结构与关键术语。逐项选择一种处置：本课给最小必要讲解（一两句约定，深入进概念页）；可安全黑盒的后置并登记；当前 Bite 无法独立理解的，回 Phase 2 拆分或调序后再写。黑盒项在 `tutor/STATE.md` 登记概念、首次代码位置、当前约定、承诺展开的 Milestone/Bite 和状态。所有核心首次概念都有前两种处置之一，才准交给 writer；没处置完不交 writer，概念页链接不算处置。交给 writer 的每项至少含概念名、首次代码位置、当前约定、承诺展开的 Milestone/Bite；当场讲解项另附正文要讲的最小内容（约定级，不是讲透）。本课到期的后置项一并交给 writer。关闭时整步跳过，不向 writer 索要或伪造审计产物。
4. **diagram**，架构有变动时：[agents/diagram.md](../agents/diagram.md)。更新 `book/arch/current.svg`，并把副本放到本课目录的 `arch.svg`。变动图放 `book/mNN-<slug>/figures/arch-diff.svg`。架构没变就不画，`current.svg` 保持原样。
5. **concept-writer**，本 Bite 有非阻塞概念页要写或要改时：[agents/concept-writer.md](../agents/concept-writer.md)。在 writer 之前完成，背景卡片才链得到页。需求来自 Phase 2 判为不阻塞实操的概念缺口、知识依赖审计或 writer 退回的需求。交接必须带上本次用点（代码位置）与依赖边界；概念页正文讲清这些点到够用，「去学」只给深入。概念页只作补充，不能替代正文最小解释。writer 退回时，主 agent 先区分课文问题与关键前置缺失；后者回到先修门槛，不用概念页代替先修。与 diagram 可以并行。同一个概念文件同时只有一个作者。
6. **概念页验收**：简单和中等核对成稿、返回说明，以及本次用点能否在页内（或所链概念页）对上解释。max 核对返回里的技术验收、语义验收和冷读结论，缺一项退回 concept-writer。主 agent 不代写。写作类子代理用主模型。
7. **writer**：[agents/writer.md](../agents/writer.md)。开启审计时交接当场讲解项、黑盒项和到期后置项；关闭时只交接「知识依赖审计：关闭」。它发现还缺概念页时只在返回里提需求，由主 agent 回到第 5 步。
8. **主 agent 收口**。看这四样：验收结论和 `tutor/runs/` 里的记录一致；diagram 返回的边清单里，对不上代码的那几条（只打开对不上的位置）；课文「验收」是否给出完整 raw 运行命令以及操作、观察、预期现象，而不是用 `accept/` 脚本代替项目运行；浏览器打开本课 HTML，面板不是空的，图在，该动的动画在动，自测是 1 到 5 道选择题。仅审计开启时再检查：核心首次概念均有正文最小解释或合规黑盒说明，到期后置项已展开并在 STATE 标为已闭环，其余后置项仍有明确回访位置。课文里若嵌入了架构决定，其「放弃的成本」必须写出没选方案的好处；写成实现注意事项的，改 `decisions/` 后让 writer 按 writer 规则重写那一节，不要整篇原文回贴。流程类决定本来就不该出现在 HTML 里。

然后更新 `spec/features.md` 里本课交付的行和验收脚本链接，更新 `book/index.md` 和 README 的「现在读到」，更新 `tutor/STATE.md`。最后一个 Bite 通过后，复核本里程碑有没有做了清单外的功能，或漏了清单内的承诺。

向用户交付的摘要按课文实际有的节各用几句话，不要为没有的节编一段。停下，等用户确认。用户确认后，把本 Bite 里仍为「起草」的架构决定改为「已采纳」，再开始下一个 Bite。

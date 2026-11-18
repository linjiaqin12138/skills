# M：薄变更

适用于：对外行为或协议局部变化，或改动面中等，仍是既有能力修补。

典型请求：对外 API 字段改名、新增多个可选字段并展示、把单步「自动跳过」改为「必须确认」。若牵动主路径或多份契约，则升 L。

## 步骤

1. **判档确认**：复述语义影响与改动面；用户点头。
2. **写 `brief.md`**（半页内）：目标、In/Out、验收句、关联契约/模块。放到 `changes/NNNN-slug/`（或仓库约定的变更包根）。
3. **可选交互补丁**：仅当主路径**人可感知操作**变了。有 GUI → 派 [`pd-experience`](../roles/experience/SKILL.md) 改一处 Demo 或补 `interaction.md`；无 GUI → 只补几行 `interaction.md` / ops，**不写 HTML Demo**。主路径操作没变 → 跳过 experience gate。
4. **现行契约/架构补丁**：改到的事件、API、配置写入 `contracts/current`；模块关系变了才改 `architecture/`。
5. **contract gate **（动协议时）：有未决拍板点才问一个字母。不请人读表。
6. **确认可写码** → 派 [`pd-implement`](../roles/implement/SKILL.md)（其内读 layered-codebase）。
7. **验收**：派 [`pd-accept`](../roles/accept/SKILL.md)。有界面则证据包（已有 verify 脚本优先，否则 agent-browser）；无界面则脚本输出。付费边界走 trace，真调用规则见 [`gates.md`](gates.md)。**人**说「过」。
8. **人说「过」之后冻结**：
   - CHANGELOG 一行；
   - 变更包 `brief` 状态 → **Frozen**（gate进度写验收通过）；
   - 本包推进的 CAP：`CAPABILITIES` **Next → Launch**（若开包时写的是 Next）；MVP 仍写「未实现」则改掉；
   - 若仓库有「Approve 后收尾」清单（如 `AGENTS.md`），按其执行。
9. **禁止**回写其它 Frozen 包正文；**禁止**无故改 `product/MVP`（纠正「未实现」标注除外）。

## brief 最小模板

见 [`doc-model.md`](doc-model.md) §5。

## 何时升 L

- 需要完整**有 GUI**交互 Demo 对齐主路径操作（Demo 只确认交互，不代替视觉规格；无 GUI 升 L 仍不写 HTML）
- 要 Grill 多轮才能定价值机制或扩展点
- 要分多领域 design 才能安全开写

升档后改走 [`flows-L.md`](flows-L.md)。

## 完成标准

- brief 已写给实现 agent，实现未超出 In/Out。人只确认过拍板字母（若有）。
- 现行契约及必要 `architecture/` 已更新。
- 有界面则探路证据已交，且人说「过」。付费调用未超过 3 次。
- CHANGELOG 已写；若开了变更包，brief 已 Frozen，关联 CAP 已 Launch（若适用）。
- 写码是否做完不在本清单。

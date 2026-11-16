# M：薄变更

适用于：对外行为或协议局部变化，或改动面中等，仍是既有能力修补。

典型请求：对外 API 字段改名、新增多个可选字段并展示、把单步「自动跳过」改为「必须确认」。若牵动主路径或多份契约，则升 L。

## 步骤

1. **判档确认**：复述语义影响与改动面；用户点头。
2. **写 `brief.md`**（半页内）：目标、In/Out、验收句、关联契约/模块。放到 `changes/NNNN-slug/`（或仓库约定的变更包根）。
3. **可选交互补丁**：仅当主路径操作变了——改一处 Demo 或补几行 `interaction.md`；否则跳过体验闸。
4. **现行契约/架构补丁**：改到的事件、API、配置写入 `contracts/current`；模块关系变了才改 `architecture/`。
5. **契约闸**（动协议时）：请用户确认表/草案。
6. **确认可写码** → 编码阶段：读并遵循 [`layered-codebase`](../../layered-codebase/SKILL.md)。
7. **轻量验收**：按 brief 验收句过一遍（真机或脚本）。
8. **用户 Approve（= 验收闸通过）后冻结**：
   - CHANGELOG 一行；
   - 变更包 `brief` 状态 → **Frozen**（闸门进度写验收通过）；
   - 本包推进的 CAP：`CAPABILITIES` **Next → Launch**（若开包时写的是 Next）；MVP 仍写「未实现」则改掉；
   - 若仓库有「Approve 后收尾」清单（如 `AGENTS.md`），按其执行。
9. **禁止**回写其它 Frozen 包正文；**禁止**无故改 `product/MVP`（纠正「未实现」标注除外）。

## brief 最小模板

见 [`doc-model.md`](doc-model.md) §5。

## 何时升 L

- 需要完整交互 Demo 对齐主路径操作（Demo 只确认交互，不代替视觉规格）
- 要 Grill 多轮才能定价值机制或扩展点
- 要分多领域 design 才能安全开写

升档后改走 [`flows-L.md`](flows-L.md)。

## 完成标准

- brief 已确认，实现未超出 In/Out。
- 现行契约及必要 `architecture/` 已更新。
- 已按 brief 做轻量验收并留下依据。
- CHANGELOG 已写；若开了变更包，brief 已 Frozen，关联 CAP 已 Launch（若适用）。
- 若改代码，满足 [`layered-codebase`](../../layered-codebase/SKILL.md) 的 DoD。

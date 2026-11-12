# L：完整澄清车道

适用于：新产品/MVP、新能力、改价值机制或产品级扩展点、主交互重做，或改动面大到需要 Demo + 分领域设计。

闸门细则见 [`gates.md`](gates.md)。拷问题库见 [`discovery.md`](discovery.md)。文档模板见 [`doc-model.md`](doc-model.md)。

## 阶段总览

| 阶段 | 主代理 | 可派子代理 | 停等确认 |
|---|---|---|---|
| A Grill | 提问、复述、记决策 | — | 意图闸 |
| B 范围落盘 | 写/改 product、brief；必要 architecture 意图 | explore 现状 | brief/产品范围 |
| C 交互与 Demo | 定操作清单与验收路径 | 做 Demo、画流程图 | 体验闸 |
| D 契约 | 对齐动作↔协议 | 起草 events/api/config | 契约闸 |
| E 设计 | Grill 待决、合并 design | 按需领域 design-* | design 确认 |
| F 编码 | 集成与歧义升级 | 实现切片 | （已确认可写码） |
| G 验收 | 主持演示脚本 | 勾选 verification | 验收闸 |
| H 冻结 | 标 Frozen、提升图、CHANGELOG | — | — |

## A — Grill 拷问

1. 读 `product/*`、`architecture/*`。
2. 按 [`discovery.md`](discovery.md) 一次一问；需要时给 2～3 方案。
3. 结论写入变更包 `design-decisions.md`（可先只有范围类结论）。
4. **意图闸**通过后再进入 B。

## B — 范围落盘

1. 更新或创建 `product/VISION`、`MVP`、`CAPABILITIES`（仅宏观变化时）。
2. 若模块清单有变，更新 `architecture/overview` 的意图描述（可先粗）。
3. 开 `changes/NNNN-slug/brief.md`。
4. 需要「现在代码怎样」时派 explore，主代理读报告，不自己沉进全库。
5. 用户确认 brief 范围后再进 C。

## C — 交互与 Demo

1. `ops-inventory.md`：阶段 × 动作 × 所见 × 系统反应。
2. `interaction.md`：流程/状态 Mermaid（可用 **`draw-diagram`**）。
3. 价值路径仍糊：可用 **`processize`** 先捋手搓步骤。
4. `prototypes/` 可点主路径 + README 演示步骤（低保真优先）。
5. **体验闸**：用户按 README 自己走通。

## D — 契约

1. 从 Demo 每个动作反推事件/API/配置行。
2. 草案可放变更包 `contracts/`；关键协作画 sequence。
3. **契约闸**通过后，再进 E（字段名与方向视为已对齐）。

## E — 分领域设计并汇总

1. 补全 Grill 待决（技术取舍）；更新 `design-decisions.md`。
2. 用户确认**无遗留异议**、同意拟派领域清单之前，不派 designer。
3. 按需并行派子代理写 `design-api.md` / `design-config.md` / `design-module.md` 等；禁止凑齐全派。
4. Mind-set：能简单则简单；扩展点写边界，不预埋空框架。
5. 主代理合并短 `design.md`（仅含本期领域）。
6. 用户确认 design → 才允许 F。

## F — 编码

见主 SKILL「编码阶段」：读并遵循 [`layered-codebase`](../../layered-codebase/SKILL.md)。  
只实现已确认范围；歧义停回闸门。大项目可按切片派实现子代理，主代理集成。

## G — 验收

1. 按 `plans/verification.md` 或 Demo README 在真系统勾选。
2. **验收闸**通过后再 H。

## H — 冻结

1. 变更包状态 → Frozen。
2. 同步 `contracts/current`（若草案有增量）。
3. 长期有效的模块图/时序 **提升**到 `architecture/`。
4. `product/*` 宏观若变了已在 B 处理；此处复核。
5. CHANGELOG。
6. 回复：如何验收、产物路径、未做项。

## 完成标准

见主 SKILL Definition of Done（L）。

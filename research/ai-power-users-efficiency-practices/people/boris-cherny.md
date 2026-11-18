# Boris Cherny

| 字段 | 内容 |
|---|---|
| 身份与领域 | Anthropic；Claude Code 创造者之一（与 Cat Wu 等） |
| 代表平台 | X 线程 / InfoQ 综述；Pragmatic Engineer 访谈；Training Data 播客；YouTube *Reflecting on a year of Claude Code* |
| 工具栈 | Claude Code（少定制）、并行本地 checkout + 远程 session、`CLAUDE.md`、`.claude/commands/` slash、PostToolUse format hook、Chrome 扩展验证、`/permissions`、routines/loops |
| 效率方法论 | Plan 迭代 → auto-accept 一枪实现；5 本地 + 5–10 远程并行；slash 固化 commit/PR；给验证环（测/浏览器） |
| 为何有效 | 计划质量决定一枪成功率；验证环据称提质 2–3×；团队共享 CLAUDE.md 沉淀错误 |
| 人机痛点对策 | 权限：几乎不用 skip-permissions，白名单常用命令；冲突：独立 git checkout；废弃会话 10–20%；CI 格式：hook 自动 format |
| RSI | PR 评论 `@.claude` 写回 CLAUDE.md；slash/routines 把重复劳动自动化；「写 loop 而不是写 prompt」 |
| 可迁移 | 团队 CLAUDE.md；slash 预计算 git 状态；验证环优先于更多 prompt |
| 可信度 | 高（一手实践）；吞吐量为自述；模型版本随时间变化 |

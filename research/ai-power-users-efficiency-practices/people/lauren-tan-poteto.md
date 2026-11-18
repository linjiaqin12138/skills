# Lauren Tan（土豆姐 / poteto）

| 字段 | 内容 |
|---|---|
| 身份与领域 | 工程师；React Compiler 核心成员背景；曾 Meta / Netflix；Cursor → SpaceXAI / Grok Bot（公开报道口径，见 references） |
| 代表平台 | X `@poteto`；Cursor Marketplace **pstack**；AI Engineer CODE 2026 演讲 *All Code Is Slop (Except Mine)* |
| 工具栈 | **Cursor**（主）、Agent Skills / playbooks、多模型子代理、浏览器/应用控制验证、Git/PR 流水线；公开包：`cursor/plugins` → pstack |
| 效率方法论 | `/setup-pstack` → `/poteto-mode` 路由 23 套 playbook → `/how` `/why` `/architect` `/arena` → 真应用验证 → 小 PR；原则名可中途转向 |
| 为何有效 | 把「严谨工程」编码成可路由流程；验证 skill 解除人当保姆；实现与审查分模型家族 |
| 人机痛点对策 | 幻觉/假完成 → prove-it-works；上下文 → `/recall` + 文件传上下文；review 成本 → 证据（截图/录屏）优先于散文 diff；一致性 → 原则文件 + arena；权限 → 按角色模型与只读分离 |
| RSI | **pstack 本身即飞轮**：会话纠错 → 原则/skill → 更少重复纠正；`/correct` 固化反复错误；公开插件持续精简 |
| 可迁移 | ① 验收条件写进提示 ② 验证不能只靠测试绿 ③ 多候选设计再综合 ④ 纠错升格为约束 |
| 可信度 | 高可复现（开源技能包 + 官方指南）；吞吐量数字为团队/本人自述，当量级信号而非对照实验 |

详见主报告 §6 RSI 专题。

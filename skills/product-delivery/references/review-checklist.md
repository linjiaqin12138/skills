# 流程与文档 Review 清单

用于检查「文档/流程是否乱」或变更包是否可冻结。不检查代码分层。

## 阻塞项（必须改）

- [ ] 用 Frozen 变更包或过期 requirements 描述**现行**产品行为
- [ ] S/M 小改回写了 Frozen 包正文
- [ ] gate未过已有业务实现（或「先跑一版」）
- [ ] 已确认 / 建议 / 待确认未区分，把推测写成确认
- [ ] 对外契约变了，`contracts/current`（或生成物）未同步
- [ ] 长期模块关系/关键时序只存在于变更包，未进 `architecture/`
- [ ] L 无操作对齐（有 GUI 无 Demo，或无 GUI 无 ops/interaction）却宣称交互已定
- [ ] **无 GUI 表面仍写了 / 派了 `prototypes/demo.html` 或整页可点 HTML**
- [ ] 交互 Demo 被当成视觉规格，或演示专用控件进了实现且未经单独立项
- [ ] 角色未钉（谁改配置 / 谁执行对外动作 / GUI / Agent）就写了长协议
- [ ] 契约含 `null` 键、无用 API、运行时快照进了配置，或为扩展预埋字段
- [ ] 指代不清时擅自选了常见默认（合并目标、skill 路径）
- [ ] 承诺交付 Agent SKILL，但权威路径未按仓库约定落地
- [ ] 还在产品阶段就按编码规范开写
- [ ] L 合并 design 后未派核对子代理，或带着未消化阻塞项请人开始写码
- [ ] 请人读 brief / 契约 / design / README，或一次抛出多条拍板
- [ ] 本期写了业务码却未派 `pd-codereview`，或仍有 layered-codebase 阻塞项却请人 Approve
- [ ] 有界面未做 agent-browser 探路（无界面时无脚本输出）却请人验收；或声称测试绿 / 分层无阻塞 / agent 探路即可过
- [ ] 付费验收没有 trace，或真调用不符合该仓库 `project-rules.md`
- [ ] 人说不过或打断后原样重试；重复纠正没有改 API / 校验 / lint / schema / skill
- [ ] 收尾未做 Frozen / CAP Launch，或没点明动了哪条验收点

## 建议项（可商量）

- [ ] `product/MVP` 过长，混进了实现细节 → 下沉到 architecture/变更包
- [ ] `architecture/flows` 图过多、不关键 → 删减
- [ ] 变更包内空目录/空文件 → 删除
- [ ] Grill 记录散落在聊天未写入 `design-decisions.md`
- [ ] CHANGELOG 缺失但用户可见行为已变
- [ ] 有 GUI 的 L 整页/重做 Demo、explore、契约起草或领域 design 本应默认派却堆在主会话
- [ ] 事事派子代理导致交接 noise（S/小补丁也派；无 GUI 硬派 experience 写网页）
- [ ] 主会话被整页 HTML/大 diff 污染，挤掉gate结论与下一问
- [ ] 契约偏长：重复、架构空谈或实现猜测还在；配置/快照、全局/工程目录、参数档混写
- [ ] 主代理犹豫，没按明文把**有 GUI 的** L 整页 Demo、定稿前 review、或写码后 `pd-codereview` 派出去

## 冻结前（L/M，用户 Approve / acceptance gate通过之后）

- [ ] 档位正确（语义 × 改动面）
- [ ] 对应gate均有明确确认（含acceptance gate = Approve）
- [ ] 人已说「过」；有业务码则 pd-codereview 无未消化阻塞；有界面则 agent-browser 探路证据已交；付费边界有 trace，真调用符合 `project-rules.md`；「仅演示」控件未进产品
- [ ] `brief` 状态改为 **Frozen**；gate进度已写验收通过（非 Implementing）
- [ ] 本包推进的 CAP：`CAPABILITIES` **Next → Launch**（若适用）；MVP「未实现」说明已改
- [ ] CHANGELOG 已写；文首可加关联日期
- [ ] 活文档路径在回复中列出（含 brief、CAP）

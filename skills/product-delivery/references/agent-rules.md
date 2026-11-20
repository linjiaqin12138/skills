# Agent 行为红线（product-delivery）

本文只约束需求澄清、文档寿命与 gate。编码分层细则不在这里（Approve 前派 `pd-codereview` 去读 layered-codebase）。

## 必须先问再做

- 判不准 S/M/L，或 explore 后改动面与口头不一致
- 改 `product/` 宪法或 `architecture/spine` 扩展点
- 把某次变更升/降档
- 新建顶层文档约定（改 `docs/` 根结构）且仓库尚未约定
- 删除或大幅改写未 Frozen 但用户已确认过的 Demo/契约（先确认）
- 指代不清：合并到哪条分支、skill 权威路径放哪。禁止擅自选更常见默认（例如并进 main、只写某一工具目录）
- 目标 Agent 宿主实际能力未知，却要设计监听 / 推送 / 进度通道

## 确定性禁止

- gate未过写业务实现，或「先跑一版再对齐」；环境/依赖准备可先做，但须用户知情且不得 scaffold 业务
- 把「嗯 / 看看 / 差不多」当成确认
- 回写 Frozen 变更包正文「保持同步」
- 推荐用户读历史变更包当现行产品说明
- Grill 一次抛长问卷
- 把能从官方文档 / 本仓库代码 / 现行契约查到的事实（如 provider 原生键名）当成对人拍板题
- 还在本 skill 的产品阶段就写业务实现
- 对外契约已变却不更新现行契约/生成物
- 长期行为图只留在 Frozen 包、不提升 `architecture/`
- 把交互 Demo 当视觉规格，或把控件清单拷进成品
- 把演示专用控件（顶栏已能导航仍加的跳转按钮等）默认带进产品
- **无 GUI 表面**（CLI/API/配置/Provider-only、本期不改可点页面）仍写或派 `prototypes/demo.html` / 整页可点 HTML
- Grill / experience gate不拿用户差异化句子回检，默认套竞品/旧品类模板（关卡式流程、缺产物锁死导航、照搬其按钮文案）
- 角色未钉就写长协议；或默认 Agent 经 HTTP 调本机服务、直连第三方、自己充当执行器报进度
- 先发明未经验证的通信开关 / 推送通道 / 「禁止某调用」类协议字段，再问宿主能不能做到
- 契约写 `null` 键、无用 API、把运行时快照（如 pid、端口）塞进配置文档、为以后扩展堆字段
- 共享对外形状变更只改一侧 Protocol（如只改 server 不改 web / shared），或在同一 version 内用双字段 / 双解析 / 无谓 `{ <params> }` 包装「软兼容」——对外保兼容应靠显式 version 分路，而非同版双轨
- 在契约、help、体验文案里声称未调查或未实现的 provider / model / 通道能力
- 用硬编码映射表或自由文本猜测表达本可落在可配置参数档（profile 等）上的能力分类
- 把某一项目的进程模型、热更新端口或模型档位写进本 skill。这类规则放目标仓库 `docs/conventions/project-rules.md`
- 主代理犹豫后把 **有 GUI 的** L 整页 Demo 留在主会话（明文：有 GUI 才默认派）
- 请人读 brief、契约、design 或 README
- 本期写了业务码却未派 `pd-codereview`、或仍有 layered-codebase 阻塞项，却请人 Approve / 说「待验收」
- 有界面未做 agent-browser 探路（或无界面无脚本输出）却说「待验收」；agent 探路后自行宣称通过；用贵模型或超过 3 次付费调用做验收
- 人打断或说不过之后原样重试
- 纠正只回答「记住了」，或只在 AGENTS.md / 注释里加一句

## 文档与措辞

- 区分：**已确认** / **建议** / **待确认**
- 不把推测写成用户已确认的事实
- 代码与活文档冲突：改活文档对齐代码，并告知用户
- CHANGELOG 只记用户可见或对外契约变化（见 [`doc-model.md`](doc-model.md)）
- 契约短。可丢：重复、废稿、架构空谈、实现猜测。不可丢：数据形状、错误码、HTTP/CLI/磁盘/WS/退出码
- 配置 vs 运行快照、全局配置 vs 工程目录、可复用参数档与单次覆盖等概念分开写
- 精简后担心丢信息：子代理只审「外部可感知缺口」，不靠把文档写长

## 角色与 pstack

- 派发 kickoff 第一行：`Read <product-delivery 根绝对路径>/roles/<role>/SKILL.md`。角色自己再 Read 其文件里点名的 `references/pstack/` 叶子。
- 角色不得宣布 gate 通过，不得请人读散文，不得把测试绿当成验收。
- 用户没点名时，不得 Read 并执行 `arena`、`interrogate`、`swarm`、`tdd`、`create-verification-skill`、`maintain-verification-skill`。
- 体验角色不录屏、不用 agent-browser 点低保真 Demo。分层 review 角色只报阻塞/建议，不宣布可以 Approve。验收角色交证据包，报告里不写通过。

## 编排

- 主会话只留决策与指针；长产出（**有 GUI 的**整页 Demo、explore 报告、契约起草、领域 design、**design 定稿前对照代码+文档 review**、实现切片、**写码后分层 codereview**）在 **L 默认派**子代理（例外见 [`flows-L.md`](flows-L.md)「编排细则」）
- 「可派」不得读成爱派不派；有 GUI 时 L 整页 Demo 堆主会话算偏离；**无 GUI 仍派 HTML Demo 也算偏离**；写了业务码却跳过 `pd-codereview` 也算偏离
- 不为「显得规范」事事派（交接本身变 noise）；S/改动清晰时主代理独扛，M 仅在需要时派 explore 或单领域起草，小补丁不硬派；无 GUI 不硬派 experience 写网页；**但 M/L 写码后的分层 review 不是「可跳过的规范秀」**
- 子代理 prompt 必须含：目标、约束、路径、已确认结论、禁止事项、交回格式（短报告 + 路径）；派 experience 时写明有/无 GUI；派 codereview 时写明 layered-codebase 根与 diff 范围
- 不依赖子代理再套娃派子代理；由主会话按回传再派
- 对用户回复 Demo 结果时给路径与怎么点，不粘贴整页 HTML；无 GUI 给命令/操作句
- 派 HTML Demo 的 prompt 写明：交互确认，非视觉源；「仅演示」控件单列；**无 GUI 根本不派 HTML**
- L：合并 `design.md` 后、请用户开始写码前，**必须**派核对子代理；无半页例外
- L/M：写码后、请人 Approve 前，**必须**派 `pd-codereview`；有阻塞先修再复审，再派 `pd-accept`
- 人说不过、打断、或同一纠正第二次：指挥者开会。跨项目机制改本 skill；项目禁令改该仓库 lint / schema / API。不写纪要，不原样重试
- 验收：先 `roles/codereview`（有码时），无阻塞再派 `roles/accept`。有界面优先已有 verify/control 脚本，否则 agent-browser 点通并交证据。最终过/不过必须是人。付费边界默认 trace
- 低保真体验 Demo（含「仅演示」模拟按钮）给人路径即可；体验闸例外要视频时，Demo 须对齐成品 UI + HTML 动画，直接出动画页视频，不派 agent 点网页

## 交付回复最低要求

- 有界面：探路证据指针（截图/settle/命令/generation）。无界面：脚本命令与输出。付费边界：trace 路径；若真调用，说明符合该仓库 `project-rules.md` 的哪一条
- 有业务码：分层 review「无阻塞项」指针（或阻塞已清的复审结论）
- 一行：`brief Frozen，CAP-x Launch`（若适用）和「本次动了哪条验收点」
- 每个已回字母一行：`Dn=字母 仍成立，因为…`
- 测试绿不能代替人拍板；分层 review 无阻塞不能代替人拍板；agent 探路不能代替人拍板。写码证据不在本 skill 的交付格式里
- L 进入编码前：核对报告已消化，用户已说开始写码
- 产品承诺交付 Agent SKILL：回复里给出权威路径（及可选镜像）

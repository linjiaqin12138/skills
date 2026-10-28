---
name: clone-from-scratch-tutor
description: 把一个开源仓库做成可断点续作的从零实现教程。当用户给出仓库地址，并要求手搓、从零实现、按里程碑学习作者的设计时使用。只想要代码导读、跑起来或一次架构介绍时不用。
compatibility: 需要 git、Node 20+、可探测到的 answer-me-with-html 渲染命令，以及同级 skill draw-diagram 与 concept-writer。架构图默认 fireworks-tech-graph。页面目检需要 agent-browser 或 cairosvg。多轮先修学习默认用 learn-coach 加终端实验协议；没有时需安装或由用户改选。manim 仅在用户批准视频后，用本 skill 的 scripts/setup-manim.sh 安装。
---

# 从零手搓开源项目教程

把开源项目做成一套从零实现的教程。复刻优先级：架构设计、关键权衡、可运行的最小复刻。能下载的数据和服务不重写。

## 何时不用

用户只要代码导读、把项目跑起来，或一次架构介绍。不要开教程仓库。

## 启动

1. 用户级偏好：`~/.clone-from-scratch-tutor/user-prefs.md`。没有则从本 skill 的 `templates/user-prefs.md` 复制，按其中的首次提问清单问（答多少算多少）。之后用户纠正背景或说出新偏好，立即写入该文件；**仅本项目成立**的约定（如 docker 构建 / 宿主机跑 qemu、验收要不要写清容器边界）写入项目级 `tutor/user-prefs.md`，不要塞进 skill 硬规则或用户级通用偏好。
2. 编码规则：用户级 `~/.clone-from-scratch-tutor/code-rules.md` 和项目级 `tutor/code-rules.md` 都是可选文件，存在才读；规则冲突时项目级覆盖用户级。
3. 找工作区，三选一：
   - 已有 `<项目名>-from-scratch/tutor/STATE.md`：读 `tutor/STATE.md`、`tutor/user-prefs.md`、可能存在的 `tutor/code-rules.md`、`spec/features.md`、`spec/deviations.md`。只打开当前阶段那一份文件，从「下一步」继续。不重做已完成的课，不重问已记下的偏好。
   - 已有 `docs/STATE.md`、且没有 `tutor/STATE.md`：这是旧布局项目。不迁移，不新建 `impl/`、`book/`、`spec/`、`decisions/`。沿用它现有的目录，从那份 STATE 继续。
   - 都没有：向用户要仓库地址。然后在当前目录创建 `<项目名>-from-scratch/`，按 [phases/phase-0.md](phases/phase-0.md) 从 `templates/` 落盘。

## 不变式

1. 每轮只交付一个停顿单位。Phase 0、1、2、4 停在阶段末；Phase 3 停在一个 Bite；先修学习停在一个动作或问题。用户说继续才进入下一轮。
2. 读不到的内容标【合理推断】。禁止编造文件内容、链接、命令输出。
3. 终态范围以用户批准过的 `spec/features.md` 为准。标了复刻的项必须交付。豁免写在清单里并经用户批准。
4. 偏差写入 `spec/deviations.md` 并编号。功能缺失是待收敛的偏差。架构层面的永久偏离写明理由后留在偏差簿。豁免只写在功能清单。
5. 依赖没找到，或验收没过，就停下说明。不交付未验证的产物。
6. 用户选择先修学习且未达到出口时，对应项目 Bite 不启动 coder。短补可同会话，多轮先修默认独立会话；每轮写盘，不靠 auto compact 记进度。
7. 先修路线依据、范围裁剪、会话状态与 audit 只进 `tutor/`，不进入 `book/` 或 README 课文。writer 只接收已掌握能力与残余误区。
8. 知识依赖审计默认开启，可在用户级和项目级偏好文件的「验收」一节关闭，项目级覆盖用户级；后置概念只登记在 `tutor/STATE.md`，不进 `decisions/`。审计与闭环规则见 [phases/phase-3-bite.md](phases/phase-3-bite.md)。

## 项目目录

新项目只用这一套。正式代码在 `impl/`，一棵树连续演进，不按课复制。学习者练习在 `exercises/`，不计入正式实现。`upstream/` 是钉死在某一个 commit 的 submodule，只读。

```text
<项目名>-from-scratch/
├── README.md
├── impl/                          # 手搓实现，内部按原项目的语言惯例
├── upstream/                      # 上游 submodule
├── book/
│   ├── index.md
│   ├── arch/
│   │   ├── upstream.svg           # 原项目模块图
│   │   ├── current.svg            # impl 的当前架构，路径固定
│   │   └── src/
│   ├── concepts/<slug>.md         # 要 HTML 时旁边放同名 .html
│   └── m01-<slug>/
│       ├── b01-<slug>.md          # 课文源稿
│       ├── b01-<slug>.html
│       ├── arch.svg               # 本课验收时的 current 副本
│       └── figures/
├── decisions/NNNN-<slug>.md
├── spec/features.md
├── spec/deviations.md
├── accept/m01-b01.sh              # 自动验收用；不替代课文里的 raw 项目运行命令
├── exercises/<slug>/              # 先修与项目内引导的学习者练习
└── tutor/
    ├── STATE.md
    ├── user-prefs.md
    ├── code-rules.md              # 可选的项目编码规则
    ├── phases/
    ├── prerequisites/<slug>.md    # 内部调度、实时交接与 audit
    └── runs/m01-b01.txt
```

序号补零。人从 `README.md` 进 `book/index.md`。agent 从 `tutor/STATE.md` 续作。

| 路径 | 谁写 |
|---|---|
| `impl/` | coder |
| `accept/` | 主 agent，在 coder 之前 |
| `tutor/runs/` | verifier；独立 verifier 关闭时由主 agent |
| `book/arch/`、课目录里的 `arch.svg` 与 `figures/arch-diff.svg` | diagram |
| `book/mNN-*/bKK-*.md` 与同名 `.html` | writer |
| `book/concepts/` | concept-writer |
| `exercises/` | 主 agent 搭脚手架与验收，学习者完成核心练习 |
| `decisions/`、`spec/`、`tutor/`、`book/index.md`、README 的进度行 | 主 agent；家教会话可更新当前先修记录与 STATE |

格式从 `templates/` 复制。不要凭记忆另写一种 STATE、清单、偏差簿或决定。

## 读哪份文件

| 当前阶段 | 打开 |
|---|---|
| Phase 0 | [phases/phase-0.md](phases/phase-0.md) |
| Phase 1 | [phases/phase-1.md](phases/phase-1.md) |
| Phase 2 | [phases/phase-2.md](phases/phase-2.md) |
| Phase 3 | [phases/phase-3-bite.md](phases/phase-3-bite.md) |
| 先修学习 / 家教会话 | [phases/prerequisite.md](phases/prerequisite.md) |
| Phase 4 | [phases/phase-4.md](phases/phase-4.md) |

子代理没有上下文。启动时让它先读下表对应文件，交接按该文件的「输入」逐项给全，缺一项不开工。主文件不复述字段。每个子代理的 prompt 都写上用户级偏好和 `tutor/user-prefs.md` 的路径；coder 和验收执行者还要写上两个 `code-rules.md` 的路径。

| 角色 | 打开 |
|---|---|
| coder | [agents/coder.md](agents/coder.md) |
| verifier | [agents/verifier.md](agents/verifier.md) |
| diagram | [agents/diagram.md](agents/diagram.md) |
| writer | [agents/writer.md](agents/writer.md) |
| concept-writer | [agents/concept-writer.md](agents/concept-writer.md) |

概念页怎么写，读同级 skill [concept-writer](../concept-writer/SKILL.md)。架构图怎么画，读同级 skill [draw-diagram](../draw-diagram/SKILL.md)。

## 环境

按顺序探测，找到就用，不写死家目录里的某一个路径：

- `node`，主版本 ≥ 20
- 渲染：`<项目>/bin/am-render`，否则工作区 `vendor/answer-me-with-html/scripts/am.mjs`。风格用 strict。警告不清零就不出页
- 架构图：`draw-diagram`。默认 fireworks-tech-graph。d2 只在该 skill 允许的简单图上作退路
- 目检：`agent-browser`，否则 `cairosvg`
- manim：用户批准了视频才运行本 skill 的 `scripts/setup-manim.sh`。脚本注释里的系统包需要 sudo 时，停下让用户执行

缺任何一个当前步骤要用的命令，列出缺什么并停下。

## 提问之后

先区分课文没讲清与关键前置缺失。课文问题就改对应的 `book/mNN-*/bKK-*.md`，再整页重渲；概念材料进 `book/concepts/`。架构决定的后果改 `decisions/` 里那一篇；只有该决定已经在课文里时，才按 [agents/writer.md](agents/writer.md) 重写那一节。流程/验收类决定不进课文。不在课文里另开问答面板。

课文节是按需的，不是清单。没有实现架构取舍就不嵌决定原文；没有设计可领读就不写代码领读；没有新概念、偏差、踩坑就不写背景、差异、坑。禁止为凑面板硬塞。

关键前置缺失时，不默认继续补长课文。更新 `tutor/user-prefs.md` 的能力评估，让用户选择项目内引导、先修学习或直接继续；选先修则按 [phases/prerequisite.md](phases/prerequisite.md) 暂停相关 Bite。用户说跳步了，退回上一 Bite 补。原项目设计有问题就照实写。每个阶段结束时，列出本阶段尚未从代码确认的推断。

## 主 agent 的话

中文，直接。呈现次序按用户级偏好里的「认知负荷」一节。

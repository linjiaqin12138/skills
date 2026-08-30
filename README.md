# Skills 探索仓库

一个用来探索市面上各类 Agent Skill 和项目点子的个人工作仓库。

平时会在这里让 Agent 调研网上有没有可用的 Skill，把调研结果整理总结下来；遇到感兴趣的就下载一两个回来实际体验一下，偶尔也会顺手产出一些市场调研和小 demo。

## 目录结构

| 目录 | 内容 |
|------|------|
| `skills/` | 收集的 Skill 源文件（目前主要是 The Minimalist Entrepreneur 系列，源自 [slavingia/skills](https://github.com/slavingia/skills)） |
| `.claude/skills/` | 实际启用挂载的 Skill，供 Claude Code 直接使用 |
| `vendor/` | 下载回来体验的第三方项目源码（如 agent-browser、redditlens） |
| `bin/` | 编译/安装好的可执行文件（vendor 项目的产物） |
| `deliverables/` | Agent 产出的调研报告：市场调研、Skill 盘点、技术选型等 |
| `research/` | 深度调研项目（多章节、带完整产出物的大型调研） |
| `demo/` | 随手做的小 demo |

## 使用方式

- **调研 Skill / 点子**：直接让 Agent 联网调研，产出报告存入 `deliverables/` 或 `research/`
- **体验 Skill**：下载源码到 `vendor/`，编译产物放 `bin/`，可用的 Skill 挂载到 `.claude/skills/`
- **做 demo**：放到 `demo/` 下

## 现有 Skill 一览

源自《The Minimalist Entrepreneur》一书的 10 个 Skill：`find-community`、`validate-idea`、`mvp`、`processize`、`first-customers`、`pricing`、`marketing-plan`、`grow-sustainably`、`company-values`、`minimalist-review`。

另有体验中的第三方 Skill：`agent-browser`（浏览器自动化）、`redditlens`（Reddit 调研）。

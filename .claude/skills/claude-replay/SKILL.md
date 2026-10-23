---
name: claude-replay
description: Turn AI coding agent session logs into a single self-contained, interactive HTML replay page — playable like a video, shareable, embeddable. Use when the user wants to show, share, publish, demo, debug, or review an agent session; wants to see what an AI actually did step by step; asks for a "回放", "会话记录展示", "把操作历史做成网页", "session replay", "agent transcript visualization", or wants a visual timeline of tool calls, thinking blocks, and file edits. Also covers converting Claude Code / Cursor / Codex CLI / Gemini CLI / OpenCode / Kimi Code transcripts into HTML, and live-monitoring a running session. Skip for plain-text transcripts or when the user just wants a log dump.
allowed-tools: Bash(bin/claude-replay:*), Bash(claude-replay:*), Bash(npx claude-replay:*)
---

# claude-replay：把 agent 操作历史做成可播放的网页

**本地安装（这个项目）：** CLI 已经 vendored 在 `vendor/claude-replay/`，不要用全局安装。统一调用项目根目录的 `bin/claude-replay`。下面的 `claude-replay ...` 都指 `bin/claude-replay ...`。

上游：https://github.com/es617/claude-replay ｜ MIT ｜ 完整文档见 `vendor/claude-replay/README.md`，收录说明见 `vendor/claude-replay/VENDOR.md`

## 什么时候用

用户想把一次 AI 编程会话"展示出来"——做成网页、发给别人、写进文档、做演示、或者排查 agent 到底干了什么。产出一个 HTML 文件即可。

## 快速开始

```bash
# 1. 列出本机 Kimi Code 会话（Kimi Code 的日志在 ~/.kimi-code/sessions/<项目>/<session>/agents/<name>/wire.jsonl）
bin/claude-replay --list

# 2. 生成回放页（输出到 replays/）
bin/claude-replay <session-id> --title "标题"

# 3. 打开
xdg-open replays/*.html
```

也支持 Claude Code、Cursor、Codex CLI、Gemini CLI、OpenCode 的日志，直接给文件路径即可：

```bash
bin/claude-replay ~/.claude/projects/<project>/<session-id>.jsonl -o replay.html
```

## 常用参数（透传给上游）

| 参数 | 用途 |
|------|------|
| `--theme NAME` | 主题：`tokyo-night`（默认）、`monokai`、`solarized-dark`、`github-light`、`dracula`、`bubbles`；`--list-themes` 可列出 |
| `--title TEXT` | 页面标题（分享到群里时显示） |
| `--description TEXT` | 链接预览的描述 |
| `--speed N` | 初始倍速（默认 1.0） |
| `--turns N-M` | 只保留第 N 到 M 轮 |
| `--exclude-turns N,N` | 排除指定轮次 |
| `--from / --to TIMESTAMP` | 按时间范围截取 |
| `--mark "N:Label"` | 加书签/章节（可重复） |
| `--no-thinking` | 默认折叠 thinking 块 |
| `--no-tool-calls` | 默认折叠工具调用 |
| `--redact "text"` | 额外脱敏指定字符串（可重复） |
| `--no-auto-redact` | 关闭自动密钥脱敏（仅限内部使用） |
| `--timing paced` | 无时间戳时用合成节奏，演示效果好 |
| `--serve --watch` | 起本地服务实时监视会话变化 |
| `--open` | 生成后直接用默认浏览器打开 |

不带任何参数运行 `bin/claude-replay` 会启动 **Web 编辑器**：浏览本地会话、增删轮次、编辑提示词、实时预览、导出 HTML。适合需要精挑内容的场合。

## 给用户交付时的注意

1. **生成后先自己检查一遍再交付**——HTML 是自包含单文件，内容不可再改。
2. **脱敏是尽力而为**：默认会替换 API key、JWT、连接串、私钥等常见模式，但**不会过滤代码内容和文件路径**。转录数据以压缩 blob 嵌在 HTML 里，删掉某轮只是隐藏渲染，原始数据仍可被提取。公开分享前提醒用户自行过一遍。
3. **私有代码不要外发**：回放页会包含源码片段、内部路径、工具输入输出。
4. 想让内容嵌进文档/博客，用 iframe：`<iframe src="replay.html" width="100%" height="600"></iframe>`。

## 目录约定

- `vendor/claude-replay/` — 上游源码（vendored 0.11.0，无本地补丁）
- `bin/claude-replay` — 项目本地 wrapper，含 Kimi Code 会话查找
- `replays/` — 生成的 HTML 回放页存放处

# 视频总结 Skill 调研

> 调研日期：2026-08-30
> 需求：给一个视频链接（如小红书视频），自动总结视频内容

## 背景

本项目现有的 12 个 skill（pricing、mvp、marketing-plan 等）均为商业/产品方向，没有视频处理类 skill。以下为网上已有的现成方案。

## 方案一：video-to-subtitle-summary-skill（开源，最贴合需求）

- 仓库：https://github.com/imlewc/video-to-subtitle-summary-skill （MIT 协议）
- 定位：Claude Code / Codex 的本地 skill，明确支持**小红书、抖音、B 站、YouTube** 链接及本地视频/音频文件
- 核心流程：解析链接 → 下载视频 → 生成字幕 → AI 总结
- 字幕转写后端（二选一）：
  - `faster-whisper`（默认）：本地转写，零 API 费用，消耗本地 CPU/GPU
  - 火山引擎 VC：云端转写，需配置 `BYTEDANCE_VC_TOKEN` / `BYTEDANCE_VC_APPID`
- 链接解析：
  - YouTube：用 `yt-dlp` 直接抓人工/自动字幕，免费，无需代理
  - 小红书/抖音/B 站：依赖第三方付费解析代理（AI Douyin 或 TikHub），新用户有免费额度，成功解析一次扣 1 积分
- 输出：视频信息表、AI 标题、摘要、核心要点、SRT 字幕、纯文本
- 安装：`git clone` 后复制到 `~/.claude/skills/`，配置 `.env`（`ASR_BACKEND`、`AI_DOUYIN_API_KEY` 等）
- 依赖：FFmpeg、yt-dlp、Python 3.9+、faster-whisper

## 方案二：bibigpt-skill（商业服务，省心稳定）

- 官网介绍：https://bibigpt.co/zh/blog/posts/xiaohongshu-ai-skill-video-summary-agent-guide
- 定位：BibiGPT 出的 CLI skill，调用其云端 AI 视频总结能力，支持小红书、B 站、YouTube、抖音、TikTok、播客等 **30+ 平台**
- 安装：`npx skills add JimmyLv/bibigpt-skill`，需配合 BibiGPT 桌面端/账号
- 优势：
  - 平台解析由官方维护，稳定，不怕小红书反爬变动
  - 支持批量总结后交叉分析（例：总结 10 个测评视频的正负反馈，输出决策报告）
  - 声称已生成 500 万+ 摘要，服务 100 万+ 用户
- 劣势：按额度付费，数据走第三方云端

## 方案三：Summarize CLI skill（通用型，偏海外平台）

- 介绍：https://mcpmarket.com/tools/skills/summarize-cli-integration
- 定位：把 `summarize` CLI 包成 skill，支持 URL、PDF、音频、YouTube 视频
- 可切换 Gemini / OpenAI / Anthropic 等模型后端
- 对小红书等国内平台支持不明确，主要面向海外平台

## 其他参考

- [Audio Transcription + Summarization Skill](https://heyclau.de/entry/skills/audio-transcription-summarization)：Whisper + ffmpeg 转写音频并总结，偏会议/播客场景
- [whisper-transcribe](https://github.com/spillwavesolutions/whisper-transcribe)：Claude Code skill，用 Whisper 转写本地音视频文件（mp3/wav/m4a/mp4），不含在线视频下载
- [NVIDIA video-search-and-summarization](https://github.com/NVIDIA-AI-Blueprints/video-search-and-summarization)：企业级视频理解微服务 blueprint，过重，不适合个人使用

## 对比与结论

| 维度 | video-to-subtitle-summary-skill | bibigpt-skill | Summarize CLI |
|------|------|------|------|
| 小红书支持 | ✅（经第三方代理解析） | ✅ 官方维护 | ❓ 不明确 |
| 费用 | 转写免费；小红书解析扣积分（有免费额度） | 按额度付费 | 取决于所选模型 API |
| 部署 | 本地自托管 | 云端服务 | 本地 CLI |
| 稳定性 | 解析代理可能失效 | 高 | 中 |
| 离线/隐私 | 好（转写本地） | 差（上云） | 中 |

**结论：**

- 想免费、自托管：选方案一。faster-whisper 本地转写不花钱；小红书链接解析靠第三方代理免费额度，长期高频使用有积分成本
- 想省心稳定、有批量分析需求：选方案二（bibigpt-skill），花钱换稳定
- 另有自研思路：当前环境模型支持直接读视频文件，可自写轻量 skill——`yt-dlp` 下载小红书视频后让模型直接看视频总结，不经过语音转写，能覆盖画面信息（字幕贴纸、演示动作），依赖最少；缺点是 token 消耗高于纯文本转写方案

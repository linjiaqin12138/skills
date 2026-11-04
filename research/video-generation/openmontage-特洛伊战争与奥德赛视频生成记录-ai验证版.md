# OpenMontage 生成「特洛伊战争与奥德赛」AI 验证版记录

> 记录日期：2026-09-05  
> 项目位置：`/home/jqlin/Project/OpenMontage`（相对于当前 skills 仓库的 `../OpenMontage`）  
> 项目目录：`/home/jqlin/Project/OpenMontage/projects/trojan-odyssey-ai`  
> 最终输出：`/home/jqlin/Project/OpenMontage/projects/trojan-odyssey-ai/renders/trojan-odyssey-ai-1x-subtitled.mp4`

---

## 1. 任务目标

在上一版「免费 PPT 风格」视频的基础上，验证 **AI 生成视觉** 能否显著提升质量，同时保持**零成本**：

- 同一主题：特洛伊战争与奥德赛
- 中文旁白与字幕
- 视觉升级为 AI 生成插画（非 Pillow 手绘）
- 旁白改为男声、正常语速（1x）
- 走完整的 OpenMontage `animated-explainer` pipeline
- 尽量使用免费工具与本地资源

---

## 2. 最终产出

| 项目 | 内容 |
|------|------|
| 视频文件 | `projects/trojan-odyssey-ai/renders/trojan-odyssey-ai-1x-subtitled.mp4` |
| 时长 | 107.17 秒（约 1 分 47 秒） |
| 分辨率 | 1920×1080 |
| 帧率 | 30 fps |
| 文件大小 | 约 15.8 MB |
| 音轨 | 1 条旁白（edge-tts 男声）+ 1 条背景音乐 |
| 字幕 | 中文字幕已烧录进画面 |
| 视觉风格 | 古希腊黑绘陶瓶画风格：扁平剪影人物、羊皮纸底色、赤陶红与墨色 |
| 场景数 | 12 个（7 个 AI 生成图 + 5 个 Remotion/FFmpeg 动画） |
| 总成本 | **$0** |

---

## 3. 与上一版对比

| 项目 | 上一版（免费 PPT 版） | 本版（AI 验证版） |
|------|----------------------|-------------------|
| 视觉 | 2 张 Pillow 静态图 | **12 张 AI 生成场景图** |
| 旁白音色 | XiaoxiaoNeural（女声） | **YunjianNeural（男声，播音员风格）** |
| 语速 | -20%（慢） | **+0%（1x 正常）** |
| 时长 | 235.75 秒 | **107.17 秒** |
| 字幕 | 单字拆分（已修复） | **整句显示** |
| Pipeline | 直接脚本 `generate.py` | **完整 `animated-explainer` pipeline** |
| 成本 | $0 | **$0** |

---

## 4. 用到的核心工具与依赖

| 工具 / 库 | 用途 | 费用 |
|-----------|------|------|
| `edge-tts` (Microsoft Edge 在线 TTS) | 生成中文男声旁白（`zh-CN-YunjianNeural`） | 免费 |
| `Pollinations.ai` | 免费文生图 API，生成 12 张场景插画 | 免费 |
| `pixabay_music` (OpenMontage 内置) | 下载免版税背景音乐 | 免费 |
| `ffmpeg` | 图像转视频片段、音频混合、字幕烧录、最终编码 | 免费/本地 |
| `OpenMontage` 框架 | Pipeline 管理、Artifact 校验、Checkpoint 跟踪 | 本地 |

> **注意**：原计划使用 `fal.ai`（FLUX + ElevenLabs），但因账户被锁定（`TOP_UP`）而全部替换为免费方案。

---

## 5. Pipeline 执行记录

本次严格遵循 OpenMontage 的 `animated-explainer` pipeline：

| Stage | 产出 | 关键决策 |
|-------|------|----------|
| `research` | `research_brief.json` | 发现诺兰《奥德赛》2026 年 7 月上映是热点；中文语境缺乏 AI 生成视觉内容 |
| `proposal` | `proposal_packet.json` | 用户选择 **Concept 2**（视觉化重述）；确认 `render_runtime=remotion` |
| `script` | `script.json` | 469 字、9 节、journey 结构；含 voice_performance 和 delivery_cues |
| `scene_plan` | `scene_plan.json` | 12 场景；7 generated + 5 animation；3 个 hero moment |
| `assets` | `asset_manifest.json` | 9 段旁白 + 12 张图 + 1 首音乐；总成本 $0 |
| `edit` | `edit_decisions.json` | 时间轴按实际旁白时长重新分配 |
| `compose` | `render_report.json` | **Remotion 崩溃，降级 FFmpeg** |
| `publish` | `publish_log.json` + `exports/` | SEO 元数据 + 章节标记 + 导出包 |

---

## 6. 生成脚本与工具

### 6.1 自定义工具（新增）

为接入免费方案，新增两个 OpenMontage 工具：

**`tools/graphics/pollinations_image.py`**
- 调用 Pollinations.ai 免费 API
- 无需 API Key，无需注册
- 支持 seed、尺寸、模型参数
- 成本估算固定为 $0

**`tools/audio/edge_tts_tool.py`**
- 封装 edge-tts 为 OpenMontage BaseTool
- 支持 voice、rate 参数
- 内置常用中文 voice 映射（xiaoxiao/yunxi/yunjian 等）
- 成本估算固定为 $0

### 6.2 资产生成脚本

**`projects/trojan-odyssey-ai/generate_assets.py`**
- 批量调用 `edge_tts_tool` 生成 9 段旁白
- 批量调用 `pollinations_image` 生成 12 张场景图
- 调用 `pixabay_music` 下载背景音乐
- 输出 `asset_manifest.json`

**`projects/trojan-odyssey-ai/prepare_compose.py`**
- 把 `edit_decisions` 中的 asset IDs 替换为实际文件路径
- 生成 phrase-level captions（按中文标点分句）
- 添加 `themeConfig` 和 `wordSeparator=""`
- 输出 `composition_data.json`

**`projects/trojan-odyssey-ai/render_ffmpeg_1x.py`**
- FFmpeg 降级渲染脚本
- 把每张图转为带 zoompan Ken Burns 效果的视频片段
- 拼接片段、混合旁白和音乐、烧录 SRT 字幕

---

## 7. 遇到的问题与解决

| 问题 | 现象 | 解决方式 |
|------|------|----------|
| **fal.ai 账户锁定** | 所有 fal.ai 工具返回 `403 User is locked. Reason: TOP_UP` | 改用 Pollinations.ai + edge-tts 全免费方案 |
| **Pollinations 500 错误** | scene-4 首次生成失败 | 添加重试逻辑 + 简化 fallback prompt |
| **Remotion 渲染崩溃** | `Compositor error: No frame found at position 0` | **根因：Pollinations 返回 JPEG 但保存为 `.png`**；修复扩展名后仍崩溃，最终降级 FFmpeg |
| **旁白时长变化** | 1x 语速比 2x 长一倍（53.8s → 107.1s） | 重新计算 `edit_decisions` 时间轴，12 场景重新分配 |
| **字幕格式** | transcriber（whisperx）不可用 | 按 script sections 生成 phrase-level SRT 字幕 |

---

## 8. 关键发现

### 8.1 关于 fal.ai 免费额度

- fal.ai 新注册账户**不会自动获得免费额度**，或额度已过期
- 错误信息明确：`User is locked. Reason: TOP_UP`
- **必须先充值才能使用任何模型**（包括 FLUX、ElevenLabs、视频生成）
- 建议：如要用 fal.ai，先充值 $5–10 激活账户

### 8.2 关于 Pollinations.ai

- 完全免费，无需注册，无需 API Key
- 图像质量中等，适合风格化插画（陶瓶画、剪影、扁平风）
- 不适合写实人脸或高精度细节
- 返回格式为 JPEG，需注意文件扩展名匹配

### 8.3 关于 edge-tts 男声

- `zh-CN-YunjianNeural`（云健）：播音员风格，低沉有力，适合史诗叙事
- `zh-CN-YunxiNeural`（云希）：温和男声，适合日常讲解
- `rate="+0%"` 为正常语速，`"+100%"` 为 2x 速

### 8.4 关于 Remotion 崩溃

- 表面错误：`Compositor error: No frame found at position 0`
- 根因：`Img` 组件加载图像失败（格式/扩展名不匹配）
- 深层问题：Remotion 的 Rust compositor 在图像加载失败后会触发线程崩溃（`clone3` 错误）
- 即使修复图像格式，本机 Remotion 仍不稳定；FFmpeg 是可靠 fallback

---

## 9. 生成内容的目录结构

```text
/home/jqlin/Project/OpenMontage/projects/trojan-odyssey-ai/
├── generate_assets.py        # 批量生成资产脚本
├── prepare_compose.py        # 准备 Remotion composition 数据
├── regenerate_narration.py   # 重新生成 1x 语速旁白
├── render_ffmpeg.py          # FFmpeg 渲染脚本（2x 版）
├── render_ffmpeg_1x.py       # FFmpeg 渲染脚本（1x 版）
├── update_edit_decisions.py  # 更新 edit_decisions 时间轴
├── project.json              # 项目元数据
├── artifacts/
│   ├── research_brief.json
│   ├── proposal_packet.json
│   ├── script.json
│   ├── scene_plan.json
│   ├── asset_manifest.json
│   ├── edit_decisions.json
│   ├── composition_data.json
│   ├── render_report.json
│   └── publish_log.json
├── assets/
│   ├── narration/            # s1.mp3 ~ s9.mp3 + narration_full.mp3
│   ├── images/               # scene-1.jpg ~ scene-12.jpg
│   ├── music/                # background_music.mp3
│   └── subtitles.srt         # 字幕文件
├── renders/
│   ├── trojan-odyssey-ai-subtitled.mp4      # 2x 语速版
│   ├── trojan-odyssey-ai-1x-subtitled.mp4   # 1x 语速版（最终）
│   ├── review_frames/                       # 2x 版审阅帧
│   └── review_frames_1x/                    # 1x 版审阅帧
└── exports/                                 # 发布包
    ├── video/output.mp4
    └── metadata/
        ├── metadata.json
        ├── chapters.txt
        ├── description.txt
        └── tags.txt
```

---

## 10. 从头到尾复现步骤

### 步骤 1：准备环境

```bash
cd /home/jqlin/Project/OpenMontage
source .venv/bin/activate
pip install edge-tts
```

### 步骤 2：初始化项目

```bash
python -c "from lib.checkpoint import init_project; init_project('trojan-odyssey-ai', title='特洛伊战争与奥德赛 AI 验证版', pipeline_type='animated-explainer')"
```

### 步骤 3：按 pipeline 执行各 stage

```bash
# 1. research（写 research_brief.json）
# 2. proposal（写 proposal_packet.json，需用户批准）
# 3. script（写 script.json，需用户批准）
# 4. scene_plan（写 scene_plan.json，需用户批准）
# 5. assets（运行 generate_assets.py，需用户批准）
# 6. edit（写 edit_decisions.json）
# 7. compose（运行 render_ffmpeg_1x.py）
# 8. publish（运行 export_bundle）
```

### 步骤 4：检查输出

```bash
ls -lh projects/trojan-odyssey-ai/renders/trojan-odyssey-ai-1x-subtitled.mp4
ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 \
  projects/trojan-odyssey-ai/renders/trojan-odyssey-ai-1x-subtitled.mp4
```

---

## 11. 后续可改进点

1. **充值 fal.ai 解锁高质量生成**：FLUX + ElevenLabs + MiniMax 视频，质量会显著提升
2. **修复 Remotion 渲染**：可能是本机 Node.js/Chrome 环境问题，值得进一步排查
3. **增加 AI 视频片段**：用 Pollinations 视频 API 或免费额度生成 2-3 段动态镜头
4. **优化字幕样式**：当前是整句显示，可升级为词级高亮（需修复 transcriber 或手动生成 WordCaption）
5. **加长内容**：当前 107 秒偏短，可扩充到 3-5 分钟，增加更多场景和细节
6. **提交自定义工具到 OpenMontage**：`pollinations_image` 和 `edge_tts_tool` 对零预算用户有通用价值

---

## 12. 关键文件速查

| 文件 | 作用 |
|------|------|
| `/home/jqlin/Project/OpenMontage/projects/trojan-odyssey-ai/renders/trojan-odyssey-ai-1x-subtitled.mp4` | 最终视频（1x 语速） |
| `/home/jqlin/Project/OpenMontage/projects/trojan-odyssey-ai/artifacts/edit_decisions.json` | 剪辑决策（含时间轴、字幕、音频配置） |
| `/home/jqlin/Project/OpenMontage/projects/trojan-odyssey-ai/generate_assets.py` | 资产批量生成脚本 |
| `/home/jqlin/Project/OpenMontage/tools/graphics/pollinations_image.py` | 自定义 Pollinations 图像工具 |
| `/home/jqlin/Project/OpenMontage/tools/audio/edge_tts_tool.py` | 自定义 edge-tts 工具 |
| `/home/jqlin/Project/OpenMontage/projects/trojan-odyssey-ai/render_ffmpeg_1x.py` | FFmpeg 渲染脚本 |

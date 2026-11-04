# OpenMontage 生成「特洛伊战争与奥德赛」视频记录

> 记录日期：2026-09-05  
> 项目位置：`/home/jqlin/Project/OpenMontage`（相对于当前 skills 仓库的 `../OpenMontage`）  
> 生成脚本：`/home/jqlin/Project/OpenMontage/projects/trojan-odyssey/generate.py`  
> 最终输出：`/home/jqlin/Project/OpenMontage/projects/trojan-odyssey/renders/trojan-odyssey.mp4`

---

## 1. 任务目标

用 OpenMontage 的免费路径生成一条约 5 分钟的中文讲解视频，主题为**特洛伊战争与奥德赛**，要求：

- 中文旁白与字幕
- 视觉「简笔化」、风格连贯一致
- 讲述两场史诗的过程
- 尽量使用免费工具与本地资源

---

## 2. 最终产出

| 项目 | 内容 |
|------|------|
| 视频文件 | `projects/trojan-odyssey/renders/trojan-odyssey.mp4` |
| 时长 | 235.75 秒（约 3 分 56 秒） |
| 分辨率 | 1920×1080 |
| 帧率 | 30 fps |
| 文件大小 | 约 33 MB |
| 音轨 | 1 条旁白 + 1 条背景音乐（循环） |
| 字幕 | 中文字幕已烧录进画面 |
| 视觉风格 | 羊皮纸底色 + 墨色线条 + 赤陶红点缀的简笔/线稿风格 |

---

## 3. 用到的核心工具与依赖

| 工具 / 库 | 用途 | 费用 |
|-----------|------|------|
| `edge-tts` (Microsoft Edge 在线 TTS) | 生成中文旁白 | 免费 |
| `Pillow` | 本地绘制简笔时间轴、奥德赛路线图 | 免费/本地 |
| `pixabay_music` (OpenMontage 内置) | 下载免版税背景音乐 | 免费 |
| `Remotion` (React + Node) | 按时间轴合成画面、字幕、转场 | 免费/本地渲染 |
| `ffmpeg` | 音频拼接、最终编码封装 | 免费/本地 |
| `OpenMontage` 框架 | 项目管理、Artifact 校验、VideoCompose 渲染调度 | 本地 |

---

## 4. 生成脚本的关键设计

脚本 `projects/trojan-odyssey/generate.py` 自己完成了整条管线：

1. **初始化项目**：调用 `lib.checkpoint.init_project`。
2. **分段生成旁白**：把全文拆成 10 个 Section，每段调用 `edge_tts.Communicate(text, "zh-CN-XiaoxiaoNeural", rate="-20%")` 生成 MP3，再用 `ffmpeg concat` 拼成完整音轨。
3. **绘制视觉素材**：用 Pillow 生成两张 1920×1080 的线稿图：
   - `assets/images/timeline_trojan.png`（特洛伊战争时间线）
   - `assets/images/odyssey_map.png`（奥德修斯归途路线图）
4. **生成字幕时间轴**：按每段音频时长按字数比例分配每句字幕的 `startMs` / `endMs`。
5. **构建 Asset Manifest / Edit Decisions / Proposal Packet**：按 OpenMontage 的 artifact schema 输出 JSON。
6. **调用 `VideoCompose` 渲染**：最终通过 Remotion 本地渲染出 MP4。

主题配色（`THEME_CONFIG`）：

```python
{
    "backgroundColor": "#F7F2E8",   # 羊皮纸底色
    "surfaceColor": "#FFFFFF",
    "primaryColor": "#2C2C2C",      # 墨色
    "accentColor": "#A85C48",       # 赤陶红
    "textColor": "#2C2C2C",
    "captionBackgroundColor": "rgba(247, 242, 232, 0.88)",
    "captionHighlightColor": "#A85C48",
    "headingFont": "'Noto Serif CJK SC', 'Noto Sans CJK SC', serif",
    "bodyFont": "'Noto Sans CJK SC', ...",
}
```

---

## 5. 实际执行的命令

### 5.1 克隆与更新（按用户要求 `../` 位置）

```bash
# 已在 ../OpenMontage 存在并更新到最新
cd /home/jqlin/Project/OpenMontage
git pull
```

### 5.2 进入虚拟环境并设置代理

```bash
cd /home/jqlin/Project/OpenMontage
source .venv/bin/activate
export HTTP_PROXY=http://127.0.0.1:7890
export HTTPS_PROXY=http://127.0.0.1:7890
```

> `edge-tts` 和 `pixabay_music` 都需要外网，按用户要求统一走本地 `127.0.0.1:7890` 代理。

### 5.3 运行生成脚本

```bash
cd /home/jqlin/Project/OpenMontage
python projects/trojan-odyssey/generate.py
```

脚本会依次打印：

```text
Initializing project...
Generating narration with edge-tts...
Generating line-art diagrams...
Building captions...
Building asset manifest...
Building edit decisions...
Validating artifacts...
Writing artifacts...
Rendering video via Remotion...
Render complete: .../trojan-odyssey.mp4
Final duration: 235.75s
```

### 5.4 验证输出（可选）

```bash
ffprobe -v error -show_entries format=duration,size,bit_rate \
  -show_entries stream=width,height,avg_frame_rate \
  -of default=noprint_wrappers=1 \
  /home/jqlin/Project/OpenMontage/projects/trojan-odyssey/renders/trojan-odyssey.mp4
```

输出：

```text
width=1920
height=1080
avg_frame_rate=30/1
duration=235.754667
size=34101913
bit_rate=1157200
```

---

## 6. 对 OpenMontage 源码的改动

为了让这条中文、简笔风格的视频跑通，修改了两处源码（均未提交）：

### 6.1 `remotion-composer/src/Explainer.tsx`

**问题**：Remotion 默认按空格分词显示字幕，中文没有空格会被切成单字，影响阅读。

**改动**：在 `ExplainerProps` 增加 `wordSeparator?: string`，并传给 `CaptionOverlay`。

```diff
 export interface ExplainerProps {
   ...
   captions?: WordCaption[];
   audio?: AudioConfig;
+  wordSeparator?: string;
 }

 export const Explainer: React.FC<ExplainerProps> = (props) => {
-  const { cuts, overlays, captions, audio } = props;
+  const { cuts, overlays, captions, audio, wordSeparator } = props;
   ...
       <CaptionOverlay
         ...
+        wordSeparator={wordSeparator}
       />
```

生成脚本在 `composition_data` 里传 `"wordSeparator": ""`，让 Remotion 按整句/整词渲染中文。

### 6.2 `tools/video/video_compose.py`

**问题**：`_stage_remotion_media` 只会把 `source`、`src`、`backgroundSrc` 指向的媒体复制到 Remotion 的 `public` 目录，但 `cut` 里用的 `backgroundImage` 没被识别，导致带背景图的 cut 渲染时找不到文件。

**改动**：把 `backgroundImage` 和 `backgroundVideo` 也加入 `media_keys`。

```diff
-        media_keys = {"source", "src", "backgroundSrc"}
+        media_keys = {"source", "src", "backgroundSrc", "backgroundImage", "backgroundVideo"}
```

---

## 7. 遇到的问题与解决

| 问题 | 现象 | 解决方式 |
|------|------|----------|
| 中文字幕被拆成单字 | Remotion CaptionOverlay 默认按空格切词 | 给 `Explainer` 增加 `wordSeparator` 属性，传空字符串 |
| `backgroundImage` 丢失 | 带时间轴/路线图背景的 cut 渲染报找不到资源 | 在 `video_compose.py` 的 `media_keys` 里加入 `backgroundImage` |
| `pixabay_music` 下载不稳定 | 首次/第二次出现 SSL 握手超时 | 脚本里用 `try/except` 包裹，失败则跳过；后续重试成功，最终使用了 110 秒的背景音乐 |
| 前两次 Remotion 渲染失败 | `events.jsonl` 显示 `video_compose` 返回 `success: false` | 排查后第三次渲染成功，总耗时约 21 分钟 |
| 视频时长不足 5 分钟 | 旁白总时长 234.67 秒，已比目标短 | 当前版本约 3 分 56 秒；如需 5 分钟可进一步降速 TTS 或补充脚本 |

---

## 8. 生成内容的目录结构

```text
/home/jqlin/Project/OpenMontage/projects/trojan-odyssey/
├── generate.py                 # 主生成脚本
├── project.json                # 项目元数据
├── events.jsonl                # 工具执行日志
├── artifacts/
│   ├── proposal_packet.json    # 概念/预算/排期包
│   ├── asset_manifest.json     # 资源清单
│   └── edit_decisions.json     # 剪辑决策（含字幕、主题、音频配置）
├── assets/
│   ├── audio/
│   │   ├── narration-s1.mp3 ~ narration-s10.mp3
│   │   ├── narration_full.mp3
│   │   └── concat.txt
│   ├── images/
│   │   ├── timeline_trojan.png
│   │   └── odyssey_map.png
│   └── music/
│       └── background_music.mp3
└── renders/
    ├── trojan-odyssey.mp4
    └── .final_review_frames/
        └── review_frame_*.png
```

---

## 9. 从头到尾复现步骤

### 步骤 1：准备环境

```bash
# 1. 克隆 OpenMontage（如尚未克隆）
cd /home/jqlin/Project
git clone https://github.com/OpenMontage/OpenMontage.git

# 2. 更新到最新
cd OpenMontage
git pull

# 3. 进入 Python 虚拟环境
source .venv/bin/activate

# 4. 确保 edge-tts 已安装
pip install edge-tts

# 5. 设置代理（必须，否则 edge-tts / Pixabay 可能超时）
export HTTP_PROXY=http://127.0.0.1:7890
export HTTPS_PROXY=http://127.0.0.1:7890
```

### 步骤 2：应用必要的源码补丁

**补丁 A：中文分词**

编辑 `remotion-composer/src/Explainer.tsx`：

1. 在 `ExplainerProps` 接口里加 `wordSeparator?: string;`
2. 在 `Explainer` 组件解构处加 `wordSeparator`
3. 在 `<CaptionOverlay ... />` 处加 `wordSeparator={wordSeparator}`

**补丁 B：背景图 staging**

编辑 `tools/video/video_compose.py`，找到 `_stage_remotion_media` 里的：

```python
media_keys = {"source", "src", "backgroundSrc"}
```

改为：

```python
media_keys = {"source", "src", "backgroundSrc", "backgroundImage", "backgroundVideo"}
```

### 步骤 3：放置生成脚本

将本记录同名的 `generate.py` 放入：

```text
/home/jqlin/Project/OpenMontage/projects/trojan-odyssey/generate.py
```

### 步骤 4：运行

```bash
cd /home/jqlin/Project/OpenMontage
python projects/trojan-odyssey/generate.py
```

### 步骤 5：检查输出

```bash
ls -lh projects/trojan-odyssey/renders/trojan-odyssey.mp4
ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 \
  projects/trojan-odyssey/renders/trojan-odyssey.mp4
```

---

## 10. 后续可改进点

1. **加长到 5 分钟**：把 `edge_tts.Communicate(..., rate="-20%")` 改成 `-25%` 或 `-30%`，或补充更多讲解内容。
2. **背景音乐音量**：当前音乐音量仅 0.06，若觉得太轻，可在 `build_edit_decisions` 里调高 `audio_cfg["music"]["volume"]`。
3. **更多视觉素材**：目前只有两张 Pillow 图；可按 Section 生成更多示意图（如帕里斯裁决对比图、木马剖面图、奥德修斯遇到的怪物图标等）。
4. **提交上游补丁**：`wordSeparator` 与 `backgroundImage` 的改动对中文/非英文内容有通用价值，可考虑整理成 PR 提交回 OpenMontage。

---

## 11. 关键文件速查

| 文件 | 作用 |
|------|------|
| `/home/jqlin/Project/OpenMontage/projects/trojan-odyssey/generate.py` | 完整生成脚本 |
| `/home/jqlin/Project/OpenMontage/projects/trojan-odyssey/renders/trojan-odyssey.mp4` | 最终视频 |
| `/home/jqlin/Project/OpenMontage/projects/trojan-odyssey/artifacts/edit_decisions.json` | Remotion 渲染用的剪辑决策 |
| `/home/jqlin/Project/OpenMontage/remotion-composer/src/Explainer.tsx` | 被修改的 Remotion 组件 |
| `/home/jqlin/Project/OpenMontage/tools/video/video_compose.py` | 被修改的 staging 逻辑 |

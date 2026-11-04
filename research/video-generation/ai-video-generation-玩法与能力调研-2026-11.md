# AI 视频生成：玩法图谱、能力强度与配音技术调研

> 调研日期：2026-11-04  
> 方法：以网上官方文档、厂商新闻稿、第三方榜单为主；本地既有复盘仅作交叉验证。  
> 结论时效：视频模型榜单变化极快，Elo / 定价以抓取当日为准。

---

## 一句话结论

1. **玩法已从「文生一段短片」扩成四层**：通用生成 → 参考/控制 → 垂直编辑（换人/换嘴/动作迁移）→ 配音与数字人。  
2. **能力上**，2026 下半年「原生音画」是标配；单段时长主流仍是 **8–15 秒**，头部产品（Seedance 2.5）已到 **单次 30 秒 + 多轮延长**。  
3. **公开偏好榜**（Artificial Analysis T2V+Audio，抓取当日）：Wan 3.0 ≈ Utopai X ≈ MiniMax H3 领先；可灵/Veo 更吃「可控/企业」而不是纯 Elo。  
4. **AI 配音不是一个模型**：TTS/翻译配音（ElevenLabs）与口型同步（Sync Labs / MuseTalk / VideoRetalk）经常拆开，再与视频生成拼工作流。  
5. **能商用短内容，难一次出电影**：复杂物理、多主体交互、像素级保留原片、长片一致性仍是硬缺口。

---

## 1. 玩法有多少种？——按「输入控制什么」分类

阿里云百炼官方把视频能力拆成「通用生成 + 垂直场景」，结构最清晰，可作行业词典。[1]

### 1.1 通用生成（从零造片）

| 玩法 | 输入 | 典型产品 | 成熟度（2026-11） |
|------|------|----------|-------------------|
| 文生视频 T2V | 文本 | Wan / Seedance / Kling / H3 / Veo / Gen-4.5 | ★★★★★ 主战场 |
| 图生视频 I2V | 首帧图 + 文本 | 同上几乎全覆盖 | ★★★★★ |
| 首尾帧 FL2V | 起止两图 | H3、Seedance、Kling、Wan | ★★★★ 过渡镜头常用 |
| 参考生视频 R2V / Omni | 图/视频/音频多模态参考 | Seedance 2.5、Kling Omni、MiniMax H3 Ref2VA | ★★★★ 商业短片核心 |
| 视频延长 Extend | 已有片段末尾续写 | Veo Scene Extension、Seedance 多轮延长 | ★★★★ |
| 多镜头一次出 | 分镜提示 / storyboard | Kling 3.0 多镜头、Seedance 宫格分镜 | ★★★★ |

Seedance 2.5 官方：单次最多 **30 图 + 10 视频 + 10 音频（合计 ≤50）**，并强调白模参考、运动参考、绿幕/时间戳编辑。[2][3]  
MiniMax H3 Ref2VA：最多 **9 图 + 3 视频 + 3 音频（合计 ≤12）**，输出最高 **2K / 15s**，原生立体声音频。[4]  
可灵 3.0：最长 **15s**；原生多语言音频；Omni 可从参考视频抽视觉+音色；多镜头故事板可指定景别/运镜。[5]

### 1.2 控制类（指定「怎么动」）

| 玩法 | 本质 | 代表 | 能力边界 |
|------|------|------|----------|
| **动作控制 / Motion Control** | 角色图 + 动作参考视频 → 新角色演同一动作 | 可灵 Motion Control 3.0、Kling on Runway | 舞蹈/武术/手势强；非人肢体（猫爪）不如专用替换模型稳 |
| **图生动作 Image-to-Action** | 静态人像吃参考动作，背景跟图 | 通义 Wan 图生动作 | 背景固定在参考图侧 |
| **运镜/关键帧控制** | 文本运镜、关键帧、相机路径 | Veo Motion controls、Runway 提示词运镜、Seedance 白模 | 文字运镜已可用；旧 Motion Brush（Gen-2）已退役 |
| **角色一致性锁 ID** | 多参考图锁同一人/物 | Kling Elements、Runway References、H3 多图参考 | 跨镜头可用；服饰细徽标仍会漂 |

### 1.3 垂直编辑（改已有视频）

| 玩法 | 保留什么 | 改什么 | 代表 |
|------|----------|--------|------|
| **人物替换 / Character Swap** | 原背景、运镜、节奏 | 主体外观 | Wan 视频换人、Wan2.2-Animate Mix、Viggle |
| **视频换脸 Face Swap** | 身体与场景 | 脸 | 各家垂直工具 |
| **视频指令编辑** | 大部分镜头 | 增删改主体/风格/运镜 | Seedance 时间戳编辑、Runway Aleph 2.0 |
| **风格转绘 / V2V** | 构图运动骨架 | 画风材质 | Wan 视频编辑、Aleph |
| **局部换嘴 Lip Replace** | 镜头与身体 | 嘴部纹理 | VideoRetalk、MuseTalk、Wav2Lip |
| **数字人 Talking Head** | — | 整张头像说话 | Wan 数字人、Hedra、ElevenLabs Lip Sync、LivePortrait |

**关键区分（易混）：**

- **动作迁移**：新场景可换，重在「动作像」。  
- **人物替换**：场景尽量不动，重在「镜头像原片」。  
- **通用 R2V**：两者都会「重新生成」，不保证像素级保留。[本地报告交叉验证]

### 1.4 配音与本地化（音轨侧玩法）

| 玩法 | 做什么 | 是否改嘴型 | 代表 |
|------|--------|------------|------|
| TTS 旁白 | 文本 → 语音 | 否 | ElevenLabs、IndexTTS、edge-tts |
| **AI Dubbing 翻译配音** | 多语种重配 + 情绪/音色迁移 | **ElevenLabs 官方明确：Dubbing 不含 lip sync** [6][7] | ElevenLabs Dubbing v2（90+ 语言） |
| **配音 + 口型同步** | 先配音再改嘴 | 是 | Sync Labs `dubParams`（ElevenLabs + lipsync-2）[8] |
| 原生音画同出 | 生成时直接出对白/环境音 | 模型内置 | Kling / Seedance / H3 / Veo / Wan 3.0 |
| 多角色说话映射 | 同一镜头两人各自口型 | 部分模型 | Kling Omni 多角色对话 [5] |

---

## 2. 能力有多强？——用公开榜 + 产品规格衡量

### 2.1 Artificial Analysis Video Arena（T2V + Audio）

来源：[AA Text-to-Video Leaderboard](https://artificialanalysis.ai/video/leaderboard/text-to-video)（抓取 2026-11-04）[9]

| 大致位次 | 模型 | Elo（约） | 列表价（约） | 备注 |
|---------|------|-----------|--------------|------|
| 1 | **Wan 3.0** | 1172 | $12/min | 当前 with-audio 榜首 |
| 2 | Utopai X | 1169 | 无公开 API | — |
| 3 | **MiniMax H3 768p** | 1160 | $4.80/min | **开源权重榜领先** |
| 4–5 | FLUX 3 Max / Seedance 2.5 | ~1155 | Seedance 标价很高 | 质量强、成本敏感 |
| 6–7 | HeyGen Video 1 / H3 Max | ~1150 | 偏低–中 | — |
| ~10 | Gemini Omni Flash 1.1 | 1129 | ~$9/min | Google 默认推荐方向之一 |
| ~18–19 | Kling 3.0 Omni / 3.0 Pro | 1022 / 1000(锚点) | ~$8–10/min | Elo 中游，**控制力产品强** |
| ~22 | Veo 3.1 | 961 | $24/min | 企业栈 + 4K；纯偏好未必第一 |

解读：

- **「好看」≠「好用」**：可灵/Veo 的卖点是多镜头、动作控制、合规与工作流，不是 Arena 第一。  
- **性价比故事**：H3 开源/API 在前列，适合本地或低成本量产（与本地 Odyssey-H3 复盘一致）。  
- **榜单锚点**：Kling 3.0 1080p 被设为 Elo=1000 参考点，跨代分数不可硬比旧文。

### 2.2 各家「硬规格」速览

| 维度 | 头部现状 | 仍弱的地方 |
|------|----------|------------|
| 单段时长 | 主流 8–15s；Seedance 2.5 **30s** + 延长到数分钟叙事 [2] | 一次稳定出 2–5 分钟电影级仍靠拼接 |
| 分辨率 | 1080p 普及；Kling/Veo/Seedance 部分档位宣称原生 4K | 很多「4K」实为放大 |
| 原生音频 | 已成标配（对白+环境音） | 多语种口型、方言、多角色说话质量参差 |
| 物理真实感 | 大幅度改善（衣褶、水、碰撞） | Seedance 官方自承：复杂物理、极多主体交互仍不足 [2] |
| 可控性 | R2V / 动作控制 / 分镜 / 白模 | 像素级「只改人不动景」仍要专用替换模型 |
| 一致性 | 多参考锁人可用 | 长片服饰/logo/手指数仍会崩 |

### 2.3 第三方「能力分层」观点（需打折）

《The Real State of AI Video in 2026》把 Seedance / Kling Omni / Veo 放在 S 档，并给出「脚本 LLM → ElevenLabs → 角色静帧 → Kling/Runway → CapCut」工作流；单片人力仍约 **8–12 小时**。[10]  
可灵官方博客也列了当前局限：复杂精确动作靠文字不够，需要 Motion Control；长叙事需多镜头工具。[11]

---

## 3. AI 配音相关技术（单独拆开看）

### 3.1 三条技术路线

```
A. 原生音画模型
   文本/参考 → 视频+音频一次出
   优点：口型天然对齐；缺点：后期改台词难、音色库弱

B. 后期配音管线
   成片/脚本 → TTS/Dubbing →（可选）Lip Sync → 混音
   优点：多语言、可编辑；缺点：链路长

C. 数字人/换嘴
   人像或已有视频 + 音频 → 说话视频
   优点：新闻播报/客服/本地化；缺点：多为半身或局部脸
```

### 3.2 关键产品能力边界（官方口径）

| 产品 | 职责 | 官方明确边界 |
|------|------|----------------|
| **ElevenLabs Dubbing v2** | 90+ 语言翻译配音、声线克隆、节奏对齐 | **不含 lip sync**；口型要另找模型 [6][7] |
| **Sync Labs** | lipsync-2 + 可选 `dubParams` 调 ElevenLabs | 一键「翻译配音 + 改嘴」[8] |
| **通义 VideoRetalk** | 已有视频换嘴，最长接近 120s | 只改唇形区域 [1] |
| **通义数字人 / LivePortrait** | 图+音频说话；长片可用 LivePortrait | 背景常固定 [1] |
| **可灵原生音频** | 生成时对白+多角色 | 改语种更像重生成，不像后期配音 [5] |

### 3.3 开源/本地常见栈

| 模块 | 常见选型 | 用途 |
|------|----------|------|
| 中文 TTS | IndexTTS、CosyVoice、edge-tts | 旁白 |
| 局部换嘴 | MuseTalk、Wav2Lip | 已有片重配音 |
| 表情迁移 | LivePortrait | Hedra 说话头 + 全身视频融合 |
| 配乐 | ACE-Step 等 | BGM |

（本地 OpenMontage / Odyssey-H3 复盘：FLUX 出角色图 + H3/Wan 出片 + IndexTTS + ACE-Step，是「旁白轨」路线，不是「原生对白」路线。）

---

## 4. 按需求选路线（实操）

| 你想要的效果 | 优先工具类型 | 不建议 |
|--------------|--------------|--------|
| 一句话出炫酷短片 | T2V：Wan 3.0 / H3 / Seedance | 纠结哪个 Elo 第一 |
| 指定角色演指定动作 | Motion Control（可灵）或 H3 Ref 视频 | 只用文字描述复杂舞步 |
| 原片背景不动只换人 | Wan Animate Mix / Viggle-Animate | 把通用 R2V 当像素级换人 |
| 多语言本地化已有片 | ElevenLabs Dubbing → Sync/MuseTalk | 指望 Dubbing 单独改嘴 |
| 角色对白短剧 | Kling Omni / Seedance 原生音频 | 无审核批量烧额度 |
| 5 分钟解说片 | 分段生成 + TTS 旁白 + 剪辑 | 一次 prompt 出整片 |
| 零 API 成本 | 开源 H3 / Wan / LTX 本地 | 把免费额度当稳定产能 |

---

## 5. 网上主要参考资料（本次抓取）

### 榜单与总览
1. Artificial Analysis Text-to-Video Leaderboard（With Audio）  
   https://artificialanalysis.ai/video/leaderboard/text-to-video  
2. The Real State of AI Video in 2026  
   https://beginnersinai.org/state-of-ai-video-2026/  
3. RECATOOLS 中文模型规格对照（即梦/可灵/海螺等）  
   https://recatools.com/chinese-ai-video-model-comparison/

### 厂商官方
4. 字节 Seedance 2.5 发布稿  
   https://seed.bytedance.com/zh/blog/一镜成片-随心参考-seedance-2-5-正式发布  
5. 火山方舟 Seedance 2.5 提示词指南（能力表最全）  
   https://ark.volcengine.com/region:cn-beijing/docs/seedance-2-5-prompt-guide  
6. 快手可灵 3.0 新闻稿  
   https://ir.kuaishou.com/zh-hans/news-releases/news-release-details/keling30xiliemoxingquanmianshangxian/  
7. MiniMax H3 / API 文档  
   https://www.minimax.io/blog/minimax-h3  
   https://platform.minimax.io/docs/guides/video-generation  
8. 阿里云百炼「视频生成」能力总表（含换人/换嘴/数字人）  
   https://www.alibabacloud.com/help/en/model-studio/use-video-generation  
9. Google Veo  
   https://deepmind.google/models/veo/  
10. Runway Gen-4.5 / Changelog（Aleph、Act-Two、References）  
    https://help.runwayml.com/hc/en-us/articles/46974685288467-Creating-with-Gen-4-5  
    https://runway.com/changelog  

### 配音
11. ElevenLabs Dubbing API /「不含 lip sync」说明  
    https://elevenlabs.io/blog/ai-dubbing-api  
    https://help.elevenlabs.io/hc/en-us/articles/23793433149073-Do-you-offer-lip-sync-in-Dubbing  
12. Sync Labs Dubbing + Lipsync  
    https://sync.so/docs/tutorials/dubbing.md  

### 局限与控制
13. Kling：当前视频生成局限与绕过方式  
    https://kling.ai/blog/limitations-of-current-ai-video-generation-technology  

---

## 6. 与本地既有报告的关系

| 本地文档 | 与本次网调的关系 |
|----------|------------------|
| `minimax-h3-人物替换与15秒短视频工具调查-2026-11-03.md` | 动作替换/H3 vs Viggle 边界与本文一致；网调补了行业全景 |
| `openmontage-免费生成平台与comfyui方案调研.md` | 偏「怎么便宜跑」；本文偏「玩法与能力」 |
| `odyssey-h3-三支5分钟视频制作复盘.md` | 证明「分段 + TTS」可出 5 分钟；不是单次生成 5 分钟 |
| `comfyui-5090-openmontage-环境搭建指南.md` | 本地 Wan/FLUX/IndexTTS 栈；对应本文「旁白轨」 |

---

## 7. 来源编号

[1] Alibaba Cloud Model Studio — Video generation  
[2] ByteDance Seed — Seedance 2.5 发布博客（2026-07-31）  
[3] 火山方舟 — Seedance 2.5 Prompt Guide  
[4] MiniMax H3 HF / API docs  
[5] Kuaishou IR — 可灵 3.0 上线新闻稿（2026-02-05）  
[6] ElevenLabs — AI dubbing API（Dubbing v2）  
[7] ElevenLabs Help — Dubbing 不含 lip sync  
[8] Sync Labs — Dubbing tutorial（lipsync-2 + elevenlabs）  
[9] Artificial Analysis AA-Video-T2V v2.1 Leaderboard（抓取日）  
[10] beginnersinai.org — State of AI Video 2026  
[11] Kling blog — Limitations of current AI video generation  

---

*本文不构成采购建议；定价与 Elo 请以官方页面实时数据为准。*

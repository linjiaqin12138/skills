# Odyssey-H3：三支 5 分钟「奥德赛」解说片制作复盘（可复刻版）

> 2026-09-11 ~ 09-12 完成。三支视频全部本地推理（API 成本 $0，仅 VAST 租金）。
> 本文档目标：照着做能完整复刻这三支片子。
> 所有脚本本地备份在 `~/Project/odyssey-h3/`，成片在 `~/Project/odyssey-h3/finals/`。

## 1. 成果

| 片 | 标题 | 时长 | 体积 | 视角 |
|---|---|---|---|---|
| v1 | 《归乡者：一个海员的奥德赛》 | 4'51" | 285M | 现代中国远洋船员版 |
| v2 | 《没有名字的奥德赛》 | 4'24" | 221M | 古希腊水手版 |
| v3 | 《佩涅洛佩》 | 4'18" | 246M | 佩涅洛佩（妻子）视角 |

- 每片 20 场景 × 1 条 15s 音画片段，共 60/60 片段全部成功。
- `finals/` 下另有 `*_wx.mp4`（~25M/支），微信发送用压缩版；`*_final.mp4` 为无损原版，MD5 与服务器源文件校验一致。
- 验收：无黑场（片头 3s 黑场字幕卡是有意的）、音轨正常（mean ≈ -36dB）、配旁白+背景音乐+烧录字幕。

## 2. 总体架构

```
剧本 JSON（20 场景，每场景：中文旁白 + 英文画面 prompt + 引用角色 + 时长）
   │
   ├─ FLUX.2 fp8      → 角色参考图（1024×1024，锁人物一致性）
   ├─ IndexTTS-2.5    → 每场景中文旁白 mp3（48k mono）
   ├─ ACE-Step 3.5B   → 180s 背景音乐（循环铺底）
   ├─ MiniMax H3      → 每场景 15s 音画直出片段（1344×768@24fps，R2V 带参考图）
   │
   └─ FFmpeg 合成     → 片段+旁白对齐 → concat → 片头字幕卡 → 升 1080p → 烧 ASS 字幕 → 混配乐
```

整条生产线跑在 GPU 服务器本机（ComfyUI API + 本地脚本），控制端只负责监控和拉回成品。
**不要**在弱控制机上渲染/合成。

## 3. 硬件与实例选择（第一个大坑）

- 平台：VAST.ai，GPU = **RTX 5090（32G VRAM）**。
- **系统 RAM 必须 ≥64GB**（本次用 251G/64C/500G 实例）。H3 的 qwen3vl-nvfp4 文本编码器（14.6G）+ int8 扩散模型（19.5G）加载峰值超 30G，30G RAM 的实例会**无 traceback 被 SIGKILL / 整机卡死**（容器内禁 swap）。`--lowvram` / `--reserve-vram` 对此无效。
- VAST 筛选注意：多数 5090 offer 只配 15~32G RAM。RAM 滑块在筛选面板 CPU/Instance Resources 区（GPU 筛选下方，需往下滚），下限拉 64GB；Disk ≥250GB（模型 ~110GB）；inet down ≥500Mbps 更好。租后先 `free -g` 验证再开装。
- 32G 显存装不下「TE + 扩散 + 采样激活」同时驻留 → 启动加 `--reserve-vram 10` 部分驻留解决。

## 4. 模型清单（共 ~110GB）

全部用 `HF_HUB_ENABLE_HF_TRANSFER=1 /venv/main/bin/hf download <repo> <path> --local-dir <dest>` 下载，注意 hf 会保留仓库子目录结构，需扁平化（见 setup_server.sh 第 3 节）。

### MiniMax H3（主力，音画直出）— repo `Comfy-Org/MiniMax-H3`

| 文件 | 目录 | 用途 |
|---|---|---|
| minimax_h3_fl2va_pruned_int8_convrot.safetensors | diffusion_models/ | T2V / 首尾帧 |
| minimax_h3_ref2va_pruned_int8_convrot.safetensors | diffusion_models/ | **参考图生成（R2V，最多 9 图锁一致性）** |
| qwen3vl_32b_minimax_h3_nvfp4_awq.safetensors | text_encoders/ | 文本编码器（CPU 加载） |
| minimax_h3_video_vae_fp16.safetensors | vae/ | 视频 VAE |
| minimax_h3_audio_vae_fp32.safetensors | vae/ | 音频 VAE（原生 32kHz 立体声） |
| minimax_h3_fl2v_turbo_4step_v1.0_768p_comfyui_bf16.safetensors | loras/ | 官方 4 步 turbo（**实际未用，会炸音频**） |
| minimax_h3_ref2v_turbo_4step_v0.1_comfyui_bf16.safetensors | loras/ | 官方 4 步 turbo（同上） |

⚠️ **setup_server.sh 与 pipeline.py 的 LoRA 文件名不一致（重要）**：setup 下的是官方 4 步版，但实测官方 4 步 turbo 会损坏原生音频。实际生产用的是社区验证的 turbo LoRA（pipeline.py 内引用）：
- T2V：`minimax_h3_turbo_v4_step600_ema_pruned_comfyui.safetensors`
- R2V：`minimax_h3_ref2v_turbo_8step_v1.0_768p_comfyui_resized_avg_rank_64_bf16.safetensors`

来源为社区分享（HuggingFace/Civitai，A/B 测试贴作者 BennyDaBall 的配方），**具体仓库名当时未记录**，复刻时按文件名搜索下载后放入 `models/loras/` 即可；找不到就用非 turbo 模式（20 步，慢约 2.5 倍但质量更好）。

### FLUX.2（角色参考图）— repo `Comfy-Org/flux2-dev`

| 文件 | 目录 |
|---|---|
| split_files/diffusion_models/flux2_dev_fp8mixed.safetensors | diffusion_models/ |
| split_files/text_encoders/mistral_3_small_flux2_fp8.safetensors | text_encoders/ |
| split_files/vae/flux2-vae.safetensors | vae/ |

文本编码器**必须用 fp8 版**（fp4 版 eager 反量化崩溃）。

### ACE-Step（配乐）— repo `Comfy-Org/ACE-Step_ComfyUI_repackaged`

- `all_in_one/ace_step_v1_3.5b.safetensors` → checkpoints/

### IndexTTS-2.5（旁白）

- 由 ComfyUI 自定义节点 **TTS-Audio-Suite**（github.com/diodiogod/TTS-Audio-Suite）自动下载，模型名 `IndexTTS-2.5`。
- 参考音用内置 `voices_examples/male/male_02.wav`（librosa pyin 实测 pitch_std=11Hz、mean=84Hz，最沉稳；zh_man_sichuan 偏浮夸已被否）。

## 5. 服务器环境搭建

一键脚本：`~/Project/odyssey-h3/setup_server.sh`（在全新 VAST 实例上以 root 执行）。内容摘要：

1. ComfyUI 升级到 master 最新（H3 需要 ≥0.30 原生支持，节点在 `comfy_extras/nodes_minimax_h3.py`）；装 `huggingface_hub[hf_transfer]`、`sageattention`（**装了但启动别用**，见坑 2）。
2. 下载上节全部模型并扁平化目录。
3. clone TTS-Audio-Suite，装依赖：
   `librosa matplotlib sentencepiece descript-audiotools descript-audio-codec argbind accelerate faster-whisper json5 omegaconf openai-whisper munch`（后三个是隐性依赖，漏装会报 import 错）。
4. `apt install -y ffmpeg fonts-noto-cjk`（字幕/标题卡需要中文字体）。
5. 生成启动脚本（**以下是修正后的最终版**，setup 里的原版含 sage-attention 需去掉）：

```bash
#!/bin/bash
cd /workspace/ComfyUI
exec env PYTORCH_CUDA_ALLOC_CONF=expandable_segments:True /venv/main/bin/python main.py \
  --disable-auto-launch --port 18188 --enable-cors-header \
  --reserve-vram 10 --disable-cuda-malloc >> /root/comfyui_run.log 2>&1
```

启动 ~90s。8188 端口是 VAST 的 Caddy 反代（Basic auth 401），ComfyUI 本体在 **18188**；代码里一律用 `127.0.0.1`（`localhost` 可能解析到 ::1）。

## 6. 生产脚本（本地 `~/Project/odyssey-h3/` ↔ 服务器 `/workspace/production/`）

| 文件 | 作用 |
|---|---|
| `setup_server.sh` | 实例初始化（上节） |
| `build_scripts.py` | 生成三个剧本 JSON（角色定义、场景旁白、英文 prompt、refs、dur） |
| `script_v1/v2/v3.json` | 剧本产物，风格前缀统一：`Photorealistic live-action cinematic footage, shot on ARRI Alexa with anamorphic lenses, … documentary-grade realism, handheld micro-shake, no CGI look…` |
| `pipeline.py` | 核心管线，`refs / tts / clips / music / one_t2v / one_r2v` 子命令，全部幂等断点续跑（文件存在即跳过），每片段失败自动重试 3 次 |
| `produce_all.sh` | 总控：三支片 refs→tts→music→clips→compose 串行，`setsid bash produce_all.sh </dev/null >/root/produce_all.log 2>&1 &` 点火 |
| `compose.py` | FFmpeg 合成（见第 8 节） |

服务器目录结构：

```
/workspace/production/
  scripts/script_{v1,v2,v3}.json
  projects/<vid>/
    refs/<角色>.png        # FLUX.2 参考图
    audio/s01.mp3 …        # IndexTTS 旁白（48k mono）
    music/music.mp3        # ACE-Step 180s
    clips/s01.mp4 …        # H3 15s 音画片段
    compose/               # 中间产物
    <vid>_final.mp4        # 成片
```

pipeline.py 与 ComfyUI 交互方式：POST `/prompt` 提交 API 格式 workflow JSON → 轮询 `/history/<prompt_id>`（5s 间隔，连续 24 次连接失败判定服务已死——**必须加这个判定，否则无限等待**）。

## 7. 关键参数配方（都是踩坑换来的）

### H3 视频（turbo 模式）

- T2V：`euler + beta 调度，8 步，CFG-free（BasicGuider 直连），turbo v4 step600 ema LoRA @1.0`
- R2V：`res_multistep + beta，8 步，ref2v_turbo_8step_rank64 LoRA @1.0`
- 画布 **1344×768**（H3 原生 16:9 ~1MP），24fps；帧数必须落 **17k+5 网格**（15s = 362 帧），`snap_frames()` 函数处理。
- 单条 15s ≈ **18 分钟**（5090）；非 turbo 20 步质量更好但更慢。
- R2V 参考图在 API JSON 里必须**嵌套**传：`"ref_images": {"ref_image_1": ["21",0], …}`，并在 prompt 末尾加图例：`<Picture 1> is LAOZHOU (use for exact face, hair and wardrobe consistency)`。
- CLIPLoader 的 H3 文本编码器 `device: "cpu"`。

### IndexTTS 旁白

`IndexTTSEngineNode(IndexTTS-2.5, language=Chinese, emotion_alpha=0.6, temperature=0.6, top_p=0.8, top_k=30, num_beams=3, repetition_penalty=10.0, use_fp16=True)` → `UnifiedTTSTextNode(narrator_voice=voices_examples/male/male_02.wav, seed=42, max_chars_per_chunk=300)` → SaveAudio → ffmpeg 转 48k mono mp3。中文语速 ~4.2 字/s，5 分钟片 ≈ 1250~1350 字。

### ACE-Step 配乐

180s，`ModelSamplingSD3 shift=5.0 + LatentTonemapReinhard + LatentApplyOperationCFG`，KSampler euler/simple 50 步 cfg=5，SaveAudioMP3 V0。每片 tags 不同（v1 忧郁钢琴弦乐 / v2 古希腊里拉琴 / v3 大提琴独奏，详见 produce_all.sh）。

### 合成（compose.py）

- 时间轴由**旁白驱动**：每场景时长 = 旁白时长 + 1.2s 尾（片段不足则 `tpad=stop_mode=clone` 冻结末帧，超出则裁剪）。
- 混音：片段原生音（环境声）压到 **0.16**，配乐 **0.05**，旁白 1.0。
- 硬切（纪录片风格），3s 黑场标题卡（Noto Sans CJK Bold）。
- 最终升 1920×1080（lanczos）+ 烧 ASS 字幕（22 字/行，按标点断行），libx264 crf18（分段）/crf19（终片），aac 192k，`+faststart`。
- **该容器无 nvenc** → 一律 libx264。

## 8. 环境问题与教训清单（复刻必读）

1. **RAM OOM 无报错 SIGKILL**：见第 3 节，≥64G RAM 是硬门槛。
2. **`--use-sage-attention` 导致 H3 输出全黑**：setup 里装了 sageattention 但启动参数必须去掉它。
3. **官方 4 步 turbo LoRA 炸音频**：换社区 v4 step600 ema（T2V）/ 8step rank64（R2V），euler+beta+8 步配方。
4. **显存**：32G 不够 TE+扩散+采样同驻 → `--reserve-vram 10`；`cudaMallocAsync` 会预占 31G 不放 → `--disable-cuda-malloc`；`PYTORCH_CUDA_ALLOC_CONF=expandable_segments:True`。
5. **ComfyUI 队列 FIFO**：慢任务（H3 片段）会堵住后面的快任务（TTS），曾致 TTS 超时失败——分阶段串行跑，别混投。
6. **ssh 陷阱**：远程 `pkill -f xxx` 会匹配到自身命令行自杀（用 `[x]xx` 括号技巧）；ssh 起远程后台任务会挂住本地会话（用 `setsid … </dev/null` 或本地 background）；VAST 的 authorized_keys 文件末尾无换行，直接 `>>` 追加会把新 key 粘进上一行注释里失效——先规范化换行再写，写后验证条数并另开连接测试。
7. **文件回拉**：VAST→本地单连接会掉到 ~20KB/s。解法：①每文件一条并行 rsync（聚合 ~1.1MB/s）；②更狠的走 HK 服务器中转（VAST→HK 1.6MB/s，HK=`ssh -p 27963 root@103.106.189.163`），本地直连不用停，谁先齐用谁。rsync 写临时文件，`ls` 看不到进度，用 `du -sb` 估。
8. **OpenMontage GitHub HEAD 过时**（引用 nvfp4/fp4 旧模型名）：本项目没用 OpenMontage 的 workflow，pipeline.py 直接构造 API JSON，不受此影响；若复用 OpenMontage 需打补丁。
9. **Wan3.0 无开源权重**（2026-09 确认，仅托管 API）——这就是选 H3 的原因。
10. **模型下载**：必须 `HF_HUB_ENABLE_HF_TRANSFER=1`，110GB 全速约 20~30 分钟。

## 9. 验收清单（交付前必跑）

1. clips 数 == 剧本场数（20/片）；`*_final.mp4` 存在且 ffprobe 时长符合预期。
2. 抽 10%/50%/90% 三帧查黑场（PIL 灰度均值 <10 可疑）；`ffmpeg -vf blackdetect=d=0.5:pix_th=0.05` 全片扫（片头 3s 字幕卡除外）。
3. `ffprobe -select_streams a` 确认 aac 存在；`volumedetect` 参考值 mean ≈ -36dB（全 -91dB 即无声事故）。
4. 回拉后 MD5 与服务器源文件比对一致。

## 10. 复刻 Quickstart

```bash
# 0. 租 VAST 5090（RAM≥64G, Disk≥250G），ssh 上去
# 1. 初始化（~40min，含 110G 模型）
bash setup_server.sh
# 2. 补下社区 turbo LoRA（见第 4 节 ⚠️），修正启动脚本后启动 ComfyUI
setsid bash /root/start_comfy.sh </dev/null >/dev/null 2>&1 &
# 3. 部署生产代码
mkdir -p /workspace/production/scripts
scp pipeline.py compose.py produce_all.sh → /workspace/production/
scp script_v*.json → /workspace/production/scripts/
# 4. 冒烟（可选但强烈建议）
python3 pipeline.py one_t2v "test prompt" 15   # 确认非黑场、有音频
# 5. 点火量产（~18-20h，60 条片段）
setsid bash produce_all.sh </dev/null >/root/produce_all.log 2>&1 &
# 6. 监控：tail -f /root/produce_all.log，看到 ALL_VIDEOS_DONE 即完成
# 7. 验收（第 9 节）→ 并行 rsync 回拉 → 销毁实例
```

成本参考：全程约 20~24h 实例时间（含搭建），VAST 5090 按时计费；API 费用 $0。

## 11. 相关文档

- `~/Project/skills/research/comfyui-5090-openmontage-环境搭建指南.md` — ComfyUI 5090 环境 + 10 踩坑
- `~/Project/skills/research/openmontage-特洛伊战争与奥德赛视频生成记录*.md` — 前作（Wan2.2 方案）
- `~/Project/skills/research/odyssey-firstperson-交接文档.md`、`odyssey-firstperson-制作复盘.md` — 第一支第一人称版
- Hermes 技能 `openmontage-comfyui-video`（含 `references/minimax-h3-comfyui.md` 节点接线细节、`templates/h3_pipeline.py` 管线模板）

## 12. 用户验收反馈（2026-09-12，总评 68/100）

用户看完三支成片后的意见，按问题分类，附归因与改进方向：

| # | 问题 | 具体现象 | 归因 | 改进方向 |
|---|---|---|---|---|
| 1 | **字幕过小** | 手机上观看吃力 | compose.py 中 ASS 字号 22（PlayResY 1080），偏小 | 字号提到 32~40，加粗；或按目标平台（竖屏手机）加大 |
| 2 | **配音单一** | 三片全是 male_02 男声；《佩涅洛佩》是女性视角，应用女声 | TTS 参考音写死 male_02 | 按剧本视角选参考音；对话类需多角色音色（IndexTTS 支持每段换 voice） |
| 3 | **H3 原生音频有怪声** | 如《归乡者》老周拿望远镜片段前有电话铃声；有人说话的片段人声像「咒语」（无意义嘟囔） | H3 音画直出的原生音频不可控，模型会自造语音/音效 | **解说类视频应完全弃用片段原生音轨**（compose 时不混 clip audio，即 CLIP_AUDIO_VOL=0 或不接该输入），只保留画面，后期铺独立 BGM + 旁白即可 |
| 4 | **画面风格跳变** | 如《没有名字的奥德赛》2:00 起画面像「玩具总动员」（3D 动画感） | 部分场景 prompt 没压住风格 / 模型对某类场景的先验偏卡通；style_prefix 约束力有限 | 每场景 prompt 强化风格锚点；出片后逐段验收，风格跑偏的重渲（改 prompt/换 seed） |
| 5 | **抽卡现象** | 片段质量不稳定，时好时坏 | 单 seed 单次生成，无筛选 | 关键场景多 seed 生成 2~3 条选优（成本 ×2~3，需权衡时长） |
| 6 | **表情僵硬、眼神空洞** | 如《归乡者》人物保持笑容但眼角嘴角全程不动，「静态的笑」，一眼 AI | 15s 长片段里人物近景微表情是 H3 短板 | prompt 里明确要求表情变化/微动作；近景面部特写少用或切短；优先中远景和动作场景 |
| 7 | **镜头时长一刀切** | 60 条片段全部 15s，节奏单调 | pipeline 写死 `dur=15` | **分镜级时长规划**：按内容混排 5s / 10s / 15s；章节之间可用「插图 + 标题卡 + 短暂停顿」作故事转折/主题切换的提示（静态图可接受，但绝大部分仍应是视频）。成本上有利：生成时长非线性——15s H3 片段 ~18-20min，而 5s 片段（Wan2.2 时代实测）~2min，切短反而更省。风险：片段越短越多，场景/人物一致性越难维持 → **必须提前做好人物、场景参考图**（R2V refs），并在生成视频前把每个镜头的时长、景别、转场规划进剧本 JSON，不要一律 15s |

### 经验提炼（下个项目直接生效）

- **解说类视频的标准做法**：画面（静音）+ 独立旁白 + 独立 BGM 三层分离，不要用模型直出音频。H3 的音画直出能力在此场景反而是减分项。
- TTS 音色必须匹配叙事视角（女性视角用女声）。
- 字幕按手机端观看设计（字号 ≥32）。
- **剧本阶段就做分镜规划**：每镜头标注时长（5/10/15s 混排）、景别、是否用插图标题卡转场；pipeline 的 `dur` 字段已支持逐场景覆盖，不要再默认 15s。
- 人物一致性靠 R2V 参考图已解决，但**风格一致性**、**表情自然度**、**短片段间场景一致性**是 H3 当前的主要短板——短镜头混排方案下参考图体系要更重。

### 下一步方向（用户已定）

尝试**爽文剧场**类视频：不再是旁白解说，而是**有角色对话、有剧情推进**的短剧形式。需要解决：多角色 TTS 音色分配、对话与画面口型/表演的匹配（或规避口型特写）、分镜级剧本格式。

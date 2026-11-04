# OpenMontage 生成「特洛伊战争与奥德赛」ComfyUI 5090 版记录

> 记录日期：2026-09-09
> 执行机器：腾讯云 VM（OpenMontage 端，`~/Project/OpenMontage`）
> 算力端：VAST.ai 租的 RTX 5090 实例（`ssh -p 51849 root@58.224.7.137`）
> 最终输出：`~/Project/OpenMontage/projects/trojan-odyssey-v3/renders/trojan-odyssey-v3.mp4`

---

## 1. 任务目标

在前两版（PPT 版 → AI 验证版）基础上，用 **ComfyUI + RTX 5090** 全本地方案生成新版本：

- 中文旁白用 ComfyUI 生态 TTS（不用 edge-tts）
- 图片 + **真视频片段**结合（解决「只是静态图片」）
- 时长 5-10 分钟
- 零 API 成本（除 VAST 租金）

## 2. 最终产出

| 项目 | 内容 |
|------|------|
| 视频文件 | `projects/trojan-odyssey-v3/renders/trojan-odyssey-v3.mp4` |
| 时长 | 333.4 秒（5 分 33 秒） |
| 分辨率 | 1920×1080 @ 30fps，h264 + aac 48kHz |
| 文件大小 | 约 102 MB |
| 音轨 | IndexTTS-2.5 中文男声旁白 + ACE-Step 生成配乐（混音，音乐音量 0.10） |
| 字幕 | 中文短语级字幕，ASS 烧录（Noto Serif CJK SC） |
| 视觉 | 12 张 FLUX.2 场景图 + 12 条 Wan 2.2 i2v 动态片段（每场景 5s 动态 + 末帧 Ken Burns 延伸） |
| 总成本 | $0 API 费用 |

## 3. 技术栈（全本地/自托管）

| 组件 | 方案 |
|------|------|
| 图像 | FLUX.2 dev fp8mixed + Mistral-3 fp8 文本编码器（ComfyUI） |
| 视频 | Wan 2.2 14B i2v fp8 + LightX2V 4 步 LoRA（ComfyUI，1280×720×81帧） |
| TTS | **IndexTTS-2.5**（TTS-Audio-Suite 节点，参考音 `zh_man_sichuan.wav` 克隆） |
| 音乐 | ACE-Step v1 3.5B（ComfyUI，180s 史诗管弦） |
| 合成 | FFmpeg（分段 + zoompan + 混音 + ASS 字幕） |
| 框架 | OpenMontage（项目结构、artifact、comfyui_image/video/music 工具） |

## 4. 5090 服务器踩坑记录（重要）

1. **ComfyUI 0.3.47 不支持 FLUX.2** → `git reset --hard origin/master` 升到 0.35.0 解决
2. **Mistral fp4 文本编码器在 eager 模式反量化崩溃**（`index out of range`）→ 换 `mistral_3_small_flux2_fp8.safetensors`（18GB）
3. **Wan 采样 OOM 假根因排查**：看起来是显存不足，实际是 **cudaMallocAsync 分配器 reserve 31GB 不释放**，active 仅 18.4GB → 加 `--disable-cuda-malloc` 解决。配套加 `--use-sage-attention`（pip install sageattention）
4. **UMT5-xxl fp8 文件会被 cast 成 fp16 常驻 13.4GB** → 工作流 CLIPLoader 设 `device: cpu`（文本编码走 CPU，67GB 内存足够，每次编码约 30-60s）
5. **SSH 端口转发隧道传大文件会断**（IncompleteRead）→ 小请求（提交/轮询/上传）走隧道，**成品文件用 scp 直接拉取**
6. **ssh 远程 pkill 自杀陷阱**：`ssh host 'pkill -f "main.py"'` 的 pkill 会匹配到远端 bash 自身的命令行 → 用 `[m]ain.py` 写法规避
7. VAST 实例下 HF 模型用 `HF_HUB_ENABLE_HF_TRANSFER=1` + `hf download`，~100GB 约 40 分钟

## 5. OpenMontage 端的改动

- `tools/_comfyui/workflows/flux2-txt2img.json`：模型名改为 `flux2_dev_fp8mixed.safetensors` + fp8 文本编码器
- `tools/_comfyui/workflows/wan22-{i2v,t2v}-4step.json`：CLIPLoader `device: cpu`
- `tools/graphics/comfyui_image.py`、`tools/_comfyui/metadata.py`：同步模型名
- `remotion-composer/src/Explainer.tsx`：补 `wordSeparator` 透传（中文字幕，来自上一版报告的补丁，上游仍未修）
- `tools/video/video_compose.py`：media_keys 加 `backgroundImage/backgroundVideo`（同上）

## 6. 已知不足 / 下版改进

1. **Remotion 渲染超时**：344s×30fps 在 3.6GB 内存小机上跑不动（960s 超时），改用 FFmpeg。若换大内存机器可回 Remotion 获得更精致的字幕/转场
2. **动态占比仍偏低**：每场景只有前 5 秒是真动态（之后是末帧 Ken Burns）。下版可 i2v 链式续帧（末帧→下一段）把动态占比提到 50%+
3. **TTS 参考音是四川口音男声**（套件唯一中文参考），可以录一段标准普通话参考音改善
4. 字幕是按字符比例均分的短语级，非真实对齐；可用 whisperx 对齐后生成词级高亮
5. 本次视觉验证因本机 vision API 故障未做逐帧人工审，由用户抽查

## 7. 复现要点

```bash
# 隧道（带自动重连）
ssh -N -L 8188:127.0.0.1:18188 -p 51849 root@58.224.7.137
# .env: COMFYUI_SERVER_URL=http://localhost:8188
# 资产生成（断点续跑）
python projects/trojan-odyssey-v3/generate_assets.py
# 补剩余片段（scp 下载版）
python projects/trojan-odyssey-v3/gen_remaining_clips.py s9 s10 s11 s12
# 合成
python projects/trojan-odyssey-v3/render_ffmpeg.py
```

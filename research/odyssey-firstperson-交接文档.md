# 交接文档：奥德赛第一人称版视频（odyssey-firstperson）生产中断了

> 写于：2026-09-10
> **2026-09-10 更新3：✅ 已完成交付**。成片 `~/Project/OpenMontage/projects/odyssey-firstperson/renders/odyssey-firstperson.mp4`（10'01"，1920x1080，223MB，48 条链式 i2v + 183 条短语字幕 + 音乐）。服务器原件：122.117.118.68:/workspace/OpenMontage/projects/odyssey-firstperson/renders/。合成用 compose.py（nvenc 在该容器不可用——`OpenEncodeSessionEx failed: unsupported device`，改 libx264）。本文档以下内容为历史记录。
> **2026-09-10 更新2：已在新 5090 实例恢复生产**。第一台新机（97.70.195.221）宿主机 GPU 驱动被别的租户搞挂（容器内无解，见指南坑 11），换机到 `ssh -p 35456 root@122.117.118.68`（备选 ssh7.vast.ai:20761）。环境按指南重建完成，produce.py 运行中（TTS 18 段 duration_factor=1.28 已完成，总旁白 ~601s 达标；FLUX 图 + Wan 链式片段生成中，预计 ~3h）。compose.py（FFmpeg+nvenc 合成）已上传服务器。看门狗 cron `74086662cca6` 在资产齐/崩溃时发微信。
> **注意坑**：① 本地 produce.py 的 duration_factor 曾是 1.0（旧实例上的 1.28 版没同步回来），已修正；② OpenMontage GitHub HEAD 的工作流/metadata/comfyui_image.py 仍引用 nvfp4/fp4 模型名，服务器克隆后必须用本地已补丁文件覆盖（tools/_comfyui/workflows/*.json、metadata.py、tools/graphics/comfyui_image.py）；③ pkill 远程自杀陷阱升级版：同一条 ssh 命令行里只要出现 "produce.py" 路径字样就会被 pkill -f 匹配杀掉自己——杀进程永远单独一条 ssh。
> 状态：**中断待恢复**。VAST 实例的 GPU 从 5090 被重置成 3060（12GB），Wan 14B 跑不了，生产停在中途。用户决定先停机省钱，之后择机恢复。
> 本档目标：任何人（或未来的 agent）读完即可续跑，不需要重新踩坑。

---

## 1. 任务定义（用户原始需求）

做一版新的《奥德赛》讲解视频：

- **奥德修斯第一人称自述**视角
- 人声换**沉稳**的（v3 用的 `zh_man_sichuan.wav` 克隆音被评价"太浮夸"）
- 视觉：**吉卜力/哆啦A梦画风**（最终定的 prompt 风格是吉卜力手绘水彩）
- **尽可能用真视频片段**而非静态图（每场景 2-3 条链式 i2v）
- 时长 **10 分钟以上**
- 完成后微信通知（cron job `wechat-notify`，id `5292cd1bcb21`）

## 2. 当前进度

| 阶段 | 状态 |
|------|------|
| 脚本 | ✅ 18 节 2515 字第一人称脚本（`artifacts/script.json`），含占位时间轴 |
| 声音选型 | ✅ 已选定 `voices_examples/male/male_02.wav`（4 候选中 pitch_std=11 最稳、84Hz 最低沉）；emotion_alpha=0.6、temperature=0.6 |
| TTS 旁白 | ✅ **18 段全部生成**（IndexTTS-2.5，duration_factor=1.28 放慢后总时长约 595s + 间隔 ≈ 10.2 分钟，达标） |
| 场景图（FLUX 吉卜力风） | 🔵 s1、s2 完成（另有 s1 的两个链式末帧图） |
| 视频片段（Wan i2v 链式） | 🔵 仅 s1 的 a/b/c 三条 + s2_a 完成 |
| 音乐 | ❌ 未生成（prompt 已写好：Joe Hisaishi 风格） |
| 合成 | ❌ 未开始（方案已定为服务器上 FFmpeg+nvenc） |

**注意**：TTS 生成过两轮。第一轮 duration_factor=1.0（总时长仅 8 分钟不达标）已全部删除重生成。当前磁盘上的音频是 1.28 版。

## 3. 资产位置

- **服务器**：`/workspace/OpenMontage/projects/odyssey-firstperson/`（TTS 18 段 + s1 全部片段 + s2_a + 2 张场景图只存在这里；VAST **停机保留磁盘，销毁才丢失**）
- **本机**：`~/Project/OpenMontage/projects/odyssey-firstperson/` 只有 `artifacts_script_draft.json`（脚本）和 `produce.py`（生产脚本）
- ⚠️ 2026-09-10 曾尝试把服务器资产 rsync 回本机，但实例已被停机（connection refused），**备份未成功**。恢复时若重启了原实例，先把 projects/odyssey-firstperson 拉回本机再继续
- 生产脚本 `produce.py` 断点续跑：跳过已存在的资产

## 4. 恢复生产的两条路

### 路线 A：重开 5090/4090 实例（推荐，质量不变）

1. VAST 租 24GB+ 显存实例（5090 最优），ComfyUI 模板
2. 按 `comfyui-5090-openmontage-环境搭建指南.md` 重建环境（升级 ComfyUI → 下模型 ~40min → TTS-Audio-Suite → sageattention → 正确启动参数）
3. 同机部署 OpenMontage（指南第 5 节；注意 **COMFYUI_SERVER_URL 用 127.0.0.1:18188，8188 是 Caddy 反代会 401**）
4. 重启原实例则先拉回资产（`rsync -avP -e 'ssh -p 51849' root@<ip>:/workspace/OpenMontage/projects/odyssey-firstperson/ <本地>/`）；新实例则只传本机的脚本和 produce.py，TTS 重跑只要 3 分钟
5. `nohup .venv/bin/python projects/odyssey-firstperson/produce.py &` 续跑（已完成的资产自动跳过）
6. 全部资产完成后：合成 → 拉回成品 → 微信通知

### 路线 B：就在 3060 上跑完（省钱，质量略降）

Wan 2.2 14B fp8（13.6GB）装不进 12GB。两个替代：

- **Wan 2.2 TI2V-5B**（Comfy-Org 有 repackaged，单模型 ~10GB，12GB 可跑 720p，质量可用）
- 或 **14B GGUF Q4**（~8GB，需装 ComfyUI-GGUF 节点，质量损失较小）

同时把 `produce.py` 里工作流从 `wan22-i2v-4step` 换成对应 5B/GGUF 工作流（OpenMontage `tools/_comfyui/workflows/` 里加一个新模板），FLUX.2 图像在 12GB 上也紧张，可换 FLUX.1-schnell fp8。

## 5. 关键技术细节（恢复时要用的）

- **ComfyUI 启动**：`/venv/main/bin/python main.py --disable-auto-launch --port 18188 --enable-cors-header --use-sage-attention --disable-cuda-malloc`（两个 flag 都是血泪，见指南坑 #3）
- **5090 实例重置后 ComfyUI 会被 VAST supervisor 以默认参数拉起**——恢复时先确认进程参数对不对
- **Wan 工作流 CLIPLoader 必须 `device: cpu`**（UMT5 会被 cast 成 fp16 占 13.4GB 显存）
- **FLUX.2 用 fp8mixed 版 + mistral fp8 文本编码器**（fp4 版有 eager 反量化 bug）
- **链式 i2v**：clip a 用 FLUX 关键帧 → ffmpeg 取末帧 → 作为 clip b 的参考图 → 以此类推；长场景（>30s 旁白）3 条，短场景 2 条
- **TTS 工作流**：IndexTTSEngineNode（language=Chinese, duration_factor=1.28）→ UnifiedTTSTextNode（narrator_voice=voices_examples/male/male_02.wav）→ SaveAudio
- **ssh 远程 pkill 自杀陷阱**：pkill 和后续命令里不要再出现同一进程名（方括号技巧也救不了同命令行里的第二个匹配），杀进程单独一条 ssh
- **微信通知流程**：`hermes cron resume 5292cd1bcb21 --at <未来时间>` 激活 → `cronjob action='run'` 带消息文本触发 → 别连发（iLink 有 30s 冷却）

## 6. 合成方案（资产齐了之后）

在服务器上跑 FFmpeg 合成（比 Remotion 快 10 倍+，见指南 7.5 节）：
- 每场景：clip a → clip b →（clip c）→ 末帧 zoompan 补足旁白时长
- 旁白重编码拼接（**不要 `-c copy`**，会丢 3-4 秒导致音画错位，见指南坑 #8）
- 音乐 ACE-Step 生成 180s 循环，音量 0.10，首尾淡入淡出
- 字幕：按中文标点切短语级 ASS 烧录（Noto Serif CJK SC）
- 编码：`-c:v h264_nvenc`
- 参考实现：本机 `~/Project/OpenMontage/projects/trojan-odyssey-v3/render_ffmpeg.py`（改一下分段逻辑即可用）

## 7. 相关文档

- 环境搭建与踩坑：`./comfyui-5090-openmontage-环境搭建指南.md`
- v3 版（陶瓶画风格，已交付）记录：`./openmontage-特洛伊战争与奥德赛视频生成记录-comfyui5090版.md`
- 免费 API 与 ComfyUI 方案调研：`./openmontage-免费生成平台与comfyui方案调研.md`

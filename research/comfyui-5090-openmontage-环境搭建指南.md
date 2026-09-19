# ComfyUI 5090 服务器环境搭建指南（OpenMontage 全本地视频生产）

> 整理日期：2026-09-09
> 验证环境：VAST.ai RTX 5090 实例（12 核 / 62GB RAM / 200GB 盘）
> 目标：从零复现「FLUX.2 出图 + Wan 2.2 出片 + IndexTTS 中文配音 + ACE-Step 配乐」全本地、零 API 成本的 OpenMontage 生产环境

---

## 0. 前置条件

- 一台带 RTX 5090（32GB）的机器。VAST.ai 选 ComfyUI 模板实例最省事（自带 ComfyUI + venv `/venv/main`）
- SSH 免密登录配置好
- 磁盘：模型约 **120GB**，建议 200GB 起
- 本文所有命令在服务器上以 root 执行（VAST 默认）

---

## 1. 升级 ComfyUI（必须）

VAST 模板自带的 ComfyUI 0.3.47 **不支持 FLUX.2**，必须升级：

```bash
cd /workspace/ComfyUI
git fetch origin master
git reset --hard origin/master        # 0.3.47 -> 0.35.0
/venv/main/bin/pip install -r requirements.txt
```

验证 torch 没被动坏：

```bash
/venv/main/bin/python -c "import torch; print(torch.__version__, torch.cuda.get_device_name(0))"
# 2.7.1+cu128 NVIDIA GeForce RTX 5090
```

## 2. 安装 huggingface CLI 并下载模型

```bash
/venv/main/bin/pip install "huggingface_hub[hf_transfer]"
export HF_HUB_ENABLE_HF_TRANSFER=1
```

下载清单（~120GB，VAST 国际带宽约 40 分钟）：

```bash
HF=/venv/main/bin/hf
M=/workspace/ComfyUI/models

# --- Wan 2.2 视频（t2v + i2v 各两个模型 + LoRA + 文本编码器 + VAE）---
W22=Comfy-Org/Wan_2.2_ComfyUI_Repackaged
$HF download $W22 split_files/diffusion_models/wan2.2_t2v_high_noise_14B_fp8_scaled.safetensors --local-dir $M/diffusion_models
$HF download $W22 split_files/diffusion_models/wan2.2_t2v_low_noise_14B_fp8_scaled.safetensors  --local-dir $M/diffusion_models
$HF download $W22 split_files/diffusion_models/wan2.2_i2v_high_noise_14B_fp8_scaled.safetensors --local-dir $M/diffusion_models
$HF download $W22 split_files/diffusion_models/wan2.2_i2v_low_noise_14B_fp8_scaled.safetensors  --local-dir $M/diffusion_models
$HF download $W22 split_files/text_encoders/umt5_xxl_fp8_e4m3fn_scaled.safetensors --local-dir $M/text_encoders
$HF download $W22 split_files/vae/wan_2.1_vae.safetensors --local-dir $M/vae
$HF download $W22 split_files/loras/wan2.2_t2v_lightx2v_4steps_lora_v1.1_high_noise.safetensors --local-dir $M/loras
$HF download $W22 split_files/loras/wan2.2_t2v_lightx2v_4steps_lora_v1.1_low_noise.safetensors  --local-dir $M/loras
$HF download $W22 split_files/loras/wan2.2_i2v_lightx2v_4steps_lora_v1_high_noise.safetensors    --local-dir $M/loras
$HF download $W22 split_files/loras/wan2.2_i2v_lightx2v_4steps_lora_v1_low_noise.safetensors     --local-dir $M/loras

# --- FLUX.2 图像（注意选 fp8，别选 fp4，见坑 #3）---
FX=Comfy-Org/flux2-dev
$HF download $FX split_files/diffusion_models/flux2_dev_fp8mixed.safetensors          --local-dir $M/diffusion_models
$HF download $FX split_files/text_encoders/mistral_3_small_flux2_fp8.safetensors      --local-dir $M/text_encoders
$HF download $FX split_files/vae/flux2-vae.safetensors                                --local-dir $M/vae

# --- ACE-Step 音乐 ---
$HF download Comfy-Org/ACE-Step_ComfyUI_repackaged all_in_one/ace_step_v1_3.5b.safetensors --local-dir $M/checkpoints
```

**`hf download --local-dir` 会保留仓库子目录**（文件落在 `$M/diffusion_models/split_files/diffusion_models/...`），下载完要扁平化：

```bash
cd /workspace/ComfyUI/models
for d in diffusion_models text_encoders vae loras checkpoints; do
  find $d -mindepth 2 -name "*.safetensors" -exec mv -n {} $d/ \;
done
rm -rf */split_files checkpoints/all_in_one
```

## 3. 安装中文 TTS（TTS-Audio-Suite + IndexTTS-2.5）

```bash
cd /workspace/ComfyUI/custom_nodes
git clone --depth 1 https://github.com/diodiogod/TTS-Audio-Suite.git
/venv/main/bin/pip install -r TTS-Audio-Suite/requirements.txt
# IndexTTS-2.5 的额外依赖（官方 requirements 没列全，逐个试出来的）：
/venv/main/bin/pip install librosa matplotlib sentencepiece \
    descript-audiotools descript-audio-codec argbind accelerate faster-whisper json5 \
    omegaconf openai-whisper munch   # 2026-09-10 新实例实测还要这三个（版本漂移）
```

IndexTTS-2.5 模型首次调用时自动从 HF 下载（约 5GB，存到 `models/TTS/IndexTTS/`）。

用法（API 工作流）：`IndexTTSEngineNode`(model_path=IndexTTS-2.5, language=Chinese) → `UnifiedTTSTextNode`(narrator_voice 选参考音) → `SaveAudio`。参考音用插件自带的 `voices_examples/higgs_audio/zh_man_sichuan.wav`（唯一中文样本，四川口音），或自己上传标准普通话样本到 `custom_nodes/TTS-Audio-Suite/voices_examples/`。

## 4. 安装 SageAttention + 关键启动参数（5090 显存生死线）

```bash
/venv/main/bin/pip install sageattention   # nvcc 已在 VAST 镜像里，triton 即时编译
```

**启动命令（两个 flag 缺一不可）：**

```bash
cd /workspace/ComfyUI
nohup /venv/main/bin/python main.py \
  --disable-auto-launch --port 18188 --enable-cors-header \
  --use-sage-attention --disable-cuda-malloc \
  > /root/comfyui_run.log 2>&1 &
```

启动需要 **约 90 秒**（TTS-Audio-Suite 导入一堆引擎 + ComfyUI-Manager 拉注册表），别以为它死了。

## 5. （可选）同机部署 OpenMontage，用本机 GPU 加速渲染

如果控制机性能弱（比如 2 核 4GB 的云 VM），可以把 OpenMontage 直接装在 ComfyUI 服务器上——12 核 + 62GB RAM + 5090，Remotion/FFmpeg 都快得多，且 `COMFYUI_SERVER_URL=http://localhost:8188` 本机直连、不需要 SSH 隧道：

```bash
cd /workspace
git clone https://github.com/calesthio/OpenMontage.git
cd OpenMontage
# 无 node 的话先装：curl -fsSL https://deb.nodesource.com/setup_22.x | bash && apt install -y nodejs
make setup
echo "COMFYUI_SERVER_URL=http://127.0.0.1:18188" >> .env
```

⚠️ **端口别写错**：VAST ComfyUI 模板上 **8188 是 Caddy 反代**（带 Basic auth，API 调用返回 401），ComfyUI 本体在 **18188**。同机部署要用 18188。另外 `localhost` 可能解析到 ::1（IPv6）而 ComfyUI 只听 127.0.0.1，写 `127.0.0.1` 最稳。

GPU 加速点：
- Remotion 用 headless Chrome，可走 GPU（`--enable-gpu` 场景）；
- FFmpeg 编码换 `-c:v h264_nvenc`（5090 有 NVENC），比 libx264 快 5-10 倍；
- 资产不跨网络传输（tunnel 传大文件会断，见坑 #5）。

---

## 6. 踩坑与解决（按出现顺序）

### 坑 1：FLUX.2 工作流在旧版 ComfyUI 直接报错
0.3.47 没有 FLUX.2 架构支持（`EmptyFlux2LatentImage`、`Flux2Scheduler` 等节点不存在）。**升级到 0.35.0 解决**（见第 1 节）。

### 坑 2：Mistral 文本编码器 fp4 版反量化崩溃
`mistral_3_small_flux2_fp4_mixed.safetensors` 在 eager 模式下 nvfp4 反量化报 `index out of range in self`（comfy_kitchen 的 bug）。**换 fp8 版** `mistral_3_small_flux2_fp8.safetensors`（18GB，5090 轻松装下）。

### 坑 3：Wan 采样 OOM 是假象 —— cudaMallocAsync 分配器
症状：5090 32GB 空着，Wan 14B fp8（13.6GB）采样时却 OOM。查 PyTorch 内存摘要发现：**Active memory 峰值仅 18.4GB，但 GPU reserved 高达 31.4GB** —— cudaMallocAsync 后端 reserve 后不释放，ComfyUI 的可用显存计算被它骗过。
**解决：启动加 `--disable-cuda-malloc`（回到 PyTorch 原生分配器）。**
配套 `--use-sage-attention` 降低注意力显存。

### 坑 4：UMT5 文本编码器偷占 13GB 显存
`umt5_xxl_fp8` 文件加载时被 cast 成 fp16，常驻 13.4GB 显存，挤压采样空间。
**解决：Wan 工作流里 CLIPLoader 设 `"device": "cpu"`**（文本编码走 CPU，每次约 30-60 秒，换来 13GB 显存余量）。OpenMontage 仓库的 `tools/_comfyui/workflows/wan22-*.json` 已打此补丁。

### 坑 5：SSH 隧道传大文件会断流
OpenMontage 在控制机、ComfyUI 在 5090 时，用 `ssh -N -L 8188:...` 隧道传 API 请求没问题，但**下载生成的 mp4 会中途 IncompleteRead 并把隧道打挂**。
**解决：小流量（提交 prompt、轮询 history、上传图片）走隧道，成品文件用 scp 直接拉。** 或者干脆按第 5 节把 OpenMontage 装到 ComfyUI 同机，彻底绕开。
隧道记得加重连：`while true; do ssh -N -L ... ; sleep 5; done`。

### 坑 6：ssh 远程 pkill 自杀
`ssh host 'pkill -f "main.py"'` 会把远端 bash 自己杀掉（它的命令行里含有 "main.py" 字样，被 pkill 匹配到）。
**解决：`pkill -f "[m]ain.py"`（方括号技巧），或杀进程和起进程分两条 ssh 执行。**

### 坑 7：重启 ComfyUI 后"起不来"
VAST 模板的 supervisor 脚本（`/opt/supervisor-scripts/comfyui.sh`）被杀后不自动重启，需要自己 nohup 拉起；且启动要 ~90 秒，startup 期间 curl 不通是正常的。

### 坑 8：ffmpeg 拼接 mp3 用 `-c copy` 导致时间轴漂移
分段 TTS 拼旁白时 `ffmpeg -f concat -c copy` 会产生 dts 非单调警告，且成品比理论值短 3-4 秒（编码器 padding 被丢），导致音画错位。
**解决：拼接时重编码**：`ffmpeg -f concat -i list.txt -af aresample=48000 -c:a libmp3lame -q:a 3 out.mp3`，并用 ffprobe 实测时长校准时间轴。

### 坑 9：Remotion 渲染长视频在小内存机器上超时
344s×30fps 的合成在 2 核 4GB 机器上 960s 超时。**两个出路**：① 同机部署（第 5 节）；② 改用 FFmpeg 分段合成（视频片段 + 末帧 zoompan + ASS 字幕烧录，本指南验证可行）。

### 坑 10：Remotion 中文字幕被拆成单字
`CaptionOverlay` 默认按空格分词，中文没空格 → 逐字蹦。给 `Explainer` 组件加 `wordSeparator` prop 透传，合成时传 `""`。（补丁在 trojan-odyssey 项目记录里有完整 diff，上游至今未修。）

---

### 坑 11：VAST 宿主机 GPU 驱动整体卡死（租户无法自救）
症状：CUDA 初始化永久 hang，进程进 D 态（`os_acquire_rwlock_read`，开着 /dev/nvidia-uvm），`nvidia-smi` 本身也挂死；容器内 `reboot`/`reboot -f` 均 `Operation not permitted`。
原因：VAST 实例是共享宿主机的容器，别的租户（或自己之前的 GPU 任务）把驱动搞挂后全宿主的 GPU 都不可用。
排查信号：`uptime` 显示多天（=宿主机 uptime 而非实例）、CPU 核心数巨大（512 核）都是共享宿主机的特征。
解决：容器内无解。只能 ① VAST 控制台点重启试试；② 等宿主恢复；③ 销毁换机。**环境配置都在实例磁盘上，换机前能打包就打包**（models/ 120GB 重下约 40min）。

## 7. 验证清单

```bash
# ComfyUI 活着且版本对
curl -s http://127.0.0.1:18188/system_stats | grep comfyui_version   # 0.35.0
# 模型齐全
ls /workspace/ComfyUI/models/diffusion_models  # flux2_dev_fp8mixed + 4 个 wan2.2
ls /workspace/ComfyUI/models/loras             # 4 个 lightx2v
# GPU 启动参数生效
grep -E "sage attention" /root/comfyui_run.log
```

## 7.5 合成阶段的性能分析：GPU 加速值不值

实测背景：333 秒（约 10300 帧）的视频，Remotion 在 2 核 4GB 小机上 960s 超时渲不完；同一片子在 5090 服务器（12 核 62GB）上 CPU 渲染实测 **21 分钟**完成（334.5s 成片，361MB）。

**Remotion 渲染的时间分布（估算）与 GPU 加速空间：**

| 环节 | 占比 | GPU 加速 | 说明 |
|------|------|----------|------|
| Chrome 逐帧渲染 React 场景（Skia 光栅化） | 50-60% | ⚠️ `--gl=angle`/Vulkan 理论 2-4x，无头环境不稳定 | 真正的瓶颈，本质是 CPU 活 |
| ffmpeg 解码素材（OffthreadVideo 取帧） | 20-25% | ✅ NVDEC 硬件解码 | 但本来就不是瓶颈 |
| libx264 编码输出 | 20-30% | ✅ **h264_nvenc 快 5-10 倍** | 最稳的加速项 |

**结论：Remotion + GPU 大约只能从 ~28 分钟压到 12-18 分钟，到不了数量级。**

**真正的数量级提速是换合成器：**

| 路线 | 2 核 4GB VM | 5090 服务器（CPU） | 5090 + nvenc |
|------|------------|-------------------|--------------|
| Remotion（精致字幕/动效） | 渲不完（>16min 超时） | **21 min（实测）** | ~12-18 min（估） |
| FFmpeg（拼接+zoompan+ASS 字幕） | ~8 min（实测） | ~2-3 min | **~1-2 min** |

**选型建议：**
- 内容以「真视频片段 + 静帧 + 字幕」为主（讲解类视频）→ **FFmpeg 路线 + nvenc 作为默认**，速度可忽略
- 需要复杂文字动效/图表/卡片排版 → Remotion，接受它的慢，GPU 只是锦上添花
- 无头服务器上 Chrome 的 "gpu-process" 进程名不代表在用 GPU——Remotion 渲染全程 GPU 0% 是正常现象

**其他小优化：**
- Remotion 渲染超时上限 = `remotion_timeout_ms/1000 + 60` 秒（subprocess 墙钟时间），要渲染长视频得显式传大值（如 2700000），默认公式 `max(600, 场景数×15)` 对长视频不够
- Remotion 首次渲染前确保 Chrome 系统依赖齐全：`libnss3 libnspr4 libatk1.0-0 libatk-bridge2.0-0 libcups2 libdrm2 libxkbcommon0 libxcomposite1 libxdamage1 libxrandr2 libgbm1 libasound2t64`，并用 `npx remotion browser ensure` 装自带的 headless shell

---

## 8. 参考产物

用这套环境产出的成片：`trojan-odyssey-v3.mp4`（5'33"，12 场景全动态开头，零 API 成本）。
生成记录：`./openmontage-特洛伊战争与奥德赛视频生成记录-comfyui5090版.md`

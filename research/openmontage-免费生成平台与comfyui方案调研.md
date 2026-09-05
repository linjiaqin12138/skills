# OpenMontage 免费生成平台与 ComfyUI 本地方案调研

> 调研日期：2026-09-05 ~ 2026-09-06
> 背景：特洛伊战争与奥德赛 AI 验证版中，fal.ai 因无免费额度（`User is locked. Reason: TOP_UP`）不可用，临时改用 Pollinations.ai + edge-tts 零预算方案。本文调研两类替代路径：**有免费额度的第三方图片/视频生成 API**，以及**自建 ComfyUI 服务器（RTX 5090）方案**。
> 结论信息均通过 WebSearch + 官方文档核实，文末标注来源；区分「已核实」与「未能核实」。

---

## 一、结论速览

| 角色 | 首选 | 备选 |
|------|------|------|
| 图片生成（每日可持续免费） | **Cloudflare Workers AI** `flux-1-schnell`（10k neurons/天，无需信用卡） | ModelScope 魔搭（2000 次/天）、Gemini 免费层、Pollinations（无需 key，有水印） |
| 视频生成（免费额度） | **阿里云百炼 DashScope** 通义万相 wan2.6-t2v/i2v（约 30s 免费额度） | ModelScope（视频模型覆盖待实测）、硅基流动赠金（待实测） |
| TTS | 维持 edge-tts（免费） | — |
| **最优长期方案** | **自建 ComfyUI 服务器（RTX 5090 32GB）**，零 API 成本、质量最高 | — |

---

## 二、图片生成 API 调研

### 2.1 Cloudflare Workers AI —— 强烈推荐【已核实】

- **免费额度**：每天 10,000 Neurons，Free/Paid 计划均有，**无需信用卡**，每日 00:00 UTC 重置。FLUX.1-schnell 每张约 5~几十 neurons，一天可出几十到几百张图
- **模型**：`@cf/black-forest-labs/flux-1-schnell`、SDXL、FLUX.2 klein 系列（注意 klein 要求 multipart/form-data）
- **接入**：纯 REST（`https://api.cloudflare.com/client/v4/accounts/{id}/ai/run/{model}`），Bearer Token，**同步返回图片字节**，`requests` 即可
- 来源：<https://developers.cloudflare.com/workers-ai/platform/pricing/>

### 2.2 Google Gemini API / AI Studio —— 推荐【已核实】

- 免费层仍在，但 2026 年图片生成免费配额偏低；**Imagen 4 免费层不可用，仅付费层**
- 官方 `google-genai` Python SDK，API key 在 AI Studio 免费生成（无需信用卡），同步返回
- 注意：免费层数据可能被用于训练
- 来源：<https://www.aifreeapi.com/en/posts/gemini-image-free-tier-2026>

### 2.3 ModelScope 魔搭 —— 强烈推荐（国内）【已核实】

- 每天 **2000 次免费 API 调用**，OpenAI 兼容接口（`https://api-inference.modelscope.cn/v1`，SDK Token 鉴权），支持 FLUX.1、Qwen-Image
- 官方明确不建议高并发/线上商用；视频模型是否覆盖待实测
- 来源：<https://modelscope.cn/headlines/article/795>

### 2.4 硅基流动 SiliconFlow —— 推荐（国内）【已核实】

- 新用户注册送 **2000 万 tokens（约 ¥14）**，部分小模型永久免费；OpenAI 兼容（`api.siliconflow.cn/v1`）
- 平台含图像（FLUX、Qwen-Image）和视频模型，但**赠金能否抵扣图像/视频调用待实测**
- 来源：<https://cloud.tencent.com/developer/news/3726438>

### 2.5 一次性赠金类（用完即止）

| 平台 | 赠金 | 状态 |
|------|------|------|
| Together AI | 注册送 $1–$25 不等（来源不一） | 二手来源，建议实测 |
| Stability AI | 注册送一次性免费 credit（官方文档确认存在，数额未核实） | 已核实存在 |

### 2.6 确认排除

- **Replicate**：2025-07-16 起新账号预付费制，必须先充值（已被 Cloudflare 收购）❌
- **OpenRouter**：免费 `:free` 模型全是文本模型，图片模型全部付费 ❌
- **Recraft / Ideogram / Leonardo / Freepik**：网页端有免费额度，但 **API 均为单独付费计价** ❌
- **Hugging Face Inference Providers**：免费账号每月仅约 $0.10 credits，只能跑几张图，兜底价值低 ❌
- **fal.ai**：无免费额度，需先充值（亲测 `User is locked. Reason: TOP_UP`）❌

### 2.7 Pollinations.ai 现状【已核实】

- 仍免费、无需 key（匿名 15s/次，注册 Seed 层 5s/次），2025-03 起免费层带水印（注册可去）
- **官方 APIDOCS.md 确认：当前公开 API 只有图像/文本/音频，没有视频生成 API**——之前记录中"用 Pollinations 视频 API"的改进点不可行

---

## 三、视频生成 API 调研

免费视频 API 比图片难找得多：

- ❌ **Google Veo 3/3.1**：官方定价页确认仅付费层（$0.15–0.40/秒）；Vertex AI $300 赠金需绑信用卡
- ❌ **Hugging Face** 路由开源视频模型：走每月 $0.10 免费额度，一条视频都跑不完
- ❌ **Replicate / fal.ai**：均无免费层
- ❌ **PiAPI / Eachlabs**：付费，无可用免费层
- ✅ **阿里云百炼（DashScope）——唯一已核实的真免费视频 API**：通义万相 wan2.6-t2v/i2v 新用户免费额度合计约 30 秒视频（仅华北2地域），DashScope Python SDK，异步任务+轮询，需阿里云实名。来源：<https://help.aliyun.com/zh/model-studio/model-pricing>
- ⚠️ **待实测**：ModelScope 每日 2000 次是否覆盖 Wan 视频模型；硅基流动赠金能否抵扣视频调用
- Runway：新账号 125 一次性 credits（带水印），有 API，量太小仅适合验证接入

---

## 四、ComfyUI 本地方案（RTX 5090）

### 4.1 OpenMontage 已内置 ComfyUI 支持，无需从零开发

| 工具 | 文件 | 默认工作流 |
|------|------|-----------|
| `comfyui_image` | `tools/graphics/comfyui_image.py` | FLUX 2 Dev (NVFP4) + Mistral text encoder，文生图 |
| `comfyui_video` | `tools/video/comfyui_video.py` | Wan 2.2 14B fp8 + LightX2V 4 步加速 LoRA，文生视频 / 图生视频 |
| `comfyui_music` | `tools/audio/comfyui_music.py` | ACE-Step v1 文生音乐 |

底层：`tools/_comfyui/client.py`（`POST /prompt` 提交、`/history` 轮询、websocket 监听、`/view` 下载），内置 workflow JSON 模板（`tools/_comfyui/workflows/`）。

关键机制：

- **服务器配置**：默认 `http://localhost:8188`，`COMFYUI_SERVER_URL` 指向远程；图/视频/音乐可用 `COMFYUI_IMAGE_SERVER_URL` 等分别指定不同实例
- **缺模型检测**：返回结构化 `missing_models[]`（文件名、应放目录、下载链接）
- **自定义工作流**：支持 `workflow_json`/`workflow_path` + `output_node`（注意必须导出 API 格式 JSON）
- **断点续等**：长任务超时后用 `resume_prompt_id` 继续等同一 job
- Skill 文档：`.agents/skills/comfyui/SKILL.md`

### 4.2 RTX 5090 适配要点

- 5090 为 Blackwell 架构（sm_120），**必须用 PyTorch nightly + CUDA 12.8+**，稳定版 PyTorch 会报 `no kernel image available`：

  ```bash
  pip install --pre torch torchvision torchaudio --index-url https://download.pytorch.org/whl/nightly/cu128
  ```

- `comfyui_image` 的 best_for 明确支持 Blackwell 硬件
- `comfyui_video` 的 Wan 2.2 14B fp8 是 16GB 显存级路径，5090 的 32GB 很宽裕，可换 fp16 提质量

### 4.3 部署清单

1. 环境：Python venv + PyTorch nightly cu128 + ComfyUI + ComfyUI-Manager
2. 模型（按 `missing_models[]` 提示下载）：
   - 图片：`flux2-dev-nvfp4.safetensors`、`mistral_3_small_flux2_fp4_mixed.safetensors`、`flux2-vae.safetensors`
   - 视频：`wan2.2_{t2v,i2v}_{high,low}_noise_14B_fp8_scaled.safetensors`、`wan_2.1_vae.safetensors`、`umt5_xxl_fp8_e4m3fn_scaled.safetensors`、LightX2V 4 步 LoRA ×2
   - 磁盘预留 100–200GB
3. 国内服务器走 `HF_ENDPOINT=https://hf-mirror.com` 或 ModelScope 下载
4. 设置 `COMFYUI_SERVER_URL` 指向服务器，pipeline 直接调用

---

## 五、建议

1. **有 5090 服务器**：直接走 ComfyUI 方案，图片换 `comfyui_image`（FLUX 2 Dev）、视频用 `comfyui_video`（Wan 2.2），彻底摆脱 API 额度和成本问题，质量也远超 Pollinations
2. **无 GPU 服务器、纯 API 零预算**：Cloudflare Workers AI（图）+ 阿里云百炼 wan2.6（视频，约 30s 免费额度）+ edge-tts
3. **待实测清单**：① Together AI 注册赠金数额；② Stability AI 免费 credit 数额；③ ModelScope 2000 次/天是否覆盖视频模型；④ 硅基流动赠金能否抵扣图像/视频模型

---

## 六、相关文档

- [特洛伊战争与奥德赛 AI 验证版记录](./openmontage-特洛伊战争与奥德赛视频生成记录-ai验证版.md)
- OpenMontage 项目：`/home/jqlin/Project/OpenMontage`（`docs/PROVIDERS.md`、`docs/comfyui-adapter-plan.md`）

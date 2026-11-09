---
name: video-generation
description: 使用 MiniMax H3 等视频模型从首帧或文本生成视频，支持多提供商 token 配置、低成本 480P 测试、异步任务轮询、断点恢复和成片下载。用户要求生成视频、图生视频或测试动态镜头时使用。
---

# Video Generation

当前支持 MiniMax H3 首帧/多模态参考生视频，以及 Ofox Seedance 多参考图生视频。脚本只依赖 Python 标准库。

## 配置

在本 skill 根目录的 `config.json` 填写 token，结构与 image-generation 一致：

```json
{
  "providers": {
    "minimax-plan": {"token": ""},
    "ofoxai": {"token": ""}
  }
}
```

新增 provider 时在 `providers` 中添加成员并实现对应后端。配置文件必须保持 Git 忽略、权限 `600`，不要打印或提交 token。

## MiniMax H3 首帧图生视频

```bash
python3 skills/video-generation/scripts/generate_video.py \
  --provider minimax-plan \
  --model MiniMax-H3-Lite \
  --resolution 480P \
  --duration 4 \
  --first-frame path/to/first-frame.png \
  --prompt "镜头固定，角色做一个简单动作，保持身份和场景不变" \
  --out output/shot.mp4
```

多参考图模式（首帧与参考图互斥）：

```bash
python3 skills/video-generation/scripts/generate_video.py \
  --model MiniMax-H3 \
  --resolution 768P \
  --duration 4 \
  --ratio 9:16 \
  --reference-image path/to/shot-1.png \
  --reference-image path/to/shot-2.png \
  --prompt "0–2 秒镜头一，2–4 秒镜头二，中间硬切，正常速度" \
  --out output/two-shots.mp4
```

Ofox Seedance 多参考图：

```bash
python3 skills/video-generation/scripts/generate_video.py \
  --provider ofoxai \
  --model bytedance/seedance-2.0 \
  --resolution 720p \
  --duration 4 \
  --ratio 9:16 \
  --reference-image path/to/shot-1.png \
  --reference-image path/to/shot-2.png \
  --prompt "两个各 2 秒的镜头，硬切，配同步音效" \
  --out output/seedance-two-shots.mp4
```

可选模型：

- `MiniMax-H3-Lite`：轻量低成本测试，支持 480P/768P，4–15 秒。
- `MiniMax-H3-Highspeed`：速度优先，支持 480P/768P，4–15 秒。
- `MiniMax-H3`：质量优先，支持 768P/2K，4–15 秒。
- `MiniMax-H3-Max`：极速版本，支持 480P/768P，5–15 秒。
- `bytedance/seedance-2.0` 及 Fast/Mini：Ofox，多镜头、多参考图，4–15 秒。
- `bytedance/seedance-2.5`：Ofox，长时序与多镜头，4–30 秒。

`--dry-run` 校验配置和参数，不创建任务。Ofox 默认生成同步音轨，可用 `--no-audio` 关闭；真人参考图需要 `--real-person`。

## 任务恢复与输出

- 创建成功后，任务 ID 保存为 `<输出文件>.task.json`。
- 中断后使用同一命令继续查询，不重复创建任务。
- 成功后自动下载 MP4 并输出 usage。
- 输出文件已存在且非空时直接返回，不重复扣费。
- 角色测试先用 480P、4 秒和每镜一个动作；验收后再提高分辨率。

## 动态镜头验收

1. 实际读取视频，不以任务成功代替质量验收。
2. 检查角色脸型、耳形、服装和场景是否跨帧漂移。
3. 检查手指、爪子、道具接触、按钮按压和嘴部动作。
4. 独立配音与音效通常比直接采用模型原生语音更可控。

官方文档：https://platform.minimax.cn/docs/api-reference/video-generation-v2-create

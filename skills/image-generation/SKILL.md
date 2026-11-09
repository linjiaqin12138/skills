---
name: image-generation
description: 统一调用 MiniMax M Plan 与 OfoxAI 的图像生成和参考图编辑模型，支持 Nano Banana、Seedream、image-01、低清测试、多提供商配置与结果落盘。用户要求生成角色参考图、图像编辑或使用这些提供商时使用。
---

# Image Generation

统一入口：`scripts/generate_image.py`。只依赖 Python 标准库。模型名自动决定 provider，不需要分别记两套命令。

## 配置

在本 skill 根目录的 `config.json` 填写各 provider 的 token：

```json
{
  "providers": {
    "minimax-plan": {"token": ""},
    "ofoxai": {"token": ""}
  }
}
```

以后新增 provider 时在同一对象中添加成员，并补充脚本后端。配置文件必须保持 Git 忽略、权限 `600`。不要打印、提交或在命令行传递 token。

## 常用命令

低成本耄耋角色图草稿：

```bash
python3 skills/image-generation/scripts/generate_image.py \
  --model nano-banana-2 \
  --resolution 512 \
  --reference research/video-generation/assets/maodi-face-reference.png \
  --prompt "保持同一只猫，仅移除背景" \
  --out output/cat.png
```

MiniMax 图生图：

```bash
python3 skills/image-generation/scripts/generate_image.py \
  --model image-01 \
  --reference path/to/reference.png \
  --prompt "同一角色，正面全身，纯色背景" \
  --n 4 \
  --out-dir output/candidates
```

可用模型：

| 模型别名 | Provider | 默认分辨率 |
|---|---|---|
| `image-01` / `image-01-live` | `minimax-plan` | 按 `--aspect-ratio` |
| `nano-banana-2` | `ofoxai` | 512，适合低成本测试 |
| `nano-banana-2.1` | `ofoxai` | 1K |
| `nano-banana-pro` | `ofoxai` | 1K |
| `seedream-5-flash` / `seedream-5-pro` | `ofoxai` | 1024x1024 |

Ofox 每次请求生成一张；多个候选分别调用。MiniMax 可用 `--n 1..9` 和 `--out-dir` 批量生成。`--reference` 可重复使用：Nano Banana 最多 14 张，MiniMax/Seedream 限 1 张。多图时在 prompt 中明确“第一张是角色、第二张是场景”等对应关系。`--dry-run` 校验配置和参数，不发送生成请求。

## 角色参考图流程

1. 从原图裁出清晰、无遮挡、无字幕的单主体画面。
2. 低清阶段优先 `nano-banana-2 --resolution 512`，不要直接用 2K/4K。
3. 提示词采用“保持主体不变，只改背景/构图”的编辑任务，不重新设计角色。
4. 逐张查看脸型、耳形、眼睛、毛色和体型；命令成功不等于身份一致。
5. 选定低清结果后再提高分辨率或生成全身、侧面等扩展参考。

## 接口资料

- MiniMax：https://platform.minimax.cn/docs/guides/image-generation
- Ofox Nano Banana：https://ofox.ai/docs/develop/models/nano-banana
- Ofox Images：https://ofox.ai/docs/api/openai/images

---
name: short-drama
description: 将故事创意制作成多镜头 AI 短剧，支持角色/场景/道具参考包、分镜提示词、独立时长、多供应商、单镜重跑、连续性审核、字幕和配乐。适用于动物、真人、荒诞喜剧、治愈剧情和连续 IP。
---

# Short Drama

把故事拆成可控的镜头任务，用不同模型生成、审核和替换镜头，再合成完整短剧。不要把它当成单集剧情模板。

## 项目结构

```text
projects/<drama-name>/
├── asset-pack/
│   ├── characters/
│   ├── scenes/
│   └── props/
└── episodes/<episode-id>/
    ├── prompts.md
    ├── tasks.json
    ├── shots/
    ├── episode.ass        # 可选
    ├── joined.mp4
    └── final.mp4
```

`asset-pack` 可跨集复用；`prompts.md` 保存中文提示词；`tasks.json` 只保存参数、模型、时长和参考图。探索素材与正式素材分开。

## 工作流程

### 1. 建立资产包

先准备角色正面/侧面/全身、必要服装、主要场景、关键道具和空间关系首图。缺少资产时调用 `skills/image-generation`，不要直接烧视频额度。

### 2. 剧本转分镜

- 每镜一个主要动作，独立设置 `duration`。
- 短镜头可与下一镜合并，写清内部时间段和硬切点。
- 不把接口最短时长当表演时长，不用统一 trim 拖动作。
- 提示词写清人物位置、视线、门的内外与开合、道具形状和首次出现时机。
- 文字、标签和精确字幕时序优先后期制作。

### 3. 审核生产方案

生成前向用户展示剧情、台词、参考图、模型、分辨率、时长和预计费用。用户确认后再执行。

### 4. 分镜生成

任务配置示例：

```json
{
  "provider": "ofoxai",
  "model": "bytedance/seedance-2.5",
  "resolution": "480p",
  "ratio": "9:16",
  "generate_audio": true,
  "prompt_file": "prompts.md",
  "output": {"width": 480, "height": 864},
  "tasks": [
    {
      "id": "01-02",
      "duration": 5,
      "references": ["../asset-pack/characters/hero.png", "../asset-pack/scenes/main.png"]
    },
    {
      "id": "03",
      "duration": 4,
      "model": "alibaba/wan-3.0",
      "references": ["../asset-pack/characters/hero.png"]
    }
  ]
}
```

参考图相对于 episode 目录。任务可用 `model`、`provider_type`、`real_person` 覆盖默认设置。

在 `prompts.md` 中用二级标题和代码块维护每个任务：

````markdown
## 任务 01-02

```text
这里写该任务完整中文提示词。
```
````

### 5. 生成与单镜重跑

```bash
python3 skills/short-drama/scripts/produce_episode.py --project projects/<drama-name>/episodes/episode-01
```

只重跑一个任务：

```bash
python3 skills/short-drama/scripts/produce_episode.py \
  --project projects/<drama-name>/episodes/episode-01 \
  --shot 01-02 --force
```

其它参数：`--jobs 3`、`--dry-run`、`--no-compose`、`--bgm /path/to/music.mp3`。

已存在且非空的镜头会跳过；`--force` 会删除该任务旧视频和任务 ID 后重跑。

### 6. 逐镜验收

- 角色脸型、发型、服装是否漂移；
- 人物位置、视线和首次出现时机；
- 门窗内外、开合方向、道具大小形状和数量；
- 是否静态大头、动作拖长、手指或嘴部变形；
- 字幕是否在对白和动作发生时出现；
- 音轨是否清晰，BGM 是否盖住台词。

只修改有问题任务，再重新合成。

## 合成规则

- 按任务顺序完整拼接，统一为 24fps、H.264、AAC 48kHz。
- 输出尺寸默认 480×864，可在 JSON `output` 中覆盖。
- 存在 `episode.ass` 时烧录字幕。
- `--bgm` 混入音乐并自动在人声时压低；没有 `--bgm` 时保留原音轨。

## 不可跳过的检查

1. 生成成功不等于验收通过，必须实际查看视频。
2. 不强行用一个模型生成全片；真人审核失败时更换路径。
3. 不无限重试同一审核失败。
4. 不把探索用首图自动当正式首图。
5. 单镜重跑优先于整片重跑。

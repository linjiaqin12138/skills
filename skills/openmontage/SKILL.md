---
name: openmontage
description: 用 OpenMontage 工作流制作短视频，按需求选择 Cinematic、Animation 或自定义 short-drama，支持 Ofox Provider、分镜审核、单镜重跑、预算控制、字幕和配乐。用户要求用 OpenMontage 制作视频、短剧或调度多模型生产时使用。
---

# OpenMontage 短视频调度

把 OpenMontage 作为生产治理层，把现有 image-generation、video-generation 和 short-drama 作为执行能力。不要另起一套平行工作流。

## 路由

- 知识讲解：`animated-explainer`
- 情绪短片、剧情、喜剧：`cinematic`
- 动态图形和插画动画：`animation`
- 真人/动物短剧、连续 IP：自定义 `short-drama`
- 库存素材混剪：`documentary-montage`

当前仅 `short-drama` 接入本地生产脚本；其它路由先检查官方 pipeline manifest，再按其 director skills 执行。

## 准备

官方上游放在 `vendor/OpenMontage`，不 fork、不修改官方源码。扩展位于本 skill 的 `provider_extensions/`，通过软链接接入。

```bash
# 首次准备上游
git clone https://github.com/calesthio/OpenMontage.git vendor/OpenMontage
bash skills/openmontage/scripts/link_extensions.sh
python3 skills/openmontage/scripts/preflight.py
```

上游更新后重新运行链接脚本即可。token 放在本 skill 的 `config.json`，结构为 `providers.<provider>.token`，文件需 Git 忽略且权限 600。OpenMontage 原生适配器也可读取 `OFOX_API_KEY`。

## 制作 short-drama

```bash
python3 skills/openmontage/scripts/produce.py \
  --pipeline short-drama \
  --project projects/<drama-name>/episodes/episode-01
```

`--project` 必须包含 `prompts.md` 和 `tasks.json`。生产前向用户展示剧情、提示词、参考图、模型、时长与预算并等待确认。

单镜重跑：

```bash
python3 skills/openmontage/scripts/produce.py \
  --pipeline short-drama \
  --project projects/<drama-name>/episodes/episode-01 \
  --shot 01-02 --force
```

其它参数：`--jobs`、`--dry-run`、`--no-compose`、`--bgm`。

## short-drama 规则

- 中文提示词只在 episode 的 `prompts.md` 维护，JSON 只保存参数。
- 每镜独立时长；短镜头可合并，明确内部硬切点，不用统一 trim。
- 角色、场景、道具多参考图；探索素材和正式素材分开。
- 每个任务可覆盖模型与上游通道，不强迫一个模型完成全片。
- 真人审核失败时切换路径，不无限重试。
- 单镜重跑优先于整片重跑。
- 生成后必须实际查看门方向、角色出现时机、道具形状、视线和字幕时机。

## 文件职责

- `pipeline_defs/short-drama.yaml`：阶段与审批规则。
- `provider_extensions/tools/video/ofox_video.py`：OpenMontage BaseTool 适配器。
- `provider_extensions/skills/pipelines/short-drama/SKILL.md`：director 指南。
- `scripts/preflight.py`：上游、链接、脚本和 token 预检。
- `scripts/produce.py`：pipeline 路由与生产。
- `scripts/link_extensions.sh`：幂等创建扩展软链接。

参考：`references/production-rules.md`。

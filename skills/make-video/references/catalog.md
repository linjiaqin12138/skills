# 视频 skill：上游与 prepare

工作目录：仓库根。目标已有 `vendor/<id>/SKILL.md` 则跳过 prepare。

## 通用 sparse 模板

```bash
NAME=<id>
REPO=<owner/repo>
SKILL_PATH=<path/inside/repo>
DEST=vendor/$NAME
rm -rf /tmp/skill-prep-$NAME
git clone --depth 1 --filter=blob:none --sparse "https://github.com/$REPO.git" /tmp/skill-prep-$NAME
git -C /tmp/skill-prep-$NAME sparse-checkout set "$SKILL_PATH"
mkdir -p "$DEST"
cp -a "/tmp/skill-prep-$NAME/$SKILL_PATH/." "$DEST/"
rm -rf /tmp/skill-prep-$NAME
```

根目录即 skill 时改为浅克隆整仓：

```bash
git clone --depth 1 "https://github.com/$REPO.git" "vendor/$NAME"
```

可选挂载：`ln -sfn "../../vendor/$NAME" ".claude/skills/$NAME"`

`VENDOR.md` 最小内容：上游 URL、许可、收录方式、日期、本地改动（无 / PATCHES.md）。

---

### manim-teaching-figures

- 上游：https://github.com/yangwhale/CloseCrab → `skills/manim-teaching-figures`
- 许可：以仓库为准
- prepare：`NAME=manim-teaching-figures` `REPO=yangwhale/CloseCrab` `SKILL_PATH=skills/manim-teaching-figures`
- 备选：`npx -y skills add yangwhale/CloseCrab --skill manim-teaching-figures --agent claude-code` 后把目录拷进 `vendor/manim-teaching-figures/`

### manim-video

- 上游（择一，写入 VENDOR.md）：
  - https://github.com/browser-use/video-use → `skills/manim-video`（优先）
  - https://github.com/NousResearch/hermes-agent → `skills/creative/manim-video`
- prepare：sparse 到 `vendor/manim-video/`

### claude2video

- 上游：https://github.com/likefallwind/claude2video
- prepare：浅克隆整仓到 `vendor/claude2video/`（若 skill 在子目录则 sparse）

### manim-skill

- 上游：https://github.com/awesome-skills/manim-skill
- prepare：浅克隆到 `vendor/manim-skill/`

### recursive-math-animator

- 上游：https://github.com/Science-Prof-Robot/recursive-math-animator
- prepare：浅克隆到 `vendor/recursive-math-animator/`（目录名以仓内 `SKILL.md` 所在为准）

---

## 降级顺序

1. manim-video / claude2video → manim-teaching-figures（缩短为无声 loop）
2. manim-teaching-figures → 退回 explain-it：`courseware-builder` 或分帧静图
3. recursive-math-animator / manim-skill 失败 → manim-video 或 manim-teaching-figures
4. 任一失败且网上有现成片 → 给 B 站/YouTube 链接

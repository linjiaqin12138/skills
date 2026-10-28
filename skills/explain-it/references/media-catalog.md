# 媒介 → skill 目录与 prepare

本文件只在 explain-it 第三步选中某个 `vendor/*` skill，或需要查上游地址时打开。

工作目录：仓库根（含 `skills/`、`vendor/`、`.claude/skills/` 的那一层）。

## 已在仓库内（一般不用 prepare）

| id | 路径 | 用途 |
|----|------|------|
| draw-diagram | `skills/draw-diagram/SKILL.md` | 静图路由 |
| concept-writer | `skills/concept-writer/SKILL.md` | 落盘概念页 |
| answer-me-with-html | `vendor/answer-me-with-html/SKILL.md` | 交互讲解页；CLI：`bin/am-render` |
| d2-diagrams | `skills/d2-diagrams/SKILL.md` | 简单图（经 draw-diagram 选用） |

## 第三方：缺则 prepare 到 vendor/

下列命令在**目标目录尚不存在**时执行。已存在且有 `SKILL.md` 则跳过。每条成功后写 `vendor/<name>/VENDOR.md`。

通用模板（sparse clone 单 skill 目录）：

```bash
NAME=<name>
REPO=<owner/repo>
SKILL_PATH=<path/inside/repo>   # 仓库内含 SKILL.md 的目录
DEST=vendor/$NAME
rm -rf /tmp/skill-prep-$NAME
git clone --depth 1 --filter=blob:none --sparse "https://github.com/$REPO.git" /tmp/skill-prep-$NAME
git -C /tmp/skill-prep-$NAME sparse-checkout set "$SKILL_PATH"
mkdir -p "$DEST"
# 若 SKILL 就在仓库根：cp -a /tmp/skill-prep-$NAME/. "$DEST/"
# 若在子目录：
cp -a "/tmp/skill-prep-$NAME/$SKILL_PATH/." "$DEST/"
rm -rf /tmp/skill-prep-$NAME
```

整仓本身就是一个 skill（根目录有 `SKILL.md`）时，用浅克隆整仓到 `vendor/<name>/` 即可，不必 sparse。

可选挂载：

```bash
mkdir -p .claude/skills
ln -sfn "../../vendor/$NAME" ".claude/skills/$NAME"
```

### courseware-builder — 分步演示 HTML

- 上游：https://github.com/nasrulhazim/claude （路径 `skills/courseware-builder`）
- 许可：以仓库为准，prepare 时核对 LICENSE
- prepare：

```bash
NAME=courseware-builder
REPO=nasrulhazim/claude
SKILL_PATH=skills/courseware-builder
# …按上面 sparse 模板…
```

适用：状态逐步变化、带 Prev/Next/Simulate 的 walkthrough（内存映射、握手、流水线步骤）。

### dashmotion — 架构流动画

- 上游：https://github.com/csthink/dashmotion
- 许可：MIT
- prepare：整仓浅克隆到 `vendor/dashmotion/`（根目录即 skill）

```bash
git clone --depth 1 https://github.com/csthink/dashmotion.git vendor/dashmotion
```

或：`npx skills add csthink/dashmotion` 后，把装到本机 skills 目录的副本**挪/拷**进 `vendor/dashmotion/`（以仓库为唯一来源）。

适用：拓扑上请求在飞、连接线流动；不适合「第 N 步页表内容变成什么」。

### manim-teaching-figures — 循环教学动画

- 上游：https://github.com/yangwhale/CloseCrab （路径 `skills/manim-teaching-figures`）
- 许可：以仓库为准
- prepare：

```bash
NAME=manim-teaching-figures
REPO=yangwhale/CloseCrab
SKILL_PATH=skills/manim-teaching-figures
# …sparse 模板…
```

备选安装探测（装完仍要落到 vendor）：

```bash
npx -y skills add yangwhale/CloseCrab --skill manim-teaching-figures --agent claude-code
# 再把得到的 skill 目录复制到 vendor/manim-teaching-figures/
```

运行时：按其 SKILL 要求的 Manim CE / LaTeX；缺 sudo 包时停问用户。

适用：变换、轨迹、场、KV cache 时间演化等；产出无声 loop mp4 + 页面嵌入规范。

### manim-video — 讲解视频流水线

- 上游（择一，prepare 时写进 VENDOR.md）：
  - https://github.com/browser-use/video-use （路径 `skills/manim-video`）
  - 或 https://github.com/NousResearch/hermes-agent （路径 `skills/creative/manim-video`）
- prepare：sparse 对应路径到 `vendor/manim-video/`

适用：用户明确要旁白/长叙事讲解片。短循环嵌课件优先 manim-teaching-figures。

### 其它可选用（按需列入 VENDOR，不默认）

| id | 上游 | 何时 |
|----|------|------|
| manim-skill | https://github.com/awesome-skills/manim-skill | 要通用 Manim playbook、不强制教学 figure 房规 |
| interactive-diagram | https://github.com/LizardLiang/interactive-diagram | 可点可缩放的架构/时序图，非分步状态演示 |
| claude2video | https://github.com/likefallwind/claude2video | 大纲 → 多场景旁白视频 |

## VENDOR.md 最小模板

```markdown
# <name>（vendored <date>）

- 上游：<url>
- 许可：<spdx 或见 LICENSE>
- 收录方式：<sparse clone 路径 / 浅克隆 / 从 npx skills 拷入>
- 本地改动：无 | 见 PATCHES.md
```

## 降级顺序

目标 skill prepare 失败时，按媒介降级，不要空转：

1. manim-video → manim-teaching-figures → courseware-builder
2. dashmotion → draw-diagram（静图）或 answer-me-with-html
3. courseware-builder → answer-me-with-html → 分帧静图（draw-diagram）→ 文字
4. 任一动画 → 搜现成 B 站/YouTube 链接（第二步 §0）

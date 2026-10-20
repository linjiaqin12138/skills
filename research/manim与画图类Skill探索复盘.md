# manim / 画图类 Skill 探索复盘

> 日期：2026-10-14  
> 目的：评估「生动讲数学公式」与「OS 内存布局抽象示意图」两类能力，各用独立 agent 探一个 Skill。  
> 试材：`microduck` M8 BAM 积分/转动惯量；`linux-0.11` M1 线性 framebuffer。  
> 产物目录：`.tmp-skill-explore/`（含各 Skill 的 `REPORT.md` 与成品）。

## 一览

| Skill | 任务 | 结果 | 主产物 |
|-------|------|------|--------|
| manim-composer | 公式讲解分镜 | ✅ | `manim-composer/scenes.md` |
| manim-video | 公式讲解成片 | ✅（装齐依赖后） | `manim-video/final.mp4`（~59s） |
| Felo Video | 文档→讲解视频 | ❌ blocked | `felo-video/BLOCKED.md` |
| diagram-design | framebuffer 示意图 | ✅ | `diagram-design/framebuffer-layout.html` |
| fireworks-tech-graph | 同上 | ✅ | `fireworks-tech-graph/framebuffer-layout.{html,png,svg,json}` |

结论先说：

- **讲公式**：composer 管「怎么演」，video 管「渲出来」；成片有 aha，但本机依赖重。  
- **内存图**：diagram-design 更像讲义插图；fireworks 更像可校验的工程蓝图。两者对本题都够用。  
- **Felo Video**：Web 产品（`felo.ai/video`），**不是**本地 `felo-skills` 里的同名 skill；无 key / 无文档→视频 CLI 时不要硬凑脚本包。

---

## 怎么装

本次没有写进 Cursor 全局 skills，而是 **shallow clone 到工作区** 做一次性探索（便于对照源码与装依赖）：

```bash
cd /home/jqlin/Project/skills/.tmp-skill-explore

git clone --depth 1 https://github.com/browser-use/video-use.git
# → video-use/skills/manim-video/

git clone --depth 1 https://github.com/adithya-s-k/manim_skill.git
# → manim_skill/skills/manim-composer/

git clone --depth 1 https://github.com/cathrynlavery/diagram-design.git diagram-design-repo
# → diagram-design-repo/skills/diagram-design/

git clone --depth 1 https://github.com/yizhiyanhua-ai/fireworks-tech-graph.git fireworks-tech-graph-repo

git clone --depth 1 https://github.com/Felo-Inc/felo-skills.git
# 注意：包里有 slides/search 等，没有 felo-video
```

若要装进 Cursor / Claude Code（官方索引常见写法，未在本机全局落地）：

```bash
# 示例（需本机有对应 CLI / epx）
# epx add browser-use/video-use --skill manim-video
# epx add adithya-s-k/manim_skill --skill manim-composer
# npx skills add cathrynlavery/diagram-design
```

本仓库已有路由说明：`skills/draw-diagram/SKILL.md`（架构图默认 fireworks；序列图 mermaid）。

### manim-video 系统依赖（WSL/Ubuntu，实踩）

只 `pip install manim` 不够，按顺序：

```bash
# 1) 渲染/拼接/公式
sudo apt-get update
sudo apt-get install -y ffmpeg \
  texlive-latex-base texlive-latex-recommended \
  texlive-fonts-recommended texlive-latex-extra \
  pkg-config libcairo2-dev python3-dev libpango1.0-dev

# 2) Python 包（清华源曾 SSL/超时；阿里云可用）
cd .tmp-skill-explore/manim-video
python3 -m venv .venv && source .venv/bin/activate
pip install -U pip
pip install -i https://mirrors.aliyun.com/pypi/simple \
  --trusted-host mirrors.aliyun.com manim

# 3) 验
manim --version    # 实测 Manim Community v0.22.0
pdflatex --version | head -1
ffmpeg -version | head -1
```

缺 `pkg-config` / `libcairo2-dev` 时，`pycairo` meson 报 `Dependency lookup for cairo ... failed`。

### Felo

- API Key：[felo.ai → Settings → API Keys](https://felo.ai/en/settings/api-keys)（只显示一次）→ `export FELO_API_KEY=...` 或 `felo config set FELO_API_KEY ...`  
- **文档讲解成片**：Web [felo.ai/video](https://felo.ai/video)（贴 URL / 上传文件）  
- OpenAPI 的 `POST /api/v1/videos/generations` 是 **Model API 短视频**，≠ Felo Video 文档→讲解片；勿混用。

---

## 怎么用（按场景）

### A. 生动讲公式（推荐链路）

1. **manim-composer**：读主题 → 产出 `scenes.md`（Overview、逐场 Visual/Content/Narration/Technical Notes、调色板）。**不写代码、不渲染。**  
2. **manim-video**：按 `plan.md` / `scenes.md` → `script.py`（一场一类）→ `manim -ql script.py Scene1 ...` → `ffmpeg` concat → `final.mp4`。

本次数材核心式：

\[
I\ddot\theta=\tau,\quad
\omega(T)=\omega_0+\int_0^T\frac{\tau}{I}\,dt,\quad
\theta(T)=\theta_0+\int_0^T\omega\,dt
\]

半隐式一步：先 \(\omega\leftarrow\omega+(\tau/I)\Delta t\)，再 \(\theta\leftarrow\theta+\omega_{\mathrm{new}}\Delta t\)。

成片草稿三场约 59s：惯量轻重杆对比 → 积分堆叠 → 半隐式两步高亮。

短平快可跳过 composer，直接 video；复杂叙事先 composer 再落地。

### B. OS 内存布局抽象图

主题要点：`offset = y * pitch + x * (bpp/8)`；pitch 常含对齐，勿用 `y*width*4`。

| 工具 | 用法概要 | 何时选 |
|------|----------|--------|
| **diagram-design** | 读 SKILL → 选视觉类型（本题 Layer stack）→ Taste Gate → 独立 HTML（内联 SVG） | 讲义风、概念心智模型、中文旁注 |
| **fireworks-tech-graph** | 写 JSON → `validate` → `render` → `check` → `export-html`；PNG 另栅格化 | 要几何校验、可迭代改坐标、工程拓扑感 |

本仓库 fireworks 习惯命令（路径以 clone 为准）：

```bash
python3 fireworks-tech-graph-repo/scripts/fireworks.py validate|render|check|export-html 输入.json
```

### C. Felo Video（未跑通）

产品路径：登录 → 打开 Video → 上传 Markdown/PDF/URL。  
Agent 路径：需 `FELO_API_KEY` **且** 确认存在「文档→视频」开放 API/CLI；仅有 slides skill 不能顶替。  
给「知识研究」试提示时，提示词必须 **自洽**，不要默认对方读过你的仿真代码。

---

## 踩坑清单

### manim / 渲染

1. **无 sudo**：agent 装不了 apt 包，应停下让用户执行，不要 Text 糊弄成片。  
2. **镜像**：`pypi.tuna.tsinghua.edu.cn` SSL/超时；换阿里云或官方源。  
3. **pycairo**：缺 cairo 开发包与 pkg-config。  
4. **`MathTex` 不能塞中文**：`I=小` 会 Unicode LaTeX error；中文用 `Text`，公式用纯 TeX。  
5. **字体**：`fonts/msyh.ttc` 路径不被 Pango 当字体族；本机可用 `Droid Sans Fallback`（`fc-list :lang=zh`）。Skill 建议英文用等宽字体。  
6. **草稿质量**：`-ql` 即 480p15，够评估叙事；定稿再 `-qh`。

### Felo

1. **缺 key 就停**，不要交付「可上传脚本包」冒充已验证。  
2. `felo-skills` ≠ Felo Video；Model API video generations ≠ 文档讲解成片。  
3. 无 drop-in 换 key 的等效 provider；HeyGen / NotebookLM 等能力接近但协议不同，要改适配代码。

### diagram-design

1. 无专用「memory map」类型，pitch 填充要在 Layer stack 里手绘。  
2. 偏版式插图，不是模块拓扑流水线。

### fireworks-tech-graph

1. `check` 对短边中文标签、跨层弯折预算很严——好事，逼结构收敛。  
2. 本机无 `cairosvg` 时 PNG 需另找栅格化（如 resvg）并喂 CJK 字体，否则方块。  
3. architecture 盒子隐喻下「真像素网格」感偏弱；抽象三层叙事够用。

### 流程纪律（本次约定）

依赖不满足（API key、manim、ffmpeg、LaTeX）时 **停并写 BLOCKED**，不盲目降级硬撑——否则对比结论失真。

### 补充（2026-10-20）：不装 manim 也能出片

上面说 manim 链路「依赖重」——但真正想解决的问题只是「把一张静态图讲成动画」。本机实测了一条**零 LaTeX、零 GPU、零 pip 包**的路径：

1. 动画写成一个自包含 HTML（`<canvas>` + 手写投影，页面暴露 `window.seek(t)`：给定秒数立刻画出该帧，纯函数式，不依赖 rAF 时钟 → 逐帧抓图是确定性、可复现的）。
2. 用 headless Chromium 的 **CDP**（`Target.attachToTarget` + `Runtime.evaluate` + `Page.captureScreenshot`）逐帧截图；Chrome 二进制可从 `~/.cache/ms-playwright/chromium-*/chrome-linux64/chrome` 或本仓库 `vendor/browsers/` 取。
3. `ffmpeg -framerate 30 -i f%05d.jpg -c:v libx264 -pix_fmt yuv420p` 合成 mp4。

实测：1600×900 / 30fps / 92 秒 = 2760 帧，**CPU 单进程约 4 分钟**，产物 3.3 MB。中文用 `Noto Sans CJK SC` 直接 `fillText`，不用 LaTeX。

脚本原型（页面约定 `window.seek(t)` + 可选 `window.__duration`；`--fps/--width/--height/--duration/--keep-frames`）**已随当时的产物一并删除**——需要时按上面三步重写即可，单文件约 200 行。

**取舍**：manim 强在「数学公式变形动画」（LaTeX 排版、方程逐项变形）；canvas 路径强在「自定义 3D 场景 + 中文标注 + 精确数值联动」，且不绑 LaTeX 生态。讲惯量张量这种「矩阵 + 坐标轴 + 真实数值」的题材，canvas 路径更省事。

**踩坑**：`MathTex` 塞中文会炸（见上）；canvas 路径里 `dot3` 这类函数名容易和向量点积重名，ES module 之前会直接 `Identifier already declared` 白屏——写完先用 `node --check` 过一遍。

---

## 对比与选用建议

| 需求 | 首选 | 备选 / 备注 |
|------|------|-------------|
| 公式直觉动画 | manim-video | 先 manim-composer 打磨分镜 |
| 只要导演脚本 | manim-composer | — |
| 文档一键口播片 | Felo Video（Web） | 需账号；本地 skill 未覆盖 |
| OS 概念示意图（讲义） | diagram-design | — |
| 架构/模块图 + 几何验收 | fireworks-tech-graph | 本仓库 `draw-diagram` 默认路由 |
| 序列/时序 | mermaid（见 draw-diagram） | 勿强上 fireworks |

**对「读 OS 代码要在脑中有内存画面」**：diagram-design 与 fireworks 都够；前者打开 HTML 更像教材，后者 JSON 可回归修改。  
**对「生动讲 \(I\)、积分、半隐式」**：manim 链路已验证；Felo 更适合「已有长文档要传播」，不擅长精确几何推导。

---

## 产物索引

```
.tmp-skill-explore/
  TOPIC-math-inertia.md
  TOPIC-framebuffer-layout.md
  manim-composer/{scenes.md,REPORT.md}
  manim-video/{plan.md,script.py,final.mp4,REPORT.md,BLOCKED.md}
  diagram-design/{framebuffer-layout.html,REPORT.md}
  fireworks-tech-graph/{framebuffer-layout.html,.png,.svg,.json,REPORT.md}
  felo-video/BLOCKED.md
  video-use/ … manim_skill/ … diagram-design-repo/ … fireworks-tech-graph-repo/ … felo-skills/
```

## 参考链接

- manim-video：https://github.com/browser-use/video-use/tree/main/skills/manim-video  
- manim-composer：https://github.com/adithya-s-k/manim_skill  
- diagram-design：https://github.com/cathrynlavery/diagram-design  
- fireworks-tech-graph：https://github.com/yizhiyanhua-ai/fireworks-tech-graph  
- Felo Video：https://felo.ai/video ；API Keys：https://felo.ai/en/settings/api-keys  
- Felo Auth：https://openapi.felo.ai/docs/authentication  

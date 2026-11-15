---
name: md-html-ppt
description: >-
  把长 Markdown / Skill 规范压成可翻页的 HTML 幻灯片（Marp），方便扫读与 review。
  提炼精华要点，流程/决策/对比用 Mermaid 图；不改源文件。
  在用户说「做成 HTML PPT」「做成幻灯片 review」「md 太长看不下去」「用 md-html-ppt」
  或显式 @/call 本 skill 时使用。
---

# md-html-ppt：Markdown → HTML 幻灯片（扫读 / review）

目标：把难扫的长 md（尤其 skill 规范）压成**一页一要点**的本地 HTML PPT，带关键关系图，方便人 review。源 md 只读，不改写正文。

## 何时用

- 用户要 review / 扫读某份或某个目录下的 Markdown
- 明确提到 HTML PPT、幻灯片、Marp、`md-html-ppt`

不要用：从零做对外演讲 deck、美化主题大会（那是另一类需求）。

## 输入

1. 用户点名的文件或目录；未点名则问一句，默认当前对话里正在讨论的 skill/`docs`。
2. **默认只读主文件**（如 `SKILL.md`）；`references/`、`templates/` 仅当用户点名，或主文件反复跳转到某篇且缺了就无法理解时，才抽 1～3 个关键引用进 deck。
3. 不要把整个多文件 skill 合成上百页。

## 输出

写到临时目录（优先 `/tmp/md-html-ppt/<slug>/`），**不要**写进 skill 源目录，**不要**提交：

| 文件 | 用途 |
|------|------|
| `deck.md` | Marp 源（可再改） |
| `deck.html` | 浏览器打开的 HTML PPT |

完成后用绝对路径告诉用户，并建议 `xdg-open deck.html`（或等价方式）打开。

## 工作流

```
Task Progress:
- [ ] 1. 确认输入范围（主文件 ± 点名引用）
- [ ] 2. 读源 md，列出将做成幻灯片的大纲（标题级）
- [ ] 3. 写 deck.md（有损压缩 + Mermaid）
- [ ] 4. 渲染 deck.html
- [ ] 5. 回报路径；大改听用户反馈再迭代 deck.md
```

### 1–2. 读与取舍

- 忠实保留：**规则措辞、闸门条件、分档标准、禁令、错误码/契约名**——可缩短，不可改义。
- 可删/可压：冗长例子、重复说明、模板全文、changelog 流水。
- 每页脚注一行溯源：`来源: <相对路径>#<原标题>`，方便跳回原文。

### 3. 写 `deck.md`

Frontmatter 固定：

```markdown
---
marp: true
theme: default
paginate: true
size: 16:9
---
```

分页：`---` 分隔。结构建议：

1. **封面**：材料名 + 一句话用途 + 日期
2. **地图**：整份材料结构（3～7 条或一张总览 Mermaid）
3. **正文页**：一页一个主题
4. **可选收尾**：review 时要注意的坑 / 未展开的 references 列表

版式约束（ADHD / 扫读友好）：

- 标题短；每页要点 **≤5** 条；每条一行能扫完
- 禁止整段粘贴原文；禁止一页塞两个无关主题
- 流程、分档决策、状态、对比、依赖方向 → 用 **Mermaid**；图承担关系，文字只写约束与边界
- **图必须写成 HTML 块**（Marp 默认不会渲染 \`\`\`mermaid fence）。在启用 `--html` 的前提下写：

```html
<div class="mermaid">
flowchart LR
  A[输入] --> B{闸门}
</div>
```

- 对比矩阵可保留极简表，或改成决策流程图
- 中文引号优先，避免弄坏 Mermaid 节点文本
- 不写「本文由 AI 生成」「如何打开」等元说明页

体量参考：单份 `SKILL.md` 约 **25～40 页**；明显超了就再压，不要靠缩小字号硬塞。

### 4. 渲染

用本 skill 自带脚本（Marp + 注入 Mermaid CDN，使 `div.mermaid` 可显示）：

```bash
# 相对本 skill 根目录
bash scripts/render.sh /tmp/md-html-ppt/<slug>/deck.md /tmp/md-html-ppt/<slug>/deck.html
```

脚本会找 `marp`（或 `~/.local/node/bin/marp`，或回退 `npx @marp-team/marp-cli`）。

- 浏览器打开 `deck.html`，方向键翻页；图需能访问 jsDelivr（默认有网）。
- 渲染失败先修 `deck.md`（常见：Mermaid 引号、`<div class="mermaid">` 未闭合），再跑；不要改用在线转换（隐私）。

### 5. 交付

只告诉用户：

- `deck.html` 绝对路径（主产物）
- `deck.md` 路径（要改字/加页时改这个再渲染）
- 压缩了什么、刻意没进幻灯片的 references（若有）

## 禁止

- 改源 Markdown / 把 deck 提交进 skill 仓库
- 上传正文到在线 PPT/PDF 服务
- 为「好看」大段改写规则语义
- 把 `md2html` 那种单页长文 HTML 冒充本 skill 的产出

## Marp 速查

```markdown
---
marp: true
theme: default
paginate: true
size: 16:9
---

# 标题页

副标题

---

## 一页一个主题

- 要点一
- 要点二

来源: skills/foo/SKILL.md#某节

---

## 流程

<div class="mermaid">
flowchart LR
  A[输入] --> B{闸门}
  B -->|过| C[落地]
  B -->|停| D[对齐]
</div>
```

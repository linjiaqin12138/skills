---
name: draw-diagram
description: 画图路由器——根据图的类型选择最合适的画图工具。序列图/流程图用 mermaid（嵌 markdown，不渲染）；模块关系图/架构图默认用 fireworks-tech-graph（手动坐标、可控、文本化几何校验）；简单图可用 d2，d2 自动布局调整两轮仍不合格就切 fireworks。当需要画任何图时使用。
---

# 画图路由器

先判断图的类型，再选工具。选型错了的代价（自动布局洗牌、反复截图烧 token）远大于多花 10 秒判断。

## 路由规则

| 图的类型 | 工具 | 理由 |
|---|---|---|
| 序列图、时序交互、状态机、简单流程图 | **mermaid**，直接嵌 markdown，不单独渲染 | 语义即布局，几乎没有排版自由度，也就不存在布局翻车 |
| 模块关系图、架构图（容器框 + 色块模块 + 带标签的边） | 默认 **fireworks-tech-graph**；图很简单（≤6 模块、无嵌套容器）可用 d2 | 见下「为什么架构图默认 fireworks」 |
| 数据图表（柱状/折线/散点） | 不用本 skill，用绘图库 | — |

**升级规则**：选了 d2 但自动布局连续调整 **2 轮仍不合格**（边穿框、标签压字、跨容器边绕远路），立即切 fireworks，不要继续跟布局引擎搏斗。预估"模块多、关系杂、有嵌套容器"时跳过 d2 直接 fireworks。

## 为什么架构图默认 fireworks（2026-10-03 实测结论）

- **手动坐标**：模块位置写死在 JSON 里，改一处绝不动全图；d2 的自动布局修一处会把全图重新洗牌
- **文本化几何校验**：`check` 命令输出碰撞/截断/交叉的文本报告（精确到元素 ID），迭代不用把截图读回模型；d2 只能渲染成图亲眼看，每张截图几千 token
- **成对图像素对齐**：同一 JSON 复制改色即得"变动对比图"，d2 做不到

## fireworks-tech-graph 用法（本机已验证可用）

- skill 目录：`skills/fireworks-tech-graph/`，输入格式见 `schemas/diagram-v1.schema.json`，风格见 `references/style-*.md`（教程文档推荐 Style 1 或 4，浅色）
- 命令：`python3 skills/fireworks-tech-graph/scripts/fireworks.py validate|render|check|export-html <输入.json>`
- 工作流：写 JSON → `validate` → `render` → `check` 拿文本报告修几何问题 → `export-html`（可缩放拖拽的离线查看版）
- **定稿时才目检一次**：cairosvg 转 PNG 再 ReadMediaFile。注意 cairosvg 不吃 SVG 的字体回退列表，中文会成方块——转换前先把 SVG 里 `font-family: ...` 整串替换为 `font-family: 'Noto Sans CJK SC', sans-serif`（本机已装该字体与 cairosvg）：
  ```bash
  sed "s/font-family: [^;\"]*/font-family: 'Noto Sans CJK SC', sans-serif/g" in.svg > in-cjk.svg
  python3 -c "import cairosvg; cairosvg.svg2png(url='in-cjk.svg', write_to='out.png', scale=2.0)"
  ```
- **不要用 firefox headless 截图**：本机 snap 版会卡死
- PNG 是给模型目检/贴文档用的；交付给人的主格式是 SVG + HTML

## d2 用法（仅限简单图）

- skill 目录：`skills/d2-diagrams/SKILL.md`，二进制在 `bin/d2`（v0.7.1，ELK 引擎；TALA 未装）
- 流程仍是"渲染→看图→修"，但记住上面的 2 轮升级规则

## 成本控制（所有工具通用）

- 迭代期靠文本反馈（fireworks 的 check 报告 / d2 的错误输出），**每张图只在定稿时目检一次**
- 已有同系列图时基于上一张增量改，禁止每轮从零探索布局

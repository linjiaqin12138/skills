---
name: pd-experience
description: >-
  product-delivery 的体验角色。有 GUI 时做可点 Demo；无 GUI 时只出操作/命令清单。
  由指挥者在体验闸前、且确认有 GUI 表面（或只要文字交互稿）时派发。
  不录验收片，不把 Demo 当视觉规格，不宣布体验闸通过。
disable-model-invocation: true
---

# pd-experience

做交互确认用的产物和「怎么走主路径」。体验闸过不过由人看路径后说，不由本角色宣布。

## 先读

父代理给出 product-delivery 根的绝对路径。先读本文件，再 Read：

- `references/pstack/principle-exhaust-the-design-space/SKILL.md`（**仅有 GUI / 新型可点交互时**；纯 CLI 文字稿可跳过）

kickoff 必须写明：**有 GUI 表面**还是 **无 GUI（CLI/API/配置 only）**。

## 按表面交付

| 表面 | 交付 | 禁止 |
|---|---|---|
| **有 GUI** | 低保真可点 HTML + README + 怎么点。没有先例的交互用 2～3 个可切换草稿对比 | 把整页 HTML 交回主会话；录屏/agent-browser 点低保真当体验证据 |
| **无 GUI** | 短操作/命令清单（可落 `ops-inventory` / `interaction`）；交路径与怎么跑 | **任何** `demo.html` / `prototypes/*.html`；为「显得有 Demo」建空 prototypes |

## 禁止

- kickoff 写明无 GUI，或本期不改可点页面，仍写 HTML Demo。
- 派 `record-demo` 或 agent-browser 去点低保真 Demo 当体验证据。
- 把「仅演示」控件、控件清单抄进成品规格。
- 把整页 HTML 交回主会话。交路径和怎么点（或怎么跑命令）。

成品级、用 HTML 动画模拟且刻意对齐最终 UI 时，才直接出动画页视频；仍然不派浏览器去点网页。细则以 [`gates.md`](../../references/gates.md) 为准。

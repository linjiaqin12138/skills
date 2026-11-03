# M11 结构来源与许可（2026-11-03）

## 官方来源的访问边界

- `microduck_rl` 的 MJCF / `config_mjcf_*.json` 指向官方 Onshape document `804927696f06d877f3f1803e`。
- 已在登录后的 Edge 中实测 Assembly element `ef6e972847fec8d82570b35e` 与脚底 PartStudio element `451317b8e3aacc7f88e8f8ed`：两者都重定向到 `Owned by me`，当前账号不能读取原始特征树。
- 仓内 `.part` 仅含 document / element / part 身份元数据；STL 是 `simplify_stls: true`、`max_stl_size: 1.0` 的仿真网格。不能把它们冒充原始参数化 CAD。

## 用户批准的替代基准

- 仓库：https://github.com/fanhao375/microduck-replica-cad
- 版本：**飞特 HD-1910 v2.2（2026-10-20）**，release ID `418136980`。
- 作者：机械行者Robo。模型由作者依据官方公开 STL 逆向重建为可编辑 SolidWorks 参数模型；不是官方设计，不保证与官方原始 CAD 完全一致。
- 源包：`SolidWorks-FT2026-10-20.zip`，417,656,282 bytes。
- 发布页声明：77 个 SolidWorks 文件（18 装配体 + 59 零件），另含 10 个 STEP 导出；独立改动件 STEP 包有 13 个文件。
- **解包复核**：发布计数包含 1 个 `~$*.SLDASM` 与 3 个 `~$*.SLDPRT` 临时锁文件；有效文件为 **17 装配体 + 56 零件 + 10 STEP**。17 个装配体中，00 总装 + 01–15 分组件共 16 个核心装配，另 1 个轮滑组件。用于 Onshape 的清理包为 `vendor/microduck-ft-v2_2-onshape.zip`，75 个有效 SolidWorks/DDS 文件，401,984,788 bytes，SHA-256 `66efbff0a8bd78307ba6bd02584918be69aa4959079de0ac2b4cdeef9caed325`。
- SHA-256：`71e5b6a711ec9b4dd7154627e036a5180207088536aa230e42949212a550eaa1`（原始发布包，已实测匹配）。
- 资料仓库快照：`a3ac692b9e874a568b021a9191dcc9ba8c5cea9f`（2026-10-25）。
- 许可：CC BY-NC-SA 4.0。保留机械行者Robo署名，非商业使用，衍生图纸与文档相同方式共享。

## Onshape 网页自动化的可用边界（2026-11-03 实测）

- **可靠**：打开/新建文档、导入文件、右键菜单项、拖拽创建几何（圆、线）、对话框里的 DOM 输入框（例如 Extrude 的 Depth）、截图。
- **不可靠**：Dimension 工具对合成鼠标点击/拖拽无响应（数值输入框渲染在 canvas 上，不是 DOM 输入），约束工具（Equal/Tangent/Concentric）的多选无法用合成按键完成（无 Ctrl 组合键接口）。
- 结论：**精确的约束与尺寸仍需人在浏览器里点**；脚本可以代劳的是"取值、算坐标、建文档、导入、导出、跑检查"。M11 的 CAD 建模课以人机协作为准，agent 负责数字与流程。

## 路线后果

- M11b/M11c 以飞特 HD-1910-C001 结构为准，不混用 XL330 舵盘配合件。
- v2.2 改动：头部上壳散热孔、左右上腿间隙；源包新增 PCB 模型。
- 飞特版有整机实物站立、行走、前滚翻后起身验证；本课程只继承来源证据，不把这些第三方实测记作本项目验收。
- 用户 2026-11-03 决定暂不打印；接口试片、实物测量和整机装配门保留，不标完成或豁免。

# M11b 进度快照（2026-11-03）

## 加固板 01-upper_leg_rigidity_plate（Onshape 文档 microduck-m11b-legs，PartStudio 01-upper_leg_rigidity_plate）

已完成（在 Onshape 里，Sketch 1）：
- 两个让位孔 Ø19，左孔圆心在 Origin，右孔 (35.643, −22)，两个方向尺寸均已标注
- 两个 R11 外圆，右外圆已与右孔同心（Concentric）
- 两条外公切线连接两外圆，切点按解析解给出
- 两段相向弧已 Trim，得到闭合哑铃形外轮廓
- 草图已 Confirm（Features 5 项，Parts 0 —— 尚未拉伸）

待办：
1. **拉伸 2 mm**：选中 Sketch 1 → Extrude → Distance 2 → 绿勾（Extrude 面板的输入框渲染在 canvas 上，合成鼠标事件进不去，需人工点）
2. Measure 复核整体宽高，与官方 58.472 × 45.195 × 2.0 对账（哑铃近似预期 57.643 × 44.0 × 2.0，差 −0.83 / −1.20，已留档）
3. 截图存档到 exercises/m11b-ft-source/ 或 book/m11b-*/
4. 下一件：hip_l / yaw2roll（髋部）

## 已知边界
- 官方外形腰部有外凸弧、两端不是正 R11，本轮用哑铃近似代替，差值已记录，不作 1:1 声明
- 小安装孔与中间镂空待补

## 2026-11-03 收工点（commit 02b961a）

明天从这里开始：
1. 用户删掉 Onshape 里 `microduck-m11b-legs` 的 Sketch 1 + Extrude 1（哑铃近似，作废）
2. 用户按 `parts/01-upper_leg_rigidity_plate.recipe.md` 重画加固板
3. 我出 `hip_l` + `yaw2roll` 的配方（轮廓锚点与孔位已算好，见 parts/*.section.json）
4. 复核：孔径/孔心距/厚度一致 + 外轮廓偏差 ≤1mm，截图存档

遗留决策点：右踝主用 `ankle_right脚踝右1-FT`，变体（`1013…`、`改外面轴承`）待与 3MF 网格量尺寸对照后定案；`unnamed-07/08` 切片失败（退化件）待查。

## 2026-11-03 续作（本会话）

- Bite 1 视为已确认（用户说 continue）。
- 已交付配方：`parts/hip_l.recipe.md`、`parts/yaw2roll.recipe.md`（yaw2roll 勿误用 `99-yaw_roll_motion` 切片）。
- 哑铃版右孔曾标成 (35.643, **−22**)——已纠正为 (35.643, **+22.000**)。

## 加固板验收（2026-11-03 晚）

- 用户按配方重画完成；自报硬指标全过（Ø19×2、孔心距 35.643/22、厚 2、小孔直径与位置、外轮廓可接受）。
- **下一件：`hip_l`**——打开导入参考件 Measure，再按 `parts/hip_l.recipe.md` 新建 PartStudio 参数化重画。
## 浏览器说明（2026-11-03 收工时）

Onshape 的 Edge 窗口已关闭；独立 profile 保留在 `microduck-from-scratch/.edge-onshape/`（258MB，已 gitignore），登录状态在里面。
明天重开（带调试端口，可被 agent 接管）：

```sh
microsoft-edge --user-data-dir=/home/jqlin/Project/skills/microduck-from-scratch/.edge-onshape \
  --remote-debugging-port=9222 'https://cad.onshape.com/documents'
```

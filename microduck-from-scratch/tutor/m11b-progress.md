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

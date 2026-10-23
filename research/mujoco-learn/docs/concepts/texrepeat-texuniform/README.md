# texrepeat / texuniform 直觉示意

本地双击 `index.html`，或：

```bash
xdg-open /home/jqlin/Project/skills/research/mujoco-learn/docs/concepts/texrepeat-texuniform/index.html
# 或
cd /home/jqlin/Project/skills/research/mujoco-learn/docs/concepts/texrepeat-texuniform && python -m http.server 8766
# 浏览器打开 http://127.0.0.1:8766/
```

**示意**：页面用 Canvas 画棋盘格，演示「重复次数」和「是否按世界尺度均匀铺」的视觉差别，**不是** MuJoCo 渲染器逐比特复现。官方说明：[material](https://mujoco.readthedocs.io/en/stable/XMLreference.html#material)。

## 控件 → MuJoCo

| 控件 | 对应 | 直觉 |
|------|------|------|
| repeatX / repeatY | `texrepeat="rx ry"` | 贴图在材质 UV 的 U/V 方向各重复多少次；越大格子越密 |
| texuniform 开关 | `texuniform="true/false"` | 关（默认直觉）：格子更多跟 geom 局部 UV/尺寸走，矩形被拉长时格子可能扯扁；开：更接近按世界尺度均匀铺，大平面上格子大小更一致 |
| 半尺寸 X / Y | 平面 geom `size="hx hy …"` 的半宽/半高 | 用来「拉长地板」，对比 uniform 开/关 |

对照 **step05** 常见写法：

```xml
<material name="grid" texture="grid" texrepeat="2 2" texuniform="true" …/>
```

密格子示例（同文件另一段）：`texrepeat="6 6" texuniform="true"`。

## 预设

| 按钮 | 意图 |
|------|------|
| 教程 2×2 + uniform | 与 step05 默认地板一致的起点 |
| 密格子 6×6 | 只加大 texrepeat，格子明显变密 |
| 扁长地板 + uniform 关 | 验收：格子被拉扁 |
| 扁长地板 + uniform 开 | 验收：格子更接近方形、铺得更匀 |

## 建议亲手试三步

1. 保持正方形地板，只拖 **repeatX/Y**：格子变密/变稀（这是 texrepeat）。
2. 点「扁长地板 + uniform 关」：同一套 2×2 重复，格子被扯成扁长方形。
3. 再点「扁长地板 + uniform 开」：同样扁平面，格子更接近正方形。

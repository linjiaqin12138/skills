# solref / solimp 直觉示意

本地双击 `index.html`，或：

```bash
xdg-open /home/jqlin/Project/skills/research/mujoco-learn/docs/concepts/solref-solimp/index.html
# 或
cd /home/jqlin/Project/skills/research/mujoco-learn/docs/concepts/solref-solimp && python -m http.server 8765
# 浏览器打开 http://127.0.0.1:8765/
```

## 滑块 → MuJoCo

| 控件 | 对应 | 直觉 |
|------|------|------|
| timeconst | `solref[0]` | 越小越硬、回弹越快；越大陷得越深 |
| dampratio | `solref[1]` | &lt;1 易弹，≈1 较稳，&gt;1 肉 |
| d0 / dmax / width | `solimp` 前三项 | 穿透浅偏 d0、深到 width 偏 dmax；d 接近 1 更硬 |

**示意模型**：页面内是简化弹簧–阻尼 + 阻抗曲线，不是 MuJoCo 求解器复现。官方说明：[Solver parameters](https://mujoco.readthedocs.io/en/stable/modeling.html#solver-parameters)。

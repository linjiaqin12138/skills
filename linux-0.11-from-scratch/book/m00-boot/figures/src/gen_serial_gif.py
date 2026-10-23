# -*- coding: utf-8 -*-
"""生成 'hello' 通过 16550 串口发出的逐比特动画 GIF：上=8N1 帧结构逐格点亮，下=UART 波形。"""
import matplotlib
matplotlib.use("Agg")
from matplotlib import font_manager
font_manager.fontManager.addfont("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc")
font_manager.fontManager.addfont("/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc")
import matplotlib.pyplot as plt
plt.rcParams["font.family"] = ["Noto Sans CJK SC", "DejaVu Sans"]
plt.rcParams["axes.unicode_minus"] = False
from matplotlib.patches import Rectangle, FancyBboxPatch
from matplotlib.animation import FuncAnimation, PillowWriter

MSG = "hello"
IDLE = 1          # 帧间空闲位数
BITS_PER_FRAME = 10  # ST + 8 data + SP

# ---- 预计算比特流: [(frame_idx, bit_idx_in_frame, level, label)] ----
stream = []
for fi, ch in enumerate(MSG):
    for _ in range(IDLE):
        stream.append((fi, -1, 1, "idle"))
    b = ord(ch)
    stream.append((fi, 0, 0, "ST"))
    for i in range(8):
        stream.append((fi, 1 + i, (b >> i) & 1, f"D{i}"))  # LSB first
    stream.append((fi, 9, 1, "SP"))
TOTAL = len(stream)

# 波形采样点（每比特 10 采样）
def build_wave(upto):
    xs, ys = [], []
    for k in range(upto):
        _, _, lv, _ = stream[k]
        for s in range(10):
            xs.append(k + s / 10)
            ys.append(lv)
    return xs, ys

FRAME_X0 = [fi * (BITS_PER_FRAME + IDLE) for fi in range(len(MSG))]

# ---- 画布 ----
fig = plt.figure(figsize=(11.2, 6.0), dpi=110)
ax_frame = fig.add_axes([0.04, 0.60, 0.92, 0.32])   # 上：帧结构
ax_wave  = fig.add_axes([0.04, 0.10, 0.92, 0.40])   # 下：波形
for ax in (ax_frame, ax_wave):
    ax.set_xticks([]); ax.set_yticks([])
    for s in ax.spines.values(): s.set_visible(False)

fig.suptitle("'hello' 经 COM1 串口发出 · 115200 8N1 · 每帧 10 比特（空闲高电平）",
             fontsize=15, fontweight="bold", y=0.97)

# 波形轴范围
ax_wave.set_xlim(-0.5, TOTAL + 0.5)
ax_wave.set_ylim(-0.9, 1.75)

# 帧分隔与标签（静态）
for fi, ch in enumerate(MSG):
    x0 = FRAME_X0[fi] + IDLE
    ax_wave.axvline(x0 - 0.5, color="#e5e7eb", lw=0.8, zorder=0)
    ax_wave.text(x0 + 5, 1.55, f"'{ch}'  0x{ord(ch):02X}", ha="center",
                 fontsize=11, fontweight="bold", color="#374151")
ax_wave.axvline(FRAME_X0[-1] + IDLE + 10 - 0.5, color="#e5e7eb", lw=0.8, zorder=0)
ax_wave.axhline(1, color="#d1d5db", lw=0.6, ls=":", zorder=0)
ax_wave.axhline(0, color="#d1d5db", lw=0.6, ls=":", zorder=0)
ax_wave.text(-0.4, 1.0, "高=1", ha="right", va="center", fontsize=9, color="#9ca3af")
ax_wave.text(-0.4, 0.0, "低=0", ha="right", va="center", fontsize=9, color="#9ca3af")
ax_wave.text(TOTAL + 0.3, -0.75, "时间 →（1 比特 ≈ 8.68 µs）", ha="right",
            fontsize=10, color="#6b7280")

wave_line, = ax_wave.plot([], [], color="#2563eb", lw=2.2, solid_joinstyle="miter")
bit_dot, = ax_wave.plot([], [], "o", color="#dc2626", ms=7, zorder=5)
bit_label = ax_wave.text(0, 0, "", fontsize=9, color="#dc2626", ha="center",
                        fontweight="bold", zorder=5)

# ---- 上：当前帧结构 ----
ax_frame.set_xlim(-0.3, 11.6)
ax_frame.set_ylim(-1.4, 2.3)
cells = []
labels = ["ST", "D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "SP"]
for i, lb in enumerate(labels):
    r = Rectangle((i, 0), 0.9, 1.0, facecolor="#f9fafb", edgecolor="#9ca3af",
                  lw=1.2, zorder=2)
    ax_frame.add_patch(r)
    cells.append(r)
    ax_frame.text(i + 0.45, 0.5, lb, ha="center", va="center", fontsize=11,
                  color="#6b7280", zorder=3)
frame_title = ax_frame.text(0, 1.9, "", fontsize=13, fontweight="bold", color="#111827")
frame_sub   = ax_frame.text(0, 1.45, "", fontsize=10.5, color="#6b7280")
ax_frame.text(10.3, 0.5, "", fontsize=10, color="#6b7280")  # placeholder
bitval_texts = []
for i in range(10):
    t = ax_frame.text(i + 0.45, -0.55, "", ha="center", fontsize=10,
                      fontweight="bold", color="#374151")
    bitval_texts.append(t)

def colors_for(i, lv, active):
    if not active:
        return "#f9fafb", "#9ca3af"
    if lv == 1:
        return "#f0fdf4", "#16a34a"
    return "#fef2f2", "#dc2626"

def update(k):
    k = min(k, TOTAL - 1)
    fi, bi, lv, lb = stream[k]

    # 波形
    xs, ys = build_wave(k + 1)
    wave_line.set_data(xs, ys)
    bit_dot.set_data([k + 0.5], [lv])
    if lb != "idle":
        bit_label.set_position((k + 0.5, -0.35))
        bit_label.set_text(lb)
    else:
        bit_label.set_text("")

    # 帧面板
    if lb == "idle":
        ch = MSG[fi]
        frame_title.set_text(f"第 {fi+1} 帧：'{ch}'（0x{ord(ch):02X}）—— 线路空闲（高电平）")
        frame_sub.set_text("UART 无时钟线：双方靠事先约定的波特率各自数时间")
        for i in range(10):
            cells[i].set_facecolor("#f9fafb"); cells[i].set_edgecolor("#9ca3af")
            bitval_texts[i].set_text("")
        return [wave_line, bit_dot, bit_label, frame_title, frame_sub]

    ch = MSG[fi]
    b = ord(ch)
    frame_title.set_text(f"第 {fi+1} 帧：'{ch}'（0x{b:02X} = 0b{b:08b}，LSB 先发）")
    frame_sub.set_text("ST=起始位拉低（叫醒接收方）· D0~D7=数据位 · SP=停止位拉高（收尾）")
    for i in range(10):
        if i < bi:
            lv_i = 0 if i == 0 else (1 if i == 9 else (b >> (i - 1)) & 1)
            f, e = colors_for(i, lv_i, True)
            bitval_texts[i].set_text(str(lv_i))
        elif i == bi:
            f, e = colors_for(i, lv, True)
            bitval_texts[i].set_text(str(lv))
        else:
            f, e = colors_for(i, 0, False)
            bitval_texts[i].set_text("")
        cells[i].set_facecolor(f); cells[i].set_edgecolor(e)
    return [wave_line, bit_dot, bit_label, frame_title, frame_sub]

anim = FuncAnimation(fig, update, frames=TOTAL + 6, interval=140, blit=False)
import pathlib
root = pathlib.Path(__file__).resolve().parents[2]   # 里程碑目录 m0-boot/
dst = root / "assets" / "dst"
dst.mkdir(parents=True, exist_ok=True)
out = dst / "serial-hello-wave.gif"
anim.save(str(out), writer=PillowWriter(fps=7))
print("saved", out)

# -*- coding: utf-8 -*-
L = []
a = L.append
FF = "'Noto Sans CJK SC', sans-serif"

W, H = 1200, 1120
a(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}">')
a('<defs>')
for mid, col in [("arrow-green","#16a34a"),("arrow-gray","#6b7280"),("arrow-purple","#9333ea"),("arrow-blue","#2563eb")]:
    a(f'<marker id="{mid}" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto"><polygon points="0 0, 10 3.5, 0 7" fill="{col}"/></marker>')
a('</defs>')
a(f'<rect width="{W}" height="{H}" fill="#ffffff"/>')

def text(x,y,s,size=12,fill="#111827",weight=400,anchor="start"):
    a(f'<text x="{x}" y="{y}" font-family="{FF}" font-size="{size}" fill="{fill}" font-weight="{weight}" text-anchor="{anchor}">{s}</text>')

def box(x,y,w,h,fill="#ffffff",stroke="#d1d5db",rx=8,dash=None,sw=1.5):
    d = f' stroke-dasharray="{dash}"' if dash else ''
    a(f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" ry="{rx}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"{d}/>')

def bitcells(x, y, cells, label_above=True):
    """cells: (bit_label, name, value, fill, stroke, width); returns end x"""
    for bn,nm,val,fill,stroke,w in cells:
        if label_above: text(x+w/2, y-8, bn, 9, "#6b7280", 400, "middle")
        box(x, y, w, 48, fill, stroke, 6)
        dim = fill == "#f9fafb"
        text(x+w/2, y+20, nm, 11, "#6b7280" if dim else stroke, 600, "middle")
        text(x+w/2, y+36, val, 9, "#9ca3af" if dim else "#6b7280", 400, "middle")
        x += w + 4
    return x

GRAYF, GRAYS = "#f9fafb", "#9ca3af"

# ===== Title =====
text(40,46,"COM1 16550 UART 寄存器地图",22,"#111827",600)
text(40,72,"serial.rs 实际用到的寄存器与状态位 · 端口基址 0x3F8（COM1） · 访问方式：in / out 端口 IO",12,"#6b7280")

# ===== Left: register stack =====
rows = [
    ("+0","DATA（收发数据）","读+写","#eff6ff","#2563eb",
     "DLAB=0：写 = 发送一字节 · 读 = 接收一字节",
     "DLAB=1：波特率除数低字节（init 写 0x01）"),
    ("+1","INTERRUPT_ENABLE（中断使能）","读+写","#f0fdfa","#0d9488",
     "DLAB=0：中断开关（init 写 0x00，纯轮询）",
     "DLAB=1：波特率除数高字节（init 写 0x00）"),
    ("+2","FIFO_CONTROL（FIFO 控制）","只写","#fff7ed","#ea580c",
     "init 写 0xC7：使能 + 清空芯片内 16 字节收发缓冲（位图见右）",None),
    ("+3","LINE_CONTROL（线路控制）","读+写","#faf5ff","#9333ea",
     "数据格式 + DLAB 换挡开关（位图见右）",None),
    ("+4","MODEM_CONTROL（Modem 控制）","读+写","#fff7ed","#ea580c",
     "init 写 0x0B：DTR + RTS + OUT2（位图见右）",None),
    ("+5","LINE_STATUS（线路状态）","只读","#f0fdf4","#16a34a",
     "bit5 = 发送保持寄存器空（位图与轮询见右下）",None),
]
for i,(off,name,tag,fill,stroke,d1,d2) in enumerate(rows):
    y = 120 + i*76
    box(40,y,560,66,fill,stroke)
    box(52,y+14,52,38,"#ffffff",stroke,6)
    text(78,y+39,off,15,stroke,600,"middle")
    text(120,y+25,name,15,"#111827",600)
    box(492,y+12,88,22,"#ffffff",stroke,11)
    text(536,y+28,tag,11,stroke,600,"middle")
    text(120,y+45,d1,12,"#4b5563")
    if d2: text(120,y+60,d2,11,"#9333ea")

# DLAB dashed arrow from row+3 to rows +0/+1
a('<path d="M 40 386 L 22 386 L 22 163 L 36 163" fill="none" stroke="#9333ea" stroke-width="1.5" stroke-dasharray="5,3" marker-end="url(#arrow-purple)"/>')
text(40,592,"◆ 一址两用：LINE_CONTROL 的 bit7（DLAB）= 1 时，+0 / +1 变脸为波特率除数寄存器",12,"#9333ea")
text(40,610,"—— init 先「换挡」设除数，再「换挡」回来收发数据（serial.rs:43-46）",12,"#9333ea")

# ===== Left-bottom: poll loop =====
box(40,640,560,180)
text(56,664,"write_byte：发一个字节的轮询循环（serial.rs:51-54）",14,"#111827",600)
box(64,712,170,48,"#eff6ff","#2563eb",8)
text(149,732,"inb(0x3F8 + 5)",12,"#2563eb",600,"middle")
text(149,748,"读 LINE_STATUS",10,"#6b7280",400,"middle")
a('<path d="M 234 736 L 256 736" fill="none" stroke="#2563eb" stroke-width="1.5" marker-end="url(#arrow-blue)"/>')
a('<polygon points="340,698 400,736 340,774 280,736" fill="#ffffff" stroke="#d1d5db" stroke-width="1.5"/>')
text(340,741,"bit5 == 1 ?",12,"#111827",600,"middle")
a('<path d="M 400 736 L 412 736" fill="none" stroke="#16a34a" stroke-width="1.5" marker-end="url(#arrow-green)"/>')
a('<rect x="398" y="712" width="20" height="15" fill="#ffffff" opacity="0.95"/>')
text(408,724,"是",11,"#16a34a",600,"middle")
box(416,712,150,48,"#f0fdf4","#16a34a",8)
text(491,732,"outb(0x3F8 + 0)",12,"#16a34a",600,"middle")
text(491,748,"发出一字节",10,"#6b7280",400,"middle")
a('<path d="M 340 698 L 340 690 L 149 690 L 149 708" fill="none" stroke="#6b7280" stroke-width="1.5" stroke-dasharray="4,2" marker-end="url(#arrow-gray)"/>')
a('<rect x="206" y="680" width="76" height="15" fill="#ffffff" opacity="0.95"/>')
text(244,692,"否：继续等",10,"#6b7280",400,"middle")
text(56,806,"空循环等硬件，慢但零依赖 —— 中断驱动是后续里程碑的事",11,"#6b7280")

# ===== Right Panel A: LINE_CONTROL bitmap + 8N1 frame =====
box(680,120,480,250)
text(696,144,"LINE_CONTROL（+3）位图 · init 写它两次",14,"#111827",600)
cellsA = [("b7","DLAB","1=换挡","#faf5ff","#9333ea",64),
          ("b6","BRK","0",GRAYF,GRAYS,44),
          ("b5","SP","0",GRAYF,GRAYS,44),
          ("b4","EPS","0",GRAYF,GRAYS,44),
          ("b3","PEN","0=无",GRAYF,GRAYS,56),
          ("b2","STB","0=1位",GRAYF,GRAYS,56),
          ("b1:0","WLS","11=8位","#eff6ff","#2563eb",120)]
bitcells(694,172,cellsA)
box(694,238,452,30,"#faf5ff","#9333ea",6)
text(710,258,"第一次写 0x80：DLAB=1 → 换挡，+0/+1 变波特率除数寄存器",11,"#9333ea",600)
box(694,276,452,30,"#ffffff","#d1d5db",6)
text(710,296,"第二次写 0x03：DLAB=0，8 数据位 · 无校验 · 1 停止位（8N1）",11,"#111827",600)
text(696,330,"8N1 一帧的形状（线上先起始位，数据位低位先行）：",11,"#111827",600)
frame = [("ST","0"),("D0",""),("D1",""),("D2",""),("D3",""),("D4",""),("D5",""),("D6",""),("D7",""),("SP","1")]
x = 694
for nm,val in frame:
    data = nm.startswith("D")
    fill = "#eff6ff" if data else GRAYF
    stroke = "#2563eb" if data else GRAYS
    box(x,338,41,26,fill,stroke,4)
    text(x+20.5,355,nm,10,stroke if data else "#6b7280",600,"middle")
    x += 45

# ===== Right Panel B: FIFO_CONTROL bitmap =====
box(680,386,480,170)
text(696,410,"FIFO_CONTROL（+2）位图 · init 写 0xC7",14,"#111827",600)
cellsB = [("b7:6","TRIG","11=14B","#fff7ed","#ea580c",120),
          ("b5","—","0",GRAYF,GRAYS,44),
          ("b4","—","0",GRAYF,GRAYS,44),
          ("b3","DMA","0",GRAYF,GRAYS,56),
          ("b2","TXCL","1=清","#fff7ed","#ea580c",64),
          ("b1","RXCL","1=清","#fff7ed","#ea580c",64),
          ("b0","EN","1=使能","#fff7ed","#ea580c",64)]
bitcells(694,436,cellsB)
text(696,508,"FIFO = 16550 芯片内部的 16 字节收发缓冲（前身 8250 没有，每字节打断 CPU）",11,"#111827",600)
text(696,526,"触发深度：接收攒到 14 字节才申请中断；纯轮询模式用不上，主要为使能+清垃圾",11,"#6b7280")

# ===== Right Panel C: MODEM_CONTROL bitmap =====
box(680,572,480,170)
text(696,596,"MODEM_CONTROL（+4）位图 · init 写 0x0B",14,"#111827",600)
cellsC = [("b7:5","—","000",GRAYF,GRAYS,70),
          ("b4","LOOP","0=关","#f9fafb","#9ca3af",76),
          ("b3","OUT2","1=中断总闸","#faf5ff","#9333ea",90),
          ("b2","OUT1","0",GRAYF,GRAYS,56),
          ("b1","RTS","1=请求发","#fff7ed","#ea580c",70),
          ("b0","DTR","1=我就绪","#fff7ed","#ea580c",70)]
bitcells(694,622,cellsC)
text(696,694,"DTR / RTS：Modem 时代的握手线——qemu 不看，真硬件与终端程序会看",11,"#111827",600)
text(696,712,"OUT2：串口中断出芯片的总闸门（将来中断模式必开）· LOOP：回环自测",11,"#6b7280")

# ===== Right Panel D: LINE_STATUS bitmap =====
box(680,758,480,190)
text(696,782,"LINE_STATUS（+5）位图 · write_byte 只认 bit5",14,"#111827",600)
cellsD = [("b7","ERR"),("b6","TEMT"),("b5","THRE"),("b4","BI"),("b3","FE"),("b2","PE"),("b1","OE"),("b0","DR")]
cellsD = [(bn,nm,("1=可发" if nm=="THRE" else "未用"),
           ("#f0fdf4" if nm=="THRE" else GRAYF),("#16a34a" if nm=="THRE" else GRAYS),52)
          for bn,nm in cellsD]
bitcells(691,812,cellsD)
text(696,884,"bit5 THRE（Transmit Holding Register Empty）：发送保持寄存器空",12,"#111827",600)
text(696,902,"= 1 时才允许向 DATA 写下一字节，否则硬件会丢字节",11,"#6b7280")

# ===== Bottom: init sequence =====
text(40,996,"init() 初始化序列 —— 每步就是一次 outb（serial.rs:42-48）",14,"#111827",600)
steps = [("① 关中断","IER ← 0x00"),
         ("② 开 DLAB","LCR ← 0x80"),
         ("③ 设波特率","除数 0x0001 → 115200"),
         ("④ 定格式 8N1","LCR ← 0x03（关 DLAB）"),
         ("⑤ 开 FIFO","FCR ← 0xC7 清队列"),
         ("⑥ 握手就绪","MCR ← 0x0B（DTR+RTS）")]
for i,(t1,t2) in enumerate(steps):
    x = 40 + i*188
    box(x,1010,176,58,"#ffffff","#d1d5db",8)
    text(x+14,1032,t1,12,"#111827",600)
    text(x+14,1052,t2,10,"#6b7280")
    if i < 5:
        text(x+182,1050,"→",14,"#9ca3af",400,"middle")

# legend + footer
a('<line x1="40" y1="1094" x2="70" y2="1094" stroke="#2563eb" stroke-width="1.5" marker-end="url(#arrow-blue)"/>')
text(76,1098,"读寄存器",10,"#6b7280")
a('<line x1="150" y1="1094" x2="180" y2="1094" stroke="#16a34a" stroke-width="1.5" marker-end="url(#arrow-green)"/>')
text(186,1098,"是 / 可发",10,"#6b7280")
a('<line x1="250" y1="1094" x2="280" y2="1094" stroke="#6b7280" stroke-width="1.5" stroke-dasharray="4,2" marker-end="url(#arrow-gray)"/>')
text(286,1098,"否 / 回等",10,"#6b7280")
a('<line x1="350" y1="1094" x2="380" y2="1094" stroke="#9333ea" stroke-width="1.5" stroke-dasharray="5,3" marker-end="url(#arrow-purple)"/>')
text(386,1098,"DLAB 换挡",10,"#6b7280")
text(1160,1098,"Style 1 · COM1 16550 · linux-0.11-from-scratch M0",11,"#9ca3af",400,"end")

a('</svg>')
import pathlib
root = pathlib.Path(__file__).resolve().parents[2]   # 里程碑目录 m0-boot/
dst = root / "assets" / "dst"
dst.mkdir(parents=True, exist_ok=True)
p = dst / "serial-16550.svg"
p.write_text("\n".join(L), encoding="utf-8")
print("written", p)

# PNG 导出（供目检/贴文档；需要 cairosvg，如 uv pip install cairosvg）
try:
    import cairosvg
    cairosvg.svg2png(url=str(p), write_to=str(dst / "serial-16550.png"), scale=2.0)
    print("png ok")
except ImportError:
    print("skip png: cairosvg 未安装")

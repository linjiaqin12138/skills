# 线性 framebuffer：显存到底是一块什么数组

> 概念深读（M1 Bite1 背景卡片承诺）。面向无嵌入式背景的读者。
> 读完你应能回答：写一个像素时偏移怎么算；为什么行步长 `pitch` 常常不等于 `width × 每像素字节数`。

## 一句话版本

**线性 framebuffer** 是一块连续内存，按「从左到右、从上到下」扫描整屏。每个像素占固定字节数（bpp，bits per pixel）。第 `(x, y)` 个像素的字节偏移是：

```
offset = y * pitch + x * (bpp / 8)
```

这里的 `pitch`（行步长）是**一行在内存里实际占用的字节数**。它往往大于 `width * (bpp / 8)`，多出来的是对齐填充。

## 为什么需要搞懂它

本项目从 M1 起不再靠 BIOS 文本模式写字，而是自己往显存里填 RGB。算错偏移，字会花屏、错位、或写穿别的行。验收里的 `800x600x32`、串口打印的 `mask r8@16 g8@8 b8@0`，都是在告诉你这块数组的几何与颜色打包规则。

## 最小可运行示例：pitch 与 width 脱钩

下面这段在 Node 里可直接跑。它造一块「宽 4、高 2、32bpp」的假显存，但故意把 `pitch` 设成 32（每行多填 16 字节），再写两个像素，打印第一行前 16 字节。

```js
// node 直接粘贴运行
const width = 4;
const height = 2;
const bpp = 32;
const pitch = 32; // 故意大于 width*(bpp/8)=16
const buf = new Uint8Array(height * pitch);

function putPixel(x, y, r, g, b) {
  const off = y * pitch + x * (bpp / 8);
  // 小端 BGRA：与常见 x8r8g8b8 的字节序一致的一种写法
  buf[off] = b;
  buf[off + 1] = g;
  buf[off + 2] = r;
  buf[off + 3] = 0;
}

putPixel(0, 0, 255, 0, 0); // 红
putPixel(1, 0, 0, 255, 0); // 绿
console.log("pitch", pitch, "vs packed", (width * bpp) / 8);
console.log([...buf.slice(0, 16)]);
```

预期输出：

```
pitch 32 vs packed 16
[
  0, 0, 255, 0,
  0, 255, 0, 0,
  0, 0, 0, 0,
  0, 0, 0, 0
]
```

要点：

1. 第一个像素从偏移 0 起；第二个从偏移 4 起（32bpp → 4 字节）。
2. 若你误用 `y * width * 4` 当行基址，而硬件给的 `pitch` 更大，第二行会写到错误偏移。
3. 真实硬件里 `pitch` 常按 64/128 字节对齐；Limine 会把实际 `pitch` 写进 `struct limine_framebuffer`。

## 颜色怎么塞进一个像素字

同一块线性显存还带 **mask**：红/绿/蓝各占几位、从哪一位开始。本项目 qemu + Limine 实测常见为：

- `bpp = 32`
- `memory_model = RGB (1)`
- `r8@16 g8@8 b8@0` → 一个 `u32` 里大致是 `0x00RRGGBB`（高字节未用）

内核用 mask 把 `(r,g,b)` 折成像素字再 `write_volatile` 写进 `address + offset`。写死「永远是 x8r8g8b8」在当前 qemu 能过，换 bpp 或 mask 就会花屏——这是 M1 Bite1 决策卡片里选「按 mask 打包」的失效边界。

## 和本项目的对应关系

| 协议字段 | 含义 |
|---|---|
| `address` | 显存起始指针 |
| `width` / `height` | 可见像素宽高 |
| `pitch` | 一行字节数（可含填充） |
| `bpp` | 每像素位数 |
| `*_mask_size` / `*_mask_shift` | RGB 打包规则 |

配置请求 `resolution: 800x600x32`（`limine.conf`）只是「想要这个模式」；最终以 response 里的几何与 mask 为准。内核启动时会把实测值打到串口：`M1: fb 800x600x32`。

## 深入阅读

- Limine 协议：`reference/limine-protocol/PROTOCOL.md` 的 Framebuffer Feature
- 头文件布局：`reference/limine-protocol/include/limine.h` 的 `struct limine_framebuffer`
- 本仓库写像素：`kernel/src/fb.rs` 的 `put_pixel` / `pack_rgb`

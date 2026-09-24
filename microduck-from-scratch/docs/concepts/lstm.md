# LSTM：带便签的函数

## 一句话

普通网络是纯函数：同样的输入，永远同样的输出。LSTM 多带两张便签（`h` 和 `c`），下一拍必须把上一拍吐出来的便签再喂回去，所以同样的观测可以走出不同的动作。

## 为什么需要

走路不是「看这一帧就决定迈哪只脚」。左脚还在空中、还是已经落地，单帧关节角经常分不清。网络要记住「我走到这一步的哪一拍」。

velstand **不是**这种网络。它是 1 个输入、1 个输出，没有便签。这段概念是为了读懂 `policy.rs` 里那条目前不执行的分支，不是为了改站立行为。

和 TypeScript 的差异：可以想成闭包里藏了一个 `let note`。差异是 ONNX 图不会自己留闭包——便签是调用方拿着的两块数组，下一拍当参数传回去。进程重启、或换成另一份权重，便签必须清零，否则新网络读到旧网络的记忆。

三道门（遗忘 / 写入 / 露出）先不用管。看动画时知道「有一条不会每步乘一个小数就衰减掉的传送带」就够了。

## 最小例子

下面不是 LSTM，只演示「有没有便签」对同一输入的差别。已用 Python 跑过：

```python
def feedforward(obs):
    return round(obs * 0.5, 3)

def with_notepad(obs, note):
    action = round(obs * 0.5 + note, 3)
    note = round(0.9 * note + obs, 3)
    return action, note
```

同一输入 `1` 连喂三次：

```text
same input 1, no memory: [0.5, 0.5, 0.5]
same input 1, notepad starts at 0:
  action=0.5  note_after=1.0
  action=1.5  note_after=1.9
  action=2.4  note_after=2.71
```

没有便签时三次都是 0.5。有便签时动作一次比一次大，因为便签被带进了下一次调用。

## 素材

- 可拖的三道门：[LSTM 的门控记忆](https://nndl.ai/viz/lstm-gates/)（蒲公英书）。先把遗忘门拉到接近 1、输入门拉到接近 0，看金线几乎平着走过去。
- 动画讲解：[LSTM 原理动画](https://www.bilibili.com/video/BV1ih4y147YQ/)（约 20 分钟，前半段就够）。
- 公式版可后看：[动手学深度学习 · LSTM](https://zh-v1.d2l.ai/chapter_recurrent-neural-networks/lstm.html)。

## 回到项目

`src/policy.rs:105-130` 数的是 **图上有几块命名张量**，不是 `obs` 里面那 61 个数。

| 输入块数, 输出块数 | 判定 | velstand |
|---|---|---|
| 1, 1 | 纯函数。输入名 `obs`，唯一那块输出就是动作 | 就是这种。`state = None` |
| 3, 3 | 带便签。输入 `obs` / `h_in` / `c_in`，输出 `actions` / `h_out` / `c_out` | 不会走到 |
| 其他 | load 直接报错 | 不会走到 |

`h` 是露出给外面的那张便签，`c` 是内部传送带。下一拍调用方把 `h_out`、`c_out` 抄回 `h_in`、`c_in`（`src/policy.rs:158-165` 和 `:180-189`）。velstand 的 `state` 是 `None`，推理只塞 `obs`。

## 自测

velstand 连喂三次全零观测，三次动作会不会像上面的便签例子那样一次比一次变？为什么？

# 换电脑验证：官方仿真站得住，前进键迈不开

验证方式是你看着官方窗口、自己按键。不要写测量脚本，也不要代发 `robot.move`。

要验证的现象：`scripts/infer_policy.py` 默认零速度时鸭子站直；在启动它的终端里按一次 ↑ 之后，鸭子保持直立，脚几乎不动。

Linux x86_64。要有一块能弹出窗口的显示器。WSL 用 WSLg，窗口会直接出现在 Windows 上。

## 安装

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev curl ca-certificates git \
  python3 python3-venv python3-pip libgl1 libglfw3
```

```bash
mkdir -p "$HOME/microduck-verify" && cd "$HOME/microduck-verify"
git clone https://github.com/pollen-robotics/microduck_rl.git
git -C microduck_rl checkout b02fdea

mkdir -p policies
curl -fL --retry 5 --connect-timeout 30 -o impl/policies/velstand.onnx \
  "https://huggingface.co/pollen-robotics/microduck-policies/resolve/v5/velstand.onnx"
ls -l impl/policies/velstand.onnx
```

`b02fdea` 是 microduck_rl `develop`（2026-10-12）。`velstand.onnx` 大约 800 KB。只有几十字节就是下载失败，不要往下走。

```bash
python3 -m venv .venv
.venv/bin/pip install -U pip
.venv/bin/pip install mujoco numpy onnxruntime
.venv/bin/pip install "better-actuator-models @ git+https://github.com/Rhoban/bam.git@62bd8ce12154340be97e06f7f41a0ca8f116d967"
```

## 本地补丁：↑ 一步到 0.8（2026-10-14 实验）

原版脚本按一次 ↑ 直接把速度命令设到上限 0.30（`infer_policy.py:1501` 调 `set_vel_cmd(vel_max_x, ...)`，行走模式上限在 `:1369`）。本验证要把上限改成 0.8：

```bash
sed -i 's/^        policy\.vel_max_x = 0\.3$/        policy.vel_max_x = 0.8/' \
  "$HOME/microduck-verify/microduck_rl/scripts/infer_policy.py"
```

注意：脚本里这些上限的注释写明是"对齐训练采样范围"的（`:1358`），0.3 就是这份 velstand 训练时见过的最大前进命令。**0.8 是训练分布外的命令**，策略没学过怎么响应它——步态变形、摔倒都属于预期内的可能，这正是本实验要观察的东西。

## 打开界面

在系统终端里跑。收不到按键的面板里，↑ 不会进到程序。工作目录必须是 `microduck_rl`，场景路径相对这条目录。

```bash
cd "$HOME/microduck-verify/microduck_rl"
../.venv/bin/python scripts/infer_policy.py \
  --walking "$HOME/microduck-verify/policies/velstand.onnx" \
  --action-scale 0.9 \
  --new-cmd-obs
```

不要加 `--lin-vel-x`。默认速度是 0。不要加 `--no-bam`。

终端打印键盘说明，并出现 `Vel cmd: [0.00, 0.00, 0.00]`。同时弹出 MuJoCo 窗口。

## 在界面里做的事

1. 先什么都别按。看窗口里的鸭子从空中落到地面，站直，停住。大约数到 5。
2. 用鼠标点回刚才那条终端，不要点仿真窗口。按键由终端接收，窗口不接收前进键。
3. 按一次 ↑。终端应打印 `Vel cmd: [0.80, 0.00, 0.00]`（已按上节补丁把上限改到 0.8；没打补丁是 0.30）。一次就设到上限，不用按住。
4. 看窗口大约 10 秒：脚有没有交替迈步，身体有没有明显往前走。
5. 按空格把速度清回 0。按 q 退出。

不要按 P。那一键是手动把躯干速度设成 1 m/s，不是收到前进命令后的动作。

## 应该看到什么

0.30（未打补丁）时：↑ 之后鸭子保持直立，脚几乎不动，人在原地。终端里的速度命令是 0.30，窗口里没有步态。

0.80（打了补丁）时：待观察记录——命令已超出训练分布，没有"应该"的基准行为，看到什么记什么（前倾不动 / 迈步 / 摔倒）。

`duck-body` 的窗口只负责看，里面没有前进键。给人操作的官方键盘在上面这个界面里。

---
template: sheet
title: M9d Bite 2：首训能站能走？（第 28 章）
subtitle: 官方 export → 本机 viewer；headless 门禁为辅
source: clone-from-scratch-tutor · 第 28 章
---

## 问题

第 27 章只证明「能训完」。mean_reward 涨了，不等于鸭子在 MuJoCo 里站得住、迈得出步。云机没有显示器，`infer_policy.py` 又默认开 viewer——容易卡在「权重有了，却看不了」。

本课消掉的故障是：不会把 `model_999.pt` 变成可部署 ONNX，也不会把权重拷回有显示器的本机用官方 viewer 肉眼验收站/走雏形。

## 背景

**官方 export。** `uv run scripts/export.py <TASK> --checkpoint-file …` 是烘进 obs normalizer 的安全路径。训练目录里自动冒出的 `.onnx` 不要默认当发布物。

**验收分工。** **肉眼主路径 = 本机**：scp `policy_export.onnx` → `bash impl/scripts/m9d_infer_local.sh`（官方 `infer_policy.py` + GLFW）。云机无显示器时的 JSON 门禁用 `m9d_infer_headless.py`（辅助）。取舍见 [0009](../../decisions/0009-m9d-infer-headless.md)。

## 设计

```flow TB
(model_999.pt) -> 官方 export
官方 export -> *policy.onnx
policy.onnx -> scp 到本机
scp 到本机 -> 本机 infer_policy viewer
policy.onnx -> headless JSON 门禁
```

门禁（雏形，不是最终速度跟踪）：

| 试验 | 通过条件 |
|---|---|
| 站 8s，BAM，`vx=0` | 未倒（`fell=false`），`min_trunk_z ≥ 0.10` |
| 走 10s，BAM，`vx=0.3` | 未倒，`forward_m ≥ 0.30` |
| 走 10s，`--no-bam` | 对照：看侧偏与前移差；不挡本课主门 |
| 本机 viewer | 肉眼：站得住、能迈步；键盘可调速 |

### 云机 headless 实测（2026-10-29）

| 试验 | `forward_m` / `min_z` | fell |
|---|---|---|
| 站 BAM | dx≈−0.04 m / z_min≈0.114 | false |
| 走 BAM 0.3 | **+1.06 m** / 0.114 | false |
| 走 `--no-bam` 0.3 | +0.44 m（侧偏更大） | false |

说明：1000 iter 仍欠跟踪，但「能走雏形」数字门禁已过；**请你本机开 viewer 再肉眼确认一遍**。

## 架构决定

肉眼验收走 **本机官方 viewer**；云上只用 headless 做可重复 JSON 门禁。见 [0009](../../decisions/0009-m9d-infer-headless.md)。

## 验收

### A. 本机肉眼（主路径，今晚做这条）

在教程仓根执行。本课云机：`ssh -p 23 root@117.50.221.204`（数据盘 `/cloud/cloud-ssd1`）。

**1. 拷官方 export 的 ONNX**

若云上还没有 `eval/policy_export.onnx`，先在云机执行：

```bash
export PATH="$HOME/.local/bin:$PATH"
cd /cloud/cloud-ssd1/microduck-m9d/microduck_rl
uv run scripts/export.py Mjlab-Velocity-Flat-MicroDuck \
  --checkpoint-file logs/rsl_rl/velocity/2026-10-29_08-00-43_velocity/model_999.pt
cp -f output.onnx /cloud/cloud-ssd1/microduck-m9d/eval/policy_export.onnx
```

本机拉取（约 800KB，秒级）：

```bash
mkdir -p tutor/artifacts/m09d
scp -P 23 root@117.50.221.204:/cloud/cloud-ssd1/microduck-m9d/eval/policy_export.onnx \
  tutor/artifacts/m09d/policy_export.onnx
ls -la tutor/artifacts/m09d/policy_export.onnx
```

（若文件已在 `tutor/artifacts/m09d/policy_export.onnx`，可跳过 scp。）

**2. 本机准备 `microduck_rl`（一次）**

`infer_policy` 只要 CPU，但官方 `uv sync` 仍会拉 torch/CUDA 全家桶（数 GB）。网速差时换快节点 / 全局代理后再 sync；**不要**在已开 FlClash TUN 时再叠 `HTTP_PROXY=127.0.0.1:7890`。

```bash
git clone --depth 1 https://github.com/pollen-robotics/microduck_rl.git vendor/microduck_rl
cd vendor/microduck_rl
UV_HTTP_TIMEOUT=600 uv sync
cd ../..
```

**3. 开官方 viewer**

```bash
bash impl/scripts/m9d_infer_local.sh
```

预期：弹出 MuJoCo 窗；鸭子站稳后能迈步。键盘调 `vx/vy/yaw`（以终端/官方脚本提示为准）。对照 XML PD：

```bash
bash impl/scripts/m9d_infer_local.sh --no-bam
```

脚本：`impl/scripts/m9d_infer_local.sh`（默认 `--new-cmd-obs`，61D）。

### B. 自动门禁（云上 JSON，已跑过）

```bash
bash accept/m09d-b02.sh
```

读 `tutor/runs/m09d-b02.txt`。云机重跑 headless 见附录。

## 坑

- 云机无显示器 / xvfb 缺 Mesa：不要硬开 viewer，拷 ONNX 回本机。
- Velocity ONNX 是 **61D**：必须 `--new-cmd-obs`（本地脚本已默认）。
- 训练目录自动 `.onnx` ≠ 官方 export；部署与肉眼验收都走 `scripts/export.py` 产物。
- 本机 `uv sync` 体积大、易慢：瓶颈常在 PyPI/CUDA 轮子，与 ONNX 拷贝无关。

## 自测

**1. 部署前应如何从 `model_999.pt` 得到 ONNX？**

- A. 直接拷训练目录自动 `.onnx`
- B. `uv run scripts/export.py <TASK> --checkpoint-file model_999.pt`
- C. `torch.onnx.export` 手转 actor
- D. 只上传 wandb，不需要本地文件

<details><summary>答案</summary>

B。官方路径烘 normalizer。

</details>

**2. 本课肉眼验收主路径是什么？**

- A. 只看 wandb mean_reward
- B. 云机强开 `infer_policy` viewer
- C. scp `policy_export.onnx` 到本机，跑 `m9d_infer_local.sh`
- D. 只用 `--no-bam` headless

<details><summary>答案</summary>

C。云机无显示器；本机 viewer 是观感主路径。

</details>

**3. 数字门禁「走雏形」看什么？**

- A. 必须均速达到 0.3 m/s
- B. BAM、`vx=0.3`、10s、`forward_m≥0.30` 且未倒
- C. 只能在有显示器的机器上点 viewer 才算过
- D. wandb 曲线即可

<details><summary>答案</summary>

B。雏形看位移与不倒；精确跟踪留给更长训 / M9e。本机 viewer 另做肉眼确认。

</details>

## 附录：云机 headless（可选重跑）

```bash
export PATH="$HOME/.local/bin:$PATH" MUJOCO_GL=egl
cd /cloud/cloud-ssd1/microduck-m9d/microduck_rl
uv run python /cloud/cloud-ssd1/microduck-m9d/eval/m9d_infer_headless.py \
  --repo . \
  --walking /cloud/cloud-ssd1/microduck-m9d/eval/policy_export.onnx \
  --seconds 10 --lin-vel-x 0.3 \
  --out /cloud/cloud-ssd1/microduck-m9d/eval/walk_bam.json
```

脚本源：`impl/scripts/m9d_infer_headless.py`。

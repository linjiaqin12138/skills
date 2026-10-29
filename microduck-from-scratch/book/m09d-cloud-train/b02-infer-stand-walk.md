---
template: sheet
title: M9d Bite 2：首训能站能走？（第 28 章）
subtitle: 官方 export → headless infer；BAM 与 --no-bam
source: clone-from-scratch-tutor · 第 28 章
---

## 问题

第 27 章只证明「能训完」。mean_reward 涨了，不等于鸭子在 MuJoCo 里站得住、迈得出步。云机没有显示器，`infer_policy.py` 又默认开 viewer——容易卡在「权重有了，却看不了」。

本课消掉的故障是：不会把 `model_999.pt` 变成可部署 ONNX，也不会在无 GUI 的机器上用与官方同一条物理路径验收站/走雏形。

## 背景

**官方 export。** `uv run scripts/export.py <TASK> --checkpoint-file …` 是烘进 obs normalizer 的安全路径。训练目录里自动冒出的 `.onnx` 不要默认当发布物。

**两路验收。** 云机无显示器 → `m9d_infer_headless.py` 做 JSON 门禁；**有显示器的本机** → scp `policy_export.onnx` 后 `bash impl/scripts/m9d_infer_local.sh` 开官方 `infer_policy.py` viewer（肉眼主路径）。取舍见 [0009](../../decisions/0009-m9d-infer-headless.md)。

## 设计

```flow TB
(model_999.pt) -> 官方 export
官方 export -> *policy.onnx
policy.onnx -> headless 站 vx=0
policy.onnx -> headless 走 vx=0.3 BAM
policy.onnx -> headless 走 vx=0.3 --no-bam
```

门禁（雏形，不是最终速度跟踪）：

| 试验 | 通过条件 |
|---|---|
| 站 8s，BAM，`vx=0` | 未倒（`fell=false`），`min_trunk_z ≥ 0.10` |
| 走 10s，BAM，`vx=0.3` | 未倒，`forward_m ≥ 0.30` |
| 走 10s，`--no-bam` | 对照：看侧偏与前移差；不挡本课主门 |

### 本次实测（2026-10-29）

| 试验 | `forward_m` / `min_z` | fell |
|---|---|---|
| 站 BAM | dx≈−0.04 m / z_min≈0.114 | false |
| 走 BAM 0.3 | **+1.06 m** / 0.114 | false |
| 走 `--no-bam` 0.3 | +0.44 m（侧偏更大） | false |

说明：1000 iter 仍欠跟踪（指令 0.3、实际平均速度更低），但「能走雏形」已成立。更长训与接我们 daemon 留给后续。

## 架构决定

云上验收选 **headless 复用 PolicyInference**，不用强开 viewer。见 [0009](../../decisions/0009-m9d-infer-headless.md)。

## 验收

自动门禁（教程仓根）：

```bash
bash accept/m09d-b02.sh
```

**云机上手跑（逐步）：**

**1. 官方 export**

```bash
export PATH="$HOME/.local/bin:$PATH"
cd /cloud/cloud-ssd1/microduck-m9d/microduck_rl
uv run scripts/export.py Mjlab-Velocity-Flat-MicroDuck \
  --checkpoint-file logs/rsl_rl/velocity/2026-10-29_08-00-43_velocity/model_999.pt
```

预期打印 `Written …/output.onnx`。可拷到 `eval/policy_export.onnx`。

**2. 站立（BAM）**

```bash
export PATH="$HOME/.local/bin:$PATH" MUJOCO_GL=egl
cd /cloud/cloud-ssd1/microduck-m9d/microduck_rl
uv run python /cloud/cloud-ssd1/microduck-m9d/eval/m9d_infer_headless.py \
  --repo . \
  --walking /cloud/cloud-ssd1/microduck-m9d/eval/policy_export.onnx \
  --seconds 8 --lin-vel-x 0 \
  --out /cloud/cloud-ssd1/microduck-m9d/eval/stand_bam.json
```

观察 JSON：`fell` 为 false，`min_trunk_z` ≥ 0.10。

**3. 行走雏形（BAM）**

```bash
uv run python /cloud/cloud-ssd1/microduck-m9d/eval/m9d_infer_headless.py \
  --repo . \
  --walking /cloud/cloud-ssd1/microduck-m9d/eval/policy_export.onnx \
  --seconds 10 --lin-vel-x 0.3 \
  --out /cloud/cloud-ssd1/microduck-m9d/eval/walk_bam.json
```

观察：`forward_m` ≥ 0.30，未倒。

**4. 对照 `--no-bam`（可选）**

```bash
uv run python /cloud/cloud-ssd1/microduck-m9d/eval/m9d_infer_headless.py \
  --repo . \
  --walking /cloud/cloud-ssd1/microduck-m9d/eval/policy_export.onnx \
  --seconds 10 --lin-vel-x 0.3 --no-bam \
  --out /cloud/cloud-ssd1/microduck-m9d/eval/walk_nobam.json
```

预期仍可能不倒，但轨迹与 BAM 不同——提醒「训练分布是 BAM」。

脚本源在教程仓：`impl/scripts/m9d_infer_headless.py`（拷到云机 eval 目录即可）。

**5. 本机 viewer（推荐肉眼看）**

云机没有显示器时，把 ONNX 拷回本机开官方 viewer：

```bash
# 本机：先备好权重（示例端口/主机按你的云机改）
mkdir -p tutor/artifacts/m09d
scp -P 23 root@<云机IP>:/cloud/cloud-ssd1/microduck-m9d/eval/policy_export.onnx \
  tutor/artifacts/m09d/policy_export.onnx

# 一次：clone + uv sync（只要 CPU；infer 不需要训练 GPU）
git clone --depth 1 https://github.com/pollen-robotics/microduck_rl.git vendor/microduck_rl
cd vendor/microduck_rl && UV_HTTP_TIMEOUT=600 uv sync && cd ../..

# 开窗（键盘调 vx/vy/yaw）
bash impl/scripts/m9d_infer_local.sh
```

## 坑

- 只开 xvfb、不装 Mesa DRI：GLFW `could not create window`。本课改 headless stub。
- Velocity ONNX 是 **61D**：必须 `new_cmd_obs`（脚本默认开）；legacy 51D 会维数错。
- 自动训练目录 `.onnx` ≠ 官方 export；部署前走 `scripts/export.py`。

## 自测

**1. 部署前应如何从 `model_999.pt` 得到 ONNX？**

- A. 直接拷训练目录自动 `.onnx`
- B. `uv run scripts/export.py <TASK> --checkpoint-file model_999.pt`
- C. `torch.onnx.export` 手转 actor
- D. 只上传 wandb，不需要本地文件

<details><summary>答案</summary>

B。官方路径烘 normalizer。

</details>

**2. 本课「走雏形」主门禁看什么？**

- A. wandb mean_reward > 200
- B. BAM、`vx=0.3`、10s、`forward_m≥0.30` 且未倒
- C. 必须达到指令速度 0.3 m/s 均速
- D. 只能在有显示器的机器上点 viewer

<details><summary>答案</summary>

B。雏形看位移与不倒；精确跟踪留给更长训 / M9e。

</details>

**3. 为什么还要跑 `--no-bam`？**

- A. 官方训练不用 BAM
- B. 对照：同一策略在 XML PD 下行为会漂，强调训练分布是 BAM
- C. `--no-bam` 更快所以替代 BAM 验收
- D. 只有 `--no-bam` 支持 61D

<details><summary>答案</summary>

B。

</details>

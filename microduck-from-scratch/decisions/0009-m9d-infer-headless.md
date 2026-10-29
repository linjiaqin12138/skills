# 0009 M9d Bite 2：无桌面云机用 xvfb + 官方 export 验收站/走

- 状态：已采纳
- 来源：手搓
- 出现于：m09d-b02
- 被取代于：

起草可以改。用户确认这一课之后，手搓决定改为「已采纳」。用户确认 Phase 1 之后，原项目决定改为「已记录」。此后不改正文。要推翻，新开一篇，本篇状态改为「被取代」，并填上新编号。

## 上下文

M9d 里程碑验收：首训 checkpoint 在 `infer_policy.py` 里能站 / 能走雏形。云 4090 无显示器；`infer_policy.py` 默认 `mujoco.viewer.launch_passive`。训练目录里已有自动 `.onnx`，但部署路径要求官方 export 烘 obs normalizer。

## 备选

1. **官方 `scripts/export.py --checkpoint-file` 出 ONNX**，再 **`xvfb-run` + `timeout` 跑 `infer_policy.py`**（BAM 默认；另跑 `--no-bam` 对照），`--save-csv` 留轨迹；用躯干高度 / 前移距离做门禁。
2. 只用 `uv run play --video` 录视频，不跑 `infer_policy.py`。
3. 把权重拷回本机有 GUI 的环境再人手点 viewer。

## 决定

**两条路径并用，分工不同：**

1. **自动门禁（云机）**：官方 `export.py` 出 ONNX；`impl/scripts/m9d_infer_headless.py` 复用 `infer_policy.PolicyInference` + BAM（stub viewer，固定时长）；BAM / `--no-bam` JSON 门禁。云机无可用 GLFW/Mesa，不能当真开 viewer。
2. **肉眼验收（本机）**：把 `policy_export.onnx` scp 回本机；`bash impl/scripts/m9d_infer_local.sh` 调官方 `scripts/infer_policy.py` 开 GLFW viewer（键盘控速）。这是备选 3，观感主路径。

## 放弃的成本

备选 2（play --video）更贴近 mjlab 工具链；放弃它因点名对照路径是 `infer_policy` + BAM，且本机已有 viewer。  
云上强开 viewer：按量机常无显示器/Mesa，不如拷 ONNX 回本机。

## 失效边界

headless stub 与官方 viewer 观测不一致（obs 维、动作缩放）时，以本机 `infer_policy.py` viewer 为准；门禁脚本跟着修。

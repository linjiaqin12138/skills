# 0009 M9d Bite 2：官方 export + 本机 viewer；云上 headless 门禁

- 状态：起草
- 来源：手搓
- 出现于：m09d-b02
- 被取代于：

起草可以改。用户确认这一课（含本机肉眼）之后，手搓决定改为「已采纳」。用户确认 Phase 1 之后，原项目决定改为「已记录」。此后不改正文。要推翻，新开一篇，本篇状态改为「被取代」，并填上新编号。

## 上下文

M9d 里程碑验收：首训 checkpoint 经官方 export 后能站 / 能走雏形。云 4090 无显示器；`infer_policy.py` 默认开 viewer。学员本机有显示器。

## 备选

1. 云上 xvfb / headless 跑 `infer_policy`，JSON 门禁。
2. 只用 `uv run play --video`。
3. **把 ONNX 拷回本机，官方 `infer_policy.py` 开 viewer（肉眼）。**

## 决定

**肉眼主路径 = 备选 3**：`export.py` → scp `policy_export.onnx` → `impl/scripts/m9d_infer_local.sh`。  
**自动门禁（辅）**：云上 `m9d_infer_headless.py` JSON（BAM / `--no-bam`）；不替代本机观感确认。

## 放弃的成本

备选 2 放弃：对照路径点名 `infer_policy` + BAM。  
云上强开 viewer 放弃：无显示器/Mesa。

## 失效边界

headless 与本机 viewer 行为不一致时，以本机 `infer_policy.py` 为准。

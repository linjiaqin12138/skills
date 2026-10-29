# 0010 M9e Bite 1：自训 walk 用 action_scale=1.0，公开 velstand 仍默认 0.9

- 状态：被取代（M9e 搁置，未交付；2026-10-29 用户改走 handmade 重训）
- 来源：手搓
- 出现于：m09e-b01
- 被取代于：待 M9f 决定编号（若日后 daemon 对接仍需要 scale 旋钮，新开一篇，勿直接复活本文）

起草可以改。用户确认这一课之后，手搓决定改为「已采纳」。用户确认 Phase 1 之后，原项目决定改为「已记录」。此后不改正文。要推翻，新开一篇，本篇状态改为「被取代」，并填上新编号。

## 上下文

M9d 首训按官方 Velocity 配方，`JointPositionActionCfg.scale = 1.0`（`microduck_velocity_env_cfg.py`）。  
`infer_policy.py` 也是 `target = DEFAULT_POSE + action * 1.0`。  
我们 daemon 的 walk 槽常量仍是 `WALK_ACTION_SCALE = 0.9`（对齐公开 HF `velstand` / 原版 Tuning 原型 alpha）。  
同一份 `policy_export.onnx` 若在我们栈里仍乘 0.9，等于把训练契约压掉 10%。

本 Bite 要把自训 ONNX 接进 `miniduckd` + handmade BAM，验收站立与 `vx=0.3` 行走雏形。

## 备选

1. **环境变量覆盖**：默认仍 0.9；验收 / 自训部署设 `MINIDUCK_WALK_ACTION_SCALE=1.0`。公开 `velstand` 路径不动。
2. **全局改成 1.0**：与新训一致；公开 `velstand` 在我们栈上动作放大约 11%，站/走回归可能漂。
3. **按文件名猜 scale**：路径含 `policy_export` 就用 1.0——脆弱，且和原版「scale 是权重属性」语义不符。

## 决定

采用备选 1。walk 槽读 `MINIDUCK_WALK_ACTION_SCALE`（缺省或非法 → `0.9`）。  
M9e 闭环验收显式设 `1.0`，与训练 / 官方 infer 契约对齐；日常公开权重不改环境，行为与 M8/M9a 回归一致。

策略文件仍用已有 `MINIDUCK_POLICY` 指向 `tutor/artifacts/m09d/policy_export.onnx`，不改默认 `policies/velstand.onnx`。

## 放弃的成本

备选 2 少一个旋钮、部署少记一步；放弃它是为了不把已验收的公开 `velstand` 闭环一起搅动。  
备选 3 免设环境变量；放弃它是为了不把契约绑在文件名上。

## 失效边界

若上游公开 walk 权重也改为训练 scale 1.0，或我们弃用公开 `velstand`、只跑自训，应把默认改成 1.0 并删掉覆盖旋钮。  
若某份自训权重的 manifest / 训练配置写明非 1.0 的 scale，覆盖值必须跟那份配置，而不是写死 1.0。

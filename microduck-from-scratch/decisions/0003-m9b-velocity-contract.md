# 0003 M9b Bite 1：Velocity 契约用静态对账，不强制整仓 uv sync

- 状态：已采纳
- 来源：手搓
- 出现于：m09b-b01
- 被取代于：

起草可以改。用户确认这一课之后，手搓决定改为「已采纳」。用户确认 Phase 1 之后，原项目决定改为「已记录」。此后不改正文。要推翻，新开一篇，本篇状态改为「被取代」，并填上新编号。

## 上下文

M9b 目标是 CPU 侧学透训练管线。Bite 1 只钉 Velocity 配方的 **scene/obs/action 契约**（61D obs、14D action、twist 采样范围）以及与我们 `impl/src/obs.rs` 的对账。完整 `uv sync` microduck_rl 会拉 CUDA/torch 轮子，本 VM 无 GPU，且 Bite 1 不需要 step env。先修返回摘要已掌握 PPO/奖励/DR 直觉，本 Bite 不重教。

## 备选

1. 在 `impl/scripts/` 写契约对账脚本：解析 `reference/microduck_rl` 的 velocity cfg 关键常量 + 对照 `obs.rs` 偏移；验收默认只跑静态断言与既有 `obs` 单测。若本机已有可用 RL venv，可选跑 CPU pytest 子集。
2. Bite 1 强制 `uv sync` 全依赖并跑通全仓 `pytest tests/`，再写课文。
3. 不写脚本，只靠课文领读官方 cfg，验收改成口头自测。

## 决定

采用备选 1。本 Bite 交付物在 `impl/scripts/m9b_velocity_contract.py`（及脚本自测所需最小夹具）；不改 miniduckd 控制逻辑。reward/curriculum 深读留给 Bite 2；train/export 命令留给 Bite 3。官方源只读 `reference/microduck_rl/`（或 `MICRODUCK_RL_ROOT`），禁止抄进 `impl/`。

## 放弃的成本

备选 2 能一次证明「官方测试绿」；放弃它的代价是本 Bite 不把 CUDA 安装与 pytest 全绿当成硬门槛，避免环境装机淹没配方地图本身。

备选 3 写作最快；放弃它的代价是必须有可重复的契约断言，否则「我们 61D 与训练一致」只能靠记忆。

## 失效边界

若官方把 actor obs 从 61D 改版，或命令范围写入不可静态解析的工厂深处，本脚本解析策略必须重写。若用户要求本机必须带 RL venv 作 CI 硬门，改判备选 2。

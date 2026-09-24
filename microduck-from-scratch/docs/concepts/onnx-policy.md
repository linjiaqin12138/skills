# ONNX 策略会话

## 一句话

ONNX 会话是已经打开的模型文件：你塞进一条定长 `float32` 数组，它吐出另一条定长数组。没有 HTTP，没有字段名。

## 为什么需要

训练框架（PyTorch 等）把网络导出成 `.onnx` 图。部署端只装 [ONNX Runtime](https://onnxruntime.ai/docs/get-started/with-python.html)，按图上的算子做前向计算。机器人控制循环每 20 ms 要跑一次这个前向；装整个训练栈既慢又重。

对 TypeScript/Node 读者：可以先把它想成「把模型当成无 HTTP 的本地函数调用」——`session.run(obs) → actions`。差异立刻写清：

| 类比 | 差异 |
|---|---|
| npm 包 / 本地函数 | 没有 URL、没有 JSON；输入输出是定长二进制张量 |
| `fetch` 返回任意 JSON | 维数错了要么 load 时拒绝，要么跑起来像调参问题 |
| Node 事件循环单线程 | Runtime 可以多线程算图；本仓库把 intra-threads 钉成 1，避免和控制循环抢核 |
| TypeScript `interface` | 图上的名字（`obs` / `actions`）是契约；写错名字会在 Runtime 里失败 |

velstand 是 feed-forward（1 入 1 出），不是 LSTM。同输入同输出；没有跨拍隐藏状态要回灌。

## 最小示例

不经过 `src/policy.rs`，只打开权重文件看契约并推理一次：

```bash
export ORT_DYLIB_PATH=third_party/onnxruntime/lib/libonnxruntime.so
docker compose exec -e ORT_DYLIB_PATH=/work/third_party/onnxruntime/lib/libonnxruntime.so \
  rust cargo run --example ex_onnx_session
```

实测（直立重力 `obs[5]=-1`，其余 0）：

```text
inputs:  1 [("obs", "Tensor { ty: Float32, shape: [1, 61], ... }")]
outputs: 1 [("actions", "Tensor { ty: Float32, shape: [1, 14], ... }")]
action_len: 14
action: -0.048791,-0.083072,0.009139,-0.015095,-0.138913,-0.091922,0.010701,-0.039631,-0.017179,0.080170,0.100976,0.046658,0.000604,0.194302
all_finite: true
```

同一契约也由 `cargo test policy::tests::velstand_zero_obs_then_feedback_stays_finite` 覆盖（零观测推理 → 把动作回灌再推理，输出仍有限）。

## 素材

- ONNX Runtime 入门（Python，概念可迁移）：https://onnxruntime.ai/docs/get-started/with-python.html
- 本仓库权重来源（v5，顶层 `model_api: 1`）：https://huggingface.co/pollen-robotics/microduck-policies/tree/v5
- velstand 条目原文：*Walking gait that also stands still at zero command; the default walk policy since v5, with no separate standing network.*

不需要自制动画：通用概念官方文档已讲清；本项目特有的是维数 61/14 和「零指令站立」语义。

## 回到项目

- 打开、校验、warmup：`Policy::load`（`src/policy.rs:85-144`）
- 每拍前向：`Policy::infer`（`src/policy.rs:147-193`）
- 维数常量：`OBS_LEN` / `ACTION_LEN`（`src/obs.rs:10-11`）
- 动作铺到关节并缩放低通：`apply_action`（`src/control.rs:59-86`）

`policy.rs` 里有 LSTM 状态分支（3 入 3 出）。velstand 是 1 入 1 出，`state` 为 `None`，那条路径当前不执行。现在不要顺着它学；见偏差 D20。

## 自测

load 时把观测宽校验成 61。若权重的 `obs` 实际是 `[1, 51]`，进程应该在打开 socket 之前失败，而不是以 50 Hz「正常」推理并表现为走路不稳。哪条测试钉住了这件事？

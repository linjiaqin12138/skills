# 第 4 章 · M3：加载 velstand，闭环站立

> 本章对应里程碑 M3（Bite 4，已验收）。交付物：`src/policy.rs`（ONNX 会话）、`src/control.rs` 的 `apply_action` 与 Skip 控制循环，以及 `src/main.rs` 在 bind 之前加载权重。权重与 Runtime：`scripts/fetch-m3.sh`。

## 1. 动机

M2 已经能每拍拼出一条 61 维观测，但电机目标仍是插值到 home。策略网络还没进环：没有「看观测 → 出动作 → 写关节」的闭环。

这一章在依赖树上卡在 M2 和 M4 之间：

- **上游**：观测布局（M2）必须和训练契约一致，否则装上权重也是在对网络撒谎。
- **本章**：打开 `velstand.onnx`，load 时校验 61/14，插值满 100 拍后开始推理，目标 = home + 0.9 × scatter(action)，再低通。
- **下游**：M4 才接速度命令和仿真行走。本章命令仍是默认零——零指令下 velstand 自己站着，这正是选它的理由。

验收不看机器人会不会走，只看：进程能加载真实权重、站立后姿态停在 home 附近、上一拍动作回灌进观测。

## 2. 概念铺垫

### 【背景卡片】ONNX 会话 / 策略当函数

- **一句话定义**：打开 `.onnx` 得到一个会话；每拍把 `[1, 61]` 的 `float32` 观测塞进去，取出 `[1, 14]` 的动作。像无 HTTP 的本地函数调用。
- **为什么需要**：控制循环要 50 Hz 前向。部署端只跑 Runtime，不装训练框架。velstand 是 feed-forward（1 入 1 出），不是 LSTM——同输入同输出，没有隐藏状态要跨拍保存。
- **和 TS/Node 的差异**：可类比 `const actions = await localModel(obs)`，但没有 JSON、没有 URL；维数是硬契约。crate `ort` 像一个 npm 包，真正算图的是动态加载的 `libonnxruntime.so`（`load-dynamic`）。tokio 是多线程运行时，和 Node 单线程事件循环不同；本仓库把推理线程数钉成 1（`src/policy.rs:18`），避免和控制循环抢核。
- **最小示例**（不经过 `src/policy.rs`）：

```bash
export ORT_DYLIB_PATH=third_party/onnxruntime/lib/libonnxruntime.so
docker compose exec -e ORT_DYLIB_PATH=/work/third_party/onnxruntime/lib/libonnxruntime.so \
  rust cargo run --example ex_onnx_session
```

实测：

```text
inputs:  1 [("obs", "Tensor { ty: Float32, shape: [1, 61], ... }")]
outputs: 1 [("actions", "Tensor { ty: Float32, shape: [1, 14], ... }")]
action_len: 14
action: -0.048791,-0.083072,0.009139,-0.015095,-0.138913,-0.091922,0.010701,-0.039631,-0.017179,0.080170,0.100976,0.046658,0.000604,0.194302
all_finite: true
```

同一契约的单测（已跑过）：`cargo test policy::tests::velstand_zero_obs_then_feedback_stays_finite` —— 零观测出 14 个有限动作，回灌后再推理仍有限。

卡片看不懂就先读 [ONNX 策略会话](../../concepts/onnx-policy.md)。素材：[ONNX Runtime 入门](https://onnxruntime.ai/docs/get-started/with-python.html)、[HF microduck-policies v5](https://huggingface.co/pollen-robotics/microduck-policies/tree/v5)（manifest 顶层 `model_api: 1`）。

### 【决策卡片】单文件 velstand + scale 0.9

- **决策点**：站立用哪份权重、动作乘多少再加到 home。
- **备选**：
  1. 只加载 `velstand.onnx`（walk 槽），`action_scale = 0.9`；
  2. 同时加载独立 stand 网络，站立时用 `standing_action_scale = 1.0`。
- **选择**：备选 1。HF v5 写明 velstand 是 *default walk policy since v5, with no separate standing network*；零指令时自己站着。`0.9` 来自原版 `Tuning::default` 的原型 alpha 默认值（`reference/robotd/src/control.rs:66-76`）。本仓库**没有**测量说明为什么不是 1.0。站立网络注释里的 1.0 是「该策略按完整幅度训练」——本里程碑没加载那份权重。
- **放弃的成本**：没有「站立专用」增益/幅度切换；多策略调度（stand 槽、优先级链）整段推到 M6（D3）。若将来权重改回「walk 不会原地站、必须另挂 stand」，本章的单一文件假设失效。
- **失效边界**：换一套不是围绕这组 home / 0.9 / 低通系数训练的权重；或命令不再恒零却仍假定「零指令 = 站立」。低通必须和训练一致——注释写在 `src/control.rs:27-31`。

## 3. 实现走读

**下载与 Runtime。** `scripts/fetch-m3.sh` 经代理 `http://172.19.160.1:7890` 拉 `velstand.onnx` 和 ONNX Runtime ≥ 1.23（脚本钉 1.23.2）。`Cargo.toml` 里 `ort = "=2.0.0-rc.11"`，`load-dynamic`：进程用 `ORT_DYLIB_PATH` 找到 `libonnxruntime.so`，不是把 Runtime 静态链进二进制。

**load 时校验维数。** `Policy::load`（`src/policy.rs:85-144`）先读文件（打不开就 `Read` 错误），再建 Session，对名为 `obs` 的输入和动作输出跑 `check_matrix`（`src/policy.rs:263-278`）：必须是 float32、两维、宽分别为 61 和 14。错了现在失败，而不是跑起来像调参问题。随后用全零观测 warmup 一次，把冷启动挡在控制循环外，再 `reset_state`。

**为什么在 bind 之前退出。** `main`（`src/main.rs:27-31`）加载失败就 `eprintln` 并返回错误；Unix socket 在更后面才 `bind`（`src/main.rs:45-46`）。权重坏了不会出现「socket 活着、健康恒 true、机器人却没有策略」的半活状态。这和原版「抱住姿态 + unhealthy」不同，记为 D19，预定 M5 与 D11 一起收敛。

**插值 100 拍不推理。** 控制循环（`src/control.rs:187-189`）在 `tick_in_ramp < RAMP_TICKS` 时只跑 `ramp_target`，`action_out` 全 0，`last_action` 保持 0（`src/control.rs:132-133`）。策略的 `entry_pose` 是 standing——先走到 home 再交给网络。满 100 拍后才 `policy.infer`（`src/control.rs:191-197`）。

**目标怎么从动作来。** `apply_action`（`src/control.rs:59-86`）：`scatter_action` 把 14 维铺回 15 关节（嘴保持 0），再 `home + 0.9 * offset`。有 `previous` 时头关节 5..8 用 alpha 0.5、腿用 0.7 做一阶低通；嘴始终等于 home。第一拍 `previous` 为 `None`，不滤波（`src/control.rs:134-135`、`69-81`）。常量对齐原版 `Tuning::default` / walk 模式。

**上一拍动作回灌。** 观测用**当前** `last_action` 组装，成功推理**之后**才更新（`src/control.rs:176-185`、`195`）。因此订阅帧里 `obs[34..48]` 是上一拍输出，与本帧 `action` 不相同——这是回灌，不是同一拍拷贝。D17 在本里程碑收敛。

**MissedTickBehavior::Skip。** `src/control.rs:124-125`。原版实测 `Delay` 掉到 43.1 Hz（`reference/robotd/src/main.rs` 约 1834–1849）。本项目仍用 `interval_at`，第一拍推迟一个周期（`src/control.rs:119-123`）。D16 收敛。

**不要顺着 LSTM 分支学。** `src/policy.rs:105-130` 按张量个数区分 feed-forward / recurrent，并分配 `LstmState`；`infer` 里有拷回状态的路径（`src/policy.rs:180-189`）。velstand 是 1 入 1 出，`state` 为 `None`，这些行当前不执行。铁律：当前里程碑不提前实现。记为 D20——第一次真正加载 recurrent 文件时核销；若到 M8 仍没有，删掉死代码。LSTM 本身见 [带便签的函数](../../concepts/lstm.md)。

## 4. 验收

```bash
# 容器内（不要包 bash -lc，否则 PATH 可能丢，见常见坑）
docker compose exec rust cargo test
```

15 passed, 0 failed。新增：`apply_action` 两条、`rejects_wrong_observation_width`、`velstand_zero_obs_then_feedback_stays_finite`。

宿主机（`ORT_DYLIB_PATH` 指向 `third_party/onnxruntime/lib/libonnxruntime.so`）：

```bash
export ORT_DYLIB_PATH=third_party/onnxruntime/lib/libonnxruntime.so
./target/debug/miniduckd &
sleep 4
./target/debug/mini-duckctl health
timeout 2 ./target/debug/mini-duckctl subscribe
pkill -x miniduckd
```

health（主 agent 复跑原文）：

```text
hello <- {"jsonrpc":"2.0","id":1,"result":{"api_version":1,"service":"miniduckd"}}
{"jsonrpc":"2.0","id":2,"result":{"healthy":true,"reads":198,"skipped_reads":0,"tick":198,"uptime_s":4,"writes":198}}
```

subscribe 第一帧（tick=198）原文：

```json
{"jsonrpc":"2.0","method":"robot.state","params":{"action":[0.08304142951965332,-0.07303529977798462,0.03339257091283798,-0.1969955712556839,0.15026922523975372,-0.09230320900678635,0.060564205050468445,-0.12751872837543488,0.03637963533401489,-0.023319348692893982,-0.008788954466581345,-0.010213661938905716,0.04183083772659302,-0.1898752748966217],"imu":{"gravity":[0.0,0.0,-1.0],"gyro":[0.0,0.0,0.0],"quat":[1.0,0.0,0.0,0.0]},"obs":[0.0,0.0,0.0,0.0,0.0,-1.0,-0.0713433176279068,0.005063704680651426,0.009753949008882046,-0.01877407915890217,-0.1799294501543045,-0.05903230607509613,-0.012959652580320835,-0.061287615448236465,-0.007611006498336792,0.05980410426855087,0.03727089986205101,0.05026549473404884,0.03644273430109024,0.15298353135585785,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,-0.12796390056610107,0.029224850237369537,0.0040713027119636536,0.03198055177927017,-0.3049789071083069,-0.05223552882671356,-0.051881588995456696,-0.03838670253753662,-0.030874788761138916,0.09337951242923737,0.05647241696715355,0.07566983997821808,0.04009019583463669,0.2779388129711151,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],"positions":[-0.0713433168731369,-0.08223629548116002,-0.44814605090379267,-0.02367407856095489,0.2730705444580334,0.29006769283086203,0.3361403473284782,-0.06128761556089236,-0.007611006388276364,0.0,0.059804105023306915,0.12457089950620528,0.5081654929566595,0.041342736022789844,-0.3000164734233878],"tick":198}}
```

守护进程日志：`tick_rate=50.0Hz p99_jitter_ms=0.00`。

**断言解释**

- `obs` 长 61，下标 5 为 -1（重力 z）。
- `action` 长 14。
- 相对 home（`src/model.rs` `DEFAULT_POSITION`）最大偏差在左踝：`|0.2730705444580334 − 0.4530| ≈ 0.180 < 0.25`。这是「没走掉」的上界，不是逐关节精确答案。
- `obs[34..48]` 与本帧 `action` 不相同——上一拍回灌，不是同一拍拷贝。

## 5. 与原版差异

| 项 | 现在 | 收敛 |
|---|---|---|
| D3 | 单策略、无调度器（本里程碑引入） | M6 |
| D16 | 控制循环已用 `MissedTickBehavior::Skip` | 本里程碑收敛 |
| D17 | `last_action` 为真实上一拍策略输出 | 本里程碑收敛 |
| D18 | command 恒默认零 | M4 |
| D19 | load 失败直接退出，不做抱住姿态 + unhealthy | M5（与 D11） |
| D20 | `policy.rs` 含 LSTM 分支，velstand 不走；当前死代码 | 首次加载 recurrent 文件时核销；否则 M8 删除 |

`examples/ex_onnx_session.rs` 与测试用的 python3-onnx 写坏图，是教程/测试资产，不计入对原版结构的偏差。

## 6. 常见坑

1. **代理下载。** HF / GitHub 直连常超时。用 `scripts/fetch-m3.sh`（默认代理 `http://172.19.160.1:7890`）。WSL 里 `127.0.0.1:7890` 到不了 Windows 主机上的代理。
2. **`docker compose exec` 不要包 `bash -lc`。** 登录 shell 可能丢掉容器里为 `cargo` 配好的 PATH，表现为找不到 `cargo`。直接 `docker compose exec rust cargo test`。
3. **`SessionOutputs` 要先 drop 才能改 LSTM 状态。** `src/policy.rs:180-189` 先拷结果再 `drop(outputs)`。这是死分支上的坑——velstand 不走进去；真加载 recurrent 权重时再认真看，现在不要把它当成必需实现。

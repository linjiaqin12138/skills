# Bite 3 · M2：61 维观测向量（已通过验收）

主 agent 复跑。代码在子代理中途卡住前已经落盘，注释里一处因果写反已改掉后再验收。

## 交付

- `src/obs.rs`：`Observation::build` / `scatter_action`，命名偏移，嘴的映射只有 `joint_of`
- `src/control.rs`：每拍成功 read 后组观测，`FrameSnapshot` 经 watch 送出。电机目标仍是插值。`last_action` 与 `Command` 传零
- `src/main.rs`：`robot.state` 增加 `imu` 与 `obs`，仍 1 Hz
- `src/lib.rs`：`pub mod obs`
- `examples/obs_layout.mjs`：概念示例，不是收敛对照项

## 验收实测

`docker compose exec rust cargo test`：11 passed, 0 failed。新增 5 个 obs 测试（宽度、六块边界、home 相对角为 0、嘴不进观测、scatter 跳过嘴）。

站稳后 `mini-duckctl subscribe` 的一帧（tick=200）：

```json
{"jsonrpc":"2.0","method":"robot.state","params":{"imu":{"gravity":[0.0,0.0,-1.0],"gyro":[0.0,0.0,0.0],"quat":[1.0,0.0,0.0,0.0]},"obs":[0.0,0.0,0.0,0.0,0.0,-1.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],"positions":[0.0,-0.0873,-0.4579,-0.0049,0.453,0.3491,0.3491,0.0,0.0,0.0,0.0,0.0873,0.4579,0.0049,-0.453],"tick":200}}
```

断言：`obs` 长度 61；下标 5（重力 z）为 -1，与 `imu.gravity` 一致；下标 6..19 为 0（已在 home，相对位置为 0）；`positions` 与 `DEFAULT_POSITION` 一致（JSON 把 0.4530 印成 0.453）。

## 偏差

- D14：`imu` 已补上。推送仍 1 Hz，50 Hz 留到 M5
- D17：`last_action` 恒 0，M3 收敛
- D18：`command` 恒默认，M4 收敛
- `scatter_action` 有单测，控制循环仍用插值写电机，M3 才拿它铺动作

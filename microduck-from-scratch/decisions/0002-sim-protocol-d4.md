# 0002 仿真线协议对齐官方 gain/torque 面

- 状态：已采纳
- 来源：手搓
- 出现于：m09a-b04
- 被取代于：

起草可以改。用户确认这一课之后，手搓决定改为「已采纳」。用户确认 Phase 1 之后，原项目决定改为「已记录」。此后不改正文。要推翻，新开一篇，本篇状态改为「被取代」，并填上新编号。

## 上下文

D4 登记自 M4：线协议形状对齐原版 `duck-control/src/sim.rs`，帧细节自定。M9a 已具备 PROTOCOL=1、15 槽含嘴、trunk 系投影重力；残余是线上 op 名仍用教学期的 `set_gain`/`set_torque`，字段名 `gain` 而非官方 `kp`。官方对照是 `reference/duck-control/src/sim.rs` 的 `Request` 与 `reference/microduck_rl/.../body_server.py` 的 `dispatch`。验收脚本仍依赖教学附加 op `push`/`body` 与 read 回显 `gain`/`torque`/`body_pos`。

## 备选

1. 线协议改用官方 op 名 `gain`（字段 `kp`）与 `torque`（字段 `on`）；SimIo 与 duck_body 同改；保留 `push`/`body` 与 read 回显作为教学附加帧；`slow`/`tof`/`currents_ma` 本 Bite 不扩面（`slow_sensors` 仍归 D12→M10）。
2. 同时实现官方全 op 面（`gain`/`torque`/`slow`/`tof`）并去掉教学附加帧，逼既有 accept 改走别的观测路径。
3. 服务端同时接受新旧 op 名做兼容层，SimIo 先发新名。

## 决定

采用备选 1。本 Bite 收敛 D4 的可观察协议差：op 名与 gain 字段。已对齐的 PROTOCOL=1、15 槽嘴映射、trunk 重力用验收断言钉死，不重做物理。`push`/`body`/read 回显保留并写进课文注意事项——官方 serde 会忽略未知字段，教学工具需要它们。不实现 `slow`/`tof`，不把 `currents_ma` 做成必填。

## 放弃的成本

备选 2 能一次与 body_server 的 op 表齐名齐行；放弃它的代价是本 Bite 不碰 ToF 资产与慢速传感面，既有跌倒/位置验收不必改观测通道。

备选 3 能降低一次性改线的回归风险；放弃兼容层后，旧 op 必须立即失败，验收能正面证明「已不再说 set_gain」，但所有直连仿真的脚本与单测必须同 Bite 改完。

## 失效边界

若官方 PROTOCOL 升到 2 且改了 hello/read 形状，本决定只覆盖 PROTOCOL=1 的命名对齐，必须新开决定。若 M10 落地 `slow_sensors`/`tof`，教学附加帧是否仍保留再裁决。

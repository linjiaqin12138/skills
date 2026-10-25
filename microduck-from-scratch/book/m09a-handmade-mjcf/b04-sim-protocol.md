---
template: sheet
title: M9a Bite 4：仿真线协议对齐（第 21 章）
subtitle: 线上 op 改为 gain/torque；字段 kp/on；钉死 PROTOCOL、15 槽与 trunk 重力
source: clone-from-scratch-tutor · 第 21 章
---

自建身体已能走。本课只改线协议命名：`SimIo` 发出官方 `gain`/`torque`，`duck_body` 按 `kp`/`on` 读字段。旧名 `set_gain`/`set_torque` 必须失败。102 项测试与冒烟联调均通过。

```callout ok 收敛 D4 的可观察协议差
改的是帧上的 op 名与 gain 字段。PROTOCOL=1、15 槽嘴映射、trunk 系重力保持原状，用断言钉死。
```

## 问题

Bite 1–3 已交付自建身体与 BAM 驱动。线协议形状仍停在教学期：控制侧方法叫 `set_gain`/`set_torque`，线上却发 `set_gain`/`set_torque`，字段还叫 `gain`。

官方对照是 `reference/duck-control/src/sim.rs` 的 `Request`，以及 `reference/microduck_rl/.../body_server.py` 的 `dispatch`。线上 op 名是 `gain`（字段 `kp`）与 `torque`（字段 `on`）。名字不对，帧就对不上官方服务端。

D4 从 M4 起登记「形状对齐、帧细节自定」。PROTOCOL=1、15 槽、trunk 重力已齐。本课收敛残余命名差。

## 设计

```sequence num
participants: SimIo, duck_body.py
SimIo -> duck_body.py: hello protocol=1 joints=15
duck_body.py --> SimIo: protocol joints
SimIo -> duck_body.py: gain kp
duck_body.py --> SimIo: empty ok
SimIo -> duck_body.py: torque on
duck_body.py --> SimIo: empty ok
SimIo -> duck_body.py: write targets×15
duck_body.py --> SimIo: empty ok
SimIo -> duck_body.py: read
duck_body.py --> SimIo: sensors + 教学回显
```

daemon 侧 `RobotIo` 方法名仍是 `set_gain`/`set_torque`。发出的 JSON 用官方 op 名。仿真侧只认新名；旧名回 `unknown op`。

`kp` 的舵机含义见[舵机 gain 与 torque](../concepts/servo-gain-torque.html)。本课只对齐线上字段名。

**来源：decisions/0002-sim-protocol-d4.md**

```text
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
```

## 代码领读

`handle` 按 `op` 分派。`gain` 读整型 `kp`（0..200），`torque` 读布尔 `on`。两者都调用 `_apply_gains`，再回空对象。

```python src=impl/sim/duck_body.py lines=361-375 hl=361-374
```

`hello` 校验 `protocol` 与 `joints`。官方 `body_server` 通常只验 protocol；本仓多验关节数，握手更严。

```python src=impl/sim/duck_body.py lines=351-356 hl=352-356
```

`bus_order` 把 14 维关节序扩成 15 槽，嘴位补 0。`write` 要求长度正好 15。

```python src=impl/sim/duck_body.py lines=281-286
```

`sensors` 把世界重力旋进 trunk 系，直立时约 `[0,0,-1]`。响应里仍带教学回显 `gain`/`torque`/`body_pos`。

```python src=impl/sim/duck_body.py lines=288-309 hl=294-308
```

`push` 给躯干加水平速度；`body` 只查位置不步进。二者不是官方 op，验收脚本仍依赖它们。

```python src=impl/sim/duck_body.py lines=376-388 hl=376-388
```

Rust 侧方法名不变。`SimIo::set_gain` 发出 `"op":"gain"` 与 `"kp"`；`set_torque` 发出 `"op":"torque"` 与 `"on"`。

```rust src=impl/src/io.rs lines=375-390 hl=377-389
```

单测截获发出的帧，断言新 op 名与字段，防止回退到旧名。

```rust src=impl/src/io.rs lines=620-629 hl=626-629
```

| 层 | 名字 | 含义 |
|---|---|---|
| `RobotIo` 方法 | `set_gain` / `set_torque` | daemon 内部调用面 |
| 线协议 op | `gain` / `torque` | JSON 帧上的官方名 |
| 字段 | `kp` / `on` | 官方 Request 字段 |

## 验收

在仓库根执行容器命令：

```bash
docker compose -p microduck-from-scratch -f impl/docker-compose.yml exec -T rust bash /work/accept/m09a-b04.sh
```

脚本注释给出六组预期，见 `accept/m09a-b04.sh:7-13`：

| 组 | 验收意图 |
|---|---|
| A | `protocol=1` 握手成功；错误 protocol 被拒绝 |
| B | 线上 op 为 `gain`/`torque`；旧名必须失败 |
| C | write/read 走 15 槽；直立 `imu.gravity` ≈ `[0,0,-1]` |
| D | 教学附加 `push`/`body` 仍可用 |
| E | `cargo test` 全绿（含 SimIo 帧断言） |
| F | BAM 默认 handmade 冒烟：enable 后 2s 内 `skill=walk` 且 `fallen=false` |

实测见 `tutor/runs/m09a-b04.txt`：

- 102 tests passed；`sim_io_set_gain_and_torque_frames` 通过。
- `protocol asserts PASS`（A–D）。
- `smoke PASS walk fallen=False`（F）。
- 退出码 0。verifier 结论：**通过**。

## 与原版差异

收敛 [D4](../../spec/deviations.md)。线上 op 与 gain 字段对齐官方 `sim.rs` / `body_server`。

仍保留教学附加：`push`/`body`，以及 read 回显 `gain`/`torque`/`body_pos`。官方 serde 会忽略未知字段；本仓验收需要它们。`slow`/`tof`/`currents_ma` 本课不扩面。

`hello` 仍校验 `joints`。官方 body_server 通常只校验 protocol。本仓多验关节数，属于更严握手，不是协议名回退。

## 自测

```html
<details>
<summary><b>1.</b> daemon 调用 <code>set_gain(50)</code>。线上 JSON 应是哪项？（A）<code>{"op":"set_gain","gain":50}</code>　（B）<code>{"op":"gain","kp":50}</code>　（C）<code>{"op":"gain","gain":50}</code>　（D）<code>{"op":"kp","value":50}</code></summary>
<p><b>答案：B。</b>方法名仍是 <code>set_gain</code>。线协议 op 是 <code>gain</code>，字段是 <code>kp</code>。见 <code>impl/src/io.rs:375-381</code>。</p>
</details>
<details>
<summary><b>2.</b> 客户端发送 <code>{"op":"set_torque","on":false}</code>。仿真应如何响应？（A）卸力并回空对象　（B）回 <code>unknown op</code> 错误　（C）静默忽略　（D）自动改写成 <code>torque</code></summary>
<p><b>答案：B。</b>无兼容层。旧 op 必须立即失败，验收据此证明已不再说旧名。</p>
</details>
<details>
<summary><b>3.</b> 直立时 <code>read</code> 的 <code>imu.gravity</code> 约为 <code>[0,0,-1]</code>。该向量在哪个坐标系？（A）世界系　（B）trunk 系　（C）脚掌系　（D）相机系</summary>
<p><b>答案：B。</b><code>sensors</code> 把世界重力旋进躯干系。见 <code>impl/sim/duck_body.py:294</code>。</p>
</details>
<details>
<summary><b>4.</b> 验收脚本调用 <code>op=body</code> 测位移。相对官方协议，这属于哪类？（A）官方必选 op　（B）教学附加帧　（C）PROTOCOL=2 新字段　（D）慢速传感面</summary>
<p><b>答案：B。</b><code>push</code>/<code>body</code> 与 read 回显是教学附加。官方 serde 忽略未知字段；本仓验收依赖它们。</p>
</details>
<details>
<summary><b>5.</b> 本课收敛 D4 后，哪项仍故意不做？（A）钉死 PROTOCOL=1　（B）实现 <code>slow</code>/<code>tof</code>　（C）15 槽含嘴　（D）旧名必须失败</summary>
<p><b>答案：B。</b>决定采用备选 1：不扩 <code>slow</code>/<code>tof</code>，不把 <code>currents_ma</code> 做成必填。慢速传感仍归 D12→M10。</p>
</details>
```

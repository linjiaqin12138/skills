# M8 收敛验收 — 验收档案

> M8 按 Bite 推进，本档案逐 Bite 追加。当前收录：Bite 1（robot.drive → robot.move）、Bite 2（robot.enable {on, toggle} + notification 统一入口 + disable 语义对齐）。

## Bite 1：robot.drive → robot.move（通知式连续意图 + vy 侧向）

### 交付物

- `src/lib.rs`：`Request.id` → `Option<u64>`（无 id = JSON-RPC notification，`skip_serializing_if` 保证序列化回去不带 `"id"` 字段）；`ServerMessage::Response.id` 同步 `Option<u64>`（解析失败按规范回 `"id":null`，修掉旧实现的假 id 0）；新增 `MoveParams`（`#[serde(default, deny_unknown_fields)]`，doc 注释含承重的单位/坐标系约定：m/s 与 rad/s、躯干系 x 前 y 左 z 上、vyaw 正左转）+ `finite_twist()` 有限性校验（线路不可达的深度防御）；8 条新单测。
- `src/main.rs`：删 `robot.drive` 分支，新增 `robot.move`（无 id 静默应用/静默丢弃，有 id 回 accepted 或 INVALID_PARAMS）；抽出 `parse_move` 与 `apply_move_intent`（通知/请求两路共用，只写 twist + 刷新 `last_intent_at`，无限幅）；分发 match 改返回 `Option<ServerMessage>`（`None` = 本帧无回复）。
- `src/bin/mini-duckctl.rs`：`drive <vx> <vyaw>` → `move <vx> <vy> <vyaw> [--secs N]`；仍请求式逐条调用（打印 accepted 回显）；`--secs` 期间 100ms 重发扮演手柄、到时发零命令停车（deadman 适配不变）。
- `src/updater/ipc.rs` + `src/updater/gate.rs`：id 类型适配（`Option<u64>`），语义不变；ipc.rs 解析失败路径同样从假 id 0 改 `"id":null`。
- `src/control.rs`：两处注释 `robot.drive` → `robot.move`（无代码变动）。
- `scripts/accept-m8.sh`（新建）：FakeIo 起 daemon，python3 裸 socket 发帧（CLI 发不了无 id 通知），单连接时间线断言 a–h。
- `scripts/accept-m4.sh` / `accept-m5.sh` / `accept-m6.sh`：CLI 调用 `drive 0.15 0 …` → `move 0.15 0 0 …`（vy=0 保持原行为）。
- **控制循环 / 调度器 / Safety / RobotIo / 仿真体一行未动**；进程边界与模块零增删。

### 验收实测

`docker compose exec rust cargo test`（2026-10-05 复跑）：

```text
test result: ok. 68 passed; 0 failed
```

构成：M7 基线 60 条 + 本章新增 8 条（全部在 `src/lib.rs`：move 参数全字段/缺省/未知字段/错误类型四条、非有限拒绝一条、JSON 溢出解析期拒绝一条、notification 帧形态一条、`"id":null` 响应一条）。

`docker compose exec rust bash scripts/accept-m8.sh`（2026-10-05 复跑实测关键行）：

```text
握手 + 订阅  OK
a. 通知生效 twist=[0.10000000149011612, 0.20000000298023224, -0.30000001192092896]  OK
b. 通知全程无响应行  OK
c. 请求式 accepted 回显 [0.05, -0.05, 0.1]  OK
d. 缺省 vy 按 0 接受 {'accepted': True, 'vx': 0.1, 'vy': 0.0, 'vyaw': 0.2}  OK
e1. 未知字段 INVALID_PARAMS  OK
e2. 非法通知静默丢弃，15 帧 twist 保持 [0.3, 0.1, -0.2]  OK
f. 1e999 → PARSE_ERROR id=null: number out of range  OK
g. 停发 0.7s 后 obs twist=[0,0,0]（deadman 生效）  OK
h. robot.drive → METHOD_NOT_FOUND  OK

M8（本 Bite）断言 a–h 全部通过
```

断言要点：a 中 twist 经 obs f32 落盘（`src/obs.rs:105`），容差 1e-6，偏移 `OFF_TWIST=48`（`src/obs.rs:22`）；e2 的 15 帧 = 300ms（< deadman 500ms）内的全部推送；f 的帧是手写字面量（python `json.dumps(1e999)` 会印成非法 JSON 的裸 `Infinity`，测不成数值溢出——见 tutorial「常见坑」1）。

**回归**：

- `docker compose exec rust bash scripts/accept-m7.sh`：PASS=39 FAIL=0（updater id 适配语义不变）。
- `accept-m4.sh` / `accept-m5.sh` / `accept-m6.sh`（MuJoCo 仿真，仅改了脚本里的 CLI 调用）：10 秒行走位移实测 0.764m / 0.848m / 0.816m，均过 0.5m 门槛，exit 0——vy=0 的新发法与旧发法行为一致。

**实现期踩坑记录**（已修复并变成断言）：① 断言 f 第一版假阳性——`json.dumps(1e999)` 印成裸 `Infinity`（非法 JSON 词法），daemon 回词法错误而非数值溢出；改为脚本内手写帧字面量 `"vx":1e999` 才真正测到 serde_json 的 number out of range（`scripts/accept-m8.sh:153-161`）；② `finite_twist()` 的 `is_finite` 检查线上不可达（`serde_json::Value` 装不下 inf/NaN，解析阶段即拒），作为深度防御保留，由单测直接构造 NaN/±inf 验证（`src/lib.rs:167-175`）；③ CLI 发不了无 id 通知，验收脚本必须用 python3 裸 socket 发帧（`scripts/accept-m8.sh:36-46`）。

### 涉及代码文件清单

| 文件 | 变动 |
|---|---|
| `src/lib.rs` | `Request.id`/`Response.id` → `Option<u64>`；+`MoveParams`/`finite_twist()`；+8 单测 |
| `src/main.rs` | 删 robot.drive，+robot.move 双路径；+`parse_move`/`apply_move_intent`；match 返回 `Option<ServerMessage>` |
| `src/bin/mini-duckctl.rs` | `drive` → `move vx vy vyaw [--secs N]` 子命令 |
| `src/updater/ipc.rs` | id 类型适配（`Option<u64>`），解析失败回 `"id":null` |
| `src/updater/gate.rs` | 发请求 id 包 `Some` |
| `src/control.rs` | 两处注释改名（robot.drive → robot.move） |
| `scripts/accept-m8.sh` | 新建：a–h 八项断言 |
| `scripts/accept-m4/m5/m6.sh` | CLI 调用改名（vy=0） |
| `examples/jsonrpc_notification.py` | 新建：教程概念演示（request/notification 双形态） |
| `docs/concepts/jsonrpc-notification.md` | 新建：概念深读 |
| `docs/milestones/m8-convergence/{tutorial,acceptance}.md` | 本档案与教程章节 |
| `docs/milestones/m8-convergence/{arch,arch-diff}.{d2,svg}` | 架构图（M7 图改标签；变动示意灰化未动模块） |

### 偏差变动

- **D43 部分收敛**（已更新 `docs/deviations.md`）：robot.drive {vx,vyaw} 请求式 → 原版 robot.move {vx,vy,vyaw} 通知/请求双形态，vy 侧向补齐。残余：① robot.head / robot.mouth 的通知语义未开（M8 后续 Bite）；② 原版对任何方法的 notification 都走 apply_intent 统一入口静默处理，我们只给 robot.move 开了通知语义（后续 Bite 随 enable/subscribe 收敛）；③ mini-duckctl 仍请求式逐条调用（教学工具，daemon 两种形态都收）。

---

## Bite 2：robot.enable {on, toggle} + notification 统一入口 + disable 语义对齐（Stopped 取代 RampDown）

### 交付物

- `src/lib.rs`：新增 `EnableParams`（`{on: bool}` 必填无 default、`toggle` 缺省 false、`deny_unknown_fields`，doc 注释承重：toggle=手柄 Start 语义、daemon 侧翻转、客户端信念会漂移）+ `parse_enable`（仿 `parse_move`，Err 文案即 INVALID_PARAMS message）；6 条新单测（on=true / toggle 缺省 / on+toggle 同帧 / 单发 toggle 拒 / 未知字段拒 / 缺 on 与 Null 拒）。
- `src/main.rs`：notification 统一入口（`src/main.rs:282-302`）——任何方法的无 id 帧都不应答：move/head/mouth 解析合法静默应用、非法静默丢弃，非意图方法静默丢弃；整帧解析失败仍在 parse 分支回 PARSE_ERROR + `"id":null`。`robot.enable` 带 id 分支重写（`src/main.rs:319-340`）：toggle 优先（daemon 侧翻转，忽略 on）、永不拒绝、reason 文案逐字对齐原版；`robot.disable` 分支删除（→ METHOD_NOT_FOUND）。抽取 `parse_mouth`/`apply_mouth_intent`/`parse_head`/`apply_head_intent`（`src/main.rs:213-243`），通知/请求两路共用。
- `src/control.rs`：状态机改造——`Phase::Stopped`（上电抱持 home）取代 `RampDown`；模块头 ASCII 状态图重写（`src/control.rs:1-29`）；转移规则抽纯函数 `edge_transition()`（`src/control.rs:307-319`，`EdgeAction` doc 注释逐条引原版证据）与 `ramp_done_phase()`（`src/control.rs:322-328`）；`StopToHome` 边沿处理（`src/control.rs:447-457`：scheduler.reset + 锚点/last_action 清零 + 切 Stopped）；Stopped 臂当拍写 `DEFAULT_POSITION`、gain 维持 200、torque 保持 on（`src/control.rs:530-537`）；全文件无 `set_torque(false)` 路径。顺带修掉"RampDown 途中再 enable 卡死 Held"的自家 bug。6 条新单测（`src/control.rs:723-776`）。
- `src/scheduler.rs`：`Scheduler::reset()`（`src/scheduler.rs:398-413`）清全部已加载槽 LSTM；Cascade 窗口/坐姿锁存不动【残余存疑，见偏差变动】；单测 `reset_clears_every_loaded_slot`（`src/scheduler.rs:488-523`，增量断言）。
- `src/bin/mini-duckctl.rs`：`enable [on|off]`（缺省 on）发 `robot.enable {on}`；`disable` 子命令删除（用户裁决，不留别名）。
- `scripts/accept-m5.sh`：A1c 重写（`scripts/accept-m5.sh:99-116`）——`enable off` 后断言 positions 精确等于 home（阈值 1e-6）、enabled=false、**torque=true**（原版语义：卸 torque 是 relax 的活）。
- `scripts/accept-m8.sh`：追加断言 i–p + n2 + o2（`scripts/accept-m8.sh:210-333`）；等 Driving 改用推送 `skill` 字段而非 gain（gain=200 在 RampUp/Stopped 同样成立）。
- **进程边界与模块零增删**；变化全在模块内部（main.rs 分发结构、control.rs 状态机、CLI 子命令）。

### 验收实测

`docker compose exec rust cargo test`（2026-10-06 主 agent 复跑）：

```text
test result: ok. 81 passed; 0 failed
```

构成：Bite 1 的 68 条 + 本 Bite 新增 13 条（lib 6 + control 6 + scheduler 1）。

`docker compose exec rust bash scripts/accept-m8.sh`（2026-10-06 主 agent 复跑）——断言 a–p + n2 + o2 全过，Bite 2 新增部分关键实测行：

```text
i. enable on → 'enabled — driving'，推送 enabled=true  OK
j. 无 id robot.head 静默生效 obs[51:55]=[0.1, -0.1, 0.2, 0.05]，全程无响应行  OK
k. 无 id robot.mouth 静默生效 positions[9]=0.524rad（目标 0.524）  OK
l. 无 id robot.health 静默，25 帧内只有 robot.state 推送  OK
m. 带 id head/mouth 请求式回显不变（mouth 回显 0.5）  OK
n. toggle 翻转 开→关 → 'disabled — returning to the home pose'，推送 enabled=false  OK
n2. disable 后 positions 逐位等于 home，26 帧 torque 全为 true  OK
o. on+toggle 同帧 toggle 优先 关→开 → 'enabled — driving'  OK
o2. 1s 内 skill=walk——从 Stopped 直接回 Driving，无 2 秒斜坡  OK
p1. enable 未知字段 INVALID_PARAMS: robot.enable wants {on: bool} (optional toggle: bool): unknown field…  OK
p2. robot.disable → METHOD_NOT_FOUND  OK

M8 断言 a–p 全部通过
```

断言要点：j 的 obs head 块偏移 `OFF_HEAD=51`（直通 command.head 无低通）；k 的 `MOUTH_INDEX=9`，0..1 → −5°..+30°，position=1 目标 0.524rad；n2 的 home 逐位相等阈值 1e-9 依赖 FakeIo 精确回写（`src/io.rs:100-102`、`src/io.rs:340-344`）；o2 的 1s 预算 < 旧斜坡 2s，只可能是 Stopped→Driving 直回；i/n/o 的 reason 文案逐字对齐原版（`reference/robotd/src/main.rs:4500-4510`）。

**回归**（2026-10-06 复跑）：

- `accept-m5.sh` 全过：A1c 实测 `|positions-home|max=0.00e+00`，enabled=false，torque=true——Stopped 上电抱持 home 的端到端证据。
- `accept-m4.sh` PASS（10 秒行走位移 0.764m，过 0.5m 门槛）、`accept-m6.sh` 全过——控制链路行为不变。
- `accept-m7.sh` 本机 D1/E1 两处时序断言 flake（**预先存在**：新机器更快，Busy 窗口抓空），与本 Bite 无关——教训已记入 tutorial「常见坑」4。

**实现期坑位记录**：① gain==200 判不出 Driving（RampUp/Stopped 同样 200），改用推送 skill 字段（`scripts/accept-m8.sh:219-223`）；② `Policy::load` 自带一次 reset，reset_calls 断言用增量（`src/scheduler.rs:511-523`）；③ FakeIo 是精确回写不是一阶滞后，收敛断言可用严阈值（`scripts/accept-m8.sh:244` 注释写"一阶滞后"与 io.rs 自述矛盾，以 io.rs 为准，疑似历史遗留待清理）。

### 涉及代码文件清单

| 文件 | 变动 |
|---|---|
| `src/lib.rs` | +`EnableParams`/`parse_enable`；+6 单测 |
| `src/main.rs` | notification 统一入口（无 id 帧提前拦截）；robot.enable 重写（toggle daemon 翻转/永不拒绝/reason 对齐）；删 robot.disable；+`parse_mouth`/`apply_mouth_intent`/`parse_head`/`apply_head_intent` 抽取 |
| `src/control.rs` | `Phase::Stopped` 取代 `RampDown`；`edge_transition()`/`ramp_done_phase()` 纯函数；模块头 ASCII 图重写；+6 单测 |
| `src/scheduler.rs` | +`Scheduler::reset()`；+1 单测 |
| `src/bin/mini-duckctl.rs` | `enable [on|off]`；删 `disable` 子命令 |
| `scripts/accept-m8.sh` | 追加断言 i–p + n2 + o2 |
| `scripts/accept-m5.sh` | A1c 重写（torque=true + positions 精确等于 home） |
| `docs/deviations.md` | D43 更新（残余①②收敛）、+D64（新登记即收敛） |
| `docs/feature-inventory.md` | D44 标注 M8 Bite 2 收敛 |
| `docs/milestones/m8-convergence/{tutorial,acceptance}.md` | 本档案与教程章节追加 |
| `docs/milestones/m8-convergence/{arch,arch-diff}.{d2,svg}` | 更新到 Bite 2 状态（阶段机 Stopped、enable{on,toggle}、disable 消失） |

### 偏差变动

- **D44 收敛**：`robot.enable {on, toggle}` 全对齐（参数形状 / toggle daemon 侧翻转 / 永不拒绝 / reason 文案）；`robot.disable` 方法与 CLI 子命令一并删除（用户裁决，不留别名）。
- **D64 新登记并即收敛**：disable 语义（旧 RampDown=2s 斜坡+卸 torque vs 原版直接命令回 home+上电抱持）。复核中发现、用户裁决立即对齐不挂账。残余存疑：`Scheduler::reset()` 是否应连带清 Cascade 坐姿锁存，无原版逐行确证【合理推断：按"锁存不是 episode 记忆"判断不动】。连带代价：`robot.relax` 未实现前卸 torque 无出口（已登记）。
- **D43 残余①②收敛**：head/mouth 通知语义已开、非意图方法通知统一静默。D43 仅剩残余③：mini-duckctl move 仍请求式逐条调用（教学工具定位保留）。
- **联动记录**：请求式 move 应答是 `{accepted}` 超集（多回显 vx/vy/vyaw）；head/mouth 回显参数值、缺 `accepted` 字段（原版三者统一回 `IntentResult::accepted()`）——待后续 Bite 裁决。

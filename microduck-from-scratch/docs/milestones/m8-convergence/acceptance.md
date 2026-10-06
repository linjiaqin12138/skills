# M8 收敛验收 — 验收档案

> M8 按 Bite 推进，本档案逐 Bite 追加。当前收录：Bite 1（robot.drive → robot.move）。

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

---
template: sheet
title: M9e Bite 1：自训策略进我们的 daemon（第 29 章）
subtitle: 同一份 ONNX；官方 viewer 能走，换 miniduckd + handmade 也要能走
source: clone-from-scratch-tutor · 第 29 章
---

## 问题

第 28 章（M9d Bite 2）已经用官方 `infer_policy` 证明：云上训出的 `policy_export.onnx` 在本机能站、能走雏形。那条路绕过了我们的 `miniduckd` 和 handmade BAM——权重「能用」还没接到我们自己的栈。

本课要消掉的故障是：自训 ONNX 只在官方 viewer 里跑得通，一换 `miniduckd` + 默认 handmade 场景就站不稳或几乎不走；或者仍用公开 `velstand` 的默认 `action_scale=0.9`，把自训约定压掉一截。

## 设计

```flow TB
(M9d policy_export.onnx) -> MINIDUCK_POLICY
MINIDUCK_POLICY -> *miniduckd
miniduckd -> Sit::Up / walk 槽
Sit::Up / walk 槽 -> MINIDUCK_WALK_ACTION_SCALE=1.0
MINIDUCK_WALK_ACTION_SCALE=1.0 -> duck_body handmade BAM
duck_body handmade BAM -> enable / move 0.3
```

| 层 | 本课约定 |
|---|---|
| 权重 | `tutor/artifacts/m09d/policy_export.onnx`（M9d 官方 export） |
| 策略路径 | `MINIDUCK_POLICY` 指向该文件；不改默认 `policies/velstand.onnx` |
| walk 尺度 | 默认仍 `0.9`（公开 velstand）；自训验收显式 `MINIDUCK_WALK_ACTION_SCALE=1.0` |
| 身体 / 执行器 | 默认 `scene_handmade.xml` + BAM（`duck_body.py` 不改场景参数） |
| 门禁 | 容器内站稳 + `vx=0.3`×10s 前移 ≥ 0.30 m 且未倒 |

**walk `action_scale` 是训练约定。** 公开 HF `velstand` 按 0.9 训；M9d Velocity 按官方 `JointPositionActionCfg.scale = 1.0` 训，官方 `infer_policy` 也是 `target = DEFAULT_POSE + action * 1.0`。daemon 里 `Sit::Up` 走 walk 槽时乘的尺度必须和训练一致——自训用 1.0，仍用 0.9 等于把动作压掉约 10%。ONNX Runtime 怎么把文件装进会话，本课当黑盒；只认「路径对、能起、能推」。

## 架构决定

采用 [0010](../../decisions/0010-m9e-walk-action-scale.md) **备选 1**：环境变量覆盖 walk 尺度。

| 项 | 选择 |
|---|---|
| 默认 | `WALK_ACTION_SCALE = 0.9`，公开 `velstand` 路径不动 |
| 覆盖 | `MINIDUCK_WALK_ACTION_SCALE`；非法 / 非正数回落 0.9 |
| M9e 验收 | 显式设 `1.0`，对齐自训 / 官方 infer |

没选全局改成 1.0：会把已验收的公开 `velstand` 站/走一起放大。没选按文件名猜 scale：脆弱，且和「scale 是权重属性」对不上。

## 代码领读

自顶向下：策略从哪进 → walk 尺度怎么读 → `Sit::Up` 用运行时值。

### 1. 策略路径：`MINIDUCK_POLICY`

`main.rs` 启动时读环境变量，默认仍是仓库内公开 `velstand`；本课验收把它指到 M9d 产物（`75-82`）：

```rust title="main.rs · sketch"
let policy_path = std::env::var("MINIDUCK_POLICY")
    .unwrap_or_else(|_| DEFAULT_POLICY.into());
// ...
let (scheduler, policy_error, load_failure) = match Scheduler::load(
    std::path::Path::new(&policy_path),
    std::path::Path::new(&policy_dir),
) { /* ... */ };
```

加载失败不退出进程（Held + unhealthy）；验收要确认日志里没有 `failed to load policy`。ORT 会话细节不展开。

### 2. 尺度旋钮 → `Sit::Up`

解析与读取（`scheduler.rs:122-137`）：

```rust title="scheduler.rs · walk_action_scale · sketch"
pub const WALK_ACTION_SCALE: f64 = 0.9;

pub fn parse_walk_action_scale(raw: Option<&str>) -> f64 {
    match raw {
        Some(s) => match s.parse::<f64>() {
            Ok(v) if v.is_finite() && v > 0.0 => v,
            _ => WALK_ACTION_SCALE,
        },
        None => WALK_ACTION_SCALE,
    }
}

pub fn walk_action_scale() -> f64 {
    parse_walk_action_scale(
        std::env::var("MINIDUCK_WALK_ACTION_SCALE").ok().as_deref(),
    )
}
```

调度在站稳态选 walk 网时，把**运行时**尺度塞进 `Decision`（`351`），不是写死常量：

```rust title="scheduler.rs · Sit::Up · sketch"
Sit::Up => (Net::Walk, *client, walk_action_scale()),
```

**映射：** 设计里的「自训用 1.0、公开仍 0.9」→ 环境变量覆盖 → `Sit::Up` 每拍读 `walk_action_scale()`。

## 验收

权重须已在 `tutor/artifacts/m09d/policy_export.onnx`（M9d 拷回）。数字门禁在**容器**（`/opt/sim-venv`）；肉眼可先在宿主机看我们的栈。官方 viewer 对照一句带过：`bash impl/scripts/m9d_infer_local.sh`（M9d 主路径，不证明本课的 daemon 接线）。

在仓库根 `microduck-from-scratch/` 操作。

### A. 宿主机可视化（肉眼，可选先做）

二进制若尚未编出，先在容器 `cargo build --bins`（见 B 步骤 2）。开三个终端。宿主机 `.venv` 见 `tutor/dev-container.md`。

终端 A（仓库根）：

```sh
.venv/bin/python impl/sim/duck_body.py --viewer --port 7810
```

应弹出 MuJoCo；默认 handmade BAM。daemon 未连时物理冻结。

终端 B（`impl/`）：

```sh
cd impl
```

```sh
env MINIDUCK_POLICY=../tutor/artifacts/m09d/policy_export.onnx \
  MINIDUCK_WALK_ACTION_SCALE=1.0 \
  ORT_DYLIB_PATH=third_party/onnxruntime/lib/libonnxruntime.so \
  ./target/debug/miniduckd --sim 127.0.0.1:7810
```

终端 C（仍在 `impl/`）：

```sh
./target/debug/mini-duckctl enable
```

等约 2 秒，窗口里应站稳。再：

```sh
./target/debug/mini-duckctl move 0.3 0 0 --secs 10
```

观察：约 10 秒内明显前移、不倒。看够后 Ctrl+C 停 B/A。

### B. 容器数字门禁（回归主路径）

compose 服务名 `rust`，容器内工作目录 `/work/impl`。先保证服务在跑：

```sh
docker compose -f impl/docker-compose.yml up -d
```

**1. 进容器 shell**（后面命令都在这个 shell；也可另开 `exec` 开第二、第三终端）

```sh
docker compose -f impl/docker-compose.yml exec rust bash
```

```sh
cd /work/impl
```

**2. 尺度单测 + 编二进制**

```sh
cargo test --lib walk_action_scale -- --nocapture
```

```sh
cargo build --bins
```

预期：`walk_action_scale_defaults_and_overrides` 通过。

**3. 起仿真**（本终端占住，或再开一个 `exec rust bash`）

```sh
MUJOCO_GL=disable /opt/sim-venv/bin/python sim/duck_body.py --port 7810
```

日志应出现 `scene=…scene_handmade.xml`。等约 2 秒确认进程还在。

**4. 另开容器终端，起 daemon**

```sh
docker compose -f impl/docker-compose.yml exec rust bash
```

```sh
cd /work/impl
```

```sh
env MINIDUCK_POLICY=/work/tutor/artifacts/m09d/policy_export.onnx \
  MINIDUCK_WALK_ACTION_SCALE=1.0 \
  ./target/debug/miniduckd --sim 127.0.0.1:7810
```

等约 2 秒；日志不应出现 `failed to load policy` / `policy unavailable`。

**5. 再开容器终端，发站 / 走**

```sh
docker compose -f impl/docker-compose.yml exec rust bash
```

```sh
cd /work/impl
```

```sh
./target/debug/mini-duckctl enable
```

等约 2～5 秒让它站稳（可用 `mini-duckctl state --every 1` 看 `skill`/`fallen`，期望 `walk` / 未倒）。再：

```sh
./target/debug/mini-duckctl move 0.3 0 0 --secs 10
```

CLI 是位置参数 `move <vx> <vy> <yaw> --secs …`，**不是** `--vx`。

预期（与跑证同门）：站稳躯干 z 约 0.12；`vx=0.3`×10s 前移至少 0.30 m（实测可到约 1.3 m）、走后仍 `skill=walk`、`fallen=false`。自动封装：`bash accept/m09e-b01.sh`（容器内 `/work/accept/…`）；跑证 `tutor/runs/m09e-b01.txt`。

## 坑

- CLI 写作 `move 0.3 0 0 --secs 10`，不是 `--vx 0.3`。
- 自训 ONNX 必须 `MINIDUCK_WALK_ACTION_SCALE=1.0`；仍用默认 0.9 等于把训练约定压掉 10%。
- 本课数字验收在容器 `/opt/sim-venv`，不是宿主机官方 viewer；官方 viewer 只作对照，不证明 `miniduckd` + handmade 已吃到自训权重。

## 自测

**1. 本课验收的主路径是什么？**

- A. 只跑 `bash impl/scripts/m9d_infer_local.sh`
- B. `MINIDUCK_POLICY` 指自训 ONNX + `miniduckd` + 默认 handmade BAM，`SCALE=1.0`
- C. 把默认 `policies/velstand.onnx` 改成自训文件名即可，尺度不用管
- D. 只看 M9d 的 headless JSON

<details><summary>答案</summary>

B。官方 viewer 是对照；本课要证明我们的 daemon + handmade 能吃自训权重。

</details>

**2. 为什么自训验收要把 walk scale 设成 1.0？**

- A. 随便设，和训练无关
- B. M9d Velocity / 官方 infer 按 1.0；默认 0.9 会压掉约 10% 动作
- C. 只有公开 `velstand` 才用 1.0
- D. 1.0 只影响 sitstand 槽

<details><summary>答案</summary>

B。`Sit::Up` 走 walk 槽；尺度必须对齐训练约定。

</details>

**3. `mini-duckctl` 正确发 `vx=0.3` 走 10 秒的命令是？**

- A. `move --vx 0.3 --secs 10`
- B. `move 0.3 0 0 --secs 10`
- C. `walk 0.3`
- D. `enable --vx 0.3`

<details><summary>答案</summary>

B。三个速度分量是位置参数，时长用 `--secs`。

</details>

**4. 不设 `MINIDUCK_WALK_ACTION_SCALE` 时，walk 尺度是？**

- A. 1.0
- B. 0.9
- C. 按 ONNX 文件名自动猜
- D. 0.0（不输出动作）

<details><summary>答案</summary>

B。常量默认 0.9，保护公开 `velstand`；自训部署显式覆盖。

</details>

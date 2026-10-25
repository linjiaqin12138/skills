---
template: sheet
title: M9b Bite 3：训练操作手册（第 24 章）
subtitle: 冒烟→长跑→export→manifest；resume 与 warm start 分叉
source: clone-from-scratch-tutor · 第 24 章
---

Bite 1–2 钉住契约与配方。本课学上云前就能做对的命令与决策。本机无 GPU，不真训；真训留 M9d。

```callout warn 本课不领读源码
验收是操作判断题，不是抽表。不打开 `m9b_ops_quiz.py` 当教材，也不领读官方 `train` / `export` 实现。看决策树与命令清单即可。
```

## 问题

公开权重能走，不代表你会训。典型故障发生在命令层，不在奖励公式：

- 一上来 `4096` envs 长跑，配置错误烧几小时才暴露。
- 跨任务只 `load-checkpoint`，课程计数跳到终态，第一步就「课完了」。
- 手转 `.pt` 成 ONNX，漏烘 obs normalizer；仿真 play 正常，真机/daemon 立刻漂。
- 本机无 CUDA 仍硬跑 Warp，或关掉检查假装可训。

本课消除的故障是：上云前分不清冒烟与长跑、resume 与 warm start、安全导出与手转 checkpoint。

## 设计

主路径：先便宜冒烟，再长跑，再走官方导出，最后过 manifest 形状门。

```flow TB
(冒烟 64×5) -> 长跑 4096
长跑 4096 -> *官方 export
官方 export -> manifest 门禁
```

冒烟惯例：`--env.scene.num-envs 64 --agent.max_iterations 5`。长跑才上 `4096`。无 GPU 时加 `--hf-jobs`，或等到 M9d；本课只练判断。

加载已有 checkpoint 时，先问「同一任务还是跨任务」：

```flow TB
(load checkpoint) -> {同一任务?}
同一任务? -> *resume + load-checkpoint: 是
同一任务? -> MICRODUCK_WARM_START=1: 否·跨任务
```

- **同一任务续训**：`--agent.load-checkpoint <ckpt> --agent.resume True`。两者都要；只写 `resume` 不够。
- **跨任务热启**：要权重/归一化/优化器，但课程从零。必须 `MICRODUCK_WARM_START=1`。只 load 别的任务 ckpt 且不设该变量时，`common_step_counter` 会跳到终态，课程一步到位。

导出与发布门禁（操作结论，不展开实现）：

| 步骤 | 正确做法 | 禁止 |
|---|---|---|
| 导出 | `scripts/export.py`（或同等官方路径），把 obs normalizer 烘进 ONNX | 手写 `torch.onnx.export`、直接拷 `.pt` 当部署产物 |
| manifest | 期望 `[1,61]→[1,14]`，本仓 **schema 2** | 与 daemon「v5」编号混为一谈；形状反写 |

## 操作决策

按场景选动作。入口文件只需知道存在：`train_cli`、`scripts/export.py`（或包内 `export.py`）、`publish/manifest.py`。

**1. 长跑前**

先 `64` envs × `5` iterations。不要跳过冒烟；也不要把冒烟缩成 `1×1` 当「过了」。冒烟要覆盖构建、步进、奖励项与导出路径的常见失败面。

**2. resume vs warm start**

| 意图 | 标志 |
|---|---|
| 同任务断点续训，课程步数接着走 | `--agent.load-checkpoint <path>` **且** `--agent.resume True` |
| 跨任务保留权重等，课程重来 | `MICRODUCK_WARM_START=1`（外加 load）；不要把 resume 当成跨任务解法 |

把 resume 与 warm start 当成同义词，是本课最高频误判。

**3. 部署导出**

只走官方 export。obs 归一化在训练里是开的：漏烘进图时，in-sim play 仍会套 normalizer，真机侧没有，漂移看起来像「调参问题」。

**4. 本机无 GPU**

本课正确做法是学判断，训练命令加 `--hf-jobs`，或等 M9d 上云机。关掉 CUDA 检查、指望 CPU Warp、或把 `num-envs` 改成 1 冒充有 GPU，都算做错。

## 验收

在仓库根执行：

```bash
bash accept/m09b-b03.sh
```

容器内：`docker exec -w /work miniduck-rust bash /work/accept/m09b-b03.sh`。

脚本注释中的预期：

| 项 | 预期 |
|---|---|
| A | `m9b_ops_quiz.py` 全部判断题通过（冒烟 64×5、resume≠warm start、export 烘 normalizer、manifest `[1,61]→[1,14]` / schema 2 等） |
| B | 官方 RL 树存在可指认入口：`train_cli` / `export.py` / `publish/manifest.py` |
| C | 不要求 `uv sync`、不 step env、不跑 GPU |
| D | 退出码 0 |

实测见 `tutor/runs/m09b-b03.txt`：七题全 PASS；入口存在性通过；`exit_code=0`。

## 自测

**1. 配置刚改完，准备训 Velocity。第一步该怎么开？**

- A. 直接 `--env.scene.num-envs 4096` 开长跑，省一次启动
- B. 先 `64` envs、`5` iterations 冒烟，通过后再上长跑
- C. 本机无 GPU 就改 `num-envs=1`，当冒烟已过
- D. 跳过 train，直接拷别人的 `.pt` 去真机

<details>
<summary>答案</summary>

B。冒烟惯例是 64×5，用来便宜抓配置错误。A 违反「长跑前先冒烟」；C 把缺 GPU 伪装成可训；D 跳过训练与导出门禁。
</details>

**2. 任务 A 训到一半宕了，还要在同一任务上接着课程走。应怎么写？**

- A. 只设 `MICRODUCK_WARM_START=1`
- B. 只加 `--agent.resume True`，让 runner 自己找最新 ckpt
- C. `--agent.load-checkpoint <ckpt>` 与 `--agent.resume True` 一起给
- D. 换任务 ID 再 train，不传任何 resume 相关标志

<details>
<summary>答案</summary>

C。同任务续训需要显式路径 + resume。A 是跨任务热启语义；B 缺 load 路径；D 会丢掉进度。
</details>

**3. 想用任务 A 的权重去开任务 B，但课程必须从零。若只 load A 的 checkpoint、不设 warm start，会发生什么？**

- A. 与 resume 等价，课程会正确从零开始
- B. `common_step_counter` 被恢复，课程可能一步跳到终态；跨任务应设 `MICRODUCK_WARM_START=1`
- C. 优化器状态自动清空，无需额外标志
- D. 只需再加 `--agent.resume True` 即可跨任务

<details>
<summary>答案</summary>

B。warm start ≠ resume：跨任务只 load 会把步进计数一并恢复。A/C/D 都把两条路径混成一条。
</details>

**4. 要把策略交给 daemon / 真机。哪条导出路径正确？**

- A. `torch.save` 后自己 `onnx.export`，跳过 normalizer
- B. 把 `.pt` 直接放到板子上当部署产物
- C. 走官方 `scripts/export.py`（或同等路径），把 obs normalizer 烘进 ONNX
- D. 任意 onnxruntime 转换工具转一下权重即可

<details>
<summary>答案</summary>

C。训练开了 obs 归一化，必须烘进图；手转或拷 checkpoint 会在部署侧静默漂。A/B/D 都是禁止项。
</details>

**5. publish/manifest 对本仓策略的形状与 schema，哪组对？**

- A. `[1,48]→[1,14]`，schema 5（与 daemon v5 同一编号）
- B. `[1,61]→[1,14]`，本仓 schema 2
- C. `[1,61]→[1,12]`，schema 1
- D. `[1,14]→[1,61]`，schema 2

<details>
<summary>答案</summary>

B。门禁是 61 维观测进、14 维动作出；本仓 manifest 为 schema 2，不要和 daemon 侧 v5 编号混用。A/C/D 形状或编号错误。
</details>

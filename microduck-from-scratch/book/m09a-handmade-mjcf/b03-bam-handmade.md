---
template: sheet
title: M9a Bite 3：BAM 驱动自建身体（第 20 章）
subtitle: 固定参数对账、20ms 目标队列、5ms 物理子步与站立行走验收
source: clone-from-scratch-tutor · 第 20 章
---

自建 `scene_handmade.xml` 已成为 BAM 默认场景。公开 `velstand` 策略能驱动它站立、直走和坐下起身。三次行走的 min=2.604m、median=2.605m；两次 sit 均回到 walk/fallen=false。

```callout ok 细分承重轮廓，摩擦回到 1.0
每侧鞋底用两个 box 拟合承重轮廓。碰撞保持零 mesh、零 STL，滑动摩擦与官方 CPU 回放一致。D70 已收敛；primitive 近似仍不等于逐接触点等价。
```

## 问题

Bite 1 的身体能编译、贴地、抗穿透，但旧行走验收默认加载官方场景。它能证明 BAM 驱动官方身体，不能证明同一套 Rust 控制栈能驱动自建身体。

切到 handmade 后，单 AABB 鞋底在摩擦 1.0 下能站却打滑。同一鞋底在 sit 中更直观地暴露接触失配。旧单 AABB 加过渡态摩擦 2.0 能坐稳，起身却在 rise 后约 0.3 秒分离。右腿 knee/ankle 先偏，walk 交接时已不稳，最终 fallen=true、gain=50。

旧单 AABB 加摩擦 1.0 甚至坐不下去，坐姿 z≈0.1157。摩擦 2.0 只补偿了“能坐下”，没有修正起身接触。真正没对齐的是鞋底承重轮廓，同时还要对账 BAM 参数与动作延迟。

## 设计

```sequence num
participants: miniduckd, duck_body.py, target_queue, BAM, scene_handmade.xml
miniduckd -> duck_body.py: read
duck_body.py --> miniduckd: sensors
miniduckd -> duck_body.py: write
duck_body.py -> target_queue: push
target_queue --> duck_body.py: pop
duck_body.py -> BAM: q_target
== 4 × 5ms ==
BAM -> scene_handmade.xml: torque
scene_handmade.xml --> BAM: state
```

一个控制拍是 20ms。`read` 只回传感数据，`write` 才推进 BAM 物理。目标先按控制拍排队，再写入 `q_target`。每拍由 BAM 计算力矩，MuJoCo 走四个 5ms 子步。

默认参数对齐官方 CPU `infer_policy.py` 回放。训练范围只记账，不在每次启动时随机采样。显式 CLI 参数用于单变量实验。

**来源：decisions/0001-handmade-bam-replay.md**

```text
## 上下文

M9a Bite 1 已交付 `scene_handmade.xml`，但 `duck_body.py` 的默认 BAM 场景仍是官方 `scene_groundcontact.xml`，所以既有行走验收没有验证自建身体。BAM 核心已经接入，训练侧的电压、压降与动作延迟范围尚未完整落账；官方 CPU `infer_policy.py` 则把这些随机范围折叠成固定值，并默认零延迟。

自建身体的单 box 足底与官方 mesh 接触形状不同。coder 实测官方滑动摩擦 1.0 下能站但行走打滑，1.5 单次约走 1.0m，2.0 约走 2.38m。2.0 超出官方训练足底摩擦随机范围 0.7..1.3，说明真正的偏差在接触几何，不能把调大系数当成对齐。

## 备选

1. 默认闭环切到自建身体；CPU 回放对齐 `infer_policy.py`：`vin=7.4`、`vin_drop_gain=0.1`、`kp_fw=200`、`delay=0`，提供显式参数与延迟入口；自建 box 足底用摩擦代理值 2.0 并登记偏差。
2. 默认闭环继续使用官方身体，只额外跑一份自建身体验收。
3. 每次启动随机采样训练范围 `vin=6.5..8.2`、压降 `0..0.2`、延迟 `3..6` 控制拍，把单鸭回放直接变成域随机化实验。
4. 把每只脚的鞋底碰撞拆成至少两个 box，逼近官方鞋底的承重轮廓；滑动摩擦保持官方 CPU 回放值 1.0，再用三次站立与行走验收。
5. 只在脚掌恢复官方 mesh 碰撞，其余部位保留 box。

## 决定

采用备选 4。默认 BAM 场景改为 `scene_handmade.xml`；`--no-bam` 诊断路径不动，显式 `--scene` 仍可回官方模型。常量表记录训练范围，默认行为对齐官方 CPU 回放。增加 `kp_fw/vin/vin_drop_gain/delay` 的显式配置，延迟作用于 20ms 控制拍目标队列，线上 15 槽总线契约不变。

足底碰撞继续零 mesh、零 STL，拆成多个 box 逼近鞋底轮廓，滑动摩擦回到 1.0。验收分别断言自建与官方摩擦、脚掌 primitive 数量、参数对账、lag=3 队列行为，以及 3 次全新进程的自建模型站立和 `vx=0.8` 行走。若细分几何仍无法在 1.0 下达标，回到备选 1 并保留 D70；本轮先不把 2.0 作为交付结果。

## 放弃的成本

备选 1 已经实测能过行走门槛，交付风险最低；放弃它的代价是必须重新调碰撞几何，可能暴露「公开权重在自建接触上仍不能走」这一更差的结论。

备选 2 能一直保留官方模型的已知好基线，切换身体的回归风险更低；放弃它的代价是必须用 3 次闭环和既有 M9a 结构验收证明自建身体，而不能靠默认路径继续避开差异。

备选 3 能在每次运行时自动覆盖训练扰动，更容易暴露鲁棒性问题；放弃默认随机化后，单次结果更可复现，但电压与延迟覆盖要由显式实验逐一执行，不能声称一次验收覆盖了训练分布。

备选 5 直接复用官方鞋底，接触形状最准，调参量最少；放弃它的代价是要用多个 box 手工逼近轮廓，无法逐点复制曲面，验收必须同时守住零 STL 与行走门槛。

## 失效边界

若细分 box 在官方摩擦范围内仍无法行走，说明单个 primitive 的形状误差不是主要问题，必须补接触点/法向力/剪切力测量后再裁决。若自训策略使用不同鞋底或硬件构型，本决定中的几何拆分必须重算。若未来允许自建凸包 mesh，备选 5 可重开，但需先验证 mesh-plane 大冲击不会复现 D26。
```

## 代码领读

BAM 常量把三组口径放在一起：电机与模型、CPU 回放默认值、训练随机范围。`BAM_TIMESTEP=0.005` 决定每拍四子步。鞋底几何与摩擦写在 MJCF，不混进执行器参数表。

```python src=impl/sim/duck_body.py lines=66-87 hl=73-83
```

| CLI | 作用 | 默认 | 训练范围 |
|---|---|---:|---:|
| `--kp-fw` | 固件 P 环增益 | 200 | 固定训练真值 200 |
| `--vin` | 供电电压 | 7.4V | 6.5..8.2V |
| `--vin-drop-gain` | 总力矩引起压降的系数 | 0.1 | 0..0.2 |
| `--delay` | 目标延迟，单位为控制拍 | 0 | 3..6 拍 |

这些范围不是默认随机化开关。默认实例使用固定值，实验时逐个显式覆盖。`vin` 还会参与力矩上限计算，所以改电压不只是改一个显示字段。

编译时把 XML 的 `position` 执行器改成 `motor`。力矩上限按 `vin × kt / R` 重算。关节旧阻尼与摩擦损失清零，BAM 随后按自己的电机模型接管。物理步长在这里再次固定为 5ms。

```python src=impl/sim/duck_body.py lines=113-146 hl=124-144
```

初始化先读 `STAND`，再把 BAM 初始躯干高度设为 0.125m。`target_queue` 预填站姿，延迟期间不会先把关节目标放空。无延迟时队列为空，新目标当拍生效。

```python src=impl/sim/duck_body.py lines=172-196 hl=183-193
```

`step_tick` 每拍循环四次。每次先让 `bam_ctrl.update()` 根据位置目标和电机状态算力矩，再调用 `mj_step`。因此延迟排队发生在 20ms 层，电机与接触积分发生在 5ms 层。

```python src=impl/sim/duck_body.py lines=256-279 hl=274-279
```

`write_targets` 仍接收 15 槽总线序，嘴位被移除后得到 14 个关节目标。`delay=3` 时，前三拍弹出预填站姿，第四拍才弹出第一拍新目标。队列只延迟控制目标，不把线上协议改成另一条契约。

```python src=impl/sim/duck_body.py lines=321-347 hl=338-344
```

`--delay` 模拟命令滞后，不是模拟 TCP 慢。真实链路里，策略输出要经过 IPC、舵机固件和电机响应；训练侧把这份滞后随机放在 3 到 6 个控制拍，也就是 60 到 120ms。这样策略不能只学会「命令一发出，关节立刻执行」。官方 CPU 回放默认零延迟，本实现也默认零延迟；这个参数是留给单变量破坏实验的。

| 20ms 控制拍 | 新写入 | 实际执行 |
|---:|---|---|
| 0 | T0 | STAND |
| 1 | T1 | STAND |
| 2 | T2 | STAND |
| 3 | T3 | T0 |

BAM 的电机方程仍会在每个 5ms 子步里继续算。两者叠加后，才接近「命令先迟到，电机再追目标」的真实链路。

每只脚有两个碰撞 box。薄承重底层拟合官方 `sole` 的平地接触带，主体上层保留 AABB。MJCF 的 `size` 是半长，所以底层全尺寸约为 3.0mm 厚、11.83mm 宽、39.22mm 长。右足结构相同，见 `impl/sim/assets/duck_handmade.xml:243-248`。

```xml src=impl/sim/assets/duck_handmade.xml lines=102-118 hl=112-118
```

原项目不是两个 geom。它在 `impl/sim/assets/robot_groundcontact.xml:166` 和 `:325` 直接用一个鞋底 mesh 做碰撞。训练侧再把 `*_collision` 选进 FULL_COLLISION，并给两脚设置 `condim=3`、`priority=1`、`friction=1.0`。mesh geom 仍是一个 geom，求解器会从三角面生成接触点。

我们不用 mesh 有两个原因。第一，官方 STL 是 CC BY-NC-SA，本章要保住零受限资产的碰撞层。第二，mesh-plane 接触曾造成 D26 穿透；box-plane 的角点约束更稳定。两个 box 是我们的 primitive 替代，不是原版设计。背景见[接触求解与穿透](../concepts/contact-penetration.html)。

自己生成新 mesh 可以绕开许可，但仍要重新验证大冲击稳定性。本章优先使用 box，避免同时引入新几何和新接触风险。

两层不是冗余，而是分工。站立时接触由薄层主导；深蹲时实测接触发生在主体上层和腿盒，薄层不参与。单层消融实验说明两层都必要：

| 只保留 | 实测结果 |
|---|---|
| 薄承重底层 | 能站能坐起，行走 10s 倒溜 0.176m；坐姿视觉鞋底扎地约 4mm |
| 主体上层 | 站立矮约 3mm，坐不下去；行走 10s 倒溜 0.193m |
| 两层都在 | 站立、行走、sit 都通过 |

行走中主体上层具体在哪些拍接管接触，本轮没有逐拍记录。已知结论是缺它会失败，完整机制仍属推断。

四个脚掌 geom 的滑动摩擦都为 1.0。初版单 AABB 的问题出在承重轮廓，不在摩擦系数。细分几何后，摩擦无需补偿即可达标。

几何探针给出了失配位置。旧单盒底面 z=0.65mm，比官方 `sole` 网格最低点 2.82mm 低约 2.2mm。旧盒还用整个 55×44mm AABB 底面接触。当前薄底层底面 z=2.38mm，更贴近官方承重带。

| 鞋底碰撞 | 滑动摩擦 | 单次探测或最终验收 | 行走结果 |
|---|---:|---|---|
| 单 AABB | 1.0 | 单次探测 | 能站但打滑 |
| 单 AABB | 1.5 | 单次探测 | 约 1.0m |
| 单 AABB | 2.0 | 单次探测 | 约 2.38m |
| 薄底层 + 主体上层 | 1.0 | verifier 三次记录 | min=2.604m；median=2.605m |

单次探测只用于排查。最终结论只认 verifier 的三次记录。这里没有逐接触点力测量，不能声称 primitive 与 mesh 接触逐点等价。

场景负责组合身体、地板、陀螺仪和站姿。BAM 编译分支接受这个场景，再把执行器与步长改成运行时所需形式。

```xml src=impl/sim/assets/scene_handmade.xml lines=31-44 hl=33-42
```

CLI 默认值传入 `DuckBody`。BAM 开启且未显式给 `--scene` 时，默认选择 `scene_handmade.xml`。`--no-bam` 仍走旧 `scene.xml`，显式 `--scene` 仍可固定官方场景。

```python src=impl/sim/duck_body.py lines=480-509 hl=488-509
```

宿主 `.venv` 只用于模型目检。下面的 viewer 不启动 Rust 控制栈，也不代替容器验收：

```bash
.venv/bin/python -m mujoco.viewer --mjcf=impl/sim/assets/scene_handmade.xml
```

## 验收

在仓库根执行容器命令：

```bash
docker compose -p microduck-from-scratch -f impl/docker-compose.yml exec -T rust bash /work/accept/m09a-b03.sh
```

脚本注释给出四组预期，见 `accept/m09a-b03.sh:7-16`：

| 组 | 验收意图 |
|---|---|
| A | BAM 对账；每脚至少两个 box；两侧摩擦均为 1.0 |
| B | 参数可覆盖；`delay=3` 时第四拍执行第一拍目标 |
| C | 默认 handmade；三次全新进程站立并直走达标 |
| D | 两次全新进程完成 sit_toggle 坐下与起身 |

行走实测见 `tutor/runs/m09a-b03.txt:37-53`。三次都先满足站立条件，再以 `vx=0.8` 走 10 秒。

| 样本 | 站立 z | dx | 走后 z | skill / fallen |
|---|---:|---:|---:|---|
| 1 | 0.117035m | 2.605573m | 0.124522m | walk / false |
| 2 | 0.117035m | 2.604251m | 0.124044m | walk / false |
| 3 | 0.117035m | 2.604783m | 0.124044m | walk / false |

位移 min=2.604m、median=2.605m，超过 2.0m 门槛。三次是同一确定性配置的重复，不是随机独立试验。它们不支撑 p 值、置信区间或统计成功率。

D 组实测见 `tutor/runs/m09a-b03.txt:54-65`：

| 样本 | 坐姿 z | 坐姿状态 | 起身 z | 起身状态 |
|---|---:|---|---:|---|
| 1 | 0.0644237m | sit / false | 0.1159861m | walk / false |
| 2 | 0.0644237m | sit / false | 0.1159861m | walk / false |

D 组只断言 2.5 秒后的坐姿快照，以及起身 3 秒后的状态。它不是全程窗口断言，连续坐姿窗口仍未覆盖。

最终 verifier 结论为**通过**，见 `tutor/runs/m09a-b03.txt:285-294`：

- 含 D 组的主验收退出码 0。
- Rust 测试 102 passed，0 failed。
- `m09a-b01` 回归通过。
- `m08-b06` 固定官方场景后通过。

当前覆盖仍有明确边界：

- handmade 的 roulade 未验收。
- D 组未记录连续坐姿窗口。
- 非直走、横移、转向、扰动与随机种子未覆盖。
- 训练范围的速度、地形和初始扰动未覆盖。
- 所有结果都来自仿真，真机未覆盖。

## 与原版差异

收敛 [D70](../../spec/deviations.md)。每侧鞋底细分为两个 box，摩擦回到官方 CPU 回放值 1.0。残余是 primitive 近似，不声称与 mesh 接触逐点等价。

D4 不在本 Bite 收敛。D26 已由 Bite 1 收敛，本课没有新结论。

## 坑

**能站不代表能走。** 单 AABB 在摩擦 1.0 下能站却打滑。摩擦 1.5 单次约走 1.0m，2.0 约走 2.38m。细分承重轮廓后，摩擦 1.0 也能达标。偏差源头是承重轮廓，不是摩擦系数。

**坐姿也依赖碰撞几何。** 禁用腿、躯干或头的碰撞盒会穿地，说明坐姿确实靠腿部盒承重。三个变体间唯一改动是鞋底，失配因此锁定在脚。

【合理推断】接触不匹配使策略反馈出分布，起身因此失稳。分离点、关节先后和探针数字是实测；这条机制链不是直接测力结论。

**单次探测不能代替最终验收。** 摩擦扫描只用于定位故障。交付结论使用 verifier 的三次行走和两次 sit 记录，不拿临时尝试拼成功率。

**固定默认值不等于覆盖训练分布。** `--vin`、`--vin-drop-gain` 和 `--delay` 可做单变量实验。要评估鲁棒性，还需独立随机采样路径与分布统计。

## 自测

```html
<details>
<summary><b>1.</b> 你设置 <code>--delay 3</code>，随后连续写入四个不同目标。第四拍应执行哪个？（A）第四个目标　（B）第一个目标　（C）四个目标的均值　（D）始终执行 STAND</summary>
<p><b>答案：B。</b>队列预填三个 STAND。前三拍依次弹出它们，第四拍才弹出第一拍目标。延迟单位是 20ms 控制拍，不是 5ms 子步。</p>
</details>
<details>
<summary><b>2.</b> 你把 <code>--vin</code> 从 7.4 改成 6.8。除供电电压外，哪项也必须随之变化？（A）力矩上限 <code>forcerange</code>　（B）关节数　（C）总线槽数　（D）控制拍长度</summary>
<p><b>答案：A。</b>编译分支按 <code>vin × kt / R</code> 重算力矩上下限。验收用 vin=6.8 实测 ±0.885288 N·m。关节数、15 槽总线和 20ms 控制拍不变。</p>
</details>
<details>
<summary><b>3.</b> 三次直走都约 2.605m。哪项结论可以成立？（A）真实硬件成功率不小于 99%　（B）随机扰动下也稳定　（C）当前固定配置的集成与直走门槛通过　（D）handmade 的 roulade 已通过</summary>
<p><b>答案：C。</b>三次是确定性重复，只覆盖当前配置。它们不覆盖随机种子、扰动、真机或 roulade，也不能推出统计成功率。</p>
</details>
<details>
<summary><b>4.</b> M8 官方身体回归要在默认 handmade 之后继续有效，脚本应怎样启动仿真？（A）依赖当前默认场景　（B）显式固定 <code>scene_groundcontact.xml</code>　（C）把足底摩擦改成 2.0　（D）跳过 sit 与 roulade</summary>
<p><b>答案：B。</b>验收脚本必须声明并检查自己的场景归属。显式固定官方场景后，M8 官方回归全绿。handmade 的 roulade 仍未验收。</p>
</details>
<details>
<summary><b>5.</b> 单 AABB 在摩擦 1.0 下能站却打滑。哪项修改最符合本次证据？（A）把摩擦永久改为 2.0　（B）细分鞋底承重轮廓并保持摩擦 1.0　（C）增加控制拍延迟　（D）把碰撞换成视觉 STL</summary>
<p><b>答案：B。</b>摩擦扫描只是排查手段。细分几何后在官方摩擦值下达标，说明偏差源头是承重轮廓。视觉 STL 不参与碰撞。</p>
</details>
```

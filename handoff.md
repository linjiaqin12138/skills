# Handoff — microduck M8 Bite 8（2026-10-14，官方对照之后）

工作区：`/home/jqlin/Project/skills`。项目在 `microduck-from-scratch/`。续传先读 `microduck-from-scratch/docs/STATE.md`。

## current status

M8 Bite 8 代码已改，验收未全过。教程未写。D68 不标收敛。不装回 `push(0.30)`。

用户已选「先做官方对照」。`infer_policy.py` 走不动（Q80）。原项目自己的 `robotd --sim` + `duck-body` 也走不动（Q82）：坐姿起身到 z=0.116，随后 `vx=0.15` × 8 s，dx=0.006 m。

BAM 路径与官方同一步：`read` 不步进，`write` 先写 `q_target`，再 `bam_ctrl.update()` + `mj_step`。非 BAM 仍是读时步进。`--sim` 且策略已加载时第一拍 Driving。假总线仍 Held→RampUp。

能站、起身接得住、滚翻结束能直立。站稳之后给速度命令走不动（dx≈0.005 m，z≈0.116）。官方脚本同一条件 dx=0.0052 m。

## official comparison（Q80）

直连 GitHub，不用 `127.0.0.1:7890`。WSL 里该端口没在听；透明代理下 `curl https://www.google.com` 返回 200。容器里 PyPI 也直连可达。

脚本：宿主机 `/tmp/infer_policy.py`（microduck_rl `develop`，1820 行），已 `docker cp` 到容器 `/tmp/infer_policy.py`。无头驱动在宿主机 `/tmp/official_stand_then_walk.py`，容器内同名。这两份都不在仓库里。

场景：官方默认 `scene.xml` **include 的是 `robot_groundcontact.xml`**，不是 `robot_walk.xml`。与 `sim/assets/robot_groundcontact.xml` 逐字节相同。我们的 `sim/assets/scene.xml` 包的是 `robot_walk`，不要把它当成官方默认场景。容器里的拼装目录是 `/tmp/md-official/`（scene + robot xml + `assets` 软链到 `/work/sim/assets/assets`）。

参数：`policies/velstand.onnx`，`--action-scale 0.9`，`--new-cmd-obs`，BAM M6，`kp_fw=200`，`vin=7.4`，`vin_drop_gain=0.1`，`vin_min=6`，无电流限制，timestep 0.005，decimation 4，投影重力（CLI 默认，没有 `--raw-accelerometer`），初始躯干 z=0.125，无 stand 槽，无 push。

- 零命令站 2 s，再 `vx=0.15` × 10 s：dx=0.0052 m，z 0.1166→0.1158，命令到来时平均 |关节速度|=0.00257，动作标准差均值 0.00289，`obs[48]=0.15`。
- 从第一拍就给 `vx=0.15` × 10 s：前 0.5 s 平均 |关节速度|峰值 0.31978，dx=0.0079 m，z 0.125→0.1155。落地残留存在，步态没起来。

仿真 venv 里为这次装了 `onnxruntime==1.31.0`（`/opt/sim-venv`）。重建容器会丢。

## files changed

- `microduck-from-scratch/sim/duck_body.py`：BAM 改为先写 `q_target` 再步进。
- `microduck-from-scratch/docs/questions.md`：Q80–Q82。
- `microduck-from-scratch/docs/STATE.md`、`docs/deviations.md`：行走仍红，官方对照同样红。
- `microduck-from-scratch/docs/verify-official-walk.md`：换电脑时用官方窗口和 ↑ 键验证。
- `microduck-from-scratch/.gitignore`：忽略 `.tmp-official/`（本地克隆的 microduck_rl，不入库）。

`docs/milestones/m8-convergence/arch.d2` 仍是 Bite 6 文案（含已删除的 `push(0.30)` / `skip_bringup_ramp`）。

## passing / failing

与上一份交接相同，这轮没有重跑 `accept-m8-bam.sh` / `cargo test`。

- 站立、sit 起身、roulade 结束直立：上一轮已过。
- 行走：`vx=0.15` × 10 s，前移 0.005 m，门槛 0.4 m。官方对照同量级。

## exact next step

用户要换时间自己看官方窗口。步骤在 `microduck-from-scratch/docs/verify-official-walk.md`：系统终端启动 `infer_policy.py`，先看它站直，再在该终端按一次 ↑。不要写测量脚本，不要代发 `robot.move`。

用户看完并点头之前：不写 Bite 8 教程，不改门槛，不改 `SIT_RISE_TICKS`，不把 D68 标成收敛，不装回 `push(0.30)`。点头之后才写教程（起身收敛，行走不动点仍开放）并按当前代码重画架构图，删掉 Bite 6 的 push / skip ramp。

## what not to revisit

- 不要把冻结、nudge、`push(0.30)` 装回去。官方无推力路径也走不动，这一脚不是对齐。
- 不要再追「落地残留能启动步态」。官方从落下就给 `vx=0.15`，关节速度峰值 0.32，10 s 仍只有 8 mm。
- 不要把我们的 `scene.xml`（robot_walk）当成官方默认场景。官方默认 include 的是 groundcontact。
- 不要为了走路去接 `alpha_stand.onnx`。
- 不要改回「读时步进」。起身能接住依赖写后步进。
- 不要放宽 `accept-m8-bam.sh` 的 0.4 m，不要改起身 1.0 s 窗口。
- 不要改 Rust 的 read → compute → write 顺序。
- 不要动 D35、D65。

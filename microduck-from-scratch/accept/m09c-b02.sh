#!/usr/bin/env bash
# M9c Bite 2：相对 odometry（双足 FK + ALPHA16 锚点）。
#
# 在仓库根执行：bash accept/m09c-b02.sh
# 容器用法：docker compose -f impl/docker-compose.yml exec rust bash /work/accept/m09c-b02.sh
#
# 预期：
#   A. 全部默认 Rust 单测通过。覆盖：ALPHA16 数量/镜像/边界；静止原点与高度；yaw；
#      嘴不影响；单拍候选不换锚、同脚两拍确认；换锚连续；新鲜/IMU-ready 门控；
#      滑行不推进估计或 pending；disable、Limp/Posing、scheduler reset 不清零；
#      robot.state 仍无 odom（D50 不提前）；400 拍参考轨迹与上游逐位一致。
#   B. 实时集成测试 impl/tests/odometry_live.rs 使用当前 control::spawn + handmade BAM sim，
#      不走 RPC 暴露估计。3 次全新进程 vx=0.8 行走 10s + 3 次 vyaw=2.0 转身 6s。
#      每次记录真值/估计位移、终点水平误差、最大单拍跳动、锚点切换、实际 yaw 变化和 fallen。
#   C. 行走每次真实前移 >=2.0m 且结束 upright/fallen=false；转身每次 |实际 yaw| >=0.5rad。
#      每次 odometry 终点水平误差 <=0.12m，6 次中位数 <=0.07m；最大单拍跳动 <0.02m；
#      yaw 与同一 IMU 真值的环绕误差 <1e-6rad。打印全部样本、median、max，不以单次成功代替分布证据。
#      门槛来自修正后实测：前行误差 <=0.109811m，vyaw=2.0 两次转身 >3rad、误差约 0.024m；
#      合成六样本 median 约 0.064m。它们是 handmade 本地回归门，不是上游精度承诺。
#   D. accept/m08-b01.sh 全回归通过，M9c Bite 1 的 RPC/状态机行为不回退。
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO/impl"

SIM_PYTHON="${SIM_PYTHON:-/opt/sim-venv/bin/python}"
[ -x "$SIM_PYTHON" ] || { echo "FAIL: 缺 $SIM_PYTHON；请在容器内验收" >&2; exit 1; }
[ -f policies/velstand.onnx ] || { echo "FAIL: 缺 velstand.onnx" >&2; exit 1; }
[ -f third_party/onnxruntime/lib/libonnxruntime.so ] || { echo "FAIL: 缺 ONNX Runtime" >&2; exit 1; }

export SIM_PYTHON
export ORT_DYLIB_PATH="${ORT_DYLIB_PATH:-$REPO/impl/third_party/onnxruntime/lib/libonnxruntime.so}"

echo "===== A. 默认 Rust 单测（live 测试默认 ignored） ====="
cargo test

echo "===== B/C. handmade sim 实时 odometry：3 行走 + 3 转身 ====="
cargo test --test odometry_live -- --ignored --nocapture

echo "===== D. M8 RPC/状态机回归 ====="
bash "$REPO/accept/m08-b01.sh"

echo "M9c Bite 2 验收全部通过"

#!/usr/bin/env python3
"""M9b Bite 3：train / export / manifest 操作判断题（无 GPU 真训）。

内置情景→应选操作断言；静态确认 MICRODUCK_RL_ROOT 下入口文件存在。
不领读官方源码、不抽 cfg 奖励权重。

用法（由 accept/m09b-b03.sh 设置 MICRODUCK_RL_ROOT）：
    MICRODUCK_RL_ROOT=... python impl/scripts/m9b_ops_quiz.py
"""

from __future__ import annotations

import os
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# 情景 → 选项；EXPECTED 是标准答案键。
# 证据：tutor/recon-microduck-rl.md；rl:AGENTS.md / README；docs/velstand_policy.md
# ---------------------------------------------------------------------------

QUIZ: list[dict] = [
    {
        "id": "smoke_64x5",
        "prompt": (
            "长跑前要先冒烟。应选哪组参数？"
            "（不是一上来 4096 envs 长跑）"
        ),
        "choices": {
            "A": "--env.scene.num-envs 4096 --agent.max_iterations 50000",
            "B": "--env.scene.num-envs 64 --agent.max_iterations 5",
            "C": "--env.scene.num-envs 1 --agent.max_iterations 1",
            "D": "跳过冒烟，直接 4096×长跑",
        },
        "expected": "B",
        # 内容锚点：改错选项文案也会被发现
        "must_contain": ("64", "5"),
    },
    {
        "id": "resume_same_task",
        "prompt": (
            "同一任务断点续训、要接着课程步数往下走。应选？"
        ),
        "choices": {
            "A": "只设 MICRODUCK_WARM_START=1，不带 --agent.resume",
            "B": "--agent.load-checkpoint <ckpt> --agent.resume True",
            "C": "手改 checkpoint 里的 common_step_counter 再 load",
            "D": "换任务 ID 再 train，不传 resume",
        },
        "expected": "B",
        "must_contain": ("--agent.resume",),
    },
    {
        "id": "warm_start_cross_task",
        "prompt": (
            "跨任务热启动（保留权重/归一化/优化器，但课程要从头）。"
            "若只 load 别的任务 checkpoint 且不设 MICRODUCK_WARM_START=1，"
            "会怎样？正确应对是？"
        ),
        "choices": {
            "A": "没事，resume 与 warm start 等价",
            "B": (
                "common_step_counter 会跳到终态、课程一步到位；"
                "跨任务必须 MICRODUCK_WARM_START=1"
            ),
            "C": "只用 --agent.resume True 即可跨任务",
            "D": "删掉 wandb run 再训",
        },
        "expected": "B",
        "must_contain": ("MICRODUCK_WARM_START", "common_step_counter"),
    },
    {
        "id": "export_bake_normalizer",
        "prompt": "要把策略部署到真机/daemon。导出应怎么做？",
        "choices": {
            "A": "torch.save 后手写 torch.onnx.export，跳过 normalizer",
            "B": "直接把 .pt checkpoint 拷到板子当部署产物",
            "C": (
                "走 scripts/export.py（或同等官方导出路径），"
                "把 obs normalizer 烘进 ONNX；禁止手转 checkpoint"
            ),
            "D": "只用 onnxruntime 随便转一下权重即可",
        },
        "expected": "C",
        "must_contain": ("export.py", "normalizer"),
    },
    {
        "id": "manifest_shape_schema",
        "prompt": (
            "publish/manifest 门禁：本仓期望的输入输出形状与 schema 是？"
        ),
        "choices": {
            "A": "[1,48]→[1,14]，schema 5（daemon v5）",
            "B": "[1,61]→[1,12]，schema 1",
            "C": "[1,61]→[1,14]，本仓 schema 2",
            "D": "[1,14]→[1,61]，schema 2",
        },
        "expected": "C",
        "must_contain": ("[1,61]", "[1,14]", "schema 2"),
    },
    {
        "id": "no_gpu_hf_jobs",
        "prompt": (
            "本机无 GPU，本课（M9b）也不能假装本机可训。正确做法？"
        ),
        "choices": {
            "A": "照常 uv run train，指望 CPU 跑 Warp",
            "B": "改代码关掉 CUDA 检查后本机长跑",
            "C": (
                "训练命令加 --hf-jobs，或等到有云机/GPU 的 M9d；"
                "本课只学操作判断"
            ),
            "D": "把 num-envs 改成 1 就当有 GPU",
        },
        "expected": "C",
        "must_contain": ("--hf-jobs", "M9d"),
    },
    {
        "id": "resume_needs_checkpoint_path",
        "prompt": (
            "断点续训时，除了 --agent.resume True，还必须带什么？"
        ),
        "choices": {
            "A": "只加 --agent.resume True 就够，runner 会自动找最新 ckpt",
            "B": (
                "--agent.load-checkpoint <checkpoint 路径>，"
                "并带 --agent.resume True"
            ),
            "C": "只设 MICRODUCK_WARM_START=1",
            "D": "只传 --wandb-run-path",
        },
        "expected": "B",
        "must_contain": ("load-checkpoint",),
    },
]

# 脚本内「作答」表——改错对应题会 FAIL（与 expected 对账）。
ANSWERS: dict[str, str] = {
    "smoke_64x5": "B",
    "resume_same_task": "B",
    "warm_start_cross_task": "B",
    "export_bake_normalizer": "C",
    "manifest_shape_schema": "C",
    "no_gpu_hf_jobs": "C",
    "resume_needs_checkpoint_path": "B",
}


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def rl_root() -> Path:
    root = os.environ.get("MICRODUCK_RL_ROOT")
    if not root:
        fail("环境变量 MICRODUCK_RL_ROOT 未设置（应指向 microduck_rl 根）")
    p = Path(root)
    if not p.is_dir():
        fail(f"MICRODUCK_RL_ROOT 不是目录：{p}")
    return p


def check_entry_files(root: Path) -> None:
    """静态确认官方入口文件存在（不读源码内容）。"""
    train_cli = root / "src" / "mjlab_microduck" / "train_cli.py"
    if not train_cli.is_file():
        fail(f"缺入口：{train_cli}")

    export_scripts = root / "scripts" / "export.py"
    export_pkg = root / "src" / "mjlab_microduck" / "export.py"
    if not export_scripts.is_file() and not export_pkg.is_file():
        fail(
            "缺 export 入口：需要 scripts/export.py "
            "或 src/mjlab_microduck/export.py"
        )

    manifest = root / "src" / "mjlab_microduck" / "publish" / "manifest.py"
    if not manifest.is_file():
        fail(f"缺入口：{manifest}")


def run_quiz() -> None:
    if len(QUIZ) < 6:
        fail(f"判断题不足 6 题（当前 {len(QUIZ)}）")

    for q in QUIZ:
        qid = q["id"]
        prompt = q["prompt"]
        choices: dict[str, str] = q["choices"]
        expected: str = q["expected"]
        must_contain: tuple[str, ...] = q["must_contain"]

        print(f"[{qid}] {prompt}")
        for key, text in choices.items():
            mark = "←" if key == expected else " "
            print(f"  ({mark}) {key}. {text}")

        if expected not in choices:
            fail(f"题 {qid}：expected={expected!r} 不在选项中")

        if qid not in ANSWERS:
            fail(f"题 {qid}：ANSWERS 未作答")

        got = ANSWERS[qid]
        if got != expected:
            fail(f"题 {qid} 答错：选了 {got}，应为 {expected}")

        chosen_text = choices[got]
        for needle in must_contain:
            if needle not in chosen_text:
                fail(
                    f"题 {qid}：所选操作文案缺关键内容 {needle!r} "
                    f"（选项 {got}）"
                )

        print(f"  → 判定：选 {got} PASS\n")


def main() -> None:
    root = rl_root()
    check_entry_files(root)
    run_quiz()
    print("PASS")


if __name__ == "__main__":
    main()

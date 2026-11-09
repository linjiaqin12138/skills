#!/usr/bin/env python3
import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description="OpenMontage 短视频 pipeline 调度")
    parser.add_argument("--pipeline", default="short-drama", choices=("short-drama", "cinematic", "animation", "animated-explainer", "documentary-montage"))
    parser.add_argument("--project", required=True)
    parser.add_argument("--shot")
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--jobs", type=int, default=3)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--no-compose", action="store_true")
    parser.add_argument("--bgm")
    args = parser.parse_args()
    if args.pipeline != "short-drama":
        print(f"路由 {args.pipeline}：请读取 vendor/OpenMontage/pipeline_defs/{args.pipeline}.yaml 及其 director skills 执行；本地生产脚本目前仅实现 short-drama。")
        return 2
    command = [
        sys.executable,
        str(ROOT / "skills" / "short-drama" / "scripts" / "produce_episode.py"),
        "--project", args.project,
        "--jobs", str(args.jobs),
    ]
    if args.shot:
        command += ["--shot", args.shot]
    if args.force:
        command.append("--force")
    if args.dry_run:
        command.append("--dry-run")
    if args.no_compose:
        command.append("--no-compose")
    if args.bgm:
        command += ["--bgm", args.bgm]
    return subprocess.run(command).returncode


if __name__ == "__main__":
    sys.exit(main())
